use gtk::gdk::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn add_resize_handles(
    overlay: &gtk::Overlay,
    window: &gtk::ApplicationWindow,
    fullscreen: Rc<RefCell<bool>>,
) {
    for spec in resize_specs() {
        overlay.add_overlay(&create_resize_handle(window, fullscreen.clone(), spec));
    }
}

struct ResizeHandleSpec {
    edge: gtk::gdk::SurfaceEdge,
    halign: gtk::Align,
    valign: gtk::Align,
    width: i32,
    height: i32,
    cursor: &'static str,
}

fn resize_specs() -> [ResizeHandleSpec; 8] {
    [
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::North,
            halign: gtk::Align::Fill,
            valign: gtk::Align::Start,
            width: -1,
            height: 6,
            cursor: "n-resize",
        },
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::South,
            halign: gtk::Align::Fill,
            valign: gtk::Align::End,
            width: -1,
            height: 6,
            cursor: "s-resize",
        },
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::West,
            halign: gtk::Align::Start,
            valign: gtk::Align::Fill,
            width: 6,
            height: -1,
            cursor: "w-resize",
        },
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::East,
            halign: gtk::Align::End,
            valign: gtk::Align::Fill,
            width: 6,
            height: -1,
            cursor: "e-resize",
        },
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::NorthWest,
            halign: gtk::Align::Start,
            valign: gtk::Align::Start,
            width: 12,
            height: 12,
            cursor: "nw-resize",
        },
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::NorthEast,
            halign: gtk::Align::End,
            valign: gtk::Align::Start,
            width: 12,
            height: 12,
            cursor: "ne-resize",
        },
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::SouthWest,
            halign: gtk::Align::Start,
            valign: gtk::Align::End,
            width: 12,
            height: 12,
            cursor: "sw-resize",
        },
        ResizeHandleSpec {
            edge: gtk::gdk::SurfaceEdge::SouthEast,
            halign: gtk::Align::End,
            valign: gtk::Align::End,
            width: 12,
            height: 12,
            cursor: "se-resize",
        },
    ]
}

fn create_resize_handle(
    window: &gtk::ApplicationWindow,
    fullscreen: Rc<RefCell<bool>>,
    spec: ResizeHandleSpec,
) -> gtk::Box {
    let handle = gtk::Box::new(gtk::Orientation::Vertical, 0);
    handle.add_css_class("resize-handle");
    handle.set_halign(spec.halign);
    handle.set_valign(spec.valign);
    handle.set_size_request(spec.width, spec.height);
    handle.set_cursor_from_name(Some(spec.cursor));
    if spec.halign == gtk::Align::Fill {
        handle.set_hexpand(true);
    }
    if spec.valign == gtk::Align::Fill {
        handle.set_vexpand(true);
    }

    let click = gtk::GestureClick::new();
    click.set_button(1);
    {
        let window = window.clone();
        let fullscreen = fullscreen.clone();
        click.connect_pressed(move |gesture, n_press, x, y| {
            if n_press == 1 && !*fullscreen.borrow() {
                begin_window_resize(&window, gesture, spec.edge, x, y);
            }
        });
    }
    handle.add_controller(click);
    handle
}

fn begin_window_resize(
    window: &gtk::ApplicationWindow,
    gesture: &gtk::GestureClick,
    edge: gtk::gdk::SurfaceEdge,
    x: f64,
    y: f64,
) {
    let Some(device) = gesture.current_event_device() else {
        return;
    };
    let Some(surface) = window.surface() else {
        return;
    };
    let Ok(toplevel) = surface.downcast::<gtk::gdk::Toplevel>() else {
        return;
    };

    toplevel.begin_resize(edge, Some(&device), 1, x, y, gesture.current_event_time());
}
