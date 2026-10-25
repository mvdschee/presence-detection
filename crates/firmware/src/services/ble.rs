use crate::{Result, config};
use bt_hci::{
	cmd::{
		self,
		controller_baseband::Reset,
		le::{LeSetAdvData, LeSetAdvEnable, LeSetAdvParams},
	},
	controller::{Controller, ControllerCmdSync, ExternalController},
	param::{AddrKind, AdvChannelMap, AdvFilterPolicy, AdvKind, BdAddr, Duration},
};
use core::future::Future;
use embassy_futures::select::{Either, select};
use esp_hal::peripherals::BT;
use esp_radio::ble::controller::{BleConnector, BleConnectorError};
use presence_core::models::advertisement::Advertisement;

type Hci<'d> = ExternalController<BleConnector<'d>, 1>;

/// A non-connectable broadcaster: the controller repeats the current
/// advertisement on its own; `advertise` only swaps the data.
pub struct Ble<'d> {
	hci: Hci<'d>,
}

impl<'d> Ble<'d> {
	pub async fn start(bt: BT<'d>, first: &Advertisement) -> Result<Self> {
		let connector = BleConnector::new(bt, Default::default())?;
		let ble = Self {
			hci: ExternalController::new(connector),
		};
		let interval = Duration::from_millis(config::ADV_INTERVAL_MS);

		ble.serve(ble.hci.exec(&Reset::new())).await?;
		ble.serve(ble.hci.exec(&LeSetAdvParams::new(
			interval,
			interval,
			AdvKind::AdvNonconnInd,
			// the chip's own address, which stays the same across reboots;
			// Home Assistant knows a BTHome device by it
			AddrKind::PUBLIC,
			AddrKind::PUBLIC,
			BdAddr::default(),
			AdvChannelMap::ALL,
			AdvFilterPolicy::Unfiltered,
		)))
		.await?;
		ble.advertise(first).await?;
		ble.serve(ble.hci.exec(&LeSetAdvEnable::new(true))).await?;
		Ok(ble)
	}

	pub async fn advertise(&self, adv: &Advertisement) -> Result<()> {
		let (data, len) = adv.raw();
		self.serve(self.hci.exec(&LeSetAdvData::new(len, data))).await
	}

	/// Drives one command. Its completion event only arrives while the
	/// controller is being read, so this reads until the command returns.
	async fn serve(
		&self,
		command: impl Future<Output = core::result::Result<(), cmd::Error<BleConnectorError>>>,
	) -> Result<()> {
		let mut buf = [0u8; 259];
		let read = async {
			loop {
				if let Err(e) = self.hci.read(&mut buf).await {
					break e;
				}
			}
		};
		match select(command, read).await {
			Either::First(result) => Ok(result?),
			Either::Second(e) => Err(cmd::Error::Io(e).into()),
		}
	}
}
