//! Windows tray implementation using the `tray-icon` crate.

use super::{UiBackend, UiEvent};
use crate::airpods::AirPodsDevice;
use crate::ui::tray;
use anyhow::Result;
use log::debug;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

pub struct WindowsTray {
    _tray: TrayIcon,
    quit_id: tray_icon::menu::MenuId,
}

impl WindowsTray {
    pub fn new() -> Result<Self> {
        let menu = Menu::new();
        let quit_item = MenuItem::new("Quit", true, None);
        let quit_id = quit_item.id().clone();
        menu.append_items(&[
            &MenuItem::new("PodBridge", false, None),
            &PredefinedMenuItem::separator(),
            &quit_item,
        ])?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("PodBridge — no devices")
            .with_icon(placeholder_icon()?)
            .build()?;

        Ok(Self { _tray: tray, quit_id })
    }
}

/// Generate a simple 32×32 white circle on transparent background as a
/// placeholder until a real icon asset is added.
fn placeholder_icon() -> Result<Icon> {
    const SIZE: u32 = 32;
    const RADIUS: f32 = 14.0;
    const CENTER: f32 = 15.5;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - CENTER;
            let dy = y as f32 - CENTER;
            let idx = ((y * SIZE + x) * 4) as usize;
            if dx * dx + dy * dy <= RADIUS * RADIUS {
                rgba[idx] = 255;     // R
                rgba[idx + 1] = 255; // G
                rgba[idx + 2] = 255; // B
                rgba[idx + 3] = 255; // A
            }
        }
    }
    Ok(Icon::from_rgba(rgba, SIZE, SIZE)?)
}

impl UiBackend for WindowsTray {
    fn update_devices(&mut self, devices: &[AirPodsDevice]) -> Result<()> {
        let tooltip = tray::build_tooltip(devices);
        self._tray.set_tooltip(Some(tooltip))?;
        debug!("Tray updated with {} device(s)", devices.len());
        Ok(())
    }

    fn show_notification(&self, title: &str, body: &str) -> Result<()> {
        // Windows 10/11 toast notifications via winrt would go here.
        // For now just log; a proper implementation can use the `windows` crate
        // ToastNotification API.
        log::info!("Notification — {}: {}", title, body);
        Ok(())
    }

    fn poll_events(&mut self) -> Vec<UiEvent> {
        let mut events = Vec::new();
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_id {
                events.push(UiEvent::Quit);
            }
        }
        events
    }
}
