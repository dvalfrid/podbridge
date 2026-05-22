pub mod aap;
pub mod models;

use serde::{Deserialize, Serialize};

/// Unique Bluetooth address for a device.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BluetoothAddress(pub u64);

impl std::fmt::Display for BluetoothAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = self.0.to_be_bytes();
        write!(f, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            b[2], b[3], b[4], b[5], b[6], b[7])
    }
}

/// Battery level, 0–100. `None` means unknown/unavailable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryLevel(pub Option<u8>);

impl std::fmt::Display for BatteryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(v) => write!(f, "{}%", v),
            None => write!(f, "?"),
        }
    }
}

/// Battery info for a device. AirPods Max has only `single`; AirPods 4 uses
/// `left`, `right`, and `case`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    /// Left earbud (AirPods 4) or headphone (AirPods Max).
    pub left: BatteryLevel,
    /// Right earbud (AirPods 4). Not used for AirPods Max.
    pub right: BatteryLevel,
    /// Charging case (AirPods 4). Not used for AirPods Max.
    pub case: BatteryLevel,
    /// Whether the device is currently charging.
    pub is_charging: bool,
}

impl BatteryInfo {
    pub fn unknown() -> Self {
        Self {
            left: BatteryLevel(None),
            right: BatteryLevel(None),
            case: BatteryLevel(None),
            is_charging: false,
        }
    }
}

/// Noise control mode supported by ANC-capable AirPods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseControlMode {
    Off,
    NoiseCancellation,
    Transparency,
    Adaptive, // AirPods Pro 2 / AirPods 4 with ANC
}

impl std::fmt::Display for NoiseControlMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::NoiseCancellation => write!(f, "ANC"),
            Self::Transparency => write!(f, "Transparency"),
            Self::Adaptive => write!(f, "Adaptive"),
        }
    }
}

/// Identifies which AirPods model this is — drives capability detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AirPodsModel {
    AirPodsMax1,   // 1st gen, Lightning + USB-C variants
    AirPods4,      // with ANC
    AirPods4Basic, // without ANC
    Unknown,
}

impl AirPodsModel {
    pub fn has_anc(&self) -> bool {
        matches!(self, Self::AirPodsMax1 | Self::AirPods4)
    }

    pub fn has_case_battery(&self) -> bool {
        matches!(self, Self::AirPods4 | Self::AirPods4Basic)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AirPodsMax1 => "AirPods Max",
            Self::AirPods4 => "AirPods 4 (ANC)",
            Self::AirPods4Basic => "AirPods 4",
            Self::Unknown => "Unknown AirPods",
        }
    }
}

/// A discovered or paired AirPods device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirPodsDevice {
    pub address: BluetoothAddress,
    pub name: String,
    pub model: AirPodsModel,
    pub battery: BatteryInfo,
    pub noise_mode: Option<NoiseControlMode>,
    pub is_connected: bool,
}

impl AirPodsDevice {
    pub fn new(address: BluetoothAddress, name: String, model: AirPodsModel) -> Self {
        Self {
            address,
            name,
            model,
            battery: BatteryInfo::unknown(),
            noise_mode: None,
            is_connected: false,
        }
    }
}
