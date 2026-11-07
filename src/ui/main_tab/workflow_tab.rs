use eframe::egui::{self, Color32, CornerRadius, Rect, Stroke, Vec2};

use crate::core::{Action, ClickType};
use crate::ui::ThemeMode;
use super::MainTab;

impl MainTab {
    pub(super) fn render_workflow_tab(
        &mut self,
        ui: &mut egui::Ui,
        macro_idx: usize,
        engine: &crate::core::MacroEngine,
        theme: &ThemeMode,
    ) {
        let m = &mut self.macros[macro_idx];

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

        ui.add_space(8.0);

        theme.show_action_panel(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Add Action:")
                        .color(theme.text_secondary())
                        .size(12.5)
                        .strong(),
                );
                ui.add_space(4.0);
                if theme.action_add_button(ui, "+ Mouse").clicked() {
                    m.actions.push(Action::new_mouse());
                    self.selected_action_idx = Some(m.actions.len() - 1);
                }
                if theme.action_add_button(ui, "+ Keys").clicked() {
                    m.actions.push(Action::new_keyboard());
                    self.selected_action_idx = Some(m.actions.len() - 1);
                }
                if theme.action_add_button(ui, "+ Scroll").clicked() {
                    m.actions.push(Action::new_scroll());
                    self.selected_action_idx = Some(m.actions.len() - 1);
                }
                if theme.action_add_button(ui, "+ Wait").clicked() {
                    m.actions.push(Action::new_delay());
                    self.selected_action_idx = Some(m.actions.len() - 1);
                }
            });

            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .id_salt("actions_list_scroll")
                .max_height(140.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    if m.actions.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("No actions yet. Click '+ Mouse', '+ Keys', '+ Scroll' or '+ Wait' to add actions.")
                                .color(theme.text_secondary())
                                .size(13.0),
                        );
                        ui.add_space(8.0);
                    } else {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 5.0);
                        let mut move_up_idx = None;
                        let mut move_down_idx = None;
                        let mut remove_idx = None;
                        let actions_count = m.actions.len();

                        for i in 0..actions_count {
                            let is_sel = self.selected_action_idx == Some(i);
                            let a = &m.actions[i];
                            let label = match a.action_type {
                                crate::core::ActionType::Mouse => {
                                    if a.mouse.click_type == ClickType::DragAndDrop {
                                        format!(
                                            "{}. 🖱 Drag ({},{}) ➔ ({},{})  •  {:.2}s",
                                            i + 1,
                                            a.mouse.x,
                                            a.mouse.y,
                                            a.mouse.end_x,
                                            a.mouse.end_y,
                                            a.delay_time
                                        )
                                    } else {
                                        format!(
                                            "{}. 🖱 {} ({},{})  •  {:.2}s",
                                            i + 1,
                                            a.mouse.click_type.display_name(),
                                            a.mouse.x,
                                            a.mouse.y,
                                            a.delay_time
                                        )
                                    }
                                }
                                crate::core::ActionType::Keyboard => {
                                    let preview = if a.keyboard.text.len() > 24 {
                                        format!("{}…", &a.keyboard.text[..24])
                                    } else {
                                        a.keyboard.text.clone()
                                    };
                                    format!("{}. ⌨ Keys \"{}\"  •  {:.2}s", i + 1, preview, a.delay_time)
                                }
                                crate::core::ActionType::Delay => {
                                    format!("{}. ⏳ Wait {:.2}s", i + 1, a.delay_time)
                                }
                                crate::core::ActionType::Scroll => {
                                    format!("{}. 📜 Scroll {} ({} lines)  •  {:.2}s", i + 1, a.scroll.direction.display_name(), a.scroll.amount, a.delay_time)
                                }
                            };

                            let row_w = ui.available_width();
                            let (row_rect, _) = ui.allocate_exact_size(Vec2::new(row_w, 30.0), egui::Sense::hover());

                            let btn_size = Vec2::splat(22.0);
                            let btn_gap = 4.0;
                            let btn_y = row_rect.min.y + (row_rect.height() - btn_size.y) * 0.5;

                            let rem_x = row_rect.max.x - 14.0 - btn_size.x;
                            let down_x = rem_x - btn_gap - btn_size.x;
                            let up_x = down_x - btn_gap - btn_size.x;

                            let rem_rect = Rect::from_min_size(egui::pos2(rem_x, btn_y), btn_size);
                            let down_rect = Rect::from_min_size(egui::pos2(down_x, btn_y), btn_size);
                            let up_rect = Rect::from_min_size(egui::pos2(up_x, btn_y), btn_size);

                            let pill_rect = Rect::from_min_max(
                                row_rect.min,
                                egui::pos2(up_x - 6.0, row_rect.max.y),
                            );

                            let resp = ui.interact(pill_rect, ui.id().with(("act_pill", i)), egui::Sense::click());
                            if resp.clicked() {
                                self.selected_action_idx = Some(i);
                            }
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            let is_currently_active = engine.is_active() && engine.current_action() == Some(i);
                            if is_currently_active {
                                ui.ctx().request_repaint();
                            }

                            let active_t = ui.ctx().animate_bool_with_time(resp.id.with("act_pill_active"), is_currently_active, 0.12);
                            let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("act_pill_anim"), resp.hovered(), 0.15);

                            let (type_color, type_bg_dark, type_bg_light) = match a.action_type {
                                crate::core::ActionType::Mouse => (
                                    Color32::from_rgb(59, 130, 246),  
                                    Color32::from_rgb(26, 47, 85),
                                    Color32::from_rgb(219, 234, 254),
                                ),
                                crate::core::ActionType::Keyboard => (
                                    Color32::from_rgb(168, 85, 247), 
                                    Color32::from_rgb(52, 27, 82),
                                    Color32::from_rgb(243, 232, 255),
                                ),
                                crate::core::ActionType::Delay => (
                                    Color32::from_rgb(234, 179, 8),  
                                    Color32::from_rgb(68, 52, 14),
                                    Color32::from_rgb(254, 243, 199),
                                ),
                                crate::core::ActionType::Scroll => (
                                    Color32::from_rgb(16, 185, 129),  
                                    Color32::from_rgb(6, 78, 59),
                                    Color32::from_rgb(209, 250, 229),
                                ),
                            };

                            let (bg, text_color, border_color) = if is_currently_active {
                                let active_bg = if theme.is_dark() { type_bg_dark } else { type_bg_light };
                                let active_border = type_color;
                                let active_text = if theme.is_dark() { Color32::WHITE } else { Color32::from_rgb(15, 23, 42) };
                                (active_bg, active_text, active_border)
                            } else if is_sel {
                                if theme.is_dark() {
                                    (
                                        Color32::from_rgb(30, 58, 138),
                                        Color32::WHITE,
                                        Color32::from_rgb(96, 165, 250),
                                    )
                                } else {
                                    (
                                        Color32::from_rgb(224, 237, 255),
                                        Color32::from_rgb(29, 78, 216),
                                        Color32::from_rgb(59, 130, 246),
                                    )
                                }
                            } else {
                                let bg_normal = if theme.is_dark() { Color32::from_rgb(26, 31, 46) } else { Color32::from_rgb(255, 255, 255) };
                                let bg_hover = if theme.is_dark() { Color32::from_rgb(38, 46, 68) } else { Color32::from_rgb(241, 245, 249) };
                                (
                                    crate::ui::theme::lerp_color(bg_normal, bg_hover, hover_t),
                                    crate::ui::theme::lerp_color(theme.text_secondary(), theme.text_primary(), hover_t),
                                    crate::ui::theme::lerp_color(theme.border_inner_card(), theme.primary().gamma_multiply(0.7), hover_t),
                                )
                            };

                            let mut y_shift = 0.0;
                            if let Some((idx_a, idx_b, start_time)) = self.action_reorder_anim {
                                let elapsed = start_time.elapsed().as_secs_f32();
                                if elapsed < 0.22 {
                                    let progress = (elapsed / 0.22).min(1.0);
                                    let ease = (1.0 - progress) * (1.0 - progress);
                                    let travel_dist = 34.0;
                                    if i == idx_a {
                                        y_shift = travel_dist * ease;
                                    } else if i == idx_b {
                                        y_shift = -travel_dist * ease;
                                    }
                                    ui.ctx().request_repaint();
                                }
                            }

                            let active_jump = active_t * 2.0;
                            let draw_rect = if resp.is_pointer_button_down_on() {
                                pill_rect.shrink(0.6).translate(Vec2::new(active_jump, y_shift))
                            } else {
                                pill_rect.expand(0.4 * hover_t).translate(Vec2::new(active_jump, y_shift))
                            };

                            let painter = ui.painter();
                            if is_currently_active || active_t > 0.01 {
                                let pulse = ((ui.input(|i| i.time) * 7.0).sin().abs() as f32) * 0.3 + 0.35;
                                let glow = type_color.gamma_multiply(pulse * active_t);
                                painter.rect_filled(draw_rect.expand(2.5 * active_t), CornerRadius::same(9), glow);
                            } else if is_sel || hover_t > 0.01 {
                                let glow = theme.primary().gamma_multiply(if is_sel { 0.28 } else { 0.15 * hover_t });
                                painter.rect_filled(draw_rect.expand(1.2), CornerRadius::same(8), glow);
                            }
                            painter.rect_filled(draw_rect, CornerRadius::same(8), bg);
                            let stroke_w = if is_currently_active { 1.8 } else if is_sel { 1.5 } else { 1.0 };
                            painter.rect_stroke(draw_rect, CornerRadius::same(8), Stroke::new(stroke_w, border_color), egui::StrokeKind::Inside);

                            if is_currently_active {
                                let accent_rect = Rect::from_min_max(
                                    egui::pos2(draw_rect.min.x + 2.0, draw_rect.min.y + 3.0),
                                    egui::pos2(draw_rect.min.x + 6.0, draw_rect.max.y - 3.0),
                                );
                                painter.rect_filled(accent_rect, CornerRadius::same(2), type_color);
                            } else if is_sel {
                                let accent_rect = Rect::from_min_max(
                                    egui::pos2(draw_rect.min.x + 2.0, draw_rect.min.y + 4.0),
                                    egui::pos2(draw_rect.min.x + 5.5, draw_rect.max.y - 4.0),
                                );
                                painter.rect_filled(accent_rect, CornerRadius::same(2), theme.primary());
                            }

                            painter.text(
                                egui::pos2(draw_rect.min.x + if is_currently_active || is_sel { 15.0 } else { 12.0 }, draw_rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                label,
                                egui::FontId::proportional(12.5),
                                text_color,
                            );

                            let up_resp = theme.action_micro_button_at(ui, up_rect, &format!("up_{}", i), crate::ui::theme::ActionIconType::Up, i > 0);
                            if up_resp.on_hover_text("Move Up").clicked() && i > 0 {
                                move_up_idx = Some(i);
                            }

                            let down_resp = theme.action_micro_button_at(ui, down_rect, &format!("down_{}", i), crate::ui::theme::ActionIconType::Down, i + 1 < actions_count);
                            if down_resp.on_hover_text("Move Down").clicked() && i + 1 < actions_count {
                                move_down_idx = Some(i);
                            }

                            let rem_resp = theme.action_micro_button_at(ui, rem_rect, &format!("rem_{}", i), crate::ui::theme::ActionIconType::Remove, true);
                            if rem_resp.on_hover_text("Remove Action").clicked() {
                                remove_idx = Some(i);
                            }
                        }

                        if let Some(i) = move_up_idx {
                            if i > 0 && i < m.actions.len() {
                                m.actions.swap(i, i - 1);
                                self.action_reorder_anim = Some((i - 1, i, std::time::Instant::now()));
                                ui.ctx().request_repaint();
                                if self.selected_action_idx == Some(i) {
                                    self.selected_action_idx = Some(i - 1);
                                } else if self.selected_action_idx == Some(i - 1) {
                                    self.selected_action_idx = Some(i);
                                }
                            }
                        }
                        if let Some(i) = move_down_idx {
                            if i + 1 < m.actions.len() {
                                m.actions.swap(i, i + 1);
                                self.action_reorder_anim = Some((i, i + 1, std::time::Instant::now()));
                                ui.ctx().request_repaint();
                                if self.selected_action_idx == Some(i) {
                                    self.selected_action_idx = Some(i + 1);
                                } else if self.selected_action_idx == Some(i + 1) {
                                    self.selected_action_idx = Some(i);
                                }
                            }
                        }
                        if let Some(i) = remove_idx {
                            self.action_idx_to_remove = Some(i);
                            self.show_remove_action_modal = true;
                        }
                    }
                });
        });

        ui.add_space(10.0);

        if let Some(act_idx) = self.selected_action_idx {
            if act_idx < m.actions.len() {
                self.render_action_editor(ui, macro_idx, act_idx, theme);
            }
        }
    }


}
