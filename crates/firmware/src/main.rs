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
use embassy_futures::select::{Either3, select3};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{
	gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull, WakeupConfig},
	rtc_cntl::sleep::LowPower,
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
	sleep,
};

// app descriptor the esp-idf bootloader expects
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
	clippy::large_stack_frames,
	reason = "the awake window owns the sensor driver (about 2 KB)"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
	rtt_target::rtt_init_defmt!();

	let p = esp_hal::init(esp_hal::Config::default());
	// the BLE controller allocates
	esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

	let timg0 = TimerGroup::new(p.TIMG0);
	esp_rtos::start(timg0.timer0, p.FROM_CPU_INTR0);

	let mut lpwr = LowPower::new(p.LPWR);
	// the sensor's OT2 drives this pad, high while occupied; a pull would draw
	// current while occupied
	let mut ot2 = Input::new(p.GPIO3, InputConfig::default().with_pull(Pull::None));
	ot2.apply_wakeup_config(&WakeupConfig::default().with_low_power_path(true))
		.expect("GPIO3 has a low-power wake path");

	let mut state = next_state(sleep::load_state(), ot2.is_high());
	sleep::save_state(state);
	let ble = Ble::start(p.BT, &Advertisement::bthome(&state, config::DEVICE_NAME))
		.await
		.expect("ble start");

	if !sleep::woke_from_deep_sleep() {
		info!(
			"presence booted, advertising as {}: occupied={} packet_id={}",
			config::DEVICE_NAME,
			state.occupied,
			state.packet_id
		);
		// sensor OT1 (its TX) -> GPIO21, sensor RX -> GPIO20
		let uart = Uart::new(p.UART1, uart::Config::default().with_baudrate(ld2410s::BAUD_RATE))
			.expect("uart config")
			.with_rx(p.GPIO21)
			.with_tx(p.GPIO20)
			.into_async();
		// the BOOT button
		let boot = Input::new(p.GPIO9, InputConfig::default().with_pull(Pull::Up));
		let hold = Duration::from_secs(config::CALIBRATE_HOLD_SECS);
		spawner.spawn(button::watch(boot, hold).expect("button task already running"));
		// the blue LED, active low: starts off
		let blue = Output::new(p.GPIO8, Level::High, OutputConfig::default());
		spawner.spawn(led::run(blue).expect("led task already running"));

		state = awake_window(Sensor::new(uart), &ot2, &ble, state).await;
		led::set(Mode::Off);
		info!(
			"awake window over, deep sleep ends this log: occupied={} packet_id={}",
			state.occupied, state.packet_id
		);
	}

	loop {
		state = next_state(state, ot2.is_high());
		sleep::save_state(state);
		if let Err(e) = ble.advertise(&Advertisement::bthome(&state, config::DEVICE_NAME)).await {
			warn!("advertise: {}", Debug2Format(&e));
		}
		Timer::after_millis(config::ADV_BURST_MS).await;
		sleep::until_change(&mut lpwr, &mut ot2, state.occupied);
	}
}

/// Runs until the window has passed and no calibration is under way. Returns
/// the state last advertised.
#[allow(
	clippy::large_stack_frames,
	reason = "owns the sensor driver (about 2 KB); runs once per boot"
)]
async fn awake_window(
	mut sensor: Sensor<'_>,
	ot2: &Input<'_>,
	ble: &Ble<'_>,
	mut state: PresenceState,
) -> PresenceState {
	let window_end = Instant::now() + Duration::from_secs(config::AWAKE_WINDOW_SECS);
	show_presence(state);

	while let Err(e) = sensor.init().await {
		warn!("sensor init: {}", Debug2Format(&e));
		if Instant::now() >= window_end {
			return state;
		}
		Timer::after_secs(1).await;
	}
	info!("sensor ready");

	// Some while a scan runs: when to give up waiting for its 100% frame.
	// The LED shows the scan until then, not presence.
	let mut scan_deadline: Option<Instant> = None;
	let mut uart_occupied = state.occupied;
	loop {
		let wake_at = scan_deadline.unwrap_or(window_end);
		match select3(sensor.next_event(), button::LONG_PRESS.wait(), Timer::at(wake_at)).await {
			Either3::First(Event::Reading(reading)) => {
				let next = next_state(state, ot2.is_high());
				// both sides of each change, so an OT2 that doesn't follow the
				// UART shows up here rather than as missed wakes later
				if next != state || reading.occupied != uart_occupied {
					info!(
						"ot2={} uart={} distance={}cm",
						next.occupied, reading.occupied, reading.distance_cm
					);
				}
				uart_occupied = reading.occupied;
				if next != state {
					state = next;
					sleep::save_state(state);
					let adv = Advertisement::bthome(&state, config::DEVICE_NAME);
					if let Err(e) = ble.advertise(&adv).await {
						warn!("advertise: {}", Debug2Format(&e));
					}
				}
				if scan_deadline.is_none() {
					show_presence(state);
				}
			},
			Either3::First(Event::Calibration(progress)) => {
				info!("calibration {}%", progress);
				if progress >= 100 {
					scan_deadline = None;
					show_presence(state);
				}
			},
			Either3::Second(()) => {
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
						show_presence(state);
					},
				}
			},
			Either3::Third(()) => {
				if scan_deadline.take().is_none() {
					return state;
				}
				warn!("no 100% from the scan; showing presence again");
				show_presence(state);
			},
		}
	}
}

fn show_presence(state: PresenceState) {
	led::set(if state.occupied {
		Mode::On
	} else {
		Mode::Off
	});
}
