# WORK IN PROGRESS 👷‍♂️

# Presence Detection (Rust + ESP32-C3 + LD2410S)

A presence sensor for Home Assistant: an **ESP32-C3 Super Mini** reads an
**HLK-LD2410S** mmWave radar and broadcasts occupancy and distance as
[BTHome v2](https://bthome.io) over Bluetooth LE. Home Assistant discovers it
on its own, with no Wi-Fi, broker or pairing. It's `no_std` Rust on esp-hal,
built to move to battery power later.

## Hardware

- **MCU:** ESP32-C3 Super Mini
- **Sensor:** Hi-Link HLK-LD2410S
- Wiring: see [docs/setup.md](docs/setup.md#3-wiring-esp32-c3-super-mini-ld2410s)

Home Assistant needs Bluetooth in range: an adapter on its host or an ESPHome
Bluetooth proxy.

## Build and flash

Needs [probe-rs](https://probe.rs) (`cargo install --locked probe-rs-tools@0.32.0`);
`rustup` fetches the toolchain from `rust-toolchain.toml`.

```sh
make run     # flash over USB and stream logs
make test    # core tests on the host
make check   # clippy and fmt
```

## Calibrate

Mount the sensor where it stays, then hold the **BOOT** button for 3 s and
leave the room within 30 s. The sensor scans the empty room for 120 s.

## Docs

- [docs/ble.md](docs/ble.md): design, BTHome frame, battery plan
- [docs/setup.md](docs/setup.md): tools, wiring, log
- [DECISIONS.md](DECISIONS.md): why it's built this way

## Resources

- [ld2410s driver](https://github.com/mvdschee/ld2410s)
- [esp-hal](https://github.com/esp-rs/esp-hal)
- [BTHome format](https://bthome.io/format/)
