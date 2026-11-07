//! BTHome v2 advertising data (<https://bthome.io/format/>).

use super::presence::PresenceState;

/// A legacy advertisement's data is at most 31 bytes.
pub const ADV_DATA_LEN: usize = 31;

const FLAGS: [u8; 3] = [0x02, 0x01, 0x06];
/// Service data with BTHome's 16-bit UUID 0xFCD2, little-endian.
const SERVICE_DATA_HEADER: [u8; 3] = [0x16, 0xD2, 0xFC];
/// Version 2, unencrypted, trigger based: the node sleeps between
/// advertisements, so receivers shouldn't expect a regular interval.
const DEVICE_INFO: u8 = 0x44;
const OBJ_PACKET_ID: u8 = 0x00;
const OBJ_OCCUPANCY: u8 = 0x23;
/// Service data AD: length byte, header, device info, then 2 + 2 object bytes.
const SERVICE_DATA_LEN: usize = 1 + SERVICE_DATA_HEADER.len() + 1 + 4;
const AD_TYPE_COMPLETE_NAME: u8 = 0x09;

/// What's left for the complete local name after flags and service data.
pub const MAX_NAME_LEN: usize = ADV_DATA_LEN - FLAGS.len() - SERVICE_DATA_LEN - 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Advertisement {
	data: [u8; ADV_DATA_LEN],
	len: u8,
}

impl Advertisement {
	/// Objects go in ascending ID order, or older receivers stop parsing early.
	/// A name longer than `MAX_NAME_LEN` is cut off.
	pub fn bthome(state: &PresenceState, name: &str) -> Self {
		let name = &name.as_bytes()[..name.len().min(MAX_NAME_LEN)];

		let mut data = [0u8; ADV_DATA_LEN];
		let mut len = 0;
		let mut push = |bytes: &[u8]| {
			data[len..len + bytes.len()].copy_from_slice(bytes);
			len += bytes.len();
		};
		push(&FLAGS);
		push(&[(SERVICE_DATA_LEN - 1) as u8]);
		push(&SERVICE_DATA_HEADER);
		push(&[DEVICE_INFO, OBJ_PACKET_ID, state.packet_id]);
		push(&[OBJ_OCCUPANCY, state.occupied as u8]);
		push(&[name.len() as u8 + 1, AD_TYPE_COMPLETE_NAME]);
		push(name);

		Self {
			data,
			len: len as u8,
		}
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.data[..self.len as usize]
	}

	/// The fixed-size buffer and length the HCI command takes.
	pub fn raw(&self) -> ([u8; ADV_DATA_LEN], u8) {
		(self.data, self.len)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn encodes_occupied() {
		let state = PresenceState {
			occupied: true,
			packet_id: 9,
		};
		let adv = Advertisement::bthome(&state, "hall");
		#[rustfmt::skip]
		let expected = [
			0x02, 0x01, 0x06,
			0x08, 0x16, 0xD2, 0xFC, 0x44,
			0x00, 0x09,
			0x23, 0x01,
			0x05, 0x09, b'h', b'a', b'l', b'l',
		];
		assert_eq!(adv.as_bytes(), expected);
	}

	#[test]
	fn longest_name_fills_the_advertisement() {
		let name = "abcdefghijklmnopqrstuvwxyz";
		let state = PresenceState {
			occupied: false,
			packet_id: 0,
		};
		let adv = Advertisement::bthome(&state, name);
		assert_eq!(adv.as_bytes().len(), ADV_DATA_LEN);
		assert!(adv.as_bytes().ends_with(&name.as_bytes()[..MAX_NAME_LEN]));
	}
}
