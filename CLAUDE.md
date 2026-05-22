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
├── main.rs              Main loop: 5s poll cycle + UI event handling
├── airpods/
│   ├── mod.rs           Shared types (AirPodsDevice, BatteryInfo, NoiseControlMode, AirPodsModel)
│   ├── models.rs        BLE Proximity Pairing parser + AirPodsModel mapping
│   └── aap.rs           AAP protocol: packet parsing and building (L2CAP PSM 0x1001)
├── bluetooth/
│   ├── mod.rs           BluetoothBackend trait + platform factory create_backend()
│   ├── scanner.rs       BLE advertisement scanning → model detection (helper)
│   ├── windows.rs       WinRT backend: paired device enumeration + BLE watcher
│   └── linux.rs         BlueZ backend (bluer-crate, stubs)
└── ui/
    ├── mod.rs           UiBackend trait + factory create_ui()
    ├── tray.rs          Shared tray label/tooltip helpers
    ├── windows.rs       Windows tray with programmatic placeholder icon
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
- [x] AAP protocol helpers (packet parsing, command building — for future L2CAP use)
- [x] Apple Proximity Pairing BLE advertisement parser (`models.rs:parse_proximity_pairing`)
- [x] Windows: passive BLE advertisement watcher (`BluetoothLEAdvertisementWatcher`)
- [x] Windows: paired device enumeration via WinRT, enriched with BLE battery cache
- [x] Windows tray with programmatic placeholder icon (32×32 white circle)
- [x] Linux tray stub
- [x] Main loop with 5s tray refresh cycle

### Stubs (not yet implemented)

- [ ] Windows: connect / disconnect
- [ ] Windows: ANC control (via AAP L2CAP)
- [ ] Linux: all Bluetooth functionality (bluer async)
- [ ] Real tray icon asset (replace placeholder circle)
- [ ] Toast notifications on Windows

## Battery approach — how it works

Battery data comes from **passive BLE advertisement scanning**, not an active connection.
AirPods broadcast Apple Proximity Pairing packets (manufacturer data, company ID `0x004C`)
containing battery levels whenever they are on and in range.

- `WindowsBluetoothBackend` starts a `BluetoothLEAdvertisementWatcher` on init
- Each received Apple ad is parsed by `parse_proximity_pairing()` in `models.rs`
- Results are stored in a thread-safe `Arc<Mutex<HashMap<u64, BatteryInfo>>>` cache
- `paired_devices()` enriches each WinRT-enumerated device with its cached battery data
- Tray refreshes every 5 s, which re-reads the cache

## Next milestone: verify on real hardware

1. **Run with `RUST_LOG=debug`** and watch for `BLE battery update` log lines
2. **Model ID verification** — `model_from_continuity()` in `models.rs` has placeholder IDs
   for AirPods 4; update if the debug log shows unexpected device types
3. **Battery nibble mapping** — `parse_proximity_pairing()` uses community-documented layout;
   confirm L/R assignment matches physical pods (swap nibbles in `models.rs` if needed)
4. **ANC control** — next major feature; requires AAP over L2CAP (`aap.rs` has the packet
   builders ready, needs a WinRT socket implementation)

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
