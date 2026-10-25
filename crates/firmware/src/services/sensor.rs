use crate::{Result, config};
use defmt::{Debug2Format, debug, info, warn};
use embassy_time::Timer;
use esp_hal::{Async, uart::Uart};
use ld2410s::{OutputMode, Packet, TargetState, asynchronous::LD2410SAsync};
use presence_core::models::presence::Reading;

/// The LD2410S over UART. Minimal mode is enough (state and distance), and it's
/// also the mode the sensor drops back to after a calibration.
pub struct Sensor<'d> {
	driver: LD2410SAsync<Uart<'d, Async>>,
}

impl<'d> Sensor<'d> {
	#[allow(
		clippy::large_stack_frames,
		reason = "the driver's parse buffer is about 2 KB; built once at boot"
	)]
	pub fn new(uart: Uart<'d, Async>) -> Self {
		Self {
			driver: LD2410SAsync::new(uart, OutputMode::Minimal),
		}
	}

	pub async fn init(&mut self) -> Result<()> {
		self.driver.init().await?;
		self.driver.set_distance_frequency(config::REPORT_HZ).await?;
		self.driver.set_status_frequency(config::REPORT_HZ).await?;
		self.driver.set_response_speed(config::RESPONSE_SPEED).await?;
		Ok(())
	}

	/// Waits for the next presence report. Read errors (a FIFO that overflowed
	/// while nobody read) only lose bytes, so they're logged and skipped.
	pub async fn next_reading(&mut self) -> Reading {
		loop {
			match self.driver.next_packet().await {
				Ok(Some(Packet::Minimal(m))) => return reading(&m.target_state, m.distance_cm),
				Ok(Some(Packet::Standard(s))) => return reading(&s.target_state, s.distance_cm),
				Ok(Some(Packet::ThresholdProgress(p))) => info!("calibration {}%", p / 100),
				Ok(_) => {},
				Err(e) => {
					warn!("sensor read: {}", Debug2Format(&e));
					// keeps a persistent fault from spinning
					Timer::after_millis(5).await;
				},
			}
		}
	}

	/// Starts the auto-threshold scan. It runs on the sensor; reports pause
	/// until it's done and `next_reading` logs its progress meanwhile.
	pub async fn calibrate(&mut self) -> Result<()> {
		self.driver
			.set_auto_threshold(
				config::CALIBRATE_TRIGGER_FACTOR,
				config::CALIBRATE_RETENTION_FACTOR,
				config::CALIBRATE_SCAN_SECS,
			)
			.await?;
		Ok(())
	}
}

fn reading(state: &TargetState, distance_cm: u16) -> Reading {
	debug!("reading {} {}cm", Debug2Format(state), distance_cm);
	Reading {
		occupied: matches!(state, TargetState::Occupied),
		distance_cm,
	}
}
