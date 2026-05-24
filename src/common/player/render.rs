use super::session::{MpvHandle, PlayerSession};
use glib::Propagation;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::ffi::{c_char, c_int, c_ulonglong, c_void};
use std::ptr;
use std::rc::{Rc, Weak};
use std::time::Duration;

const GL_COLOR_BUFFER_BIT: u32 = 0x0000_4000;
const GL_FRAMEBUFFER_BINDING: u32 = 0x8CA6;
const MPV_RENDER_PARAM_INVALID: c_int = 0;
const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
const MPV_RENDER_PARAM_OPENGL_INIT_PARAMS: c_int = 2;
const MPV_RENDER_PARAM_OPENGL_FBO: c_int = 3;
const MPV_RENDER_PARAM_FLIP_Y: c_int = 4;

#[repr(C)]
struct MpvRenderContext {
    _private: [u8; 0],
}

#[repr(C)]
struct MpvOpenGLInitParams {
    get_proc_address: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void>,
    get_proc_address_ctx: *mut c_void,
}

#[repr(C)]
struct MpvOpenGLFbo {
    fbo: c_int,
    w: c_int,
    h: c_int,
    internal_format: c_int,
}

#[repr(C)]
struct MpvRenderParam {
    type_: c_int,
    data: *mut c_void,
}

unsafe extern "C" {
    static epoxy_eglGetProcAddress: unsafe extern "C" fn(*const c_char) -> *mut c_void;
    static epoxy_glClear: unsafe extern "C" fn(u32);
    static epoxy_glClearColor: unsafe extern "C" fn(f32, f32, f32, f32);
    static epoxy_glGetIntegerv: unsafe extern "C" fn(u32, *mut c_int);

    fn mpv_error_string(error: c_int) -> *const c_char;
    fn mpv_render_context_create(
        res: *mut *mut MpvRenderContext,
        mpv: *mut MpvHandle,
        params: *mut MpvRenderParam,
    ) -> c_int;
    fn mpv_render_context_free(ctx: *mut MpvRenderContext);
    fn mpv_render_context_render(ctx: *mut MpvRenderContext, params: *mut MpvRenderParam) -> c_int;
    fn mpv_render_context_update(ctx: *mut MpvRenderContext) -> c_ulonglong;
}

pub struct MpvVideo {
    area: gtk::GLArea,
    session: Rc<RefCell<PlayerSession>>,
    render_context: Cell<*mut MpvRenderContext>,
    last_width: Cell<i32>,
    last_height: Cell<i32>,
}

impl MpvVideo {
    pub fn new(session: Rc<RefCell<PlayerSession>>) -> Rc<Self> {
        let area = gtk::GLArea::builder()
            .auto_render(false)
            .hexpand(true)
            .vexpand(true)
            .build();
        area.add_css_class("video-area");

        let video = Rc::new(Self {
            area,
            session,
            render_context: Cell::new(ptr::null_mut()),
            last_width: Cell::new(0),
            last_height: Cell::new(0),
        });

        video.connect_signals();
        video.start_render_tick();
        video
    }

    pub fn widget(&self) -> gtk::GLArea {
        self.area.clone()
    }

    pub fn queue_render(&self) {
        self.area.queue_render();
    }

    fn connect_signals(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.area.connect_realize(move |area| {
            if let Some(video) = weak.upgrade() {
                video.realize(area);
            }
        });

        let weak = Rc::downgrade(self);
        self.area.connect_unrealize(move |area| {
            if let Some(video) = weak.upgrade() {
                area.make_current();
                video.clear_render_context();
            }
        });

        let weak = Rc::downgrade(self);
        self.area.connect_render(move |area, _context| {
            if let Some(video) = weak.upgrade() {
                video.render(area);
            }
            Propagation::Stop
        });
    }

    fn start_render_tick(self: &Rc<Self>) {
        let weak: Weak<Self> = Rc::downgrade(self);
        glib::timeout_add_local(Duration::from_millis(33), move || {
            if let Some(video) = weak.upgrade() {
                video.queue_render();
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    fn realize(&self, area: &gtk::GLArea) {
        area.make_current();
        if let Some(error) = area.error() {
            eprintln!("GTK GLArea error: {error}");
            return;
        }

        let mpv = self.session.borrow().handle();
        if mpv.is_null() {
            return;
        }

        let mut gl_init_params = MpvOpenGLInitParams {
            get_proc_address: Some(get_proc_address),
            get_proc_address_ctx: ptr::null_mut(),
        };
        let mut params = [
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_API_TYPE,
                data: c"opengl".as_ptr() as *mut c_void,
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
                data: &mut gl_init_params as *mut MpvOpenGLInitParams as *mut c_void,
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let mut render_context = ptr::null_mut();
        let status =
            unsafe { mpv_render_context_create(&mut render_context, mpv, params.as_mut_ptr()) };
        if status < 0 {
            log_mpv_error(status, "mpv render context");
            return;
        }
        self.render_context.set(render_context);
        self.session.borrow().reenable_video();
        area.queue_render();
    }

    fn render(&self, area: &gtk::GLArea) {
        let context = self.render_context.get();
        if context.is_null() {
            area.attach_buffers();
            unsafe {
                (epoxy_glClearColor)(0.02, 0.02, 0.02, 1.0);
                (epoxy_glClear)(GL_COLOR_BUFFER_BIT);
            }
            return;
        }

        let scale = area.scale_factor();
        let width = area.width() * scale;
        let height = area.height() * scale;
        if width <= 0 || height <= 0 {
            return;
        }

        unsafe {
            let _ = mpv_render_context_update(context);
        }
        area.attach_buffers();

        let mut current_fbo = 0;
        unsafe {
            (epoxy_glGetIntegerv)(GL_FRAMEBUFFER_BINDING, &mut current_fbo);
        }
        let mut fbo = MpvOpenGLFbo {
            fbo: current_fbo,
            w: width,
            h: height,
            internal_format: 0,
        };
        let mut flip_y: c_int = 1;
        let mut params = [
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_OPENGL_FBO,
                data: &mut fbo as *mut MpvOpenGLFbo as *mut c_void,
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_FLIP_Y,
                data: &mut flip_y as *mut c_int as *mut c_void,
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];
        let status = unsafe { mpv_render_context_render(context, params.as_mut_ptr()) };
        if status < 0 {
            log_mpv_error(status, "mpv render");
        } else {
            self.last_width.set(width);
            self.last_height.set(height);
        }
    }

    fn clear_render_context(&self) {
        let context = self.render_context.replace(ptr::null_mut());
        if !context.is_null() {
            unsafe {
                mpv_render_context_free(context);
            }
        }
        self.last_width.set(0);
        self.last_height.set(0);
    }
}

impl Drop for MpvVideo {
    fn drop(&mut self) {
        self.clear_render_context();
    }
}

unsafe extern "C" fn get_proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    (epoxy_eglGetProcAddress)(name)
}

fn log_mpv_error(status: c_int, action: &str) {
    unsafe {
        let message = mpv_error_string(status);
        if message.is_null() {
            eprintln!("{action}: mpv error {status}");
        } else {
            let message = std::ffi::CStr::from_ptr(message).to_string_lossy();
            eprintln!("{action}: {message}");
        }
    }
}
