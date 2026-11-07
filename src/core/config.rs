use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Mouse,
    Keyboard,
    Delay,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScrollDirection {
    #[default]
    Down,
    Up,
}

impl ScrollDirection {
    pub const ALL: [ScrollDirection; 2] = [
        ScrollDirection::Down,
        ScrollDirection::Up,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            ScrollDirection::Down => "Down",
            ScrollDirection::Up => "Up",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScrollSettings {
    pub amount: i32,
    pub direction: ScrollDirection,
    pub smooth: bool,
}

impl Default for ScrollSettings {
    fn default() -> Self {
        Self {
            amount: 3,
            direction: ScrollDirection::Down,
            smooth: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClickType {
    #[default]
    Left,
    Right,
    Middle,
    DoubleLeft,
    NoClick,
    DragAndDrop,
}

impl ClickType {
    pub const ALL: [ClickType; 6] = [
        ClickType::Left,
        ClickType::Right,
        ClickType::Middle,
        ClickType::DoubleLeft,
        ClickType::NoClick,
        ClickType::DragAndDrop,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            ClickType::Left => "Left Click",
            ClickType::Right => "Right Click",
            ClickType::Middle => "Middle Click",
            ClickType::DoubleLeft => "Double Left Click",
            ClickType::NoClick => "No Click (Move Only)",
            ClickType::DragAndDrop => "Drag & Drop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyboardLayout {
    #[default]
    Abnt2,
    US,
}

impl KeyboardLayout {
    pub const ALL: [KeyboardLayout; 2] = [
        KeyboardLayout::Abnt2,
        KeyboardLayout::US,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            KeyboardLayout::Abnt2 => "ABNT2 (Portuguese Brazil)",
            KeyboardLayout::US => "US (English United States)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CaptureEngine {
    #[default]
    Auto,
    Portal,
    Scrot,
    X11Memory,
    #[serde(alias = "Grim", alias = "Spectacle")]
    #[serde(other)]
    Fallback,
}

impl CaptureEngine {
    pub const ALL: [CaptureEngine; 4] = [
        CaptureEngine::Auto,
        CaptureEngine::Portal,
        CaptureEngine::Scrot,
        CaptureEngine::X11Memory,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            CaptureEngine::Auto => "Auto (Portal on Wayland / Scrot on X11)",
            CaptureEngine::Portal => "Portal ScreenCast (Wayland & Universal ~60 FPS)",
            CaptureEngine::Scrot => "Scrot (X11 / Fast ~20ms)",
            CaptureEngine::X11Memory => "Fast X11/XShm (Direct RAM ~2-5ms)",
            CaptureEngine::Fallback => "Auto (Portal on Wayland / Scrot on X11)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InputBehavior {
    #[default]
    Physical,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HardwareProfile {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
}

impl HardwareProfile {
    pub const DEFAULT_NAME: &'static str = "Logitech Keyboard & Mouse";
    pub const DEFAULT_VENDOR: u16 = 0x046d;
    pub const DEFAULT_PRODUCT: u16 = 0xc703;

    pub fn is_system_default(&self) -> bool {
        self.name.trim().eq_ignore_ascii_case(Self::DEFAULT_NAME)
            || (self.vendor_id == Self::DEFAULT_VENDOR && self.product_id == Self::DEFAULT_PRODUCT)
    }
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            name: Self::DEFAULT_NAME.to_string(),
            vendor_id: Self::DEFAULT_VENDOR,
            product_id: Self::DEFAULT_PRODUCT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseSettings {
    pub x: i32,
    pub y: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub click_type: ClickType,
    pub speed: f32,
    pub hold_time: f32,
    pub behavior: InputBehavior,
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            end_x: 0,
            end_y: 0,
            click_type: ClickType::Left,
            speed: 1.0,
            hold_time: 0.1,
            behavior: InputBehavior::Physical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardSettings {
    pub text: String,
    pub speed: f32,
    pub behavior: InputBehavior,
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        Self {
            text: String::new(),
            speed: 1.0,
            behavior: InputBehavior::Physical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Action {
    pub action_type: ActionType,
    pub mouse: MouseSettings,
    pub keyboard: KeyboardSettings,
    pub scroll: ScrollSettings,
    pub delay_time: f32,
    pub enabled: bool,
    pub use_image_recognition: bool,
    pub image_paths: Vec<String>,
    pub image_threshold: f32,
    pub image_poll_rate: f32,
    #[serde(default = "default_image_target_mouse")]
    pub image_target_mouse: bool,
}

fn default_image_target_mouse() -> bool {
    true
}

impl Default for Action {
    fn default() -> Self {
        Self::new_mouse()
    }
}

impl Action {
    pub fn new_mouse() -> Self {
        Self {
            action_type: ActionType::Mouse,
            mouse: MouseSettings::default(),
            keyboard: KeyboardSettings::default(),
            scroll: ScrollSettings::default(),
            delay_time: 0.5,
            enabled: true,
            use_image_recognition: false,
            image_paths: Vec::new(),
            image_threshold: 0.8,
            image_poll_rate: 0.5,
            image_target_mouse: true,
        }
    }

    pub fn new_keyboard() -> Self {
        Self {
            action_type: ActionType::Keyboard,
            mouse: MouseSettings::default(),
            keyboard: KeyboardSettings::default(),
            scroll: ScrollSettings::default(),
            delay_time: 0.5,
            enabled: true,
            use_image_recognition: false,
            image_paths: Vec::new(),
            image_threshold: 0.8,
            image_poll_rate: 0.5,
            image_target_mouse: true,
        }
    }

    pub fn new_delay() -> Self {
        Self {
            action_type: ActionType::Delay,
            mouse: MouseSettings::default(),
            keyboard: KeyboardSettings::default(),
            scroll: ScrollSettings::default(),
            delay_time: 0.5,
            enabled: true,
            use_image_recognition: false,
            image_paths: Vec::new(),
            image_threshold: 0.8,
            image_poll_rate: 0.5,
            image_target_mouse: true,
        }
    }

    pub fn new_scroll() -> Self {
        Self {
            action_type: ActionType::Scroll,
            mouse: MouseSettings::default(),
            keyboard: KeyboardSettings::default(),
            scroll: ScrollSettings::default(),
            delay_time: 0.5,
            enabled: true,
            use_image_recognition: false,
            image_paths: Vec::new(),
            image_threshold: 0.8,
            image_poll_rate: 0.5,
            image_target_mouse: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Macro {
    pub name: String,
    pub actions: Vec<Action>,
    pub start_hotkey: u32,
    pub stop_hotkey: u32,
    pub pick_window_hotkey: u32,
    pub is_active: bool,
    pub is_loop: bool,
    pub loop_count: u32,
    pub human_like: bool,
    pub left_handed_override: bool,
    pub target_window_enabled: bool,
    pub target_method: usize,
    pub target_window_idx: usize,
    pub custom_window_name: String,
    pub minimize_and_stay_active: bool,
    pub hide_seconds: f32,
    pub active_seconds: f32,
    pub hardware_idx: usize,
    #[serde(default)]
    pub hardware_profile: Option<HardwareProfile>,
    pub keyboard_layout: KeyboardLayout,
    #[serde(default)]
    pub capture_engine: CaptureEngine,
    #[serde(default)]
    pub quick_tags: Vec<String>,
}

impl Default for Macro {
    fn default() -> Self {
        Self::new("Untitled Macro")
    }
}

impl Macro {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            actions: Vec::new(),
            start_hotkey: 297,
            stop_hotkey: 298,
            pick_window_hotkey: 295,
            is_active: false,
            is_loop: false,
            loop_count: 0,
            human_like: false,
            left_handed_override: false,
            target_window_enabled: false,
            target_method: 0,
            target_window_idx: 0,
            custom_window_name: String::new(),
            minimize_and_stay_active: false,
            hide_seconds: 2.0,
            active_seconds: 5.0,
            hardware_idx: 0,
            hardware_profile: None,
            keyboard_layout: KeyboardLayout::Abnt2,
            capture_engine: CaptureEngine::Auto,
            quick_tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RecordedInputEvent {
    pub timestamp_ms: u64,
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RecordedMacro {
    pub name: String,
    pub events: Vec<RecordedInputEvent>,
    pub start_rec_hotkey: u32,
    pub stop_rec_hotkey: u32,
    pub start_play_hotkey: u32,
    pub stop_play_hotkey: u32,
    pub is_recording: bool,
    pub is_playing: bool,
    pub is_loop: bool,
    pub loop_count: u32,
    pub playback_speed: f32,
    pub left_handed_override: bool,
    #[serde(default = "default_start_coord")]
    pub start_x: i32,
    #[serde(default = "default_start_coord")]
    pub start_y: i32,
}

fn default_start_coord() -> i32 {
    -1
}

impl Default for RecordedMacro {
    fn default() -> Self {
        Self::new("Untitled Recording")
    }
}

impl RecordedMacro {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            events: Vec::new(),
            start_rec_hotkey: 295,
            stop_rec_hotkey: 296,
            start_play_hotkey: 297,
            stop_play_hotkey: 298,
            is_recording: false,
            is_playing: false,
            is_loop: false,
            loop_count: 0,
            playback_speed: 1.0,
            left_handed_override: false,
            start_x: -1,
            start_y: -1,
        }
    }
}

pub fn fkey_name(code: u32) -> &'static str {
    match code {
        290 => "F1",
        291 => "F2",
        292 => "F3",
        293 => "F4",
        294 => "F5",
        295 => "F6",
        296 => "F7",
        297 => "F8",
        298 => "F9",
        299 => "F10",
        300 => "F11",
        301 => "F12",
        _ => "None",
    }
}

#[allow(dead_code)]
pub fn fkey_to_evdev(code: u32) -> u16 {
    match code {
        290 => 59,
        291 => 60,
        292 => 61,
        293 => 62,
        294 => 63,
        295 => 64,
        296 => 65,
        297 => 66,
        298 => 67,
        299 => 68,
        300 => 87,
        301 => 88,
        _ => 0,
    }
}

pub fn evdev_to_fkey(ev_code: u16) -> Option<u32> {
    match ev_code {
        59 => Some(290),
        60 => Some(291),
        61 => Some(292),
        62 => Some(293),
        63 => Some(294),
        64 => Some(295),
        65 => Some(296),
        66 => Some(297),
        67 => Some(298),
        68 => Some(299),
        87 => Some(300),
        88 => Some(301),
        _ => None,
    }
}

