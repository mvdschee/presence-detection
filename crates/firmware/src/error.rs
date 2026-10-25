use esp_hal::uart::IoError;
use esp_radio::ble::controller::{BleConnectorError, BleInitError};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Sensor(ld2410s::Error<IoError>),
	BleInit(BleInitError),
	Ble(bt_hci::cmd::Error<BleConnectorError>),
}

impl From<ld2410s::Error<IoError>> for Error {
	fn from(e: ld2410s::Error<IoError>) -> Self {
		Error::Sensor(e)
	}
}

impl From<BleInitError> for Error {
	fn from(e: BleInitError) -> Self {
		Error::BleInit(e)
	}
}

impl From<bt_hci::cmd::Error<BleConnectorError>> for Error {
	fn from(e: bt_hci::cmd::Error<BleConnectorError>) -> Self {
		Error::Ble(e)
	}
}
