use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use evdev::KeyCode;

use crate::core::input_emulate::VirtualInputDevice;
use crate::core::{ActionType, HardwareProfile, Macro, RecordedMacro};

pub struct MacroEngine {
    is_running: Arc<AtomicBool>,
    current_action_idx: Arc<AtomicI32>,
}

impl MacroEngine {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            current_action_idx: Arc::new(AtomicI32::new(-1)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn current_action(&self) -> Option<usize> {
        let idx = self.current_action_idx.load(Ordering::SeqCst);
        if idx >= 0 {
            Some(idx as usize)
        } else {
            None
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.current_action_idx.store(-1, Ordering::SeqCst);
    }

    pub fn run_workflow(&self, macro_data: Macro, hardware: &[HardwareProfile]) {
        if self.is_active() {
            return;
        }

        let (vendor_id, product_id) = if let Some(hw) = &macro_data.hardware_profile {
            (hw.vendor_id, hw.product_id)
        } else if let Some(hw) = hardware.get(macro_data.hardware_idx) {
            (hw.vendor_id, hw.product_id)
        } else {
            (0x046d, 0xc703)
        };

        self.is_running.store(true, Ordering::SeqCst);
        self.current_action_idx.store(-1, Ordering::SeqCst);
        let flag = Arc::clone(&self.is_running);
        let action_idx_tracker = Arc::clone(&self.current_action_idx);

        thread::spawn(move || {
            let mut virtual_input = match VirtualInputDevice::new_with_hardware(
                "oGiuMacro Virtual Device",
                vendor_id,
                product_id,
            ) {
                Ok(dev) => dev,
                Err(_) => {
                    flag.store(false, Ordering::SeqCst);
                    action_idx_tracker.store(-1, Ordering::SeqCst);
                    return;
                }
            };

            let mut loop_iteration = 0;
            let infinite_loop = macro_data.is_loop && macro_data.loop_count == 0;
            let target_loops = if macro_data.is_loop {
                macro_data.loop_count.max(1)
            } else {
                1
            };

            let target_title = if macro_data.target_window_enabled {
                if macro_data.target_method == 1 {
                    macro_data.custom_window_name.clone()
                } else {
                    let wins = crate::core::list_detected_windows();
                    wins.get(macro_data.target_window_idx).cloned().unwrap_or_default()
                }
            } else {
                String::new()
            };

            let is_minimized_flag = Arc::new(AtomicBool::new(false));

            struct WindowRestoreGuard {
                target_title: String,
                is_minimized: Arc<AtomicBool>,
            }

            impl Drop for WindowRestoreGuard {
                fn drop(&mut self) {
                    if self.is_minimized.load(Ordering::SeqCst) && !self.target_title.is_empty() {
                        crate::core::activate_window(&self.target_title);
                    }
                }
            }

            let _restore_guard = if macro_data.target_window_enabled && !target_title.is_empty() {
                Some(WindowRestoreGuard {
                    target_title: target_title.clone(),
                    is_minimized: Arc::clone(&is_minimized_flag),
                })
            } else {
                None
            };

            if macro_data.target_window_enabled && !target_title.is_empty() {
                crate::core::activate_window(&target_title);
                thread::sleep(Duration::from_millis(200));
            }

            let mut stealth_timer = std::time::Instant::now();

            let check_stealth = |timer: &mut std::time::Instant, flag: &Arc<AtomicBool>, min_flag: &Arc<AtomicBool>| {
                if macro_data.target_window_enabled && macro_data.minimize_and_stay_active && !target_title.is_empty() {
                    let is_min = min_flag.load(Ordering::SeqCst);
                    let elapsed = timer.elapsed().as_secs_f32();

                    if !is_min && elapsed >= macro_data.active_seconds.max(0.5) {
                        crate::core::minimize_window(&target_title);
                        min_flag.store(true, Ordering::SeqCst);
                        *timer = std::time::Instant::now();
                    }

                    while min_flag.load(Ordering::SeqCst) && flag.load(Ordering::SeqCst) {
                        if timer.elapsed().as_secs_f32() >= macro_data.hide_seconds.max(0.5) {
                            crate::core::activate_window(&target_title);
                            min_flag.store(false, Ordering::SeqCst);
                            *timer = std::time::Instant::now();
                            thread::sleep(Duration::from_millis(150));
                            break;
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            };

            let has_image_recognition = macro_data.actions.iter().any(|a| a.enabled && a.use_image_recognition);
            if has_image_recognition {
                crate::core::start_screen_capture_worker_with_engine(macro_data.capture_engine);
            }

            let (screen_w, screen_h) = crate::core::get_screen_resolution();
            let mut last_known_mouse: Option<(i32, i32)> = None;
            let mut roi_cache: std::collections::HashMap<(usize, usize), Vec<(u32, u32, u32, u32)>> = std::collections::HashMap::new();
            let mut last_positions: std::collections::HashMap<(usize, usize), (i32, i32)> = std::collections::HashMap::new();
            let mut match_cycle_tracker: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

            let mut preloaded_templates: std::collections::HashMap<usize, Vec<crate::core::TemplatePattern>> = std::collections::HashMap::new();
            for (idx, action) in macro_data.actions.iter().enumerate() {
                if action.enabled && action.use_image_recognition && !action.image_paths.is_empty() {
                    let tmps: Vec<crate::core::TemplatePattern> = action
                        .image_paths
                        .iter()
                        .filter_map(|p| crate::core::TemplatePattern::from_path(p))
                        .collect();
                    if !tmps.is_empty() {
                        preloaded_templates.insert(idx, tmps);
                    }
                }
            }

            while flag.load(Ordering::SeqCst) && (infinite_loop || loop_iteration < target_loops) {
                check_stealth(&mut stealth_timer, &flag, &is_minimized_flag);

                for (idx, action) in macro_data.actions.iter().enumerate() {
                    if !flag.load(Ordering::SeqCst) {
                        break;
                    }

                    if !action.enabled {
                        continue;
                    }

                    action_idx_tracker.store(idx as i32, Ordering::SeqCst);
                    check_stealth(&mut stealth_timer, &flag, &is_minimized_flag);

                    let mut detected_pos: Option<(i32, i32)> = None;

                    if action.use_image_recognition && !action.image_paths.is_empty() {
                        let poll_duration = Duration::from_secs_f32(action.image_poll_rate.max(0.005));
                        let templates = preloaded_templates.get(&idx);

                        if let Some(templates) = templates {
                            while flag.load(Ordering::SeqCst) {
                                check_stealth(&mut stealth_timer, &flag, &is_minimized_flag);
                                if let Some(screen) = crate::core::capture_screen_with_engine(macro_data.capture_engine) {
                                    let mut found = false;
                                    for (t_idx, t) in templates.iter().enumerate() {
                                        let roi_key = (idx, t_idx);
                                        let cached_rois = roi_cache.get(&roi_key).cloned().unwrap_or_default();

                                        let mut matches: Vec<(i32, i32)> = Vec::new();

                                        if !cached_rois.is_empty() {
                                            for &roi_box in &cached_rois {
                                                let sub_matches = crate::core::find_all_template_matches_with_roi(
                                                    &screen,
                                                    t,
                                                    action.image_threshold,
                                                    Some(roi_box),
                                                    32,
                                                );
                                                for m in sub_matches {
                                                    if !matches.iter().any(|&(mx, my): &(i32, i32)| (mx - m.0).abs() < 10 && (my - m.1).abs() < 10) {
                                                        matches.push(m);
                                                    }
                                                }
                                            }
                                        }

                                        if matches.is_empty() {
                                            matches = crate::core::find_all_template_matches_with_roi(
                                                &screen,
                                                t,
                                                action.image_threshold,
                                                None,
                                                32,
                                            );
                                        }

                                        if !matches.is_empty() {
                                            let cycle_idx = match_cycle_tracker.entry(idx).or_insert(0);
                                            let chosen_coord = matches[*cycle_idx % matches.len()];
                                            *cycle_idx = (*cycle_idx + 1) % matches.len().max(1);

                                            detected_pos = Some(chosen_coord);

                                            let (tw, th) = t.dimensions();
                                            let margin_x = (tw * 3).max(180);
                                            let margin_y = (th * 3).max(180);

                                            let (vx, vy) = if let Some(prev) = last_positions.get(&roi_key) {
                                                (chosen_coord.0 - prev.0, chosen_coord.1 - prev.1)
                                            } else {
                                                (0, 0)
                                            };
                                            last_positions.insert(roi_key, chosen_coord);

                                            let pred_x = chosen_coord.0 + vx;
                                            let pred_y = chosen_coord.1 + vy;

                                            let rx1 = (pred_x as u32).saturating_sub(margin_x);
                                            let ry1 = (pred_y as u32).saturating_sub(margin_y);
                                            let rx2 = ((pred_x as u32) + margin_x).min(screen_w as u32);
                                            let ry2 = ((pred_y as u32) + margin_y).min(screen_h as u32);

                                            roi_cache.insert(roi_key, vec![(rx1, ry1, rx2, ry2)]);

                                            found = true;
                                            break;
                                        } else {
                                            last_positions.remove(&roi_key);
                                            roi_cache.remove(&roi_key);
                                        }
                                    }
                                    if found {
                                        break;
                                    }
                                }
                                if action.image_poll_rate > 0.001 {
                                    thread::sleep(poll_duration);
                                } else {
                                    thread::yield_now();
                                }
                            }
                        }
                    }

                    if !flag.load(Ordering::SeqCst) {
                        break;
                    }

                    if macro_data.target_window_enabled && !macro_data.minimize_and_stay_active && !target_title.is_empty() {
                        if !crate::core::is_window_focused(&target_title) {
                            crate::core::activate_window(&target_title);
                            thread::sleep(Duration::from_millis(100));
                        }
                    }

                    match action.action_type {
                        ActionType::Mouse => {
                            if action.delay_time > 0.0 {
                                let delay = if macro_data.human_like {
                                    (action.delay_time + 0.02).max(0.01)
                                } else {
                                    action.delay_time
                                };
                                let delay_ms = (delay * 1000.0) as u64;
                                let chunk = 50;
                                let mut spent = 0;
                                while spent < delay_ms && flag.load(Ordering::SeqCst) {
                                    let step = chunk.min(delay_ms - spent);
                                    thread::sleep(Duration::from_millis(step));
                                    spent += step;
                                    check_stealth(&mut stealth_timer, &flag, &is_minimized_flag);
                                }
                            }
                            if !flag.load(Ordering::SeqCst) {
                                break;
                            }

                            let (target_x, target_y) = if action.use_image_recognition && action.image_target_mouse {
                                detected_pos.unwrap_or((action.mouse.x, action.mouse.y))
                            } else {
                                (action.mouse.x, action.mouse.y)
                            };

                            if action.mouse.click_type == crate::core::ClickType::DragAndDrop {
                                let _ = virtual_input.drag_and_drop(
                                    target_x,
                                    target_y,
                                    action.mouse.end_x,
                                    action.mouse.end_y,
                                    screen_w,
                                    screen_h,
                                    action.mouse.speed,
                                );
                                last_known_mouse = Some((action.mouse.end_x, action.mouse.end_y));
                            } else {
                                if macro_data.human_like {
                                    let (start_x, start_y) = crate::core::get_mouse_location()
                                        .or(last_known_mouse)
                                        .unwrap_or((target_x, target_y));

                                    let _ = virtual_input.move_human(
                                        start_x,
                                        start_y,
                                        target_x,
                                        target_y,
                                        screen_w,
                                        screen_h,
                                        action.mouse.speed,
                                    );
                                } else {
                                    let _ = virtual_input.move_abs(
                                        target_x,
                                        target_y,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                                last_known_mouse = Some((target_x, target_y));

                                let click_type = if macro_data.left_handed_override {
                                    match action.mouse.click_type {
                                        crate::core::ClickType::Left => crate::core::ClickType::Right,
                                        crate::core::ClickType::Right => crate::core::ClickType::Left,
                                        other => other,
                                    }
                                } else {
                                    action.mouse.click_type
                                };

                                if click_type != crate::core::ClickType::NoClick {
                                    if action.delay_time > 0.0 {
                                        thread::sleep(Duration::from_millis(5));
                                    }
                                    let _ = virtual_input.click(click_type, action.mouse.hold_time);
                                }
                            }
                        }
                        ActionType::Keyboard => {
                            if action.delay_time > 0.0 {
                                let delay = if macro_data.human_like {
                                    (action.delay_time + 0.02).max(0.01)
                                } else {
                                    action.delay_time
                                };
                                let delay_ms = (delay * 1000.0) as u64;
                                let chunk = 50;
                                let mut spent = 0;
                                while spent < delay_ms && flag.load(Ordering::SeqCst) {
                                    let step = chunk.min(delay_ms - spent);
                                    thread::sleep(Duration::from_millis(step));
                                    spent += step;
                                    check_stealth(&mut stealth_timer, &flag, &is_minimized_flag);
                                }
                            }
                            if !flag.load(Ordering::SeqCst) {
                                break;
                            }

                            if macro_data.human_like {
                                let _ = virtual_input.type_text_human_with_layout(
                                    &action.keyboard.text,
                                    action.keyboard.speed,
                                    macro_data.keyboard_layout,
                                );
                            } else {
                                let _ = virtual_input.type_text_with_layout(
                                    &action.keyboard.text,
                                    action.keyboard.speed,
                                    macro_data.keyboard_layout,
                                );
                            }
                        }
                        ActionType::Delay => {
                            let delay = if macro_data.human_like {
                                (action.delay_time + 0.02).max(0.01)
                            } else {
                                action.delay_time
                            };
                            let delay_ms = (delay * 1000.0) as u64;
                            let chunk = 50;
                            let mut spent = 0;
                            while spent < delay_ms && flag.load(Ordering::SeqCst) {
                                let step = chunk.min(delay_ms - spent);
                                thread::sleep(Duration::from_millis(step));
                                spent += step;
                                check_stealth(&mut stealth_timer, &flag, &is_minimized_flag);
                            }
                        }
                        ActionType::Scroll => {
                            if action.delay_time > 0.0 {
                                let delay = if macro_data.human_like {
                                    (action.delay_time + 0.02).max(0.01)
                                } else {
                                    action.delay_time
                                };
                                let delay_ms = (delay * 1000.0) as u64;
                                let chunk = 50;
                                let mut spent = 0;
                                while spent < delay_ms && flag.load(Ordering::SeqCst) {
                                    let step = chunk.min(delay_ms - spent);
                                    thread::sleep(Duration::from_millis(step));
                                    spent += step;
                                    check_stealth(&mut stealth_timer, &flag, &is_minimized_flag);
                                }
                            }
                            if !flag.load(Ordering::SeqCst) {
                                break;
                            }

                            if action.use_image_recognition && action.image_target_mouse {
                                if let Some((target_x, target_y)) = detected_pos {
                                    let _ = virtual_input.move_abs(target_x, target_y, screen_w, screen_h);
                                    last_known_mouse = Some((target_x, target_y));
                                }
                            }

                            let _ = virtual_input.scroll(
                                action.scroll.direction,
                                action.scroll.amount,
                                action.scroll.smooth,
                            );
                        }
                    }
                }

                action_idx_tracker.store(-1, Ordering::SeqCst);
                loop_iteration += 1;
                
                thread::yield_now();
            }

            if is_minimized_flag.load(Ordering::SeqCst) && !target_title.is_empty() {
                crate::core::activate_window(&target_title);
                is_minimized_flag.store(false, Ordering::SeqCst);
            }

            action_idx_tracker.store(-1, Ordering::SeqCst);
            virtual_input.release_all_buttons();
            flag.store(false, Ordering::SeqCst);
        });
    }

    pub fn run_recorded(&self, rec_macro: RecordedMacro) {
        if self.is_active() {
            return;
        }

        self.is_running.store(true, Ordering::SeqCst);
        let flag = Arc::clone(&self.is_running);

        thread::spawn(move || {
            let mut virtual_input = match VirtualInputDevice::new_replay("oGiuMacro Rec Replay") {
                Ok(dev) => dev,
                Err(_) => {
                    flag.store(false, Ordering::SeqCst);
                    return;
                }
            };

            thread::sleep(Duration::from_millis(200));

            let (screen_w, screen_h) = crate::core::get_screen_resolution();

            let mut loop_iteration = 0;
            let infinite_loop = rec_macro.is_loop && rec_macro.loop_count == 0;
            let target_loops = if rec_macro.is_loop {
                rec_macro.loop_count.max(1)
            } else {
                1
            };

            while flag.load(Ordering::SeqCst) && (infinite_loop || loop_iteration < target_loops) {
                if rec_macro.start_x >= 0 && rec_macro.start_y >= 0 {
                    let _ = virtual_input.move_abs(rec_macro.start_x, rec_macro.start_y, screen_w, screen_h);
                    thread::sleep(Duration::from_millis(30));
                }

                let playback_start = Instant::now();

                for event in &rec_macro.events {
                    if !flag.load(Ordering::SeqCst) {
                        break;
                    }

                    let target_ms = (event.timestamp_ms as f64 / rec_macro.playback_speed.max(0.1) as f64) as u64;
                    while flag.load(Ordering::SeqCst) {
                        let elapsed_ms = playback_start.elapsed().as_millis() as u64;
                        if elapsed_ms >= target_ms {
                            break;
                        }
                        let diff = target_ms - elapsed_ms;
                        if diff > 2 {
                            thread::sleep(Duration::from_millis(diff - 1));
                        } else {
                            thread::yield_now();
                        }
                    }

                    if !flag.load(Ordering::SeqCst) {
                        break;
                    }

                    let code = if rec_macro.left_handed_override && event.event_type == evdev::EventType::KEY.0 {
                        if event.code == KeyCode::BTN_LEFT.0 {
                            KeyCode::BTN_RIGHT.0
                        } else if event.code == KeyCode::BTN_RIGHT.0 {
                            KeyCode::BTN_LEFT.0
                        } else {
                            event.code
                        }
                    } else {
                        event.code
                    };

                    let _ = virtual_input.emit_raw_input_event(
                        event.event_type,
                        code,
                        event.value,
                    );
                }

                loop_iteration += 1;
            }

            virtual_input.release_all_buttons();
            flag.store(false, Ordering::SeqCst);
        });
    }
}
