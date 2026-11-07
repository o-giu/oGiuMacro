use std::cell::RefCell;
use std::env;
use std::ffi::CStr;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use image::GrayImage;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

static GLOBAL_SCREEN_FRAME: OnceLock<Arc<Mutex<Option<GrayImage>>>> = OnceLock::new();
static CAPTURE_RUNNING: OnceLock<Arc<AtomicBool>> = OnceLock::new();
static PORTAL_CHILD: OnceLock<Arc<Mutex<Option<std::process::Child>>>> = OnceLock::new();

thread_local! {
    static SAT_WORKSPACE: RefCell<(Vec<f64>, Vec<f64>)> = const { RefCell::new((Vec::new(), Vec::new())) };
}

fn get_global_screen() -> Arc<Mutex<Option<GrayImage>>> {
    GLOBAL_SCREEN_FRAME.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

fn get_capture_running_flag() -> Arc<AtomicBool> {
    CAPTURE_RUNNING.get_or_init(|| Arc::new(AtomicBool::new(false))).clone()
}

fn get_portal_child() -> Arc<Mutex<Option<std::process::Child>>> {
    PORTAL_CHILD.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

fn read_portal_shm_frame() -> Option<GrayImage> {
    let (_, uid) = get_real_user_and_uid();
    let shm_path = format!("/dev/shm/ogiumacro_screen_{}.raw", uid);
    if let Ok(mut f) = fs::File::open(&shm_path) {
        use std::io::Read;
        let mut header = [0u8; 8];
        if f.read_exact(&mut header).is_ok() {
            let width = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
            let height = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
            if width > 0 && height > 0 && width <= 7680 && height <= 4320 {
                let total_px = (width as usize) * (height as usize);
                let mut data = vec![0u8; total_px];
                if f.read_exact(&mut data).is_ok() {
                    return GrayImage::from_raw(width, height, data);
                }
            }
        }
    }
    None
}

fn which_command(cmd: &str) -> bool {
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

#[inline]
#[allow(dead_code)]
pub fn start_screen_capture_worker() {
    start_screen_capture_worker_with_engine(crate::core::config::CaptureEngine::Auto);
}

pub fn is_screen_capture_worker_running() -> bool {
    let flag = get_capture_running_flag();
    flag.load(Ordering::SeqCst)
}

pub fn stop_screen_capture_worker() {
    let flag = get_capture_running_flag();
    flag.store(false, Ordering::SeqCst);
    if let Ok(mut lock) = get_portal_child().lock() {
        if let Some(mut child) = lock.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    let (_, uid) = get_real_user_and_uid();
    let shm_path = format!("/dev/shm/ogiumacro_screen_{}.raw", uid);
    let _ = fs::remove_file(&shm_path);

    if let Ok(mut lock) = get_global_screen().lock() {
        *lock = None;
    }
}

use crate::core::config::CaptureEngine;

#[repr(C)]
struct XShmSegmentInfo {
    shmseg: libc::c_ulong,
    shmid: libc::c_int,
    shmaddr: *mut libc::c_char,
    read_only: libc::c_int,
}

#[repr(C)]
struct XImage {
    width: libc::c_int,
    height: libc::c_int,
    xoffset: libc::c_int,
    format: libc::c_int,
    data: *mut libc::c_char,
    byte_order: libc::c_int,
    bitmap_unit: libc::c_int,
    bitmap_bit_order: libc::c_int,
    bitmap_pad: libc::c_int,
    depth: libc::c_int,
    bytes_per_line: libc::c_int,
    bits_per_pixel: libc::c_int,
    red_mask: libc::c_ulong,
    green_mask: libc::c_ulong,
    blue_mask: libc::c_ulong,
}

unsafe extern "C" {
    fn XOpenDisplay(name: *const libc::c_char) -> *mut libc::c_void;
    fn XCloseDisplay(display: *mut libc::c_void) -> libc::c_int;
    fn XDefaultScreen(display: *mut libc::c_void) -> libc::c_int;
    fn XRootWindow(display: *mut libc::c_void, screen: libc::c_int) -> libc::c_ulong;
    fn XDisplayWidth(display: *mut libc::c_void, screen: libc::c_int) -> libc::c_int;
    fn XDisplayHeight(display: *mut libc::c_void, screen: libc::c_int) -> libc::c_int;
    fn XDefaultVisual(display: *mut libc::c_void, screen: libc::c_int) -> *mut libc::c_void;
    fn XDefaultDepth(display: *mut libc::c_void, screen: libc::c_int) -> libc::c_int;
    fn XFree(data: *mut libc::c_void) -> libc::c_int;

    fn XShmCreateImage(
        display: *mut libc::c_void,
        visual: *mut libc::c_void,
        depth: libc::c_uint,
        format: libc::c_int,
        data: *mut libc::c_char,
        shminfo: *mut XShmSegmentInfo,
        width: libc::c_uint,
        height: libc::c_uint,
    ) -> *mut XImage;

    fn XShmAttach(display: *mut libc::c_void, shminfo: *mut XShmSegmentInfo) -> libc::c_int;
    fn XShmDetach(display: *mut libc::c_void, shminfo: *mut XShmSegmentInfo) -> libc::c_int;
    fn XShmGetImage(
        display: *mut libc::c_void,
        drawable: libc::c_ulong,
        image: *mut XImage,
        x: libc::c_int,
        y: libc::c_int,
        plane_mask: libc::c_ulong,
    ) -> libc::c_int;
}

pub fn is_kde_xwayland_eavesdropping_enabled() -> bool {
    let output = Command::new("kreadconfig6")
        .args(["--file", "kwinrc", "--group", "Xwayland", "--key", "XwaylandEavesdrops"])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
        text == "all"
    } else {
        false
    }
}

pub fn enable_kde_xwayland_eavesdropping() -> Result<(), String> {
    let output = Command::new("kwriteconfig6")
        .args(["--file", "kwinrc", "--group", "Xwayland", "--key", "XwaylandEavesdrops", "All"])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&out.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[repr(C)]
struct XWindowAttributes {
    x: libc::c_int,
    y: libc::c_int,
    width: libc::c_int,
    height: libc::c_int,
    border_width: libc::c_int,
    depth: libc::c_int,
    visual: *mut libc::c_void,
    root: libc::c_ulong,
    class: libc::c_int,
    bit_gravity: libc::c_int,
    win_gravity: libc::c_int,
    backing_store: libc::c_int,
    backing_planes: libc::c_ulong,
    backing_pixel: libc::c_ulong,
    save_under: libc::c_int,
    colormap: libc::c_ulong,
    map_installed: libc::c_int,
    map_state: libc::c_int,
    all_event_masks: libc::c_long,
    your_event_mask: libc::c_long,
    do_not_propagate_mask: libc::c_long,
    override_redirect: libc::c_int,
    screen: *mut libc::c_void,
}

unsafe extern "C" fn silent_x11_error(_dpy: *mut libc::c_void, _err: *mut libc::c_void) -> libc::c_int {
    0
}

unsafe extern "C" {
    fn XSetErrorHandler(
        handler: Option<unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> libc::c_int>,
    ) -> Option<unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> libc::c_int>;
    fn XQueryTree(
        display: *mut libc::c_void,
        w: libc::c_ulong,
        root_return: *mut libc::c_ulong,
        parent_return: *mut libc::c_ulong,
        children_return: *mut *mut libc::c_ulong,
        nchildren_return: *mut libc::c_uint,
    ) -> libc::c_int;
    fn XGetWindowAttributes(
        display: *mut libc::c_void,
        w: libc::c_ulong,
        window_attributes_return: *mut XWindowAttributes,
    ) -> libc::c_int;
    fn XFetchName(
        display: *mut libc::c_void,
        w: libc::c_ulong,
        window_name_return: *mut *mut libc::c_char,
    ) -> libc::c_int;
    fn XGetImage(
        display: *mut libc::c_void,
        d: libc::c_ulong,
        x: libc::c_int,
        y: libc::c_int,
        width: libc::c_uint,
        height: libc::c_uint,
        plane_mask: libc::c_ulong,
        format: libc::c_int,
    ) -> *mut XImage;
    fn XDestroyImage(image: *mut XImage) -> libc::c_int;
}

pub fn capture_x11_shm() -> Option<GrayImage> {
    unsafe {
        XSetErrorHandler(Some(silent_x11_error));
        let display_var = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
        let c_display = std::ffi::CString::new(display_var).ok()?;
        let dpy = XOpenDisplay(c_display.as_ptr());
        if dpy.is_null() {
            return None;
        }

        let screen = XDefaultScreen(dpy);
        let root = XRootWindow(dpy, screen);
        let width = XDisplayWidth(dpy, screen);
        let height = XDisplayHeight(dpy, screen);

        if width <= 0 || height <= 0 {
            XCloseDisplay(dpy);
            return None;
        }

        let mut shminfo: XShmSegmentInfo = std::mem::zeroed();
        let ximg = XShmCreateImage(
            dpy,
            XDefaultVisual(dpy, screen),
            XDefaultDepth(dpy, screen) as libc::c_uint,
            2, 
            std::ptr::null_mut(),
            &mut shminfo,
            width as libc::c_uint,
            height as libc::c_uint,
        );

        if !ximg.is_null() {
            let total_bytes = ((*ximg).bytes_per_line * (*ximg).height) as usize;
            let shmid = libc::shmget(libc::IPC_PRIVATE, total_bytes, libc::IPC_CREAT | 0o777);
            if shmid >= 0 {
                shminfo.shmid = shmid;
                let shmaddr = libc::shmat(shmid, std::ptr::null(), 0) as *mut libc::c_char;
                if shmaddr != (-1isize as *mut libc::c_char) {
                    shminfo.shmaddr = shmaddr;
                    (*ximg).data = shmaddr;
                    shminfo.read_only = 0;

                    if XShmAttach(dpy, &mut shminfo) != 0 {
                        let st = XShmGetImage(dpy, root, ximg, 0, 0, !0);
                        if st != 0 {
                            let data_slice = std::slice::from_raw_parts((*ximg).data as *const u8, total_bytes);
                            let has_content = data_slice.iter().take(20000).any(|&b| b != 0);
                            if has_content {
                                let bytes_per_line = (*ximg).bytes_per_line as usize;
                                let bpp = (*ximg).bits_per_pixel as usize / 8;
                                let w = width as u32;
                                let h = height as u32;

                                let mut gray = GrayImage::new(w, h);
                                let gray_buf = gray.as_mut();

                                for y in 0..h {
                                    let row_start = (y as usize) * bytes_per_line;
                                    let gray_row_start = (y as usize) * (w as usize);
                                    for x in 0..w {
                                        let px_start = row_start + (x as usize) * bpp;
                                        if px_start + 2 < total_bytes {
                                            let b = data_slice[px_start] as u32;
                                            let g = data_slice[px_start + 1] as u32;
                                            let r = data_slice[px_start + 2] as u32;
                                            let luma = ((299 * r + 587 * g + 114 * b) / 1000) as u8;
                                            gray_buf[gray_row_start + (x as usize)] = luma;
                                        }
                                    }
                                }

                                XShmDetach(dpy, &mut shminfo);
                                libc::shmdt(shmaddr as *const libc::c_void);
                                libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut());
                                XFree(ximg as *mut libc::c_void);
                                XCloseDisplay(dpy);
                                return Some(gray);
                            }
                        }
                        XShmDetach(dpy, &mut shminfo);
                    }
                    libc::shmdt(shmaddr as *const libc::c_void);
                }
                libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut());
            }
            XFree(ximg as *mut libc::c_void);
        }

        let mut root_ret = 0;
        let mut parent_ret = 0;
        let mut children: *mut libc::c_ulong = std::ptr::null_mut();
        let mut nchildren: libc::c_uint = 0;
        let q_status = XQueryTree(dpy, root, &mut root_ret, &mut parent_ret, &mut children, &mut nchildren);

        let mut composite_result = None;
        if q_status != 0 && !children.is_null() && nchildren > 0 {
            let child_slice = std::slice::from_raw_parts(children, nchildren as usize);
            let w = width as u32;
            let h = height as u32;
            let mut gray = GrayImage::new(w, h);
            let gray_buf = gray.as_mut();
            let mut captured_any = false;

            for &child in child_slice {
                let mut attr: XWindowAttributes = std::mem::zeroed();
                if XGetWindowAttributes(dpy, child, &mut attr) != 0 {
                    
                    if attr.map_state == 2
                        && attr.width > 20
                        && attr.height > 20
                        && attr.width <= width
                        && attr.height <= height
                    {
                        
                        if attr.width == width && attr.height == height && attr.x == 0 && attr.y == 0 {
                            let mut name_ptr: *mut libc::c_char = std::ptr::null_mut();
                            XFetchName(dpy, child, &mut name_ptr);
                            let has_name = if !name_ptr.is_null() {
                                let len = libc::strlen(name_ptr);
                                XFree(name_ptr as *mut libc::c_void);
                                len > 0
                            } else {
                                false
                            };
                            if !has_name {
                                continue;
                            }
                        }

                        let img = XGetImage(
                            dpy,
                            child,
                            0,
                            0,
                            attr.width as libc::c_uint,
                            attr.height as libc::c_uint,
                            !0,
                            2, 
                        );

                        if !img.is_null() {
                            if !(*img).data.is_null() {
                                let bpp = ((*img).bits_per_pixel / 8) as usize;
                                let bpl = (*img).bytes_per_line as usize;
                                let win_w = attr.width as usize;
                                let win_h = attr.height as usize;
                                let win_x = attr.x as isize;
                                let win_y = attr.y as isize;
                                let data_ptr = (*img).data as *const u8;

                                for dy in 0..win_h {
                                    let screen_y = win_y + dy as isize;
                                    if screen_y < 0 || screen_y >= (height as isize) {
                                        continue;
                                    }
                                    let src_row_start = dy * bpl;
                                    let dst_row_start = (screen_y as usize) * (width as usize);

                                    for dx in 0..win_w {
                                        let screen_x = win_x + dx as isize;
                                        if screen_x < 0 || screen_x >= (width as isize) {
                                            continue;
                                        }
                                        let px_idx = src_row_start + dx * bpp;
                                        let b = *data_ptr.add(px_idx) as u32;
                                        let g = *data_ptr.add(px_idx + 1) as u32;
                                        let r = *data_ptr.add(px_idx + 2) as u32;
                                        let luma = ((299 * r + 587 * g + 114 * b) / 1000) as u8;
                                        gray_buf[dst_row_start + (screen_x as usize)] = luma;
                                    }
                                }
                                captured_any = true;
                            }
                            XDestroyImage(img);
                        }
                    }
                }
            }

            if captured_any {
                composite_result = Some(gray);
            }
            XFree(children as *mut libc::c_void);
        }

        XCloseDisplay(dpy);
        composite_result
    }
}

const PORTAL_HELPER_SCRIPT: &str = r#"
import sys, os, random, signal, struct, mmap
try:
    import dbus
    from dbus.mainloop.glib import DBusGMainLoop
    import gi
    gi.require_version('Gst', '1.0')
    from gi.repository import GLib, Gst
except Exception as e:
    sys.exit(1)

DBusGMainLoop(set_as_default=True)
Gst.init(None)

try:
    bus = dbus.SessionBus()
    portal = bus.get_object('org.freedesktop.portal.Desktop', '/org/freedesktop/portal/desktop')
    screencast = dbus.Interface(portal, 'org.freedesktop.portal.ScreenCast')
except Exception as e:
    sys.exit(1)

main_loop = GLib.MainLoop()
token = f"ogiu_{random.randint(10000, 99999)}"
shm_path = f"/dev/shm/ogiumacro_screen_{os.getuid()}.raw"

pipeline = None
shm_fd = None
shm_map = None
current_size = 0

def cleanup(*args):
    global pipeline, shm_map, shm_fd
    if pipeline:
        try: pipeline.set_state(Gst.State.NULL)
        except: pass
    if shm_map:
        try: shm_map.close()
        except: pass
    if shm_fd:
        try: os.close(shm_fd)
        except: pass
    if os.path.exists(shm_path):
        try: os.remove(shm_path)
        except: pass
    sys.exit(0)

signal.signal(signal.SIGINT, cleanup)
signal.signal(signal.SIGTERM, cleanup)

def on_session_created(response, results):
    if response != 0: cleanup()
    session_handle = results['session_handle']
    select_token = f"sel_{random.randint(10000, 99999)}"
    sender_clean = bus.get_unique_name().replace('.', '_')[1:]
    select_path = f"/org/freedesktop/portal/desktop/request/{sender_clean}/{select_token}"
    bus.add_signal_receiver(lambda resp, res: on_sources_selected(resp, res, session_handle),
                            signal_name="Response",
                            path=select_path,
                            dbus_interface="org.freedesktop.portal.Request")
    # types: 1 = Screen (Monitor), 2 = Window, 3 = Both
    # For mouse targeting, picking the Monitor/Screen guarantees absolute pixel mapping
    options = {'handle_token': select_token, 'types': dbus.UInt32(3), 'multiple': dbus.Boolean(False)}
    screencast.SelectSources(session_handle, options)

def on_sources_selected(response, results, session_handle):
    if response != 0: cleanup()
    start_token = f"sta_{random.randint(10000, 99999)}"
    sender_clean = bus.get_unique_name().replace('.', '_')[1:]
    start_path = f"/org/freedesktop/portal/desktop/request/{sender_clean}/{start_token}"
    bus.add_signal_receiver(lambda resp, res: on_started(resp, res, session_handle),
                            signal_name="Response",
                            path=start_path,
                            dbus_interface="org.freedesktop.portal.Request")
    screencast.Start(session_handle, "", {'handle_token': start_token})

def on_started(response, results, session_handle):
    global pipeline
    if response != 0: cleanup()
    streams = results.get('streams', [])
    if not streams: cleanup()
    node_id = streams[0][0]
    fd = screencast.OpenPipeWireRemote(session_handle, {}).take()

    pipeline_str = f"pipewiresrc fd={fd} path={node_id} do-timestamp=true keepalive-time=1000 ! videoconvert ! video/x-raw,format=GRAY8 ! appsink name=sink emit-signals=true max-buffers=1 drop=true"
    pipeline = Gst.parse_launch(pipeline_str)
    sink = pipeline.get_by_name("sink")

    def on_sample(sink):
        global shm_fd, shm_map, current_size
        sample = sink.emit("pull-sample")
        if not sample: return Gst.FlowReturn.OK
        buf = sample.get_buffer()
        caps = sample.get_caps()
        s = caps.get_structure(0)
        w = s.get_value("width")
        h = s.get_value("height")
        success, map_info = buf.map(Gst.MapFlags.READ)
        if success:
            data = map_info.data
            needed_size = 8 + len(data)
            if shm_fd is None or needed_size != current_size:
                if shm_map:
                    try: shm_map.close()
                    except: pass
                if shm_fd:
                    try: os.close(shm_fd)
                    except: pass
                shm_fd = os.open(shm_path, os.O_RDWR | os.O_CREAT | os.O_TRUNC, 0o666)
                os.ftruncate(shm_fd, needed_size)
                shm_map = mmap.mmap(shm_fd, needed_size, mmap.MAP_SHARED, mmap.PROT_WRITE)
                current_size = needed_size
            header = struct.pack("<II", w, h)
            shm_map[0:8] = header
            shm_map[8:needed_size] = data
            buf.unmap(map_info)
        return Gst.FlowReturn.OK

    sink.connect("new-sample", on_sample)
    pipeline.set_state(Gst.State.PLAYING)
    print("READY", flush=True)

sender_clean = bus.get_unique_name().replace('.', '_')[1:]
request_path = f"/org/freedesktop/portal/desktop/request/{sender_clean}/{token}"
bus.add_signal_receiver(on_session_created,
                        signal_name="Response",
                        path=request_path,
                        dbus_interface="org.freedesktop.portal.Request")
screencast.CreateSession({'handle_token': token, 'session_handle_token': f"sess_{token}"})
try:
    main_loop.run()
except:
    cleanup()
"#;

pub fn start_screen_capture_worker_with_engine(engine: CaptureEngine) {
    let flag = get_capture_running_flag();
    if flag.swap(true, Ordering::SeqCst) {
        return; 
    }

    let is_wayland = env::var("WAYLAND_DISPLAY").map(|v| !v.is_empty()).unwrap_or(false);
    let use_portal = engine == CaptureEngine::Portal || (engine == CaptureEngine::Auto && is_wayland);

    if use_portal {
        
        let (real_user, real_uid) = get_real_user_and_uid();
        let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
        let xdg_dir = format!("/run/user/{}", real_uid);
        let dbus_addr = format!("unix:path={}/bus", xdg_dir);

        let mut cmd = if is_elevated() && !real_user.is_empty() {
            let mut c = Command::new("sudo");
            c.arg("-u")
                .arg(&real_user)
                .arg(format!("WAYLAND_DISPLAY={}", wayland_display))
                .arg(format!("XDG_RUNTIME_DIR={}", xdg_dir))
                .arg(format!("DBUS_SESSION_BUS_ADDRESS={}", dbus_addr))
                .arg("python3")
                .arg("-c")
                .arg(PORTAL_HELPER_SCRIPT);
            c
        } else {
            let mut c = Command::new("python3");
            c.arg("-c").arg(PORTAL_HELPER_SCRIPT);
            c
        };

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::null());

        if let Ok(child) = cmd.spawn() {
            if let Ok(mut lock) = get_portal_child().lock() {
                *lock = Some(child);
            }
        }
    }

    let frame_mutex = get_global_screen();
    let thread_flag = flag.clone();

    thread::Builder::new()
        .name("screen_capture_worker".to_string())
        .spawn(move || {
            
            while thread_flag.load(Ordering::SeqCst) {
                let frame = if use_portal {
                    read_portal_shm_frame().or_else(|| capture_screen_raw_with_engine(engine))
                } else {
                    capture_screen_raw_with_engine(engine)
                };

                if let Some(img) = frame {
                    if let Ok(mut lock) = frame_mutex.lock() {
                        *lock = Some(img);
                    }
                }
                thread::sleep(Duration::from_millis(1)); 
            }
        })
        .ok();
}

pub fn capture_screen_with_engine(engine: CaptureEngine) -> Option<GrayImage> {
    
    if let Ok(lock) = get_global_screen().lock() {
        if let Some(ref img) = *lock {
            return Some(img.clone());
        }
    }

    capture_screen_raw_with_engine(engine)
}

#[allow(dead_code)]
pub fn capture_screen() -> Option<GrayImage> {
    capture_screen_with_engine(CaptureEngine::Auto)
}

#[allow(dead_code)]
pub fn capture_screen_raw() -> Option<GrayImage> {
    capture_screen_raw_with_engine(CaptureEngine::Auto)
}

pub fn capture_screen_raw_with_engine(engine: CaptureEngine) -> Option<GrayImage> {
    
    if matches!(engine, CaptureEngine::Auto | CaptureEngine::X11Memory) {
        if let Some(img) = capture_x11_shm() {
            return Some(img);
        }
        if engine == CaptureEngine::X11Memory {
            
            return None;
        }
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let tmp_path = format!("/dev/shm/macro_screen_{}.ppm", timestamp);

    let (real_user, _real_uid) = get_real_user_and_uid();
    let _wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let xauthority = env::var("XAUTHORITY").unwrap_or_default();

    let mut captured = false;
    
    if !captured && matches!(engine, CaptureEngine::Auto | CaptureEngine::Scrot | CaptureEngine::Fallback) && which_command("scrot") {
        let mut cmd = if is_elevated() && !real_user.is_empty() {
            let mut c = Command::new("sudo");
            c.arg("-u")
                .arg(&real_user)
                .arg(format!("DISPLAY={}", display));
            if !xauthority.is_empty() {
                c.arg(format!("XAUTHORITY={}", xauthority));
            }
            c.arg("scrot").args(["-z", "-o", &tmp_path]);
            c.stderr(std::process::Stdio::null());
            c
        } else {
            let mut c = Command::new("scrot");
            c.args(["-z", "-o", &tmp_path]);
            c.stderr(std::process::Stdio::null());
            c
        };

        if let Ok(status) = cmd.status() {
            if status.success() && Path::new(&tmp_path).exists() {
                captured = true;
            }
        }
    }

    if !captured {
        return None;
    }

    let parsed_img = (|| -> Option<GrayImage> {
        let bytes = fs::read(&tmp_path).ok()?;
        if bytes.len() < 15 || bytes[0] != b'P' || bytes[1] != b'6' {
            return None;
        }
        let mut idx = 2;
        let mut tokens = Vec::with_capacity(3);
        while idx < bytes.len() && tokens.len() < 3 {
            while idx < bytes.len() && (bytes[idx] == b' ' || bytes[idx] == b'\t' || bytes[idx] == b'\r' || bytes[idx] == b'\n') {
                idx += 1;
            }
            if idx < bytes.len() && bytes[idx] == b'#' {
                while idx < bytes.len() && bytes[idx] != b'\n' {
                    idx += 1;
                }
                continue;
            }
            let start = idx;
            while idx < bytes.len() && !(bytes[idx] == b' ' || bytes[idx] == b'\t' || bytes[idx] == b'\r' || bytes[idx] == b'\n') {
                idx += 1;
            }
            if start < idx {
                let s = std::str::from_utf8(&bytes[start..idx]).ok()?;
                let val = s.parse::<usize>().ok()?;
                tokens.push(val);
            }
        }
        if tokens.len() < 3 {
            return None;
        }
        let w = tokens[0] as u32;
        let h = tokens[1] as u32;
        let _maxval = tokens[2];
        if idx < bytes.len() && (bytes[idx] == b' ' || bytes[idx] == b'\t' || bytes[idx] == b'\r' || bytes[idx] == b'\n') {
            idx += 1;
        }
        let rgb_data = &bytes[idx..];
        let total_pixels = (w as usize) * (h as usize);
        if rgb_data.len() < total_pixels * 3 {
            return None;
        }
        let mut gray_raw = vec![0u8; total_pixels];
        for i in 0..total_pixels {
            let r = rgb_data[i * 3] as u32;
            let g = rgb_data[i * 3 + 1] as u32;
            let b = rgb_data[i * 3 + 2] as u32;
            gray_raw[i] = ((299 * r + 587 * g + 114 * b) / 1000) as u8;
        }
        GrayImage::from_raw(w, h, gray_raw)
    })();

    let _ = fs::remove_file(&tmp_path);
    parsed_img.or_else(|| image::open(&tmp_path).ok().map(|img| img.into_luma8()))
}

pub struct TemplatePattern {
    pub gray: GrayImage,
    pub mask: Option<Vec<bool>>,
}

impl TemplatePattern {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let dyn_img = image::open(path).ok()?;
        Some(Self::from_dynamic_image(&dyn_img))
    }

    pub fn from_dynamic_image(img: &image::DynamicImage) -> Self {
        let gray = img.to_luma8();
        let (w, h) = gray.dimensions();

        let mask = if img.color().has_alpha() {
            let rgba = img.to_rgba8();
            let mut has_transparency = false;
            let mut m = Vec::with_capacity((w * h) as usize);
            for p in rgba.pixels() {
                let opaque = p[3] >= 128;
                if !opaque {
                    has_transparency = true;
                }
                m.push(opaque);
            }
            if has_transparency {
                Some(m)
            } else {
                None
            }
        } else {
            None
        };

        Self { gray, mask }
    }

    pub fn from_gray(gray: GrayImage) -> Self {
        Self { gray, mask: None }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.gray.dimensions()
    }
}

pub fn find_template_match(screen: &GrayImage, template: &GrayImage, threshold: f32) -> Option<(i32, i32)> {
    let t = TemplatePattern::from_gray(template.clone());
    find_template_match_with_roi(screen, &t, threshold, None)
}

pub fn find_template_match_with_roi(
    screen: &GrayImage,
    template: &TemplatePattern,
    threshold: f32,
    roi: Option<(u32, u32, u32, u32)>, 
) -> Option<(i32, i32)> {
    find_all_template_matches_with_roi(screen, template, threshold, roi, 1).into_iter().next()
}

fn find_all_masked_template_matches_with_roi(
    screen: &GrayImage,
    template: &TemplatePattern,
    mask: &[bool],
    threshold: f32,
    roi: Option<(u32, u32, u32, u32)>,
    max_matches: usize,
) -> Vec<(i32, i32)> {
    let (sw, sh) = screen.dimensions();
    let (tw, th) = template.dimensions();

    if tw == 0 || th == 0 || sw < tw || sh < th {
        return Vec::new();
    }

    let clamped_thresh = threshold.clamp(0.1, 1.0);
    let sw_usize = sw as usize;
    let sh_usize = sh as usize;
    let tw_usize = tw as usize;
    let th_usize = th as usize;
    let screen_raw = screen.as_raw();
    let template_raw = template.gray.as_raw();

    let mut active_offsets: Vec<(usize, f64)> = Vec::new();
    let mut sum_t = 0.0f64;
    for ty in 0..th_usize {
        let screen_row_off = ty * sw_usize;
        let tmpl_row_off = ty * tw_usize;
        for tx in 0..tw_usize {
            let tmpl_idx = tmpl_row_off + tx;
            if mask[tmpl_idx] {
                let val = template_raw[tmpl_idx] as f64;
                active_offsets.push((screen_row_off + tx, val));
                sum_t += val;
            }
        }
    }

    let n_active = active_offsets.len();
    if n_active == 0 {
        return Vec::new();
    }
    let n_active_f64 = n_active as f64;
    let mean_t = sum_t / n_active_f64;

    let mut sum_sq_t = 0.0f64;
    for item in &mut active_offsets {
        let val = item.1;
        let tp = val - mean_t;
        item.1 = tp;
        sum_sq_t += tp * tp;
    }

    let denom_t = sum_sq_t.sqrt();
    let is_flat = denom_t < 1e-4;

    let sample_target = 24.min(n_active).max(1);
    let sample_step = (n_active / sample_target).max(1);
    let mut sample_items = Vec::with_capacity(sample_target);
    let mut sample_sum_t = 0.0f64;
    for i in (0..n_active).step_by(sample_step).take(sample_target) {
        let (rel_off, _) = active_offsets[i];
        let val = active_offsets[i].1 + mean_t;
        sample_items.push((rel_off, val));
        sample_sum_t += val;
    }
    let sample_n = sample_items.len() as f64;
    let sample_mean_t = sample_sum_t / sample_n;
    let mut sample_sum_sq_t = 0.0f64;
    let mut sample_primes: Vec<(usize, f64)> = Vec::with_capacity(sample_items.len());
    for (rel_off, val) in sample_items {
        let sp = val - sample_mean_t;
        sample_sum_sq_t += sp * sp;
        sample_primes.push((rel_off, sp));
    }
    let sample_denom_t = sample_sum_sq_t.sqrt();

    let max_sy = sh_usize - th_usize;
    let max_sx = sw_usize - tw_usize;
    let (search_min_x, search_min_y, search_max_x, search_max_y) = if let Some((rx1, ry1, rx2, ry2)) = roi {
        let min_x = (rx1 as usize).min(max_sx);
        let min_y = (ry1 as usize).min(max_sy);
        let max_x = (rx2 as usize).min(max_sx);
        let max_y = (ry2 as usize).min(max_sy);
        (min_x, min_y, max_x.max(min_x), max_y.max(min_y))
    } else {
        (0usize, 0usize, max_sx, max_sy)
    };

    let is_small_roi = (search_max_x - search_min_x) <= 320 && (search_max_y - search_min_y) <= 320;
    let coarse_step = if is_small_roi {
        2usize
    } else {
        (tw_usize.min(th_usize) / 12).clamp(2, 4)
    };

    let total_y_range = search_max_y.saturating_sub(search_min_y);
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(16)
        .max(1);

    let rows_per_thread = (total_y_range / num_threads).max(coarse_step);

    let mut thread_chunks: Vec<(usize, usize)> = Vec::with_capacity(num_threads);
    let mut curr_y = search_min_y;
    while curr_y <= search_max_y {
        let end_y = (curr_y + rows_per_thread).min(search_max_y);
        thread_chunks.push((curr_y, end_y));
        curr_y = end_y + 1;
    }

    let sample_thresh = clamped_thresh * 0.60;
    let active_slice = &active_offsets[..];
    let sample_slice = &sample_primes[..];

    let mut candidate_peaks: Vec<(usize, usize, f32)> = Vec::new();

    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(thread_chunks.len());

        for (chunk_start_y, chunk_end_y) in thread_chunks {
            let handle = s.spawn(move || {
                let mut local_candidates: Vec<(usize, usize, f32)> = Vec::new();

                for sy in (chunk_start_y..=chunk_end_y).step_by(coarse_step) {
                    let base_row = sy * sw_usize;

                    for sx in (search_min_x..=search_max_x).step_by(coarse_step) {
                        let base_idx = base_row + sx;

                        if is_flat {
                            let mut s_sum = 0.0f64;
                            for &(rel_off, _) in active_slice {
                                s_sum += screen_raw[base_idx + rel_off] as f64;
                            }
                            let mean_i = s_sum / n_active_f64;
                            let avg_diff = ((mean_i - mean_t).abs() / 255.0) as f32;
                            let score = 1.0 - avg_diff;
                            if score >= clamped_thresh {
                                local_candidates.push((sx, sy, score));
                            }
                            continue;
                        }

                        if sample_denom_t > 1e-4 {
                            let mut s_sum = 0.0f64;
                            let mut s_sum_sq = 0.0f64;
                            let mut s_prod = 0.0f64;
                            for &(rel_off, sp) in sample_slice {
                                let p = screen_raw[base_idx + rel_off] as f64;
                                s_sum += p;
                                s_sum_sq += p * p;
                                s_prod += sp * p;
                            }
                            let s_var = s_sum_sq - (s_sum * s_sum / sample_n);
                            if s_var < 1.0 {
                                continue;
                            }
                            let s_score = (s_prod / (sample_denom_t * s_var.sqrt())) as f32;
                            if s_score < sample_thresh {
                                continue;
                            }
                        }

                        let mut full_sum = 0.0f64;
                        let mut full_sum_sq = 0.0f64;
                        let mut full_prod = 0.0f64;
                        for &(rel_off, tp) in active_slice {
                            let p = screen_raw[base_idx + rel_off] as f64;
                            full_sum += p;
                            full_sum_sq += p * p;
                            full_prod += tp * p;
                        }
                        let full_var = full_sum_sq - (full_sum * full_sum / n_active_f64);
                        if full_var < 1.0 {
                            continue;
                        }
                        let exact_score = (full_prod / (denom_t * full_var.sqrt())).clamp(-1.0, 1.0) as f32;
                        if exact_score >= clamped_thresh {
                            local_candidates.push((sx, sy, exact_score));
                        }
                    }
                }

                local_candidates
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Ok(mut candidates) = handle.join() {
                candidate_peaks.append(&mut candidates);
            }
        }
    });

    if candidate_peaks.is_empty() {
        return Vec::new();
    }

    candidate_peaks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    candidate_peaks.truncate(32);

    let refine_radius = coarse_step as isize;
    let mut refined_peaks: Vec<(usize, usize, f32)> = Vec::with_capacity(candidate_peaks.len());

    for (sx, sy, orig_score) in candidate_peaks {
        let mut best_s = orig_score;
        let mut best_coord = (sx, sy);

        if coarse_step > 1 {
            for dy in -refine_radius..=refine_radius {
                let ny = sy as isize + dy;
                if ny < search_min_y as isize || ny > search_max_y as isize {
                    continue;
                }
                let ny = ny as usize;
                for dx in -refine_radius..=refine_radius {
                    if dy == 0 && dx == 0 {
                        continue;
                    }
                    let nx = sx as isize + dx;
                    if nx < search_min_x as isize || nx > search_max_x as isize {
                        continue;
                    }
                    let nx = nx as usize;
                    let base_idx = ny * sw_usize + nx;

                    let mut full_sum = 0.0f64;
                    let mut full_sum_sq = 0.0f64;
                    let mut full_prod = 0.0f64;
                    for &(rel_off, tp) in active_slice {
                        let p = screen_raw[base_idx + rel_off] as f64;
                        full_sum += p;
                        full_sum_sq += p * p;
                        full_prod += tp * p;
                    }
                    let full_var = full_sum_sq - (full_sum * full_sum / n_active_f64);
                    if full_var >= 1.0 {
                        let score = (full_prod / (denom_t * full_var.sqrt())).clamp(-1.0, 1.0) as f32;
                        if score > best_s {
                            best_s = score;
                            best_coord = (nx, ny);
                        }
                    }
                }
            }
        }

        refined_peaks.push((best_coord.0, best_coord.1, best_s));
    }

    refined_peaks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let min_dist_x = (tw / 2).max(10) as i32;
    let min_dist_y = (th / 2).max(10) as i32;
    let mut results: Vec<(i32, i32)> = Vec::new();

    for (sx, sy, _score) in refined_peaks {
        let cx = (sx as u32 + tw / 2) as i32;
        let cy = (sy as u32 + th / 2) as i32;

        let too_close = results.iter().any(|&(rx, ry)| {
            (rx - cx).abs() < min_dist_x && (ry - cy).abs() < min_dist_y
        });

        if !too_close {
            results.push((cx, cy));
            if results.len() >= max_matches {
                break;
            }
        }
    }

    results
}

pub fn find_all_template_matches_with_roi(
    screen: &GrayImage,
    template: &TemplatePattern,
    threshold: f32,
    roi: Option<(u32, u32, u32, u32)>,
    max_matches: usize,
) -> Vec<(i32, i32)> {
    if let Some(ref mask) = template.mask {
        return find_all_masked_template_matches_with_roi(screen, template, mask, threshold, roi, max_matches);
    }

    let (sw, sh) = screen.dimensions();
    let (tw, th) = template.dimensions();

    if tw == 0 || th == 0 || sw < tw || sh < th {
        return Vec::new();
    }

    let clamped_thresh = threshold.clamp(0.1, 1.0);
    let sw_usize = sw as usize;
    let sh_usize = sh as usize;
    let tw_usize = tw as usize;
    let th_usize = th as usize;
    let screen_raw = screen.as_raw();
    let template_raw = template.gray.as_raw();
    let n_template_pixels = (tw * th) as f64;

    let mut sum_t = 0.0f64;
    for &p in template_raw {
        sum_t += p as f64;
    }
    let mean_t = sum_t / n_template_pixels;

    let mut t_primes: Vec<f64> = Vec::with_capacity(template_raw.len());
    let mut sum_sq_t = 0.0f64;
    for &p in template_raw {
        let val = p as f64 - mean_t;
        t_primes.push(val);
        sum_sq_t += val * val;
    }

    let denom_t = sum_sq_t.sqrt();
    let is_flat = denom_t < 1e-4;

    let max_sy = sh_usize - th_usize;
    let max_sx = sw_usize - tw_usize;
    let (search_min_x, search_min_y, search_max_x, search_max_y) = if let Some((rx1, ry1, rx2, ry2)) = roi {
        let min_x = (rx1 as usize).min(max_sx);
        let min_y = (ry1 as usize).min(max_sy);
        let max_x = (rx2 as usize).min(max_sx);
        let max_y = (ry2 as usize).min(max_sy);
        (min_x, min_y, max_x.max(min_x), max_y.max(min_y))
    } else {
        (0usize, 0usize, max_sx, max_sy)
    };

    let sat_y_start = search_min_y;
    let sat_y_end = (search_max_y + th_usize).min(sh_usize);
    let sat_x_start = search_min_x;
    let sat_x_end = (search_max_x + tw_usize).min(sw_usize);

    let stride_sat = sw_usize + 1;
    let needed_len = stride_sat * (sh_usize + 1);

    SAT_WORKSPACE.with(|ws| {
        let mut ws = ws.borrow_mut();
        let (sat, sat_sq) = &mut *ws;
        if sat.len() < needed_len {
            sat.resize(needed_len, 0.0);
            sat_sq.resize(needed_len, 0.0);
        } else {
            sat[..needed_len].fill(0.0);
            sat_sq[..needed_len].fill(0.0);
        }

        for y in sat_y_start..sat_y_end {
            let mut row_sum = 0.0f64;
            let mut row_sum_sq = 0.0f64;
            let row_screen = y * sw_usize;
            let sat_prev_row = y * stride_sat;
            let sat_curr_row = (y + 1) * stride_sat;

            for x in sat_x_start..sat_x_end {
                let val = screen_raw[row_screen + x] as f64;
                row_sum += val;
                row_sum_sq += val * val;
                sat[sat_curr_row + x + 1] = sat[sat_prev_row + x + 1] + row_sum;
                sat_sq[sat_curr_row + x + 1] = sat_sq[sat_prev_row + x + 1] + row_sum_sq;
            }
        }

        let get_window_stats = |sx: usize, sy: usize| -> (f64, f64) {
            let x1 = sx;
            let y1 = sy;
            let x2 = sx + tw_usize;
            let y2 = sy + th_usize;

            let sum = sat[y2 * stride_sat + x2] - sat[y1 * stride_sat + x2] - sat[y2 * stride_sat + x1] + sat[y1 * stride_sat + x1];
            let sum_sq = sat_sq[y2 * stride_sat + x2] - sat_sq[y1 * stride_sat + x2] - sat_sq[y2 * stride_sat + x1] + sat_sq[y1 * stride_sat + x1];

            (sum, sum_sq)
        };

        let sample_step = ((tw_usize * th_usize) / 32).max(1);
        let mut samples: Vec<(usize, usize, f64)> = Vec::with_capacity(36);
        for ty in (0..th_usize).step_by(1) {
            for tx in (0..tw_usize).step_by(1) {
                if (ty * tw_usize + tx) % sample_step == 0 {
                    let off_screen = ty * sw_usize + tx;
                    let t_prime = t_primes[ty * tw_usize + tx];
                    samples.push((off_screen, ty * tw_usize + tx, t_prime));
                }
            }
        }

        let coarse_step = if tw_usize > 30 && th_usize > 30 { 2usize } else { 1usize };
    let mut candidate_peaks: Vec<(usize, usize, f32)> = Vec::new();

    for sy in (search_min_y..=search_max_y).step_by(coarse_step) {
        let screen_row_base = sy * sw_usize;

        for sx in (search_min_x..=search_max_x).step_by(coarse_step) {
            let base_offset = screen_row_base + sx;
            let (sum_i, sum_sq_i) = get_window_stats(sx, sy);
            let mean_i = sum_i / n_template_pixels;
            let var_i = sum_sq_i - (sum_i * sum_i / n_template_pixels);

            if is_flat {
                let avg_diff = ((mean_i - mean_t).abs() / 255.0) as f32;
                let score = 1.0 - avg_diff;
                if score >= clamped_thresh {
                    candidate_peaks.push((sx, sy, score));
                }
                continue;
            }

            if var_i < 1.0 {
                continue;
            }

            let denom_i = var_i.sqrt();
            let total_denom = denom_t * denom_i;

            let mut sample_prod = 0.0f64;
            for &(off_screen, _, t_prime) in &samples {
                let p_screen = screen_raw[base_offset + off_screen] as f64;
                sample_prod += t_prime * (p_screen - mean_i);
            }
            let sample_ratio = samples.len() as f64 / n_template_pixels;
            let estimated_score = (sample_prod / (total_denom * sample_ratio)) as f32;

            if estimated_score >= clamped_thresh * 0.85 {
                let mut full_prod = 0.0f64;
                for ty in 0..th_usize {
                    let s_off = base_offset + ty * sw_usize;
                    let t_off = ty * tw_usize;
                    for tx in 0..tw_usize {
                        let p_screen = screen_raw[s_off + tx] as f64;
                        full_prod += t_primes[t_off + tx] * (p_screen - mean_i);
                    }
                }
                let exact_score = (full_prod / total_denom).clamp(-1.0, 1.0) as f32;
                if exact_score >= clamped_thresh {
                    candidate_peaks.push((sx, sy, exact_score));
                }
            }
        }
    }

    candidate_peaks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let min_dist_x = (tw / 2).max(10) as i32;
    let min_dist_y = (th / 2).max(10) as i32;
    let mut results: Vec<(i32, i32)> = Vec::new();

    for (sx, sy, _score) in candidate_peaks {
        let cx = (sx as u32 + tw / 2) as i32;
        let cy = (sy as u32 + th / 2) as i32;

        let too_close = results.iter().any(|&(rx, ry)| {
            (rx - cx).abs() < min_dist_x && (ry - cy).abs() < min_dist_y
        });

        if !too_close {
            results.push((cx, cy));
            if results.len() >= max_matches {
                break;
            }
        }
    }

        results
    })
}

#[allow(dead_code)]
pub fn match_template(screen: &GrayImage, template: &GrayImage, threshold: f32) -> bool {
    find_template_match(screen, template, threshold).is_some()
}

fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn get_real_user_and_uid() -> (String, u32) {
    if let Ok(pk_uid_str) = env::var("PKEXEC_UID") {
        if let Ok(uid) = pk_uid_str.parse::<u32>() {
            unsafe {
                let pw = libc::getpwuid(uid);
                if !pw.is_null() && !(*pw).pw_name.is_null() {
                    let user_cstr = CStr::from_ptr((*pw).pw_name);
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

pub fn has_native_screen_picker() -> bool {
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    (!wayland_display.is_empty() && which_command("slurp")) || which_command("xdotool")
}

pub fn pick_screen_coordinates(_mode: u8) -> Option<(i32, i32)> {
    let (real_user, real_uid) = get_real_user_and_uid();
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let _xauthority = env::var("XAUTHORITY").unwrap_or_default();
    let xdg_runtime_dir = format!("/run/user/{}", real_uid);

    if !wayland_display.is_empty() && which_command("slurp") {
        
        let slurp_args = [
            "-p",
            "-d",
            "-b",
            "#00000035",
            "-c",
            "#3b82f6",
            "-s",
            "#3b82f625",
            "-B",
            "#161925fa",
            "-w",
            "2",
            "-f",
            "%x %y",
        ];

        let mut cmd = if is_elevated() && !real_user.is_empty() {
            let mut c = Command::new("sudo");
            c.arg("-u")
                .arg(&real_user)
                .arg(format!("WAYLAND_DISPLAY={}", wayland_display))
                .arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir));
            if !display.is_empty() {
                c.arg(format!("DISPLAY={}", display));
            }
            c.arg("slurp").args(&slurp_args);
            c
        } else {
            let mut c = Command::new("slurp");
            c.args(&slurp_args);
            c
        };

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        return Some((x, y));
                    }
                }
            }
        }
    }

    get_mouse_location()
}

