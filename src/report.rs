use crate::{broker::Broker, config::Config, sensors::PresenceData};
use anyhow::{Error, Result};
use log::info;

pub struct Reporter<'a> {
	config: Config,
	broker: Broker<'a>,
	last_report: Option<PresenceData>,
}

impl<'a> Reporter<'a> {
	pub fn new(config: Config, broker: Broker<'a>) -> Self {
		Self {
			config,
			broker,
			last_report: None,
		}
	}

	pub fn report(&mut self, data: PresenceData) -> Result<(), Error> {
		if let Some(last) = &self.last_report {
			if *last == data {
				return Ok(());
			}
		}

		let topic = format!("{}/{}/state", self.config.program_name, self.config.client_id);
		let payload = format!(
			r#"{{
                "occupied": {},
                "distance": {}
            }}"#,
			data.occupied, data.distance
		);

		self.broker.publish(&topic, &payload)?;
		self.last_report = Some(data);

		Ok(())
	}
	pub fn register(&mut self) -> Result<(), Error> {
		let state_topic = format!("{}/{}/state", self.config.program_name, self.config.client_id);
		let device_name =
			format!("{} {}", self.config.program_name.replace("_", " "), self.config.client_id);
		let model_id = format!("{}_{}", self.config.program_name, self.config.client_id);

		// Register binary sensor for occupancy
		let binary_config_topic =
			format!("homeassistant/binary_sensor/{}_{}/config", self.config.client_id, "occupancy");
		let binary_config = format!(
			r#"{{
                "state_topic": "{}",
                "value_template": "{{{{ 'ON' if value_json.occupied else 'OFF' }}}}",
                "unique_id": "{}_occupancy",
                "name": "Occupancy",
                "device_class": "motion",
                "device": {{
                    "name": "{}",
                    "identifiers": ["{}"],
                    "manufacturer": "mvdschee",
                    "model": "LD2410S"
                }}
            }}"#,
			state_topic, self.config.client_id, device_name, model_id
		);
		self.broker.publish(&binary_config_topic, &binary_config)?;

		// Register sensor for distance
		let distance_config_topic =
			format!("homeassistant/sensor/{}_{}/config", self.config.client_id, "distance");
		let distance_config = format!(
			r#"{{
                "state_topic": "{}",
                "unit_of_measurement": "cm",
                "value_template": "{{{{ value_json.distance }}}}",
                "unique_id": "{}_distance",
                "name": "Distance",
                "device_class": "distance",
                "state_class": "measurement",
                "device": {{
                    "name": "{}",
                    "identifiers": ["{}"],
                    "manufacturer": "mvdschee",
                    "model": "LD2410S"
                }}
            }}"#,
			state_topic, self.config.client_id, device_name, model_id
		);
		self.broker.publish(&distance_config_topic, &distance_config)?;

		// Register Calibrate Button
		let cmd_topic = format!("{}/{}/cmd", self.config.program_name, self.config.client_id);
		let calibrate_config_topic =
			format!("homeassistant/button/{}_{}/config", self.config.client_id, "calibrate");
		let calibrate_config = format!(
			r#"{{
                "command_topic": "{}",
                "payload_press": "calibrate",
                "unique_id": "{}_calibrate",
                "name": "Calibrate Sensor",
                "icon": "mdi:target",
                "device": {{
                    "name": "{}",
                    "identifiers": ["{}"],
                    "manufacturer": "mvdschee",
                    "model": "LD2410S"
                }}
            }}"#,
			cmd_topic, self.config.client_id, device_name, model_id
		);
		self.broker.publish(&calibrate_config_topic, &calibrate_config)?;

		// Register Restart Button
		let restart_config_topic =
			format!("homeassistant/button/{}_{}/config", self.config.client_id, "restart");
		let restart_config = format!(
			r#"{{
                "command_topic": "{}",
                "payload_press": "restart",
                "unique_id": "{}_restart",
                "name": "Restart Device",
                "device_class": "restart",
                "device": {{
                    "name": "{}",
                    "identifiers": ["{}"],
                    "manufacturer": "mvdschee",
                    "model": "LD2410S"
                }}
            }}"#,
			cmd_topic, self.config.client_id, device_name, model_id
		);
		self.broker.publish(&restart_config_topic, &restart_config)?;

		info!("Registered sensors with Home Assistant");
		Ok(())
	}

	pub fn subscribe_cmd(&mut self) -> Result<(), Error> {
		self.broker.subscribe_cmd()?;
		Ok(())
	}
}
