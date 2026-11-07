use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Rect, Stroke, Vec2};
use serde::{Deserialize, Serialize};

pub fn lerp_color(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = (c1.r() as f32 + (c2.r() as f32 - c1.r() as f32) * t).round() as u8;
    let g = (c1.g() as f32 + (c2.g() as f32 - c1.g() as f32) * t).round() as u8;
    let b = (c1.b() as f32 + (c2.b() as f32 - c1.b() as f32) * t).round() as u8;
    let a = (c1.a() as f32 + (c2.a() as f32 - c1.a() as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(r, g, b, a)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionIconType {
    Up,
    Down,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        matches!(self, ThemeMode::Dark)
    }

    pub fn toggle(&mut self) {
        *self = match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    pub fn bg_window(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(14, 17, 26) 
        } else {
            Color32::from_rgb(241, 245, 249) 
        }
    }

    pub fn bg_card(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(22, 25, 37) 
        } else {
            Color32::from_rgb(255, 255, 255) 
        }
    }

    pub fn bg_inner_card(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(29, 34, 50) 
        } else {
            Color32::from_rgb(248, 250, 252) 
        }
    }

    pub fn border_card(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(40, 46, 66) 
        } else {
            Color32::from_rgb(226, 232, 240) 
        }
    }

    pub fn border_inner_card(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(45, 52, 75)
        } else {
            Color32::from_rgb(226, 232, 240)
        }
    }

    pub fn primary(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(59, 130, 246) 
        } else {
            Color32::from_rgb(37, 99, 235) 
        }
    }

    pub fn text_primary(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(248, 250, 252)
        } else {
            Color32::from_rgb(15, 23, 42)
        }
    }

    pub fn text_secondary(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(148, 163, 184) 
        } else {
            Color32::from_rgb(100, 116, 139) 
        }
    }

    pub fn pill_inactive_bg(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(33, 38, 56)
        } else {
            Color32::from_rgb(241, 245, 249)
        }
    }

    pub fn pill_inactive_text(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(148, 163, 184)
        } else {
            Color32::from_rgb(71, 85, 105)
        }
    }

    pub fn input_bg(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(18, 21, 32)
        } else {
            Color32::from_rgb(255, 255, 255)
        }
    }

    pub fn input_border(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(45, 52, 75)
        } else {
            Color32::from_rgb(203, 213, 225)
        }
    }

    pub fn run_green(&self) -> Color32 {
        Color32::from_rgb(16, 185, 129) 
    }

    pub fn stop_red(&self) -> Color32 {
        Color32::from_rgb(239, 68, 68) 
    }

    pub fn danger_text(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgb(248, 113, 113)
        } else {
            Color32::from_rgb(220, 38, 38)
        }
    }

    pub fn card_frame(&self) -> Frame {
        Frame::new()
            .fill(self.bg_card())
            .corner_radius(CornerRadius::same(14))
            .stroke(Stroke::new(1.0, self.border_card()))
            .inner_margin(Margin::same(14))
    }

    pub fn inner_card_frame(&self) -> Frame {
        Frame::new()
            .fill(self.bg_inner_card())
            .corner_radius(CornerRadius::same(10))
            .stroke(Stroke::new(1.0, self.border_inner_card()))
            .inner_margin(Margin::symmetric(14, 12))
    }

    pub fn show_inner_card<R>(&self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
        self.inner_card_frame().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui)
        })
    }

    pub fn top_bar_pill_container(&self) -> Frame {
        Frame::new()
            .fill(self.pill_inactive_bg())
            .corner_radius(CornerRadius::same(20))
            .inner_margin(Margin::symmetric(4, 4))
    }

    pub fn animated_tab_bar<T: Copy + PartialEq>(
        &self,
        ui: &mut egui::Ui,
        id_salt: &str,
        options: &[(&str, T)],
        active: &mut T,
    ) -> bool {
        let count = options.len();
        if count == 0 {
            return false;
        }

        let target_idx = options.iter().position(|(_, val)| val == active).unwrap_or(0);
        let anim_id = ui.make_persistent_id(id_salt);
        let current_pos = ui.ctx().animate_value_with_time(anim_id, target_idx as f32, 0.24);

        let mut changed = false;

        self.top_bar_pill_container().show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);

            let min_btn_w = 76.0;
            let btn_h = 28.0;

            let mut tab_rects = Vec::with_capacity(count);
            let mut responses = Vec::with_capacity(count);

            for (title, val) in options.iter() {
                let text_w = ui.painter().layout_no_wrap(
                    title.to_string(),
                    egui::FontId::proportional(13.0),
                    Color32::WHITE,
                ).size().x;
                let desired_w = (text_w + 24.0).max(min_btn_w);

                let (rect, resp) = ui.allocate_exact_size(Vec2::new(desired_w, btn_h), egui::Sense::click());
                if resp.clicked() && *val != *active {
                    *active = *val;
                    changed = true;
                    ui.ctx().request_repaint();
                }
                tab_rects.push(rect);
                responses.push(resp);
            }

            let base_idx = (current_pos.floor() as usize).min(count - 1);
            let next_idx = (base_idx + 1).min(count - 1);
            let t = (current_pos - base_idx as f32).clamp(0.0, 1.0);

            let r_base = tab_rects[base_idx];
            let r_next = tab_rects[next_idx];

            let min_x = r_base.min.x + (r_next.min.x - r_base.min.x) * t;
            let max_x = r_base.max.x + (r_next.max.x - r_base.max.x) * t;
            let min_y = r_base.min.y + (r_next.min.y - r_base.min.y) * t;
            let max_y = r_base.max.y + (r_next.max.y - r_base.max.y) * t;

            let mut pill_rect = Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));

            let velocity = target_idx as f32 - current_pos;
            let stretch = velocity.abs() * 14.0;
            if velocity > 0.01 {
                pill_rect.max.x += stretch;
                pill_rect = pill_rect.shrink2(Vec2::new(0.0, (stretch * 0.1).min(1.5)));
            } else if velocity < -0.01 {
                pill_rect.min.x -= stretch;
                pill_rect = pill_rect.shrink2(Vec2::new(0.0, (stretch * 0.1).min(1.5)));
            }

            let painter = ui.painter();
            let glow_color = self.primary().gamma_multiply(0.28);
            painter.rect_filled(pill_rect.expand(2.0), CornerRadius::same(16), glow_color);
            painter.rect_filled(pill_rect, CornerRadius::same(14), self.primary());

            for (i, (title, _)) in options.iter().enumerate() {
                let rect = tab_rects[i];
                let resp = &responses[i];

                let proximity = (1.0 - (current_pos - i as f32).abs()).clamp(0.0, 1.0);
                let hover_factor = ui.ctx().animate_bool_with_time(
                    resp.id.with("tab_hover"),
                    resp.hovered() && proximity < 0.5,
                    0.15,
                );

                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                let base_color = lerp_color(self.pill_inactive_text(), self.text_primary(), hover_factor);
                let text_color = lerp_color(base_color, Color32::WHITE, proximity);

                let y_lift = proximity * 1.5;
                let text_pos = egui::pos2(rect.center().x, rect.center().y - y_lift);

                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    *title,
                    egui::FontId::proportional(13.0),
                    text_color,
                );
            }
        });

        changed
    }

    pub fn animated_subtab_bar<T: Copy + PartialEq>(
        &self,
        ui: &mut egui::Ui,
        id_salt: &str,
        options: &[(&str, T)],
        active: &mut T,
    ) -> bool {
        let count = options.len();
        if count == 0 {
            return false;
        }

        let target_idx = options.iter().position(|(_, val)| val == active).unwrap_or(0);
        let anim_id = ui.make_persistent_id(id_salt);
        let current_pos = ui.ctx().animate_value_with_time(anim_id, target_idx as f32, 0.24);

        let mut changed = false;

        Frame::new()
            .fill(self.pill_inactive_bg())
            .corner_radius(CornerRadius::same(16))
            .inner_margin(Margin::symmetric(4, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);

                    let min_btn_w = 88.0;
                    let btn_h = 30.0;

                    let mut tab_rects = Vec::with_capacity(count);
                    let mut responses = Vec::with_capacity(count);

                    for (title, val) in options.iter() {
                        let text_w = ui.painter().layout_no_wrap(
                            title.to_string(),
                            egui::FontId::proportional(13.0),
                            Color32::WHITE,
                        ).size().x;
                        let desired_w = (text_w + 28.0).max(min_btn_w);

                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(desired_w, btn_h), egui::Sense::click());
                        if resp.clicked() && *val != *active {
                            *active = *val;
                            changed = true;
                            ui.ctx().request_repaint();
                        }
                        tab_rects.push(rect);
                        responses.push(resp);
                    }

                    let base_idx = (current_pos.floor() as usize).min(count - 1);
                    let next_idx = (base_idx + 1).min(count - 1);
                    let t = (current_pos - base_idx as f32).clamp(0.0, 1.0);

                    let r_base = tab_rects[base_idx];
                    let r_next = tab_rects[next_idx];

                    let min_x = r_base.min.x + (r_next.min.x - r_base.min.x) * t;
                    let max_x = r_base.max.x + (r_next.max.x - r_base.max.x) * t;
                    let min_y = r_base.min.y + (r_next.min.y - r_base.min.y) * t;
                    let max_y = r_base.max.y + (r_next.max.y - r_base.max.y) * t;

                    let mut pill_rect = Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));

                    let velocity = target_idx as f32 - current_pos;
                    let stretch = velocity.abs() * 14.0;
                    if velocity > 0.01 {
                        pill_rect.max.x += stretch;
                        pill_rect = pill_rect.shrink2(Vec2::new(0.0, (stretch * 0.1).min(1.5)));
                    } else if velocity < -0.01 {
                        pill_rect.min.x -= stretch;
                        pill_rect = pill_rect.shrink2(Vec2::new(0.0, (stretch * 0.1).min(1.5)));
                    }

                    let painter = ui.painter();
                    let glow_color = self.primary().gamma_multiply(0.28);
                    painter.rect_filled(pill_rect.expand(2.0), CornerRadius::same(14), glow_color);
                    painter.rect_filled(pill_rect, CornerRadius::same(12), self.primary());

                    for (i, (title, _)) in options.iter().enumerate() {
                        let rect = tab_rects[i];
                        let resp = &responses[i];

                        let proximity = (1.0 - (current_pos - i as f32).abs()).clamp(0.0, 1.0);
                        let hover_factor = ui.ctx().animate_bool_with_time(
                            resp.id.with("subtab_hover"),
                            resp.hovered() && proximity < 0.5,
                            0.15,
                        );

                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        let base_color = lerp_color(self.pill_inactive_text(), self.text_primary(), hover_factor);
                        let text_color = lerp_color(base_color, Color32::WHITE, proximity);

                        let y_lift = proximity * 1.5;
                        let text_pos = egui::pos2(rect.center().x, rect.center().y - y_lift);

                        painter.text(
                            text_pos,
                            egui::Align2::CENTER_CENTER,
                            *title,
                            egui::FontId::proportional(13.0),
                            text_color,
                        );
                    }
                });
            });

        changed
    }

    pub fn primary_button_animated(&self, ui: &mut egui::Ui, text: &str, min_size: Vec2) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(min_size, egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("pri_btn_hover"), resp.hovered(), 0.15);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let normal_bg = self.primary();
        let hovered_bg = if self.is_dark() {
            Color32::from_rgb(96, 165, 250)
        } else {
            Color32::from_rgb(59, 130, 246)
        };
        let bg = lerp_color(normal_bg, hovered_bg, hover_t);

        let painter = ui.painter();
        if hover_t > 0.01 {
            let glow = normal_bg.gamma_multiply(0.35 * hover_t);
            painter.rect_filled(rect.expand(2.0 * hover_t), CornerRadius::same(18), glow);
        }

        let draw_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.8)
        } else {
            rect.expand(0.5 * hover_t)
        };

        painter.rect_filled(draw_rect, CornerRadius::same(18), bg);

        painter.text(
            draw_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(13.5),
            Color32::WHITE,
        );

        resp
    }

    pub fn secondary_button_animated(&self, ui: &mut egui::Ui, text: &str, min_size: Vec2) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(min_size, egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("sec_btn_hover"), resp.hovered(), 0.15);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let bg_normal = self.pill_inactive_bg();
        let bg_hover = if self.is_dark() {
            Color32::from_rgb(45, 52, 75)
        } else {
            Color32::from_rgb(226, 232, 240)
        };
        let bg = lerp_color(bg_normal, bg_hover, hover_t);

        let stroke_color = lerp_color(self.border_inner_card(), self.primary(), hover_t);

        let draw_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.8)
        } else {
            rect.expand(0.5 * hover_t)
        };

        let painter = ui.painter();
        if hover_t > 0.01 {
            let glow = self.primary().gamma_multiply(0.2 * hover_t);
            painter.rect_filled(draw_rect.expand(1.0 * hover_t), CornerRadius::same(18), glow);
        }

        painter.rect_filled(draw_rect, CornerRadius::same(18), bg);
        painter.rect_stroke(draw_rect, CornerRadius::same(18), Stroke::new(1.0, stroke_color), egui::StrokeKind::Inside);

        let hovered_text_color = if self.is_dark() { Color32::WHITE } else { self.text_primary() };
        let text_color = lerp_color(self.text_primary(), hovered_text_color, hover_t);
        painter.text(
            draw_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(13.5),
            text_color,
        );

        resp
    }

    #[allow(dead_code)]
    pub fn danger_button_animated(&self, ui: &mut egui::Ui, text: &str, min_size: Vec2) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(min_size, egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("danger_btn_hover"), resp.hovered(), 0.15);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let bg_normal = self.stop_red();
        let bg_hover = Color32::from_rgb(248, 113, 113);
        let bg = lerp_color(bg_normal, bg_hover, hover_t);

        let draw_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.8)
        } else {
            rect.expand(0.5 * hover_t)
        };

        let painter = ui.painter();
        if hover_t > 0.01 {
            let glow = bg_normal.gamma_multiply(0.3 * hover_t);
            painter.rect_filled(draw_rect.expand(1.5 * hover_t), CornerRadius::same(14), glow);
        }

        painter.rect_filled(draw_rect, CornerRadius::same(14), bg);

        painter.text(
            draw_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(13.0),
            Color32::WHITE,
        );

        resp
    }

    pub fn action_panel_frame(&self) -> Frame {
        let bg = if self.is_dark() {
            Color32::from_rgb(20, 24, 35) 
        } else {
            Color32::from_rgb(243, 246, 250) 
        };
        let border = if self.is_dark() {
            Color32::from_rgb(42, 50, 72)
        } else {
            Color32::from_rgb(218, 226, 237)
        };
        Frame::new()
            .fill(bg)
            .corner_radius(CornerRadius::same(12))
            .stroke(Stroke::new(1.0, border))
            .inner_margin(Margin::symmetric(14, 12))
    }

    pub fn show_action_panel<R>(&self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
        self.action_panel_frame().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui)
        })
    }

    pub fn full_width_slider<'a>(
        &self,
        ui: &mut egui::Ui,
        slider: egui::Slider<'a>,
    ) -> egui::Response {
        let width = ui.available_width().max(60.0);
        ui.add_sized([width, 20.0], slider)
    }

    pub fn action_add_button(&self, ui: &mut egui::Ui, text: &str) -> egui::Response {
        let font_id = egui::FontId::proportional(13.0);
        let text_w = ui.painter().layout_no_wrap(text.to_string(), font_id.clone(), Color32::WHITE).size().x;
        let desired_w = (text_w + 24.0).max(84.0);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(desired_w, 28.0), egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("act_add_anim"), resp.hovered(), 0.15);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let bg_normal = if self.is_dark() {
            Color32::from_rgb(33, 40, 60)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        let bg_hover = if self.is_dark() {
            Color32::from_rgb(45, 56, 88)
        } else {
            Color32::from_rgb(238, 242, 255)
        };
        let bg = lerp_color(bg_normal, bg_hover, hover_t);
        let stroke_color = lerp_color(
            if self.is_dark() { Color32::from_rgb(55, 65, 95) } else { Color32::from_rgb(203, 213, 225) },
            self.primary(),
            hover_t,
        );

        let draw_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.8)
        } else {
            rect.expand(0.5 * hover_t)
        };

        let painter = ui.painter();
        if hover_t > 0.01 {
            painter.rect_filled(draw_rect.expand(1.5 * hover_t), CornerRadius::same(8), self.primary().gamma_multiply(0.22 * hover_t));
        }
        painter.rect_filled(draw_rect, CornerRadius::same(8), bg);
        painter.rect_stroke(draw_rect, CornerRadius::same(8), Stroke::new(1.0, stroke_color), egui::StrokeKind::Inside);

        let text_color = lerp_color(self.text_primary(), self.primary(), hover_t);
        painter.text(
            draw_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_id,
            text_color,
        );

        resp
    }

    #[allow(dead_code)]
    pub fn action_pill(&self, ui: &mut egui::Ui, text: &str, is_selected: bool) -> egui::Response {
        let font_id = egui::FontId::proportional(12.5);
        let desired_w = ui.available_width().max(160.0);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(desired_w, 30.0), egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("act_pill_anim"), resp.hovered(), 0.15);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let (bg, text_color, border_color) = if is_selected {
            if self.is_dark() {
                
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
            let bg_normal = if self.is_dark() { Color32::from_rgb(26, 31, 46) } else { Color32::from_rgb(255, 255, 255) };
            let bg_hover = if self.is_dark() { Color32::from_rgb(38, 46, 68) } else { Color32::from_rgb(241, 245, 249) };
            (
                lerp_color(bg_normal, bg_hover, hover_t),
                lerp_color(self.text_secondary(), self.text_primary(), hover_t),
                lerp_color(self.border_inner_card(), self.primary().gamma_multiply(0.7), hover_t),
            )
        };

        let draw_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.6)
        } else {
            rect.expand(0.4 * hover_t)
        };

        let painter = ui.painter();
        if is_selected || hover_t > 0.01 {
            let glow = self.primary().gamma_multiply(if is_selected { 0.28 } else { 0.15 * hover_t });
            painter.rect_filled(draw_rect.expand(1.2), CornerRadius::same(8), glow);
        }
        painter.rect_filled(draw_rect, CornerRadius::same(8), bg);
        painter.rect_stroke(draw_rect, CornerRadius::same(8), Stroke::new(if is_selected { 1.5 } else { 1.0 }, border_color), egui::StrokeKind::Inside);

        if is_selected {
            let accent_rect = Rect::from_min_max(
                egui::pos2(draw_rect.min.x + 2.0, draw_rect.min.y + 4.0),
                egui::pos2(draw_rect.min.x + 5.5, draw_rect.max.y - 4.0),
            );
            painter.rect_filled(accent_rect, CornerRadius::same(2), self.primary());
        }

        painter.text(
            egui::pos2(draw_rect.min.x + if is_selected { 15.0 } else { 12.0 }, draw_rect.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            font_id,
            text_color,
        );

        resp
    }

    pub fn action_micro_button_at(
        &self,
        ui: &mut egui::Ui,
        rect: Rect,
        id_salt: &str,
        icon_type: ActionIconType,
        enabled: bool,
    ) -> egui::Response {
        let resp = ui.interact(rect, ui.id().with(id_salt), egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("hover"), resp.hovered() && enabled, 0.12);

        if enabled && resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let painter = ui.painter();
        let center = rect.center();

        let (base_bg, hover_bg, icon_color) = match icon_type {
            ActionIconType::Up | ActionIconType::Down => {
                let base = if self.is_dark() { Color32::from_rgba_premultiplied(32, 38, 56, 120) } else { Color32::from_rgba_premultiplied(241, 245, 249, 140) };
                let hover = if self.is_dark() { Color32::from_rgb(59, 130, 246) } else { Color32::from_rgb(37, 99, 235) };
                let icon_col = if !enabled {
                    if self.is_dark() { Color32::from_rgb(55, 62, 80) } else { Color32::from_rgb(203, 213, 225) }
                } else if hover_t > 0.01 {
                    Color32::WHITE
                } else {
                    self.text_secondary().gamma_multiply(0.85)
                };
                (base, hover, icon_col)
            }
            ActionIconType::Remove => {
                let base = if self.is_dark() { Color32::from_rgba_premultiplied(45, 25, 30, 110) } else { Color32::from_rgba_premultiplied(254, 242, 242, 140) };
                let hover = self.stop_red();
                let icon_col = if !enabled {
                    if self.is_dark() { Color32::from_rgb(70, 50, 55) } else { Color32::from_rgb(220, 190, 195) }
                } else if hover_t > 0.01 {
                    Color32::WHITE
                } else {
                    self.danger_text().gamma_multiply(0.8)
                };
                (base, hover, icon_col)
            }
        };

        let bg = lerp_color(base_bg, hover_bg, hover_t);
        let draw_rect = if resp.is_pointer_button_down_on() && enabled {
            rect.shrink(0.8)
        } else {
            rect.expand(0.4 * hover_t)
        };

        if hover_t > 0.01 {
            let glow = hover_bg.gamma_multiply(0.25 * hover_t);
            painter.rect_filled(draw_rect.expand(1.0), CornerRadius::same(6), glow);
        }
        painter.rect_filled(draw_rect, CornerRadius::same(6), bg);

        match icon_type {
            ActionIconType::Up => {
                let half_w = 4.0;
                let half_h = 3.0;
                let p1 = egui::pos2(center.x, center.y - half_h);
                let p2 = egui::pos2(center.x - half_w, center.y + half_h);
                let p3 = egui::pos2(center.x + half_w, center.y + half_h);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    icon_color,
                    Stroke::NONE,
                ));
            }
            ActionIconType::Down => {
                let half_w = 4.0;
                let half_h = 3.0;
                let p1 = egui::pos2(center.x, center.y + half_h);
                let p2 = egui::pos2(center.x - half_w, center.y - half_h);
                let p3 = egui::pos2(center.x + half_w, center.y - half_h);
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    icon_color,
                    Stroke::NONE,
                ));
            }
            ActionIconType::Remove => {
                let r = 3.6;
                painter.line_segment(
                    [egui::pos2(center.x - r, center.y - r), egui::pos2(center.x + r, center.y + r)],
                    Stroke::new(1.6, icon_color),
                );
                painter.line_segment(
                    [egui::pos2(center.x + r, center.y - r), egui::pos2(center.x - r, center.y + r)],
                    Stroke::new(1.6, icon_color),
                );
            }
        }

        resp
    }

    #[allow(dead_code)]
    pub fn action_micro_button(
        &self,
        ui: &mut egui::Ui,
        id_salt: &str,
        icon_type: ActionIconType,
        enabled: bool,
    ) -> egui::Response {
        let size = Vec2::splat(22.0);
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
        let _ = resp;
        self.action_micro_button_at(ui, rect, id_salt, icon_type, enabled)
    }

    pub fn run_stop_button(&self, ui: &mut egui::Ui, is_running: bool) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(120.0, 34.0), egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("run_stop_hover"), resp.hovered(), 0.15);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let base_bg = if is_running { self.stop_red() } else { self.run_green() };
        let bright_bg = if is_running {
            Color32::from_rgb(248, 113, 113)
        } else {
            Color32::from_rgb(52, 211, 153)
        };
        let bg = lerp_color(base_bg, bright_bg, hover_t);

        let painter = ui.painter();
        if hover_t > 0.01 {
            let glow = base_bg.gamma_multiply(0.35 * hover_t);
            painter.rect_filled(rect.expand(2.2 * hover_t), CornerRadius::same(17), glow);
        }

        let draw_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.8)
        } else {
            rect.expand(0.5 * hover_t)
        };

        painter.rect_filled(draw_rect, CornerRadius::same(17), bg);

        let icon_center = egui::pos2(draw_rect.min.x + 28.0, draw_rect.center().y);
        if is_running {
            let sq_size = 10.0;
            let sq_rect = egui::Rect::from_center_size(icon_center, Vec2::splat(sq_size));
            painter.rect_filled(sq_rect, CornerRadius::same(2), Color32::WHITE);
        } else {
            let p1 = egui::pos2(icon_center.x - 4.5, icon_center.y - 6.5);
            let p2 = egui::pos2(icon_center.x - 4.5, icon_center.y + 6.5);
            let p3 = egui::pos2(icon_center.x + 5.5, icon_center.y);
            painter.add(egui::Shape::convex_polygon(
                vec![p1, p2, p3],
                Color32::WHITE,
                Stroke::NONE,
            ));
        }

        let label = if is_running { "STOP" } else { "RUN" };
        painter.text(
            egui::pos2(draw_rect.min.x + 48.0, draw_rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        resp
    }

    pub fn status_info(&self, ui: &mut egui::Ui, status_text: &str, is_active: bool) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Status Info:")
                    .color(self.text_secondary())
                    .size(13.0),
            );
            ui.add_space(4.0);
            let base_color = if is_active {
                if status_text == "RECORDING" {
                    self.stop_red()
                } else {
                    self.run_green()
                }
            } else {
                Color32::from_rgb(148, 163, 184)
            };

            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
            let center = dot_rect.center();
            let painter = ui.painter();

            if is_active {
                let time = ui.input(|i| i.time);
                
                let pulse_t = ((time * std::f64::consts::PI * 1.6).sin() * 0.5 + 0.5) as f32;
                ui.ctx().request_repaint();

                let halo_radius = 4.0 + pulse_t * 3.5;
                let halo_alpha = (0.35 - pulse_t * 0.25).max(0.0);
                painter.circle_filled(center, halo_radius, base_color.gamma_multiply(halo_alpha));
                painter.circle_filled(center, 3.8, base_color);
            } else {
                painter.circle_filled(center, 3.5, base_color);
            }

            ui.label(
                egui::RichText::new(status_text)
                    .color(base_color)
                    .size(13.0)
                    .strong(),
            );
        });
    }

    pub fn tag_button(&self, ui: &mut egui::Ui, tag: &str) -> egui::Response {
        let font_id = egui::FontId::proportional(11.5);
        let text_w = ui.painter().layout_no_wrap(tag.to_string(), font_id.clone(), Color32::WHITE).size().x;
        let desired_w = (text_w + 14.0).max(42.0);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(desired_w, 24.0), egui::Sense::click());
        let hover_t = ui.ctx().animate_bool_with_time(resp.id.with("tag_hover"), resp.hovered(), 0.15);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let bg = lerp_color(self.pill_inactive_bg(), if self.is_dark() { Color32::from_rgb(45, 52, 75) } else { Color32::from_rgb(226, 232, 240) }, hover_t);
        let stroke_color = lerp_color(self.border_inner_card(), self.primary(), hover_t);

        let draw_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.5)
        } else {
            rect.expand(0.5 * hover_t)
        };

        let painter = ui.painter();
        if hover_t > 0.01 {
            painter.rect_filled(draw_rect.expand(1.0 * hover_t), CornerRadius::same(6), self.primary().gamma_multiply(0.2 * hover_t));
        }
        painter.rect_filled(draw_rect, CornerRadius::same(6), bg);
        painter.rect_stroke(draw_rect, CornerRadius::same(6), Stroke::new(1.0, stroke_color), egui::StrokeKind::Inside);

        let hovered_text_color = if self.is_dark() { Color32::WHITE } else { self.primary() };
        let text_color = lerp_color(self.text_primary(), hovered_text_color, hover_t);
        painter.text(
            draw_rect.center(),
            egui::Align2::CENTER_CENTER,
            tag,
            font_id,
            text_color,
        );

        resp
    }

    pub fn empty_state_centered(&self, ui: &mut egui::Ui, title: &str, subtitle: &str) {
        let total_avail = ui.available_height();
        let content_h = 160.0;
        let top_pad = ((total_avail - content_h) * 0.45).max(10.0);

        ui.vertical_centered(|ui| {
            ui.add_space(top_pad);

            let circle_size = 72.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(circle_size), egui::Sense::hover());
            let painter = ui.painter();

            let bg_circle = if self.is_dark() {
                Color32::from_rgb(30, 45, 78)
            } else {
                Color32::from_rgb(219, 234, 254)
            };

            painter.circle_stroke(
                rect.center(),
                circle_size * 0.5 + 4.0,
                Stroke::new(1.0, self.primary().gamma_multiply(0.25)),
            );

            painter.circle_filled(rect.center(), circle_size * 0.5, bg_circle);
            painter.circle_stroke(
                rect.center(),
                circle_size * 0.5,
                Stroke::new(2.0, self.primary()),
            );

            let c = rect.center();
            let p1 = egui::pos2(c.x - 8.0, c.y - 12.0);
            let p2 = egui::pos2(c.x - 8.0, c.y + 12.0);
            let p3 = egui::pos2(c.x + 12.0, c.y);
            painter.add(egui::Shape::convex_polygon(
                vec![p1, p2, p3],
                self.primary(),
                Stroke::NONE,
            ));

            ui.add_space(18.0);
            ui.label(
                egui::RichText::new(title)
                    .size(20.0)
                    .strong()
                    .color(self.text_primary()),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(subtitle)
                    .size(13.5)
                    .color(self.text_secondary()),
            );
        });
    }

    pub fn empty_state_left_panel(&self, ui: &mut egui::Ui, text: &str) {
        let total_avail = ui.available_height();
        let content_h = 120.0;
        let top_pad = ((total_avail - content_h) * 0.45).max(10.0);

        ui.vertical_centered(|ui| {
            ui.add_space(top_pad);

            let circle_size = 48.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(circle_size), egui::Sense::hover());
            let painter = ui.painter();

            let bg_circle = self.bg_inner_card();
            painter.circle_filled(rect.center(), circle_size * 0.5, bg_circle);
            painter.circle_stroke(
                rect.center(),
                circle_size * 0.5,
                Stroke::new(1.0, self.border_inner_card()),
            );

            let c = rect.center();
            let doc_w = 16.0;
            let doc_h = 22.0;
            let doc_rect = egui::Rect::from_center_size(c, Vec2::new(doc_w, doc_h));
            painter.rect_stroke(
                doc_rect,
                CornerRadius::same(3),
                Stroke::new(1.5, self.text_secondary()),
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [egui::pos2(c.x - 4.5, c.y - 4.0), egui::pos2(c.x + 4.5, c.y - 4.0)],
                Stroke::new(1.2, self.text_secondary()),
            );
            painter.line_segment(
                [egui::pos2(c.x - 4.5, c.y), egui::pos2(c.x + 4.5, c.y)],
                Stroke::new(1.2, self.text_secondary()),
            );
            painter.line_segment(
                [egui::pos2(c.x - 4.5, c.y + 4.0), egui::pos2(c.x + 2.0, c.y + 4.0)],
                Stroke::new(1.2, self.text_secondary()),
            );

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(text)
                    .size(13.0)
                    .color(self.text_secondary()),
            );
        });
    }
    
    pub fn custom_dropdown(
        &self,
        ui: &mut egui::Ui,
        id_salt: &str,
        selected_text: &str,
        width: f32,
        options: &[String],
        selected_idx: usize,
        mut on_select: impl FnMut(usize),
    ) {
        let combo_id = ui.make_persistent_id(id_salt);
        let mut is_open = ui.data_mut(|d| d.get_temp::<bool>(combo_id).unwrap_or(false));

        let (btn_rect, btn_resp) = ui.allocate_exact_size(Vec2::new(width, 32.0), egui::Sense::click());
        if btn_resp.clicked() {
            is_open = !is_open;
            ui.data_mut(|d| d.insert_temp(combo_id, is_open));
        }
        if btn_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let screen_rect = ui.ctx()
            .input(|i| i.raw.screen_rect)
            .unwrap_or(Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(1080.0, 790.0)));

        let prev_btn_pos = ui.data_mut(|d| d.get_temp::<egui::Pos2>(combo_id.with("last_pos")));
        if is_open {
            if let Some(prev) = prev_btn_pos {
                if prev.distance(btn_rect.min) > 30.0 || !screen_rect.intersects(btn_rect) {
                    is_open = false;
                    ui.data_mut(|d| d.insert_temp(combo_id, false));
                }
            }
        }
        ui.data_mut(|d| d.insert_temp(combo_id.with("last_pos"), btn_rect.min));

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            is_open = false;
            ui.data_mut(|d| d.insert_temp(combo_id, false));
        }

        let item_h = 28.0;
        let padding_v = 10.0;
        let desired_content_h = (options.len() as f32 * item_h) + padding_v;
        let max_dropdown_h = 240.0;
        let ideal_h = desired_content_h.min(max_dropdown_h);

        let space_below = (screen_rect.bottom() - btn_rect.max.y - 12.0).max(0.0);
        let space_above = (btn_rect.min.y - screen_rect.top() - 12.0).max(0.0);

        let open_upwards = space_below < ideal_h && space_above > space_below;

        let (actual_h, popup_y) = if open_upwards {
            let avail = space_above.min(max_dropdown_h).max(60.0);
            let h = ideal_h.min(avail);
            (h, btn_rect.min.y - 4.0 - h)
        } else {
            let avail = space_below.min(max_dropdown_h).max(60.0);
            let h = ideal_h.min(avail);
            (h, btn_rect.max.y + 4.0)
        };

        let mut popup_x = btn_rect.min.x;
        if popup_x + width > screen_rect.right() - 8.0 {
            popup_x = (screen_rect.right() - 8.0 - width).max(screen_rect.left() + 8.0);
        }
        if popup_x < screen_rect.left() + 8.0 {
            popup_x = screen_rect.left() + 8.0;
        }
        let popup_w = width.min(screen_rect.width() - 16.0);

        let popup_rect = Rect::from_min_size(
            egui::pos2(popup_x, popup_y),
            Vec2::new(popup_w, actual_h),
        );

        if is_open && ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos().or_else(|| i.pointer.latest_pos())) {
                if !btn_rect.contains(pos) && !popup_rect.expand(6.0).contains(pos) {
                    is_open = false;
                    ui.data_mut(|d| d.insert_temp(combo_id, false));
                }
            }
        }

        let painter = ui.painter();
        let bg = if is_open { self.bg_inner_card() } else { self.input_bg() };
        let border = if is_open { self.primary() } else { self.input_border() };
        painter.rect_filled(btn_rect, CornerRadius::same(8), bg);
        painter.rect_stroke(btn_rect, CornerRadius::same(8), Stroke::new(1.0, border), egui::StrokeKind::Inside);

        painter.text(
            egui::pos2(btn_rect.min.x + 10.0, btn_rect.center().y),
            egui::Align2::LEFT_CENTER,
            selected_text,
            egui::FontId::proportional(13.0),
            self.text_primary(),
        );

        let arrow_x = btn_rect.max.x - 14.0;
        let arrow_y = btn_rect.center().y;
        let arrow_stroke = Stroke::new(1.8, if is_open { self.primary() } else { self.text_secondary() });
        if is_open {
            painter.line_segment([egui::pos2(arrow_x - 4.0, arrow_y + 2.0), egui::pos2(arrow_x, arrow_y - 2.0)], arrow_stroke);
            painter.line_segment([egui::pos2(arrow_x, arrow_y - 2.0), egui::pos2(arrow_x + 4.0, arrow_y + 2.0)], arrow_stroke);
        } else {
            painter.line_segment([egui::pos2(arrow_x - 4.0, arrow_y - 2.0), egui::pos2(arrow_x, arrow_y + 2.0)], arrow_stroke);
            painter.line_segment([egui::pos2(arrow_x, arrow_y + 2.0), egui::pos2(arrow_x + 4.0, arrow_y - 2.0)], arrow_stroke);
        }

        if is_open {
            let area = egui::Area::new(combo_id.with("popup"))
                .order(egui::Order::Foreground)
                .fixed_pos(popup_rect.min);

            area.show(ui.ctx(), |ui| {
                let popup_frame = Frame::new()
                    .fill(self.bg_card())
                    .corner_radius(CornerRadius::same(8))
                    .stroke(Stroke::new(1.0, self.border_card()))
                    .inner_margin(Margin::symmetric(4, 4));

                popup_frame.show(ui, |ui| {
                    ui.set_width(popup_w - 8.0);
                    egui::ScrollArea::vertical()
                        .max_height(actual_h - 8.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let avail_w = ui.available_width();
                            for (i, opt) in options.iter().enumerate() {
                                let is_sel = i == selected_idx;
                                let (item_rect, item_resp) = ui.allocate_exact_size(Vec2::new(avail_w, item_h), egui::Sense::click());
                                if item_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if item_resp.clicked() {
                                    on_select(i);
                                    ui.data_mut(|d| d.insert_temp(combo_id, false));
                                }

                                let p = ui.painter();
                                if is_sel {
                                    p.rect_filled(item_rect, CornerRadius::same(6), self.primary());
                                    p.text(
                                        egui::pos2(item_rect.min.x + 10.0, item_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        opt,
                                        egui::FontId::proportional(12.5),
                                        Color32::WHITE,
                                    );

                                    let chk_x = item_rect.max.x - 16.0;
                                    let chk_y = item_rect.center().y;
                                    let chk_stroke = Stroke::new(2.0, Color32::WHITE);
                                    p.line_segment([egui::pos2(chk_x - 5.0, chk_y), egui::pos2(chk_x - 1.5, chk_y + 3.5)], chk_stroke);
                                    p.line_segment([egui::pos2(chk_x - 1.5, chk_y + 3.5), egui::pos2(chk_x + 5.0, chk_y - 4.0)], chk_stroke);
                                } else if item_resp.hovered() {
                                    let hov_col = if self.is_dark() { Color32::from_rgb(38, 45, 66) } else { Color32::from_rgb(238, 242, 255) };
                                    p.rect_filled(item_rect, CornerRadius::same(6), hov_col);
                                    p.text(
                                        egui::pos2(item_rect.min.x + 10.0, item_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        opt,
                                        egui::FontId::proportional(12.5),
                                        self.text_primary(),
                                    );
                                } else {
                                    p.text(
                                        egui::pos2(item_rect.min.x + 10.0, item_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        opt,
                                        egui::FontId::proportional(12.5),
                                        self.text_primary(),
                                    );
                                }
                            }
                        });
                });
            });
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = if self.is_dark() {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.panel_fill = self.bg_window();
        visuals.window_fill = self.bg_card();
        visuals.window_stroke = Stroke::new(1.0, self.border_card());
        visuals.window_corner_radius = CornerRadius::same(16);

        visuals.widgets.noninteractive.bg_fill = self.bg_card();
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_card());
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_primary());
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

        visuals.widgets.inactive.bg_fill = self.input_bg();
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.input_border());
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_primary());
        visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

        visuals.widgets.hovered.bg_fill = if self.is_dark() {
            Color32::from_rgb(38, 45, 66)
        } else {
            Color32::from_rgb(238, 242, 255)
        };
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.primary());
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, if self.is_dark() { Color32::WHITE } else { self.text_primary() });
        visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

        visuals.widgets.active.bg_fill = self.primary();
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.primary());
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.active.corner_radius = CornerRadius::same(8);

        visuals.widgets.open.bg_fill = self.bg_inner_card();
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.primary());
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.text_primary());
        visuals.widgets.open.corner_radius = CornerRadius::same(8);

        visuals.widgets.noninteractive.expansion = 0.0;
        visuals.widgets.inactive.expansion = 0.0;
        visuals.widgets.hovered.expansion = 0.0;
        visuals.widgets.active.expansion = 0.0;
        visuals.widgets.open.expansion = 0.0;

        visuals.selection.bg_fill = self.primary();
        visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.slider_trailing_fill = true;

        ctx.set_visuals(visuals);

        let egui_theme = if self.is_dark() { egui::Theme::Dark } else { egui::Theme::Light };
        let mut style = (*ctx.style_of(egui_theme)).clone();
        style.animation_time = 0.0;
        style.spacing.button_padding = Vec2::new(10.0, 6.0);
        style.spacing.window_margin = Margin::symmetric(20, 18);
        style.spacing.combo_height = 260.0;
        style.spacing.scroll = egui::style::ScrollStyle {
            bar_width: 8.0,
            bar_inner_margin: 6.0,
            bar_outer_margin: 4.0,
            floating: false,
            ..style.spacing.scroll
        };
        ctx.set_style_of(egui_theme, style);
    }
}
