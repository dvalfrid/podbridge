# PodBridge

Cross-platform AirPods manager for Windows and Linux — battery levels in the system tray, faster pairing, and ANC control without unsigned drivers.

## Status

Early development. Windows tray skeleton compiles and enumerates paired devices. Battery reading and ANC control are not yet implemented.

## Supported devices

| Model | Battery | ANC |
| --- | --- | --- |
| AirPods Max (gen 1, USB-C) | Single | Yes |
| AirPods 4 (with ANC) | Left / Right / Case | Yes |
| AirPods 4 (basic) | Left / Right / Case | No |

## Features (planned)

- Battery level in system tray (Windows + Linux)
- Faster Bluetooth pairing
- ANC / Transparency mode toggle
- Simple tray menu — no bloat

## Building

```sh
cargo build
cargo run
```

Requires Rust 1.75+. On Windows, the WinRT Bluetooth APIs are used directly — no extra SDK needed. On Linux, BlueZ must be installed (`bluez` package).

## How it works

AirPods communicate over Bluetooth using Apple's proprietary AAP protocol (L2CAP, PSM 0x1001), reverse-engineered by the community. PodBridge uses this to read battery levels and control noise modes without needing unsigned drivers.

References: [LibrePods](https://github.com/nickcoutsos/LibrePods), [AAP-Protocol-Definition](https://github.com/nickcoutsos/AAP-Protocol-Definition)

## License

MIT