pub fn get_mouse_location() -> Option<(i32, i32)> {
    let (real_user, real_uid) = get_real_user_and_uid();
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let xdg_runtime_dir = format!("/run/user/{}", real_uid);

    if !wayland_display.is_empty() && which_command("python3") {
        let py_script = r#"
import dbus, dbus.service, dbus.mainloop.glib, tempfile, os, sys
from gi.repository import GLib

try:
    dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
    bus = dbus.SessionBus()
    res = None
    loop = GLib.MainLoop()

    class Recv(dbus.service.Object):
        def __init__(self):
            super().__init__(bus, "/P")
        @dbus.service.method("org.ogiu.P", in_signature="ii")
        def R(self, x, y):
            global res
            res = (x, y)
            loop.quit()

    r = Recv()
    svc = f"org.ogiu.Pos{os.getpid()}"
    name = dbus.service.BusName(svc, bus)

    with tempfile.NamedTemporaryFile(mode="w", suffix=".js", delete=False) as f:
        f.write(f'callDBus("{svc}", "/P", "org.ogiu.P", "R", Math.round(workspace.cursorPos.x), Math.round(workspace.cursorPos.y));')
        js_name = f.name

    scripting = bus.get_object("org.kde.KWin", "/Scripting")
    script_id = scripting.loadScript(js_name, dbus_interface="org.kde.kwin.Scripting")
    script_obj = bus.get_object("org.kde.KWin", f"/Scripting/Script{script_id}")
    script_obj.run(dbus_interface="org.kde.kwin.Script")
    GLib.timeout_add(150, loop.quit)
    loop.run()
    script_obj.stop(dbus_interface="org.kde.kwin.Script")
    scripting.unloadScript(js_name, dbus_interface="org.kde.kwin.Scripting")
    try:
        os.remove(js_name)
    except Exception:
        pass

    if res:
        print(f"{res[0]} {res[1]}")
        sys.exit(0)
except Exception:
    pass
sys.exit(1)
"#;
        let mut cmd = if is_elevated() && !real_user.is_empty() {
            let mut c = Command::new("sudo");
            c.arg("-u")
                .arg(&real_user)
                .arg(format!("WAYLAND_DISPLAY={}", wayland_display))
                .arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir))
                .arg(format!("DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{}/bus", real_uid));
            if !display.is_empty() {
                c.arg(format!("DISPLAY={}", display));
            }
            c.arg("python3").arg("-c").arg(py_script);
            c
        } else {
            let mut c = Command::new("python3");
            c.arg("-c").arg(py_script);
            c
        };

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        return Some((x, y));
                    }
                }
            }
        }
    }

    let mut xauthority = env::var("XAUTHORITY").unwrap_or_default();
    if xauthority.is_empty() || !Path::new(&xauthority).exists() {
        let xdg_dir = format!("/run/user/{}", real_uid);
        if let Ok(entries) = fs::read_dir(&xdg_dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("xauth_") {
                        xauthority = entry.path().to_string_lossy().to_string();
                        break;
                    }
                }
            }
        }
        if xauthority.is_empty() {
            let home_xauth = format!("/home/{}/.Xauthority", real_user);
            if Path::new(&home_xauth).exists() {
                xauthority = home_xauth;
            }
        }
    }

    if which_command("xdotool") {
        let mut cmd = if is_elevated() && !real_user.is_empty() {
            let mut c = Command::new("sudo");
            c.arg("-u")
                .arg(&real_user)
                .arg(format!("DISPLAY={}", display));
            if !xauthority.is_empty() {
                c.arg(format!("XAUTHORITY={}", xauthority));
            }
            c.arg("xdotool").arg("getmouselocation").arg("--shell");
            c
        } else {
            let mut c = Command::new("xdotool");
            if !xauthority.is_empty() {
                c.env("XAUTHORITY", &xauthority);
            }
            c.arg("getmouselocation").arg("--shell");
            c
        };

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut cur_x = None;
                let mut cur_y = None;
                for line in text.lines() {
                    if let Some(x_str) = line.strip_prefix("X=") {
                        cur_x = x_str.trim().parse::<i32>().ok();
                    } else if let Some(y_str) = line.strip_prefix("Y=") {
                        cur_y = y_str.trim().parse::<i32>().ok();
                    }
                }
                if let (Some(x), Some(y)) = (cur_x, cur_y) {
                    return Some((x, y));
                }
            }
        }
    }

    None
}

