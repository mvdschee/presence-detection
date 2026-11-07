/// One report from the sensor, already out of the driver's types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
	pub occupied: bool,
	pub distance_cm: u16,
}

/// What the node advertises. The packet id tells receivers that the data is
/// new, so it survives deep sleep together with the occupancy it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresenceState {
	pub occupied: bool,
	pub packet_id: u8,
}
