# <p align="center">oGiuMacro - Automation Tool for Linux<p>
<p align="center">
  <a href="https://github.com/o-giu/oGiuMacro/releases/" target="_blank" rel="noopener noreferrer">
    <img src="https://img.shields.io/badge/Download_AppImage-Releases-blue?style=for-the-badge"/>
  </a><br>
  <sup>Download, grant permission (<code>chmod +x</code>), and run!</sup>
</p>

---

<sup>*I built this out of personal necessity: existing Linux tools were either overly complicated to configure, broken on modern desktop environments like Wayland, or simply lacked the core features I needed in my daily workflow.*</sup>

---

A mouse and keyboard macro automation tool for Linux featuring image recognition, built natively in Rust with support for both Wayland (KDE Plasma, GNOME, Hyprland, Sway) and X11.
<br><sup>*Fedora 44 KDE Host System * Universal Build & AppImage (Debian 12 base via Podman)*</sup>

---

Automatically detects your distribution and missing optional packages on startup:<sup>
- **Debian**:
  ```bash
  sudo apt install wmctrl xdotool scrot slurp zenity
  ```
- **Fedora**:
  ```bash
  sudo dnf install wmctrl xdotool scrot slurp zenity
  ```
- **Arch**:
  ```bash
  sudo pacman -S wmctrl xdotool scrot slurp zenity
  ```
- **openSUSE**:
  ```bash
  sudo zypper install wmctrl xdotool scrot slurp zenity
  ```</sup>
---

## License

<sup>This project is licensed under the MIT License. Developed with care by **oGiu**.</sup>
