mod action_editor;
mod modals;
mod settings_tab;
mod workflow_tab;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke, Vec2};

use crate::core::{
    has_native_screen_picker, pick_screen_coordinates, HardwareProfile, Macro,
};
use crate::ui::ThemeMode;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum MainSubTab {
    #[default]
    Workflow,
    Settings,
}

pub struct MainTab {
    left_panel_width: f32,
    macros: Vec<Macro>,
    selected_macro_idx: Option<usize>,
    selected_action_idx: Option<usize>,
    current_subtab: MainSubTab,
    hardware_profiles: Vec<HardwareProfile>,
    selected_hw_idx: usize,
    detected_windows: Vec<String>,

    show_add_macro_modal: bool,
    new_macro_name: String,
    new_macro_error: bool,

    show_remove_macro_modal: bool,
    macro_idx_to_remove: Option<usize>,
    show_remove_action_modal: bool,
    action_idx_to_remove: Option<usize>,
    show_manage_hardware_modal: bool,
    new_hw_name: String,
    new_hw_vendor: u16,
    new_hw_product: u16,
    new_hw_error: Option<String>,

    show_manage_images_modal: bool,
    new_image_path: String,
    image_path_error: Option<String>,

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

    picking_position: bool,
    picking_mode: u8,
    picking_armed: bool,
    picking_activated_at: Option<std::time::Instant>,
    pending_screen_picker: Option<Arc<Mutex<Option<Option<(i32, i32)>>>>>,
    action_reorder_anim: Option<(usize, usize, std::time::Instant)>,

    show_hotkey_modal: bool,
    hotkey_target_slot: u8,
    hotkey_conflict_error: bool,
    pick_window_feedback: Option<std::time::Instant>,
    show_all_tags: bool,
    custom_quick_tags: Vec<String>,
    show_manage_tags_modal: bool,
    new_tag_input: String,
    tag_input_error: Option<String>,
    kde_eavesdropping_status: Option<String>,
    streaming_required_error: Option<std::time::Instant>,
}

impl Default for MainTab {
    fn default() -> Self {
        let loaded_tags = crate::core::load_custom_quick_tags().map(|tags| {
            tags.into_iter().map(|t| {
                if !t.starts_with('[') || !t.ends_with(']') {
                    format!("[{}]", t)
                } else {
                    t
                }
            }).filter(|t| crate::core::is_valid_keyboard_tag(t)).collect()
        }).unwrap_or_else(|| {
            vec![
                "[CTRL+V]".to_string(),
                "[CTRL+C]".to_string(),
                "[CTRL+A]".to_string(),
                "[ENTER]".to_string(),
                "[ESC]".to_string(),
                "[TAB]".to_string(),
                "[SPACE]".to_string(),
                "[BACKSPACE]".to_string(),
                "[DEL]".to_string(),
            ]
        });

        Self {
            left_panel_width: 320.0,
            macros: Vec::new(),
            selected_macro_idx: None,
            selected_action_idx: None,
            current_subtab: MainSubTab::Workflow,
            hardware_profiles: vec![HardwareProfile::default()],
            selected_hw_idx: 0,
            detected_windows: crate::core::list_detected_windows(),

            show_add_macro_modal: false,
            new_macro_name: String::new(),
            new_macro_error: false,

            show_remove_macro_modal: false,
            macro_idx_to_remove: None,
            show_remove_action_modal: false,
            action_idx_to_remove: None,
            show_manage_hardware_modal: false,
            new_hw_name: String::new(),
            new_hw_vendor: 0x046d,
            new_hw_product: 0xc077,
            new_hw_error: None,

            show_manage_images_modal: false,
            new_image_path: String::new(),
            image_path_error: None,

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

            picking_position: false,
            picking_mode: 1,
            picking_armed: false,
            picking_activated_at: None,
            pending_screen_picker: None,
            action_reorder_anim: None,

            show_hotkey_modal: false,
            hotkey_target_slot: 0,
            hotkey_conflict_error: false,
            pick_window_feedback: None,
            show_all_tags: false,
            custom_quick_tags: loaded_tags,
            show_manage_tags_modal: false,
            new_tag_input: String::new(),
            tag_input_error: None,
            kde_eavesdropping_status: None,
            streaming_required_error: None,
        }
    }
}

impl MainTab {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_macros(&self) -> &[Macro] {
        &self.macros
    }

