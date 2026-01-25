use esp_idf_svc::{
	eventloop::EspSystemEventLoop,
	hal::{
		modem::Modem,
		prelude::*,
		uart::{config::Config, UartDriver},
	},
	nvs::EspDefaultNvsPartition,
};

pub struct BoardPeripherals {
	pub uart1: UartDriver<'static>,
	pub modem: Modem,
	pub sys_loop: EspSystemEventLoop,
	pub nvs: EspDefaultNvsPartition,
}

impl BoardPeripherals {
	pub fn new() -> Self {
		let peripherals = Peripherals::take().unwrap();
		let sys_loop = EspSystemEventLoop::take().unwrap();
		let nvs = EspDefaultNvsPartition::take().unwrap();

		// Pins for ESP32-C3 Super Mini + LD2410S
		let tx = peripherals.pins.gpio21;
		let rx = peripherals.pins.gpio20;

		let uart_config = Config::default().baudrate(ld2410s::BAUD_RATE.into());

		let uart1 = UartDriver::new(
			peripherals.uart1,
			tx,
			rx,
			Option::<esp_idf_svc::hal::gpio::AnyIOPin>::None,
			Option::<esp_idf_svc::hal::gpio::AnyIOPin>::None,
			&uart_config,
		)
		.unwrap();

		Self {
			uart1,
			sys_loop,
			modem: peripherals.modem,
			nvs,
		}
	}
}
