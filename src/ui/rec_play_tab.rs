use std::path::PathBuf;
use std::sync::mpsc::Sender;
use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke, Vec2};

use crate::core::{InputRecorder, MacroEngine, RecordedInputEvent, RecordedMacro};
use crate::ui::ThemeMode;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum RecSubTab {
    #[default]
    Workflow,
    Settings,
}

pub struct RecPlayTab {
    left_panel_width: f32,
    recordings: Vec<RecordedMacro>,
    selected_rec_idx: Option<usize>,
    current_subtab: RecSubTab,

    show_add_modal: bool,
    new_rec_name: String,
    new_rec_error: bool,

    show_remove_modal: bool,
    rec_idx_to_remove: Option<usize>,

    show_import_modal: bool,
    show_export_modal: bool,
    export_filename: String,
    export_error: Option<String>,
    export_confirm_overwrite: bool,
    import_error: Option<String>,
    import_show_all: bool,
    selected_import_idx: Option<usize>,
    available_import_files: Vec<PathBuf>,
    show_delete_import_file_modal: bool,
    file_to_delete_idx: Option<usize>,

    show_hotkey_modal: bool,
    hotkey_target_slot: u8,
    hotkey_conflict_error: bool,
}

impl Default for RecPlayTab {
    fn default() -> Self {
        Self {
            left_panel_width: 320.0,
            recordings: Vec::new(),
            selected_rec_idx: None,
            current_subtab: RecSubTab::Workflow,

            show_add_modal: false,
            new_rec_name: String::new(),
            new_rec_error: false,

            show_remove_modal: false,
            rec_idx_to_remove: None,

            show_import_modal: false,
            show_export_modal: false,
            export_filename: String::new(),
            export_error: None,
            export_confirm_overwrite: false,
            import_error: None,
            import_show_all: false,
            selected_import_idx: None,
            available_import_files: Vec::new(),
            show_delete_import_file_modal: false,
            file_to_delete_idx: None,

            show_hotkey_modal: false,
            hotkey_target_slot: 0,
            hotkey_conflict_error: false,
        }
    }
}

impl RecPlayTab {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_recordings(&self) -> &[RecordedMacro] {
        &self.recordings
    }

    pub fn set_recordings(&mut self, recordings: Vec<RecordedMacro>) {
        self.recordings = recordings;
        self.selected_rec_idx = if !self.recordings.is_empty() { Some(0) } else { None };
    }

    pub fn add_recorded_event(&mut self, event: RecordedInputEvent) {
        if let Some(idx) = self.selected_rec_idx {
            if idx < self.recordings.len() {
                self.recordings[idx].events.push(event);
            }
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        engine: &MacroEngine,
        recorder: &InputRecorder,
        event_tx: &Sender<RecordedInputEvent>,
        theme: &ThemeMode,
    ) {
        let total_w = ui.available_width();
        let min_left = (total_w * 0.25).clamp(180.0, 240.0);
        let max_left = (total_w - 380.0).max(min_left);
        self.left_panel_width = self.left_panel_width.clamp(min_left, max_left);

        let avail_size = ui.available_size();
        let full_h = avail_size.y;

        ui.allocate_ui_with_layout(
            avail_size,
            egui::Layout::left_to_right(egui::Align::Min),
            |ui| {
                
                ui.allocate_ui_with_layout(
                    Vec2::new(self.left_panel_width, full_h),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        theme.card_frame().show(ui, |ui| {
                            ui.set_min_size(Vec2::new(ui.available_width(), ui.available_height()));
                            self.render_left_panel(ui, theme);
                        });
                    },
                );

                let splitter_rect = ui.allocate_response(
                    Vec2::new(6.0, full_h),
                    egui::Sense::drag(),
                );
                if splitter_rect.hovered() || splitter_rect.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if splitter_rect.dragged() {
                    self.left_panel_width += splitter_rect.drag_delta().x;
                    self.left_panel_width = self.left_panel_width.clamp(min_left, max_left);
                }

                let right_w = ui.available_width();
                ui.allocate_ui_with_layout(
                    Vec2::new(right_w, full_h),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        theme.card_frame().show(ui, |ui| {
                            ui.set_min_size(Vec2::new(ui.available_width(), ui.available_height()));
                            self.render_right_panel(ui, engine, recorder, event_tx, theme);
                        });
                    },
                );
            },
        );

