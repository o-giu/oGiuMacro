use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use eframe::egui::{self, Color32, CornerRadius, Margin, Vec2};

use crate::core::{
    load_main_macros, load_rec_macros, save_main_macros, save_rec_macros, GlobalHotkeyListener,
    InputRecorder, MacroEngine, RecordedInputEvent,
};
use crate::ui::{AboutTab, MainTab, RecPlayTab, ThemeMode};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    #[default]
    Main,
    RecPlay,
    About,
}

pub struct MacroApp {
    active_tab: Tab,
    main_tab: MainTab,
    rec_play_tab: RecPlayTab,
    about_tab: AboutTab,
    theme: ThemeMode,
    last_save_instant: Option<Instant>,
    engine: MacroEngine,
    recorder: InputRecorder,
    hotkey_listener: GlobalHotkeyListener,
    event_tx: Sender<RecordedInputEvent>,
    event_rx: Receiver<RecordedInputEvent>,
    hotkey_rx: Receiver<u32>,
    show_startup_diagnosis: bool,
    diagnosis_data: crate::core::SystemDiagnosis,
    skip_diagnosis_checkbox: bool,
}

impl Default for MacroApp {
    fn default() -> Self {
        let (tx, rx) = channel();
        let (hk_tx, hk_rx) = channel();
        let mut main_tab = MainTab::new();
        let mut rec_play_tab = RecPlayTab::new();

        if let Some((macros, hardware)) = load_main_macros() {
            main_tab.set_data(macros, hardware);
        }

        if let Some(recordings) = load_rec_macros() {
            rec_play_tab.set_recordings(recordings);
        }

        let hotkey_listener = GlobalHotkeyListener::new();
        hotkey_listener.start(hk_tx);

        let skip_diagnosis = crate::core::load_skip_startup_diagnosis();
        let diagnosis_data = crate::core::SystemDiagnosis::collect();

        Self {
            active_tab: Tab::Main,
            main_tab,
            rec_play_tab,
            about_tab: AboutTab::new(),
            theme: ThemeMode::Dark,
            last_save_instant: None,
            engine: MacroEngine::new(),
            recorder: InputRecorder::new(),
            hotkey_listener,
            event_tx: tx,
            event_rx: rx,
            hotkey_rx: hk_rx,
            show_startup_diagnosis: !skip_diagnosis,
            diagnosis_data,
            skip_diagnosis_checkbox: skip_diagnosis,
        }
    }
}

impl MacroApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        let candidates = [
            ("Symbola", "/usr/share/fonts/gdouros-symbola/Symbola.ttf"),
            ("NotoEmoji", "/usr/share/fonts/google-noto-emoji-fonts/NotoEmoji-Regular.ttf"),
            ("DejaVuSans", "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf"),
        ];

        for (name, path) in &candidates {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert(
                    name.to_string(),
                    std::sync::Arc::new(egui::FontData::from_owned(data)),
                );
                if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    prop.push(name.to_string());
                }
                if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    mono.push(name.to_string());
                }
            }
        }

        cc.egui_ctx.set_fonts(fonts);

        Self::default()
    }

    fn stop_all_activity(&mut self) {
        self.engine.stop();
        self.recorder.stop();
        crate::core::stop_screen_capture_worker();
    }

    fn handle_global_hotkey(&mut self, fkey: u32) {
        if fkey == 999 {
            self.stop_all_activity();
            return;
        }

        match self.active_tab {
            Tab::Main => {
                if let Some(m) = self.main_tab.get_active_macro() {
                    if fkey == m.start_hotkey && !self.engine.is_active() {
                        self.main_tab.toggle_run_current(&self.engine);
                    } else if fkey == m.stop_hotkey && self.engine.is_active() {
                        self.engine.stop();
                    } else if fkey == m.pick_window_hotkey {
                        self.main_tab.pick_current_window();
                    }
                }
            }
            Tab::RecPlay => {
                if let Some(r) = self.rec_play_tab.get_active_rec() {
                    if fkey == r.start_rec_hotkey && !self.recorder.is_active() {
                        self.rec_play_tab.toggle_record(&self.recorder, &self.event_tx);
                    } else if fkey == r.stop_rec_hotkey && self.recorder.is_active() {
                        self.recorder.stop();
                    } else if fkey == r.start_play_hotkey && !self.engine.is_active() {
                        self.rec_play_tab.toggle_play(&self.engine);
                    } else if fkey == r.stop_play_hotkey && self.engine.is_active() {
                        self.engine.stop();
                    }
                }
            }
            Tab::About => {}
        }
    }
}

