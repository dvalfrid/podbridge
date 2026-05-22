//! Linux Bluetooth backend using BlueZ via the `bluer` crate.

use super::BluetoothBackend;
use crate::airpods::{AirPodsDevice, AirPodsModel, BatteryInfo, BluetoothAddress, NoiseControlMode};
use anyhow::{bail, Result};
use log::warn;

pub struct LinuxBluetoothBackend;

impl LinuxBluetoothBackend {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl BluetoothBackend for LinuxBluetoothBackend {
    fn paired_devices(&self) -> Result<Vec<AirPodsDevice>> {
        // TODO: use bluer::Session to enumerate paired devices.
        // bluer is async; we'll need a tokio runtime handle here.
        warn!("paired_devices() not yet implemented on Linux");
        Ok(vec![])
    }

    fn get_battery(&self, address: &BluetoothAddress) -> Result<BatteryInfo> {
        warn!("get_battery() not yet implemented on Linux (address: {})", address);
        Ok(BatteryInfo::unknown())
    }

    fn connect(&self, address: &BluetoothAddress) -> Result<()> {
        warn!("connect() not yet implemented on Linux (address: {})", address);
        Ok(())
    }

    fn disconnect(&self, address: &BluetoothAddress) -> Result<()> {
        warn!("disconnect() not yet implemented on Linux (address: {})", address);
        Ok(())
    }

    fn set_noise_control(&self, address: &BluetoothAddress, mode: NoiseControlMode) -> Result<()> {
        bail!("ANC control not yet implemented on Linux (address: {})", address)
    }
}
