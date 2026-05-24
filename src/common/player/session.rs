use std::ffi::{c_char, c_double, c_int, c_void, CString};
use std::ptr;

const LC_NUMERIC: c_int = 1;
const MPV_FORMAT_DOUBLE: c_int = 5;

#[repr(C)]
pub struct MpvHandle {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn mpv_command(ctx: *mut MpvHandle, args: *const *const c_char) -> c_int;
    fn mpv_command_async(
        ctx: *mut MpvHandle,
        reply_userdata: u64,
        args: *const *const c_char,
    ) -> c_int;
    fn mpv_create() -> *mut MpvHandle;
    fn mpv_error_string(error: c_int) -> *const c_char;
    fn mpv_initialize(ctx: *mut MpvHandle) -> c_int;
    fn mpv_set_option_string(
        ctx: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;
    fn mpv_set_property(
        ctx: *mut MpvHandle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int;
    fn mpv_set_property_string(
        ctx: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;
    fn mpv_terminate_destroy(ctx: *mut MpvHandle);
    fn setlocale(category: c_int, locale: *const c_char) -> *mut c_char;
}

pub struct PlayerSession {
    mpv: *mut MpvHandle,
    volume: f64,
    muted: bool,
    current_url: Option<String>,
    current_label: Option<String>,
}

impl PlayerSession {
    pub fn new(hwdec: bool) -> Self {
        let mut session = Self {
            mpv: ptr::null_mut(),
            volume: 80.0,
            muted: false,
            current_url: None,
            current_label: None,
        };
        session.init(hwdec);
        session
    }

    pub fn is_ready(&self) -> bool {
        !self.mpv.is_null()
    }

    pub fn is_playing(&self) -> bool {
        self.current_url.is_some()
    }

    pub fn current_label(&self) -> Option<&str> {
        self.current_label.as_deref()
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn handle(&self) -> *mut MpvHandle {
        self.mpv
    }

    pub fn set_volume(&mut self, volume: f64) {
        if !self.is_ready() {
            return;
        }
        self.volume = volume;
        let mut value = volume;
        unsafe {
            check_mpv(
                mpv_set_property(
                    self.mpv,
                    cstr("volume").as_ptr(),
                    MPV_FORMAT_DOUBLE,
                    &mut value as *mut c_double as *mut c_void,
                ),
                "set volume",
            );
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        if !self.is_ready() {
            return;
        }
        self.muted = muted;
        unsafe {
            check_mpv(
                mpv_set_property_string(
                    self.mpv,
                    cstr("mute").as_ptr(),
                    cstr(if muted { "yes" } else { "no" }).as_ptr(),
                ),
                "set mute",
            );
        }
    }

    pub fn toggle_muted(&mut self) {
        self.set_muted(!self.muted);
    }

    pub fn set_hwdec_enabled(&mut self, enabled: bool) {
        if !self.is_ready() {
            return;
        }
        unsafe {
            check_mpv(
                mpv_set_property_string(
                    self.mpv,
                    cstr("hwdec").as_ptr(),
                    cstr(if enabled { "auto-safe" } else { "no" }).as_ptr(),
                ),
                "set hwdec",
            );
        }
    }

    pub fn load_stream(&mut self, url: &str, label: &str) {
        if !self.is_ready() || url.trim().is_empty() {
            return;
        }
        self.current_url = Some(url.to_string());
        self.current_label = Some(label.to_string());

        let args = cargs(["loadfile", url, "replace"]);
        unsafe {
            check_mpv(mpv_command_async(self.mpv, 0, args.as_ptr()), "loadfile");
        }
    }

    pub fn drop_buffers(&self) {
        if !self.is_ready() || !self.is_playing() {
            return;
        }
        let args = cargs(["drop-buffers"]);
        unsafe {
            check_mpv(
                mpv_command_async(self.mpv, 0, args.as_ptr()),
                "drop buffers",
            );
        }
    }

    pub fn toggle_stream_info(&self) {
        if !self.is_ready() {
            return;
        }
        let args = cargs(["script-binding", "stats/display-stats-toggle"]);
        unsafe {
            let status = mpv_command(self.mpv, args.as_ptr());
            if status < 0 {
                let fallback = cargs(["keypress", "i"]);
                check_mpv(
                    mpv_command(self.mpv, fallback.as_ptr()),
                    "toggle stream info",
                );
            }
        }
    }

    pub fn reenable_video(&self) {
        if !self.is_ready() || !self.is_playing() {
            return;
        }
        unsafe {
            check_mpv(
                mpv_set_property_string(self.mpv, cstr("vid").as_ptr(), cstr("no").as_ptr()),
                "disable video",
            );
            check_mpv(
                mpv_set_property_string(self.mpv, cstr("vid").as_ptr(), cstr("auto").as_ptr()),
                "enable video",
            );
        }
    }

    fn init(&mut self, hwdec: bool) {
        unsafe {
            let _ = setlocale(LC_NUMERIC, cstr("C").as_ptr());
            self.mpv = mpv_create();
            if self.mpv.is_null() {
                eprintln!("mpv_create returned NULL");
                return;
            }

            self.set_option("terminal", "no");
            self.set_option("config", "no");
            self.set_option("vo", "libmpv");
            self.set_option("hwdec", if hwdec { "auto-safe" } else { "no" });
            // Enigma2 live TV is usually on the local network. Prefer quick zapping over
            // collecting a large VOD-style cache before playback starts.
            self.set_option("cache", "yes");
            self.set_option("cache-pause-initial", "no");
            self.set_option("cache-pause-wait", "0.25");
            self.set_option("cache-secs", "2");
            self.set_option("demuxer-max-bytes", "32MiB");
            self.set_option("demuxer-readahead-secs", "1");
            self.set_option("stream-buffer-size", "512KiB");
            self.set_option("network-timeout", "3");
            self.set_option("video-sync", "display-resample");
            self.set_option("volume", "80");

            let status = mpv_initialize(self.mpv);
            if status < 0 {
                log_mpv_error(status, "mpv initialize");
                mpv_terminate_destroy(self.mpv);
                self.mpv = ptr::null_mut();
            }
        }
    }

    unsafe fn set_option(&self, name: &str, value: &str) {
        check_mpv(
            mpv_set_option_string(self.mpv, cstr(name).as_ptr(), cstr(value).as_ptr()),
            name,
        );
    }
}

impl Drop for PlayerSession {
    fn drop(&mut self) {
        if !self.mpv.is_null() {
            unsafe {
                mpv_terminate_destroy(self.mpv);
            }
            self.mpv = ptr::null_mut();
        }
    }
}

fn cstr(value: &str) -> CString {
    CString::new(value).expect("mpv strings must not contain NUL")
}

struct CArgs {
    _strings: Vec<CString>,
    pointers: Vec<*const c_char>,
}

impl CArgs {
    fn as_ptr(&self) -> *const *const c_char {
        self.pointers.as_ptr()
    }
}

fn cargs<const N: usize>(values: [&str; N]) -> CArgs {
    let strings: Vec<CString> = values.into_iter().map(cstr).collect();
    let mut pointers: Vec<*const c_char> = strings.iter().map(|value| value.as_ptr()).collect();
    pointers.push(ptr::null());
    CArgs {
        _strings: strings,
        pointers,
    }
}

unsafe fn check_mpv(status: c_int, action: &str) {
    if status < 0 {
        log_mpv_error(status, action);
    }
}

unsafe fn log_mpv_error(status: c_int, action: &str) {
    let message = mpv_error_string(status);
    if message.is_null() {
        eprintln!("{action}: mpv error {status}");
    } else {
        let message = std::ffi::CStr::from_ptr(message).to_string_lossy();
        eprintln!("{action}: {message}");
    }
}
