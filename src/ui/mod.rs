pub mod tray;

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::airpods::AirPodsDevice;
use anyhow::Result;

/// Events the UI layer emits back to the application logic.
#[derive(Debug)]
pub enum UiEvent {
    ConnectDevice { address: u64 },
    DisconnectDevice { address: u64 },
    SetNoiseMode { address: u64, mode: crate::airpods::NoiseControlMode },
    Quit,
}

/// Platform-independent UI interface.
/// Note: tray-icon is not Send (GUI must stay on the main thread), so this
/// trait does not require Send. Keep UI on the main thread.
pub trait UiBackend {
    /// Update the tray icon/menu with current device state.
    fn update_devices(&mut self, devices: &[AirPodsDevice]) -> Result<()>;

    /// Show a brief notification (battery low, connection change, etc.).
    fn show_notification(&self, title: &str, body: &str) -> Result<()>;

    /// Process pending UI events; returns any events triggered since last call.
    /// Non-blocking — returns immediately if nothing pending.
    fn poll_events(&mut self) -> Vec<UiEvent>;
}

pub fn create_ui() -> Result<Box<dyn UiBackend>> {
    #[cfg(windows)]
    {
        Ok(Box::new(windows::WindowsTray::new()?))
    }
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxTray::new()?))
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        anyhow::bail!("Unsupported platform")
    }
}
