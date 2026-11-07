use eframe::egui::{self, Color32, CornerRadius, Vec2};
use crate::ui::ThemeMode;
use crate::core::privileges;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum AboutSubTab {
    #[default]
    AboutApp,
    MainTab,
    RecPlayTab,
}

pub struct AboutTab {
    current_subtab: AboutSubTab,
    udev_installed: bool,
    permission_status_msg: Option<(String, bool)>,
    show_reset_modal: bool,
    reset_status_msg: Option<(String, bool)>,
}

impl Default for AboutTab {
    fn default() -> Self {
        Self {
            current_subtab: AboutSubTab::AboutApp,
            udev_installed: privileges::has_passwordless_access(),
            permission_status_msg: None,
            show_reset_modal: false,
            reset_status_msg: None,
        }
    }
}

impl AboutTab {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_reset_result(&mut self, msg: String, success: bool) {
        self.reset_status_msg = Some((msg, success));
    }

    pub fn render(&mut self, ui: &mut egui::Ui, theme: &ThemeMode) -> bool {
        let mut reset_confirmed = false;

        theme.card_frame().show(ui, |ui| {
            ui.set_min_size(Vec2::new(ui.available_width(), ui.available_height()));
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("oGiuMacro")
                        .size(22.0)
                        .strong()
                        .color(theme.primary()),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("v1.0.0")
                        .size(14.0)
                        .color(theme.text_secondary()),
                );
                ui.label(
                    egui::RichText::new("• Developed by oGiu")
                        .size(14.0)
                        .color(theme.text_secondary()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("GitHub Repository", "https://github.com/o-giu/oGiuMacro");
                });
            });

            ui.add_space(10.0);

            theme.animated_subtab_bar(
                ui,
                "about_subtabs",
                &[
                    ("About App", AboutSubTab::AboutApp),
                    ("Main Tab", AboutSubTab::MainTab),
                    ("Rec/Play", AboutSubTab::RecPlayTab),
                ],
                &mut self.current_subtab,
            );

