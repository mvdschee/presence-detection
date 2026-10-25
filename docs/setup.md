# Setup

Everything comes from `rustup` (read from `rust-toolchain.toml`) or
`cargo install --locked`. No ESP-IDF, espflash or ldproxy.

## 1. Tools

```sh
cargo install --locked probe-rs-tools@0.32.0
```

`rustup` installs Rust 1.99.0 and the `riscv32imc-unknown-none-elf` target on
the first build in this repo.

Check: `probe-rs list` shows `ESP JTAG (EspJtag)` with the C3 plugged in over
USB-C. The C3's USB goes straight to its built-in USB-JTAG, so no other probe
is needed.

## 2. Build, flash, test

- `make` lists the targets.
- `make run` _(works)_: builds, flashes over USB-JTAG and streams defmt logs
  until Ctrl-C. With more than one board plugged in, set `PRESENCE_PROBE` in
  `.env` to the `VID:PID:serial` from `probe-rs list`.
- Per-reading logs: `DEFMT_LOG=presence_firmware=debug,info make run`.
- `make test` _(works)_: core tests on the Mac.
- `make check` _(works)_: clippy for core (host and riscv32imc) and firmware,
  plus a fmt check. rustfmt warns that `imports_granularity` needs nightly;
  the warning is harmless.

## 3. Wiring (ESP32-C3 Super Mini, LD2410S)

| Sensor | C3 |
|---|---|
| 3V3 | 3V3 |
| GND | GND |
| OT1 (TX) | GPIO21 (RX) |
| RX | GPIO20 (TX) |
| OT2 (presence out) | GPIO3, for the battery version's wake-up, unused now |

Power the sensor from the C3's 3V3, not from a USB-serial adapter's 3V3
pin, whose noise can make an empty room read as occupied. Calibrate where it's mounted: hold BOOT for 3 s,
leave the room within 30 s, and the sensor then scans for 120 s.

## Log

- 2026-10-25: moved from ESP-IDF to esp-hal + probe-rs. BLE advertising
  starts. Walking in front of the sensor gives "occupied" within a second,
  and sitting still at about 1.4 m keeps it occupied. Home Assistant
  discovers the BTHome device on its own, and occupancy follows. Not tested
  yet: calibration via BOOT, and running for hours without the probe.