    pub fn sync_current_quick_tags(&mut self) {
        if let Some(idx) = self.selected_macro_idx {
            if idx < self.macros.len() {
                self.macros[idx].quick_tags = self.custom_quick_tags.clone();
            }
        }
    }

    pub fn get_hardware(&self) -> &[HardwareProfile] {
        &self.hardware_profiles
    }

    pub fn set_data(&mut self, macros: Vec<Macro>, mut hardware: Vec<HardwareProfile>) {
        self.macros = macros;
        self.selected_macro_idx = if !self.macros.is_empty() { Some(0) } else { None };
        if let Some(first) = self.macros.first() {
            if !first.quick_tags.is_empty() {
                self.custom_quick_tags = first.quick_tags.clone();
            }
        }
        if !hardware.is_empty() {
            if !hardware.iter().any(|hp| hp.is_system_default()) {
                hardware.insert(0, HardwareProfile::default());
            }
            self.hardware_profiles = hardware;
            self.selected_hw_idx = 0;
        }
        for m in &mut self.macros {
            if let Some(ref prof) = m.hardware_profile {
                if let Some(pos) = self.hardware_profiles.iter().position(|h| h.vendor_id == prof.vendor_id && h.product_id == prof.product_id) {
                    m.hardware_idx = pos;
                } else {
                    self.hardware_profiles.push(prof.clone());
                    m.hardware_idx = self.hardware_profiles.len() - 1;
                }
            }
        }
        if let Some(first) = self.macros.first() {
            self.selected_hw_idx = first.hardware_idx;
        }
    }

