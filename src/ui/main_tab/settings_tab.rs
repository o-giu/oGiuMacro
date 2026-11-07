use eframe::egui::{self, CornerRadius, Margin, Vec2};

use crate::ui::ThemeMode;
use super::MainTab;

impl MainTab {
    pub(super) fn render_settings_tab(&mut self, ui: &mut egui::Ui, macro_idx: usize, theme: &ThemeMode) {
        let mut do_pick_window = false;
        let m = &mut self.macros[macro_idx];

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Random")
                    .color(theme.primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.checkbox(&mut m.human_like, "Human-like behavior");
            ui.label(
                egui::RichText::new("Simulates realistic hand gestures, natural Bézier curves, micro-tremors, and human typing cadence.")
                    .color(theme.text_secondary())
                    .size(12.0),
            );
        });

        ui.add_space(8.0);

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
                egui::RichText::new("Window")
                    .color(theme.primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);

            ui.checkbox(&mut m.target_window_enabled, "Target specific app");

            if m.target_window_enabled {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut m.target_method, 0, "List Selection");
                    ui.radio_value(&mut m.target_method, 1, "Manual Entry");
                });

                ui.add_space(4.0);

                if m.target_method == 0 {
                    ui.horizontal(|ui| {
                        let win_name = self
                            .detected_windows
                            .get(m.target_window_idx)
                            .map(|s| s.as_str())
                            .unwrap_or("Active Window");

                    theme.custom_dropdown(
                        ui,
                        "win_list_dropdown",
                        win_name,
                        220.0,
                        &self.detected_windows,
                        m.target_window_idx,
                        |idx| {
                            m.target_window_idx = idx;
                        },
                    );

                        let refresh_btn = egui::Button::new("Refresh List")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add(refresh_btn).clicked() {
                            self.detected_windows = crate::core::list_detected_windows();
                        }

                        let pick_btn = egui::Button::new("Pick Active Window")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add(pick_btn).clicked() {
                            do_pick_window = true;
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Window Name:");
                        let input_w = (ui.available_width() - 170.0).max(120.0);
                        ui.add_sized(
                            Vec2::new(input_w, 28.0),
                            egui::TextEdit::singleline(&mut m.custom_window_name)
                                .hint_text("Exact or partial window title")
                                .margin(Margin::symmetric(10, 6)),
                        );
                        let pick_btn = egui::Button::new("Pick Active Window")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add(pick_btn).clicked() {
                            do_pick_window = true;
                        }
                    });
                }

                if let Some(feedback_time) = self.pick_window_feedback {
                    if feedback_time.elapsed().as_secs_f32() < 2.5 {
                        ui.colored_label(theme.run_green(), "✓ Target window picked!");
                    }
                }

                if !crate::core::which_command("wmctrl") {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.colored_label(theme.danger_text(), "⚠ 'wmctrl' is not installed.");
                        let distro = crate::core::detect_linux_distro();
                        if let Some(cmd) = distro.install_command_for_missing(&["wmctrl"]) {
                            ui.label(egui::RichText::new(format!("Run: {}", cmd)).monospace().size(11.5).color(theme.text_secondary()));
                            if ui.button(egui::RichText::new("Copy").size(11.0)).clicked() {
                                ui.ctx().copy_text(cmd);
                            }
                        }
                    });
                }

                ui.add_space(6.0);
                ui.checkbox(&mut m.minimize_and_stay_active, "Stealth Mode (minimize & cycle)");
                if m.minimize_and_stay_active {
                    if !crate::core::which_command("xdotool") {
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.colored_label(theme.danger_text(), "⚠ 'xdotool' is required for window minimization.");
                            let distro = crate::core::detect_linux_distro();
                            if let Some(cmd) = distro.install_command_for_missing(&["xdotool"]) {
                                ui.label(egui::RichText::new(format!("Run: {}", cmd)).monospace().size(11.5).color(theme.text_secondary()));
                                if ui.button(egui::RichText::new("Copy").size(11.0)).clicked() {
                                    ui.ctx().copy_text(cmd);
                                }
                            }
                        });
                    }
                    ui.label(
                        egui::RichText::new("Macro pauses while window is minimized.")
                            .color(theme.text_secondary())
                            .size(12.5),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Active time (s):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut m.active_seconds, 0.5..=60.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Hidden time (s):");
                        theme.full_width_slider(ui, egui::Slider::new(&mut m.hide_seconds, 0.5..=60.0));
                    });
                }
            }
        });

        ui.add_space(8.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Hardware Cloning")
                    .color(theme.primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);

            let hw_btn = egui::Button::new("Manage Global Hardware List")
                .fill(theme.pill_inactive_bg())
                .corner_radius(CornerRadius::same(10));
            if ui.add(hw_btn).clicked() {
                self.show_manage_hardware_modal = true;
            }

            ui.add_space(4.0);
            if let Some(ref prof) = m.hardware_profile {
                if let Some(pos) = self.hardware_profiles.iter().position(|h| h.vendor_id == prof.vendor_id && h.product_id == prof.product_id) {
                    m.hardware_idx = pos;
                } else {
                    self.hardware_profiles.push(prof.clone());
                    m.hardware_idx = self.hardware_profiles.len() - 1;
                }
            }
            if m.hardware_idx >= self.hardware_profiles.len() {
                m.hardware_idx = 0;
            }
            ui.horizontal(|ui| {
                ui.label("Selected Hardware:");
                let current_name = self
                    .hardware_profiles
                    .get(m.hardware_idx)
                    .map(|h| h.name.as_str())
                    .unwrap_or("Generic Device");

                let hw_options: Vec<String> = self.hardware_profiles.iter().map(|h| h.name.clone()).collect();
                theme.custom_dropdown(
                    ui,
                    "hw_dropdown",
                    current_name,
                    220.0,
                    &hw_options,
                    m.hardware_idx,
                    |idx| {
                        m.hardware_idx = idx;
                        m.hardware_profile = self.hardware_profiles.get(idx).cloned();
                        self.selected_hw_idx = idx;
                    },
                );
            });
            ui.label(
                egui::RichText::new("This hardware ID will be spoofed when the macro starts.")
                    .color(theme.text_secondary())
                    .size(12.5),
            );
        });

        ui.add_space(8.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Keyboard Layout")
                    .color(theme.primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);

            let layout_names: Vec<String> = crate::core::KeyboardLayout::ALL
                .iter()
                .map(|l| l.display_name().to_string())
                .collect();
            let cur_layout_idx = crate::core::KeyboardLayout::ALL
                .iter()
                .position(|l| *l == m.keyboard_layout)
                .unwrap_or(0);

            ui.horizontal(|ui| {
                ui.label("Layout:");
                theme.custom_dropdown(
                    ui,
                    "keyboard_layout_dropdown",
                    m.keyboard_layout.display_name(),
                    260.0,
                    &layout_names,
                    cur_layout_idx,
                    |idx| {
                        if idx < crate::core::KeyboardLayout::ALL.len() {
                            m.keyboard_layout = crate::core::KeyboardLayout::ALL[idx];
                        }
                    },
                );
            });
            ui.label(
                egui::RichText::new("Maps evdev hardware keycodes for your physical keyboard layout.")
                    .color(theme.text_secondary())
                    .size(12.0),
            );
        });

        ui.add_space(8.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Screen Capture Engine (Wait for Image)")
                    .color(theme.primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(4.0);

            let engine_names: Vec<String> = crate::core::CaptureEngine::ALL
                .iter()
                .map(|e| e.display_name().to_string())
                .collect();
            let cur_engine_idx = crate::core::CaptureEngine::ALL
                .iter()
                .position(|e| *e == m.capture_engine)
                .unwrap_or(0);

            ui.horizontal(|ui| {
                ui.label("Backend:");
                theme.custom_dropdown(
                    ui,
                    "capture_engine_dropdown",
                    m.capture_engine.display_name(),
                    300.0,
                    &engine_names,
                    cur_engine_idx,
                    |idx| {
                        if idx < crate::core::CaptureEngine::ALL.len() {
                            m.capture_engine = crate::core::CaptureEngine::ALL[idx];
                        }
                    },
                );
            });

            ui.label(
                egui::RichText::new("• Portal ScreenCast: Modern standard (Wayland/Universal) direct in RAM at ~60 FPS.\n• Scrot: Fast capture via native utility (~20ms).\n• Fast X11/XShm: Direct RAM memory capture (~2-5ms / 200 FPS).")
                    .color(theme.text_secondary())
                    .size(12.0),
            );

            let is_kde = std::env::var("XDG_CURRENT_DESKTOP").map(|v| v.to_uppercase().contains("KDE")).unwrap_or(false);
            if is_kde {
                let eavesdropping_active = crate::core::is_kde_xwayland_eavesdropping_enabled();
                ui.add_space(6.0);
                if !eavesdropping_active {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚡ KDE Wayland detected:")
                                .color(theme.primary())
                                .strong()
                                .size(12.5),
                        );
                        if ui.button(egui::RichText::new("Enable Ultra-Fast Capture (~2ms)").strong()).clicked() {
                            match crate::core::enable_kde_xwayland_eavesdropping() {
                                Ok(_) => {
                                    self.kde_eavesdropping_status = Some("Configuration applied! Log out and log back in to KDE to load Xwayland at 200+ FPS.".to_string());
                                }
                                Err(err) => {
                                    self.kde_eavesdropping_status = Some(format!("Configuration error: {}", err));
                                }
                            }
                        }
                    });
                    ui.label(
                        egui::RichText::new("Tip: Allows apps to read screen in real time on Wayland (KDE KWin XwaylandEavesdrops=All).")
                            .color(theme.text_secondary())
                            .size(11.5),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("✔ KDE Xwayland Eavesdrops active (Instant ~2ms memory capture enabled)")
                            .color(theme.run_green())
                            .size(12.0),
                    );
                }

                if let Some(ref status_msg) = self.kde_eavesdropping_status {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(status_msg)
                            .color(theme.primary())
                            .size(12.0),
                    );
                }
            }
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
                ui.label("Start Hotkey:");
                let label = crate::core::fkey_name(m.start_hotkey);
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
                ui.label("Stop Hotkey:");
                let label = crate::core::fkey_name(m.stop_hotkey);
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
                ui.label("Pick Window Hotkey:");
                let label = crate::core::fkey_name(m.pick_window_hotkey);
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
        });

        if do_pick_window {
            self.pick_current_window();
        }
    }


}
