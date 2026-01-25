use esp_idf_svc::sys::{esp_efuse_mac_get_default, esp_err_t};

#[derive(Debug, Clone)]
pub struct Config {
	pub broker_password: &'static str,
	pub broker_port: u16,
	pub broker_host: &'static str,
	pub broker_username: &'static str,
	pub client_id: String,
	pub program_name: &'static str,
	pub wifi_ssid: &'static str,
	pub wifi_password: &'static str,
}

impl Config {
	pub fn new() -> Config {
		let ssid = env!("WIFI_SSID");
		let password = env!("WIFI_PASS");

		let broker_username = env!("MQTT_USERNAME");
		let broker_password = env!("MQTT_PASSWORD");
		let broker_host = env!("MQTT_HOST");
		let broker_port = env!("MQTT_PORT").parse::<u16>().unwrap();

		let mac = get_unique_id().unwrap();
		let unique_id = format!("{:02x}{:02x}{:02x}", mac[3], mac[4], mac[5]);

		Config {
			broker_password,
			broker_port,
			broker_host,
			broker_username,
			client_id: unique_id,
			program_name: "presence_detection",
			wifi_ssid: ssid,
			wifi_password: password,
		}
	}
}

fn get_unique_id() -> Result<[u8; 6], esp_err_t> {
	let mut mac: [u8; 6] = [0; 6];
	let err = unsafe { esp_efuse_mac_get_default(mac.as_mut_ptr()) };
	if err == 0 {
		Ok(mac)
	} else {
		Err(err)
	}
}
