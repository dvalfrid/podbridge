# PodBridge — Claude context

## What this project is

A Rust tray application that provides AirPods support on Windows and Linux: battery levels in the system tray, faster pairing, and (if possible) ANC control. It communicates via Apple's proprietary AAP protocol (L2CAP PSM 0x1001), which has been reverse-engineered by the community.

Target user: Daniel, who has AirPods Max (gen 1) and AirPods 4 (with ANC), primarily on Windows (ASUS ROG G14) but also Linux.

## Supported devices

| Model | Batteries | ANC |
| --- | --- | --- |
| AirPods Max (gen 1, USB-C) | 1 (headset) | Yes |
| AirPods 4 (with ANC) | 3 (L/R/case) | Yes |
| AirPods 4 (basic) | 3 (L/R/case) | No |

To add more models, only change `model_from_continuity()` in `src/airpods/models.rs`.

## Architecture

```
src/
├── main.rs              Main loop: 30s poll cycle + UI event handling
├── airpods/
│   ├── mod.rs           Shared types (AirPodsDevice, BatteryInfo, NoiseControlMode, AirPodsModel)
│   ├── models.rs        BLE continuity payload → AirPodsModel mapping
│   └── aap.rs           AAP protocol: packet parsing and building (L2CAP PSM 0x1001)
├── bluetooth/
│   ├── mod.rs           BluetoothBackend trait + platform factory create_backend()
│   ├── scanner.rs       BLE advertisement scanning → model detection
│   ├── windows.rs       WinRT backend (windows-crate)
│   └── linux.rs         BlueZ backend (bluer-crate)
└── ui/
    ├── mod.rs           UiBackend trait + factory create_ui()
    ├── tray.rs          Shared tray label/tooltip helpers
    ├── windows.rs       Windows tray (tray-icon crate)
    └── linux.rs         Linux tray (tray-icon crate, GTK backend)
```

## Tech stack

- **Rust 2021**, tokio (async runtime)
- **windows-crate 0.58** — WinRT Bluetooth API for Windows
- **bluer 0.17** — BlueZ bindings for Linux
- **tray-icon 0.19** — system tray icon, cross-platform
- **anyhow** — error handling
- **serde** — serialization (for config, later)
- **env_logger** — logging (control with `RUST_LOG=debug`)

## Current state

### Implemented

- [x] All shared types and traits (`AirPodsDevice`, `BatteryInfo`, `BluetoothBackend`, `UiBackend`)
- [x] AAP protocol helpers (packet parsing, command building)
- [x] BLE advertisement scanning with model detection
- [x] Windows: paired device enumeration via WinRT
- [x] Windows and Linux tray with quit menu
- [x] Main loop with 30s poll cycle

### Stubs (not yet implemented)

- [ ] Windows: battery reading via AAP over L2CAP
- [ ] Windows: connect / disconnect
- [ ] Windows: ANC control
- [ ] Linux: all Bluetooth functionality (bluer async)
- [ ] Tray icon image (no PNG/ICO asset yet)
- [ ] Toast notifications on Windows

## Next milestone: battery display on Windows (MVP)

1. **L2CAP socket via WinRT** — open a channel to PSM 0x1001 on a paired device
2. **Send AAP device-info request** — see `src/airpods/aap.rs:build_device_info_request()`
3. **Parse the response** — see `src/airpods/aap.rs:parse_battery_packet()`
4. **Display in tray** — `WindowsTray::update_devices()` is already wired up
5. **Tray icon image** — create a simple 32x32 PNG and load it in `WindowsTray::new()`

## Protocol references

- [LibrePods](https://github.com/nickcoutsos/LibrePods) — AAP reverse engineering
- [tyalie/AAP-Protocol-Definition](https://github.com/nickcoutsos/AAP-Protocol-Definition) — protocol definitions
- [mstroecker/LinuxPods](https://github.com/nickcoutsos/LinuxPods) — Linux implementation to study
- AAP runs over L2CAP, PSM `0x1001`
- Battery packet: type `0x04`, bytes 2–4 = L/R/case (0–10 × 10 = %), byte 5 bit 0 = charging
- Model IDs in `models.rs` are preliminary — verify against real hardware

## Running

```powershell
cargo run                   # debug build
$env:RUST_LOG="debug"; cargo run   # with verbose logging
cargo check                 # fast compile check
```

## Code guidelines

- Minimal comments — only when the *why* is non-obvious
- Stubs return `Ok(...)` with `warn!()` — don't panic, just log
- Platform-specific code behind `#[cfg(windows)]` / `#[cfg(target_os = "linux")]`
- Add new AirPods models only in `models.rs:model_from_continuity()`
