use crate::game_loop::Timings;
use crate::inputmap::{Bindings, InputMap};
use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egregoria::Egregoria;
use egui::{Align2, Context, Widget};
use egui_extras::Column;
use std::time::Duration;

const SETTINGS_SAVE_NAME: &str = "settings";

#[derive(Serialize, Deserialize, Copy, Clone)]
pub(crate) enum ShadowQuality {
    NoShadows,
    Low,
    Medium,
    High,
}

impl AsRef<str> for ShadowQuality {
    fn as_ref(&self) -> &str {
        match self {
            ShadowQuality::NoShadows => "No Shadows",
            ShadowQuality::Low => "Low",
            ShadowQuality::Medium => "Medium",
            ShadowQuality::High => "High",
        }
    }
}

impl From<u8> for ShadowQuality {
    fn from(v: u8) -> Self {
        match v {
            0 => ShadowQuality::NoShadows,
            1 => ShadowQuality::Low,
            2 => ShadowQuality::Medium,
            3 => ShadowQuality::High,
            _ => ShadowQuality::High,
        }
    }
}

impl ShadowQuality {
    pub(crate) fn size(&self) -> Option<u32> {
        match self {
            ShadowQuality::Low => Some(512),
            ShadowQuality::Medium => Some(1024),
            ShadowQuality::High => Some(2048),
            ShadowQuality::NoShadows => None,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub(crate) camera_border_move: bool,
    pub(crate) camera_smooth: bool,
    pub(crate) camera_smooth_tightness: f32,
    pub(crate) camera_fov: f32,

    pub(crate) fullscreen: bool,
    pub(crate) vsync: bool,
    pub(crate) ssao: bool,
    pub(crate) shadows: ShadowQuality,
    pub(crate) realistic_sky: bool,
    pub(crate) terrain_grid: bool,

    pub(crate) gui_scale: f32,

    pub(crate) music_volume_percent: f32,
    pub(crate) effects_volume_percent: f32,
    pub(crate) ui_volume_percent: f32,

    #[serde(skip)]
    pub(crate) time_warp: u32,
    pub(crate) auto_save_every: AutoSaveEvery,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            camera_border_move: true,
            camera_smooth: true,
            music_volume_percent: 100.0,
            effects_volume_percent: 100.0,
            ui_volume_percent: 100.0,
            fullscreen: false,
            vsync: true,
            time_warp: 1,
            auto_save_every: AutoSaveEvery::FiveMinutes,
            ssao: true,
            shadows: ShadowQuality::High,
            camera_smooth_tightness: 1.0,
            realistic_sky: true,
            camera_fov: 60.0,
            terrain_grid: true,
            gui_scale: 1.0,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub(crate) enum AutoSaveEvery {
    Never,
    OneMinute,
    FiveMinutes,
}

impl From<AutoSaveEvery> for Option<Duration> {
    fn from(x: AutoSaveEvery) -> Option<Duration> {
        match x {
            AutoSaveEvery::Never => None,
            AutoSaveEvery::OneMinute => Some(Duration::from_secs(60)),
            AutoSaveEvery::FiveMinutes => Some(Duration::from_secs(5 * 60)),
        }
    }
}

impl From<u8> for AutoSaveEvery {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Never,
            1 => Self::OneMinute,
            2 => Self::FiveMinutes,
            _ => Self::Never,
        }
    }
}

impl AsRef<str> for AutoSaveEvery {
    fn as_ref(&self) -> &str {
        match self {
            AutoSaveEvery::Never => "Never",
            AutoSaveEvery::OneMinute => "Minute",
            AutoSaveEvery::FiveMinutes => "Five Minutes",
        }
    }
}

pub(crate) fn settings(
    window: egui::Window<'_>,
    ui: &Context,
    uiworld: &mut UiWorld,
    _: &Egregoria,
) {
    let mut settings = uiworld.write::<Settings>();
    let [_, h]: [f32; 2] = ui.available_rect().size().into();

    window
        .default_size([500.0, h * 0.8])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .vscroll(true)
        .collapsible(false)
        .show(ui, |ui| {
            ui.label("Gameplay");

            let mut id = settings.auto_save_every as u8 as usize;
            egui::ComboBox::from_label("Autosave").show_index(ui, &mut id, 3, |i| {
                AutoSaveEvery::from(i as u8).as_ref().to_string()
            });
            settings.auto_save_every = AutoSaveEvery::from(id as u8);

            ui.label("Input");

            ui.checkbox(
                &mut settings.camera_border_move,
                "Border screen camera movement",
            );
            ui.checkbox(&mut settings.camera_smooth, "Camera smooth");

            if settings.camera_smooth {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut settings.camera_smooth_tightness).speed(0.01));
                    ui.label("Camera smoothing tightness");
                });
            }
            ui.horizontal(|ui| {
                egui::DragValue::new(&mut settings.camera_fov)
                    .clamp_range(1.0..=179.0f32)
                    .speed(0.1)
                    .ui(ui);
                ui.label("Camera Field of View (FOV)");
            });

            let fps = 1.0 / uiworld.read::<Timings>().all.avg();

            ui.separator();
            ui.label(format!("Graphics - {:.1}FPS", fps));

            ui.checkbox(&mut settings.fullscreen, "Fullscreen");
            ui.checkbox(&mut settings.realistic_sky, "Realistic sky");
            ui.checkbox(&mut settings.terrain_grid, "Terrain Grid");
            ui.checkbox(&mut settings.ssao, "Ambient Occlusion (SSAO)");

            // shadow quality combobox
            let mut id = settings.shadows as u8 as usize;
            egui::ComboBox::from_label("Shadow Quality").show_index(ui, &mut id, 4, |i| {
                ShadowQuality::from(i as u8).as_ref().to_string()
            });
            settings.shadows = ShadowQuality::from(id as u8);

            ui.checkbox(&mut settings.vsync, "VSync");

            ui.separator();
            ui.label("GUI");
            ui.horizontal(|ui| {
                // we only change gui_scale at end of interaction to avoid feedback loops
                let mut gui_scale = settings.gui_scale;
                let res = ui.add(egui::Slider::new(&mut gui_scale, 0.5..=2.0));
                if res.drag_released() {
                    settings.gui_scale = gui_scale;
                }
                ui.label("GUI Scale");
            });

            ui.separator();
            ui.label("Audio");

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.music_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{:.0}%", x)),
                );
                ui.label("Music volume");
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.effects_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{:.0}%", x)),
                );
                ui.label("Effects volume");
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.ui_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{:.0}%", x)),
                );
                ui.label("Ui volume");
            });

            ui.separator();
            let mut bindings = uiworld.write::<Bindings>();
            ui.horizontal(|ui| {
                ui.label("Keybinds");

                if ui.button("Reset").clicked() {
                    *bindings = Bindings::default();
                    uiworld.write::<InputMap>().build_input_tree(&mut bindings);
                }
            });

            let mut sorted_inps = bindings.0.keys().copied().collect::<Vec<_>>();
            sorted_inps.sort();

            egui_extras::TableBuilder::new(ui)
                .column(Column::initial(150.0))
                .column(Column::initial(150.0))
                .column(Column::initial(150.0))
                .column(Column::initial(50.0))
                .header(30.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Action");
                    });
                    header.col(|ui| {
                        ui.label("Primary");
                    });
                    header.col(|ui| {
                        ui.label("Secondary");
                    });
                })
                .body(|body| {
                    body.rows(25.0, sorted_inps.len(), |i, mut ui| {
                        let action = &sorted_inps[i];
                        let comb = bindings.0.get_mut(action).unwrap();

                        ui.col(|ui| {
                            ui.label(format!("{}", action));
                        });
                        ui.col(|ui| {
                            let resp = if !comb.0.is_empty() {
                                ui.button(format!("{}", comb.0[0]))
                            } else {
                                ui.button("<empty>")
                            };
                            if resp.hovered() {
                                egui::show_tooltip_text(
                                    ui.ctx(),
                                    ui.make_persistent_id("notimplemented"),
                                    "Not implemented yet",
                                );
                            }
                            if resp.clicked() {}
                        });
                        ui.col(|ui| {
                            let resp = if comb.0.len() > 1 {
                                ui.button(format!("{}", comb.0[1]))
                            } else {
                                ui.button("<empty>")
                            };
                            if resp.hovered() {
                                egui::show_tooltip_text(
                                    ui.ctx(),
                                    ui.make_persistent_id("notimplemented"),
                                    "Not implemented yet",
                                );
                            }
                            if resp.clicked() {}
                        });
                        ui.col(|ui| {
                            if ui.button("↺").clicked() {
                                comb.0 = Bindings::default().0.remove(action).unwrap().0;
                            }
                        });
                    })
                });

            common::saveload::JSONPretty::save_silent(&*settings, SETTINGS_SAVE_NAME);
        });
}
