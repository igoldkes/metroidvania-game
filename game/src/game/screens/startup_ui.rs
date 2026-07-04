use macroquad::prelude::*;

use super::super::ui::components::{draw_modal_chrome, draw_wrapped_text, ModalChromeProps};
use super::super::ui::layout::{centered_clamped_rect, safe_margins, scaled_type, ui_scale};
use super::super::ui::theme::{TypeScale, UiPreferences};
use super::super::StartupState;

#[allow(clippy::too_many_arguments)]
pub fn draw_startup_overlay(
    startup_state: &StartupState,
    menu_role: usize,
) {
    let width = screen_width();
    let height = screen_height();
    
    draw_rectangle(0.0, 0.0, width, height, Color::from_rgba(0, 0, 0, 180));

    let prefs = UiPreferences::default();
    let palette = prefs.palette();
    let scale = ui_scale();
    let margins = safe_margins(scale);
    let ty = scaled_type(&TypeScale::default(), scale);

    let preferred_height = match startup_state {
        StartupState::Splash => 180.0,
        StartupState::MainMenu => 220.0,
        _ => 220.0,
    };

    let pw = 760.0 * scale;
    let ph = preferred_height * scale;
    let rect = centered_clamped_rect(pw, ph, margins);
    let x = rect.x;
    let y = rect.y;

    let semantic_id = match startup_state {
        StartupState::Splash => "splash",
        StartupState::MainMenu => "main_menu",
        _ => "unknown_state",
    };

    draw_modal_chrome(&ModalChromeProps {
        rect,
        title: None,
        palette,
        focused: true,
        semantic_id,
    });

    let row_h = 38.0 * scale;
    let row_pad_x = 18.0 * scale;
    let row_bg_w = rect.w - row_pad_x * 2.0;

    match startup_state {
        StartupState::Splash => {
            draw_text(
                "Game",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );
        }
        StartupState::MainMenu => {
            draw_text(
                "Main Menu",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );

            let row0_y = y + 92.0 * scale;

            let labels: [&str; 2] = [
                "Play",
                "Exit Game",
            ];

            for i in 0..2 {
                let ry = row0_y + i as f32 * row_h;
                if menu_role == i {
                    //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, row_bg_w, row_h);
                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }
                let label = labels[i];
                if i != 1 {
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                    );
                } else {
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        Color::from_rgba(255, 180, 160, 255),
                    );
                }
            }
        }
        _ => {}
    }
}