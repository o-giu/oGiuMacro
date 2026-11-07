use std::env;
use std::ffi::CStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{HardwareProfile, Macro, RecordedMacro};

pub fn get_user_home() -> PathBuf {
    if let Ok(real_home) = env::var("REAL_USER_HOME") {
        let p = PathBuf::from(real_home.trim());
        if p.exists() {
            return p;
        }
    }

    if let Ok(pk_uid_str) = env::var("PKEXEC_UID") {
        if let Ok(uid) = pk_uid_str.parse::<u32>() {
            unsafe {
                let pw = libc::getpwuid(uid);
                if !pw.is_null() && !(*pw).pw_dir.is_null() {
                    let dir_cstr = CStr::from_ptr((*pw).pw_dir);
                    if let Ok(dir_str) = dir_cstr.to_str() {
                        return PathBuf::from(dir_str);
                    }
                }
            }
        }
    }

    if let Ok(sudo_user) = env::var("SUDO_USER") {
        let user_c = std::ffi::CString::new(sudo_user).unwrap_or_default();
        unsafe {
            let pw = libc::getpwnam(user_c.as_ptr());
            if !pw.is_null() && !(*pw).pw_dir.is_null() {
                let dir_cstr = CStr::from_ptr((*pw).pw_dir);
                if let Ok(dir_str) = dir_cstr.to_str() {
                    return PathBuf::from(dir_str);
                }
            }
        }
    }

    if let Ok(home) = env::var("HOME") {
        let p = PathBuf::from(&home);
        
        if home != "/root" && p.exists() {
            return p;
        }
    }

    if let Ok(user) = env::var("USER") {
        if user != "root" && !user.is_empty() {
            let user_home = PathBuf::from(format!("/home/{}", user));
            if user_home.exists() {
                return user_home;
            }
        }
    }

    if let Ok(entries) = fs::read_dir("/home") {
        let mut users = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                users.push(path);
            }
        }
        if users.len() == 1 {
            return users.remove(0);
        }
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home);
    }

    PathBuf::from("/tmp")
}

pub fn pick_image_files() -> Vec<PathBuf> {
    let home = get_user_home();
    let mut initial_str = home.to_string_lossy().to_string();
    if !initial_str.ends_with('/') {
        initial_str.push('/');
    }

    let zenity_res = std::process::Command::new("zenity")
        .arg("--file-selection")
        .arg("--multiple")
        .arg("--separator=\n")
        .arg(format!("--filename={}", initial_str))
        .arg("--file-filter=Images (PNG, JPG, BMP) | *.png *.PNG *.jpg *.JPG *.jpeg *.JPEG *.bmp *.BMP")
        .arg("--title=Select Image Templates")
        .output();

    if let Ok(output) = zenity_res {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let paths: Vec<PathBuf> = stdout
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(PathBuf::from)
                .collect();
            if !paths.is_empty() {
                return paths;
            }
        }
        
        if output.status.code() == Some(1) {
            return Vec::new();
        }
    }

    if let Some(files) = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
        .set_title("Select Image Templates for Action")
        .set_directory(&home)
        .pick_files()
    {
        return files;
    }

    Vec::new()
}

pub fn get_real_uid_gid() -> Option<(u32, u32)> {
    if let Ok(pk_uid_str) = env::var("PKEXEC_UID") {
        if let Ok(uid) = pk_uid_str.parse::<u32>() {
            unsafe {
                let pw = libc::getpwuid(uid);
                if !pw.is_null() {
                    return Some((uid, (*pw).pw_gid));
                }
            }
            return Some((uid, uid));
        }
    }

    if let Ok(sudo_user) = env::var("SUDO_USER") {
        let user_c = std::ffi::CString::new(sudo_user).unwrap_or_default();
        unsafe {
            let pw = libc::getpwnam(user_c.as_ptr());
            if !pw.is_null() {
                return Some(((*pw).pw_uid, (*pw).pw_gid));
            }
        }
    }

    None
}

