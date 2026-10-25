use crate::models::presence::{PresenceState, Reading};

/// The state to advertise next, or `None` when the current advertisement still
/// holds. It changes on occupancy, or when the distance moves by at least
/// `distance_step_cm`, so 8 readings a second don't each become a new packet.
pub fn next_state(
	prev: Option<PresenceState>,
	reading: Reading,
	distance_step_cm: u16,
) -> Option<PresenceState> {
	let next = PresenceState::from(reading);
	match prev {
		Some(prev)
			if prev.occupied == next.occupied
				&& prev.distance_cm.abs_diff(next.distance_cm) < distance_step_cm =>
		{
			None
		},
		_ => Some(next),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const STEP: u16 = 25;

	fn reading(occupied: bool, distance_cm: u16) -> Reading {
		Reading {
			occupied,
			distance_cm,
		}
	}

	fn state(occupied: bool, distance_cm: u16) -> Option<PresenceState> {
		Some(PresenceState {
			occupied,
			distance_cm,
		})
	}

	#[test]
	fn first_reading_is_advertised() {
		assert_eq!(next_state(None, reading(true, 100), STEP), state(true, 100));
	}

	#[test]
	fn occupancy_change_is_advertised() {
		assert_eq!(next_state(state(false, 0), reading(true, 100), STEP), state(true, 100));
		assert_eq!(next_state(state(true, 100), reading(false, 100), STEP), state(false, 0));
	}

	#[test]
	fn small_distance_change_is_not() {
		assert_eq!(next_state(state(true, 100), reading(true, 124), STEP), None);
		assert_eq!(next_state(state(true, 100), reading(true, 76), STEP), None);
	}

	#[test]
	fn distance_step_is_advertised() {
		assert_eq!(next_state(state(true, 100), reading(true, 125), STEP), state(true, 125));
		assert_eq!(next_state(state(true, 100), reading(true, 75), STEP), state(true, 75));
	}

	#[test]
	fn distance_while_clear_is_ignored() {
		assert_eq!(next_state(state(false, 0), reading(false, 300), STEP), None);
	}
}
