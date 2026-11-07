use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke, Vec2};

use crate::core::ClickType;
use crate::ui::ThemeMode;
use super::MainTab;

impl MainTab {
    pub(super) fn render_action_editor(&mut self, ui: &mut egui::Ui, macro_idx: usize, act_idx: usize, theme: &ThemeMode) {
        let capture_engine = self.macros[macro_idx].capture_engine;
        let mut open_manage_images = false;
        let mut pick_mode_requested = None;
        let a = &mut self.macros[macro_idx].actions[act_idx];

        if a.action_type == crate::core::ActionType::Mouse || a.action_type == crate::core::ActionType::Keyboard || a.action_type == crate::core::ActionType::Scroll {
            theme.show_inner_card(ui, |ui| {
                ui.label(
                    egui::RichText::new("Image Recognition")
                        .color(theme.primary())
                        .size(14.0)
                        .strong(),
                );
                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    ui.checkbox(&mut a.use_image_recognition, "Wait for Image");
                    if a.use_image_recognition {
                        let btn = egui::Button::new("Manage Images")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add(btn).clicked() {
                            open_manage_images = true;
                        }
                        ui.label(
                            egui::RichText::new(format!("{} image(s)", a.image_paths.len()))
                                .color(theme.text_secondary())
                                .size(12.5),
                        );

                        let is_streaming = crate::core::is_screen_capture_worker_running();
                        let stream_btn_text = if is_streaming {
                            "● Stream Active"
                        } else {
                            "▶ Start Stream"
                        };
                        let stream_btn_color = if is_streaming {
                            theme.danger_text()
                        } else {
                            theme.primary()
                        };
                        let stream_btn = egui::Button::new(
                            egui::RichText::new(stream_btn_text).color(stream_btn_color).size(12.0)
                        )
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add(stream_btn).on_hover_text("Start or stop live screen streaming in RAM.\nTip: When prompted by the system, selecting 'Entire Screen' (Monitor) gives exact 1:1 mouse positioning for desktop targeting.").clicked() {
                            if is_streaming {
                                crate::core::stop_screen_capture_worker();
                            } else {
                                crate::core::start_screen_capture_worker_with_engine(capture_engine);
                            }
                        }
                    }
                });

                if a.use_image_recognition {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Match Threshold:");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.image_threshold, 0.1..=1.0).step_by(0.05));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Scan Interval (s):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.image_poll_rate, 0.0..=5.0).step_by(0.1));
                    });

                    if a.action_type == crate::core::ActionType::Mouse {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Mouse Target Location:").strong().size(12.5));
                        ui.horizontal(|ui| {
                            let r_img = ui.radio_value(&mut a.image_target_mouse, true, "Move to Detected Image");
                            let r_pos = ui.radio_value(&mut a.image_target_mouse, false, "Keep Configured Position (Pick Position)");
                            r_img.on_hover_text("When detected, cursor moves exactly to the center of the matching image on screen.\n(Note: When starting screen recording, select 'Entire Screen / Monitor' so coordinates match your desktop).");
                            r_pos.on_hover_text("When detected, cursor moves to the coordinates configured below in Mouse Settings (Pick Position).");
                        });
                    } else if a.action_type == crate::core::ActionType::Keyboard {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("When the image is detected on screen, the macro will automatically type the text configured below.")
                                .color(theme.text_secondary())
                                .size(12.5)
                                .italics(),
                        );
                    } else if a.action_type == crate::core::ActionType::Scroll {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Scroll Position:").strong().size(12.5));
                        ui.horizontal(|ui| {
                            let r_img = ui.radio_value(&mut a.image_target_mouse, true, "Move cursor to Detected Image before scrolling");
                            let r_curr = ui.radio_value(&mut a.image_target_mouse, false, "Scroll at current cursor position");
                            r_img.on_hover_text("When detected, cursor moves to the center of the detected image first, then scrolls.");
                            r_curr.on_hover_text("When detected, scrolls immediately wherever the mouse cursor currently is.");
                        });
                    }
                }
            });

            ui.add_space(8.0);
        }

        theme.show_inner_card(ui, |ui| {
            match a.action_type {
                crate::core::ActionType::Mouse => {
                    ui.label(
                        egui::RichText::new("Mouse Settings")
                            .color(theme.primary())
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(6.0);

                    let auto_target_image = a.use_image_recognition && a.image_target_mouse;

                    if a.mouse.click_type == ClickType::DragAndDrop {
                        ui.horizontal(|ui| {
                            ui.label("Start X:");
                            ui.add_enabled_ui(!auto_target_image, |ui| {
                                ui.add_sized([70.0, 24.0], egui::DragValue::new(&mut a.mouse.x).speed(1.0));
                            });
                            ui.add_space(4.0);
                            ui.label("Start Y:");
                            ui.add_enabled_ui(!auto_target_image, |ui| {
                                ui.add_sized([70.0, 24.0], egui::DragValue::new(&mut a.mouse.y).speed(1.0));
                            });
                            ui.add_space(6.0);
                            ui.add_enabled_ui(!auto_target_image, |ui| {
                                let pick_btn = egui::Button::new("Pick Start Position")
                                    .fill(theme.pill_inactive_bg())
                                    .corner_radius(CornerRadius::same(10));
                                let pick_resp = ui.add(pick_btn).on_hover_ui(|ui| {
                                    ui.set_max_width(280.0);
                                    ui.label(egui::RichText::new("🎯 Pick Start Position").strong().color(theme.primary()));
                                    ui.label("Opens a full-screen crosshair overlay to capture drag start coordinates.");
                                    ui.label(egui::RichText::new("• Click anywhere on screen to capture\n• Press ESC to cancel").color(theme.text_secondary()).size(11.5));
                                });
                                if pick_resp.clicked() {
                                    pick_mode_requested = Some(1);
                                }
                            });
                        });
                        let is_wayland = std::env::var("WAYLAND_DISPLAY").map(|v| !v.is_empty()).unwrap_or(false);
                        if is_wayland && !crate::core::which_command("slurp") {
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                ui.colored_label(theme.danger_text(), "⚠ 'slurp' missing for crosshair picker.");
                                let distro = crate::core::detect_linux_distro();
                                if let Some(cmd) = distro.install_command_for_missing(&["slurp"]) {
                                    ui.label(egui::RichText::new(format!("Run: {}", cmd)).monospace().size(11.5).color(theme.text_secondary()));
                                    if ui.button(egui::RichText::new("Copy").size(11.0)).clicked() {
                                        ui.ctx().copy_text(cmd);
                                    }
                                }
                            });
                        }
                        if auto_target_image {
                            ui.label(
                                egui::RichText::new("Start position will automatically be the detected image object on screen.")
                                    .color(theme.text_secondary())
                                    .size(11.5)
                                    .italics(),
                            );
                        }

                        ui.add_space(3.0);

                        ui.horizontal(|ui| {
                            ui.label("End X:");
                            ui.add_sized([70.0, 24.0], egui::DragValue::new(&mut a.mouse.end_x).speed(1.0));
                            ui.add_space(4.0);
                            ui.label("End Y:");
                            ui.add_sized([70.0, 24.0], egui::DragValue::new(&mut a.mouse.end_y).speed(1.0));
                            ui.add_space(6.0);
                            let pick_btn = egui::Button::new("Pick End Position")
                                .fill(theme.pill_inactive_bg())
                                .corner_radius(CornerRadius::same(10));
                            let pick_resp = ui.add(pick_btn).on_hover_ui(|ui| {
                                ui.set_max_width(280.0);
                                ui.label(egui::RichText::new("🎯 Pick End Position").strong().color(theme.primary()));
                                ui.label("Opens a full-screen crosshair overlay to capture drag end coordinates.");
                                ui.label(egui::RichText::new("• Click anywhere on screen to capture\n• Press ESC to cancel").color(theme.text_secondary()).size(11.5));
                            });
                            if pick_resp.clicked() {
                                pick_mode_requested = Some(2);
                            }
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add_enabled_ui(!auto_target_image, |ui| {
                                ui.add_sized([70.0, 24.0], egui::DragValue::new(&mut a.mouse.x).speed(1.0));
                            });
                            ui.add_space(4.0);
                            ui.label("Y:");
                            ui.add_enabled_ui(!auto_target_image, |ui| {
                                ui.add_sized([70.0, 24.0], egui::DragValue::new(&mut a.mouse.y).speed(1.0));
                            });
                            ui.add_space(6.0);
                            ui.add_enabled_ui(!auto_target_image, |ui| {
                                let pick_btn = egui::Button::new("Pick Position")
                                    .fill(theme.pill_inactive_bg())
                                    .corner_radius(CornerRadius::same(10));
                                let pick_resp = ui.add(pick_btn).on_hover_ui(|ui| {
                                    ui.set_max_width(280.0);
                                    ui.label(egui::RichText::new("🎯 Pick Position").strong().color(theme.primary()));
                                    ui.label("Activates a desktop-wide crosshair overlay to capture click coordinates.");
                                    ui.label(egui::RichText::new("• Screen dims subtly with blue crosshair & live X/Y\n• Click anywhere on screen to capture\n• Press ESC to cancel").color(theme.text_secondary()).size(11.5));
                                });
                                if pick_resp.clicked() {
                                    pick_mode_requested = Some(1);
                                }
                            });
                        });
                        let is_wayland = std::env::var("WAYLAND_DISPLAY").map(|v| !v.is_empty()).unwrap_or(false);
                        if is_wayland && !crate::core::which_command("slurp") {
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                ui.colored_label(theme.danger_text(), "⚠ 'slurp' missing for crosshair picker.");
                                let distro = crate::core::detect_linux_distro();
                                if let Some(cmd) = distro.install_command_for_missing(&["slurp"]) {
                                    ui.label(egui::RichText::new(format!("Run: {}", cmd)).monospace().size(11.5).color(theme.text_secondary()));
                                    if ui.button(egui::RichText::new("Copy").size(11.0)).clicked() {
                                        ui.ctx().copy_text(cmd);
                                    }
                                }
                            });
                        }
                        if auto_target_image {
                            ui.label(
                                egui::RichText::new("Coordinates will automatically be the detected image object on screen.")
                                    .color(theme.text_secondary())
                                    .size(11.5)
                                    .italics(),
                            );
                        }
                    }

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Click Type:");
                        let ct_options: Vec<String> = ClickType::ALL.iter().map(|c| c.display_name().to_string()).collect();
                        let cur_sel = ClickType::ALL.iter().position(|c| *c == a.mouse.click_type).unwrap_or(0);
                        theme.custom_dropdown(
                            ui,
                            &format!("click_type_dropdown_{}", act_idx),
                            a.mouse.click_type.display_name(),
                            220.0,
                            &ct_options,
                            cur_sel,
                            |idx| {
                                if idx < ClickType::ALL.len() {
                                    a.mouse.click_type = ClickType::ALL[idx];
                                }
                            },
                        );
                    });

                    if a.mouse.click_type != ClickType::DragAndDrop {
                        ui.horizontal(|ui| {
                            ui.label("Hold Time (s):");
                            theme.full_width_slider(ui, egui::Slider::new(&mut a.mouse.hold_time, 0.01..=5.0).step_by(0.01));
                        });
                    }

                    ui.horizontal(|ui| {
                        ui.label("Pre-delay (s):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.delay_time, 0.0..=10.0).step_by(0.05));
                    });
                }

                crate::core::ActionType::Keyboard => {
                    ui.label(
                        egui::RichText::new("Keyboard Settings")
                            .color(theme.primary())
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Text Input:");
                        let avail_w = ui.available_width();
                        ui.add_sized(
                            Vec2::new(avail_w, 32.0),
                            egui::TextEdit::singleline(&mut a.keyboard.text)
                                .hint_text("Type text or tags...")
                                .margin(Margin::symmetric(10, 6)),
                        );
                    });

                    let open_count = a.keyboard.text.chars().filter(|&c| c == '[').count();
                    let close_count = a.keyboard.text.chars().filter(|&c| c == ']').count();
                    if open_count != close_count {
                        ui.add_space(2.0);
                        ui.colored_label(theme.danger_text(), "⚠ Warning: Unclosed tag bracket detected (missing '[' or ']').");
                    }

                    ui.add_space(4.0);

                    let tags_tooltip = "Tag Reference & Shortcuts:\n\n\
                        • System combinations: [CTRL+C], [CTRL+V], [CTRL+A], [CTRL+Z], [CTRL+X], [WIN+D], [ALT+F4], [ALT+TAB]\n\
                        • Navigation: [UP], [DOWN], [LEFT], [RIGHT], [HOME], [END], [PGUP], [PGDN]\n\
                        • Editing & control: [ENTER], [ESC], [TAB], [SPACE], [BACKSPACE], [DEL], [INS], [PRTSC], [CAPSLOCK]\n\
                        • Function keys: [F1] through [F12]\n\n\
                        Click any tag chip below to insert it directly into your text input.";

                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("Quick Tags:")
                                .color(theme.text_secondary())
                                .size(12.5)
                                .strong(),
                        ).on_hover_text(tags_tooltip);

                        for tag in &self.custom_quick_tags {
                            let clean_display = if tag.starts_with('[') && tag.ends_with(']') && tag.len() >= 2 {
                                &tag[1..tag.len() - 1]
                            } else {
                                tag.as_str()
                            };
                            if theme.tag_button(ui, clean_display).on_hover_text(tags_tooltip).clicked() {
                                a.keyboard.text.push_str(tag);
                            }
                        }

                        let edit_tags_btn = egui::Button::new(
                            egui::RichText::new("⚙ Edit")
                                .size(11.5)
                                .color(theme.text_primary())
                        )
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(edit_tags_btn).on_hover_text("Customize your Quick Tags list (up to 16 tags)").clicked() {
                            self.new_tag_input.clear();
                            self.tag_input_error = None;
                            self.show_manage_tags_modal = true;
                        }

                        let toggle_label = if self.show_all_tags { "[-] Less Tags" } else { "[+] All Tags..." };
                        let toggle_btn = egui::Button::new(
                            egui::RichText::new(toggle_label)
                                .size(11.5)
                                .color(theme.primary())
                        )
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(toggle_btn).clicked() {
                            self.show_all_tags = !self.show_all_tags;
                        }
                    });

                    if self.show_all_tags {
                        ui.add_space(3.0);
                        egui::Frame::new()
                            .fill(theme.bg_inner_card())
                            .stroke(Stroke::new(1.0, theme.border_inner_card()))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(10, 8))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());

                                ui.horizontal_wrapped(|ui| {
                                    ui.label(egui::RichText::new("Shortcuts:").size(11.5).color(theme.text_secondary()));
                                    for tag in ["CTRL+V", "CTRL+C", "CTRL+X", "CTRL+A", "CTRL+Z", "WIN+D", "ALT+F4", "ALT+TAB"] {
                                        let tag_formatted = format!("[{}]", tag);
                                        if theme.tag_button(ui, tag).clicked() {
                                            a.keyboard.text.push_str(&tag_formatted);
                                        }
                                    }
                                });

                                ui.add_space(3.0);

                                ui.horizontal_wrapped(|ui| {
                                    ui.label(egui::RichText::new("Navigation:").size(11.5).color(theme.text_secondary()));
                                    for tag in ["UP", "DOWN", "LEFT", "RIGHT", "HOME", "END", "PGUP", "PGDN", "INS", "PRTSC", "CAPSLOCK"] {
                                        let tag_formatted = format!("[{}]", tag);
                                        if theme.tag_button(ui, tag).clicked() {
                                            a.keyboard.text.push_str(&tag_formatted);
                                        }
                                    }
                                });

                                ui.add_space(3.0);

                                ui.horizontal_wrapped(|ui| {
                                    ui.label(egui::RichText::new("Modifiers:").size(11.5).color(theme.text_secondary()));
                                    for tag in ["CTRL", "SHIFT", "ALT", "WIN", "ENTER", "ESC", "TAB", "BACKSPACE", "DEL", "SPACE"] {
                                        let tag_formatted = format!("[{}]", tag);
                                        if theme.tag_button(ui, tag).clicked() {
                                            a.keyboard.text.push_str(&tag_formatted);
                                        }
                                    }
                                });

                                ui.add_space(3.0);

                                ui.horizontal_wrapped(|ui| {
                                    ui.label(egui::RichText::new("Function:").size(11.5).color(theme.text_secondary()));
                                    for f in 1..=12 {
                                        let tag = format!("F{}", f);
                                        let tag_formatted = format!("[{}]", tag);
                                        if theme.tag_button(ui, &tag).clicked() {
                                            a.keyboard.text.push_str(&tag_formatted);
                                        }
                                    }
                                });
                            });
                    }

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Typing Speed:");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.keyboard.speed, 0.1..=5.0).step_by(0.1));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Pre-delay (s):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.delay_time, 0.0..=10.0).step_by(0.05));
                    });
                }

                crate::core::ActionType::Delay => {
                    ui.label(
                        egui::RichText::new("Wait Settings")
                            .color(theme.primary())
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Wait Time (s):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.delay_time, 0.01..=60.0).step_by(0.05));
                    });
                    ui.label(
                        egui::RichText::new("Pauses the macro for the set duration.")
                            .color(theme.text_secondary())
                            .size(12.5),
                    );
                }

                crate::core::ActionType::Scroll => {
                    ui.label(
                        egui::RichText::new("Scroll Settings")
                            .color(theme.primary())
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label("Direction:");
                        for dir in &crate::core::ScrollDirection::ALL {
                            let text = dir.display_name();
                            let is_sel = a.scroll.direction == *dir;
                            let btn = if is_sel {
                                egui::Button::new(egui::RichText::new(text).color(Color32::WHITE).strong())
                                    .fill(theme.primary())
                            } else {
                                egui::Button::new(egui::RichText::new(text).color(theme.text_secondary()))
                                    .fill(theme.pill_inactive_bg())
                            };
                            if ui.add(btn.corner_radius(CornerRadius::same(8))).clicked() {
                                a.scroll.direction = *dir;
                            }
                        }
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Amount (lines):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.scroll.amount, 1..=100).step_by(1.0));
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut a.scroll.smooth, "Smooth scrolling");
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Pre-delay (s):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut a.delay_time, 0.0..=10.0).step_by(0.05));
                    });
                }
            }
        });

        if open_manage_images {
            self.new_image_path.clear();
            self.image_path_error = None;
            self.show_manage_images_modal = true;
        }

        if let Some(mode) = pick_mode_requested {
            self.start_picking_coordinates(mode);
        }
    }


}