fn fix_ownership(path: &Path) {
    if let Some((uid, gid)) = get_real_uid_gid() {
        if let Ok(c_path) = std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
            unsafe {
                libc::chown(c_path.as_ptr(), uid, gid);
            }
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    let mut dir = get_user_home();
    dir.push(".config");
    dir.push("oGiuMacro");
    let _ = fs::create_dir_all(&dir);
    fix_ownership(&dir);
    dir
}

pub fn get_workflows_dir() -> PathBuf {
    let dir = get_config_dir().join("workflows");
    let _ = fs::create_dir_all(&dir);
    fix_ownership(&dir);
    dir
}

pub fn get_recordings_dir() -> PathBuf {
    let dir = get_config_dir().join("recordings");
    let _ = fs::create_dir_all(&dir);
    fix_ownership(&dir);
    dir
}

pub fn delete_config_folder() -> Result<(), String> {
    let mut dir = get_user_home();
    dir.push(".config");
    dir.push("oGiuMacro");
    if dir.exists() && dir.is_dir() {
        let is_valid_name = dir.file_name() == Some(std::ffi::OsStr::new("oGiuMacro"));
        let is_in_config = dir.parent().and_then(|p| p.file_name()) == Some(std::ffi::OsStr::new(".config"));
        if is_valid_name && is_in_config {
            fs::remove_dir_all(&dir).map_err(|e| format!("Failed to delete configuration directory: {}", e))?;
        }
    }
    Ok(())
}

pub fn open_config_folder() {
    let dir = get_config_dir();
    let dir_str = dir.to_string_lossy().to_string();

    let is_elevated = unsafe { libc::geteuid() == 0 };
    if is_elevated {
        if let Some((uid, _)) = get_real_uid_gid() {
            unsafe {
                let pw = libc::getpwuid(uid);
                if !pw.is_null() && !(*pw).pw_name.is_null() {
                    let user_cstr = std::ffi::CStr::from_ptr((*pw).pw_name);
                    if let Ok(user_str) = user_cstr.to_str() {
                        let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
                        let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
                        let xdg_runtime_dir = format!("/run/user/{}", uid);

                        let mut cmd = std::process::Command::new("sudo");
                        cmd.arg("-u").arg(user_str).arg("env");
                        cmd.arg(format!("DISPLAY={}", display));
                        if !wayland_display.is_empty() {
                            cmd.arg(format!("WAYLAND_DISPLAY={}", wayland_display));
                        }
                        cmd.arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir));
                        cmd.arg("xdg-open").arg(&dir_str);
                        let _ = cmd.spawn();
                        return;
                    }
                }
            }
        }
    }

    let _ = std::process::Command::new("xdg-open").arg(&dir_str).spawn();
}

pub fn write_config_file(path: &Path, content: &str) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or(path);
    let tmp_path = parent.join(format!(".tmp_{}", uuid_simple()));
    fs::write(&tmp_path, content)?;
    fix_ownership(&tmp_path);
    fs::rename(&tmp_path, path)?;
    fix_ownership(path);
    Ok(())
}

fn uuid_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
}

pub fn save_main_macros(macros: &[Macro], hardware: &[HardwareProfile]) -> std::io::Result<()> {
    let dir = get_config_dir();

    let macros_file = dir.join("macros.json");
    let macros_json = serde_json::to_string_pretty(macros)?;
    write_config_file(&macros_file, &macros_json)?;

    let hw_file = dir.join("hardware.json");
    let hw_json = serde_json::to_string_pretty(hardware)?;
    write_config_file(&hw_file, &hw_json)?;

    Ok(())
}

pub fn load_main_macros() -> Option<(Vec<Macro>, Vec<HardwareProfile>)> {
    let dir = get_config_dir();
    let macros_file = dir.join("macros.json");
    let hw_file = dir.join("hardware.json");

    if !macros_file.exists() {
        return None;
    }

    let macros_data = fs::read_to_string(macros_file).ok()?;
    let macros: Vec<Macro> = serde_json::from_str(&macros_data).ok()?;

    let mut hardware = if hw_file.exists() {
        fs::read_to_string(hw_file)
            .ok()
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or_else(|| vec![HardwareProfile::default()])
    } else {
        vec![HardwareProfile::default()]
    };

    if !hardware.iter().any(|hp| hp.is_system_default()) {
        hardware.insert(0, HardwareProfile::default());
    }

    Some((macros, hardware))
}

pub fn save_rec_macros(recordings: &[RecordedMacro]) -> std::io::Result<()> {
    let dir = get_config_dir();
    let rec_file = dir.join("recordings.json");
    let rec_json = serde_json::to_string_pretty(recordings)?;
    write_config_file(&rec_file, &rec_json)?;
    Ok(())
}

