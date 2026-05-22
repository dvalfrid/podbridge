//! Windows Bluetooth backend using the WinRT API via the `windows` crate.

use super::BluetoothBackend;
use crate::airpods::{AirPodsDevice, AirPodsModel, BatteryInfo, BluetoothAddress, NoiseControlMode};
use anyhow::{bail, Result};
use log::warn;

use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothConnectionStatus};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationCollection};

/// AQS filter that matches all paired Bluetooth (Classic) devices.
const PAIRED_BT_AQS: &str =
    "System.Devices.Aep.ProtocolId:=\"{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}\" \
     AND System.Devices.Aep.IsPaired:=System.StructuredQueryType.Boolean#True";

pub struct WindowsBluetoothBackend;

impl WindowsBluetoothBackend {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Enumerate paired BT classic devices and filter to AirPods by name prefix.
    fn enumerate_paired(&self) -> Result<Vec<AirPodsDevice>> {
        // WinRT calls are blocking here; we wrap in tokio::task::block_in_place
        // when called from async context.
        let selector = windows::core::HSTRING::from(PAIRED_BT_AQS);
        let devices: DeviceInformationCollection =
            DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()?;

        let mut result = Vec::new();
        for i in 0..devices.Size()? {
            let info = devices.GetAt(i)?;
            let id = info.Id()?.to_string();
            let name = info.Name()?.to_string();

            if !looks_like_airpods(&name) {
                continue;
            }

            let bt_device = BluetoothDevice::FromIdAsync(&windows::core::HSTRING::from(&id))?.get()?;
            let address = BluetoothAddress(bt_device.BluetoothAddress()?);
            let connected = bt_device.ConnectionStatus()? == BluetoothConnectionStatus::Connected;

            let model = model_from_name(&name);
            let mut device = AirPodsDevice::new(address, name, model);
            device.is_connected = connected;
            result.push(device);
        }
        Ok(result)
    }
}

impl BluetoothBackend for WindowsBluetoothBackend {
    fn paired_devices(&self) -> Result<Vec<AirPodsDevice>> {
        self.enumerate_paired()
    }

    fn get_battery(&self, address: &BluetoothAddress) -> Result<BatteryInfo> {
        // TODO: connect to AAP L2CAP socket and request device info.
        // Requires opening an RFCOMM/L2CAP channel — deferred to next milestone.
        warn!("Battery via AAP not yet implemented on Windows (address: {})", address);
        Ok(BatteryInfo::unknown())
    }

    fn connect(&self, address: &BluetoothAddress) -> Result<()> {
        // Triggering connection on Windows is done by accessing a service on the
        // device (e.g. opening an RFCOMM channel), not via a direct "connect" API.
        // Placeholder until we wire up the A2DP / HFP service open.
        warn!("connect() not yet implemented (address: {})", address);
        Ok(())
    }

    fn disconnect(&self, address: &BluetoothAddress) -> Result<()> {
        warn!("disconnect() not yet implemented (address: {})", address);
        Ok(())
    }

    fn set_noise_control(&self, address: &BluetoothAddress, _mode: NoiseControlMode) -> Result<()> {
        bail!("ANC control not yet implemented on Windows (address: {})", address)
    }
}

fn looks_like_airpods(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("airpods") || lower.contains("air pods")
}

/// Heuristic model detection from the device name until we have BLE parsing.
fn model_from_name(name: &str) -> AirPodsModel {
    let lower = name.to_lowercase();
    if lower.contains("max") {
        AirPodsModel::AirPodsMax1
    } else if lower.contains("airpods 4") || lower.contains("airpods4") {
        // Can't distinguish ANC vs basic from name alone — default to ANC variant
        AirPodsModel::AirPods4
    } else {
        AirPodsModel::Unknown
    }
}
