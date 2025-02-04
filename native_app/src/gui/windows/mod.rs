use egui::{Context, Ui};
use serde::{Deserialize, Serialize};

use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;
use egregoria::Egregoria;

mod config;
pub(crate) mod debug;
mod economy;
pub(crate) mod load;
#[cfg(feature = "multiplayer")]
pub(crate) mod network;
pub(crate) mod settings;

pub(crate) trait GUIWindow: Send + Sync {
    fn render_window(
        &mut self,
        window: egui::Window<'_>,
        ui: &Context,
        uiworld: &mut UiWorld,
        goria: &Egregoria,
    );
}

impl<F> GUIWindow for F
where
    F: Fn(egui::Window<'_>, &Context, &mut UiWorld, &Egregoria) + Send + Sync,
{
    fn render_window(
        &mut self,
        window: egui::Window<'_>,
        ui: &Context,
        uiworld: &mut UiWorld,
        goria: &Egregoria,
    ) {
        self(window, ui, uiworld, goria);
    }
}

struct GUIWindowStruct {
    w: Box<dyn GUIWindow>,
    name: &'static str,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct GUIWindows {
    #[serde(skip)]
    windows: Vec<GUIWindowStruct>,
    opened: Vec<bool>,
}

impl Default for GUIWindows {
    fn default() -> Self {
        let mut s = Self {
            windows: vec![],
            opened: vec![],
        };
        s.insert("Economy", economy::economy, false);
        s.insert("Config", config::config, false);
        s.insert("Debug", debug::debug, false);
        s.insert("Settings", settings::settings, false);
        #[cfg(feature = "multiplayer")]
        s.insert("Network", network::network, false);
        s.insert("Load", load::load, false);
        s
    }
}

impl GUIWindows {
    pub(crate) fn insert(&mut self, name: &'static str, w: impl GUIWindow + 'static, opened: bool) {
        self.windows.push(GUIWindowStruct {
            w: Box::new(w),
            name,
        });
        if self.opened.len() < self.windows.len() {
            self.opened.push(opened)
        }
    }

    pub(crate) fn menu(&mut self, ui: &mut Ui) {
        if self.opened.len() < self.windows.len() {
            self.opened
                .extend(std::iter::repeat(false).take(self.windows.len() - self.opened.len()))
        }
        for (opened, w) in self.opened.iter_mut().zip(self.windows.iter()) {
            *opened ^= ui.selectable_label(*opened, w.name).clicked();
        }
    }

    pub(crate) fn render(&mut self, ui: &Context, uiworld: &mut UiWorld, goria: &Egregoria) {
        if uiworld
            .write::<InputMap>()
            .just_act
            .contains(&InputAction::OpenEconomyMenu)
        {
            for (i, w) in self.windows.iter().enumerate() {
                if w.name == "Economy" {
                    self.opened[i] ^= true;
                }
            }
        }
        for (ws, opened) in self.windows.iter_mut().zip(self.opened.iter_mut()) {
            if *opened {
                ws.w.render_window(egui::Window::new(ws.name).open(opened), ui, uiworld, goria);
            }
        }
    }
}
