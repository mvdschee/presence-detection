/// One report from the sensor, already out of the driver's types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
	pub occupied: bool,
	pub distance_cm: u16,
}

/// What the node advertises. Distance is 0 while clear, so Home Assistant
/// doesn't keep showing where the last person was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresenceState {
	pub occupied: bool,
	pub distance_cm: u16,
}

impl From<Reading> for PresenceState {
	fn from(r: Reading) -> Self {
		Self {
			occupied: r.occupied,
			distance_cm: if r.occupied {
				r.distance_cm
			} else {
				0
			},
		}
	}
}