impl Drop for MacroApp {
    fn drop(&mut self) {
        self.stop_all_activity();
        self.hotkey_listener.stop();
    }
}

impl eframe::App for MacroApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.theme.apply(ui.ctx());

        while let Ok(event) = self.event_rx.try_recv() {
            self.rec_play_tab.add_recorded_event(event);
        }

        while let Ok(fkey) = self.hotkey_rx.try_recv() {
            self.handle_global_hotkey(fkey);
        }

        ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));

        let previous_tab = self.active_tab;

        egui::Frame::new()
            .fill(self.theme.bg_window())
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());

                ui.horizontal(|ui| {
                    
                    self.theme.animated_tab_bar(
                        ui,
                        "main_nav_tabs",
                        &[
                            ("Main", Tab::Main),
                            ("Rec/Play", Tab::RecPlay),
                            ("About", Tab::About),
                        ],
                        &mut self.active_tab,
                    );

                    if previous_tab != self.active_tab {
                        self.stop_all_activity();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let theme_hover = if self.theme.is_dark() {
                            "Switch to Light Mode"
                        } else {
                            "Switch to Dark Mode"
                        };

                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(36.0, 32.0), egui::Sense::click());
                        if resp.clicked() {
                            self.theme.toggle();
                        }
                        let resp = resp.on_hover_text(theme_hover);

                        let bg_color = if resp.hovered() {
                            if self.theme.is_dark() { Color32::from_rgb(34, 40, 58) } else { Color32::from_rgb(241, 245, 249) }
                        } else {
                            self.theme.bg_card()
                        };

                        let painter = ui.painter();
                        painter.rect_filled(rect, CornerRadius::same(16), bg_color);
                        painter.rect_stroke(rect, CornerRadius::same(16), egui::Stroke::new(1.0, self.theme.border_card()), egui::StrokeKind::Inside);

                        let center = rect.center();
                        let icon_color = self.theme.text_primary();

                        if self.theme.is_dark() {
                            let sun_r = 4.5;
                            painter.circle_filled(center, sun_r, icon_color);
                            let ray_len = 3.0;
                            let ray_dist = 7.0;
                            for i in 0..8 {
                                let angle = (i as f32) * std::f32::consts::PI / 4.0;
                                let dir = Vec2::new(angle.cos(), angle.sin());
                                let p_start = center + dir * ray_dist;
                                let p_end = center + dir * (ray_dist + ray_len);
                                painter.line_segment([p_start, p_end], egui::Stroke::new(1.5, icon_color));
                            }
                        } else {
                            let moon_r = 6.5;
                            painter.circle_filled(center, moon_r, icon_color);
                            let cut_center = egui::pos2(center.x + 3.0, center.y - 2.5);
                            painter.circle_filled(cut_center, moon_r * 0.85, bg_color);
                        }

                        ui.add_space(6.0);

                        let (folder_rect, folder_resp) = ui.allocate_exact_size(Vec2::new(36.0, 32.0), egui::Sense::click());
                        if folder_resp.clicked() {
                            crate::core::open_config_folder();
                        }
                        let folder_resp = folder_resp.on_hover_text("Open Config & Save Folder (~/.config/oGiuMacro/)");

                        let folder_bg = if folder_resp.hovered() {
                            if self.theme.is_dark() { Color32::from_rgb(34, 40, 58) } else { Color32::from_rgb(241, 245, 249) }
                        } else {
                            self.theme.bg_card()
                        };

                        let painter = ui.painter();
                        painter.rect_filled(folder_rect, CornerRadius::same(16), folder_bg);
                        painter.rect_stroke(folder_rect, CornerRadius::same(16), egui::Stroke::new(1.0, self.theme.border_card()), egui::StrokeKind::Inside);

                        let fc = folder_rect.center();
                        let f_color = if folder_resp.hovered() { self.theme.primary() } else { self.theme.text_primary() };
                        let f_w = 7.5;
                        let f_h = 5.5;
                        let tab_w = 3.5;
                        let tab_h = 2.0;

                        painter.rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(fc.x - f_w, fc.y - f_h - tab_h + 1.0),
                                egui::pos2(fc.x - f_w + tab_w, fc.y - f_h + 1.0),
                            ),
                            CornerRadius::same(1),
                            f_color,
                        );
                        painter.rect_stroke(
                            egui::Rect::from_min_max(
                                egui::pos2(fc.x - f_w, fc.y - f_h + 1.0),
                                egui::pos2(fc.x + f_w, fc.y + f_h + 1.0),
                            ),
                            CornerRadius::same(2),
                            egui::Stroke::new(1.4, f_color),
                            egui::epaint::StrokeKind::Inside,
                        );

                        ui.add_space(6.0);

                        let recently_saved = self
                            .last_save_instant
                            .map(|instant| instant.elapsed().as_secs_f32() < 2.0)
                            .unwrap_or(false);

                        let (btn_rect, btn_resp) = ui.allocate_exact_size(Vec2::new(36.0, 32.0), egui::Sense::click());
                        let hover_t = ui.ctx().animate_bool_with_time(btn_resp.id.with("save_btn_hover"), btn_resp.hovered(), 0.15);

                        if btn_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        let btn_resp = btn_resp.on_hover_text(if recently_saved { "All Settings Saved!" } else { "Save All Settings" });

                        let bg_col = if recently_saved {
                            self.theme.run_green().gamma_multiply(0.18)
                        } else {
                            crate::ui::theme::lerp_color(
                                self.theme.bg_card(),
                                self.theme.pill_inactive_bg(),
                                hover_t,
                            )
                        };
                        let border_col = if recently_saved {
                            self.theme.run_green()
                        } else {
                            crate::ui::theme::lerp_color(
                                self.theme.border_card(),
                                self.theme.primary(),
                                hover_t,
                            )
                        };

                        let painter = ui.painter();
                        if hover_t > 0.01 && !recently_saved {
                            painter.rect_filled(
                                btn_rect.expand(1.5 * hover_t),
                                CornerRadius::same(16),
                                self.theme.primary().gamma_multiply(0.15 * hover_t),
                            );
                        }
                        painter.rect_filled(btn_rect, CornerRadius::same(16), bg_col);
                        painter.rect_stroke(
                            btn_rect,
                            CornerRadius::same(16),
                            egui::Stroke::new(1.0 + 0.5 * hover_t, border_col),
                            egui::epaint::StrokeKind::Inside,
                        );

                        let sc = btn_rect.center();
                        let save_icon_color = if recently_saved {
                            self.theme.run_green()
                        } else if btn_resp.hovered() {
                            self.theme.primary()
                        } else {
                            self.theme.text_primary()
                        };

                        if recently_saved {
                            let chk_stroke = egui::Stroke::new(2.0, self.theme.run_green());
                            painter.line_segment([egui::pos2(sc.x - 4.5, sc.y), egui::pos2(sc.x - 1.0, sc.y + 4.0)], chk_stroke);
                            painter.line_segment([egui::pos2(sc.x - 1.0, sc.y + 4.0), egui::pos2(sc.x + 5.5, sc.y - 3.5)], chk_stroke);
                        } else {
                            let d_w = 6.5;
                            let d_h = 6.5;
                            let disk_rect = egui::Rect::from_min_max(
                                egui::pos2(sc.x - d_w, sc.y - d_h),
                                egui::pos2(sc.x + d_w, sc.y + d_h),
                            );
                            painter.rect_stroke(disk_rect, CornerRadius::same(2), egui::Stroke::new(1.4, save_icon_color), egui::epaint::StrokeKind::Inside);
                            let top_rect = egui::Rect::from_min_max(
                                egui::pos2(sc.x - 3.5, sc.y - d_h),
                                egui::pos2(sc.x + 2.5, sc.y - 2.0),
                            );
                            painter.rect_filled(top_rect, CornerRadius::same(1), save_icon_color);
                            let bottom_rect = egui::Rect::from_min_max(
                                egui::pos2(sc.x - 3.5, sc.y + 1.0),
                                egui::pos2(sc.x + 3.5, sc.y + d_h - 1.5),
                            );
                            painter.rect_stroke(bottom_rect, CornerRadius::same(1), egui::Stroke::new(1.0, save_icon_color), egui::epaint::StrokeKind::Inside);
                        }

                        if btn_resp.clicked() {
                            self.main_tab.sync_current_quick_tags();
                            let _ = save_main_macros(self.main_tab.get_macros(), self.main_tab.get_hardware());
                            let _ = save_rec_macros(self.rec_play_tab.get_recordings());
                            self.last_save_instant = Some(Instant::now());
                        }
                    });
                });

                ui.add_space(8.0);

                let content_avail = ui.available_size();
                ui.allocate_ui_with_layout(
                    content_avail,
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        match self.active_tab {
                            Tab::Main => self.main_tab.render(ui, &self.engine, &self.theme),
                            Tab::RecPlay => self.rec_play_tab.render(ui, &self.engine, &self.recorder, &self.event_tx, &self.theme),
                            Tab::About => {
                                if self.about_tab.render(ui, &self.theme) {
                                    match crate::core::delete_config_folder() {
                                        Ok(()) => {
                                            self.main_tab = MainTab::new();
                                            self.rec_play_tab = RecPlayTab::new();
                                            self.last_save_instant = None;
                                            self.about_tab.set_reset_result(
                                                "All application data and configuration have been reset to factory defaults.".to_string(),
                                                true,
                                            );
                                        }
                                        Err(e) => {
                                            self.about_tab.set_reset_result(e, false);
                                        }
                                    }
                                }
                            }
                        }
                    },
                );

                if self.show_startup_diagnosis {
                    self.render_startup_diagnosis_modal(ui.ctx());
                }
            });
    }
}