        self.render_modals(ui.ctx(), theme);
    }

    fn render_left_panel(&mut self, ui: &mut egui::Ui, theme: &ThemeMode) {
        let half_w = (ui.available_width() * 0.5 - 6.0).max(40.0);

        ui.horizontal(|ui| {
            if theme.primary_button_animated(ui, "+ New", Vec2::new(half_w, 36.0)).clicked() {
                self.new_rec_name.clear();
                self.new_rec_error = false;
                self.show_add_modal = true;
            }

            if theme.secondary_button_animated(ui, "Import", Vec2::new(half_w, 36.0)).clicked() {
                self.import_show_all = false;
                self.available_import_files = crate::core::list_exported_files_for_type(crate::core::MacroFileType::Recording);
                self.selected_import_idx = None;
                self.import_error = None;
                self.show_import_modal = true;
            }
        });

        ui.add_space(10.0);

        if self.recordings.is_empty() {
            theme.empty_state_left_panel(ui, "Your recordings will be listed here");
            return;
        }

        let mut to_remove_idx = None;

        egui::ScrollArea::vertical()
            .id_salt("rec_list_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 6.0);

                for (idx, m) in self.recordings.iter().enumerate() {
                    let is_selected = self.selected_rec_idx == Some(idx);

                    let (card_bg, stroke) = if is_selected {
                        let tint = if theme.is_dark() {
                            Color32::from_rgb(26, 38, 66)
                        } else {
                            Color32::from_rgb(238, 242, 255)
                        };
                        (tint, Stroke::new(1.5, theme.primary()))
                    } else {
                        (Color32::TRANSPARENT, Stroke::NONE)
                    };

                    egui::Frame::new()
                        .fill(card_bg)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let text_color = if m.is_recording {
                                    theme.stop_red()
                                } else if m.is_playing {
                                    theme.run_green()
                                } else if is_selected {
                                    if theme.is_dark() { Color32::WHITE } else { Color32::from_rgb(29, 78, 216) }
                                } else {
                                    theme.text_primary()
                                };

                                let remain_w = (ui.available_width() - 28.0).max(40.0);
                                let (name_rect, name_resp) = ui.allocate_exact_size(Vec2::new(remain_w, 22.0), egui::Sense::click());
                                if name_resp.hovered() && !is_selected {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if name_resp.clicked() {
                                    self.selected_rec_idx = Some(idx);
                                }

                                ui.painter().text(
                                    egui::pos2(name_rect.min.x, name_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &m.name,
                                    egui::FontId::proportional(14.0),
                                    text_color,
                                );

                                let (del_rect, del_resp) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), egui::Sense::click());
                                let del_hover_t = ui.ctx().animate_bool_with_time(del_resp.id.with("rec_del_hover"), del_resp.hovered(), 0.12);
                                if del_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                if del_hover_t > 0.01 {
                                    ui.painter().circle_filled(del_rect.center(), 10.0, theme.danger_text().gamma_multiply(0.18 * del_hover_t));
                                }

                                let cross_color = crate::ui::theme::lerp_color(
                                    if is_selected { theme.danger_text().gamma_multiply(0.7) } else { theme.text_secondary().gamma_multiply(0.5) },
                                    theme.stop_red(),
                                    del_hover_t,
                                );
                                let c = del_rect.center();
                                let d = 3.5;
                                ui.painter().line_segment([egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)], Stroke::new(1.8, cross_color));
                                ui.painter().line_segment([egui::pos2(c.x - d, c.y + d), egui::pos2(c.x + d, c.y - d)], Stroke::new(1.8, cross_color));

                                if del_resp.on_hover_text("Delete this recording").clicked() {
                                    to_remove_idx = Some(idx);
                                }
                            });
                        });
                }
            });

        if let Some(idx) = to_remove_idx {
            self.rec_idx_to_remove = Some(idx);
            self.show_remove_modal = true;
        }
    }

    fn render_right_panel(
        &mut self,
        ui: &mut egui::Ui,
        engine: &MacroEngine,
        recorder: &InputRecorder,
        event_tx: &Sender<RecordedInputEvent>,
        theme: &ThemeMode,
    ) {
        if self.recordings.is_empty() {
            theme.empty_state_centered(
                ui,
                "No recording created yet",
                "Click \"+ New\" on the left panel to create a new recording.",
            );
            return;
        }

        let Some(idx) = self.selected_rec_idx else {
            theme.empty_state_centered(
                ui,
                "No recording selected",
                "Please select a recording from the list on the left.",
            );
            return;
        };

        if idx >= self.recordings.len() {
            return;
        }

        let is_rec = recorder.is_active();
        let is_play = engine.is_active();

        self.recordings[idx].is_recording = is_rec;
        self.recordings[idx].is_playing = is_play;

        let m = &mut self.recordings[idx];

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Name")
                    .color(theme.text_secondary())
                    .size(13.5)
                    .strong(),
            );
            ui.add_space(4.0);

            let export_btn_w = 90.0;
            let name_w = (ui.available_width() - export_btn_w - 12.0).max(80.0);
            ui.add_sized(
                Vec2::new(name_w, 32.0),
                egui::TextEdit::singleline(&mut m.name)
                    .margin(Margin::symmetric(10, 6)),
            );

            if theme.secondary_button_animated(ui, "Export", Vec2::new(export_btn_w, 32.0)).clicked() {
                let clean = m.name.trim().trim_end_matches(".json").to_string();
                self.export_filename = if clean.is_empty() { "recording".to_string() } else { clean };
                self.export_error = None;
                self.export_confirm_overwrite = false;
                self.show_export_modal = true;
            }
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            
            let base_rec_bg = if is_rec {
                theme.stop_red()
            } else {
                Color32::from_rgb(185, 28, 28)
            };

            let (rec_rect, rec_resp) = ui.allocate_exact_size(Vec2::new(124.0, 34.0), egui::Sense::click());
            let rec_hover_t = ui.ctx().animate_bool_with_time(rec_resp.id.with("rec_btn_hover"), rec_resp.hovered(), 0.15);

            if rec_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let rec_bg = crate::ui::theme::lerp_color(
                base_rec_bg,
                if is_rec { Color32::from_rgb(255, 75, 75) } else { Color32::from_rgb(220, 38, 38) },
                rec_hover_t,
            );

            let rec_painter = ui.painter();
            if rec_hover_t > 0.01 {
                rec_painter.rect_filled(
                    rec_rect.expand(1.5 * rec_hover_t),
                    CornerRadius::same(17),
                    base_rec_bg.gamma_multiply(0.3 * rec_hover_t),
                );
            }
            rec_painter.rect_filled(rec_rect, CornerRadius::same(17), rec_bg);

            let rec_icon_c = egui::pos2(rec_rect.min.x + 24.0, rec_rect.center().y);
            if is_rec {
                let sq_rect = egui::Rect::from_center_size(rec_icon_c, Vec2::splat(10.0));
                rec_painter.rect_filled(sq_rect, CornerRadius::same(2), Color32::WHITE);
                rec_painter.text(egui::pos2(rec_rect.min.x + 42.0, rec_rect.center().y), egui::Align2::LEFT_CENTER, "STOP REC", egui::FontId::proportional(13.0), Color32::WHITE);
            } else {
                rec_painter.circle_filled(rec_icon_c, 5.0, Color32::WHITE);
                rec_painter.text(egui::pos2(rec_rect.min.x + 42.0, rec_rect.center().y), egui::Align2::LEFT_CENTER, "RECORD", egui::FontId::proportional(13.0), Color32::WHITE);
            }

            if !is_play && rec_resp.clicked() {
                if is_rec {
                    recorder.stop();
                    m.is_recording = false;
                } else {
                    m.events.clear();
                    if let Some((x, y)) = crate::core::get_mouse_location() {
                        m.start_x = x;
                        m.start_y = y;
                    }
                    recorder.start(event_tx.clone());
                    m.is_recording = true;
                }
            }

            ui.add_space(8.0);

            let base_play_bg = if is_play { theme.stop_red() } else { theme.run_green() };
            let (play_rect, play_resp) = ui.allocate_exact_size(Vec2::new(124.0, 34.0), egui::Sense::click());
            let play_hover_t = ui.ctx().animate_bool_with_time(play_resp.id.with("play_btn_hover"), play_resp.hovered(), 0.15);

            if play_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let play_bg = crate::ui::theme::lerp_color(
                base_play_bg,
                if is_play { Color32::from_rgb(255, 75, 75) } else { Color32::from_rgb(45, 210, 100) },
                play_hover_t,
            );

            let play_painter = ui.painter();
            if play_hover_t > 0.01 {
                play_painter.rect_filled(
                    play_rect.expand(1.5 * play_hover_t),
                    CornerRadius::same(17),
                    base_play_bg.gamma_multiply(0.3 * play_hover_t),
                );
            }
            play_painter.rect_filled(play_rect, CornerRadius::same(17), play_bg);

            let play_icon_c = egui::pos2(play_rect.min.x + 26.0, play_rect.center().y);
            if is_play {
                let sq_rect = egui::Rect::from_center_size(play_icon_c, Vec2::splat(10.0));
                play_painter.rect_filled(sq_rect, CornerRadius::same(2), Color32::WHITE);
                play_painter.text(egui::pos2(play_rect.min.x + 44.0, play_rect.center().y), egui::Align2::LEFT_CENTER, "STOP PLAY", egui::FontId::proportional(13.0), Color32::WHITE);
            } else {
                let p1 = egui::pos2(play_icon_c.x - 4.5, play_icon_c.y - 6.0);
                let p2 = egui::pos2(play_icon_c.x - 4.5, play_icon_c.y + 6.0);
                let p3 = egui::pos2(play_icon_c.x + 5.5, play_icon_c.y);
                play_painter.add(egui::Shape::convex_polygon(vec![p1, p2, p3], Color32::WHITE, Stroke::NONE));
                play_painter.text(egui::pos2(play_rect.min.x + 44.0, play_rect.center().y), egui::Align2::LEFT_CENTER, "PLAY", egui::FontId::proportional(13.0), Color32::WHITE);
            }

            let play_resp = if !is_play && m.events.is_empty() {
                play_resp.on_hover_text("⚠ Record actions first to enable playback.")
            } else {
                play_resp
            };

            if !is_rec && !m.events.is_empty() && play_resp.clicked() {
                if is_play {
                    engine.stop();
                    m.is_playing = false;
                } else {
                    engine.run_recorded(m.clone());
                    m.is_playing = true;
                }
            }

            ui.add_space(16.0);

            let (status_text, is_active) = if is_rec {
                ("RECORDING", true)
            } else if is_play {
                ("PLAYING", true)
            } else {
                ("READY", false)
            };

            theme.status_info(ui, status_text, is_active);

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("🚨 Emergency Stop: Press ESC 3x rapidly")
                    .color(theme.text_secondary())
                    .size(11.5),
            );
        });

        ui.add_space(10.0);

        theme.animated_subtab_bar(
            ui,
            "rec_play_subtabs",
            &[
                ("Workflow", RecSubTab::Workflow),
                ("Settings", RecSubTab::Settings),
            ],
            &mut self.current_subtab,
        );

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .id_salt("rec_right_panel_content_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match self.current_subtab {
                    RecSubTab::Workflow => self.render_workflow_tab(ui, idx, theme),
                    RecSubTab::Settings => self.render_settings_tab(ui, idx, theme),
                }
            });
    }

    fn render_workflow_tab(&mut self, ui: &mut egui::Ui, rec_idx: usize, theme: &ThemeMode) {
        let m = &mut self.recordings[rec_idx];

        theme.show_inner_card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut m.is_loop, "Loop Sequence");
                if m.is_loop {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Times (0=Inf):")
                            .color(theme.text_secondary())
                            .size(13.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut m.loop_count)
                            .range(0..=99999),
                    );
                }
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Playback Speed:");
                theme.full_width_slider(ui, egui::Slider::new(&mut m.playback_speed, 0.1..=5.0).step_by(0.1));
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let badge_bg = if m.is_recording {
                    theme.stop_red().gamma_multiply(0.18)
                } else if !m.events.is_empty() {
                    theme.primary().gamma_multiply(0.15)
                } else {
                    theme.pill_inactive_bg()
                };

                let badge_text_color = if m.is_recording {
                    theme.stop_red()
                } else if !m.events.is_empty() {
                    theme.primary()
                } else {
                    theme.text_secondary()
                };

                egui::Frame::new()
                    .fill(badge_bg)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("⚡ {} Actions Recorded", m.events.len()))
                                .color(badge_text_color)
                                .size(13.0)
                                .strong(),
                        );
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let clear_btn = egui::Button::new(
                        egui::RichText::new("Clear All")
                            .color(theme.danger_text())
                            .size(12.0)
                            .strong(),
                    )
                    .fill(theme.pill_inactive_bg())
                    .stroke(Stroke::new(1.0, theme.border_card()))
                    .corner_radius(CornerRadius::same(8));

                    if ui.add_enabled(!m.is_recording && !m.events.is_empty(), clear_btn).clicked() {
                        m.events.clear();
                    }
                });
            });
        });
    }

    fn render_settings_tab(&mut self, ui: &mut egui::Ui, rec_idx: usize, theme: &ThemeMode) {
        let m = &mut self.recordings[rec_idx];

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Mouse")
                    .color(theme.primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.checkbox(&mut m.left_handed_override, "Left-handed mouse (swap buttons on playback)");
        });

        ui.add_space(8.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Hotkey Customization (F1-F12)")
                    .color(theme.primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Start Record Hotkey:");
                let label = crate::core::fkey_name(m.start_rec_hotkey);
                let btn = egui::Button::new(label)
                    .fill(theme.pill_inactive_bg())
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::new(50.0, 24.0));
                if ui.add(btn).clicked() {
                    self.hotkey_target_slot = 0;
                    self.hotkey_conflict_error = false;
                    self.show_hotkey_modal = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Stop Record Hotkey:");
                let label = crate::core::fkey_name(m.stop_rec_hotkey);
                let btn = egui::Button::new(label)
                    .fill(theme.pill_inactive_bg())
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::new(50.0, 24.0));
                if ui.add(btn).clicked() {
                    self.hotkey_target_slot = 1;
                    self.hotkey_conflict_error = false;
                    self.show_hotkey_modal = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Start Playback Hotkey:");
                let label = crate::core::fkey_name(m.start_play_hotkey);
                let btn = egui::Button::new(label)
                    .fill(theme.pill_inactive_bg())
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::new(50.0, 24.0));
                if ui.add(btn).clicked() {
                    self.hotkey_target_slot = 2;
                    self.hotkey_conflict_error = false;
                    self.show_hotkey_modal = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Stop Playback Hotkey:");
                let label = crate::core::fkey_name(m.stop_play_hotkey);
                let btn = egui::Button::new(label)
                    .fill(theme.pill_inactive_bg())
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::new(50.0, 24.0));
                if ui.add(btn).clicked() {
                    self.hotkey_target_slot = 3;
                    self.hotkey_conflict_error = false;
                    self.show_hotkey_modal = true;
                }
            });
        });
    }

    fn render_modals(&mut self, ctx: &egui::Context, theme: &ThemeMode) {
        if self.show_add_modal {
            egui::Window::new("Add Recording")
                .collapsible(false)
                .resizable(false)
                .min_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Recording Name:").strong());
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_rec_name)
                            .hint_text("Enter recording name...")
                            .margin(Margin::symmetric(10, 8))
                            .desired_width(f32::INFINITY),
                    );
                    if self.new_rec_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), "Name is empty or already exists.");
                    }
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let create_btn = egui::Button::new(
                            egui::RichText::new("Create Recording").color(Color32::WHITE).strong(),
                        )
                        .fill(theme.primary())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([140.0, 32.0], create_btn).clicked() {
                            let name = self.new_rec_name.trim();
                            let exists = self.recordings.iter().any(|r| r.name == name);
                            if !name.is_empty() && !exists {
                                self.recordings.push(RecordedMacro::new(name));
                                self.selected_rec_idx = Some(self.recordings.len() - 1);
                                self.show_add_modal = false;
                            } else {
                                self.new_rec_error = true;
                            }
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.show_add_modal = false;
                        }
                    });
                });
        }

        if self.show_remove_modal {
            let target_name = self
                .rec_idx_to_remove
                .and_then(|i| self.recordings.get(i))
                .map(|r| r.name.clone())
                .unwrap_or_else(|| "this recording".to_string());

            egui::Window::new("Confirm Delete Recording")
                .collapsible(false)
                .resizable(false)
                .min_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(format!("Are you sure you want to delete recording \"{}\"?", target_name)).size(13.5));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("Delete Recording").color(Color32::WHITE).strong(),
                        )
                        .fill(theme.stop_red())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([140.0, 32.0], del_btn).clicked() {
                            if let Some(del_idx) = self.rec_idx_to_remove {
                                if del_idx < self.recordings.len() {
                                    self.recordings.remove(del_idx);
                                    if self.recordings.is_empty() {
                                        self.selected_rec_idx = None;
                                    } else if let Some(sel) = self.selected_rec_idx {
                                        if sel == del_idx {
                                            self.selected_rec_idx = Some(del_idx.min(self.recordings.len() - 1));
                                        } else if sel > del_idx {
                                            self.selected_rec_idx = Some(sel - 1);
                                        }
                                    }
                                }
                            }
                            self.rec_idx_to_remove = None;
                            self.show_remove_modal = false;
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.rec_idx_to_remove = None;
                            self.show_remove_modal = false;
                        }
                    });
                });
        }

        if self.show_import_modal {
            let mut to_delete_idx = None;

            egui::Window::new("Import Recording Macro")
                .collapsible(false)
                .resizable(false)
                .min_width(480.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let header_title = if !self.import_show_all {
                            "Saved Recordings (~/.config/oGiuMacro/recordings):"
                        } else {
                            "All Saved Macros (~/.config/oGiuMacro):"
                        };
                        ui.label(egui::RichText::new(header_title).strong().size(13.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (ref_rect, ref_resp) = ui.allocate_exact_size(Vec2::new(75.0, 22.0), egui::Sense::click());
                            let ref_bg = if ref_resp.hovered() { theme.pill_inactive_bg() } else { Color32::TRANSPARENT };
                            ui.painter().rect_filled(ref_rect, CornerRadius::same(6), ref_bg);
                            ui.painter().text(
                                ref_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "⟳ Refresh",
                                egui::FontId::proportional(11.5),
                                if ref_resp.hovered() { theme.primary() } else { theme.text_secondary() },
                            );
                            if ref_resp.clicked() {
                                self.selected_import_idx = None;
                                self.import_error = None;
                                self.available_import_files = if self.import_show_all {
                                    crate::core::list_exported_files()
                                } else {
                                    crate::core::list_exported_files_for_type(crate::core::MacroFileType::Recording)
                                };
                            }
                        });
                    });
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let rec_active = !self.import_show_all;
                        let rec_count = crate::core::list_exported_files_for_type(crate::core::MacroFileType::Recording).len();
                        let all_count = crate::core::list_exported_files().len();

                        let rec_btn = egui::Button::new(
                            egui::RichText::new(format!("Recordings ({})", rec_count))
                                .size(11.5)
                                .strong()
                                .color(if rec_active { Color32::WHITE } else { theme.text_secondary() }),
                        )
                        .fill(if rec_active { theme.primary() } else { theme.pill_inactive_bg() })
                        .corner_radius(CornerRadius::same(8));

                        if ui.add(rec_btn).clicked() && !rec_active {
                            self.import_show_all = false;
                            self.selected_import_idx = None;
                            self.import_error = None;
                            self.available_import_files = crate::core::list_exported_files_for_type(crate::core::MacroFileType::Recording);
                        }

                        let all_btn = egui::Button::new(
                            egui::RichText::new(format!("All Saved Files ({})", all_count))
                                .size(11.5)
                                .strong()
                                .color(if !rec_active { Color32::WHITE } else { theme.text_secondary() }),
                        )
                        .fill(if !rec_active { theme.primary() } else { theme.pill_inactive_bg() })
                        .corner_radius(CornerRadius::same(8));

                        if ui.add(all_btn).clicked() && rec_active {
                            self.import_show_all = true;
                            self.selected_import_idx = None;
                            self.import_error = None;
                            self.available_import_files = crate::core::list_exported_files();
                        }
                    });
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .id_salt("rec_import_files_scroll")
                        .max_height(200.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.available_import_files.is_empty() {
                                ui.add_space(20.0);
                                ui.vertical_centered(|ui| {
                                    let msg = if !self.import_show_all {
                                        "No exported Recording macros found in ~/.config/oGiuMacro/recordings."
                                    } else {
                                        "No exported files found in ~/.config/oGiuMacro."
                                    };
                                    ui.colored_label(theme.text_secondary(), msg);
                                });
                                ui.add_space(20.0);
                            } else {
                                ui.spacing_mut().item_spacing = Vec2::new(0.0, 5.0);
                                for (i, p) in self.available_import_files.iter().enumerate() {
                                    let is_selected = self.selected_import_idx == Some(i);
                                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                    let file_type = crate::core::detect_macro_file_type(p);
                                    let is_compatible = file_type == crate::core::MacroFileType::Recording;

                                    let folder_hint = if p.starts_with(crate::core::get_workflows_dir()) {
                                        "workflows/"
                                    } else if p.starts_with(crate::core::get_recordings_dir()) {
                                        "recordings/"
                                    } else {
                                        ""
                                    };

                                    let display_name = if self.import_show_all && !folder_hint.is_empty() {
                                        format!("{}{}", folder_hint, name)
                                    } else {
                                        name.to_string()
                                    };

                                    let (card_bg, stroke) = if is_selected {
                                        let tint = if !is_compatible {
                                            if theme.is_dark() { Color32::from_rgb(45, 25, 30) } else { Color32::from_rgb(254, 242, 242) }
                                        } else if theme.is_dark() {
                                            Color32::from_rgb(26, 38, 66)
                                        } else {
                                            Color32::from_rgb(238, 242, 255)
                                        };
                                        let stroke_color = if !is_compatible { theme.danger_text() } else { theme.primary() };
                                        (tint, Stroke::new(1.5, stroke_color))
                                    } else {
                                        let bg = if theme.is_dark() {
                                            Color32::from_rgb(22, 27, 40)
                                        } else {
                                            Color32::from_rgb(248, 250, 252)
                                        };
                                        (bg, Stroke::new(1.0, theme.border_inner_card()))
                                    };

                                    egui::Frame::new()
                                        .fill(card_bg)
                                        .stroke(stroke)
                                        .corner_radius(CornerRadius::same(10))
                                        .inner_margin(Margin::symmetric(10, 8))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.horizontal(|ui| {
                                                let text_color = if is_selected {
                                                    if !is_compatible {
                                                        theme.danger_text()
                                                    } else if theme.is_dark() {
                                                        Color32::WHITE
                                                    } else {
                                                        Color32::from_rgb(29, 78, 216)
                                                    }
                                                } else if !is_compatible {
                                                    theme.text_secondary()
                                                } else {
                                                    theme.text_primary()
                                                };

                                                let (badge_label, badge_color, badge_bg) = match file_type {
                                                    crate::core::MacroFileType::Workflow => (
                                                        "WORKFLOW",
                                                        theme.primary(),
                                                        theme.primary().gamma_multiply(0.20),
                                                    ),
                                                    crate::core::MacroFileType::Recording => (
                                                        "REC / PLAY",
                                                        if is_selected { Color32::WHITE } else { theme.stop_red() },
                                                        if is_selected { theme.stop_red() } else { theme.stop_red().gamma_multiply(0.20) },
                                                    ),
                                                    crate::core::MacroFileType::Unknown => (
                                                        "UNKNOWN",
                                                        theme.text_secondary(),
                                                        theme.pill_inactive_bg(),
                                                    ),
                                                };
                                                let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(74.0, 20.0), egui::Sense::hover());
                                                ui.painter().rect_filled(badge_rect, CornerRadius::same(4), badge_bg);
                                                ui.painter().text(
                                                    badge_rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    badge_label,
                                                    egui::FontId::proportional(10.0),
                                                    badge_color,
                                                );
                                                ui.add_space(4.0);

                                                let remain_w = (ui.available_width() - 30.0).max(40.0);
                                                let (name_rect, name_resp) = ui.allocate_exact_size(Vec2::new(remain_w, 22.0), egui::Sense::click());
                                                if name_resp.hovered() && !is_selected {
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                                }
                                                if name_resp.clicked() {
                                                    self.selected_import_idx = Some(i);
                                                    if !is_compatible {
                                                        self.import_error = Some("This file is a Workflow macro. Switch to the Workflow tab to import it.".to_string());
                                                    } else {
                                                        self.import_error = None;
                                                    }
                                                }

                                                let _ = name_resp.on_hover_text(format!("Path: {}", p.display()));

                                                ui.painter().text(
                                                    egui::pos2(name_rect.min.x, name_rect.center().y),
                                                    egui::Align2::LEFT_CENTER,
                                                    &display_name,
                                                    egui::FontId::proportional(13.5),
                                                    text_color,
                                                );

                                                let (del_rect, del_resp) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), egui::Sense::click());
                                                let del_hover_t = ui.ctx().animate_bool_with_time(del_resp.id.with(format!("rec_imp_del_{}", i)), del_resp.hovered(), 0.12);
                                                if del_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                                }

                                                if del_hover_t > 0.01 {
                                                    ui.painter().circle_filled(del_rect.center(), 10.0, theme.danger_text().gamma_multiply(0.18 * del_hover_t));
                                                }

                                                let cross_color = crate::ui::theme::lerp_color(
                                                    if is_selected { theme.danger_text().gamma_multiply(0.7) } else { theme.text_secondary().gamma_multiply(0.5) },
                                                    theme.stop_red(),
                                                    del_hover_t,
                                                );
                                                let c = del_rect.center();
                                                let d = 3.5;
                                                ui.painter().line_segment([egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)], Stroke::new(1.8, cross_color));
                                                ui.painter().line_segment([egui::pos2(c.x - d, c.y + d), egui::pos2(c.x + d, c.y - d)], Stroke::new(1.8, cross_color));

                                                if del_resp.on_hover_text("Delete this saved macro file").clicked() {
                                                    to_delete_idx = Some(i);
                                                }
                                            });
                                        });
                                }
                            }
                        });

                    let selected_file_type = self.selected_import_idx
                        .and_then(|idx| self.available_import_files.get(idx))
                        .map(|p| crate::core::detect_macro_file_type(p))
                        .unwrap_or(crate::core::MacroFileType::Unknown);
                    let can_import = self.selected_import_idx.is_some() && selected_file_type == crate::core::MacroFileType::Recording;

                    if let Some(err) = &self.import_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), format!("⚠ {}", err));
                    } else if can_import {
                        ui.add_space(4.0);
                        ui.colored_label(theme.run_green(), "✓ Compatible Recording ready to import.");
                    }

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let imp_btn = egui::Button::new(
                            egui::RichText::new("Import Selected").color(Color32::WHITE).strong()
                        )
                        .fill(if can_import { theme.primary() } else { theme.pill_inactive_bg() })
                        .corner_radius(CornerRadius::same(10));

                        let resp = ui.add_enabled(can_import, imp_btn);
                        let resp = if !can_import && self.selected_import_idx.is_some() {
                            resp.on_disabled_hover_text("Cannot import a Workflow macro into Rec/Play. Switch to the Workflow tab.")
                        } else {
                            resp
                        };

                        if resp.clicked() {
                            if let Some(idx) = self.selected_import_idx {
                                if let Some(path) = self.available_import_files.get(idx) {
                                    match crate::core::import_rec_macro(path) {
                                        Ok(mut imported) => {
                                            let base = imported.name.clone();
                                            let mut count = 1;
                                            while self.recordings.iter().any(|r| r.name == imported.name) {
                                                imported.name = format!("{}_{}", base, count);
                                                count += 1;
                                            }
                                            self.recordings.push(imported);
                                            self.selected_rec_idx = Some(self.recordings.len() - 1);
                                            self.import_error = None;
                                            self.show_import_modal = false;
                                        }
                                        Err(e) => {
                                            self.import_error = Some(format!("{}", e));
                                        }
                                    }
                                }
                            }
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.show_import_modal = false;
                            self.import_error = None;
                        }
                    });
                });

            if let Some(idx) = to_delete_idx {
                self.file_to_delete_idx = Some(idx);
                self.show_delete_import_file_modal = true;
            }
        }

        if self.show_delete_import_file_modal {
            let target_name = self
                .file_to_delete_idx
                .and_then(|i| self.available_import_files.get(i))
                .map(|p| {
                    let folder_hint = if p.starts_with(crate::core::get_workflows_dir()) {
                        "workflows/"
                    } else if p.starts_with(crate::core::get_recordings_dir()) {
                        "recordings/"
                    } else {
                        ""
                    };
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    format!("{}{}", folder_hint, name)
                })
                .unwrap_or_else(|| "this file".to_string());

            egui::Window::new("Confirm Delete Saved File")
                .collapsible(false)
                .resizable(false)
                .min_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Are you sure you want to permanently delete \"{}\"?",
                            target_name
                        ))
                        .size(13.5),
                    );
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("Delete File").color(Color32::WHITE).strong(),
                        )
                        .fill(theme.stop_red())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([120.0, 32.0], del_btn).clicked() {
                            if let Some(del_idx) = self.file_to_delete_idx {
                                if let Some(path) = self.available_import_files.get(del_idx) {
                                    let _ = crate::core::delete_exported_file(path);
                                }
                                self.available_import_files = if self.import_show_all {
                                    crate::core::list_exported_files()
                                } else {
                                    crate::core::list_exported_files_for_type(crate::core::MacroFileType::Recording)
                                };
                                if self.selected_import_idx == Some(del_idx) {
                                    self.selected_import_idx = None;
                                } else if let Some(sel) = self.selected_import_idx {
                                    if sel > del_idx {
                                        self.selected_import_idx = Some(sel - 1);
                                    }
                                }
                            }
                            self.file_to_delete_idx = None;
                            self.show_delete_import_file_modal = false;
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.file_to_delete_idx = None;
                            self.show_delete_import_file_modal = false;
                        }
                    });
                });
        }

        if self.show_export_modal {
            egui::Window::new("Export Recording")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .max_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui: &mut egui::Ui| {
                    ui.label(egui::RichText::new("Exporting to ~/.config/oGiuMacro/recordings/").color(theme.text_secondary()));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Filename:").strong());
                    ui.add_space(2.0);

                    let prev_filename = self.export_filename.clone();
                    self.export_filename.retain(|c| !matches!(c, '/' | '\\' | '\0' | ':' | '*' | '?' | '"' | '<' | '>' | '|'));
                    if self.export_filename.to_lowercase().ends_with(".json") {
                        let new_len = self.export_filename.len() - 5;
                        self.export_filename.truncate(new_len);
                    }
                    if self.export_filename != prev_filename {
                        self.export_confirm_overwrite = false;
                        self.export_error = None;
                    }

                    ui.horizontal(|ui| {
                        let resp = ui.add_sized(
                            Vec2::new(300.0, 32.0),
                            egui::TextEdit::singleline(&mut self.export_filename)
                                .hint_text("Recording name")
                                .margin(Margin::symmetric(10, 8)),
                        );
                        if resp.changed() {
                            self.export_confirm_overwrite = false;
                            self.export_error = None;
                        }

                        egui::Frame::new()
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(12, 7))
                            .show(ui, |ui: &mut egui::Ui| {
                                ui.label(
                                    egui::RichText::new(".json")
                                        .monospace()
                                        .size(13.5)
                                        .color(theme.primary())
                                        .strong(),
                                );
                            });
                    });

                    let clean_base = self.export_filename.trim().trim_end_matches(".json").trim_end_matches('.').trim();
                    let file_exists = !clean_base.is_empty() && crate::core::get_recordings_dir().join(format!("{}.json", clean_base)).exists();

                    if file_exists && !self.export_confirm_overwrite {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), "⚠ A saved recording with this name already exists.");
                    } else if let Some(err) = &self.export_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), format!("⚠ {}", err));
                    }
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let btn_label = if file_exists && self.export_confirm_overwrite {
                            "Overwrite"
                        } else {
                            "Export"
                        };
                        let btn_color = if file_exists && self.export_confirm_overwrite {
                            theme.stop_red()
                        } else {
                            theme.primary()
                        };
                        let exp_btn = egui::Button::new(
                            egui::RichText::new(btn_label).color(Color32::WHITE).strong()
                        )
                        .fill(btn_color)
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([110.0, 32.0], exp_btn).clicked() {
                            if clean_base.is_empty() {
                                self.export_error = Some("Filename cannot be empty.".to_string());
                                self.export_confirm_overwrite = false;
                            } else if file_exists && !self.export_confirm_overwrite {
                                self.export_confirm_overwrite = true;
                                self.export_error = Some(format!("A saved file named '{}.json' already exists in recordings. Click 'Overwrite' to replace it, or choose a different name.", clean_base));
                            } else {
                                let fname = format!("{}.json", clean_base);
                                if let Some(idx) = self.selected_rec_idx {
                                    if idx < self.recordings.len() {
                                        match crate::core::export_rec_macro(&self.recordings[idx], &fname) {
                                            Ok(_) => {
                                                self.export_error = None;
                                                self.export_confirm_overwrite = false;
                                                self.show_export_modal = false;
                                            }
                                            Err(e) => {
                                                self.export_error = Some(format!("Export failed: {}", e));
                                                self.export_confirm_overwrite = false;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.show_export_modal = false;
                            self.export_error = None;
                            self.export_confirm_overwrite = false;
                        }
                    });
                });
        }

        if self.show_hotkey_modal {
            egui::Window::new("Assign Hotkey")
                .collapsible(false)
                .resizable(false)
                .min_width(360.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let slot_name = match self.hotkey_target_slot {
                        0 => "Start Record Hotkey",
                        1 => "Stop Record Hotkey",
                        2 => "Start Play Hotkey",
                        _ => "Stop Play Hotkey",
                    };
                    ui.label(egui::RichText::new(format!("Select F-Key for {}:", slot_name)).strong());
                    if self.hotkey_conflict_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), "Hotkey already in use by another action.");
                    }
                    ui.add_space(10.0);
                    egui::Grid::new("rec_hotkey_grid").spacing(Vec2::new(8.0, 8.0)).show(ui, |ui| {
                        for row in 0..3 {
                            for col in 1..=4 {
                                let f_num = row * 4 + col;
                                let code = 289 + f_num;
                                let btn_label = format!("F{}", f_num);
                                let f_btn = egui::Button::new(
                                    egui::RichText::new(&btn_label).strong()
                                )
                                .fill(theme.pill_inactive_bg())
                                .corner_radius(CornerRadius::same(8));

                                if ui.add_sized([64.0, 32.0], f_btn).clicked() {
                                    if let Some(r_idx) = self.selected_rec_idx {
                                        if let Some(r) = self.recordings.get(r_idx) {
                                            let conflict = match self.hotkey_target_slot {
                                                0 => r.stop_rec_hotkey == code || r.start_play_hotkey == code || r.stop_play_hotkey == code,
                                                1 => r.start_rec_hotkey == code || r.start_play_hotkey == code || r.stop_play_hotkey == code,
                                                2 => r.start_rec_hotkey == code || r.stop_rec_hotkey == code || r.stop_play_hotkey == code,
                                                _ => r.start_rec_hotkey == code || r.stop_rec_hotkey == code || r.start_play_hotkey == code,
                                            };
                                            if conflict {
                                                self.hotkey_conflict_error = true;
                                            } else {
                                                if let Some(r_mut) = self.recordings.get_mut(r_idx) {
                                                    match self.hotkey_target_slot {
                                                        0 => r_mut.start_rec_hotkey = code,
                                                        1 => r_mut.stop_rec_hotkey = code,
                                                        2 => r_mut.start_play_hotkey = code,
                                                        _ => r_mut.stop_play_hotkey = code,
                                                    }
                                                }
                                                self.show_hotkey_modal = false;
                                                self.hotkey_conflict_error = false;
                                            }
                                        }
                                    }
                                }
                            }
                            ui.end_row();
                        }
                    });
                    ui.add_space(12.0);
                    let cancel_btn = egui::Button::new("Cancel")
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(10));
                    if ui.add_sized([100.0, 32.0], cancel_btn).clicked() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.show_hotkey_modal = false;
                        self.hotkey_conflict_error = false;
                    }
                });
        }
    }

    pub fn get_active_rec(&self) -> Option<&RecordedMacro> {
        self.selected_rec_idx.and_then(|idx| self.recordings.get(idx))
    }

    pub fn toggle_record(&mut self, recorder: &InputRecorder, event_tx: &Sender<RecordedInputEvent>) {
        if let Some(idx) = self.selected_rec_idx {
            if idx < self.recordings.len() {
                if recorder.is_active() {
                    recorder.stop();
                    self.recordings[idx].is_recording = false;
                } else {
                    self.recordings[idx].events.clear();
                    if let Some((x, y)) = crate::core::get_mouse_location() {
                        self.recordings[idx].start_x = x;
                        self.recordings[idx].start_y = y;
                    }
                    recorder.start(event_tx.clone());
                    self.recordings[idx].is_recording = true;
                }
            }
        }
    }

    pub fn toggle_play(&mut self, engine: &MacroEngine) {
        if let Some(idx) = self.selected_rec_idx {
            if idx < self.recordings.len() {
                if engine.is_active() {
                    engine.stop();
                    self.recordings[idx].is_playing = false;
                } else {
                    engine.run_recorded(self.recordings[idx].clone());
                    self.recordings[idx].is_playing = true;
                }
            }
        }
    }
}
