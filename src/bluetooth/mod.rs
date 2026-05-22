pub mod scanner;

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::airpods::{AirPodsDevice, BatteryInfo, BluetoothAddress, NoiseControlMode};
use anyhow::Result;

/// Platform-independent Bluetooth operations.
///
/// Each platform provides one concrete implementation. The trait uses `async_trait`
/// semantics via Tokio — implementors should use `async fn` in Rust 2024, or add
/// `#[async_trait]` if targeting older editions.
pub trait BluetoothBackend: Send + Sync {
    /// Return all currently paired/connected AirPods devices.
    fn paired_devices(&self) -> Result<Vec<AirPodsDevice>>;

    /// Refresh battery info for a specific device. Returns the updated info.
    fn get_battery(&self, address: &BluetoothAddress) -> Result<BatteryInfo>;

    /// Connect to a device (make it the active audio output).
    fn connect(&self, address: &BluetoothAddress) -> Result<()>;

    /// Disconnect a device.
    fn disconnect(&self, address: &BluetoothAddress) -> Result<()>;

    /// Set noise control mode. Returns `Err` if the device doesn't support ANC
    /// or if the AAP command fails.
    fn set_noise_control(
        &self,
        address: &BluetoothAddress,
        mode: NoiseControlMode,
    ) -> Result<()>;
}

/// Construct the correct backend for the current platform.
pub fn create_backend() -> Result<Box<dyn BluetoothBackend>> {
    #[cfg(windows)]
    {
        Ok(Box::new(windows::WindowsBluetoothBackend::new()?))
    }
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxBluetoothBackend::new()?))
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        anyhow::bail!("Unsupported platform")
    }
}
