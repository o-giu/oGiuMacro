use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::Command;

fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn get_real_user_and_uid() -> (String, u32) {
    if let Ok(pk_uid_str) = env::var("PKEXEC_UID") {
        if let Ok(uid) = pk_uid_str.parse::<u32>() {
            unsafe {
                let pw = libc::getpwuid(uid);
                if !pw.is_null() && !(*pw).pw_name.is_null() {
                    let user_cstr = std::ffi::CStr::from_ptr((*pw).pw_name);
                    if let Ok(user_str) = user_cstr.to_str() {
                        return (user_str.to_string(), uid);
                    }
                }
            }
            return (String::new(), uid);
        }
    }

    if let Ok(sudo_user) = env::var("SUDO_USER") {
        let user_c = std::ffi::CString::new(sudo_user.clone()).unwrap_or_default();
        unsafe {
            let pw = libc::getpwnam(user_c.as_ptr());
            if !pw.is_null() {
                return (sudo_user, (*pw).pw_uid);
            }
        }
    }

    let uid = unsafe { libc::getuid() };
    (env::var("USER").unwrap_or_default(), uid)
}

fn build_user_command(program: &str, args: &[&str]) -> Command {
    let (real_user, real_uid) = get_real_user_and_uid();
    let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let xauthority = env::var("XAUTHORITY").unwrap_or_default();
    let xdg_runtime_dir = format!("/run/user/{}", real_uid);

    if is_elevated() && !real_user.is_empty() {
        let mut cmd = Command::new("sudo");
        cmd.arg("-u").arg(&real_user).arg("env");
        cmd.arg(format!("DISPLAY={}", display));
        if !wayland_display.is_empty() {
            cmd.arg(format!("WAYLAND_DISPLAY={}", wayland_display));
        }
        if !xauthority.is_empty() {
            cmd.arg(format!("XAUTHORITY={}", xauthority));
        }
        cmd.arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir));
        cmd.arg(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd
    } else {
        let mut cmd = Command::new(program);
        for arg in args {
            cmd.arg(arg);
        }
        cmd
    }
}

pub fn list_detected_windows() -> Vec<String> {
    let mut set = BTreeSet::new();
    set.insert("Active Window".to_string());

    let mut cmd = build_user_command("wmctrl", &["-l"]);
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let title = parts[3..].join(" ");
                    if !title.is_empty() {
                        set.insert(title);
                    }
                }
            }
        }
    }

    let mut hypr_cmd = build_user_command("hyprctl", &["clients"]);
    if let Ok(output) = hypr_cmd.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if let Some(title) = trimmed.strip_prefix("title: ") {
                    let clean = title.trim();
                    if !clean.is_empty() {
                        set.insert(clean.to_string());
                    }
                }
            }
        }
    }

    let mut sway_cmd = build_user_command("swaymsg", &["-t", "get_tree"]);
    if let Ok(output) = sway_cmd.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            extract_sway_titles(&stdout, &mut set);
        }
    }

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.chars().all(|c| c.is_ascii_digit()) {
                        let comm_file = path.join("comm");
                        if let Ok(comm) = fs::read_to_string(comm_file) {
                            let trimmed = comm.trim();
                            if trimmed.len() > 2 {
                                set.insert(format!("[App] {}", trimmed));
                            }
                        }
                    }
                }
            }
        }
    }

    set.into_iter().collect()
}

