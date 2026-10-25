use presence_core::models::advertisement::MAX_NAME_LEN;

/// The name Home Assistant shows for the device.
pub const DEVICE_NAME: &str = "presence";
const _: () = assert!(DEVICE_NAME.len() <= MAX_NAME_LEN);

/// How often the radio repeats the current advertisement.
pub const ADV_INTERVAL_MS: u32 = 1000;
/// A new advertisement once the distance has moved this far.
pub const DISTANCE_STEP_CM: u16 = 25;

/// Distance and status reports per second.
pub const REPORT_HZ: f32 = 8.0;
pub const RESPONSE_SPEED: u16 = 10;

/// Holding BOOT this long starts a calibration.
pub const CALIBRATE_HOLD_SECS: u64 = 3;
/// Time to leave the room after the hold, before the scan starts.
pub const CALIBRATE_DELAY_SECS: u64 = 30;
/// Auto-threshold parameters, the manual's defaults.
pub const CALIBRATE_TRIGGER_FACTOR: u16 = 2;
pub const CALIBRATE_RETENTION_FACTOR: u16 = 1;
pub const CALIBRATE_SCAN_SECS: u16 = 120;
/// If the sensor's 100% frame gets lost, the LED stops blinking at the first
/// reading this long after the scan started.
pub const CALIBRATE_GIVE_UP_SECS: u64 = CALIBRATE_SCAN_SECS as u64 + 15;

/// LED half-periods: fast while counting down to the scan, slow during it.
pub const BLINK_COUNTDOWN_MS: u64 = 125;
pub const BLINK_SCAN_MS: u64 = 500;
