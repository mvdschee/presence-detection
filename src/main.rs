use crate::broker::MqttEvent;
use crate::{
	broker::Broker, config::Config, network::Network, peripherals::BoardPeripherals,
	report::Reporter, sensors::Sensors,
};
use esp_idf_svc::{
	hal::delay::Delay,
	log::EspLogger,
	sys::{esp_restart, link_patches},
};
use log::{error, info};
use std::{sync::mpsc, time::Instant};

mod broker;
mod config;
mod network;
mod peripherals;
mod report;
mod sensors;

const REFRESH_RATE: u32 = 100u32; // 100ms for faster presence detection

fn main() -> anyhow::Result<()> {
	let delay: Delay = Default::default();

	link_patches();

	EspLogger::initialize_default();

	let config = Config::new();
	let (tx, rx) = mpsc::channel::<MqttEvent>();

	let board = BoardPeripherals::new();
	let mut sensors = Sensors::new(board.uart1);
	let mut network = Network::new(board.modem, board.sys_loop, board.nvs, config.clone());

	let mut reporter = None;
	let mut last_connected = None;

	match network.init() {
		Ok(_) => {
			info!("Network initialized");
			last_connected = Some(Instant::now());

			match Broker::new(config.clone(), tx.clone()) {
				Ok(broker) => {
					reporter = Some(Reporter::new(config.clone(), broker));
					info!("Broker and reporter initialized");
				}
				Err(e) => {
					error!("Broker initialization failed: {e}");
				}
			}
		}
		Err(e) => {
			error!("Network initialization failed: {e}");
		}
	}

	info!("Initializing sensor...");
	sensors.init()?;

	if let Some(ref mut report) = reporter {
		if let Err(e) = report.register() {
			error!("Registration failed: {e}, rebooting...");
			unsafe {
				esp_restart();
			}
		}
	}

	info!("Running loop...");
	loop {
		// Check for MQTT events
		if let Ok(event) = rx.try_recv() {
			match event {
				MqttEvent::Connected => {
					info!("MQTT Connected event received");
					if let Some(ref mut report) = reporter {
						if let Err(e) = report.subscribe_cmd() {
							error!("Failed to subscribe to commands: {e}");
						}
					}
				}
				MqttEvent::Disconnected => {
					info!("MQTT Disconnected event received");
				}
				MqttEvent::Command(cmd) => {
					info!("Received command: {}", cmd);
					match cmd.as_str() {
						"calibrate" => {
							info!("Starting calibration...");
							if let Err(e) = sensors.calibrate() {
								error!("Calibration failed: {:?}", e);
							} else {
								info!("Calibration started (60s scan)");
							}
						}
						"restart" => {
							info!("Restarting...");
							unsafe { esp_restart() };
						}
						_ => info!("Unknown command: {}", cmd),
					}
				}
			}
		}

		if !network.is_connected() {
			if let Some(lc) = last_connected {
				if lc.elapsed().as_secs() > 600 {
					info!("Connection lost for more than 10 minutes, trying to reconnect...");
					match network.init() {
						Ok(_) => {
							info!("Reconnected to wifi");
							last_connected = Some(Instant::now());
						}
						Err(e) => {
							error!("Reconnect failed: {e}");
							last_connected = None;
						}
					}
				}
			}
		} else {
			last_connected = Some(Instant::now());
		}

		match sensors.measure() {
			Ok(data) => {
				if let Some(ref mut report) = reporter {
					if let Err(e) = report.report(data) {
						error!("Reporting failed: {e}, rebooting...");
						unsafe {
							esp_restart();
						}
					}
				}
			}
			Err(e) => {
				error!("Measurement failed: {e}");
			}
		}

		delay.delay_ms(REFRESH_RATE);
	}
}
