# Plan: BTHome Refactor for Battery Efficiency

## Goal
Convert the current "Always-On" WiFi/MQTT presence sensor into a "Deep Sleep" BTHome BLE device to enable >1 year battery life on a standard 2000mAh LiPo.

## Hardware
- **MCU:** ESP32-C3 (Super Mini)
- **Sensor:** HLK-LD2410S
- **Connection:**
  - Sensor TX -> ESP32 RX (GPIO20)
  - Sensor RX -> ESP32 TX (GPIO21)
  - **New:** Sensor OT1 (Out) -> ESP32 GPIO (e.g., GPIO3) for Deep Sleep Wakeup.

## Architecture Changes
1.  **Remove Network Stack:**
    - Delete `src/network.rs` (WiFi).
    - Delete `src/broker.rs` (MQTT).
    - Remove `wifi` and `mqtt` dependencies from `Cargo.toml`.

2.  **Implement BTHome (BLE):**
    - Use `esp-idf-svc::hal::ble` or a raw HCI wrapper.
    - Format: BTHome v2 (Non-encrypted for simplicity, or Encrypted for security).
    - **Payload:**
        - Packet ID `0x21` (Motion/Occupancy): 1 byte (0/1).
        - Packet ID `0x25` (Distance): 2 bytes (mm).

3.  **Logic Flow (Main Loop):**
    - **Boot:** Check wakeup cause.
    - **Measure:** Read UART from LD2410S.
    - **Broadcast:** Advertise BTHome packet for 500ms - 1s.
    - **Sleep:** Enter Deep Sleep, configured to wake on GPIO3 (High).

## Benefits
- **Zero Latency:** No WiFi connection time (save ~3-5s per event).
- **Power:** Active current ~20mA (BLE) vs ~300mA (WiFi).
- **Simplicity:** No router config, no MQTT broker, auto-discovery in Home Assistant.

## Next Steps
- Verify BTHome v2 packet structure.
- Prototype BLE advertising on ESP32-C3 using `esp-idf-svc`.