            ui.add_space(12.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                match self.current_subtab {
                    AboutSubTab::AboutApp => {
                        self.render_about_app(ui, theme);
                    }
                    AboutSubTab::MainTab => {
                        self.render_main_tab_docs(ui, theme);
                    }
                    AboutSubTab::RecPlayTab => {
                        self.render_rec_play_docs(ui, theme);
                    }
                }
            });
        });

        if self.show_reset_modal {
            egui::Window::new("Confirm Factory Reset")
                .collapsible(false)
                .resizable(false)
                .min_width(420.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(
                        egui::RichText::new("Are you sure you want to delete all application data?")
                            .size(14.0)
                            .strong()
                            .color(theme.stop_red()),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("This action will permanently delete ~/.config/oGiuMacro/ and wipe all saved macros, recordings, and settings from memory. This cannot be undone.")
                            .size(13.0)
                            .color(theme.text_primary()),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("Delete Everything").color(Color32::WHITE).strong(),
                        )
                        .fill(theme.stop_red())
                        .corner_radius(CornerRadius::same(10));

                        if ui.add_sized([140.0, 32.0], del_btn).clicked() {
                            self.show_reset_modal = false;
                            reset_confirmed = true;
                        }

                        let cancel_btn = egui::Button::new("Cancel")
                            .fill(theme.pill_inactive_bg())
                            .corner_radius(CornerRadius::same(10));
                        if ui.add_sized([100.0, 32.0], cancel_btn).clicked() {
                            self.show_reset_modal = false;
                        }
                    });
                });
        }

        reset_confirmed
    }

    fn render_about_app(&mut self, ui: &mut egui::Ui, theme: &ThemeMode) {
        ui.label(
            egui::RichText::new("oGiuMacro is a modern, high-performance automation application built in Rust. It provides precise control over system inputs with kernel-level evdev capture and uinput emulation.")
                .color(theme.text_primary())
                .size(14.0),
        );
        ui.add_space(10.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Global Features")
                    .size(16.0)
                    .strong()
                    .color(theme.primary()),
            );
            ui.add_space(6.0);

            let bullets = [
                ("Emergency Stop", "Press ESC 3 times rapidly to immediately abort any running macro or recording."),
                ("Import/Export", "Easily share and backup your macros in JSON format."),
                ("Save Configs", "Click 'Save All Settings' to persist macros, hotkeys, and preferences in ~/.config/oGiuMacro/."),
                ("Macro List", "Organize, select, and manage your macros efficiently with dedicated per-item deletion."),
                ("Global Hotkeys", "Control execution seamlessly using customizable F1-F12 keys that work across Wayland and X11."),
                ("Tab Isolation", "Switching tabs automatically stops active recordings or playbacks to prevent input conflicts."),
                ("Hardware Spoofing", "Clone real USB Vendor/Product IDs at kernel level for anti-cheat and compatibility."),
                ("Dark & Light Mode", "Full support for modern dark and light interfaces with adaptive contrast."),
                ("Factory Reset", "Safely wipe all local configuration and reset data from the Danger Zone."),
            ];

            for (title, desc) in bullets {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("•").color(theme.primary()).strong());
                    ui.label(egui::RichText::new(title).strong().color(theme.text_primary()));
                    ui.label(egui::RichText::new(format!("- {}", desc)).color(theme.text_secondary()));
                });
            }
        });

        ui.add_space(10.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("System Permissions (Run Without Root / Password)")
                    .size(16.0)
                    .strong()
                    .color(theme.primary()),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("By default, Linux requires administrator rights to access `/dev/uinput` and `/dev/input/event*`. You can install a udev rule to grant your user account direct access, so oGiuMacro will never ask for a password again.")
                    .color(theme.text_secondary())
                    .size(13.0),
            );
            ui.add_space(8.0);

            let mut checked = self.udev_installed;
            let cb_text = if checked {
                "Run without root password (access active)"
            } else {
                "Run without root password (install udev rule)"
            };

            if ui.checkbox(&mut checked, egui::RichText::new(cb_text).strong().color(theme.text_primary())).changed() {
                if checked {
                    match privileges::install_udev_rule() {
                        Ok(()) => {
                            self.udev_installed = privileges::has_passwordless_access();
                            self.permission_status_msg = Some((
                                "Udev rule installed successfully! You can now launch oGiuMacro without root password.".to_string(),
                                true,
                            ));
                        }
                        Err(e) => {
                            self.udev_installed = privileges::has_passwordless_access();
                            self.permission_status_msg = Some((
                                format!("Failed to install rule: {}", e),
                                false,
                            ));
                        }
                    }
                } else {
                    match privileges::uninstall_udev_rule() {
                        Ok(()) => {
                            self.udev_installed = privileges::has_passwordless_access();
                            self.permission_status_msg = Some((
                                "Udev rule removed successfully. Default security restored.".to_string(),
                                true,
                            ));
                        }
                        Err(e) => {
                            self.udev_installed = privileges::has_passwordless_access();
                            self.permission_status_msg = Some((
                                format!("Failed to remove rule: {}", e),
                                false,
                            ));
                        }
                    }
                }
            }

            if let Some((msg, success)) = &self.permission_status_msg {
                ui.add_space(4.0);
                let color = if *success {
                    egui::Color32::from_rgb(46, 204, 113)
                } else {
                    egui::Color32::from_rgb(231, 76, 60)
                };
                ui.label(egui::RichText::new(msg).color(color).size(12.0).strong());
            }
        });

        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("Note: Linux system permissions (uinput/evdev access via pkexec or udev rules) are required for low-level input control.")
                .color(theme.text_secondary())
                .size(12.0)
                .italics(),
        );

        ui.add_space(10.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Danger Zone (Factory Reset / Clear All Data)")
                    .size(16.0)
                    .strong()
                    .color(theme.stop_red()),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Permanently deletes the configuration directory (~/.config/oGiuMacro/) and wipes all saved macros, recordings, and custom hardware profiles from memory.")
                    .color(theme.text_secondary())
                    .size(13.0),
            );
            ui.add_space(8.0);

            if theme.danger_button_animated(ui, "Reset All Data / Factory Reset", Vec2::new(220.0, 32.0)).clicked() {
                self.show_reset_modal = true;
            }

            if let Some((msg, success)) = &self.reset_status_msg {
                ui.add_space(4.0);
                let color = if *success {
                    egui::Color32::from_rgb(46, 204, 113)
                } else {
                    egui::Color32::from_rgb(231, 76, 60)
                };
                ui.label(egui::RichText::new(msg).color(color).size(12.0).strong());
            }
        });
    }

    fn render_main_tab_docs(&self, ui: &mut egui::Ui, theme: &ThemeMode) {
        ui.label(
            egui::RichText::new("The Main Tab provides a sequence-based workflow where you can build customizable macros step-by-step.")
                .color(theme.text_primary())
                .size(14.0),
        );
        ui.add_space(10.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("How to Use")
                    .size(16.0)
                    .strong()
                    .color(theme.primary()),
            );
            ui.add_space(6.0);

            let usage_steps = [
                "Click '+ New' on the left panel to create a new macro.",
                "Build your sequence using '+ Mouse', '+ Keys', '+ Scroll', and '+ Wait' action buttons.",
                "Adjust settings for each action, such as coordinates, drag-and-drop, hold time, and typing speed.",
                "Use the interactive 'Pick Position' crosshair overlay to easily capture screen coordinates.",
                "Click 'RUN' or press your assigned Start hotkey to execute the sequence.",
            ];

            for item in usage_steps {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("•").color(theme.primary()).strong());
                    ui.label(egui::RichText::new(item).color(theme.text_primary()));
                });
            }
        });

        ui.add_space(10.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("Key Features & Settings")
                    .size(16.0)
                    .strong()
                    .color(theme.primary()),
            );
            ui.add_space(6.0);

            let settings = [
                ("Loop Sequence", "Repeat the macro sequence indefinitely (0) or for a specific count."),
                ("Hardware Cloning", "Emulate genuine manufacturer devices (Logitech, Razer, Corsair) via USB bus ID spoofing."),
                ("Keyboard Layouts", "Maps evdev hardware keycodes for your physical keyboard layout (ABNT2 and US)."),
                ("Human-Like Behavior", "Adds micro-variations to delays and coordinate precision to break mechanical patterns."),
                ("Left-Handed Mouse", "Automatically inverts left and right click events during playback."),
                ("Image Recognition", "Screen template matching (Wait for Image) for Mouse, Keys, and Scroll with alpha mask and live RAM streaming."),
                ("Target Specific App & Stealth Mode", "Restricts actions to a specific window and allows periodic minimize/unminimize cycles."),
                ("Keyboard Tags & Shortcuts", "Type rich keystrokes using tags like [ENTER], [ESC], [CTRL+C], with customizable and validated Quick Tags."),
            ];

            for (title, desc) in settings {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("•").color(theme.primary()).strong());
                    ui.label(egui::RichText::new(title).strong().color(theme.text_primary()));
                    ui.label(egui::RichText::new(format!("- {}", desc)).color(theme.text_secondary()));
                });
            }
        });
    }

    fn render_rec_play_docs(&self, ui: &mut egui::Ui, theme: &ThemeMode) {
        ui.label(
            egui::RichText::new("The Rec/Play tab records physical mouse movements, clicks, and keystrokes in real-time for high-fidelity reproduction.")
                .color(theme.text_primary())
                .size(14.0),
        );
        ui.add_space(10.0);

        theme.show_inner_card(ui, |ui| {
            ui.label(
                egui::RichText::new("How to Record & Play")
                    .size(16.0)
                    .strong()
                    .color(theme.primary()),
            );
            ui.add_space(6.0);

            let rec_steps = [
                "Create a new recording or select an existing one.",
                "Click 'RECORD' or press the Start Record hotkey to begin capturing events from kernel evdev.",
                "Perform your input sequence freely.",
                "Click 'STOP REC' or press the Stop Record hotkey when finished.",
                "Click 'PLAY' or press the Play hotkey to replay with millisecond timing precision.",
            ];

            for item in rec_steps {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("•").color(theme.primary()).strong());
                    ui.label(egui::RichText::new(item).color(theme.text_primary()));
                });
            }
        });
    }
}
