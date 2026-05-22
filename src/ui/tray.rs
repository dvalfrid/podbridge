//! Shared tray-icon helpers used by both platform backends.

use crate::airpods::{AirPodsDevice, BatteryInfo};

/// Build the tray tooltip string shown on hover.
pub fn build_tooltip(devices: &[AirPodsDevice]) -> String {
    if devices.is_empty() {
        return "PodBridge — no devices".to_string();
    }
    let parts: Vec<String> = devices.iter().map(device_summary).collect();
    format!("PodBridge\n{}", parts.join("\n"))
}

/// Build the menu label for a single device.
pub fn device_summary(device: &AirPodsDevice) -> String {
    let battery = battery_label(&device.battery, device.model.has_case_battery());
    let status = if device.is_connected { "Connected" } else { "Disconnected" };
    format!("{} — {} [{}]", device.name, battery, status)
}

fn battery_label(info: &BatteryInfo, has_case: bool) -> String {
    if has_case {
        format!(
            "L:{} R:{} Case:{}{}",
            info.left,
            info.right,
            info.case,
            if info.is_charging { " ⚡" } else { "" }
        )
    } else {
        format!(
            "{}{}",
            info.left,
            if info.is_charging { " ⚡" } else { "" }
        )
    }
}
