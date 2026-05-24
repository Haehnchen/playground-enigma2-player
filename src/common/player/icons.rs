use gtk::prelude::*;
use std::f64::consts::PI;

#[derive(Clone, Copy)]
pub enum WindowIcon {
    Minimize,
    Fullscreen,
    Restore,
    Close,
}

pub fn settings() -> gtk::DrawingArea {
    drawing_area(20, 20, |cr, width, height| {
        let size = width.min(height) as f64;
        let gear = size * 0.72;
        let cx = width as f64 / 2.0;
        let cy = height as f64 / 2.0;
        let outer = gear * 0.39;
        let inner = gear * 0.20;
        let dirs = [
            (1.0, 0.0),
            (0.7071, 0.7071),
            (0.0, 1.0),
            (-0.7071, 0.7071),
            (-1.0, 0.0),
            (-0.7071, -0.7071),
            (0.0, -1.0),
            (0.7071, -0.7071),
        ];

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.94);
        cr.set_line_cap(gtk::cairo::LineCap::Round);
        cr.set_line_join(gtk::cairo::LineJoin::Round);
        cr.set_line_width(1.2_f64.max(gear * 0.10));

        for (dx, dy) in dirs {
            cr.move_to(cx + dx * gear * 0.31, cy + dy * gear * 0.31);
            cr.line_to(cx + dx * outer, cy + dy * outer);
        }
        let _ = cr.stroke();

        cr.set_line_width(1.1_f64.max(gear * 0.09));
        cr.arc(cx, cy, gear * 0.29, 0.0, 2.0 * PI);
        let _ = cr.stroke();

        cr.arc(cx, cy, inner, 0.0, 2.0 * PI);
        let _ = cr.stroke();
    })
}

pub fn window(kind: WindowIcon) -> gtk::DrawingArea {
    drawing_area(20, 20, move |cr, width, height| {
        let size = width.min(height) as f64;
        let x = (width as f64 - size) / 2.0;
        let y = (height as f64 - size) / 2.0;
        let left = x + size * 0.25;
        let right = x + size * 0.75;
        let top = y + size * 0.25;
        let bottom = y + size * 0.75;
        let center_y = y + size * 0.55;

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.94);
        cr.set_line_width(1.8_f64.max(size * 0.10));
        cr.set_line_cap(gtk::cairo::LineCap::Round);
        cr.set_line_join(gtk::cairo::LineJoin::Round);

        match kind {
            WindowIcon::Minimize => {
                cr.move_to(left, center_y);
                cr.line_to(right, center_y);
            }
            WindowIcon::Fullscreen => {
                let inset = size * 0.06;
                cr.rectangle(
                    left + inset,
                    top + inset,
                    right - left - inset * 2.0,
                    bottom - top - inset * 2.0,
                );
            }
            WindowIcon::Restore => {
                let offset = size * 0.13;
                let inset = size * 0.05;
                cr.rectangle(
                    left + inset,
                    top + offset,
                    right - left - offset,
                    bottom - top - offset,
                );
                cr.rectangle(
                    left + offset,
                    top + inset,
                    right - left - offset,
                    bottom - top - offset,
                );
            }
            WindowIcon::Close => {
                cr.move_to(left, top);
                cr.line_to(right, bottom);
                cr.move_to(right, top);
                cr.line_to(left, bottom);
            }
        }

        let _ = cr.stroke();
    })
}

pub fn volume(muted: bool) -> gtk::DrawingArea {
    drawing_area(16, 16, move |cr, width, height| {
        let size = width.min(height) as f64;
        let x = (width as f64 - size) / 2.0;
        let y = (height as f64 - size) / 2.0;
        let left = x + size * 0.06;
        let right = x + size * 0.58;
        let top = y + size * 0.20;
        let bottom = y + size * 0.80;
        let body_right = x + size * 0.31;
        let center_y = y + size * 0.50;

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.94);
        cr.set_line_width(1.8_f64.max(size * 0.11));
        cr.set_line_cap(gtk::cairo::LineCap::Round);
        cr.set_line_join(gtk::cairo::LineJoin::Round);

        cr.move_to(left, y + size * 0.37);
        cr.line_to(body_right, y + size * 0.37);
        cr.line_to(right, top);
        cr.line_to(right, bottom);
        cr.line_to(body_right, y + size * 0.63);
        cr.line_to(left, y + size * 0.63);
        cr.close_path();
        let _ = cr.stroke();

        if muted {
            let cx = x + size * 0.80;
            let mark = size * 0.17;
            cr.set_line_width(1.8_f64.max(size * 0.11));
            cr.move_to(cx - mark, center_y - mark);
            cr.line_to(cx + mark, center_y + mark);
            cr.move_to(cx + mark, center_y - mark);
            cr.line_to(cx - mark, center_y + mark);
        } else {
            cr.set_line_width(1.6_f64.max(size * 0.10));
            cr.new_sub_path();
            cr.arc(x + size * 0.56, center_y, size * 0.23, -0.68, 0.68);
            cr.new_sub_path();
            cr.arc(x + size * 0.56, center_y, size * 0.39, -0.62, 0.62);
        }

        let _ = cr.stroke();
    })
}

fn drawing_area(
    width: i32,
    height: i32,
    draw: impl Fn(&gtk::cairo::Context, i32, i32) + 'static,
) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_content_width(width);
    area.set_content_height(height);
    area.set_size_request(width, height);
    area.set_halign(gtk::Align::Center);
    area.set_valign(gtk::Align::Center);
    area.set_draw_func(move |_, cr, width, height| draw(cr, width, height));
    area
}
