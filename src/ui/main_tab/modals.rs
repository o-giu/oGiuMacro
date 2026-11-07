use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke, Vec2};

use crate::core::{HardwareProfile, Macro};
use crate::ui::ThemeMode;
use super::MainTab;

impl MainTab {
    pub(super) fn render_modals(&mut self, ctx: &egui::Context, theme: &ThemeMode) {
        if self.show_add_macro_modal {
            egui::Window::new("Add Macro")
                .collapsible(false)
                .resizable(false)
                .min_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Macro Name:").strong());
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_macro_name)
                            .hint_text("Enter unique macro name...")
                            .margin(Margin::symmetric(10, 8))
                            .desired_width(f32::INFINITY),
                    );
                    if self.new_macro_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), "Name is empty or already exists.");
                    }
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let create_btn = egui::Button::new(
                            egui::RichText::new("Create Macro").color(Color32::WHITE).strong(),
                        )
                        .fill(theme.primary())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([130.0, 32.0], create_btn).clicked() {
                            let name = self.new_macro_name.trim();
                            let exists = self.macros.iter().any(|m| m.name == name);
                            if !name.is_empty() && !exists {
                                let mut m = Macro::new(name);
                                m.quick_tags = self.custom_quick_tags.clone();
                                self.macros.push(m);
                                self.selected_macro_idx = Some(self.macros.len() - 1);
                                self.selected_action_idx = None;
                                self.show_add_macro_modal = false;
                            } else {
                                self.new_macro_error = true;
                            }
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.show_add_macro_modal = false;
                        }
                    });
                });
        }

        if self.show_remove_macro_modal {
            let target_name = self
                .macro_idx_to_remove
                .and_then(|i| self.macros.get(i))
                .map(|m| m.name.clone())
                .unwrap_or_else(|| "this macro".to_string());

            egui::Window::new("Confirm Delete Macro")
                .collapsible(false)
                .resizable(false)
                .min_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(format!("Are you sure you want to delete macro \"{}\"?", target_name)).size(13.5));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("Delete Macro").color(Color32::WHITE).strong(),
                        )
                        .fill(theme.stop_red())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([130.0, 32.0], del_btn).clicked() {
                            if let Some(del_idx) = self.macro_idx_to_remove {
                                if del_idx < self.macros.len() {
                                    self.macros.remove(del_idx);
                                    if self.macros.is_empty() {
                                        self.selected_macro_idx = None;
                                    } else if let Some(sel) = self.selected_macro_idx {
                                        if sel == del_idx {
                                            self.selected_macro_idx = Some(del_idx.min(self.macros.len() - 1));
                                        } else if sel > del_idx {
                                            self.selected_macro_idx = Some(sel - 1);
                                        }
                                    }
                                    self.selected_action_idx = None;
                                }
                            }
                            self.macro_idx_to_remove = None;
                            self.show_remove_macro_modal = false;
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.macro_idx_to_remove = None;
                            self.show_remove_macro_modal = false;
                        }
                    });
                });
        }

        if self.show_remove_action_modal {
            egui::Window::new("Confirm Remove Action")
                .collapsible(false)
                .resizable(false)
                .min_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Are you sure you want to remove this action?").size(13.5));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("Delete Action").color(Color32::WHITE).strong(),
                        )
                        .fill(theme.stop_red())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([130.0, 32.0], del_btn).clicked() {
                            let target_act_idx = self.action_idx_to_remove.or(self.selected_action_idx);
                            if let (Some(m_idx), Some(a_idx)) = (self.selected_macro_idx, target_act_idx) {
                                if m_idx < self.macros.len() && a_idx < self.macros[m_idx].actions.len() {
                                    self.macros[m_idx].actions.remove(a_idx);
                                    if self.macros[m_idx].actions.is_empty() {
                                        self.selected_action_idx = None;
                                    } else {
                                        let current_sel = self.selected_action_idx.unwrap_or(0);
                                        if current_sel >= self.macros[m_idx].actions.len() || current_sel == a_idx {
                                            self.selected_action_idx = Some(a_idx.saturating_sub(1).min(self.macros[m_idx].actions.len() - 1));
                                        } else if current_sel > a_idx {
                                            self.selected_action_idx = Some(current_sel - 1);
                                        }
                                    }
                                }
                            }
                            self.action_idx_to_remove = None;
                            self.show_remove_action_modal = false;
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.action_idx_to_remove = None;
                            self.show_remove_action_modal = false;
                        }
                    });
                });
        }

        if self.show_manage_images_modal {
            egui::Window::new("Manage Images")
                .collapsible(false)
                .resizable(false)
                .fixed_size([560.0, 520.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Select images using the file picker or type the path manually below:")
                            .size(12.5)
                            .color(theme.text_secondary()),
                    );
                    ui.add_space(6.0);

                    theme.show_inner_card(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new("💡 Tips for Best Recognition:")
                                    .strong()
                                    .size(12.0)
                                    .color(theme.primary()),
                            );
                        });
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new("• Use crisp, high-quality images captured directly at your native display scale.")
                                .size(11.5)
                                .color(theme.text_secondary()),
                        );
                        ui.label(
                            egui::RichText::new("• For moving objects on dynamic backgrounds, use PNG with transparent background leaving only the desired object.")
                                .size(11.5)
                                .color(theme.text_secondary()),
                        );
                    });
                    ui.add_space(8.0);

                    let browse_btn = egui::Button::new(
                        egui::RichText::new("📁 Browse / Choose Images...")
                            .color(Color32::WHITE)
                            .strong()
                    )
                    .fill(theme.primary())
                    .corner_radius(CornerRadius::same(10));

                    if ui.add_sized(Vec2::new(ui.available_width(), 34.0), browse_btn).clicked() {
                        let files = crate::core::pick_image_files();
                        if !files.is_empty() {
                            if let (Some(m_idx), Some(a_idx)) = (self.selected_macro_idx, self.selected_action_idx) {
                                if m_idx < self.macros.len() && a_idx < self.macros[m_idx].actions.len() {
                                    for path in files {
                                        let p = path.to_string_lossy().to_string();
                                        if !self.macros[m_idx].actions[a_idx].image_paths.contains(&p) {
                                            self.macros[m_idx].actions[a_idx].image_paths.push(p);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(egui::RichText::new("Or Enter Path Manually:").strong());
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_image_path)
                                .hint_text("/home/user/Pictures/target.png")
                                .margin(Margin::symmetric(10, 8))
                                .desired_width(ui.available_width() - 110.0),
                        );
                        let add_img_btn = egui::Button::new(
                            egui::RichText::new("Add Path").color(theme.text_primary()).strong()
                        )
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([100.0, 32.0], add_img_btn).clicked() {
                            let p = self.new_image_path.trim().to_string();
                            if p.is_empty() {
                                self.image_path_error = Some("Path cannot be empty.".to_string());
                            } else {
                                let path = std::path::Path::new(&p);
                                if !path.exists() {
                                    self.image_path_error = Some("File not found on disk.".to_string());
                                } else if !path.is_file() {
                                    self.image_path_error = Some("The path is a directory, not an image file.".to_string());
                                } else {
                                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                                    if !["png", "jpg", "jpeg", "bmp"].contains(&ext.as_str()) {
                                        self.image_path_error = Some(format!("Unsupported format '.{}'. Use PNG, JPG, or BMP.", ext));
                                    } else {
                                        if let (Some(m_idx), Some(a_idx)) = (self.selected_macro_idx, self.selected_action_idx) {
                                            if m_idx < self.macros.len() && a_idx < self.macros[m_idx].actions.len() {
                                                if !self.macros[m_idx].actions[a_idx].image_paths.contains(&p) {
                                                    self.macros[m_idx].actions[a_idx].image_paths.push(p);
                                                }
                                                self.new_image_path.clear();
                                                self.image_path_error = None;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });

                    if let Some(err) = &self.image_path_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), format!("⚠ {}", err));
                    }

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Configured Images for this Action:").strong());
                    ui.add_space(4.0);

                    let mut paths_to_remove = None;
                    if let (Some(m_idx), Some(a_idx)) = (self.selected_macro_idx, self.selected_action_idx) {
                        if m_idx < self.macros.len() && a_idx < self.macros[m_idx].actions.len() {
                            let action = &self.macros[m_idx].actions[a_idx];
                            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                if action.image_paths.is_empty() {
                                    ui.add_space(12.0);
                                    ui.label(egui::RichText::new("No images configured yet.").color(theme.text_secondary()).italics());
                                } else {
                                    for (i, path_str) in action.image_paths.iter().enumerate() {
                                        let file_path = std::path::Path::new(path_str);
                                        let exists = file_path.exists();
                                        let filename = file_path.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or(path_str);

                                        theme.show_inner_card(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                let status_icon = if exists { "✔ OK" } else { "✖ Missing" };
                                                let status_col = if exists { theme.run_green() } else { theme.danger_text() };
                                                ui.label(
                                                    egui::RichText::new(status_icon)
                                                        .color(status_col)
                                                        .size(11.0)
                                                        .strong(),
                                                );

                                                let text_area_width = (ui.available_width() - 85.0).max(120.0);
                                                ui.allocate_ui_with_layout(
                                                    Vec2::new(text_area_width, 0.0),
                                                    egui::Layout::top_down(egui::Align::Min),
                                                    |ui| {
                                                        ui.set_max_width(text_area_width);

                                                        let mut display_name = filename.to_string();
                                                        if display_name.chars().count() > 36 {
                                                            let start: String = display_name.chars().take(22).collect();
                                                            let end: String = display_name.chars().skip(display_name.chars().count().saturating_sub(11)).collect();
                                                            display_name = format!("{}...{}", start, end);
                                                        }

                                                        ui.label(
                                                            egui::RichText::new(format!("{}. {}", i + 1, display_name))
                                                                .strong()
                                                                .size(12.5)
                                                                .color(theme.text_primary()),
                                                        ).on_hover_text(format!("Filename: {}\nFull Path: {}", filename, path_str));

                                                        let mut truncated_path = path_str.clone();
                                                        if truncated_path.chars().count() > 46 {
                                                            let start: String = truncated_path.chars().take(18).collect();
                                                            let end: String = truncated_path.chars().skip(truncated_path.chars().count().saturating_sub(25)).collect();
                                                            truncated_path = format!("{}...{}", start, end);
                                                        }
                                                        ui.label(
                                                            egui::RichText::new(truncated_path)
                                                                .size(11.0)
                                                                .color(theme.text_secondary()),
                                                        ).on_hover_text(path_str);
                                                    },
                                                );

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let rem_btn = egui::Button::new(
                                                        egui::RichText::new("Remove").color(theme.danger_text()).size(12.0)
                                                    )
                                                    .fill(theme.pill_inactive_bg())
                                                    .corner_radius(CornerRadius::same(6));
                                                    if ui.add_sized([68.0, 26.0], rem_btn).clicked() {
                                                        paths_to_remove = Some(i);
                                                    }
                                                });
                                            });
                                        });
                                        ui.add_space(3.0);
                                    }
                                }
                            });
                        }
                    }

                    if let Some(i) = paths_to_remove {
                        if let (Some(m_idx), Some(a_idx)) = (self.selected_macro_idx, self.selected_action_idx) {
                            if m_idx < self.macros.len() && a_idx < self.macros[m_idx].actions.len() {
                                self.macros[m_idx].actions[a_idx].image_paths.remove(i);
                            }
                        }
                    }

                    ui.add_space(12.0);
                    let close_btn = egui::Button::new("Close")
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(10));
                    if ui.add_sized([90.0, 30.0], close_btn).clicked() {
                        self.show_manage_images_modal = false;
                        self.image_path_error = None;
                    }
                });
        }

        if self.show_manage_hardware_modal {
            egui::Window::new("Manage Global Hardware Profiles")
                .collapsible(false)
                .resizable(false)
                .min_width(420.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Profile Name:").strong());
                    ui.add_space(2.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_hw_name)
                            .hint_text("e.g. Gaming Mouse")
                            .margin(Margin::symmetric(10, 8))
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Vendor ID (Hex):");
                        let mut v_str = format!("{:04X}", self.new_hw_vendor);
                        if ui.add(egui::TextEdit::singleline(&mut v_str).margin(Margin::symmetric(8, 6)).desired_width(70.0)).changed() {
                            if let Ok(v) = u16::from_str_radix(&v_str, 16) {
                                self.new_hw_vendor = v;
                            }
                        }
                        ui.add_space(8.0);
                        ui.label("Product ID (Hex):");
                        let mut p_str = format!("{:04X}", self.new_hw_product);
                        if ui.add(egui::TextEdit::singleline(&mut p_str).margin(Margin::symmetric(8, 6)).desired_width(70.0)).changed() {
                            if let Ok(p) = u16::from_str_radix(&p_str, 16) {
                                self.new_hw_product = p;
                            }
                        }
                    });

                    if let Some(err_msg) = &self.new_hw_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), err_msg);
                    }

                    ui.add_space(8.0);
                    let add_prof_btn = egui::Button::new(
                        egui::RichText::new("Add Profile").color(Color32::WHITE).strong()
                    )
                    .fill(theme.primary())
                    .corner_radius(CornerRadius::same(10));

                    if ui.add_sized([120.0, 30.0], add_prof_btn).clicked() {
                        let name = self.new_hw_name.trim();
                        if name.is_empty() {
                            self.new_hw_error = Some("Profile name cannot be empty.".to_string());
                        } else if self.hardware_profiles.iter().any(|hp| hp.name.trim().eq_ignore_ascii_case(name)) {
                            self.new_hw_error = Some(format!("A profile with the name \"{}\" already exists.", name));
                        } else {
                            let new_profile = HardwareProfile {
                                name: name.to_string(),
                                vendor_id: self.new_hw_vendor,
                                product_id: self.new_hw_product,
                            };
                            self.hardware_profiles.push(new_profile.clone());
                            let new_idx = self.hardware_profiles.len() - 1;
                            self.selected_hw_idx = new_idx;
                            if let Some(m_idx) = self.selected_macro_idx {
                                if m_idx < self.macros.len() {
                                    self.macros[m_idx].hardware_idx = new_idx;
                                    self.macros[m_idx].hardware_profile = Some(new_profile);
                                }
                            }
                            let _ = crate::core::save_main_macros(&self.macros, &self.hardware_profiles);
                            self.new_hw_name.clear();
                            self.new_hw_error = None;
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Existing Hardware Profiles:").strong());
                    ui.add_space(4.0);

                    let mut remove_hw_idx = None;
                    egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                        for (i, hp) in self.hardware_profiles.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: [{:04X}:{:04X}]", hp.name, hp.vendor_id, hp.product_id));
                                if !hp.is_system_default() {
                                    let rem_btn = egui::Button::new(
                                        egui::RichText::new("Remove").color(theme.danger_text())
                                    )
                                    .fill(theme.pill_inactive_bg())
                                    .corner_radius(CornerRadius::same(6));
                                    if ui.add_sized([70.0, 24.0], rem_btn).clicked() {
                                        remove_hw_idx = Some(i);
                                    }
                                } else {
                                    ui.colored_label(theme.text_secondary(), "(Default)");
                                }
                            });
                        }
                    });

                    if let Some(idx) = remove_hw_idx {
                        if !self.hardware_profiles[idx].is_system_default() {
                            self.hardware_profiles.remove(idx);
                            for m in &mut self.macros {
                                if m.hardware_idx == idx {
                                    m.hardware_idx = 0;
                                    m.hardware_profile = self.hardware_profiles.get(0).cloned();
                                } else if m.hardware_idx > idx {
                                    m.hardware_idx -= 1;
                                }
                            }
                            if self.selected_hw_idx >= self.hardware_profiles.len() {
                                self.selected_hw_idx = self.hardware_profiles.len().saturating_sub(1);
                            }
                            let _ = crate::core::save_main_macros(&self.macros, &self.hardware_profiles);
                        }
                    }

                    ui.add_space(10.0);
                    let close_btn = egui::Button::new("Close")
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(10));
                    if ui.add_sized([90.0, 30.0], close_btn).clicked() {
                        self.show_manage_hardware_modal = false;
                    }
                });
        }

        if self.show_manage_tags_modal {
            egui::Window::new("Manage Quick Tags")
                .collapsible(false)
                .resizable(false)
                .fixed_size([460.0, 480.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("Add valid keyboard tags with brackets (e.g. [CTRL+C], [ENTER], [F5], max 16):")
                            .size(12.5)
                            .color(theme.text_secondary()),
                    );
                    ui.add_space(8.0);

                    let can_add = self.custom_quick_tags.len() < 16;
                    ui.horizontal(|ui| {
                        let text_edit = egui::TextEdit::singleline(&mut self.new_tag_input)
                            .hint_text("[CTRL+C]")
                            .margin(Margin::symmetric(10, 8))
                            .desired_width(ui.available_width() - 95.0);
                        ui.add_enabled(can_add, text_edit);

                        let add_btn = egui::Button::new(
                            egui::RichText::new("Add Tag").color(Color32::WHITE).strong()
                        )
                        .fill(if can_add { theme.primary() } else { theme.pill_inactive_bg() })
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_enabled(can_add, add_btn).clicked() {
                            let raw = self.new_tag_input.trim().to_ascii_uppercase();

                            if raw.is_empty() {
                                self.tag_input_error = Some("Tag cannot be empty.".to_string());
                            } else if !raw.starts_with('[') || !raw.ends_with(']') || raw.len() < 3 {
                                self.tag_input_error = Some("Tag must be enclosed in brackets, e.g. [ENTER] or [CTRL+C].".to_string());
                            } else if !crate::core::is_valid_keyboard_tag(&raw) {
                                self.tag_input_error = Some(format!("'{}' is not a valid keyboard tag or shortcut.", raw));
                            } else if self.custom_quick_tags.iter().any(|t| t.eq_ignore_ascii_case(&raw)) {
                                self.tag_input_error = Some("This tag is already in your Quick Tags list.".to_string());
                            } else {
                                self.custom_quick_tags.push(raw);
                                if let Some(idx) = self.selected_macro_idx {
                                    if idx < self.macros.len() {
                                        self.macros[idx].quick_tags = self.custom_quick_tags.clone();
                                    }
                                }
                                let _ = crate::core::save_custom_quick_tags(&self.custom_quick_tags);
                                self.new_tag_input.clear();
                                self.tag_input_error = None;
                            }
                        }
                    });

                    if let Some(err) = &self.tag_input_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), format!("⚠ {}", err));
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Current Quick Tags ({}/16):", self.custom_quick_tags.len()))
                                .strong()
                                .size(12.5),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let reset_btn = egui::Button::new(
                                egui::RichText::new("Reset to Default").size(11.5).color(theme.text_secondary())
                            )
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(6));

                            if ui.add(reset_btn).on_hover_text("Restore default 9 Quick Tags").clicked() {
                                self.custom_quick_tags = vec![
                                    "[CTRL+V]".to_string(),
                                    "[CTRL+C]".to_string(),
                                    "[CTRL+A]".to_string(),
                                    "[ENTER]".to_string(),
                                    "[ESC]".to_string(),
                                    "[TAB]".to_string(),
                                    "[SPACE]".to_string(),
                                    "[BACKSPACE]".to_string(),
                                    "[DEL]".to_string(),
                                ];
                                if let Some(idx) = self.selected_macro_idx {
                                    if idx < self.macros.len() {
                                        self.macros[idx].quick_tags = self.custom_quick_tags.clone();
                                    }
                                }
                                let _ = crate::core::save_custom_quick_tags(&self.custom_quick_tags);
                                self.tag_input_error = None;
                            }
                        });
                    });
                    ui.add_space(4.0);

                    let mut tag_to_remove = None;
                    egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                        if self.custom_quick_tags.is_empty() {
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new("No quick tags defined. Click 'Reset to Default' or add one above.").color(theme.text_secondary()).italics());
                        } else {
                            for (idx, tag) in self.custom_quick_tags.iter().enumerate() {
                                theme.show_inner_card(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{}.", idx + 1))
                                                .color(theme.text_secondary())
                                                .size(11.5),
                                        );
                                        ui.label(
                                            egui::RichText::new(tag)
                                                .strong()
                                                .size(12.5)
                                                .color(theme.primary()),
                                        );
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let del_btn = egui::Button::new(
                                                egui::RichText::new("Remove").color(theme.danger_text()).size(11.5)
                                            )
                                            .fill(theme.pill_inactive_bg())
                                            .corner_radius(CornerRadius::same(6));
                                            if ui.add_sized([60.0, 22.0], del_btn).clicked() {
                                                tag_to_remove = Some(idx);
                                            }
                                        });
                                    });
                                });
                                ui.add_space(2.0);
                            }
                        }
                    });

                    if let Some(idx) = tag_to_remove {
                        self.custom_quick_tags.remove(idx);
                        if let Some(m_idx) = self.selected_macro_idx {
                            if m_idx < self.macros.len() {
                                self.macros[m_idx].quick_tags = self.custom_quick_tags.clone();
                            }
                        }
                        let _ = crate::core::save_custom_quick_tags(&self.custom_quick_tags);
                    }

                    ui.add_space(10.0);
                    let close_btn = egui::Button::new("Close")
                        .fill(theme.pill_inactive_bg())
                        .corner_radius(CornerRadius::same(10));
                    if ui.add_sized([90.0, 30.0], close_btn).clicked() {
                        self.show_manage_tags_modal = false;
                        self.tag_input_error = None;
                    }
                });
        }

        if self.show_export_modal {
            egui::Window::new("Export Macro")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .max_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui: &mut egui::Ui| {
                    ui.label(egui::RichText::new("Exporting to ~/.config/oGiuMacro/workflows/").color(theme.text_secondary()));
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
                                .hint_text("Macro name")
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
                    let file_exists = !clean_base.is_empty() && crate::core::get_workflows_dir().join(format!("{}.json", clean_base)).exists();

                    if file_exists && !self.export_confirm_overwrite {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), "⚠ A saved workflow macro with this name already exists.");
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
                                self.export_error = Some(format!("A saved file named '{}.json' already exists in workflows. Click 'Overwrite' to replace it, or choose a different name.", clean_base));
                            } else {
                                let fname = format!("{}.json", clean_base);
                                if let Some(idx) = self.selected_macro_idx {
                                    if idx < self.macros.len() {
                                        self.macros[idx].quick_tags = self.custom_quick_tags.clone();
                                        if self.macros[idx].hardware_profile.is_none() {
                                            self.macros[idx].hardware_profile = self.hardware_profiles.get(self.macros[idx].hardware_idx).cloned();
                                        }
                                        match crate::core::export_macro(&self.macros[idx], &fname) {
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

        if self.show_import_modal {
            let mut to_delete_idx = None;

            egui::Window::new("Import Workflow Macro")
                .collapsible(false)
                .resizable(false)
                .min_width(480.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let header_title = if !self.import_show_all {
                            "Saved Workflow Macros (~/.config/oGiuMacro/workflows):"
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
                                    crate::core::list_exported_files_for_type(crate::core::MacroFileType::Workflow)
                                };
                            }
                        });
                    });
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let wf_active = !self.import_show_all;
                        let wf_count = crate::core::list_exported_files_for_type(crate::core::MacroFileType::Workflow).len();
                        let all_count = crate::core::list_exported_files().len();

                        let wf_btn = egui::Button::new(
                            egui::RichText::new(format!("Workflow Macros ({})", wf_count))
                                .size(11.5)
                                .strong()
                                .color(if wf_active { Color32::WHITE } else { theme.text_secondary() }),
                        )
                        .fill(if wf_active { theme.primary() } else { theme.pill_inactive_bg() })
                        .corner_radius(CornerRadius::same(8));

                        if ui.add(wf_btn).clicked() && !wf_active {
                            self.import_show_all = false;
                            self.selected_import_idx = None;
                            self.import_error = None;
                            self.available_import_files = crate::core::list_exported_files_for_type(crate::core::MacroFileType::Workflow);
                        }

                        let all_btn = egui::Button::new(
                            egui::RichText::new(format!("All Saved Files ({})", all_count))
                                .size(11.5)
                                .strong()
                                .color(if !wf_active { Color32::WHITE } else { theme.text_secondary() }),
                        )
                        .fill(if !wf_active { theme.primary() } else { theme.pill_inactive_bg() })
                        .corner_radius(CornerRadius::same(8));

                        if ui.add(all_btn).clicked() && wf_active {
                            self.import_show_all = true;
                            self.selected_import_idx = None;
                            self.import_error = None;
                            self.available_import_files = crate::core::list_exported_files();
                        }
                    });
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .id_salt("main_import_files_scroll")
                        .max_height(200.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.available_import_files.is_empty() {
                                ui.add_space(20.0);
                                ui.vertical_centered(|ui| {
                                    let msg = if !self.import_show_all {
                                        "No exported Workflow macros found in ~/.config/oGiuMacro/workflows."
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
                                    let is_compatible = file_type == crate::core::MacroFileType::Workflow;

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
                                                        if is_selected { Color32::WHITE } else { theme.primary() },
                                                        if is_selected { theme.primary() } else { theme.primary().gamma_multiply(0.20) },
                                                    ),
                                                    crate::core::MacroFileType::Recording => (
                                                        "REC / PLAY",
                                                        theme.stop_red(),
                                                        theme.stop_red().gamma_multiply(0.20),
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
                                                        self.import_error = Some("This file is a Recording macro (Rec/Play). Switch to the Rec/Play tab to import it.".to_string());
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
                                                let del_hover_t = ui.ctx().animate_bool_with_time(del_resp.id.with(format!("imp_del_{}", i)), del_resp.hovered(), 0.12);
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
                    let can_import = self.selected_import_idx.is_some() && selected_file_type == crate::core::MacroFileType::Workflow;

                    if let Some(err) = &self.import_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), format!("⚠ {}", err));
                    } else if can_import {
                        ui.add_space(4.0);
                        ui.colored_label(theme.run_green(), "✓ Compatible Workflow macro ready to import.");
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
                            resp.on_disabled_hover_text("Cannot import a Rec/Play Recording into Workflow. Switch to the Rec/Play tab.")
                        } else {
                            resp
                        };

                        if resp.clicked() {
                            if let Some(idx) = self.selected_import_idx {
                                if let Some(path) = self.available_import_files.get(idx) {
                                    match crate::core::import_macro(path) {
                                        Ok(mut imported) => {
                                            let base = imported.name.clone();
                                            let mut count = 1;
                                            while self.macros.iter().any(|m| m.name == imported.name) {
                                                imported.name = format!("{}_{}", base, count);
                                                count += 1;
                                            }
                                            if !imported.quick_tags.is_empty() {
                                                self.custom_quick_tags = imported.quick_tags.clone();
                                                let _ = crate::core::save_custom_quick_tags(&self.custom_quick_tags);
                                            }
                                            if let Some(ref prof) = imported.hardware_profile {
                                                if let Some(pos) = self.hardware_profiles.iter().position(|h| h.vendor_id == prof.vendor_id && h.product_id == prof.product_id) {
                                                    imported.hardware_idx = pos;
                                                } else {
                                                    self.hardware_profiles.push(prof.clone());
                                                    imported.hardware_idx = self.hardware_profiles.len() - 1;
                                                    let _ = crate::core::save_main_macros(&self.macros, &self.hardware_profiles);
                                                }
                                            }
                                            self.macros.push(imported);
                                            self.selected_macro_idx = Some(self.macros.len() - 1);
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

            egui::Window::new("Confirm Delete Saved Macro")
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
                                    crate::core::list_exported_files_for_type(crate::core::MacroFileType::Workflow)
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

        if self.show_hotkey_modal {
            egui::Window::new("Assign Hotkey")
                .collapsible(false)
                .resizable(false)
                .min_width(360.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let slot_name = match self.hotkey_target_slot {
                        0 => "Start Hotkey",
                        1 => "Stop Hotkey",
                        _ => "Pick Window Hotkey",
                    };
                    ui.label(egui::RichText::new(format!("Select F-Key for {}:", slot_name)).strong());
                    if self.hotkey_conflict_error {
                        ui.add_space(4.0);
                        ui.colored_label(theme.danger_text(), "Hotkey already in use by another action.");
                    }
                    ui.add_space(10.0);
                    egui::Grid::new("hotkey_grid").spacing(Vec2::new(8.0, 8.0)).show(ui, |ui| {
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
                                    if let Some(m_idx) = self.selected_macro_idx {
                                        if let Some(m) = self.macros.get(m_idx) {
                                            let conflict = match self.hotkey_target_slot {
                                                0 => m.stop_hotkey == code || m.pick_window_hotkey == code,
                                                1 => m.start_hotkey == code || m.pick_window_hotkey == code,
                                                _ => m.start_hotkey == code || m.stop_hotkey == code,
                                            };
                                            if conflict {
                                                self.hotkey_conflict_error = true;
                                            } else {
                                                if let Some(m_mut) = self.macros.get_mut(m_idx) {
                                                    match self.hotkey_target_slot {
                                                        0 => m_mut.start_hotkey = code,
                                                        1 => m_mut.stop_hotkey = code,
                                                        _ => m_mut.pick_window_hotkey = code,
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


}