pub fn load_rec_macros() -> Option<Vec<RecordedMacro>> {
    let dir = get_config_dir();
    let rec_file = dir.join("recordings.json");

    if !rec_file.exists() {
        return None;
    }

    let rec_data = fs::read_to_string(rec_file).ok()?;
    serde_json::from_str(&rec_data).ok()
}

pub fn save_custom_quick_tags(tags: &[String]) -> std::io::Result<()> {
    let dir = get_config_dir();
    let tags_file = dir.join("quick_tags.json");
    let json = serde_json::to_string_pretty(tags)?;
    write_config_file(&tags_file, &json)?;
    Ok(())
}

pub fn load_custom_quick_tags() -> Option<Vec<String>> {
    let dir = get_config_dir();
    let tags_file = dir.join("quick_tags.json");
    if !tags_file.exists() {
        return None;
    }
    let data = fs::read_to_string(tags_file).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_skip_startup_diagnosis(skip: bool) -> std::io::Result<()> {
    let dir = get_config_dir();
    let settings_file = dir.join("skip_diagnosis.json");
    let json = serde_json::to_string_pretty(&skip)?;
    write_config_file(&settings_file, &json)?;
    Ok(())
}

pub fn load_skip_startup_diagnosis() -> bool {
    let dir = get_config_dir();
    let settings_file = dir.join("skip_diagnosis.json");
    if !settings_file.exists() {
        return false;
    }
    let data = fs::read_to_string(settings_file).ok().unwrap_or_default();
    serde_json::from_str(&data).unwrap_or(false)
}

pub fn export_macro(macro_item: &Macro, filename: &str) -> std::io::Result<PathBuf> {
    let dir = get_workflows_dir();
    let clean_base = filename.trim().trim_end_matches(".json").trim_end_matches('.').trim();
    let sanitized_base: String = clean_base.chars().filter(|c| !matches!(c, '/' | '\\' | '\0' | ':' | '*' | '?' | '"' | '<' | '>' | '|')).collect();
    let final_name = if sanitized_base.is_empty() {
        "macro.json".to_string()
    } else {
        format!("{}.json", sanitized_base)
    };
    let target = dir.join(final_name);
    let json = serde_json::to_string_pretty(macro_item)?;
    write_config_file(&target, &json)?;
    Ok(target)
}

pub fn import_macro(path: &Path) -> std::io::Result<Macro> {
    match detect_macro_file_type(path) {
        MacroFileType::Workflow => {
            let data = fs::read_to_string(path)?;
            let parsed: Macro = serde_json::from_str(&data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            Ok(parsed)
        }
        MacroFileType::Recording => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "This file is a Recording macro (Rec/Play), not a Workflow macro.",
        )),
        MacroFileType::Unknown => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unrecognized macro format.",
        )),
    }
}

pub fn export_rec_macro(rec_item: &RecordedMacro, filename: &str) -> std::io::Result<PathBuf> {
    let dir = get_recordings_dir();
    let clean_base = filename.trim().trim_end_matches(".json").trim_end_matches('.').trim();
    let sanitized_base: String = clean_base.chars().filter(|c| !matches!(c, '/' | '\\' | '\0' | ':' | '*' | '?' | '"' | '<' | '>' | '|')).collect();
    let final_name = if sanitized_base.is_empty() {
        "recording.json".to_string()
    } else {
        format!("{}.json", sanitized_base)
    };
    let target = dir.join(final_name);
    let json = serde_json::to_string_pretty(rec_item)?;
    write_config_file(&target, &json)?;
    Ok(target)
}

