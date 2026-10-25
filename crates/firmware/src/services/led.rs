use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	Off,
	On,
	/// Toggles every `Duration`.
	Blink(Duration),
}

static MODE: Signal<CriticalSectionRawMutex, Mode> = Signal::new();

pub fn set(mode: Mode) {
	MODE.signal(mode);
}

/// Drives the Super Mini's blue LED, which is active low.
#[embassy_executor::task]
pub async fn run(mut led: Output<'static>) {
	let mut mode = Mode::Off;
	loop {
		match mode {
			Mode::Off => {
				led.set_high();
				mode = MODE.wait().await;
			},
			Mode::On => {
				led.set_low();
				mode = MODE.wait().await;
			},
			Mode::Blink(half_period) => {
				led.toggle();
				if let Either::Second(next) = select(Timer::after(half_period), MODE.wait()).await {
					mode = next;
				}
			},
		}
	}
}
