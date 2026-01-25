use crate::config::Config;
use anyhow::Result;
use esp_idf_svc::{
	eventloop::EspSystemEventLoop,
	hal::modem::Modem,
	nvs::EspDefaultNvsPartition,
	wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;

pub struct Network<'a> {
	pub wifi: BlockingWifi<EspWifi<'a>>,
	pub ssid: &'a str,
	pub password: &'a str,
}

impl<'a> Network<'a> {
	pub fn new(
		modem: Modem,
		sys_loop: EspSystemEventLoop,
		nvs: EspDefaultNvsPartition,
		config: Config,
	) -> Self {
		let wifi_driver = EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap();
		let wifi = BlockingWifi::wrap(wifi_driver, sys_loop).unwrap();

		Self {
			wifi,
			ssid: config.wifi_ssid,
			password: config.wifi_password,
		}
	}

	pub fn init(&mut self) -> Result<()> {
		self.wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

		info!("Starting wifi...");

		self.wifi.start()?;

		info!("Scanning...");

		let ap_infos = self.wifi.scan()?;

		let ours = ap_infos.into_iter().find(|a| a.ssid == self.ssid);

		let channel = if let Some(ours) = ours {
			info!("Found configured access point {} on channel {}", self.ssid, ours.channel);
			Some(ours.channel)
		} else {
			info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            self.ssid
        );
			None
		};

		self.wifi.set_configuration(&Configuration::Client(ClientConfiguration {
			ssid: self.ssid.try_into().expect("Could not parse the given SSID into WiFi config"),
			password: self
				.password
				.try_into()
				.expect("Could not parse the given password into WiFi config"),
			channel,
			auth_method: AuthMethod::WPA2Personal,
			..Default::default()
		}))?;

		info!("Connecting wifi...");

		self.wifi.connect()?;

		info!("Waiting for DHCP lease...");

		self.wifi.wait_netif_up()?;

		let ip_info = self.wifi.wifi().sta_netif().get_ip_info()?;

		info!("Wifi DHCP info: {ip_info:?}");

		Ok(())
	}

	pub fn is_connected(&self) -> bool {
		self.wifi.is_started().unwrap_or(false)
			&& u32::from(self.wifi.wifi().sta_netif().get_ip_info().unwrap().ip) != 0
	}
}
