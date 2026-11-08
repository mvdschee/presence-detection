# BLE presence sensor

A `no_std` esp-hal firmware that broadcasts BTHome v2 occupancy over BLE. It
stays awake for a short window after plug-in or reset, then deep sleeps and
only wakes when the sensor's OT2 output changes, plus a heartbeat.

## Stack

| Concern | Choice | Version |
|---|---|---|
| HAL, runtime | `esp-hal` + `esp-rtos` (embassy) | `=1.2.2`, `=0.4.0` |
| Bootloader descriptor | `esp-bootloader-esp-idf` | `=0.6.0` |
| BLE controller | `esp-radio`, feature `ble` | `=1.0.0-beta.1` |
| HCI commands | `bt-hci` `ExternalController` | `0.9.0` (esp-radio's) |
| Heap (radio blobs need it) | `esp-alloc` | `=0.11.0` |
| Sensor | `ld2410s`, async driver | `0.3.0` (crates.io) |
| Logs, flashing | defmt + rtt-target, probe-rs runner | probe-rs 0.32.0 |
| Target | `riscv32imc-unknown-none-elf` (C3 has no atomics) | Rust 1.99.0 |

There's no BLE host stack (`trouble-host`). The node only advertises, and
that takes four HCI commands: Reset, LE Set Advertising Parameters, LE Set
Advertising Data and LE Set Advertising Enable. All four exist in `bt-hci`
0.9.0, and esp-radio's `BleConnector` implements
`bt_hci::transport::Transport`.

## BTHome v2 frame

From <https://bthome.io/format/>. Everything goes in one legacy
advertisement, which holds at most 31 bytes:

```
02 01 06                      flags (BlueZ in HA needs them)
08 16 D2 FC 44                service data, UUID 0xFCD2, device info 0x44
   00 <id>                    packet id, uint8
   23 <0|1>                   occupancy, uint8
<n> 09 <name>                 complete local name, the name HA shows
```

- Objects go in ascending ID order, or older receivers stop parsing early.
- Device info `0x44` means v2, unencrypted, trigger based (bit 2), so HA
  doesn't mark the device unavailable between advertisements.
- The packet id goes up on every change of occupancy. Receivers drop an
  advertisement whose id equals the last one, so repeating a frame costs
  nothing. A heartbeat keeps the id, which also repairs a change HA missed:
  HA hasn't seen that id yet, so it accepts it.
- The id lives in RTC memory. Every deep-sleep wake is a reset, and an id that
  restarted would make HA drop real changes.
- 3 + 9 + (2 + name) bytes: the name can be up to 17 characters.

## Data flow

```
OT2 (GPIO3) -> core handlers::presence::next_state -> PresenceState
core models::advertisement: BTHome bytes -> services/ble: HCI LE Set Adv Data
services/sleep: wake on the opposite level or the heartbeat, deep sleep
```

Every boot starts the same way: restore the state from RTC memory, read OT2,
start advertising. What follows depends on the reset reason.

- **Deep-sleep wake:** advertise for 1 s, then sleep again. The UART, button
  and LED are never touched, so the sensor isn't reconfigured on each wake.
- **Any other reset (plug-in, RST, flash):** the awake window. For 60 s the
  firmware runs the sensor over UART, shows presence on the blue LED, logs over
  RTT and accepts BOOT. Then it follows the deep-sleep path.

In the awake window:

- OT2 stays the source of occupancy. Each change of OT2 or of the UART's state
  logs both, so a disagreement between them is visible.
- The sensor runs in Minimal mode and reports 8 times a second.
- UART read errors from `next_packet` are logged, then reading continues.
  On esp-hal, the RX FIFO overflows whenever nothing reads it for a moment.
- BOOT (GPIO9) runs in its own task and sets a signal after a 3 s hold.
  The main loop then waits 30 s, so there's time to leave the room, and
  starts the sensor's 120 s auto-threshold scan. The sensor stays in Minimal
  mode afterwards, so no re-init is needed. A calibration keeps the board awake
  past the window until it ends.
- The blue LED (GPIO8, active low) is on while occupied and off while clear.
  It flashes fast during the 30 s countdown and slowly during the scan. It
  shows presence again at the sensor's 100% progress frame, or 135 s after the
  scan started if that frame gets lost. The red LED is tied to 3V3 and can't
  be switched (worth removing for battery power).

## Layout

```
crates/
├── core/                 # no_std, no esp deps, tested on the host
│   └── src/
│       ├── models/       # Reading, PresenceState, Advertisement (BTHome)
│       └── handlers/     # next_state: reading -> state to advertise
└── firmware/
    └── src/
        ├── services/
        │   ├── sensor.rs # UART + ld2410s: init, next reading, calibrate
        │   ├── ble.rs    # BleConnector + HCI: start, advertise
        │   ├── button.rs # BOOT long press -> signal
        │   ├── led.rs    # blue LED: off, on, blink
        │   └── sleep.rs  # RTC state, wake sources, deep sleep
        ├── config.rs     # name, intervals, calibration parameters
        ├── error.rs
        └── main.rs       # esp-hal init, heap, main loop
```

## Sleep

- Deep sleep, not esp-rtos auto light sleep: esp-radio holds a `WakeLock` from
  its init until it's dropped, so light sleep would never start while BLE runs.
- The sensor dominates the sleeping power budget: 45 µA average at the lowest
  report rate, 349 µA at medium, 446 µA at the highest (user manual V1.2). A C3
  in deep sleep draws single-digit µA. The firmware still uses 8 Hz, so the
  effect of sleeping can be judged on its own before lowering the rate.
- **OT2** is a presence output: high when occupied, low when empty (user
  manual V1.2). It's wired to GPIO3. esp-hal gives the C3 a per-pin low-power
  wake path on its low-power pads.
- GPIO wake is level-based. The chip arms the opposite of the state it just
  advertised, not of the level it reads then, so a flip during the 1 s burst
  still wakes it straight away. `sleep_deep_with_rejection` returns instead of
  sleeping if the level has already flipped, and the loop then advertises the
  new state. Arming and sleeping share a critical section, and the pin stops
  listening before it ends, so an asserted level can't fire an interrupt.
- The GPIO3 pad has no pull: OT2 drives it, and a pull would draw current while
  occupied. If OT2 turns out not to drive both levels the pad floats and the
  chip wakes constantly, which a USB meter shows as no drop in current.
- A timer wakes the chip every 30 min as a heartbeat. Battery level can join
  it once there's a voltage divider.
- Deep sleep powers off the radio domain (`RtcSleepConfig::deep` sets
  `bt_pd_en`), so the controller isn't torn down first. esp-radio's `ble_deinit`
  doesn't stop a running controller, so dropping it isn't obviously safer.
- During the 1 s burst the controller repeats at 100 ms, the lowest interval
  older controllers allow for non-connectable advertising. The burst length is
  a guess until HA has been seen to catch every change.

## Pitfalls

- **No Bluetooth on the HA side, no data.** HA needs a Bluetooth adapter on
  its host, or an ESPHome Bluetooth proxy in range, before BTHome shows up.
- **esp-radio's BLE API is unstable** (`#[instability::unstable]`) and still
  beta. There's no raw-HCI advertising example for it. The ESP-IDF example
  `controller_vhci_ble_adv` uses the same four commands and is the closest
  reference.
- **`ExternalController::exec` needs `read()` polled** before command-complete
  events arrive. `services/ble.rs` reads in the same task while a command
  waits.
- **Pins:** the sensor's OT1 (its TX) goes to GPIO21, the C3's **RX**.
- **GPIO21 is UART0's TX (U0TXD), and GPIO20 its RX.** At boot the ROM sends
  its log out of GPIO21, the same wire the sensor's OT1 drives, so two
  outputs fight for a moment until UART1 takes over the pin. This has been
  harmless so far. If it ever isn't, moving the sensor to other pins avoids
  it.
- **Deep sleep cuts USB-JTAG.** RTT stops and `make run` ends with a probe
  error when the board sleeps; that's expected. To flash a sleeping board, press
  RST and flash within the 60 s window. The window is the recovery path, so
  anything that shortens it risks needing BOOT + RST download mode.
- **A dropped UART hangs sleep entry.** esp-hal waits for every UART's TX
  FIFO to drain before sleeping, also one whose driver was dropped. Its clock
  is gated then, so a stale count never drains and the chip never sleeps.
  `main` keeps the sensor alive until the chip sleeps for this reason.
- **Cancelling a future resets its timers.** Selecting a fresh long-press
  future against each reading restarted the 3 s hold on every reading, so
  calibration never triggered. That's why the button has its own task.

## Verification

- **Automatic:** `make check` (clippy, fmt) and `make test` (BTHome bytes and
  the packet id rule on the host). These say nothing about sleep itself.
- **On hardware, verified:** the RTT log shows OT2 following the UART's state,
  the board sleeps after 60 s, and arriving and leaving both reach HA from deep
  sleep. See the log in `docs/setup.md`.
- **On hardware, to check:** HA doesn't mark it unavailable over an hour, and
  the heartbeat doesn't add HA events.
- **Only a person can judge:** how long HA takes to react now, whether detection
  feels as reliable, and whether the drop on the USB meter is worth what was
  given up. On USB the board draws 1.14 mA asleep while occupied, against
  44.6 mA without sleep. The red LED and regulator are part of that 1.14 mA, so
  it shows the difference but not battery life.
