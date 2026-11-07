use crate::models::presence::PresenceState;

/// The state to advertise next. A change of occupancy gets a new packet id. No
/// change keeps the id, so receivers drop the repeat, but one that missed the
/// change still accepts it.
pub fn next_state(prev: PresenceState, occupied: bool) -> PresenceState {
	if prev.occupied == occupied {
		prev
	} else {
		PresenceState {
			occupied,
			packet_id: prev.packet_id.wrapping_add(1),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn state(occupied: bool, packet_id: u8) -> PresenceState {
		PresenceState {
			occupied,
			packet_id,
		}
	}

	#[test]
	fn a_change_gets_a_new_packet_id() {
		assert_eq!(next_state(state(false, 3), true), state(true, 4));
		assert_eq!(next_state(state(true, 4), false), state(false, 5));
	}

	#[test]
	fn no_change_keeps_the_packet_id() {
		assert_eq!(next_state(state(true, 4), true), state(true, 4));
		assert_eq!(next_state(state(false, 4), false), state(false, 4));
	}

	#[test]
	fn packet_id_wraps() {
		assert_eq!(next_state(state(false, u8::MAX), true), state(true, 0));
	}

	#[test]
	fn consecutive_changes_never_share_a_packet_id() {
		let mut prev = state(false, 0);
		for i in 0..=u16::from(u8::MAX) * 2 {
			let next = next_state(prev, i % 2 == 0);
			assert_ne!(next.packet_id, prev.packet_id);
			prev = next;
		}
	}
}
