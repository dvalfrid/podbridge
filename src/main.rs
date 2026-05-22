mod airpods;
mod bluetooth;
mod ui;

use anyhow::Result;
use log::{error, info};
use std::time::{Duration, Instant};

/// How often we refresh the tray from the BLE cache + paired-device list.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    info!("PodBridge starting");

    let bt = bluetooth::create_backend()?;
    let mut ui = ui::create_ui()?;
    let mut last_poll = Instant::now() - POLL_INTERVAL; // force immediate first poll

    loop {
        // On Windows, tray-icon requires Win32 messages to be pumped on the
        // main thread — menu clicks, icon redraws, etc. all go through the
        // message queue. pump_messages() drains it without blocking.
        #[cfg(windows)]
        pump_messages();

        // Handle UI events (menu clicks, quit, etc.)
        for event in ui.poll_events() {
            match event {
                ui::UiEvent::Quit => {
                    info!("Quit requested, exiting");
                    return Ok(());
                }
                ui::UiEvent::ConnectDevice { address } => {
                    if let Err(e) = bt.connect(&airpods::BluetoothAddress(address)) {
                        error!("Connect failed: {e}");
                    }
                }
                ui::UiEvent::DisconnectDevice { address } => {
                    if let Err(e) = bt.disconnect(&airpods::BluetoothAddress(address)) {
                        error!("Disconnect failed: {e}");
                    }
                }
                ui::UiEvent::SetNoiseMode { address, mode } => {
                    if let Err(e) = bt.set_noise_control(&airpods::BluetoothAddress(address), mode) {
                        error!("Set noise mode failed: {e}");
                    }
                }
            }
        }

        // Periodic Bluetooth poll
        if last_poll.elapsed() >= POLL_INTERVAL {
            match bt.paired_devices() {
                Ok(devices) => {
                    if let Err(e) = ui.update_devices(&devices) {
                        error!("UI update failed: {e}");
                    }
                    info!("Tray updated: {} device(s) found", devices.len());
                }
                Err(e) => error!("Bluetooth poll failed: {e}"),
            }
            last_poll = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Drain the Win32 message queue without blocking. Required for tray-icon to
/// render and deliver menu/click events on Windows.
#[cfg(windows)]
fn pump_messages() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
