use crate::airpods::{AirPodsModel, BatteryInfo, BatteryLevel};

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
        // AirPods 4 ANC: 0x2024, AirPods 4 basic: 0x2025 (values unconfirmed —
        // verify against real hardware and update if needed)
        (DEVICE_TYPE_AIRPODS, 0x2024) => AirPodsModel::AirPods4,
        (DEVICE_TYPE_AIRPODS, 0x2025) => AirPodsModel::AirPods4Basic,
        _ => AirPodsModel::Unknown,
    }
}

/// Parsed battery and charging state from an Apple Proximity Pairing
/// BLE advertisement (continuity message type 0x07 / 0x0A).
///
/// Layout of the manufacturer-specific data payload (company ID already
/// stripped by the BLE stack, so index 0 is the continuity message type):
///
/// ```text
/// [0]  device type   0x07 = AirPods/Pro, 0x0A = AirPods Max
/// [1]  length        0x13 (19) for standard Proximity Pairing
/// [2]  status flags  bits described below
/// [3]  device ID byte
/// [4]  color
/// [5]  suffix
/// [6]  battery A     high nibble = right pod (0–10, 0xF = N/A)
///                    low  nibble = left  pod (0–10, 0xF = N/A)
/// [7]  battery B     high nibble = case  (0–10, 0xF = N/A)
///                    bit 3 = right charging, bit 2 = left charging,
///                    bit 1 = case charging
/// ```
///
/// Note: which pod maps to "left" vs "right" in byte 6 depends on which bud
/// is the primary. Bit 1 of byte 2 indicates left-primary (1) or
/// right-primary (0). We normalise here so `left`/`right` always match the
/// physical pod regardless of primary assignment.
pub fn parse_proximity_pairing(data: &[u8]) -> Option<BatteryInfo> {
    if data.len() < 8 {
        return None;
    }
    if data[0] != DEVICE_TYPE_AIRPODS && data[0] != DEVICE_TYPE_AIRPODS_MAX {
        return None;
    }

    let status = data[2];
    // bit 1 set → left bud is primary, stored in high nibble of byte 6
    let left_primary = status & 0x02 != 0;

    let nibble_a = (data[6] >> 4) & 0x0F; // high nibble of byte 6
    let nibble_b = data[6] & 0x0F;         // low  nibble of byte 6
    let nibble_case = (data[7] >> 4) & 0x0F;

    let (left_raw, right_raw) = if left_primary {
        (nibble_a, nibble_b)
    } else {
        (nibble_b, nibble_a)
    };

    let charging_byte = data[7] & 0x0F;
    let right_charging = charging_byte & 0x08 != 0;
    let left_charging  = charging_byte & 0x04 != 0;
    let case_charging  = charging_byte & 0x02 != 0;

    let is_charging = left_charging || right_charging || case_charging;

    Some(BatteryInfo {
        left:  nibble_to_battery(left_raw),
        right: nibble_to_battery(right_raw),
        case:  nibble_to_battery(nibble_case),
        is_charging,
    })
}

fn nibble_to_battery(n: u8) -> BatteryLevel {
    // 0–10 → 0–100 %, 0xF means unavailable
    if n == 0xF { BatteryLevel(None) } else { BatteryLevel(Some(n * 10)) }
}
