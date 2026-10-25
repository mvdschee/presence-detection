# BLE presence sensor

A `no_std` esp-hal firmware that broadcasts BTHome v2 over BLE. It runs on
mains power for now, and is built so that a battery version only changes how
the firmware sleeps.

## Stack

| Concern | Choice | Version |
|---|---|---|
| HAL, runtime | `esp-hal` + `esp-rtos` (embassy) | `=1.2.2`, `=0.4.0` |
| Bootloader descriptor | `esp-bootloader-esp-idf` | `=0.6.0` |
| BLE controller | `esp-radio`, feature `ble` | `=1.0.0-beta.1` |
| HCI commands | `bt-hci` `ExternalController` | `0.9.0` (esp-radio's) |
| Heap (radio blobs need it) | `esp-alloc` | `=0.11.0` |
| Sensor | `ld2410s`, async driver | git tag `v0.2.0` |
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
0B 16 D2 FC 40                service data, UUID 0xFCD2, device info 0x40
   00 <id>                    packet id, uint8
   23 <0|1>                   occupancy, uint8
   40 <lo hi>                 distance, uint16 mm
<n> 09 <name>                 complete local name, the name HA shows
```

- Objects go in ascending ID order, or older receivers stop parsing early.
- Device info `0x40` means v2, unencrypted, regular interval. The battery
  version sets bit 2 (trigger based), so HA doesn't mark the device
  unavailable between advertisements.
- The packet id goes up on every change of data. Receivers drop an
  advertisement whose id equals the last one, so repeating a frame costs
  nothing.
- The sensor reports distance in cm, so the value sent is mm = cm × 10.
- 3 + 12 + (2 + name) bytes: the name can be up to 14 characters.

## Data flow

```
UART (GPIO21 rx, GPIO20 tx) -> ld2410s async driver -> services/sensor: Reading
core handlers::presence::next_state: changed? -> PresenceState
core models::advertisement: BTHome bytes -> services/ble: HCI LE Set Adv Data
```

- The sensor runs in Minimal mode (state and distance) and reports several
  times a second. The advertising data is only rewritten when occupancy
  changes, or when the distance moves by 25 cm or more. Otherwise the
  controller repeats the last frame at the advertising interval (1 s).
- UART read errors from `next_packet` are logged, then reading continues.
  On esp-hal, the RX FIFO overflows whenever nothing reads it for a moment.
- BOOT (GPIO9) runs in its own task and sets a signal after a 3 s hold.
  The main loop then waits 30 s, so there's time to leave the room, and
  starts the sensor's 120 s auto-threshold scan. The sensor stays in Minimal
  mode afterwards, so no re-init is needed.
- The blue LED (GPIO8, active low) is on while occupied and off while clear.
  It flashes fast during the 30 s countdown and slowly during the scan. It
  shows presence again at the sensor's 100% progress frame, or at the first
  reading 135 s after the scan started if that frame gets lost. The red LED
  is tied to 3V3 and can't be switched (worth removing for battery power).

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
        │   └── led.rs    # blue LED: off, on, blink
        ├── config.rs     # name, intervals, calibration parameters
        ├── error.rs
        └── main.rs       # esp-hal init, heap, main loop
```

## Battery version (later)

- The sensor dominates the power budget: 45 µA average at the lowest report
  rate, 349 µA at medium, 446 µA at the highest (Hi-Link datasheet, via a
  distributor listing). A C3 in deep sleep draws single-digit µA.
- **OT2** is a presence output: high when occupied, low when empty. It's
  wired to GPIO3. On the C3 only GPIO0–GPIO5 can wake the chip from deep
  sleep; this still needs checking against the datasheet. One report says
  GPIO0/1 didn't wake the chip but GPIO2/3 did.
- Deep-sleep GPIO wake on the C3 is level-based, not edge-based. Before
  sleeping, arm the wake on the opposite of OT2's current level. Then both
  "arrived" and "left" wake it.
- On wake: advertise a few copies with the same packet id for about 1 s,
  then sleep again. Distance isn't updated between wakes; occupancy is.
- Add a timer wake, for example every 10 min, as a heartbeat. Add battery
  level once there's a voltage divider on the battery.
- esp-hal 1.2.2 has deep sleep with GPIO and timer wake sources for the C3
  (`rtc_cntl::sleep`). The exact API still needs reading.

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
- **Cancelling a future resets its timers.** Selecting a fresh long-press
  future against each reading restarted the 3 s hold on every reading, so
  calibration never triggered. That's why the button has its own task.

## Verification

- **Automatic:** `make check` (clippy, fmt), `make test` (BTHome bytes and
  the change rule on the host), and `make run` to flash and log over
  probe-rs.
- **On hardware:** HA discovers the device and occupancy follows. Still
  untested: calibration with BOOT, and running for hours without the probe
  attached.
