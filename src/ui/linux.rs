//! Linux tray implementation using the `tray-icon` crate (via GTK backend).

use super::{UiBackend, UiEvent};
use crate::airpods::AirPodsDevice;
use crate::ui::tray;
use anyhow::Result;
use log::debug;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};

pub struct LinuxTray {
    _tray: TrayIcon,
    quit_id: tray_icon::menu::MenuId,
}

impl LinuxTray {
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
            .build()?;

        Ok(Self { _tray: tray, quit_id })
    }
}

impl UiBackend for LinuxTray {
    fn update_devices(&mut self, devices: &[AirPodsDevice]) -> Result<()> {
        let tooltip = tray::build_tooltip(devices);
        self._tray.set_tooltip(Some(tooltip))?;
        debug!("Tray updated with {} device(s)", devices.len());
        Ok(())
    }

    fn show_notification(&self, title: &str, body: &str) -> Result<()> {
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
