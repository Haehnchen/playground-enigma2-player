use std::ffi::{CStr, CString, c_char, c_double, c_int, c_void};
use std::ptr;
use std::slice;

const MPV_FORMAT_STRING: c_int = 1;
const MPV_FORMAT_FLAG: c_int = 3;
const MPV_FORMAT_INT64: c_int = 4;
const MPV_FORMAT_DOUBLE: c_int = 5;
const MPV_FORMAT_NODE: c_int = 6;
const MPV_FORMAT_NODE_ARRAY: c_int = 7;
const MPV_FORMAT_NODE_MAP: c_int = 8;

#[repr(C)]
pub struct MpvHandle {
    _private: [u8; 0],
}

#[repr(C)]
union MpvNodeValue {
    string: *mut c_char,
    flag: c_int,
    int64: i64,
    double_: c_double,
    list: *mut MpvNodeList,
    ba: *mut c_void,
}

#[repr(C)]
struct MpvNode {
    value: MpvNodeValue,
    format: c_int,
}

#[repr(C)]
struct MpvNodeList {
    num: c_int,
    values: *mut MpvNode,
    keys: *mut *mut c_char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioTrack {
    pub id: i64,
    pub label: String,
    pub selected: bool,
}

unsafe extern "C" {
    fn mpv_command(ctx: *mut MpvHandle, args: *const *const c_char) -> c_int;
    fn mpv_command_async(
        ctx: *mut MpvHandle,
        reply_userdata: u64,
        args: *const *const c_char,
    ) -> c_int;
    fn mpv_create() -> *mut MpvHandle;
    safe fn mpv_error_string(error: c_int) -> *const c_char;
    fn mpv_free_node_contents(node: *mut MpvNode);
    fn mpv_get_property(
        ctx: *mut MpvHandle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int;
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
        // SAFETY: The live handle and double value remain valid throughout the call.
        let status = unsafe {
            mpv_set_property(
                self.mpv,
                c"volume".as_ptr(),
                MPV_FORMAT_DOUBLE,
                &mut value as *mut c_double as *mut c_void,
            )
        };
        check_mpv(status, "set volume");
    }

    pub fn set_muted(&mut self, muted: bool) {
        if !self.is_ready() {
            return;
        }
        self.muted = muted;
        self.set_string_property("mute", if muted { "yes" } else { "no" }, "set mute");
    }

    pub fn toggle_muted(&mut self) {
        self.set_muted(!self.muted);
    }

    pub fn set_hwdec_enabled(&mut self, enabled: bool) {
        self.set_string_property(
            "hwdec",
            if enabled { "auto-safe" } else { "no" },
            "set hwdec",
        );
    }

    pub fn load_stream(&mut self, url: &str, label: &str) {
        if !self.is_ready() || url.trim().is_empty() {
            return;
        }
        self.current_url = Some(url.to_string());
        self.current_label = Some(label.to_string());

        let args = cargs(["loadfile", url, "replace"]);
        // SAFETY: mpv copies the terminated argument list before this call returns.
        let status = unsafe { mpv_command_async(self.mpv, 0, args.as_ptr()) };
        check_mpv(status, "loadfile");
    }

    pub fn audio_tracks(&self) -> Vec<AudioTrack> {
        if !self.is_ready() || !self.is_playing() {
            return Vec::new();
        }

        let mut node = MpvNode {
            value: MpvNodeValue {
                list: ptr::null_mut(),
            },
            format: 0,
        };
        // SAFETY: The live handle writes into a properly aligned MpvNode of the requested format.
        let status = unsafe {
            mpv_get_property(
                self.mpv,
                c"track-list".as_ptr(),
                MPV_FORMAT_NODE,
                &mut node as *mut MpvNode as *mut c_void,
            )
        };
        if status < 0 {
            log_mpv_error(status, "get audio tracks");
            return Vec::new();
        }

        // SAFETY: The successful query owns a valid node tree until it is freed below.
        let tracks = unsafe { audio_tracks_from_node(&node) };
        // SAFETY: This releases the successful query exactly once, after copying its data.
        unsafe { mpv_free_node_contents(&mut node) };
        tracks
    }

    pub fn set_audio_track(&mut self, id: i64) {
        if !self.is_ready() || !self.is_playing() {
            return;
        }

        self.set_string_property("aid", &id.to_string(), "set audio track");
    }

    pub fn drop_buffers(&self) {
        if !self.is_ready() || !self.is_playing() {
            return;
        }
        let args = cargs(["drop-buffers"]);
        // SAFETY: The handle is live; mpv copies the terminated argument list on submission.
        let status = unsafe { mpv_command_async(self.mpv, 0, args.as_ptr()) };
        check_mpv(status, "drop buffers");
    }

    pub fn toggle_stream_info(&self) {
        if !self.is_ready() {
            return;
        }
        let args = cargs(["script-binding", "stats/display-stats-toggle"]);
        // SAFETY: The handle is live and the terminated argument list outlives the call.
        let status = unsafe { mpv_command(self.mpv, args.as_ptr()) };
        if status < 0 {
            let fallback = cargs(["keypress", "i"]);
            // SAFETY: The fallback list has the same lifetime and termination guarantees.
            let status = unsafe { mpv_command(self.mpv, fallback.as_ptr()) };
            check_mpv(status, "toggle stream info");
        }
    }

    pub fn reenable_video(&self) {
        if !self.is_ready() || !self.is_playing() {
            return;
        }
        self.set_string_property("vid", "no", "disable video");
        self.set_string_property("vid", "auto", "enable video");
    }

    fn set_string_property(&self, name: &str, value: &str, action: &str) {
        if !self.is_ready() {
            return;
        }
        let name = cstr(name);
        let value = cstr(value);
        // SAFETY: The session owns the live handle and the C strings outlive the call.
        let status = unsafe { mpv_set_property_string(self.mpv, name.as_ptr(), value.as_ptr()) };
        check_mpv(status, action);
    }

    fn init(&mut self, hwdec: bool) {
        // SAFETY: The executable configures LC_NUMERIC before GTK and mpv start threads.
        self.mpv = unsafe { mpv_create() };
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

        // SAFETY: This session exclusively owns the newly created handle.
        let status = unsafe { mpv_initialize(self.mpv) };
        if status < 0 {
            log_mpv_error(status, "mpv initialize");
            // SAFETY: The failed handle is live and is cleared immediately after destruction.
            unsafe { mpv_terminate_destroy(self.mpv) };
            self.mpv = ptr::null_mut();
        }
    }

    fn set_option(&self, name: &str, value: &str) {
        if !self.is_ready() {
            return;
        }
        let option_name = cstr(name);
        let value = cstr(value);
        // SAFETY: The session owns the live handle and the C strings outlive the call.
        let status =
            unsafe { mpv_set_option_string(self.mpv, option_name.as_ptr(), value.as_ptr()) };
        check_mpv(status, name);
    }
}

impl Drop for PlayerSession {
    fn drop(&mut self) {
        if !self.mpv.is_null() {
            // SAFETY: The session owns the handle; renderers retain the session until released.
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

fn check_mpv(status: c_int, action: &str) {
    if status < 0 {
        log_mpv_error(status, action);
    }
}

pub(super) fn log_mpv_error(status: c_int, action: &str) {
    let message = mpv_error_string(status);
    if message.is_null() {
        eprintln!("{action}: mpv error {status}");
    } else {
        // SAFETY: mpv returns a static, NUL-terminated string for every error code.
        let message = unsafe { CStr::from_ptr(message) }.to_string_lossy();
        eprintln!("{action}: {message}");
    }
}

/// # Safety
/// The node and all reachable mpv allocations must remain valid throughout this call.
unsafe fn audio_tracks_from_node(node: &MpvNode) -> Vec<AudioTrack> {
    let Some(list) = (unsafe { node_list(node, MPV_FORMAT_NODE_ARRAY) }) else {
        return Vec::new();
    };
    if list.values.is_null() {
        return Vec::new();
    }

    // SAFETY: The caller guarantees the mpv array and its children remain alive.
    unsafe {
        slice::from_raw_parts(list.values, list.num as usize)
            .iter()
            .filter_map(|node| audio_track_from_node(node))
            .collect()
    }
}

/// # Safety
/// The node and all reachable mpv allocations must remain valid throughout this call.
unsafe fn audio_track_from_node(node: &MpvNode) -> Option<AudioTrack> {
    let track_type = unsafe { node_string(map_value(node, "type")?) }?;
    if track_type != "audio" {
        return None;
    }
    if unsafe { node_flag(map_value(node, "albumart")) }.unwrap_or(false) {
        return None;
    }

    let id = unsafe { node_int64(map_value(node, "id")?) }?;
    let selected = unsafe { node_flag(map_value(node, "selected")) }.unwrap_or(false);
    Some(AudioTrack {
        id,
        label: unsafe { audio_track_label(node, id) },
        selected,
    })
}

/// # Safety
/// The node and all reachable mpv allocations must remain valid throughout this call.
unsafe fn audio_track_label(node: &MpvNode, id: i64) -> String {
    let title = unsafe { track_string(node, "title") };
    let lang = unsafe { track_string(node, "lang") };
    let mut label = match (title, lang) {
        (Some(title), Some(lang)) if !title.eq_ignore_ascii_case(&lang) => {
            format!("{title} ({lang})")
        }
        (Some(title), _) => title,
        (_, Some(lang)) => lang,
        _ => format!("Audio {id}"),
    };

    let mut details = Vec::new();
    if let Some(channels) = unsafe { track_string(node, "demux-channels") } {
        details.push(channels.replace("(side)", ""));
    } else if let Some(count) = unsafe {
        map_value(node, "demux-channel-count")
            .and_then(|node| node_int64(node))
            .or_else(|| map_value(node, "audio-channels").and_then(|node| node_int64(node)))
    }
    .filter(|count| *count > 0)
    {
        details.push(format!("{count}ch"));
    }
    if let Some(codec) = unsafe { track_string(node, "codec") } {
        details.push(codec);
    }

    if !details.is_empty() {
        label.push_str(" - ");
        label.push_str(&details.join(" "));
    }

    label
}

/// # Safety
/// The node and its keys and values must remain valid throughout this call.
unsafe fn track_string(node: &MpvNode, key: &str) -> Option<String> {
    unsafe { map_value(node, key).and_then(|node| node_string(node)) }
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// # Safety
/// The node's keys and values must remain valid for the lifetime of the returned borrow.
unsafe fn map_value<'a>(node: &'a MpvNode, key: &str) -> Option<&'a MpvNode> {
    let list = unsafe { node_list(node, MPV_FORMAT_NODE_MAP) }?;
    if list.values.is_null() || list.keys.is_null() {
        return None;
    }

    for i in 0..list.num as usize {
        // SAFETY: mpv guarantees num entries in both arrays for map nodes.
        let key_ptr = unsafe { *list.keys.add(i) };
        if key_ptr.is_null() {
            continue;
        }
        if unsafe { CStr::from_ptr(key_ptr) }.to_bytes() == key.as_bytes() {
            return Some(unsafe { &*list.values.add(i) });
        }
    }
    None
}

/// # Safety
/// The format must be a list format, and its allocation must outlive the returned borrow.
unsafe fn node_list(node: &MpvNode, format: c_int) -> Option<&MpvNodeList> {
    if node.format != format {
        return None;
    }
    // SAFETY: The caller keeps the mpv node alive, and the format identifies a list.
    let list = unsafe { node.value.list.as_ref() }?;
    (list.num >= 0).then_some(list)
}

/// # Safety
/// A string node must contain a valid, live, NUL-terminated C string.
unsafe fn node_string(node: &MpvNode) -> Option<String> {
    if node.format != MPV_FORMAT_STRING {
        return None;
    }
    // SAFETY: The checked format selects the string field of the live mpv node.
    let value = unsafe { node.value.string };
    if value.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// # Safety
/// The union field identified by the node format must be initialized.
unsafe fn node_int64(node: &MpvNode) -> Option<i64> {
    if node.format != MPV_FORMAT_INT64 {
        return None;
    }
    // SAFETY: The checked format selects the initialized integer field.
    Some(unsafe { node.value.int64 })
}

/// # Safety
/// The union field identified by the node format must be initialized.
unsafe fn node_flag(node: Option<&MpvNode>) -> Option<bool> {
    let node = node?;
    if node.format != MPV_FORMAT_FLAG {
        return None;
    }
    // SAFETY: The checked format selects the initialized flag field.
    Some(unsafe { node.value.flag != 0 })
}

#[cfg(test)]
#[path = "../../../tests/common/player/session.rs"]
mod tests;
