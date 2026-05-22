use crate::airpods::{AirPodsDevice, BluetoothAddress};
use crate::airpods::models::model_from_continuity;
use log::debug;

/// Minimal BLE advertisement data needed for model detection.
pub struct AdvertisementData {
    pub address: BluetoothAddress,
    pub name: Option<String>,
    /// Raw manufacturer-specific data payload (excludes the 2-byte company ID).
    pub manufacturer_data: Option<Vec<u8>>,
}

/// Try to identify an AirPods device from a BLE advertisement.
///
/// Returns `None` if the advertisement is not from an AirPods device.
pub fn identify_from_advertisement(adv: &AdvertisementData) -> Option<AirPodsDevice> {
    let data = adv.manufacturer_data.as_deref()?;

    // Apple continuity payload: [company_id_lo, company_id_hi, device_type, ...]
    // By the time we receive manufacturer data the 2-byte company ID is already
    // stripped, so data[0] = device_type, data[1..2] = model_id.
    if data.len() < 3 {
        return None;
    }

    let device_type = data[0];
    let model_id = u16::from_be_bytes([data[1], data[2]]);
    let model = model_from_continuity(device_type, model_id);

    if model == crate::airpods::AirPodsModel::Unknown {
        debug!(
            "Unknown Apple device: type=0x{:02X} model=0x{:04X}",
            device_type, model_id
        );
        return None;
    }

    let name = adv.name.clone().unwrap_or_else(|| model.display_name().to_string());
    Some(AirPodsDevice::new(adv.address.clone(), name, model))
}
