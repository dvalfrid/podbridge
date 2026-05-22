use crate::airpods::AirPodsModel;

/// Apple Bluetooth Company ID used in BLE advertisement packets.
pub const APPLE_COMPANY_ID: u16 = 0x004C;

/// Known Apple device type bytes (byte 0 of continuity payload).
pub const DEVICE_TYPE_AIRPODS: u8 = 0x07;
pub const DEVICE_TYPE_AIRPODS_MAX: u8 = 0x0A;

/// Maps a (device_type, model_id) pair from the BLE continuity payload to an
/// `AirPodsModel`. Returns `AirPodsModel::Unknown` for unrecognised values.
pub fn model_from_continuity(device_type: u8, model_id: u16) -> AirPodsModel {
    match (device_type, model_id) {
        (DEVICE_TYPE_AIRPODS_MAX, _) => AirPodsModel::AirPodsMax1,
        // AirPods 4 ANC: 0x2024, AirPods 4 basic: 0x2025 (values unconfirmed,
        // update once verified against hardware)
        (DEVICE_TYPE_AIRPODS, 0x2024) => AirPodsModel::AirPods4,
        (DEVICE_TYPE_AIRPODS, 0x2025) => AirPodsModel::AirPods4Basic,
        _ => AirPodsModel::Unknown,
    }
}
