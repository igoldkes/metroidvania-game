use macroquad::prelude::*;

pub mod theme;
pub mod layout;
pub mod components;

pub struct PanelStyle {
    pub bg: Color,
    pub border: Option<(f32, Color)>,
}

pub fn draw_panel(rect: Rect, style: PanelStyle) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, style.bg);
    if let Some((thickness, color)) = style.border {
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, color);
    }
}

/// Solid black fill in **screen space** (default camera). Use before drawing intro UI on top.
#[inline]
pub fn draw_fullscreen_opaque_black() {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 255),
    );
}