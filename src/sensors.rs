use anyhow::Result;
use esp_idf_svc::hal::uart::UartDriver;
use ld2410s::{uart::EspUartWrapper, OutputMode, TargetState, LD2410S};

pub struct Sensors<'a> {
	sensor: LD2410S<EspUartWrapper<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PresenceData {
	pub occupied: bool,
	pub distance: u16,
}

impl<'a> Sensors<'a> {
	pub fn new(uart: UartDriver<'a>) -> Self {
		let sensor = LD2410S::new(EspUartWrapper(uart), OutputMode::Standard);
		Self {
			sensor,
		}
	}

	pub fn init(&mut self) -> Result<()> {
		self.sensor.init().map_err(|e| anyhow::anyhow!("Sensor init failed: {:?}", e))?;

		// Configure sensor
		let _ = self.sensor.set_distance_frequency(8.0);
		let _ = self.sensor.set_status_frequency(8.0);
		let _ = self.sensor.set_response_speed(10);

		Ok(())
	}

	pub fn calibrate(&mut self) -> Result<()> {
		// defaults: trigger_factor=2, retention_factor=1, scanning_time=120s
		self.sensor
			.set_auto_threshold(2, 1, 120)
			.map_err(|e| anyhow::anyhow!("Calibration failed: {:?}", e))?;
		Ok(())
	}

	pub fn measure(&mut self) -> Result<PresenceData> {
		let reading =
			self.sensor.read_latest().map_err(|e| anyhow::anyhow!("Read failed: {:?}", e))?;

		match reading {
			Some(r) => {
				let (occupied, distance) = match &r.data {
					ld2410s::Packet::Standard(s) => {
						(matches!(s.target_state, TargetState::Occupied), s.distance_cm)
					}
					ld2410s::Packet::Minimal(m) => {
						(matches!(m.target_state, TargetState::Occupied), m.distance_cm)
					}
					_ => (false, 0),
				};
				Ok(PresenceData {
					occupied,
					distance,
				})
			}
			None => Ok(PresenceData {
				occupied: false,
				distance: 0,
			}),
		}
	}
}
