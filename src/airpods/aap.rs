//! Partial implementation of Apple's AAP (Apple Audio Protocol) over L2CAP.
//!
//! Reference: https://github.com/JackHMcD/AAP-Protocol-Definition
//! PSM used by AAP: 0x1001

use crate::airpods::{BatteryInfo, BatteryLevel, NoiseControlMode};
use anyhow::Result;

pub const AAP_L2CAP_PSM: u16 = 0x1001;

/// Packet type byte (first byte of every AAP message).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    /// 0x00 — handshake / connection init
    Handshake = 0x00,
    /// 0x04 — device info response (includes battery)
    DeviceInfo = 0x04,
    /// 0x09 — noise control command / status
    NoiseControl = 0x09,
}

/// Parse a raw AAP packet and extract battery information.
///
/// Returns `None` if the packet is not a DeviceInfo packet or is malformed.
pub fn parse_battery_packet(data: &[u8]) -> Option<BatteryInfo> {
    if data.len() < 6 {
        return None;
    }
    // Byte layout (reverse-engineered, subject to model variance):
    //   [0]     packet type  (must be 0x04)
    //   [1]     flags
    //   [2]     left battery  (0–10, multiply by 10 for %)
    //   [3]     right battery (0–10)
    //   [4]     case battery  (0–10)
    //   [5]     status flags  (bit 0 = charging)
    if data[0] != PacketType::DeviceInfo as u8 {
        return None;
    }
    let left = decode_battery(data[2]);
    let right = decode_battery(data[3]);
    let case = decode_battery(data[4]);
    let is_charging = data[5] & 0x01 != 0;

    Some(BatteryInfo { left, right, case, is_charging })
}

fn decode_battery(raw: u8) -> BatteryLevel {
    // Values 0–10 map to 0–100%; 0xFF means unavailable.
    if raw == 0xFF || raw > 10 {
        BatteryLevel(None)
    } else {
        BatteryLevel(Some(raw * 10))
    }
}

/// Parse an AAP noise-control status packet.
pub fn parse_noise_control_packet(data: &[u8]) -> Option<NoiseControlMode> {
    if data.len() < 2 || data[0] != PacketType::NoiseControl as u8 {
        return None;
    }
    match data[1] {
        0x01 => Some(NoiseControlMode::Off),
        0x02 => Some(NoiseControlMode::NoiseCancellation),
        0x03 => Some(NoiseControlMode::Transparency),
        0x04 => Some(NoiseControlMode::Adaptive),
        _ => None,
    }
}

/// Build an AAP command that requests the current device info.
pub fn build_device_info_request() -> Vec<u8> {
    // Minimal handshake / info-request packet — exact format TBD once tested
    // against hardware; this is a placeholder based on community docs.
    vec![PacketType::Handshake as u8, 0x00, 0x01, 0x00]
}

/// Build an AAP command to set noise control mode.
pub fn build_set_noise_control(mode: NoiseControlMode) -> Result<Vec<u8>> {
    let mode_byte = match mode {
        NoiseControlMode::Off => 0x01u8,
        NoiseControlMode::NoiseCancellation => 0x02,
        NoiseControlMode::Transparency => 0x03,
        NoiseControlMode::Adaptive => 0x04,
    };
    Ok(vec![PacketType::NoiseControl as u8, mode_byte])
}