    pub fn start_picking_coordinates(&mut self, mode: u8) {
        self.picking_mode = mode;
        self.picking_armed = false;
        self.picking_activated_at = Some(std::time::Instant::now());

        if has_native_screen_picker() {
            self.picking_position = false;
            let result_holder = Arc::new(Mutex::new(None));
            self.pending_screen_picker = Some(Arc::clone(&result_holder));

            std::thread::spawn(move || {
                let res = pick_screen_coordinates(mode);
                if let Ok(mut guard) = result_holder.lock() {
                    *guard = Some(res);
                }
            });
        } else {
            self.picking_position = true;
            self.pending_screen_picker = None;
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, engine: &crate::core::MacroEngine, theme: &ThemeMode) {
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
                            self.render_right_panel(ui, engine, theme);
                        });
                    },
                );
            },
        );

        self.render_modals(ui.ctx(), theme);

        if let Some(pending) = &self.pending_screen_picker {
            let mut done = false;
            let mut result_pos = None;
            if let Ok(guard) = pending.lock() {
                if let Some(res) = *guard {
                    done = true;
                    result_pos = res;
                }
            }
            if done {
                self.pending_screen_picker = None;
                self.picking_position = false;
                self.picking_armed = false;
                self.picking_activated_at = None;

                if let Some((px, py)) = result_pos {
                    if let (Some(m_idx), Some(a_idx)) = (self.selected_macro_idx, self.selected_action_idx) {
                        if m_idx < self.macros.len() && a_idx < self.macros[m_idx].actions.len() {
                            let action = &mut self.macros[m_idx].actions[a_idx];
                            if self.picking_mode == 2 {
                                action.mouse.end_x = px;
                                action.mouse.end_y = py;
                            } else {
                                action.mouse.x = px;
                                action.mouse.y = py;
                            }
                        }
                    }
                }
            }
        }

        if self.picking_position {
            let ctx = ui.ctx();
            ctx.request_repaint();

            ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.picking_position = false;
                self.picking_armed = false;
                self.picking_activated_at = None;
                self.pending_screen_picker = None;
            }

            let elapsed_since_activation = self
                .picking_activated_at
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0);

            if !self.picking_armed {
                if elapsed_since_activation >= 150 && !ctx.input(|i| i.pointer.any_down()) {
                    self.picking_armed = true;
                }
            }

            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("picking_crosshair_layer"),
            ));

            let screen_rect = ctx
                .input(|i| i.raw.screen_rect)
                .unwrap_or(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(1080.0, 768.0)));

            painter.rect_filled(
                screen_rect,
                0.0,
                Color32::from_rgba_premultiplied(14, 17, 26, 45),
            );

            let area_resp = egui::Area::new(egui::Id::new("picking_blocker_area"))
                .order(egui::Order::Foreground)
                .fixed_pos(screen_rect.min)
                .show(ctx, |ui| {
                    ui.allocate_response(screen_rect.size(), egui::Sense::click_and_drag())
                });

            let banner_title = if self.picking_mode == 2 {
                "🎯  PICK END POSITION"
            } else {
                "🎯  PICK MOUSE POSITION"
            };

            let banner_w = 560.0;
            let banner_h = 44.0;
            let banner_rect = egui::Rect::from_center_size(
                egui::pos2(screen_rect.center().x, 34.0),
                Vec2::new(banner_w, banner_h),
            );

            painter.rect_stroke(
                banner_rect,
                CornerRadius::same(10),
                Stroke::new(1.2, Color32::from_rgba_premultiplied(59, 130, 246, 190)),
                egui::StrokeKind::Inside,
            );

            painter.text(
                egui::pos2(banner_rect.center().x, banner_rect.top() + 14.0),
                egui::Align2::CENTER_CENTER,
                format!("{}  •  Press ESC to cancel", banner_title),
                egui::FontId::proportional(12.5),
                Color32::from_rgb(96, 165, 250),
            );
            
            painter.text(
                egui::pos2(banner_rect.center().x, banner_rect.top() + 30.0),
                egui::Align2::CENTER_CENTER,
                "Click anywhere on screen to capture coordinates",
                egui::FontId::proportional(10.5),
                Color32::from_rgb(148, 163, 184),
            );

            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                let (px, py) = (pos.x, pos.y);
                let primary_blue = Color32::from_rgb(59, 130, 246);
                let cyan_accent = Color32::from_rgb(56, 189, 248);

                let guide_color = Color32::from_rgba_premultiplied(59, 130, 246, 45);
                painter.line_segment(
                    [egui::pos2(screen_rect.left(), py), egui::pos2(px - 38.0, py)],
                    Stroke::new(1.0, guide_color),
                );
                painter.line_segment(
                    [egui::pos2(px + 38.0, py), egui::pos2(screen_rect.right(), py)],
                    Stroke::new(1.0, guide_color),
                );
                painter.line_segment(
                    [egui::pos2(px, screen_rect.top()), egui::pos2(px, py - 38.0)],
                    Stroke::new(1.0, guide_color),
                );
                painter.line_segment(
                    [egui::pos2(px, py + 38.0), egui::pos2(px, screen_rect.bottom())],
                    Stroke::new(1.0, guide_color),
                );

                painter.line_segment(
                    [egui::pos2(px - 32.0, py), egui::pos2(px - 10.0, py)],
                    Stroke::new(2.2, primary_blue),
                );
                painter.line_segment(
                    [egui::pos2(px + 10.0, py), egui::pos2(px + 32.0, py)],
                    Stroke::new(2.2, primary_blue),
                );
                painter.line_segment(
                    [egui::pos2(px, py - 32.0), egui::pos2(px, py - 10.0)],
                    Stroke::new(2.2, primary_blue),
                );
                painter.line_segment(
                    [egui::pos2(px, py + 10.0), egui::pos2(px, py + 32.0)],
                    Stroke::new(2.2, primary_blue),
                );

                painter.circle_stroke(
                    pos,
                    20.0,
                    Stroke::new(1.2, Color32::from_rgba_premultiplied(59, 130, 246, 180)),
                );
                painter.circle_stroke(pos, 7.0, Stroke::new(1.6, cyan_accent));
                painter.circle_filled(pos, 2.5, Color32::WHITE);

                let coord_badge_w = 175.0;
                let coord_badge_h = 36.0;
                let mut badge_x = px + 24.0;
                let mut badge_y = py - 46.0;

                if badge_x + coord_badge_w > screen_rect.right() - 12.0 {
                    badge_x = px - coord_badge_w - 24.0;
                }
                if badge_y < screen_rect.top() + 98.0 {
                    badge_y = py + 26.0;
                }

                let badge_rect = egui::Rect::from_min_size(
                    egui::pos2(badge_x, badge_y),
                    Vec2::new(coord_badge_w, coord_badge_h),
                );

                painter.rect_filled(
                    badge_rect.expand(2.0),
                    18.0,
                    Color32::from_rgba_premultiplied(0, 0, 0, 90),
                );
                
                painter.rect_filled(
                    badge_rect,
                    18.0,
                    Color32::from_rgba_premultiplied(22, 25, 37, 248),
                );
                
                painter.rect_stroke(
                    badge_rect,
                    CornerRadius::same(18),
                    Stroke::new(1.2, Color32::from_rgba_premultiplied(59, 130, 246, 180)),
                    egui::StrokeKind::Inside,
                );

                let dot_center = egui::pos2(badge_rect.left() + 16.0, badge_rect.center().y);
                painter.circle_filled(dot_center, 4.0, cyan_accent);

                let coord_text = format!("X: {:<4}  Y: {:<4}", px as i32, py as i32);
                painter.text(
                    egui::pos2(badge_rect.left() + 28.0, badge_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    coord_text,
                    egui::FontId::monospace(12.5),
                    Color32::from_rgb(248, 250, 252),
                );

                let triggered_click = self.picking_armed
                    && (area_resp.inner.clicked()
                        || ctx.input(|i| i.pointer.button_released(egui::PointerButton::Primary)));
                if triggered_click {
                    if let (Some(m_idx), Some(a_idx)) =
                        (self.selected_macro_idx, self.selected_action_idx)
                    {
                        if m_idx < self.macros.len() && a_idx < self.macros[m_idx].actions.len() {
                            let action = &mut self.macros[m_idx].actions[a_idx];
                            if self.picking_mode == 2 {
                                action.mouse.end_x = px as i32;
                                action.mouse.end_y = py as i32;
                            } else {
                                action.mouse.x = px as i32;
                                action.mouse.y = py as i32;
                            }
                        }
                    }
                    self.picking_position = false;
                    self.picking_armed = false;
                    self.picking_activated_at = None;
                    self.pending_screen_picker = None;
                }
            }
        }
    }

    fn render_left_panel(&mut self, ui: &mut egui::Ui, theme: &ThemeMode) {
        let half_w = (ui.available_width() * 0.5 - 6.0).max(40.0);

        ui.horizontal(|ui| {
            if theme.primary_button_animated(ui, "+ New", Vec2::new(half_w, 36.0)).clicked() {
                self.new_macro_name.clear();
                self.new_macro_error = false;
                self.show_add_macro_modal = true;
            }

            if theme.secondary_button_animated(ui, "Import", Vec2::new(half_w, 36.0)).clicked() {
                self.import_show_all = false;
                self.available_import_files = crate::core::list_exported_files_for_type(crate::core::MacroFileType::Workflow);
                self.selected_import_idx = None;
                self.import_error = None;
                self.show_import_modal = true;
            }
        });

        ui.add_space(10.0);

        if self.macros.is_empty() {
            theme.empty_state_left_panel(ui, "Your macros will be listed here");
            return;
        }

        let mut to_remove_idx = None;
        let mut new_selected_idx = None;

        egui::ScrollArea::vertical()
            .id_salt("macro_list_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 6.0);
                for (idx, m) in self.macros.iter().enumerate() {
                    let is_selected = self.selected_macro_idx == Some(idx);

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
                                let text_color = if m.is_active {
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
                                    new_selected_idx = Some(idx);
                                }

                                ui.painter().text(
                                    egui::pos2(name_rect.min.x, name_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &m.name,
                                    egui::FontId::proportional(14.0),
                                    text_color,
                                );

                                let (del_rect, del_resp) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), egui::Sense::click());
                                let del_hover_t = ui.ctx().animate_bool_with_time(del_resp.id.with("del_hover"), del_resp.hovered(), 0.12);
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

                                if del_resp.on_hover_text("Delete this macro").clicked() {
                                    to_remove_idx = Some(idx);
                                }
                            });
                        });
                }
            });

        if let Some(idx) = new_selected_idx {
            if let Some(cur) = self.selected_macro_idx {
                if cur < self.macros.len() {
                    self.macros[cur].quick_tags = self.custom_quick_tags.clone();
                }
            }
            self.selected_macro_idx = Some(idx);
            self.selected_action_idx = None;
            if !self.macros[idx].quick_tags.is_empty() {
                self.custom_quick_tags = self.macros[idx].quick_tags.clone();
            }
        }

        if let Some(idx) = to_remove_idx {
            self.macro_idx_to_remove = Some(idx);
            self.show_remove_macro_modal = true;
        }
    }

    fn render_right_panel(&mut self, ui: &mut egui::Ui, engine: &crate::core::MacroEngine, theme: &ThemeMode) {
        if self.macros.is_empty() {
            theme.empty_state_centered(
                ui,
                "No macro created yet",
                "Click \"+ New\" on the left panel to create a new macro.",
            );
            return;
        }

        let Some(idx) = self.selected_macro_idx else {
            theme.empty_state_centered(
                ui,
                "No macro selected",
                "Please select a macro from the list on the left.",
            );
            return;
        };

        if idx >= self.macros.len() {
            return;
        }

        let is_running = engine.is_active();
        self.macros[idx].is_active = is_running;
        let m = &mut self.macros[idx];

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
                self.export_filename = if clean.is_empty() { "macro".to_string() } else { clean };
                self.export_error = None;
                self.export_confirm_overwrite = false;
                self.show_export_modal = true;
            }
        });

        ui.add_space(8.0);

        let has_actions = !m.actions.is_empty();
        let needs_streaming = m.actions.iter().any(|a| a.enabled && a.use_image_recognition);
        let is_streaming = crate::core::is_screen_capture_worker_running();

        ui.horizontal(|ui| {
            let run_resp = theme.run_stop_button(ui, is_running);
            let run_resp = if !is_running && !has_actions {
                run_resp.on_hover_text("⚠ Add at least one action to the workflow before running.")
            } else if !is_running && needs_streaming && !is_streaming {
                run_resp.on_hover_text("⚠ Start screen stream ('Start Stream') before running this macro with Wait for Image.")
            } else {
                run_resp
            };

            if run_resp.clicked() {
                if is_running {
                    engine.stop();
                    m.is_active = false;
                } else if has_actions {
                    if needs_streaming && !is_streaming {
                        self.streaming_required_error = Some(std::time::Instant::now());
                    } else {
                        self.streaming_required_error = None;
                        engine.run_workflow(m.clone(), &self.hardware_profiles);
                        m.is_active = true;
                    }
                }
            }

            ui.add_space(16.0);
            let status_text = if is_running { "RUNNING" } else { "READY" };
            theme.status_info(ui, status_text, !is_running);

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("🚨 Emergency Stop: Press ESC 3x rapidly")
                    .color(theme.text_secondary())
                    .size(11.5),
            );
        });

        if let Some(error_time) = self.streaming_required_error {
            if error_time.elapsed().as_secs() < 6 {
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("⚠ Cannot start: this macro uses image recognition (Wait for Image). Start screen capture first by clicking 'Start Stream' below.")
                            .color(theme.danger_text())
                            .strong()
                            .size(12.5),
                    );
                });
            } else {
                self.streaming_required_error = None;
            }
        }

        ui.add_space(10.0);

        theme.animated_subtab_bar(
            ui,
            "main_subtabs",
            &[
                ("Workflow", MainSubTab::Workflow),
                ("Settings", MainSubTab::Settings),
            ],
            &mut self.current_subtab,
        );

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .id_salt("right_panel_content_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match self.current_subtab {
                    MainSubTab::Workflow => self.render_workflow_tab(ui, idx, engine, theme),
                    MainSubTab::Settings => self.render_settings_tab(ui, idx, theme),
                }
            });
    }

    pub fn get_active_macro(&self) -> Option<&Macro> {
        self.selected_macro_idx.and_then(|idx| self.macros.get(idx))
    }

    pub fn toggle_run_current(&mut self, engine: &crate::core::MacroEngine) {
        if let Some(idx) = self.selected_macro_idx {
            if idx < self.macros.len() {
                if engine.is_active() {
                    engine.stop();
                } else {
                    let needs_streaming = self.macros[idx].actions.iter().any(|a| a.enabled && a.use_image_recognition);
                    let is_streaming = crate::core::is_screen_capture_worker_running();
                    if needs_streaming && !is_streaming {
                        self.streaming_required_error = Some(std::time::Instant::now());
                    } else {
                        self.streaming_required_error = None;
                        engine.run_workflow(self.macros[idx].clone(), &self.hardware_profiles);
                    }
                }
            }
        }
    }

    pub fn pick_current_window(&mut self) {
        if let Some(win_title) = crate::core::get_active_window_title() {
            if let Some(idx) = self.selected_macro_idx {
                if let Some(m) = self.macros.get_mut(idx) {
                    if m.target_method == 0 {
                        self.detected_windows = crate::core::list_detected_windows();
                        if let Some(pos) = self.detected_windows.iter().position(|w| {
                            w.to_lowercase().contains(&win_title.to_lowercase())
                                || win_title.to_lowercase().contains(&w.to_lowercase())
                        }) {
                            m.target_window_idx = pos;
                        } else {
                            self.detected_windows.push(win_title.clone());
                            m.target_window_idx = self.detected_windows.len() - 1;
                        }
                    } else {
                        m.custom_window_name = win_title;
                    }
                    self.pick_window_feedback = Some(std::time::Instant::now());
                }
            }
        }
    }
}
