use esp_idf_svc::{hal::prelude::Peripherals, log::EspLogger, sys::link_patches};
use log::info;

fn main() -> anyhow::Result<()> {
	link_patches();

	EspLogger::initialize_default();

	info!("Hello, world!");

	let _peripherals = Peripherals::take().unwrap();

	Ok(())
}
