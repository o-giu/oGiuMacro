use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn can_access_uinput() -> bool {
    if is_root() {
        return true;
    }
    let uinput_ok = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/uinput")
        .is_ok();

    let input_dir = std::path::Path::new("/dev/input");
    let mut event_ok = false;
    if let Ok(entries) = std::fs::read_dir(input_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("event") {
                    if std::fs::File::open(&path).is_ok() {
                        event_ok = true;
                        break;
                    }
                }
            }
        }
    }

    uinput_ok && event_ok
}

pub const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/99-ogiumacro.rules";

pub fn is_udev_rule_installed() -> bool {
    Path::new(UDEV_RULE_PATH).exists()
}

pub fn has_passwordless_access() -> bool {
    can_access_uinput() || is_udev_rule_installed()
}

pub fn install_udev_rule() -> Result<(), String> {
    let rule_content = "KERNEL==\"uinput\", SUBSYSTEM==\"misc\", TAG+=\"uaccess\", OPTIONS+=\"static_node=uinput\", MODE=\"0660\", GROUP=\"input\"\nSUBSYSTEM==\"input\", ENV{ID_INPUT}==\"1\", TAG+=\"uaccess\"\nKERNEL==\"event*\", SUBSYSTEM==\"input\", TAG+=\"uaccess\", MODE=\"0660\", GROUP=\"input\"\n";

    let temp_script = format!(
        "cat << 'EOF' > {}
{}
EOF
udevadm control --reload-rules && udevadm trigger
",
        UDEV_RULE_PATH, rule_content
    );

    let mut cmd = Command::new("pkexec");
    cmd.arg("sh").arg("-c").arg(&temp_script);

    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("Command exited with status: {}", status)),
        Err(e) => Err(format!("Failed to execute pkexec: {}", e)),
    }
}

pub fn uninstall_udev_rule() -> Result<(), String> {
    let script = format!(
        "rm -f {} && udevadm control --reload-rules && udevadm trigger",
        UDEV_RULE_PATH
    );

    let mut cmd = Command::new("pkexec");
    cmd.arg("sh").arg("-c").arg(&script);

    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("Command exited with status: {}", status)),
        Err(e) => Err(format!("Failed to execute pkexec: {}", e)),
    }
}

pub fn elevate_if_needed() -> bool {
    if is_root() || can_access_uinput() {
        return false;
    }

    let current_exe = env::var("APPIMAGE")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| env::current_exe().ok());

    let Some(exe_path) = current_exe else {
        return false;
    };

    let pkexec_candidates = [
        "/usr/bin/pkexec",
        "/bin/pkexec",
        "/usr/local/bin/pkexec",
    ];
    let pkexec = pkexec_candidates
        .iter()
        .find(|&&path| Path::new(path).exists())
        .map(|&s| s.to_string());

    let pkexec_bin = pkexec.or_else(|| {
        env::var("PATH").ok().and_then(|p| {
            env::split_paths(&p)
                .map(|dir| dir.join("pkexec"))
                .find(|p| p.is_file())
                .and_then(|p| p.to_str().map(String::from))
        })
    });

    let Some(pkexec_bin) = pkexec_bin else {
        return false;
    };

    let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let xauthority = env::var("XAUTHORITY").unwrap_or_default();
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    let dbus_address = env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_default();

    let original_home = env::var("HOME").unwrap_or_default();

    let mut cmd = Command::new(pkexec_bin);
    cmd.arg("env");
    cmd.arg(format!("DISPLAY={}", display));
    if !wayland_display.is_empty() {
        cmd.arg(format!("WAYLAND_DISPLAY={}", wayland_display));
    }
    if !xauthority.is_empty() {
        cmd.arg(format!("XAUTHORITY={}", xauthority));
    }
    cmd.arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir));
    if !dbus_address.is_empty() {
        cmd.arg(format!("DBUS_SESSION_BUS_ADDRESS={}", dbus_address));
    }
    if !original_home.is_empty() {
        cmd.arg(format!("REAL_USER_HOME={}", original_home));
    }
    cmd.arg("XKB_DEFAULT_LAYOUT=br");
    cmd.arg(exe_path);

    if let Ok(mut child) = cmd.spawn() {
        let _ = child.wait();
        std::process::exit(0);
    }

    false
}

pub fn detect_primary_mouse() -> Option<(String, u16, u16)> {
    let content = fs::read_to_string("/proc/bus/input/devices").ok()?;
    for block in content.split("\n\n") {
        let mut name = String::new();
        let mut vendor: u16 = 0;
        let mut product: u16 = 0;
        let mut has_mouse_handler = false;
        let mut has_rel_xy = false;

        for line in block.lines() {
            if line.starts_with("I: ") {
                for part in line.split_whitespace() {
                    if let Some(hex_str) = part.strip_prefix("Vendor=") {
                        vendor = u16::from_str_radix(hex_str, 16).unwrap_or(0);
                    } else if let Some(hex_str) = part.strip_prefix("Product=") {
                        product = u16::from_str_radix(hex_str, 16).unwrap_or(0);
                    }
                }
            } else if line.starts_with("N: Name=") {
                name = line.trim_start_matches("N: Name=").trim_matches('"').to_string();
            } else if line.starts_with("H: Handlers=") {
                if line.contains("mouse") {
                    has_mouse_handler = true;
                }
            } else if line.starts_with("B: REL=") {
                if let Some(hex_str) = line.strip_prefix("B: REL=") {
                    if let Ok(rel_bits) = u64::from_str_radix(hex_str.trim(), 16) {
                        if (rel_bits & 3) == 3 {
                            has_rel_xy = true;
                        }
                    }
                }
            }
        }

        if has_mouse_handler && has_rel_xy {
            let lower = name.to_lowercase();
            if !lower.contains("keyboard") && !lower.contains("consumer") {
                return Some((name, vendor, product));
            }
        }
    }
    None
}
