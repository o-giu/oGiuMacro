use std::fs;
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::time::UNIX_EPOCH;

use evdev::Device;

use crate::core::RecordedInputEvent;

pub struct InputRecorder {
    is_recording: Arc<AtomicBool>,
}

impl InputRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.is_recording.store(false, Ordering::SeqCst);
    }

    pub fn start(&self, tx: Sender<RecordedInputEvent>) {
        if self.is_active() {
            return;
        }

        self.is_recording.store(true, Ordering::SeqCst);
        let flag = Arc::clone(&self.is_recording);

        thread::spawn(move || {
            let devices = find_input_devices();
            let mut active_devices = Vec::new();
            let mut poll_fds = Vec::new();

            for path in devices {
                if let Ok(dev) = Device::open(&path) {
                    let _ = dev.set_nonblocking(true);
                    let supported = dev.supported_events();
                    if supported.contains(evdev::EventType::KEY) || supported.contains(evdev::EventType::RELATIVE) {
                        let name = dev.name().unwrap_or("").to_lowercase();
                        if !name.contains("imu") && !name.contains("sensor") && !name.contains("accelerometer") && !name.contains("ogiumacro") {
                            poll_fds.push(libc::pollfd {
                                fd: dev.as_raw_fd(),
                                events: libc::POLLIN,
                                revents: 0,
                            });
                            active_devices.push(dev);
                        }
                    }
                }
            }

            if active_devices.is_empty() {
                flag.store(false, Ordering::SeqCst);
                return;
            }

            let mut first_hardware_ts: Option<u64> = None;

            while flag.load(Ordering::SeqCst) {
                let poll_ret = unsafe {
                    libc::poll(
                        poll_fds.as_mut_ptr(),
                        poll_fds.len() as libc::nfds_t,
                        5,
                    )
                };

                if poll_ret <= 0 {
                    continue;
                }

                for (idx, dev) in active_devices.iter_mut().enumerate() {
                    if poll_fds[idx].revents & libc::POLLIN == 0 {
                        continue;
                    }

                    if let Ok(events) = dev.fetch_events() {
                        for ev in events {
                            let ev_type = ev.event_type();
                            if ev_type == evdev::EventType::KEY
                                || ev_type == evdev::EventType::RELATIVE
                                || ev_type == evdev::EventType::SYNCHRONIZATION
                            {
                                let ev_micros = match ev.timestamp().duration_since(UNIX_EPOCH) {
                                    Ok(d) => d.as_micros() as u64,
                                    Err(_) => 0,
                                };

                                let base_ts = *first_hardware_ts.get_or_insert(ev_micros);
                                let timestamp_ms = if ev_micros >= base_ts {
                                    (ev_micros - base_ts) / 1000
                                } else {
                                    0
                                };

                                let recorded = RecordedInputEvent {
                                    timestamp_ms,
                                    event_type: ev_type.0,
                                    code: ev.code(),
                                    value: ev.value(),
                                };
                                let _ = tx.send(recorded);
                            }
                        }
                    }
                }
            }
        });
    }
}

pub struct GlobalHotkeyListener {
    running: Arc<AtomicBool>,
}

impl GlobalHotkeyListener {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn start(&self, tx: Sender<u32>) {
        if self.is_active() {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let flag = Arc::clone(&self.running);

        thread::spawn(move || {
            let devices = find_input_devices();
            let mut key_devices = Vec::new();

            for path in devices {
                if let Ok(dev) = Device::open(&path) {
                    let _ = dev.set_nonblocking(true);
                    let supported = dev.supported_events();
                    if supported.contains(evdev::EventType::KEY) {
                        let name = dev.name().unwrap_or("").to_lowercase();
                        if !name.contains("imu") && !name.contains("sensor") && !name.contains("accelerometer") {
                            key_devices.push(dev);
                        }
                    }
                }
            }

            let mut esc_press_times: Vec<std::time::Instant> = Vec::new();

            while flag.load(Ordering::SeqCst) {
                let mut captured_any = false;

                for dev in &mut key_devices {
                    if let Ok(events) = dev.fetch_events() {
                        for ev in events {
                            if ev.event_type() == evdev::EventType::KEY && ev.value() == 1 {
                                if ev.code() == 1 {
                                    let now = std::time::Instant::now();
                                    esc_press_times.retain(|&t| now.duration_since(t) <= std::time::Duration::from_millis(1200));
                                    esc_press_times.push(now);

                                    if esc_press_times.len() >= 3 {
                                        esc_press_times.clear();
                                        captured_any = true;
                                        let _ = tx.send(999);
                                    }
                                }

                                if let Some(fkey) = crate::core::evdev_to_fkey(ev.code()) {
                                    captured_any = true;
                                    let _ = tx.send(fkey);
                                }
                            }
                        }
                    }
                }

                if !captured_any {
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        });
    }
}

fn find_input_devices() -> Vec<String> {
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("event") {
                    result.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    result
}
