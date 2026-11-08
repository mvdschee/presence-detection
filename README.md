# WORK IN PROGRESS 👷‍♂️

# Presence Detection (Rust + ESP32-C3 + LD2410S)

A presence sensor for Home Assistant: an **ESP32-C3 Super Mini** reads an
**HLK-LD2410S** mmWave radar and broadcasts occupancy as
[BTHome v2](https://bthome.io) over Bluetooth LE. Home Assistant discovers it
on its own, with no Wi-Fi, broker or pairing. It's `no_std` Rust on esp-hal,
and it deep sleeps between changes, ready to move to battery power later.

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

The board deep sleeps 60 s after a plug-in or reset, which also ends USB. Press
RST first if it's asleep, then flash within those 60 s.

## Calibrate

Mount the sensor where it stays and press **RST**. Within 60 s, hold the
**BOOT** button for 3 s and leave the room within 30 s (the blue LED flashes
fast). The sensor then scans the empty room for 120 s (slow flashing). During
those first 60 s the blue LED is otherwise on while someone is present; it stays
off once the board sleeps.

## Docs

- [docs/ble.md](docs/ble.md): design, BTHome frame, battery plan
- [docs/setup.md](docs/setup.md): tools, wiring, log
- [DECISIONS.md](DECISIONS.md): why it's built this way

## Resources

- [ld2410s driver](https://github.com/mvdschee/ld2410s)
- [esp-hal](https://github.com/esp-rs/esp-hal)
- [BTHome format](https://bthome.io/format/)

## License

Copyright (c) 2026 Max van der Schee. Licensed under the [EUPL-1.2](LICENSE).
A commercial license is available for use outside the EUPL terms.
