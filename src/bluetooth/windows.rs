//! Windows Bluetooth backend using the WinRT API via the `windows` crate.

use super::BluetoothBackend;
use crate::airpods::{
    AirPodsDevice, AirPodsModel, BatteryInfo, BluetoothAddress, NoiseControlMode,
};
use crate::airpods::models::{
    parse_proximity_pairing, model_from_continuity, APPLE_COMPANY_ID,
};
use anyhow::{bail, Result};
use log::{debug, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothConnectionStatus};
use windows::Devices::Bluetooth::Advertisement::{
    BluetoothLEAdvertisementReceivedEventArgs,
    BluetoothLEAdvertisementWatcher,
    BluetoothLEScanningMode,
};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationCollection};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::DataReader;

/// AQS filter that matches all paired Bluetooth Classic devices.
const PAIRED_BT_AQS: &str =
    "System.Devices.Aep.ProtocolId:=\"{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}\" \
     AND System.Devices.Aep.IsPaired:=System.StructuredQueryType.Boolean#True";

/// Battery info observed from BLE advertisements, keyed by Bluetooth address.
type BleCache = Arc<Mutex<HashMap<u64, BatteryInfo>>>;

pub struct WindowsBluetoothBackend {
    ble_cache: BleCache,
    // Held alive for the lifetime of the backend; dropping stops the watcher.
    _watcher: BluetoothLEAdvertisementWatcher,
}

impl WindowsBluetoothBackend {
    pub fn new() -> Result<Self> {
        let ble_cache: BleCache = Arc::new(Mutex::new(HashMap::new()));
        let watcher = start_ble_watcher(ble_cache.clone())?;
        Ok(Self { ble_cache, _watcher: watcher })
    }

    fn enumerate_paired(&self) -> Result<Vec<AirPodsDevice>> {
        let selector = windows::core::HSTRING::from(PAIRED_BT_AQS);
        let devices: DeviceInformationCollection =
            DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()?;

        let cache = self.ble_cache.lock().unwrap();
        let mut result = Vec::new();

        for i in 0..devices.Size()? {
            let info = devices.GetAt(i)?;
            let id = info.Id()?.to_string();
            let name = info.Name()?.to_string();

            if !looks_like_airpods(&name) {
                continue;
            }

            let bt_device =
                BluetoothDevice::FromIdAsync(&windows::core::HSTRING::from(&id))?.get()?;
            let addr = bt_device.BluetoothAddress()?;
            let connected =
                bt_device.ConnectionStatus()? == BluetoothConnectionStatus::Connected;

            let model = model_from_name(&name);
            let mut device = AirPodsDevice::new(BluetoothAddress(addr), name, model);
            device.is_connected = connected;

            // Enrich with BLE-observed battery data if available.
            if let Some(battery) = cache.get(&addr) {
                device.battery = battery.clone();
            }

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
        let cache = self.ble_cache.lock().unwrap();
        Ok(cache.get(&address.0).cloned().unwrap_or_else(BatteryInfo::unknown))
    }

    fn connect(&self, address: &BluetoothAddress) -> Result<()> {
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

/// Start a passive BLE advertisement watcher that listens for Apple Proximity
/// Pairing packets and writes parsed battery info into `cache`.
fn start_ble_watcher(cache: BleCache) -> Result<BluetoothLEAdvertisementWatcher> {
    let watcher = BluetoothLEAdvertisementWatcher::new()?;
    // Passive scanning avoids sending scan requests — AirPods broadcast
    // Proximity Pairing unsolicited, so passive is sufficient and less disruptive.
    watcher.SetScanningMode(BluetoothLEScanningMode::Passive)?;

    let handler = TypedEventHandler::new(
        move |_watcher: &Option<BluetoothLEAdvertisementWatcher>,
              args: &Option<BluetoothLEAdvertisementReceivedEventArgs>| {
            if let Some(args) = args {
                handle_advertisement(args, &cache);
            }
            Ok(())
        },
    );
    watcher.Received(&handler)?;
    watcher.Start()?;
    debug!("BLE advertisement watcher started");
    Ok(watcher)
}

fn handle_advertisement(
    args: &BluetoothLEAdvertisementReceivedEventArgs,
    cache: &BleCache,
) {
    let addr = match args.BluetoothAddress() {
        Ok(a) => a,
        Err(_) => return,
    };
    let adv = match args.Advertisement() {
        Ok(a) => a,
        Err(_) => return,
    };
    let sections = match adv.ManufacturerData() {
        Ok(s) => s,
        Err(_) => return,
    };

    let count = match sections.Size() {
        Ok(n) => n,
        Err(_) => return,
    };

    for i in 0..count {
        let section = match sections.GetAt(i) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let company_id = match section.CompanyId() {
            Ok(id) => id,
            Err(_) => continue,
        };
        if company_id != APPLE_COMPANY_ID {
            continue;
        }

        let buf = match section.Data() {
            Ok(b) => b,
            Err(_) => continue,
        };
        let len = match buf.Length() {
            Ok(l) => l as usize,
            Err(_) => continue,
        };
        if len < 8 {
            continue;
        }

        let reader = match DataReader::FromBuffer(&buf) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let mut data = vec![0u8; len];
        if reader.ReadBytes(&mut data).is_err() {
            continue;
        }

        if let Some(battery) = parse_proximity_pairing(&data) {
            debug!("BLE battery update addr={:012X} L={} R={} Case={}",
                addr, battery.left, battery.right, battery.case);
            // Also detect model from the advertisement while we're here.
            let _ = model_from_continuity(data[0], u16::from_be_bytes([data[3], data[4]]));
            cache.lock().unwrap().insert(addr, battery);
        }
    }
}

fn looks_like_airpods(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("airpods") || lower.contains("air pods")
}

fn model_from_name(name: &str) -> AirPodsModel {
    let lower = name.to_lowercase();
    if lower.contains("max") {
        AirPodsModel::AirPodsMax1
    } else if lower.contains("airpods 4") || lower.contains("airpods4") {
        AirPodsModel::AirPods4
    } else {
        AirPodsModel::Unknown
    }
}
