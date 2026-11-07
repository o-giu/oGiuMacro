use std::thread;
use std::time::Duration;
use evdev::uinput::VirtualDevice;
use evdev::{
    AbsInfo, AbsoluteAxisCode, AttributeSet, BusType, EventType, InputEvent, InputId, KeyCode,
    PropType, RelativeAxisCode, UinputAbsSetup,
};

use crate::core::ClickType;

#[allow(dead_code)]
pub struct VirtualInputDevice {
    device: VirtualDevice,
    last_abs_x: Option<i32>,
    last_abs_y: Option<i32>,
}

#[allow(dead_code)]
impl VirtualInputDevice {
    pub fn new(name: &str) -> std::io::Result<Self> {
        Self::new_replay(name)
    }

    pub fn new_replay(name: &str) -> std::io::Result<Self> {
        let (dev_name, vendor_id, product_id) = if let Some((m_name, v, p)) = crate::core::detect_primary_mouse() {
            (m_name, v, p)
        } else {
            (name.to_string(), 0x046d, 0xc703)
        };

        let mut keys = AttributeSet::<KeyCode>::new();
        for k in 1..256 {
            keys.insert(KeyCode::new(k));
        }
        for b in 272..=279 {
            keys.insert(KeyCode::new(b));
        }
        for k in 352..512 {
            keys.insert(KeyCode::new(k));
        }

        let mut rel_axes = AttributeSet::<RelativeAxisCode>::new();
        rel_axes.insert(RelativeAxisCode::REL_X);
        rel_axes.insert(RelativeAxisCode::REL_Y);
        rel_axes.insert(RelativeAxisCode::REL_WHEEL);
        rel_axes.insert(RelativeAxisCode::REL_HWHEEL);
        rel_axes.insert(RelativeAxisCode::REL_WHEEL_HI_RES);
        rel_axes.insert(RelativeAxisCode::REL_HWHEEL_HI_RES);

        let abs_x_setup = UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_X,
            AbsInfo::new(0, 0, 65535, 0, 0, 10),
        );
        let abs_y_setup = UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_Y,
            AbsInfo::new(0, 0, 65535, 0, 0, 10),
        );

        let mut props = AttributeSet::<PropType>::new();
        props.insert(PropType::POINTER);

        let input_id = InputId::new(BusType::BUS_USB, vendor_id, product_id, 1);

        let device = VirtualDevice::builder()?
            .name(&dev_name)
            .input_id(input_id)
            .with_properties(&props)?
            .with_keys(&keys)?
            .with_relative_axes(&rel_axes)?
            .with_absolute_axis(&abs_x_setup)?
            .with_absolute_axis(&abs_y_setup)?
            .build()?;

        Ok(Self {
            device,
            last_abs_x: None,
            last_abs_y: None,
        })
    }

    pub fn new_with_hardware(name: &str, vendor_id: u16, product_id: u16) -> std::io::Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for k in 1..256 {
            keys.insert(KeyCode::new(k));
        }
        for b in 272..=279 {
            keys.insert(KeyCode::new(b));
        }
        for k in 352..512 {
            keys.insert(KeyCode::new(k));
        }

        let mut rel_axes = AttributeSet::<RelativeAxisCode>::new();
        rel_axes.insert(RelativeAxisCode::REL_X);
        rel_axes.insert(RelativeAxisCode::REL_Y);
        rel_axes.insert(RelativeAxisCode::REL_WHEEL);
        rel_axes.insert(RelativeAxisCode::REL_HWHEEL);
        rel_axes.insert(RelativeAxisCode::REL_WHEEL_HI_RES);
        rel_axes.insert(RelativeAxisCode::REL_HWHEEL_HI_RES);

        let abs_x_setup = UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_X,
            AbsInfo::new(0, 0, 65535, 0, 0, 10),
        );
        let abs_y_setup = UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_Y,
            AbsInfo::new(0, 0, 65535, 0, 0, 10),
        );

        let mut props = AttributeSet::<PropType>::new();
        props.insert(PropType::POINTER);

        let input_id = InputId::new(BusType::BUS_USB, vendor_id, product_id, 1);

        let device = VirtualDevice::builder()?
            .name(name)
            .input_id(input_id)
            .with_properties(&props)?
            .with_keys(&keys)?
            .with_relative_axes(&rel_axes)?
            .with_absolute_axis(&abs_x_setup)?
            .with_absolute_axis(&abs_y_setup)?
            .build()?;

        Ok(Self {
            device,
            last_abs_x: None,
            last_abs_y: None,
        })
    }

    pub fn emit_event(&mut self, event_type: EventType, code: u16, value: i32) -> std::io::Result<()> {
        let ev = InputEvent::new(event_type.0, code, value);
        self.device.emit(&[ev])
    }

    pub fn emit_raw_input_event(&mut self, event_type: u16, code: u16, value: i32) -> std::io::Result<()> {
        use std::os::fd::AsRawFd;
        let raw_ev = libc::input_event {
            time: libc::timeval { tv_sec: 0, tv_usec: 0 },
            type_: event_type,
            code,
            value,
        };
        let ret = unsafe {
            libc::write(
                self.device.as_raw_fd(),
                &raw_ev as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::input_event>(),
            )
        };
        if ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn click(&mut self, click_type: ClickType, hold_time_secs: f32) -> std::io::Result<()> {
        let key = match click_type {
            ClickType::Left | ClickType::DoubleLeft => KeyCode::BTN_LEFT,
            ClickType::Right => KeyCode::BTN_RIGHT,
            ClickType::Middle => KeyCode::BTN_MIDDLE,
            _ => return Ok(()),
        };

        let hold_duration = Duration::from_secs_f32(hold_time_secs.max(0.01));

        match click_type {
            ClickType::DoubleLeft => {
                self.press_key(key.0)?;
                thread::sleep(hold_duration);
                self.release_key(key.0)?;
                thread::sleep(Duration::from_millis(60));
                self.press_key(key.0)?;
                thread::sleep(hold_duration);
                self.release_key(key.0)?;
            }
            _ => {
                self.press_key(key.0)?;
                thread::sleep(hold_duration);
                self.release_key(key.0)?;
            }
        }

        Ok(())
    }

    pub fn drag_and_drop(
        &mut self,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        width: i32,
        height: i32,
        speed: f32,
    ) -> std::io::Result<()> {
        self.move_abs(start_x, start_y, width, height)?;
        thread::sleep(Duration::from_millis(50));
        self.press_key(KeyCode::BTN_LEFT.0)?;
        thread::sleep(Duration::from_millis(50));

        let steps = ((20.0 * speed.max(0.5)) as i32).max(10);
        for i in 1..=steps {
            let cur_x = start_x + (end_x - start_x) * i / steps;
            let cur_y = start_y + (end_y - start_y) * i / steps;
            self.move_abs(cur_x, cur_y, width, height)?;
            thread::sleep(Duration::from_millis(15));
        }

        self.release_key(KeyCode::BTN_LEFT.0)?;
        thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    pub fn move_rel(&mut self, dx: i32, dy: i32) -> std::io::Result<()> {
        let ev_x = InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, dx);
        let ev_y = InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, dy);
        self.device.emit(&[ev_x, ev_y])
    }

    pub fn move_abs(&mut self, x: i32, y: i32, width: i32, height: i32) -> std::io::Result<()> {
        let max_coord = 65535;
        let abs_x = (x * max_coord) / width.max(1);
        let abs_y = (y * max_coord) / height.max(1);

        if self.last_abs_x == Some(abs_x) && self.last_abs_y == Some(abs_y) {
            let offset_x = if abs_x > 0 { abs_x - 1 } else { abs_x + 1 };
            let pre_x = InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, offset_x);
            let _ = self.device.emit(&[pre_x]);
            thread::sleep(Duration::from_millis(2));
        }

        let ev_x = InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, abs_x);
        let ev_y = InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_Y.0, abs_y);
        self.device.emit(&[ev_x, ev_y])?;

        self.last_abs_x = Some(abs_x);
        self.last_abs_y = Some(abs_y);
        Ok(())
    }

    pub fn scroll(&mut self, direction: crate::core::ScrollDirection, amount: i32, smooth: bool) -> std::io::Result<()> {
        let val = match direction {
            crate::core::ScrollDirection::Up => 1,
            crate::core::ScrollDirection::Down => -1,
        };
        let hi_res_val = val * 120;
        let count = amount.max(1);

        for _ in 0..count {
            let ev_wheel = InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_WHEEL.0, val);
            let ev_hires = InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_WHEEL_HI_RES.0, hi_res_val);
            let _ = self.device.emit(&[ev_hires, ev_wheel]);

            if smooth {
                thread::sleep(Duration::from_millis(25));
            } else {
                thread::sleep(Duration::from_millis(8));
            }
        }
        Ok(())
    }

    pub fn press_key(&mut self, key_code: u16) -> std::io::Result<()> {
        let ev = InputEvent::new(EventType::KEY.0, key_code, 1);
        self.device.emit(&[ev])
    }

    pub fn release_key(&mut self, key_code: u16) -> std::io::Result<()> {
        let ev = InputEvent::new(EventType::KEY.0, key_code, 0);
        self.device.emit(&[ev])
    }

    pub fn release_all_buttons(&mut self) {
        for b in 272..=279 {
            let _ = self.release_key(b);
        }
    }

    pub fn move_human(
        &mut self,
        start_x: i32,
        start_y: i32,
        target_x: i32,
        target_y: i32,
        screen_w: i32,
        screen_h: i32,
        speed_factor: f32,
    ) -> std::io::Result<()> {
        let dx = (target_x - start_x) as f32;
        let dy = (target_y - start_y) as f32;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 4.0 {
            return self.move_abs(target_x, target_y, screen_w, screen_h);
        }

        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(42);
        let mut rng_seed = (now_nanos ^ (start_x as u128) ^ ((target_y as u128) << 16)) as u64;
        let mut next_rng = move || {
            rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_seed >> 32) as f32) / (u32::MAX as f32)
        };

        let base_steps = (dist / 14.0).clamp(20.0, 75.0) / speed_factor.clamp(0.2, 4.0);
        let steps = (base_steps as i32).max(18);

        let perp_x = -dy / dist;
        let perp_y = dx / dist;

        let dev_mag1 = (dist * 0.18).min(80.0) * (next_rng() * 2.0 - 1.0);
        let dev_mag2 = (dist * 0.14).min(60.0) * (next_rng() * 2.0 - 1.0);

        let p0 = (start_x as f32, start_y as f32);
        let p3 = (target_x as f32, target_y as f32);

        let p1 = (
            p0.0 + dx * (0.25 + next_rng() * 0.15) + perp_x * dev_mag1,
            p0.1 + dy * (0.25 + next_rng() * 0.15) + perp_y * dev_mag1,
        );
        let p2 = (
            p0.0 + dx * (0.65 + next_rng() * 0.15) + perp_x * dev_mag2,
            p0.1 + dy * (0.65 + next_rng() * 0.15) + perp_y * dev_mag2,
        );

        let base_delay_ms = 7.0 / speed_factor.clamp(0.2, 3.0);

        for step in 1..=steps {
            let u = step as f32 / steps as f32;
            
            let t = if u < 0.5 {
                2.0 * u * u
            } else {
                -1.0 + (4.0 - 2.0 * u) * u
            };

            let omt = 1.0 - t;
            let omt2 = omt * omt;
            let omt3 = omt2 * omt;
            let t2 = t * t;
            let t3 = t2 * t;

            let bx = omt3 * p0.0 + 3.0 * omt2 * t * p1.0 + 3.0 * omt * t2 * p2.0 + t3 * p3.0;
            let by = omt3 * p0.1 + 3.0 * omt2 * t * p1.1 + 3.0 * omt * t2 * p2.1 + t3 * p3.1;

            let taper = (1.0 - u).max(0.05);
            let jitter_x = (next_rng() * 2.0 - 1.0) * 1.8 * taper;
            let jitter_y = (next_rng() * 2.0 - 1.0) * 1.8 * taper;

            let cur_x = (bx + jitter_x).round() as i32;
            let cur_y = (by + jitter_y).round() as i32;

            self.move_abs(cur_x, cur_y, screen_w, screen_h)?;

            let step_delay = (base_delay_ms + (next_rng() * 4.0 - 2.0)).max(2.0);
            thread::sleep(Duration::from_millis(step_delay as u64));
        }

        self.move_abs(target_x, target_y, screen_w, screen_h)?;
        thread::sleep(Duration::from_millis(15));
        Ok(())
    }

    pub fn type_text(&mut self, text: &str, speed_multiplier: f32) -> std::io::Result<()> {
        self.type_text_internal(text, speed_multiplier, false, crate::core::KeyboardLayout::Abnt2)
    }

    pub fn type_text_with_layout(
        &mut self,
        text: &str,
        speed_multiplier: f32,
        layout: crate::core::KeyboardLayout,
    ) -> std::io::Result<()> {
        self.type_text_internal(text, speed_multiplier, false, layout)
    }

    pub fn type_text_human(&mut self, text: &str, speed_multiplier: f32) -> std::io::Result<()> {
        self.type_text_internal(text, speed_multiplier, true, crate::core::KeyboardLayout::Abnt2)
    }

    pub fn type_text_human_with_layout(
        &mut self,
        text: &str,
        speed_multiplier: f32,
        layout: crate::core::KeyboardLayout,
    ) -> std::io::Result<()> {
        self.type_text_internal(text, speed_multiplier, true, layout)
    }

    fn type_text_internal(
        &mut self,
        text: &str,
        speed_multiplier: f32,
        human_like: bool,
        layout: crate::core::KeyboardLayout,
    ) -> std::io::Result<()> {
        let base_delay_ms = (30.0 / speed_multiplier.max(0.1)) as u64;
        let char_delay = Duration::from_millis(base_delay_ms.max(5));
        let chars: Vec<char> = text.chars().collect();
        let mut idx = 0;

        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(12345);
        let mut rng_seed = (now_nanos ^ ((chars.len() as u128) << 24)) as u64;
        let mut next_rng = move || {
            rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_seed >> 32) as f32) / (u32::MAX as f32)
        };

        while idx < chars.len() {
            if chars[idx] == '[' {
                if let Some(close_offset) = chars[idx..].iter().position(|&c| c == ']') {
                    let end_pos = idx + close_offset;
                    let tag_content: String = chars[idx + 1..end_pos].iter().collect();
                    let key_codes = parse_tag_to_keys(&tag_content, layout);

                    if !key_codes.is_empty() {
                        
                        if human_like {
                            let tag_hesitation = match tag_content.to_ascii_uppercase().as_str() {
                                "ENTER" | "RETURN" => 40 + (next_rng() * 80.0) as u64,
                                "ESC" | "ESCAPE" => 30 + (next_rng() * 60.0) as u64,
                                s if s.contains('+') => 40 + (next_rng() * 70.0) as u64,
                                _ => 15 + (next_rng() * 30.0) as u64,
                            };
                            thread::sleep(Duration::from_millis(tag_hesitation));
                        }

                        for &k in &key_codes {
                            self.press_key(k)?;
                            if human_like && key_codes.len() > 1 {
                                thread::sleep(Duration::from_millis(8 + (next_rng() * 12.0) as u64));
                            }
                        }

                        let hold_ms = if human_like {
                            35 + (next_rng() * 45.0) as u64
                        } else {
                            char_delay.as_millis() as u64
                        };
                        thread::sleep(Duration::from_millis(hold_ms));

                        for &k in key_codes.iter().rev() {
                            self.release_key(k)?;
                            if human_like && key_codes.len() > 1 {
                                thread::sleep(Duration::from_millis(5 + (next_rng() * 10.0) as u64));
                            }
                        }

                        let post_tag_delay = if human_like {
                            40 + (next_rng() * 70.0) as u64
                        } else {
                            char_delay.as_millis() as u64
                        };
                        thread::sleep(Duration::from_millis(post_tag_delay));

                        idx = end_pos + 1;
                        continue;
                    }
                }
            }

            let ch = chars[idx];
            if let Some(steps) = char_to_input_steps(ch, layout) {
                if human_like {
                    let pause = match ch {
                        ' ' => 25 + (next_rng() * 50.0) as u64,
                        '.' | '!' | '?' => 90 + (next_rng() * 120.0) as u64,
                        ',' | ';' | ':' => 50 + (next_rng() * 80.0) as u64,
                        _ => (next_rng() * 25.0) as u64,
                    };
                    if pause > 0 {
                        thread::sleep(Duration::from_millis(pause));
                    }
                }

                for step in steps {
                    if step.altgr {
                        self.press_key(KeyCode::KEY_RIGHTALT.0)?;
                        if human_like {
                            thread::sleep(Duration::from_millis(10 + (next_rng() * 15.0) as u64));
                        }
                    } else if step.shift {
                        self.press_key(KeyCode::KEY_LEFTSHIFT.0)?;
                        if human_like {
                            thread::sleep(Duration::from_millis(10 + (next_rng() * 15.0) as u64));
                        }
                    }

                    self.press_key(step.key)?;

                    let char_hold_ms = if human_like {
                        20 + (next_rng() * 30.0) as u64
                    } else {
                        char_delay.as_millis() as u64
                    };
                    thread::sleep(Duration::from_millis(char_hold_ms));

                    self.release_key(step.key)?;

                    if step.altgr {
                        if human_like {
                            thread::sleep(Duration::from_millis(8 + (next_rng() * 12.0) as u64));
                        }
                        self.release_key(KeyCode::KEY_RIGHTALT.0)?;
                    } else if step.shift {
                        if human_like {
                            thread::sleep(Duration::from_millis(8 + (next_rng() * 12.0) as u64));
                        }
                        self.release_key(KeyCode::KEY_LEFTSHIFT.0)?;
                    }

                    let step_gap = if human_like {
                        10 + (next_rng() * 15.0) as u64
                    } else {
                        char_delay.as_millis() as u64
                    };
                    thread::sleep(Duration::from_millis(step_gap));
                }

                let inter_char_delay = if human_like {
                    let speed_adj = (30.0 / speed_multiplier.max(0.1)) as u64;
                    speed_adj.max(10) + (next_rng() * 28.0) as u64
                } else {
                    char_delay.as_millis() as u64
                };
                thread::sleep(Duration::from_millis(inter_char_delay));
            }
            idx += 1;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyStep {
    pub key: u16,
    pub shift: bool,
    pub altgr: bool,
}

impl KeyStep {
    pub const fn new(key: u16) -> Self {
        Self { key, shift: false, altgr: false }
    }
    pub const fn shift(key: u16) -> Self {
        Self { key, shift: true, altgr: false }
    }
    pub const fn altgr(key: u16) -> Self {
        Self { key, shift: false, altgr: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeadAccent {
    Acute,
    Grave,
    Tilde,
    Circumflex,
    Diaeresis,
}

fn decompose_accent(ch: char) -> Option<(DeadAccent, char, bool)> {
    match ch {
        'á' => Some((DeadAccent::Acute, 'a', false)),
        'é' => Some((DeadAccent::Acute, 'e', false)),
        'í' => Some((DeadAccent::Acute, 'i', false)),
        'ó' => Some((DeadAccent::Acute, 'o', false)),
        'ú' => Some((DeadAccent::Acute, 'u', false)),
        'Á' => Some((DeadAccent::Acute, 'a', true)),
        'É' => Some((DeadAccent::Acute, 'e', true)),
        'Í' => Some((DeadAccent::Acute, 'i', true)),
        'Ó' => Some((DeadAccent::Acute, 'o', true)),
        'Ú' => Some((DeadAccent::Acute, 'u', true)),

        'à' => Some((DeadAccent::Grave, 'a', false)),
        'è' => Some((DeadAccent::Grave, 'e', false)),
        'ì' => Some((DeadAccent::Grave, 'i', false)),
        'ò' => Some((DeadAccent::Grave, 'o', false)),
        'ù' => Some((DeadAccent::Grave, 'u', false)),
        'À' => Some((DeadAccent::Grave, 'a', true)),
        'È' => Some((DeadAccent::Grave, 'e', true)),
        'Ì' => Some((DeadAccent::Grave, 'i', true)),
        'Ò' => Some((DeadAccent::Grave, 'o', true)),
        'Ù' => Some((DeadAccent::Grave, 'u', true)),

        'ã' => Some((DeadAccent::Tilde, 'a', false)),
        'õ' => Some((DeadAccent::Tilde, 'o', false)),
        'ñ' => Some((DeadAccent::Tilde, 'n', false)),
        'Ã' => Some((DeadAccent::Tilde, 'a', true)),
        'Õ' => Some((DeadAccent::Tilde, 'o', true)),
        'Ñ' => Some((DeadAccent::Tilde, 'n', true)),

        'â' => Some((DeadAccent::Circumflex, 'a', false)),
        'ê' => Some((DeadAccent::Circumflex, 'e', false)),
        'î' => Some((DeadAccent::Circumflex, 'i', false)),
        'ô' => Some((DeadAccent::Circumflex, 'o', false)),
        'û' => Some((DeadAccent::Circumflex, 'u', false)),
        'Â' => Some((DeadAccent::Circumflex, 'a', true)),
        'Ê' => Some((DeadAccent::Circumflex, 'e', true)),
        'Î' => Some((DeadAccent::Circumflex, 'i', true)),
        'Ô' => Some((DeadAccent::Circumflex, 'o', true)),
        'Û' => Some((DeadAccent::Circumflex, 'u', true)),

        'ä' => Some((DeadAccent::Diaeresis, 'a', false)),
        'ë' => Some((DeadAccent::Diaeresis, 'e', false)),
        'ï' => Some((DeadAccent::Diaeresis, 'i', false)),
        'ö' => Some((DeadAccent::Diaeresis, 'o', false)),
        'ü' => Some((DeadAccent::Diaeresis, 'u', false)),
        'Ä' => Some((DeadAccent::Diaeresis, 'a', true)),
        'Ë' => Some((DeadAccent::Diaeresis, 'e', true)),
        'Ï' => Some((DeadAccent::Diaeresis, 'i', true)),
        'Ö' => Some((DeadAccent::Diaeresis, 'o', true)),
        'Ü' => Some((DeadAccent::Diaeresis, 'u', true)),

        _ => None,
    }
}

fn dead_accent_step(accent: DeadAccent, layout: crate::core::KeyboardLayout) -> KeyStep {
    match layout {
        crate::core::KeyboardLayout::Abnt2 => match accent {
            DeadAccent::Acute => KeyStep::new(KeyCode::KEY_LEFTBRACE.0),
            DeadAccent::Grave => KeyStep::shift(KeyCode::KEY_LEFTBRACE.0),
            DeadAccent::Tilde => KeyStep::new(KeyCode::KEY_APOSTROPHE.0),
            DeadAccent::Circumflex => KeyStep::shift(KeyCode::KEY_APOSTROPHE.0),
            DeadAccent::Diaeresis => KeyStep::shift(KeyCode::KEY_6.0),
        },
        crate::core::KeyboardLayout::US => match accent {
            DeadAccent::Acute => KeyStep::new(KeyCode::KEY_APOSTROPHE.0),
            DeadAccent::Grave => KeyStep::new(KeyCode::KEY_GRAVE.0),
            DeadAccent::Tilde => KeyStep::shift(KeyCode::KEY_GRAVE.0),
            DeadAccent::Circumflex => KeyStep::shift(KeyCode::KEY_6.0),
            DeadAccent::Diaeresis => KeyStep::shift(KeyCode::KEY_APOSTROPHE.0),
        },
    }
}

fn ascii_letter_to_code(c: char) -> Option<u16> {
    match c.to_ascii_lowercase() {
        'a' => Some(KeyCode::KEY_A.0),
        'b' => Some(KeyCode::KEY_B.0),
        'c' => Some(KeyCode::KEY_C.0),
        'd' => Some(KeyCode::KEY_D.0),
        'e' => Some(KeyCode::KEY_E.0),
        'f' => Some(KeyCode::KEY_F.0),
        'g' => Some(KeyCode::KEY_G.0),
        'h' => Some(KeyCode::KEY_H.0),
        'i' => Some(KeyCode::KEY_I.0),
        'j' => Some(KeyCode::KEY_J.0),
        'k' => Some(KeyCode::KEY_K.0),
        'l' => Some(KeyCode::KEY_L.0),
        'm' => Some(KeyCode::KEY_M.0),
        'n' => Some(KeyCode::KEY_N.0),
        'o' => Some(KeyCode::KEY_O.0),
        'p' => Some(KeyCode::KEY_P.0),
        'q' => Some(KeyCode::KEY_Q.0),
        'r' => Some(KeyCode::KEY_R.0),
        's' => Some(KeyCode::KEY_S.0),
        't' => Some(KeyCode::KEY_T.0),
        'u' => Some(KeyCode::KEY_U.0),
        'v' => Some(KeyCode::KEY_V.0),
        'w' => Some(KeyCode::KEY_W.0),
        'x' => Some(KeyCode::KEY_X.0),
        'y' => Some(KeyCode::KEY_Y.0),
        'z' => Some(KeyCode::KEY_Z.0),
        _ => None,
    }
}

fn char_to_input_steps(ch: char, layout: crate::core::KeyboardLayout) -> Option<Vec<KeyStep>> {
    if let Some((accent, base, is_upper)) = decompose_accent(ch) {
        let accent_step = dead_accent_step(accent, layout);
        let base_code = ascii_letter_to_code(base)?;
        let letter_step = if is_upper {
            KeyStep::shift(base_code)
        } else {
            KeyStep::new(base_code)
        };
        return Some(vec![accent_step, letter_step]);
    }

    if layout == crate::core::KeyboardLayout::Abnt2 {
        char_to_input_steps_abnt2(ch)
    } else {
        char_to_input_steps_us(ch)
    }
}

fn char_to_input_steps_abnt2(ch: char) -> Option<Vec<KeyStep>> {
    if ch == '§' {
        return Some(vec![KeyStep::altgr(KeyCode::KEY_EQUAL.0)]);
    }

    if ch.is_ascii_alphabetic() {
        let code = ascii_letter_to_code(ch)?;
        return Some(vec![if ch.is_ascii_uppercase() {
            KeyStep::shift(code)
        } else {
            KeyStep::new(code)
        }]);
    }

    if ch == 'ç' {
        return Some(vec![KeyStep::new(KeyCode::KEY_SEMICOLON.0)]);
    }
    if ch == 'Ç' {
        return Some(vec![KeyStep::shift(KeyCode::KEY_SEMICOLON.0)]);
    }

    let digit_code = match ch {
        '1' => Some(KeyCode::KEY_1.0),
        '2' => Some(KeyCode::KEY_2.0),
        '3' => Some(KeyCode::KEY_3.0),
        '4' => Some(KeyCode::KEY_4.0),
        '5' => Some(KeyCode::KEY_5.0),
        '6' => Some(KeyCode::KEY_6.0),
        '7' => Some(KeyCode::KEY_7.0),
        '8' => Some(KeyCode::KEY_8.0),
        '9' => Some(KeyCode::KEY_9.0),
        '0' => Some(KeyCode::KEY_0.0),
        _ => None,
    };
    if let Some(code) = digit_code {
        return Some(vec![KeyStep::new(code)]);
    }

    let simple = match ch {
        '!' => KeyStep::shift(KeyCode::KEY_1.0),
        '@' => KeyStep::shift(KeyCode::KEY_2.0),
        '#' => KeyStep::shift(KeyCode::KEY_3.0),
        '$' => KeyStep::shift(KeyCode::KEY_4.0),
        '%' => KeyStep::shift(KeyCode::KEY_5.0),
        '&' => KeyStep::shift(KeyCode::KEY_7.0),
        '*' => KeyStep::shift(KeyCode::KEY_8.0),
        '(' => KeyStep::shift(KeyCode::KEY_9.0),
        ')' => KeyStep::shift(KeyCode::KEY_0.0),
        '-' => KeyStep::new(KeyCode::KEY_MINUS.0),
        '_' => KeyStep::shift(KeyCode::KEY_MINUS.0),
        '=' => KeyStep::new(KeyCode::KEY_EQUAL.0),
        '+' => KeyStep::shift(KeyCode::KEY_EQUAL.0),
        ' ' => KeyStep::new(KeyCode::KEY_SPACE.0),
        '\n' => KeyStep::new(KeyCode::KEY_ENTER.0),
        '\t' => KeyStep::new(KeyCode::KEY_TAB.0),
        ';' => KeyStep::new(KeyCode::KEY_SLASH.0),
        ':' => KeyStep::shift(KeyCode::KEY_SLASH.0),
        '/' => KeyStep::new(KeyCode::KEY_RO.0),
        '?' => KeyStep::shift(KeyCode::KEY_RO.0),
        ',' => KeyStep::new(KeyCode::KEY_COMMA.0),
        '<' => KeyStep::shift(KeyCode::KEY_COMMA.0),
        '.' => KeyStep::new(KeyCode::KEY_DOT.0),
        '>' => KeyStep::shift(KeyCode::KEY_DOT.0),
        '\\' => KeyStep::new(KeyCode::KEY_102ND.0),
        '|' => KeyStep::shift(KeyCode::KEY_102ND.0),
        '\'' => KeyStep::new(KeyCode::KEY_GRAVE.0),
        '"' => KeyStep::shift(KeyCode::KEY_GRAVE.0),
        '[' => KeyStep::new(KeyCode::KEY_RIGHTBRACE.0),
        '{' => KeyStep::shift(KeyCode::KEY_RIGHTBRACE.0),
        ']' => KeyStep::new(KeyCode::KEY_BACKSLASH.0),
        '}' => KeyStep::shift(KeyCode::KEY_BACKSLASH.0),
        '°' => KeyStep::altgr(KeyCode::KEY_E.0),
        'ª' => KeyStep::altgr(KeyCode::KEY_RIGHTBRACE.0),
        'º' => KeyStep::altgr(KeyCode::KEY_BACKSLASH.0),
        _ => return match ch {
            '~' => Some(vec![KeyStep::new(KeyCode::KEY_APOSTROPHE.0), KeyStep::new(KeyCode::KEY_SPACE.0)]),
            '^' => Some(vec![KeyStep::shift(KeyCode::KEY_APOSTROPHE.0), KeyStep::new(KeyCode::KEY_SPACE.0)]),
            '`' => Some(vec![KeyStep::shift(KeyCode::KEY_LEFTBRACE.0), KeyStep::new(KeyCode::KEY_SPACE.0)]),
            '´' => Some(vec![KeyStep::new(KeyCode::KEY_LEFTBRACE.0), KeyStep::new(KeyCode::KEY_SPACE.0)]),
            '¨' => Some(vec![KeyStep::shift(KeyCode::KEY_6.0), KeyStep::new(KeyCode::KEY_SPACE.0)]),
            _ => None,
        },
    };

    Some(vec![simple])
}

fn char_to_input_steps_us(ch: char) -> Option<Vec<KeyStep>> {
    if ch.is_ascii_alphabetic() {
        let code = ascii_letter_to_code(ch)?;
        return Some(vec![if ch.is_ascii_uppercase() {
            KeyStep::shift(code)
        } else {
            KeyStep::new(code)
        }]);
    }

    let digit_code = match ch {
        '1' => Some(KeyCode::KEY_1.0),
        '2' => Some(KeyCode::KEY_2.0),
        '3' => Some(KeyCode::KEY_3.0),
        '4' => Some(KeyCode::KEY_4.0),
        '5' => Some(KeyCode::KEY_5.0),
        '6' => Some(KeyCode::KEY_6.0),
        '7' => Some(KeyCode::KEY_7.0),
        '8' => Some(KeyCode::KEY_8.0),
        '9' => Some(KeyCode::KEY_9.0),
        '0' => Some(KeyCode::KEY_0.0),
        _ => None,
    };
    if let Some(code) = digit_code {
        return Some(vec![KeyStep::new(code)]);
    }

    let simple = match ch {
        '!' => KeyStep::shift(KeyCode::KEY_1.0),
        '@' => KeyStep::shift(KeyCode::KEY_2.0),
        '#' => KeyStep::shift(KeyCode::KEY_3.0),
        '$' => KeyStep::shift(KeyCode::KEY_4.0),
        '%' => KeyStep::shift(KeyCode::KEY_5.0),
        '^' => KeyStep::shift(KeyCode::KEY_6.0),
        '&' => KeyStep::shift(KeyCode::KEY_7.0),
        '*' => KeyStep::shift(KeyCode::KEY_8.0),
        '(' => KeyStep::shift(KeyCode::KEY_9.0),
        ')' => KeyStep::shift(KeyCode::KEY_0.0),
        '-' => KeyStep::new(KeyCode::KEY_MINUS.0),
        '_' => KeyStep::shift(KeyCode::KEY_MINUS.0),
        '=' => KeyStep::new(KeyCode::KEY_EQUAL.0),
        '+' => KeyStep::shift(KeyCode::KEY_EQUAL.0),
        ' ' => KeyStep::new(KeyCode::KEY_SPACE.0),
        '\n' => KeyStep::new(KeyCode::KEY_ENTER.0),
        '\t' => KeyStep::new(KeyCode::KEY_TAB.0),
        '[' => KeyStep::new(KeyCode::KEY_LEFTBRACE.0),
        '{' => KeyStep::shift(KeyCode::KEY_LEFTBRACE.0),
        ']' => KeyStep::new(KeyCode::KEY_RIGHTBRACE.0),
        '}' => KeyStep::shift(KeyCode::KEY_RIGHTBRACE.0),
        ';' => KeyStep::new(KeyCode::KEY_SEMICOLON.0),
        ':' => KeyStep::shift(KeyCode::KEY_SEMICOLON.0),
        '\'' => KeyStep::new(KeyCode::KEY_APOSTROPHE.0),
        '"' => KeyStep::shift(KeyCode::KEY_APOSTROPHE.0),
        ',' => KeyStep::new(KeyCode::KEY_COMMA.0),
        '<' => KeyStep::shift(KeyCode::KEY_COMMA.0),
        '.' => KeyStep::new(KeyCode::KEY_DOT.0),
        '>' => KeyStep::shift(KeyCode::KEY_DOT.0),
        '/' => KeyStep::new(KeyCode::KEY_SLASH.0),
        '?' => KeyStep::shift(KeyCode::KEY_SLASH.0),
        '\\' => KeyStep::new(KeyCode::KEY_BACKSLASH.0),
        '|' => KeyStep::shift(KeyCode::KEY_BACKSLASH.0),
        '`' => KeyStep::new(KeyCode::KEY_GRAVE.0),
        '~' => KeyStep::shift(KeyCode::KEY_GRAVE.0),
        _ => return None,
    };

    Some(vec![simple])
}

pub fn is_valid_keyboard_tag(tag: &str) -> bool {
    let clean = tag.trim();
    let content = if clean.starts_with('[') && clean.ends_with(']') && clean.len() >= 2 {
        clean[1..clean.len() - 1].trim()
    } else {
        return false;
    };

    if content.is_empty() {
        return false;
    }

    let parts: Vec<&str> = content.split('+').collect();
    if parts.is_empty() {
        return false;
    }

    for part in parts {
        let part_clean = part.trim();
        if part_clean.is_empty() {
            return false;
        }
        let keys_abnt = parse_tag_to_keys(part_clean, crate::core::KeyboardLayout::Abnt2);
        let keys_us = parse_tag_to_keys(part_clean, crate::core::KeyboardLayout::US);
        if keys_abnt.is_empty() && keys_us.is_empty() {
            return false;
        }
    }

    true
}

fn parse_tag_to_keys(tag: &str, layout: crate::core::KeyboardLayout) -> Vec<u16> {
    let mut keys = Vec::new();
    let parts: Vec<&str> = tag.split('+').collect();

    for part in parts {
        let upper = part.trim().to_ascii_uppercase();
        let matched = match upper.as_str() {
            "CTRL" | "CONTROL" | "LCTRL" => Some(KeyCode::KEY_LEFTCTRL.0),
            "RCTRL" => Some(KeyCode::KEY_RIGHTCTRL.0),
            "SHIFT" | "LSHIFT" => Some(KeyCode::KEY_LEFTSHIFT.0),
            "RSHIFT" => Some(KeyCode::KEY_RIGHTSHIFT.0),
            "ALT" | "LALT" => Some(KeyCode::KEY_LEFTALT.0),
            "ALTGR" | "RALT" => Some(KeyCode::KEY_RIGHTALT.0),
            "WIN" | "SUPER" | "META" => Some(KeyCode::KEY_LEFTMETA.0),
            "ENTER" | "RETURN" => Some(KeyCode::KEY_ENTER.0),
            "ESC" | "ESCAPE" => Some(KeyCode::KEY_ESC.0),
            "TAB" => Some(KeyCode::KEY_TAB.0),
            "BACKSPACE" => Some(KeyCode::KEY_BACKSPACE.0),
            "SPACE" => Some(KeyCode::KEY_SPACE.0),
            "DEL" | "DELETE" => Some(KeyCode::KEY_DELETE.0),
            "INS" | "INSERT" => Some(KeyCode::KEY_INSERT.0),
            "UP" => Some(KeyCode::KEY_UP.0),
            "DOWN" => Some(KeyCode::KEY_DOWN.0),
            "LEFT" => Some(KeyCode::KEY_LEFT.0),
            "RIGHT" => Some(KeyCode::KEY_RIGHT.0),
            "HOME" => Some(KeyCode::KEY_HOME.0),
            "END" => Some(KeyCode::KEY_END.0),
            "PGUP" | "PAGEUP" => Some(KeyCode::KEY_PAGEUP.0),
            "PGDN" | "PAGEDOWN" => Some(KeyCode::KEY_PAGEDOWN.0),
            "PRTSC" | "PRINTSCREEN" => Some(KeyCode::KEY_SYSRQ.0),
            "CAPSLOCK" => Some(KeyCode::KEY_CAPSLOCK.0),
            "F1" => Some(KeyCode::KEY_F1.0),
            "F2" => Some(KeyCode::KEY_F2.0),
            "F3" => Some(KeyCode::KEY_F3.0),
            "F4" => Some(KeyCode::KEY_F4.0),
            "F5" => Some(KeyCode::KEY_F5.0),
            "F6" => Some(KeyCode::KEY_F6.0),
            "F7" => Some(KeyCode::KEY_F7.0),
            "F8" => Some(KeyCode::KEY_F8.0),
            "F9" => Some(KeyCode::KEY_F9.0),
            "F10" => Some(KeyCode::KEY_F10.0),
            "F11" => Some(KeyCode::KEY_F11.0),
            "F12" => Some(KeyCode::KEY_F12.0),
            other => {
                if other.len() == 1 {
                    char_to_input_steps(other.chars().next().unwrap(), layout)
                        .and_then(|steps| steps.first().map(|s| s.key))
                } else {
                    None
                }
            }
        };

        if let Some(code) = matched {
            keys.push(code);
        }
    }

    keys
}