pub fn import_rec_macro(path: &Path) -> std::io::Result<RecordedMacro> {
    match detect_macro_file_type(path) {
        MacroFileType::Recording => {
            let data = fs::read_to_string(path)?;
            let parsed: RecordedMacro = serde_json::from_str(&data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            Ok(parsed)
        }
        MacroFileType::Workflow => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "This file is a Workflow macro, not a Recording macro.",
        )),
        MacroFileType::Unknown => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unrecognized recording format.",
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroFileType {
    Workflow,
    Recording,
    Unknown,
}

pub fn is_system_config_file(name: &str) -> bool {
    matches!(
        name,
        "macros.json" | "recordings.json" | "hardware.json" | "quick_tags.json" | "skip_diagnosis.json"
    ) || name.starts_with('.')
}

pub fn detect_macro_file_type(path: &Path) -> MacroFileType {
    if !path.is_file() {
        return MacroFileType::Unknown;
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if is_system_config_file(name) {
        return MacroFileType::Unknown;
    }
    if path.extension().and_then(|e| e.to_str()) != Some("json") {
        return MacroFileType::Unknown;
    }
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if val.get("actions").is_some() && serde_json::from_value::<Macro>(val.clone()).is_ok() {
                return MacroFileType::Workflow;
            }
            if val.get("events").is_some() && serde_json::from_value::<RecordedMacro>(val).is_ok() {
                return MacroFileType::Recording;
            }
        }
    }
    MacroFileType::Unknown
}

pub fn is_valid_macro_file(path: &Path) -> bool {
    detect_macro_file_type(path) != MacroFileType::Unknown
}

pub fn list_exported_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in [get_workflows_dir(), get_recordings_dir(), get_config_dir()] {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && is_valid_macro_file(&path) {
                    if !files.contains(&path) {
                        files.push(path);
                    }
                }
            }
        }
    }
    files.sort();
    files
}

