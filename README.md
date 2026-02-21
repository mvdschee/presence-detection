# WORK IN PROGRESS 👷‍♂️

# Presence Detection (Rust + ESP32-C3 + LD2410S)

This project implements a presence detection node for Home Assistant using the **ESP32-C3 Super Mini** and the **HLK-LD2410S** mmWave radar sensor. It is written in Rust using the `esp-idf-svc` framework.

## Hardware

- **MCU:** ESP32-C3 Super Mini (or generic ESP32-C3)
- **Sensor:** Hi-Link HLK-LD2410S (mmWave Radar)

## Software Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/tools/install) (Nightly required, configured in `rust-toolchain.toml`)
- [ESP-IDF Prerequisites](https://docs.espressif.com/projects/rust/book/getting-started/index.html)
- [ESP-IDF Template](https://github.com/esp-rs/esp-idf-template) (we follow the instructions from the template because we are NOT using 'no_std' environment)

## Setup & Configuration

1.  **Configure Environment Variables:**
    Copy the example configuration and fill in your details (WiFi credentials, MQTT broker).
    ```sh
    cp .env.example .env
    ```
    Edit `.env` with your actual values:
    ```ini
    WIFI_SSID="YourWiFiName"
    WIFI_PASS="YourWiFiPassword"
    MQTT_USERNAME="mqtt_user"
    MQTT_PASSWORD="mqtt_password"
    MQTT_HOST="192.168.1.100"
    MQTT_PORT="1883"
    ```

## Building & Running

1.  **Setup Environment (on startup of the project):**
    We need to setup the linker to compile for the right env
    You need to run this for every time you start the project.

    ```sh
    make init
    ```

2.  **Build the project:**
    This will compile the ESP-IDF framework and the application. The first run will take significantly longer as it builds the entire framework.

    ```sh
    make build
    ```

3.  **Flash and Monitor:**
    Connect your ESP32-C3 via USB and run:
    ```sh
    make dev
    ```
    This command will build, flash the firmware to the device, and open the serial monitor to see the logs.

## Resources

- [esp-idf-svc Documentation](https://github.com/esp-rs/esp-idf-svc)
- [ld2410s Driver Source](https://github.com/mvdschee/ld2410s)
- [Rust on ESP Book](https://docs.esp-rs.org/book/)