pub fn get_screen_resolution() -> (i32, i32) {
    let (real_user, real_uid) = get_real_user_and_uid();
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let xauthority = env::var("XAUTHORITY").unwrap_or_default();
    let xdg_runtime_dir = format!("/run/user/{}", real_uid);

    if which_command("kscreen-doctor") {
        let mut cmd = if is_elevated() && !real_user.is_empty() {
            let mut c = Command::new("sudo");
            c.arg("-u")
                .arg(&real_user)
                .arg(format!("WAYLAND_DISPLAY={}", wayland_display))
                .arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir))
                .arg("kscreen-doctor")
                .arg("-o");
            c
        } else {
            let mut c = Command::new("kscreen-doctor");
            c.arg("-o");
            c
        };

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("Geometry:") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let geom = parts[1]; 
                            let res_part = if parts.len() >= 3 && geom.contains(',') {
                                parts[2]
                            } else {
                                geom
                            };
                            let dims: Vec<&str> = res_part.split('x').collect();
                            if dims.len() == 2 {
                                if let (Ok(w), Ok(h)) = (dims[0].parse::<i32>(), dims[1].parse::<i32>()) {
                                    if w > 0 && h > 0 {
                                        return (w, h);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if which_command("xrandr") {
        let mut cmd = if is_elevated() && !real_user.is_empty() {
            let mut c = Command::new("sudo");
            c.arg("-u")
                .arg(&real_user)
                .arg(format!("DISPLAY={}", display));
            if !xauthority.is_empty() {
                c.arg(format!("XAUTHORITY={}", xauthority));
            }
            c.arg("xrandr");
            c
        } else {
            let c = Command::new("xrandr");
            c
        };

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.contains(" current ") {
                        let parts: Vec<&str> = line.split(" current ").collect();
                        if parts.len() > 1 {
                            let rest = parts[1];
                            let comma_parts: Vec<&str> = rest.split(',').collect();
                            let dims: Vec<&str> = comma_parts[0].trim().split('x').collect();
                            if dims.len() == 2 {
                                if let (Ok(w), Ok(h)) = (dims[0].trim().parse::<i32>(), dims[1].trim().parse::<i32>()) {
                                    if w > 0 && h > 0 {
                                        return (w, h);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (1920, 1080)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    #[test]
    fn test_match_template_exact_hit() {
        let mut screen = GrayImage::new(100, 100);
        let mut template = GrayImage::new(10, 10);

        for y in 0..10 {
            for x in 0..10 {
                template.put_pixel(x, y, Luma([200]));
                screen.put_pixel(x + 20, y + 30, Luma([200]));
            }
        }

        assert!(match_template(&screen, &template, 0.9));
    }

    #[test]
    fn test_match_template_no_match() {
        let screen = GrayImage::new(50, 50);
        let mut template = GrayImage::new(10, 10);

        for y in 0..10 {
            for x in 0..10 {
                template.put_pixel(x, y, Luma([255]));
            }
        }

        assert!(!match_template(&screen, &template, 0.8));
        assert_eq!(find_template_match(&screen, &template, 0.8), None);
    }

    #[test]
    fn test_find_template_match_coordinates() {
        let mut screen = GrayImage::new(100, 100);
        let mut template = GrayImage::new(10, 10);

        for y in 0..10 {
            for x in 0..10 {
                template.put_pixel(x, y, Luma([180]));
                screen.put_pixel(x + 40, y + 60, Luma([180]));
            }
        }

        let coord = find_template_match(&screen, &template, 0.9);
        assert!(coord.is_some());
        let (cx, cy) = coord.unwrap();
        
        assert_eq!(cx, 45);
        assert_eq!(cy, 65);
    }

    #[test]
    fn test_find_template_match_with_alpha_mask() {
        let mut screen = GrayImage::new(120, 120);
        for y in 0..120 {
            for x in 0..120 {
                screen.put_pixel(x, y, Luma([((x * 7 + y * 13) % 150 + 20) as u8]));
            }
        }

        let mut rgba = image::RgbaImage::new(20, 20);
        for y in 0..20 {
            for x in 0..20 {
                if x >= 5 && x < 15 && y >= 5 && y < 15 {
                    let c = ((x + y) * 8 + 40) as u8;
                    rgba.put_pixel(x, y, image::Rgba([c, c, c, 255]));
                    screen.put_pixel(40 + x, 30 + y, Luma([c]));
                } else {
                    rgba.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
                }
            }
        }

        let dyn_img = image::DynamicImage::ImageRgba8(rgba);
        let pattern = TemplatePattern::from_dynamic_image(&dyn_img);
        assert!(pattern.mask.is_some());

        let matches = find_all_template_matches_with_roi(&screen, &pattern, 0.9, None, 1);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (40 + 10, 30 + 10));
    }

    #[test]
    fn test_capture_screen_exec() {
        let _cap = capture_screen();
    }
}

