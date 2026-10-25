#![no_std]
#![no_main]
#![deny(
	clippy::mem_forget,
	reason = "mem::forget is not safe with esp_hal types that hold buffers during a transfer"
)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

mod config;
mod error;
mod services;

use defmt::{Debug2Format, info, warn};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{
	gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
	timer::timg::TimerGroup,
	uart::{self, Uart},
};
use panic_rtt_target as _;
use presence_core::{
	handlers::presence::next_state,
	models::{advertisement::Advertisement, presence::PresenceState},
};

pub use error::{Error, Result};
use services::{
	ble::Ble,
	button,
	led::{self, Mode},
	sensor::{Event, Sensor},
};

// app descriptor the esp-idf bootloader expects
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(clippy::large_stack_frames, reason = "the main future owns the sensor driver (about 2 KB)")]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
	rtt_target::rtt_init_defmt!();

	let p = esp_hal::init(esp_hal::Config::default());
	// the BLE controller allocates
	esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

	let timg0 = TimerGroup::new(p.TIMG0);
	esp_rtos::start(timg0.timer0, p.FROM_CPU_INTR0);

	info!("presence booted");

	// sensor OT1 (its TX) -> GPIO21, sensor RX -> GPIO20. OT2 (GPIO3) is for
	// the battery version's wake-up and unused here.
	let uart = Uart::new(p.UART1, uart::Config::default().with_baudrate(ld2410s::BAUD_RATE))
		.expect("uart config")
		.with_rx(p.GPIO21)
		.with_tx(p.GPIO20)
		.into_async();
	let mut sensor = Sensor::new(uart);
	// the BOOT button
	let boot = Input::new(p.GPIO9, InputConfig::default().with_pull(Pull::Up));
	let hold = Duration::from_secs(config::CALIBRATE_HOLD_SECS);
	spawner.spawn(button::watch(boot, hold).expect("button task already running"));
	// the blue LED, active low: starts off
	let blue = Output::new(p.GPIO8, Level::High, OutputConfig::default());
	spawner.spawn(led::run(blue).expect("led task already running"));

	// Until the first reading, advertise "clear": a fresh boot hasn't seen anyone.
	let mut state = PresenceState {
		occupied: false,
		distance_cm: 0,
	};
	let mut packet_id: u8 = 0;
	let ble = Ble::start(p.BT, &Advertisement::bthome(&state, packet_id, config::DEVICE_NAME))
		.await
		.expect("ble start");
	info!("advertising as {}", config::DEVICE_NAME);

	while let Err(e) = sensor.init().await {
		warn!("sensor init: {}", Debug2Format(&e));
		Timer::after_secs(1).await;
	}
	info!("sensor ready");

	// Some while a scan runs: when to give up waiting for its 100% frame.
	// The LED shows the scan until then, not presence.
	let mut scan_deadline: Option<Instant> = None;
	loop {
		match select(sensor.next_event(), button::LONG_PRESS.wait()).await {
			Either::First(Event::Reading(reading)) => {
				if scan_deadline.is_some_and(|d| Instant::now() >= d) {
					warn!("no 100% from the scan; showing presence again");
					scan_deadline = None;
				}
				if let Some(next) = next_state(Some(state), reading, config::DISTANCE_STEP_CM) {
					state = next;
					packet_id = packet_id.wrapping_add(1);
					info!("occupied={} distance={}cm", state.occupied, state.distance_cm);
					let adv = Advertisement::bthome(&state, packet_id, config::DEVICE_NAME);
					if let Err(e) = ble.advertise(&adv).await {
						warn!("advertise: {}", Debug2Format(&e));
					}
				}
				if scan_deadline.is_none() {
					led::set(if state.occupied {
						Mode::On
					} else {
						Mode::Off
					});
				}
			},
			Either::First(Event::Calibration(progress)) => {
				info!("calibration {}%", progress);
				if progress >= 100 {
					scan_deadline = None;
					led::set(if state.occupied {
						Mode::On
					} else {
						Mode::Off
					});
				}
			},
			Either::Second(()) => {
				info!("calibrating in {}s: leave the room", config::CALIBRATE_DELAY_SECS);
				led::set(Mode::Blink(Duration::from_millis(config::BLINK_COUNTDOWN_MS)));
				Timer::after_secs(config::CALIBRATE_DELAY_SECS).await;
				match sensor.calibrate().await {
					Ok(()) => {
						info!("calibration started ({}s scan)", config::CALIBRATE_SCAN_SECS);
						led::set(Mode::Blink(Duration::from_millis(config::BLINK_SCAN_MS)));
						scan_deadline = Some(
							Instant::now() + Duration::from_secs(config::CALIBRATE_GIVE_UP_SECS),
						);
					},
					Err(e) => {
						warn!("calibration: {}", Debug2Format(&e));
						led::set(if state.occupied {
							Mode::On
						} else {
							Mode::Off
						});
					},
				}
			},
		}
	}
}
