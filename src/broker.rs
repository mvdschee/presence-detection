use crate::config::Config;
use anyhow::Result;
use esp_idf_svc::mqtt::client::{EspMqttClient, EventPayload, MqttClientConfiguration, QoS};
use log::{error, info};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum MqttEvent {
	Connected,
	Disconnected,
	Command(String),
}

pub struct Broker<'a> {
	client: EspMqttClient<'a>,
	cmd_topic: String,
}

impl<'a> Broker<'a> {
	pub fn new(config: Config, event_sender: Sender<MqttEvent>) -> Result<Self> {
		info!("Creating MQTT client...");

		let client_config = MqttClientConfiguration {
			client_id: Some(&config.client_id),
			username: Some(config.broker_username),
			password: Some(config.broker_password),
			..Default::default()
		};

		let host = format!("mqtt://{}:{}", config.broker_host, config.broker_port);
		let cmd_topic = format!("{}/{}/cmd", config.program_name, config.client_id);
		let cmd_topic_cb = cmd_topic.clone();

		info!("Connecting to MQTT broker at {host}");

		let client =
			EspMqttClient::new_cb(&host, &client_config, move |event| match event.payload() {
				EventPayload::Connected(_) => {
					info!("MQTT Connected");
					let _ = event_sender.send(MqttEvent::Connected);
				}
				EventPayload::Disconnected => {
					error!("MQTT Disconnected");
					let _ = event_sender.send(MqttEvent::Disconnected);
				}
				EventPayload::Error(err) => error!("MQTT Error: {err:?}"),
				EventPayload::Received {
					id: _,
					topic,
					data,
					details: _,
				} => {
					if let Some(topic) = topic {
						if topic == cmd_topic_cb {
							if let Ok(payload) = std::str::from_utf8(data) {
								let _ = event_sender.send(MqttEvent::Command(payload.to_string()));
							}
						}
					}
				}
				_ => {}
			})?;

		Ok(Self {
			client,
			cmd_topic,
		})
	}

	pub fn subscribe_cmd(&mut self) -> Result<()> {
		self.client.subscribe(&self.cmd_topic, QoS::AtLeastOnce)?;
		info!("Subscribed to command topic: {}", self.cmd_topic);
		Ok(())
	}

	pub fn publish(&mut self, topic: &str, payload: &str) -> Result<()> {
		match self.client.publish(topic, QoS::AtLeastOnce, false, payload.as_bytes()) {
			Ok(_) => Ok(()),
			Err(err) => Err(err.into()),
		}
	}
}