impl MacroApp {
    fn render_startup_diagnosis_modal(&mut self, ctx: &egui::Context) {
        let mut close_modal = false;
        let theme = &self.theme;

        egui::Window::new("System Health & Environment")
            .collapsible(false)
            .resizable(false)
            .fixed_size([520.0, 440.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Detected System:")
                            .size(13.0)
                            .strong()
                            .color(theme.text_secondary()),
                    );
                    ui.label(
                        egui::RichText::new(self.diagnosis_data.distro.display_name())
                            .size(13.5)
                            .strong()
                            .color(theme.primary()),
                    );
                    ui.label(
                        egui::RichText::new(format!("• Session: {}", self.diagnosis_data.session_type))
                            .size(12.5)
                            .color(theme.text_secondary()),
                    );
                });

                ui.add_space(8.0);

                theme.show_inner_card(ui, |ui| {
                    ui.label(egui::RichText::new("Core Permissions:").strong().size(13.0));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let (icon, color, text) = if self.diagnosis_data.has_uinput {
                            ("✓", theme.run_green(), "/dev/uinput detected (Hardware Input Simulation active)")
                        } else {
                            ("✕", theme.danger_text(), "/dev/uinput missing (Kernel uinput module required)")
                        };
                        ui.colored_label(color, icon);
                        ui.label(egui::RichText::new(text).size(12.5));
                    });