pub fn list_exported_files_for_type(expected: MacroFileType) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let sub_dir = match expected {
        MacroFileType::Workflow => get_workflows_dir(),
        MacroFileType::Recording => get_recordings_dir(),
        MacroFileType::Unknown => get_config_dir(),
    };

    if let Ok(entries) = fs::read_dir(sub_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if detect_macro_file_type(&path) == expected {
                files.push(path);
            }
        }
    }

    if let Ok(entries) = fs::read_dir(get_config_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && detect_macro_file_type(&path) == expected {
                if !files.iter().any(|p| p.file_name() == path.file_name()) {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    files
}

pub fn delete_exported_file(path: &Path) -> Result<(), String> {
    if path.exists() && path.is_file() {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !is_system_config_file(name) {
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let config_dir = get_config_dir();
                if let (Ok(c_canon), Ok(p_canon)) = (config_dir.canonicalize(), path.canonicalize()) {
                    if p_canon.starts_with(&c_canon) && p_canon != c_canon {
                        fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;
                        return Ok(());
                    }
                }
            }
        }
    }
    Err("Invalid file or file not in config directory".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Action, ClickType};

    #[test]
    fn test_macro_serialization_roundtrip() {
        let mut m = Macro::new("Test Macro");
        m.loop_count = 5;
        let mut a = Action::new_mouse();
        a.mouse.x = 100;
        a.mouse.y = 200;
        m.actions.push(a);

        let json = serde_json::to_string(&m).unwrap();
        let decoded: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "Test Macro");
        assert_eq!(decoded.loop_count, 5);
        assert_eq!(decoded.actions.len(), 1);
        assert_eq!(decoded.actions[0].mouse.x, 100);
        assert_eq!(decoded.actions[0].mouse.y, 200);
    }

    #[test]
    fn test_recorded_macro_serialization_roundtrip() {
        let mut rm = RecordedMacro::new("Rec 1");
        rm.events.push(crate::core::RecordedInputEvent {
            timestamp_ms: 120,
            event_type: 1,
            code: 30,
            value: 1,
        });

        let json = serde_json::to_string(&rm).unwrap();
        let decoded: RecordedMacro = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "Rec 1");
        assert_eq!(decoded.events.len(), 1);
        assert_eq!(decoded.events[0].code, 30);
    }

    #[test]
    fn test_export_and_import_macro() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("ogiumacro_test_export.json");

        let m = Macro::new("Export Test");
        let json = serde_json::to_string_pretty(&m).unwrap();
        fs::write(&file_path, json).unwrap();

        let imported = import_macro(&file_path).unwrap();
        assert_eq!(imported.name, "Export Test");

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_prevent_cross_import_between_workflow_and_recording() {
        let temp_dir = std::env::temp_dir();
        let wf_path = temp_dir.join("ogiumacro_wf_test.json");
        let rec_path = temp_dir.join("ogiumacro_rec_test.json");

        let m = Macro::new("WF Test");
        let wf_json = serde_json::to_string_pretty(&m).unwrap();
        fs::write(&wf_path, wf_json).unwrap();

        let rm = RecordedMacro::new("REC Test");
        let rec_json = serde_json::to_string_pretty(&rm).unwrap();
        fs::write(&rec_path, rec_json).unwrap();

        assert_eq!(detect_macro_file_type(&wf_path), MacroFileType::Workflow);
        assert_eq!(detect_macro_file_type(&rec_path), MacroFileType::Recording);

        assert!(import_macro(&wf_path).is_ok());
        assert!(import_rec_macro(&rec_path).is_ok());

        assert!(import_macro(&rec_path).is_err());
        assert!(import_rec_macro(&wf_path).is_err());

        let _ = fs::remove_file(wf_path);
        let _ = fs::remove_file(rec_path);
    }

    #[test]
    fn test_export_macro_filename_sanitization() {
        let m = Macro::new("Sanitize Test");
        let path = export_macro(&m, "test/sub:dir\\macro..json.json").unwrap();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.ends_with(".json"));
        assert!(!name.contains('/'));
        assert!(!name.contains('\\'));
        assert!(!name.contains(':'));
        assert_eq!(name, "testsubdir__macro.json".replace("__", ""));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_backward_compatibility_missing_fields() {
        let legacy_json = r#"{
            "name": "Legacy Macro",
            "actions": [
                {
                    "action_type": "Mouse",
                    "mouse": {
                        "x": 450,
                        "y": 600
                    }
                }
            ]
        }"#;

        let parsed: Result<Macro, _> = serde_json::from_str(legacy_json);
        assert!(parsed.is_ok());
        let m = parsed.unwrap();
        assert_eq!(m.name, "Legacy Macro");
        assert_eq!(m.actions.len(), 1);
        assert_eq!(m.actions[0].mouse.x, 450);
        assert_eq!(m.actions[0].mouse.y, 600);
        assert_eq!(m.actions[0].mouse.click_type, ClickType::Left);
        assert_eq!(m.loop_count, 0);
        assert!(!m.human_like);
    }

    #[test]
    fn test_scroll_action_serialization_roundtrip() {
        let mut m = Macro::new("Scroll Macro");
        let mut a = Action::new_scroll();
        a.use_image_recognition = true;
        a.image_paths.push("/home/user/test.png".to_string());
        a.image_threshold = 0.85;
        a.image_target_mouse = true;
        m.actions.push(a);

        let json = serde_json::to_string(&m).unwrap();
        let decoded: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.actions.len(), 1);
        assert_eq!(decoded.actions[0].action_type, crate::core::ActionType::Scroll);
        assert_eq!(decoded.actions[0].scroll.amount, 3);
        assert_eq!(decoded.actions[0].scroll.direction, crate::core::ScrollDirection::Down);
        assert!(decoded.actions[0].use_image_recognition);
        assert_eq!(decoded.actions[0].image_paths.len(), 1);
        assert_eq!(decoded.actions[0].image_threshold, 0.85);
        assert!(decoded.actions[0].image_target_mouse);
    }

    #[test]
    fn test_delete_config_folder_simulation() {
        let temp_dir = std::env::temp_dir();
        let target_dir = temp_dir.join("ogiumacro_sim_delete").join(".config").join("oGiuMacro");
        fs::create_dir_all(&target_dir).unwrap();
        let test_file = target_dir.join("macros.json");
        fs::write(&test_file, "{}").unwrap();
        assert!(test_file.exists());

        let wf_sub = target_dir.join("workflows");
        fs::create_dir_all(&wf_sub).unwrap();
        let wf_file = wf_sub.join("Farm.json");
        fs::write(&wf_file, "{}").unwrap();

        let rec_sub = target_dir.join("recordings");
        fs::create_dir_all(&rec_sub).unwrap();
        let rec_file = rec_sub.join("Farm.json");
        fs::write(&rec_file, "{}").unwrap();

        assert!(wf_file.exists());
        assert!(rec_file.exists());

        let is_valid_name = target_dir.file_name() == Some(std::ffi::OsStr::new("oGiuMacro"));
        let is_in_config = target_dir.parent().and_then(|p| p.file_name()) == Some(std::ffi::OsStr::new(".config"));
        if target_dir.exists() && target_dir.is_dir() && is_valid_name && is_in_config {
            let res = fs::remove_dir_all(&target_dir);
            assert!(res.is_ok());
        }
        assert!(!target_dir.exists());
    }

    #[test]
    fn test_delete_exported_file_safety() {
        let temp_dir = std::env::temp_dir();
        let invalid_file = temp_dir.join("outside_config.json");
        fs::write(&invalid_file, "{}").unwrap();
        let res = delete_exported_file(&invalid_file);
        assert!(res.is_err());
        let _ = fs::remove_file(&invalid_file);
    }

    #[test]
    fn test_is_system_config_file() {
        assert!(is_system_config_file("macros.json"));
        assert!(is_system_config_file("recordings.json"));
        assert!(is_system_config_file("hardware.json"));
        assert!(is_system_config_file("quick_tags.json"));
        assert!(is_system_config_file("skip_diagnosis.json"));
        assert!(is_system_config_file(".hidden.json"));
        assert!(!is_system_config_file("MyMacro.json"));
        assert!(!is_system_config_file("recording_game.json"));
    }

    #[test]
    fn test_detect_macro_file_type() {
        let temp_dir = std::env::temp_dir();
        let wf_path = temp_dir.join("test_detect_wf.json");
        let rec_path = temp_dir.join("test_detect_rec.json");
        let tags_path = temp_dir.join("quick_tags.json");

        let m = Macro::new("Detect WF");
        fs::write(&wf_path, serde_json::to_string(&m).unwrap()).unwrap();

        let rm = RecordedMacro::new("Detect Rec");
        fs::write(&rec_path, serde_json::to_string(&rm).unwrap()).unwrap();

        fs::write(&tags_path, r#"["[CTRL+C]", "[CTRL+V]"]"#).unwrap();

        assert_eq!(detect_macro_file_type(&wf_path), MacroFileType::Workflow);
        assert_eq!(detect_macro_file_type(&rec_path), MacroFileType::Recording);
        assert_eq!(detect_macro_file_type(&tags_path), MacroFileType::Unknown);

        let _ = fs::remove_file(&wf_path);
        let _ = fs::remove_file(&rec_path);
        let _ = fs::remove_file(&tags_path);
    }

    #[test]
    fn test_macro_with_quick_tags_roundtrip() {
        let mut m = Macro::new("Tags Macro");
        m.quick_tags = vec!["[CTRL+C]".to_string(), "[WIN+D]".to_string()];
        let json = serde_json::to_string(&m).unwrap();
        let decoded: Macro = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.quick_tags.len(), 2);
        assert_eq!(decoded.quick_tags[0], "[CTRL+C]");
        assert_eq!(decoded.quick_tags[1], "[WIN+D]");
    }

    #[test]
    fn test_same_name_in_workflow_and_recording_isolation() {
        let m = Macro::new("Farm");
        let wf_path = export_macro(&m, "Farm.json").unwrap();
        assert_eq!(wf_path.parent().unwrap(), get_workflows_dir());

        let rm = RecordedMacro::new("Farm");
        let rec_path = export_rec_macro(&rm, "Farm.json").unwrap();
        assert_eq!(rec_path.parent().unwrap(), get_recordings_dir());

        assert!(wf_path.exists());
        assert!(rec_path.exists());
        assert_ne!(wf_path, rec_path);

        let wf_list = list_exported_files_for_type(MacroFileType::Workflow);
        assert!(wf_list.contains(&wf_path));
        assert!(!wf_list.contains(&rec_path));

        let rec_list = list_exported_files_for_type(MacroFileType::Recording);
        assert!(rec_list.contains(&rec_path));
        assert!(!rec_list.contains(&wf_path));

        let all_list = list_exported_files();
        assert!(all_list.contains(&wf_path));
        assert!(all_list.contains(&rec_path));

        let imported_wf = import_macro(&wf_path).unwrap();
        assert_eq!(imported_wf.name, "Farm");
        assert!(import_rec_macro(&wf_path).is_err());

        let imported_rec = import_rec_macro(&rec_path).unwrap();
        assert_eq!(imported_rec.name, "Farm");
        assert!(import_macro(&rec_path).is_err());

        assert!(delete_exported_file(&wf_path).is_ok());
        assert!(!wf_path.exists());
        assert!(rec_path.exists());

        assert!(delete_exported_file(&rec_path).is_ok());
        assert!(!rec_path.exists());
    }
}