fn extract_sway_titles(json_str: &str, set: &mut BTreeSet<String>) {
    for line in json_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"name\":") {
            if let Some(start) = trimmed.find('"') {
                if let Some(after) = trimmed.get(start + 7..) {
                    if let Some(q_start) = after.find('"') {
                        if let Some(q_end) = after[q_start + 1..].rfind('"') {
                            let title = &after[q_start + 1..q_start + 1 + q_end];
                            if !title.is_empty() && !title.starts_with("__i3") {
                                set.insert(title.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_window_title_by_id(win_id: &str) -> Option<String> {
    let mut xprop_name = build_user_command("xprop", &["-id", win_id, "_NET_WM_NAME", "WM_NAME"]);
    if let Ok(name_out) = xprop_name.output() {
        if name_out.status.success() {
            let name_stdout = String::from_utf8_lossy(&name_out.stdout);
            for line in name_stdout.lines() {
                if let Some(q_start) = line.find('"') {
                    if let Some(q_end) = line[q_start + 1..].rfind('"') {
                        let title = &line[q_start + 1..q_start + 1 + q_end];
                        if !title.is_empty() {
                            return Some(title.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn get_active_window_title() -> Option<String> {
    if let Some(title) = get_kwin_active_window() {
        return Some(title);
    }

    let mut hypr_active = build_user_command("hyprctl", &["activewindow"]);
    if let Ok(output) = hypr_active.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if let Some(title) = trimmed.strip_prefix("title: ") {
                    let clean = title.trim();
                    if !clean.is_empty() {
                        return Some(clean.to_string());
                    }
                }
            }
        }
    }

    let mut sway_active = build_user_command("swaymsg", &["-t", "get_tree"]);
    if let Ok(output) = sway_active.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(title) = get_sway_focused_title(&stdout) {
                return Some(title);
            }
        }
    }

    let mut cmd = build_user_command("xdotool", &["getactivewindow", "getwindowname"]);
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    let mut xprop_root = build_user_command("xprop", &["-root", "_NET_ACTIVE_WINDOW"]);
    if let Ok(output) = xprop_root.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(pos) = stdout.find('#') {
                let win_id = stdout[pos + 1..].trim();
                if !win_id.is_empty() && win_id != "0x0" {
                    if let Some(title) = get_window_title_by_id(win_id) {
                        return Some(title);
                    }
                }
            }
        }
    }

    let is_wayland = env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var("XDG_SESSION_TYPE").map(|v| v.to_lowercase() == "wayland").unwrap_or(false);

    if !is_wayland {
        let mut xprop_stack = build_user_command("xprop", &["-root", "_NET_CLIENT_LIST_STACKING"]);
        if let Ok(output) = xprop_stack.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(pos) = stdout.find('#') {
                    let ids: Vec<&str> = stdout[pos + 1..]
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty() && *s != "0x0")
                        .collect();
                    for win_id in ids.into_iter().rev() {
                        if let Some(title) = get_window_title_by_id(win_id) {
                            return Some(title);
                        }
                    }
                }
            }
        }
    }

    None
}

fn get_kwin_active_window() -> Option<String> {
    let py_code = r#"
import dbus, dbus.service, dbus.mainloop.glib, tempfile, os, sys
from gi.repository import GLib

try:
    dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
    bus = dbus.SessionBus()

    class Receiver(dbus.service.Object):
        def __init__(self):
            super().__init__(bus, '/Receiver')
            self.result = ''
        @dbus.service.method('org.ogiumacro.Receiver', in_signature='ss', out_signature='')
        def report(self, caption, resource_class):
            self.result = caption or resource_class
            loop.quit()

    bus_name = dbus.service.BusName('org.ogiumacro.WindowQuery', bus)
    receiver = Receiver()
    loop = GLib.MainLoop()

    js = "var w = workspace.activeWindow; callDBus('org.ogiumacro.WindowQuery', '/Receiver', 'org.ogiumacro.Receiver', 'report', w ? (w.caption || '') : '', w ? (w.resourceClass || '') : '');"

    with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
        f.write(js)
        script_path = f.name

    kwin_scripting = bus.get_object('org.kde.KWin', '/Scripting')
    script_id = kwin_scripting.loadScript(script_path, dbus_interface='org.kde.kwin.Scripting')
    script_obj = bus.get_object('org.kde.KWin', f'/Scripting/Script{script_id}')
    script_obj.run(dbus_interface='org.kde.kwin.Script')
    script_obj.stop(dbus_interface='org.kde.kwin.Script')
    try:
        kwin_scripting.unloadScript(script_path, dbus_interface='org.kde.kwin.Scripting')
    except Exception:
        pass
    if os.path.exists(script_path):
        os.remove(script_path)

    GLib.timeout_add(800, loop.quit)
    loop.run()
    if receiver.result:
        print(receiver.result.strip())
        sys.exit(0)
except Exception:
    pass
sys.exit(1)
"#;

    let mut py_cmd = build_user_command("python3", &["-c", py_code]);
    if let Ok(output) = py_cmd.output() {
        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !out.is_empty() {
                return Some(out);
            }
        }
    }
    None
}

fn get_sway_focused_title(json_str: &str) -> Option<String> {
    let mut last_name: Option<String> = None;
    for line in json_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"name\":") {
            if let Some(start) = trimmed.find('"') {
                if let Some(after) = trimmed.get(start + 7..) {
                    if let Some(q_start) = after.find('"') {
                        if let Some(q_end) = after[q_start + 1..].rfind('"') {
                            let title = &after[q_start + 1..q_start + 1 + q_end];
                            if !title.is_empty() && !title.starts_with("__i3") {
                                last_name = Some(title.to_string());
                            }
                        }
                    }
                }
            }
        }
        if trimmed == "\"focused\": true," || trimmed == "\"focused\": true" {
            if let Some(name) = last_name.take() {
                return Some(name);
            }
        }
    }
    None
}

pub fn is_window_focused(title: &str) -> bool {
    if title.is_empty() || title == "Active Window" {
        return true;
    }

    let clean_target = title.strip_prefix("[App] ").unwrap_or(title).to_lowercase();
    if let Some(active) = get_active_window_title() {
        let active_lower = active.to_lowercase();
        if active_lower.contains(&clean_target) || clean_target.contains(&active_lower) {
            return true;
        }

        let target_tokens: Vec<&str> = clean_target
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() >= 3)
            .collect();
        if !target_tokens.is_empty() && target_tokens.iter().any(|&tok| active_lower.contains(tok)) {
            return true;
        }
        false
    } else {
        true
    }
}

pub fn activate_window(title: &str) {
    if title.is_empty() || title == "Active Window" {
        return;
    }

    let clean_title = title.strip_prefix("[App] ").unwrap_or(title);

    let kwin_script = format!(
        r#"
(function() {{
    let target = {target:?}.toLowerCase();
    let targetClean = target.replace(/[^a-z0-9]+/g, ' ').trim();
    let tokens = targetClean.split(/\s+/).filter(function(s) {{ return s.length >= 3; }});
    let list = workspace.windowList();

    let bestClient = null;
    let bestScore = 0;

    for (let i = 0; i < list.length; i++) {{
        let c = list[i];
        if (c.normalWindow === false || !c.caption) {{
            continue;
        }}
        let cap = (c.caption || "").toLowerCase();
        let res = (c.resourceClass || "").toLowerCase();
        let name = (c.resourceName || "").toLowerCase();
        let score = 0;

        if (cap === target) {{
            score += 1000;
        }} else if (cap.includes(target)) {{
            score += 500;
        }} else if (target.includes(cap) && cap.length >= 4) {{
            score += 300;
        }}

        if (res.length >= 3 && target.includes(res)) {{
            score += 200;
        }}
        if (name.length >= 3 && target.includes(name)) {{
            score += 200;
        }}

        for (let t of tokens) {{
            if (cap.includes(t)) {{
                score += 10;
            }}
            if (res.includes(t)) {{
                score += 15;
            }}
        }}

        if (score > bestScore) {{
            bestScore = score;
            bestClient = c;
        }}
    }}

    if (bestClient && bestScore > 0) {{
        bestClient.minimized = false;
        workspace.activeWindow = bestClient;
    }}
}})();
"#,
        target = clean_title
    );
    if execute_kwin_script(&kwin_script) {
        return;
    }

    let is_kde = env::var("XDG_CURRENT_DESKTOP").map(|v| v.to_uppercase().contains("KDE")).unwrap_or(false);
    let is_wayland = env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var("XDG_SESSION_TYPE").map(|v| v.to_lowercase() == "wayland").unwrap_or(false);

    if !is_kde {
        let mut hypr_cmd = build_user_command("hyprctl", &["dispatch", "focuswindow", &format!("title:{}", clean_title)]);
        let _ = hypr_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

        let mut sway_cmd = build_user_command("swaymsg", &[&format!("[title=\"(?i).*{}.*\"] focus", clean_title)]);
        let _ = sway_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    }

    if !is_wayland {
        let mut unhide = build_user_command("wmctrl", &["-r", clean_title, "-b", "remove,hidden"]);
        let _ = unhide.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

        let mut activate = build_user_command("wmctrl", &["-a", clean_title]);
        let status = activate.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

        if status.is_err() || !status.as_ref().map(|s| s.success()).unwrap_or(false) {
            let mut xdo = build_user_command("xdotool", &["search", "--name", clean_title, "windowactivate"]);
            let _ = xdo.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
        }
    }
}

pub fn minimize_window(title: &str) {
    if title.is_empty() || title == "Active Window" {
        return;
    }

    let clean_title = title.strip_prefix("[App] ").unwrap_or(title);

    let kwin_script = format!(
        r#"
(function() {{
    let target = {target:?}.toLowerCase();
    let targetClean = target.replace(/[^a-z0-9]+/g, ' ').trim();
    let tokens = targetClean.split(/\s+/).filter(function(s) {{ return s.length >= 3; }});
    let list = workspace.windowList();

    let bestClient = null;
    let bestScore = 0;

    for (let i = 0; i < list.length; i++) {{
        let c = list[i];
        if (c.normalWindow === false || !c.caption) {{
            continue;
        }}
        let cap = (c.caption || "").toLowerCase();
        let res = (c.resourceClass || "").toLowerCase();
        let name = (c.resourceName || "").toLowerCase();
        let score = 0;

        if (cap === target) {{
            score += 1000;
        }} else if (cap.includes(target)) {{
            score += 500;
        }} else if (target.includes(cap) && cap.length >= 4) {{
            score += 300;
        }}

        if (res.length >= 3 && target.includes(res)) {{
            score += 200;
        }}
        if (name.length >= 3 && target.includes(name)) {{
            score += 200;
        }}

        for (let t of tokens) {{
            if (cap.includes(t)) {{
                score += 10;
            }}
            if (res.includes(t)) {{
                score += 15;
            }}
        }}

        if (score > bestScore) {{
            bestScore = score;
            bestClient = c;
        }}
    }}

    if (bestClient && bestScore > 0) {{
        bestClient.minimized = true;
    }}
}})();
"#,
        target = clean_title
    );
    if execute_kwin_script(&kwin_script) {
        return;
    }

    let is_kde = env::var("XDG_CURRENT_DESKTOP").map(|v| v.to_uppercase().contains("KDE")).unwrap_or(false);
    let is_wayland = env::var_os("WAYLAND_DISPLAY").is_some()
        || env::var("XDG_SESSION_TYPE").map(|v| v.to_lowercase() == "wayland").unwrap_or(false);

    if !is_kde {
        let mut hypr_cmd = build_user_command("hyprctl", &["dispatch", "movetoworkspacesilent", "special:ogiumin"]);
        let _ = hypr_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

        let mut sway_cmd = build_user_command("swaymsg", &[&format!("[title=\"(?i).*{}.*\"] move scratchpad", clean_title)]);
        let _ = sway_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    }

    if !is_wayland {
        let mut hide = build_user_command("wmctrl", &["-r", clean_title, "-b", "add,hidden"]);
        let status = hide.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

        if status.is_err() || !status.as_ref().map(|s| s.success()).unwrap_or(false) {
            let mut xdo = build_user_command("xdotool", &["search", "--name", clean_title, "windowminimize"]);
            let _ = xdo.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
        }
    }
}

fn execute_kwin_script(script: &str) -> bool {
    let py_code = format!(
        r#"
import dbus, tempfile, os, sys

js = {script:?}
try:
    bus = dbus.SessionBus()
    with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
        f.write(js)
        temp_path = f.name

    kwin = bus.get_object('org.kde.KWin', '/Scripting')
    script_id = kwin.loadScript(temp_path, dbus_interface='org.kde.kwin.Scripting')
    script_obj = bus.get_object('org.kde.KWin', f'/Scripting/Script{{script_id}}')
    script_obj.run(dbus_interface='org.kde.kwin.Script')
    script_obj.stop(dbus_interface='org.kde.kwin.Script')
    try:
        kwin.unloadScript(temp_path, dbus_interface='org.kde.kwin.Scripting')
    except Exception:
        pass
    if os.path.exists(temp_path):
        os.remove(temp_path)
    sys.exit(0)
except Exception:
    sys.exit(1)
"#,
        script = script
    );

    let mut py_cmd = build_user_command("python3", &["-c", &py_code]);
    py_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    if let Ok(status) = py_cmd.status() {
        if status.success() {
            return true;
        }
    }

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let temp_path = format!("/tmp/ogiumacro_kwin_{}_{}.js", pid, nanos);

    if fs::write(&temp_path, script).is_err() {
        return false;
    }

    let mut load_cmd = build_user_command("qdbus", &["org.kde.KWin", "/Scripting", "loadScript", &temp_path]);
    load_cmd.stderr(std::process::Stdio::null());
    let success = if let Ok(output) = load_cmd.output() {
        if output.status.success() {
            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
                let script_path = format!("/Scripting/Script{}", id);
                let mut run_cmd = build_user_command("qdbus", &["org.kde.KWin", &script_path, "run"]);
                run_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
                let _ = run_cmd.status();

                let mut stop_cmd = build_user_command("qdbus", &["org.kde.KWin", &script_path, "stop"]);
                stop_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
                let _ = stop_cmd.status();

                let mut unload_cmd = build_user_command("qdbus", &["org.kde.KWin", "/Scripting", "unloadScript", &temp_path]);
                unload_cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
                let _ = unload_cmd.status();
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    let _ = fs::remove_file(&temp_path);
    success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_window_focused_active_or_empty() {
        assert!(is_window_focused(""));
        assert!(is_window_focused("Active Window"));
    }

    #[test]
    fn test_clean_title_extraction() {
        let app_title = "[App] firefox";
        let clean = app_title.strip_prefix("[App] ").unwrap_or(app_title);
        assert_eq!(clean, "firefox");

        let normal_title = "Document - LibreOffice Writer";
        let clean_normal = normal_title.strip_prefix("[App] ").unwrap_or(normal_title);
        assert_eq!(clean_normal, "Document - LibreOffice Writer");
    }
}
