mod airpods;
mod bluetooth;
mod ui;

use anyhow::Result;
use log::{error, info};
use std::time::{Duration, Instant};

/// How often we poll Bluetooth for updated device/battery state.
const POLL_INTERVAL: Duration = Duration::from_secs(30);

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
                }
                Err(e) => error!("Bluetooth poll failed: {e}"),
            }
            last_poll = Instant::now();
        }

        // Yield briefly to avoid busy-looping.
        std::thread::sleep(Duration::from_millis(100));
    }
}
