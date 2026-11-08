# Decisions

## 2026-10-25 - no_std esp-hal with probe-rs, not ESP-IDF
**Decision:** The firmware moves from `esp-idf-svc` (std) to `esp-hal` + `esp-rtos` (embassy), logs with defmt over RTT and flashes with probe-rs.
**Why:** Everything installs through `rustup` and `cargo install`; ESP-IDF needs ldproxy plus its own Python and C toolchain. The ld2410s async driver is verified on an ESP32-C3 with esp-hal.
**Alternatives:** Keep ESP-IDF and flash with probe-rs (logs would no longer come over RTT, so a separate serial monitor is needed).

## 2026-10-25 - BTHome v2 over BLE instead of Wi-Fi + MQTT
**Decision:** The node broadcasts BTHome v2 advertisements (occupancy, distance) and has no Wi-Fi, MQTT or Home Assistant discovery code.
**Why:** It's the path to battery power: no Wi-Fi association, no broker, and HA discovers BTHome on its own. Advertising is one-way, so commands like calibrate need another trigger.
**Alternatives:** Keep MQTT on mains and add BLE later (two radios' worth of code); ESPHome (not Rust).

## 2026-10-25 - Raw HCI advertising, no BLE host stack
**Decision:** Advertise through esp-radio's `BleConnector` and `bt-hci`'s `ExternalController` with four HCI commands (reset, adv params, adv data, adv enable).
**Why:** A broadcaster needs no GATT, pairing or connections. Four commands are less code and less flash than a host stack.
**Alternatives:** `trouble-host` (needed only once the node accepts connections).

## 2026-10-25 - Sensor in Minimal output mode
**Decision:** The firmware runs the LD2410S in Minimal mode (state and distance) rather than Standard (plus per-gate energy).
**Why:** Energy isn't advertised, and the sensor drops back to Minimal after an auto-threshold scan anyway, so no re-init is needed after calibrating.
**Alternatives:** Standard mode with `init()` again after the scan (only worth it if energy gets reported).

## 2026-10-25 - New advertisement on occupancy change or a 25 cm step
**Decision:** `next_state` returns a new state only when occupancy flips or the distance moves by at least `DISTANCE_STEP_CM` (25). Distance is 0 while clear.
**Why:** The sensor reports several times a second and distance jitters by a few cm; each new state bumps the BTHome packet id and becomes an HA state change. Sitting still at ~1.4 m varied by about ±5 cm.
**Alternatives:** Advertise every reading (HA history fills with noise); time-based throttling (delays real changes).

## 2026-11-06 - Deep sleep on OT2 with a 60 s awake window, no awake build
**Decision:** After a deep-sleep wake the firmware reads OT2, advertises for 1 s and sleeps again, waking on the opposite OT2 level or a 10 min heartbeat. Any other reset first stays awake for 60 s, for flashing, logs and BOOT calibration. There is one firmware, with no always-awake mode.
**Why:** Sleep behaviour should run on mains now, so it's proven before the battery arrives. Deep sleep cuts USB-JTAG and BOOT (GPIO9) can't wake the chip, so a window after reset is what keeps the board flashable and calibratable.
**Alternatives:** esp-rtos auto light sleep (esp-radio holds a `WakeLock` while BLE runs, so it would never sleep); a cargo feature for an awake build (easier before/after comparison, more code to keep working).

## 2026-11-06 - Occupancy only, from OT2; distance leaves the advertisement
**Decision:** OT2 is the source of occupancy everywhere, also in the awake window. The BTHome frame no longer carries distance, and the frame is trigger based (`0x44`). This replaces the 25 cm step rule from 2026-10-25. The UART still runs in the awake window for setup, calibration and logs.
**Why:** The chip is off between wakes, so a distance would go stale right away. Reading a UART frame on each wake would add up to 2 s awake at the lowest report rate. Using OT2 in the awake window too means the wake signal gets exercised where logs can still show it.
**Alternatives:** One distance reading per wake (a stale snapshot, longer awake time); UART as the source while awake and OT2 while asleep (two sources that can disagree at the switch).

## 2026-11-06 - Packet id and occupancy in RTC fast memory
**Decision:** Both live in `#[ram(unstable(rtc_fast, persistent))]` atomics. A change bumps the id; a heartbeat keeps it.
**Why:** Each wake is a reset, and receivers drop an id equal to the last one, so an id that restarted could hide a real change. Keeping the id on a heartbeat avoids extra HA events, and still repairs a change HA missed, since HA hasn't seen that id.
**Alternatives:** Bump the id on every wake (a heartbeat becomes an HA event); a `static mut` (the firmware forbids unsafe code).

## 2026-11-08 - EUPL-1.2 plus a commercial license, not MIT
**Decision:** The project moves from MIT to EUPL-1.2, matching the `ld2410s` driver, and is also offered under a separate commercial license.
**Why:** The firmware will be sold with hardware. EUPL makes anyone who ships changed firmware share the source; a commercial license covers buyers who don't want to. Dual licensing only works while all code has one copyright holder.
**Alternatives:** Keep MIT (anyone can sell it closed); a noncommercial source-available license (stops others selling it, but isn't open source).

## 2026-11-08 - Heartbeat every 30 min, not 10
**Decision:** `HEARTBEAT_SECS` is 1800.
**Why:** At the sensor's lowest report rate, the 1 s wakes become the largest share of the sleeping budget; 30 min cuts them to a third. The frame is trigger based (`0x44`), so HA doesn't mark the device unavailable between heartbeats. The cost: a change HA missed takes up to 30 min to repair.
**Alternatives:** 10 min (faster repair, roughly three times the wake cost); longer still (slower repair for a small extra gain).
