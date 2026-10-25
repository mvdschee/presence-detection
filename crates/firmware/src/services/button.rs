use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, with_timeout};
use esp_hal::gpio::Input;

/// Set each time the button has been held long enough.
pub static LONG_PRESS: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Its own task, so the main loop's readings can't restart a hold that's
/// under way.
#[embassy_executor::task]
pub async fn watch(mut button: Input<'static>, hold: Duration) {
	loop {
		long_press(&mut button, hold).await;
		LONG_PRESS.signal(());
	}
}

/// Waits until the button (active low) is held for `hold`. Shorter presses are
/// ignored.
async fn long_press(button: &mut Input<'_>, hold: Duration) {
	loop {
		button.wait_for_low().await;
		if with_timeout(hold, button.wait_for_high()).await.is_err() {
			return;
		}
	}
}
