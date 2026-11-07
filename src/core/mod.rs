pub mod config;
pub mod input_capture;
pub mod input_emulate;
pub mod macro_engine;
pub mod privileges;
pub mod screen_image;
pub mod storage;
pub mod system_diagnosis;
pub mod window_target;

pub use config::{
    evdev_to_fkey, fkey_name, Action, ActionType, CaptureEngine, ClickType, HardwareProfile,
    KeyboardLayout, Macro, RecordedInputEvent, RecordedMacro, ScrollDirection,
};
pub use input_capture::{GlobalHotkeyListener, InputRecorder};
pub use input_emulate::is_valid_keyboard_tag;
pub use macro_engine::MacroEngine;
pub use privileges::{detect_primary_mouse, elevate_if_needed};
#[allow(unused_imports)]
pub use screen_image::{
    capture_screen, capture_screen_raw, capture_screen_with_engine,
    enable_kde_xwayland_eavesdropping, find_all_template_matches_with_roi, find_template_match, find_template_match_with_roi, get_mouse_location,
    get_screen_resolution, has_native_screen_picker, is_kde_xwayland_eavesdropping_enabled,
    is_screen_capture_worker_running, match_template, pick_screen_coordinates,
    start_screen_capture_worker, start_screen_capture_worker_with_engine, stop_screen_capture_worker,
    TemplatePattern,
};
#[allow(unused_imports)]
pub use storage::{
    delete_config_folder, delete_exported_file, detect_macro_file_type, export_macro, export_rec_macro, get_config_dir, get_recordings_dir, get_workflows_dir, import_macro, import_rec_macro, list_exported_files,
    list_exported_files_for_type, load_custom_quick_tags, load_main_macros, load_rec_macros, load_skip_startup_diagnosis, open_config_folder, pick_image_files, save_custom_quick_tags, save_main_macros, save_rec_macros, save_skip_startup_diagnosis, MacroFileType,
};
#[allow(unused_imports)]
pub use system_diagnosis::{detect_linux_distro, which_command, LinuxDistro, SystemDiagnosis};
pub use window_target::{
    activate_window, get_active_window_title, is_window_focused, list_detected_windows,
    minimize_window,
};
