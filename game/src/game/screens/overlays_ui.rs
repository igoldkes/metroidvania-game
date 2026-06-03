use macroquad::prelude::*;

use super::super::ui::components::{draw_modal_chrome, draw_wrapped_text, ModalChromeProps};
use super::super::ui::layout::{centered_clamped_rect, safe_margins, scaled_type, ui_scale};
use super::super::ui::theme::{TypeScale, UiPreferences};
use super::super::ui::{draw_panel, PanelStyle};
use super::super::PauseMenuState;

pub fn draw_pause_menu_overlay( menu_state: PauseMenuState ) {
    match menu_state {
        PauseMenuState::Menu { pause_menu_role } => {
            let prefs = UiPreferences::default();
            let palette = prefs.palette();
            let scale = ui_scale();
            let margin = safe_margins(scale);
            let ty = scaled_type(&TypeScale::default(), scale);
            
            let width = screen_width();
            let height = screen_height();

            draw_rectangle(0.0, 0.0, width, height, Color::from_rgba(0, 0, 0, 160));
            
            let pw = 640.0 * scale;
            let ph = 285.0 * scale;

            let rect = centered_clamped_rect(pw, ph, margin);
            let x = rect.x;
            let y = rect.y;

            let row0_y = y + 92.0 * scale;
            let row_h = 38.0 * scale;
            let row_pad_x = 18.0 * scale;
            let row_bg_w = rect.w - row_pad_x * 2.0;

            draw_panel(
                Rect::new(x, y, pw, ph),
                PanelStyle {
                    bg: Color::from_rgba(12, 14, 28, 245),
                    border: Some((2.0, Color::from_rgba(130, 150, 220, 255))),
                },
            );

            let labels: [&str; 3] = [
                "Resume Game",
                "Return to Main Menu",
                "Exit to Desktop",
            ];

            for i in 0..3 {
                let ry = row0_y + (1.25 * (i as f32)) * row_h;
                if pause_menu_role == i {
                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }
                let label = labels[i];
                if i != 2 {
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body + 4.0,
                        palette.text_primary,
                    );
                } else {
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body + 4.0,
                        Color::from_rgba(255, 180, 160, 255),
                    );
                }
            }
        }
        PauseMenuState::None => {}
    }
}