                    ui.horizontal(|ui| {
                        let (icon, color, text) = if self.diagnosis_data.has_passwordless_access {
                            ("✓", theme.run_green(), "Udev rule installed (Runs without root password)")
                        } else {
                            ("ℹ", theme.text_secondary(), "Standard access (Prompts for password or requires udev rule)")
                        };
                        ui.colored_label(color, icon);
                        ui.label(egui::RichText::new(text).size(12.5));
                    });
                });

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Optional Desktop Tools:").strong().size(13.0));
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 4.0);
                        for dep in &self.diagnosis_data.dependencies {
                            theme.show_inner_card(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let (icon, color) = if dep.installed {
                                        ("✓", theme.run_green())
                                    } else {
                                        ("✕", theme.danger_text())
                                    };
                                    ui.colored_label(color, egui::RichText::new(icon).strong());
                                    ui.label(egui::RichText::new(dep.name).strong().size(12.5))
                                        .on_hover_text(dep.description);
                                    ui.colored_label(theme.text_secondary(), format!("({})", dep.command));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if dep.installed {
                                            ui.colored_label(theme.run_green(), "Installed");
                                        } else {
                                            ui.colored_label(theme.danger_text(), format!("Needed for: {}", dep.required_for));
                                        }
                                    });
                                });
                            });
                        }
                    });

                if let Some(cmd) = self.diagnosis_data.install_command() {
                    ui.add_space(8.0);
                    theme.show_inner_card(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "To install missing optional tools via {}:",
                                    self.diagnosis_data.distro.package_manager_name()
                                ))
                                .size(12.0)
                                .color(theme.text_secondary()),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("📋 Copy").size(11.0)).clicked() {
                                    ui.ctx().copy_text(cmd.clone());
                                }
                            });
                        });
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new(&cmd).monospace().size(11.5).color(theme.primary()));
                    });
                } else {
                    ui.add_space(8.0);
                    ui.colored_label(theme.run_green(), "✓ All system tools and dependencies are fully installed!");
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.skip_diagnosis_checkbox, "Do not show this screen again on startup").changed() {
                        let _ = crate::core::save_skip_startup_diagnosis(self.skip_diagnosis_checkbox);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let continue_btn = egui::Button::new(
                            egui::RichText::new("Continue to oGiuMacro").color(Color32::WHITE).strong()
                        )
                        .fill(theme.primary())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([160.0, 30.0], continue_btn).clicked() {
                            close_modal = true;
                        }
                    });
                });
            });

        if close_modal {
            self.show_startup_diagnosis = false;
        }
    }
}

