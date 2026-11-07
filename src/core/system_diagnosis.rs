use std::env;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinuxDistro {
    Fedora,
    Debian,
    Ubuntu,
    Arch,
    OpenSUSE,
    Generic(String),
}

impl LinuxDistro {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Fedora => "Fedora / RHEL",
            Self::Debian => "Debian",
            Self::Ubuntu => "Ubuntu / Mint / Pop!_OS",
            Self::Arch => "Arch Linux / Manjaro",
            Self::OpenSUSE => "openSUSE",
            Self::Generic(name) => name.as_str(),
        }
    }

    pub fn package_manager_name(&self) -> &str {
        match self {
            Self::Fedora => "dnf",
            Self::Debian | Self::Ubuntu => "apt",
            Self::Arch => "pacman",
            Self::OpenSUSE => "zypper",
            Self::Generic(_) => "package manager",
        }
    }

    pub fn install_command_for_missing(&self, missing: &[&str]) -> Option<String> {
        if missing.is_empty() {
            return None;
        }

        match self {
            Self::Fedora => {
                let pkgs: Vec<&str> = missing.iter().map(|&tool| match tool {
                    "wmctrl" => "wmctrl",
                    "xdotool" => "xdotool",
                    "slurp" => "slurp",
                    "scrot" => "scrot",
                    "zenity" => "zenity",
                    other => other,
                }).collect();
                Some(format!("sudo dnf install {}", pkgs.join(" ")))
            }
            Self::Debian | Self::Ubuntu => {
                let pkgs: Vec<&str> = missing.iter().map(|&tool| match tool {
                    "wmctrl" => "wmctrl",
                    "xdotool" => "xdotool",
                    "slurp" => "slurp",
                    "scrot" => "scrot",
                    "zenity" => "zenity",
                    other => other,
                }).collect();
                Some(format!("sudo apt install {}", pkgs.join(" ")))
            }
            Self::Arch => {
                let pkgs: Vec<&str> = missing.iter().map(|&tool| match tool {
                    "wmctrl" => "wmctrl",
                    "xdotool" => "xdotool",
                    "slurp" => "slurp",
                    "scrot" => "scrot",
                    "zenity" => "zenity",
                    other => other,
                }).collect();
                Some(format!("sudo pacman -S {}", pkgs.join(" ")))
            }
            Self::OpenSUSE => {
                let pkgs: Vec<&str> = missing.iter().map(|&tool| match tool {
                    "wmctrl" => "wmctrl",
                    "xdotool" => "xdotool",
                    "slurp" => "slurp",
                    "scrot" => "scrot",
                    "zenity" => "zenity",
                    other => other,
                }).collect();
                Some(format!("sudo zypper install {}", pkgs.join(" ")))
            }
            Self::Generic(_) => {
                Some(format!("Install using your package manager: {}", missing.join(", ")))
            }
        }
    }
}

pub fn which_command(cmd: &str) -> bool {
    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let full_path = dir.join(cmd);
            if full_path.is_file() {
                return true;
            }
        }
    }
    Path::new(&format!("/usr/bin/{}", cmd)).exists()
        || Path::new(&format!("/bin/{}", cmd)).exists()
        || Path::new(&format!("/usr/local/bin/{}", cmd)).exists()
}

pub fn parse_os_release(content: &str) -> LinuxDistro {
    let mut id = String::new();
    let mut id_like = String::new();
    let mut pretty_name = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("ID=") {
            id = val.trim_matches('"').to_lowercase();
        } else if let Some(val) = trimmed.strip_prefix("ID_LIKE=") {
            id_like = val.trim_matches('"').to_lowercase();
        } else if let Some(val) = trimmed.strip_prefix("PRETTY_NAME=") {
            pretty_name = val.trim_matches('"').to_string();
        }
    }

    if id == "fedora" || id_like.contains("fedora") || id == "rhel" || id == "centos" {
        return LinuxDistro::Fedora;
    }
    if id == "ubuntu" || id == "linuxmint" || id == "pop" || id_like.contains("ubuntu") {
        return LinuxDistro::Ubuntu;
    }
    if id == "debian" || id_like.contains("debian") {
        return LinuxDistro::Debian;
    }
    if id == "arch" || id == "manjaro" || id_like.contains("arch") {
        return LinuxDistro::Arch;
    }
    if id.contains("suse") || id_like.contains("suse") {
        return LinuxDistro::OpenSUSE;
    }

    if !pretty_name.is_empty() {
        return LinuxDistro::Generic(pretty_name);
    }

    LinuxDistro::Generic("Linux (Unknown)".to_string())
}

pub fn detect_linux_distro() -> LinuxDistro {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        parse_os_release(&content)
    } else {
        LinuxDistro::Generic("Linux (Unknown)".to_string())
    }
}

#[derive(Clone, Debug)]
pub struct DependencyStatus {
    pub name: &'static str,
    pub command: &'static str,
    pub description: &'static str,
    pub installed: bool,
    pub required_for: &'static str,
}

#[derive(Clone, Debug)]
pub struct SystemDiagnosis {
    pub distro: LinuxDistro,
    pub session_type: &'static str,
    pub has_uinput: bool,
    pub has_passwordless_access: bool,
    pub dependencies: Vec<DependencyStatus>,
}

