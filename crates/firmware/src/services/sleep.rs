use crate::config;
use esp_hal::{
	gpio::{Event, Input},
	rtc_cntl::{
		SocResetReason, reset_reason,
		sleep::{LowPower, RtcSleepConfig},
	},
	system::Cpu,
	time::{Duration, Instant},
};
use portable_atomic::{AtomicU8, Ordering};
use presence_core::models::presence::PresenceState;

// A wake is a reset, so the state lives in RTC memory. Any bit pattern is a
// valid state, which is what a reset before the first write can leave behind.
#[esp_hal::ram(unstable(rtc_fast, persistent))]
static OCCUPIED: AtomicU8 = AtomicU8::new(0);
#[esp_hal::ram(unstable(rtc_fast, persistent))]
static PACKET_ID: AtomicU8 = AtomicU8::new(0);

pub fn woke_from_deep_sleep() -> bool {
	reset_reason(Cpu::ProCpu) == Some(SocResetReason::CoreDeepSleep)
}

pub fn load_state() -> PresenceState {
	PresenceState {
		occupied: OCCUPIED.load(Ordering::Relaxed) != 0,
		packet_id: PACKET_ID.load(Ordering::Relaxed),
	}
}

pub fn save_state(state: PresenceState) {
	OCCUPIED.store(state.occupied as u8, Ordering::Relaxed);
	PACKET_ID.store(state.packet_id, Ordering::Relaxed);
}

/// Deep sleeps until OT2 differs from `occupied` or the heartbeat is due. It
/// only returns if OT2 already differs, because then the hardware refuses to
/// sleep.
pub fn until_change(lpwr: &mut LowPower<'_>, ot2: &mut Input<'_>, occupied: bool) {
	let wake_on = if occupied {
		Event::LowLevel
	} else {
		Event::HighLevel
	};
	// No interrupt may see the pin listen: a pin already at its wake level would
	// fire straight away. If the sleep is refused the level is still asserted,
	// so the pin stops listening before interrupts come back.
	critical_section::with(|_| {
		ot2.listen(wake_on);
		lpwr.set_wakeup_deadline(Instant::now() + Duration::from_secs(config::HEARTBEAT_SECS));
		lpwr.sleep_deep_with_rejection(RtcSleepConfig::deep());
		ot2.unlisten();
		ot2.clear_interrupt();
	});
}
