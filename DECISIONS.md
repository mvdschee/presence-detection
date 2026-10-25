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