impl SystemDiagnosis {
    pub fn collect() -> Self {
        let distro = detect_linux_distro();
        let is_wayland = env::var("WAYLAND_DISPLAY").map(|v| !v.is_empty()).unwrap_or(false);
        let session_type = if is_wayland { "Wayland" } else { "X11" };

        let has_uinput = Path::new("/dev/uinput").exists();
        let has_passwordless_access = crate::core::privileges::has_passwordless_access();

        let mut dependencies = vec![
            DependencyStatus {
                name: "Window Controller",
                command: "wmctrl",
                description: "Window listing, focus, and target detection",
                installed: which_command("wmctrl"),
                required_for: "Window Target & Focus",
            },
            DependencyStatus {
                name: "X11 Automation",
                command: "xdotool",
                description: "Window minimization for Stealth Mode",
                installed: which_command("xdotool"),
                required_for: "Stealth Mode Minimize",
            },
        ];

        if is_wayland {
            dependencies.push(DependencyStatus {
                name: "Wayland Crosshair",
                command: "slurp",
                description: "Full-screen pixel coordinate picker overlay",
                installed: which_command("slurp"),
                required_for: "Pick Position on Wayland",
            });
        }

        dependencies.push(DependencyStatus {
            name: "X11 Screen Capture",
            command: "scrot",
            description: "High-speed screen capturing on X11 sessions",
            installed: which_command("scrot"),
            required_for: "Image Recognition on X11",
        });

        dependencies.push(DependencyStatus {
            name: "File Dialog",
            command: "zenity",
            description: "Root-safe graphical image file selection dialog",
            installed: which_command("zenity"),
            required_for: "Manage Images File Picker",
        });

        Self {
            distro,
            session_type,
            has_uinput,
            has_passwordless_access,
            dependencies,
        }
    }

    pub fn missing_commands(&self) -> Vec<&'static str> {
        self.dependencies
            .iter()
            .filter(|d| !d.installed)
            .map(|d| d.command)
            .collect()
    }

    pub fn install_command(&self) -> Option<String> {
        let missing = self.missing_commands();
        self.distro.install_command_for_missing(&missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_distro_display_and_pm() {
        assert_eq!(LinuxDistro::Fedora.display_name(), "Fedora / RHEL");
        assert_eq!(LinuxDistro::Fedora.package_manager_name(), "dnf");

        assert_eq!(LinuxDistro::Debian.display_name(), "Debian");
        assert_eq!(LinuxDistro::Debian.package_manager_name(), "apt");

        assert_eq!(LinuxDistro::Ubuntu.display_name(), "Ubuntu / Mint / Pop!_OS");
        assert_eq!(LinuxDistro::Ubuntu.package_manager_name(), "apt");

        assert_eq!(LinuxDistro::Arch.display_name(), "Arch Linux / Manjaro");
        assert_eq!(LinuxDistro::Arch.package_manager_name(), "pacman");

        assert_eq!(LinuxDistro::OpenSUSE.display_name(), "openSUSE");
        assert_eq!(LinuxDistro::OpenSUSE.package_manager_name(), "zypper");

        let generic = LinuxDistro::Generic("Void Linux".to_string());
        assert_eq!(generic.display_name(), "Void Linux");
        assert_eq!(generic.package_manager_name(), "package manager");
    }

    #[test]
    fn test_install_command_generation() {
        let tools = vec!["wmctrl", "slurp"];
        let fedora_cmd = LinuxDistro::Fedora.install_command_for_missing(&tools).unwrap();
        assert_eq!(fedora_cmd, "sudo dnf install wmctrl slurp");

        let debian_cmd = LinuxDistro::Debian.install_command_for_missing(&tools).unwrap();
        assert_eq!(debian_cmd, "sudo apt install wmctrl slurp");

        let arch_cmd = LinuxDistro::Arch.install_command_for_missing(&tools).unwrap();
        assert_eq!(arch_cmd, "sudo pacman -S wmctrl slurp");

        let suse_cmd = LinuxDistro::OpenSUSE.install_command_for_missing(&tools).unwrap();
        assert_eq!(suse_cmd, "sudo zypper install wmctrl slurp");

        let generic_cmd = LinuxDistro::Generic("Other".to_string()).install_command_for_missing(&tools).unwrap();
        assert!(generic_cmd.contains("wmctrl, slurp"));

        assert_eq!(LinuxDistro::Fedora.install_command_for_missing(&[]), None);
    }

    #[test]
    fn test_parse_os_release() {
        let fedora_sample = r#"
NAME="Fedora Linux"
VERSION="41 (Workstation Edition)"
ID=fedora
VERSION_ID=41
PRETTY_NAME="Fedora Linux 41 (Workstation Edition)"
"#;
        assert_eq!(parse_os_release(fedora_sample), LinuxDistro::Fedora);

        let ubuntu_sample = r#"
NAME="Ubuntu"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 24.04 LTS"
"#;
        assert_eq!(parse_os_release(ubuntu_sample), LinuxDistro::Ubuntu);

        let debian_sample = r#"
NAME="Debian GNU/Linux"
ID=debian
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
"#;
        assert_eq!(parse_os_release(debian_sample), LinuxDistro::Debian);

        let arch_sample = r#"
NAME="Arch Linux"
ID=arch
PRETTY_NAME="Arch Linux"
"#;
        assert_eq!(parse_os_release(arch_sample), LinuxDistro::Arch);

        let custom_sample = r#"
NAME="Gentoo"
ID=gentoo
PRETTY_NAME="Gentoo Linux"
"#;
        assert_eq!(parse_os_release(custom_sample), LinuxDistro::Generic("Gentoo Linux".to_string()));
    }

    #[test]
    fn test_system_diagnosis_collect_smoke() {
        let diagnosis = SystemDiagnosis::collect();
        assert!(!diagnosis.dependencies.is_empty());
        let _ = diagnosis.install_command();
    }
}

