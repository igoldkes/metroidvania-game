use macroquad::prelude::*;

use super::super::ui::components::{draw_modal_chrome, draw_wrapped_text, ModalChromeProps};
use super::super::ui::layout::{centered_clamped_rect, safe_margins, scaled_type, ui_scale};
use super::super::ui::theme::{TypeScale, UiPreferences};
use super::super::{StartupState, AudioSettingsState};

#[allow(clippy::too_many_arguments)]
pub fn draw_startup_overlay(
    startup_state: &StartupState,
    menu_role: usize,
    dash_enabled: bool,
    sprint_enabled: bool,
    wall_jump_enabled: bool,
    double_jump_enabled: bool,
    menu_music_settings_toggle: bool,
    game_music_settings_toggle: bool,
    ambient_sounds_settings_toggle: bool,
    footsteps_settings_toggle: bool,
    menu_clicks_settings_toggle: bool,
    music_volume: usize,
    sfx_volume: usize,
    menu_clicks_volume: usize,
    controls_settings_row: usize,
    controls_settings_col: usize,
    move_left_kb: KeyCode,
    move_right_kb: KeyCode,
    look_up_kb: KeyCode,
    look_down_kb: KeyCode,
    jump_kb: KeyCode,
    dash_sprint_kb: KeyCode,
    melee_attack_kb: KeyCode,
    ranged_attack_kb: KeyCode,
    interact_kb: KeyCode,
    inventory_kb: KeyCode,
    awaiting_kb_input: bool,
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
        StartupState::Settings => 300.0,
        StartupState::GameSettings => 300.0,
        StartupState::AudioSettings { .. } => 460.0,
        StartupState::VideoSettings => 220.0,
        StartupState::ControlsSettings => 340.0,
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
            // Main Menu

            draw_text(
                "Main Menu",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );

            let row0_y = y + 92.0 * scale;

            const MAIN_MENU_OPTIONS: usize = 3;

            let labels: [&str; MAIN_MENU_OPTIONS] = [
                "Play",
                "Settings",
                "Exit Game",
            ];

            for i in 0..MAIN_MENU_OPTIONS {
                let ry = row0_y + i as f32 * row_h;

                // draw selected option highlight box
                if menu_role == i {

                    //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, row_bg_w, row_h);

                    // highlight box for menu_role 0: top-left: (278, 327), width: 724, height: 38
                    // highlight box for menu_role 1: top-left: (278, 365), width: 724, height: 38
                    // highlight box for menu_role 2: top-left: (278, 403), width: 724, height: 38

                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }
                let label = labels[i];
                if i != MAIN_MENU_OPTIONS - 1 {
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
        StartupState::Settings => {
            // Settings

            draw_text(
                "Settings",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );

            let row0_y = y + 92.0 * scale;

            const SETTINGS_OPTIONS: usize = 5;

            let labels: [&str; SETTINGS_OPTIONS] = [
                "Game",
                "Audio",
                "Video",
                "Controls",
                "Back",
            ];

            for i in 0..SETTINGS_OPTIONS {
                let ry = row0_y + i as f32 * row_h;

                // draw selected option highlight box
                if menu_role == i {

                    //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, row_bg_w, row_h);

                    // 

                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }

                let label = labels[i];
                if i != SETTINGS_OPTIONS - 1 {
                    // if not the Back option, draw text in primary text color
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                    );
                } else {
                    // if the Back option, draw text in red color
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
        StartupState::GameSettings => {
            draw_text(
                "Game",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );

            let row0_y = y + 92.0 * scale;

            const GAME_OPTIONS: usize = 5;

            let labels: [&str; GAME_OPTIONS] = [
                    "Dash",
                    "Sprint",
                    "Wall Jump",
                    "Double Jump",
                    "Back",
            ];

            for i in 0..GAME_OPTIONS {
                let ry = row0_y + i as f32 * row_h;

                // draw selected option highlight box
                if menu_role == i {

                    //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, row_bg_w, row_h);

                    //

                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }

                let label = labels[i];
                if i != GAME_OPTIONS - 1 {
                    // if not the Back option, draw text in primary text color
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                    );
                } else {
                    // if the Back option, draw text in red color
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        Color::from_rgba(255, 180, 160, 255),
                    );
                }
            }

            //let row0_y_opt = y + 92.0 * scale;
            let labels: [&str; GAME_OPTIONS - 1] = [
                if dash_enabled { "On" } else { "Off" },
                if sprint_enabled { "On" } else { "Off" },
                if wall_jump_enabled { "On" } else { "Off" },
                if double_jump_enabled { "On" } else { "Off" },
            ];

            for i in 0..GAME_OPTIONS - 1 {
                let ry = row0_y + i as f32 * row_h;

                let label = labels[i];

                if label == "On" {
                    draw_rectangle(
                        x + row_pad_x + 150.0 * scale,
                        ry - 11.0 * scale,
                        40.0 * scale,
                        row_h - 8.0,
                        Color::from_rgba(88, 94, 150, 235),
                    );
                    draw_text(
                        label,
                        x + row_pad_x + 160.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                        //Color::from_rgba(10, 163, 13, 1),
                    );
                } else {
                    draw_rectangle(
                        x + row_pad_x + 150.0 * scale,
                        ry - 11.0 * scale,
                        40.0 * scale,
                        row_h - 8.0,
                        Color::from_rgba(88, 94, 150, 235),
                    );
                    draw_text(
                        label,
                        x + row_pad_x + 157.5 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                        //Color::from_rgba(163, 10, 10, 1),
                    );
                }
            }
        }
        StartupState::AudioSettings { audio_settings_state } => {
            draw_text(
                "Audio",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );

            let row0_y = y + 92.0 * scale;

            const AUDIO_OPTIONS: usize = 9;

            let labels: [&str; AUDIO_OPTIONS] = [
                    "Menu Music",
                    "Game Music",
                    "Ambient Sounds",
                    "Footsteps",
                    "Menu Clicks",
                    "Music Volume",
                    "SFX Volume",
                    "Menu Clicks Volume",
                    "Back",
            ];
            
            for i in 0..AUDIO_OPTIONS {
                let ry = row0_y + i as f32 * row_h;

                // draw selected option highlight box
                if i == 5 {
                    if !matches!(audio_settings_state, AudioSettingsState::MusicVolume) {
                        if menu_role == i {
                            //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, 200.0 * scale, row_h);
                            draw_rectangle(
                                x + row_pad_x,
                                ry - 15.0 * scale,
                                200.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    } else {
                        if menu_role == i {
                            //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x + 210.0 * scale, ry - 15.0 * scale, row_bg_w - 210.0 * scale, row_h);
                            draw_rectangle(
                                x + row_pad_x + 210.0 * scale,
                                ry - 15.0 * scale,
                                row_bg_w - 210.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    }
                    //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x + 220.0 * scale, ry + (1.0 * scale), row_bg_w - 236.0 * scale, row_h / (8.0 * scale));
                    draw_rectangle(
                        x + row_pad_x + 220.0 * scale,
                        ry + (1.0 * scale),
                        row_bg_w - 236.0 * scale, // x + row_pad_x + 210.0 * scale,
                        row_h / (8.0 * scale),
                        Color::from_rgba(90, 115, 210, 255),
                    );
                    
                    //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x + 220.0 * scale + (row_bg_w - 236.0 * scale) / 10.0 * music_volume as f32 - music_volume as f32 * scale, ry - (6.0 * scale), 10.0 * scale, row_h / (2.0 * scale));
                    draw_rectangle(
                        x + row_pad_x + 220.0 * scale + (row_bg_w - 236.0 * scale) / 10.0 * music_volume as f32 - music_volume as f32 * scale,
                        ry - (6.0 * scale),
                        10.0 * scale,
                        row_h / (2.0 * scale),
                        Color::from_rgba(145, 150, 180, 255),
                    );
                }
                if i == 6 {
                    if !matches!(audio_settings_state, AudioSettingsState::SFXVolume) {
                        if menu_role == i {
                            draw_rectangle(
                                x + row_pad_x,
                                ry - 15.0 * scale,
                                200.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    } else {
                        if menu_role == i {
                            draw_rectangle(
                                x + row_pad_x + 210.0 * scale,
                                ry - 15.0 * scale,
                                row_bg_w - 210.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    }
                    draw_rectangle(
                        x + row_pad_x + 220.0 * scale,
                        ry + (1.0 * scale),
                        row_bg_w - 236.0 * scale, // x + row_pad_x + 210.0 * scale,
                        row_h / (8.0 * scale),
                        Color::from_rgba(90, 115, 210, 255),
                    );
                    draw_rectangle(
                        x + row_pad_x + 220.0 * scale + (row_bg_w - 236.0 * scale) / 10.0 * sfx_volume as f32 - sfx_volume as f32 * scale,
                        ry - (6.0 * scale),
                        10.0 * scale,
                        row_h / (2.0 * scale),
                        Color::from_rgba(145, 150, 180, 255),
                    );
                }
                if i == 7 {
                    if !matches!(audio_settings_state, AudioSettingsState::MenuClicksVolume) {
                        if menu_role == i {
                            draw_rectangle(
                                x + row_pad_x,
                                ry - 15.0 * scale,
                                200.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    } else {
                        if menu_role == i {
                            draw_rectangle(
                                x + row_pad_x + 210.0 * scale,
                                ry - 15.0 * scale,
                                row_bg_w - 210.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    }
                    draw_rectangle(
                        x + row_pad_x + 220.0 * scale,
                        ry + (1.0 * scale),
                        row_bg_w - 236.0 * scale, // x + row_pad_x + 210.0 * scale,
                        row_h / (8.0 * scale),
                        Color::from_rgba(90, 115, 210, 255),
                    );
                    draw_rectangle(
                        x + row_pad_x + 220.0 * scale + (row_bg_w - 236.0 * scale) / 10.0 * menu_clicks_volume as f32 - menu_clicks_volume as f32 * scale,
                        ry - (6.0 * scale),
                        10.0 * scale,
                        row_h / (2.0 * scale),
                        Color::from_rgba(145, 150, 180, 255),
                    );
                }
                if menu_role != 5 && menu_role != 6 && menu_role != 7 {
                    if menu_role == i {
                        //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, row_bg_w, row_h);

                        //

                        draw_rectangle(
                            x + row_pad_x,
                            ry - 15.0 * scale,
                            row_bg_w,
                            row_h,
                            Color::from_rgba(88, 94, 118, 235),
                        );
                    }
                }
                let label = labels[i];
                if i != AUDIO_OPTIONS - 1 {
                    // if not the Back option, draw text in primary text color
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                    );
                } else {
                    // if the Back option, draw text in red color
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        Color::from_rgba(255, 180, 160, 255),
                    );
                }
            }

            let labels: [&str; AUDIO_OPTIONS - 4] = [
                    if menu_music_settings_toggle { "On" } else { "Off" },
                    if game_music_settings_toggle { "On" } else { "Off" },
                    if ambient_sounds_settings_toggle { "On" } else { "Off" },
                    if footsteps_settings_toggle { "On" } else { "Off" },
                    if menu_clicks_settings_toggle { "On" } else { "Off" },
            ];
            for i in 0..AUDIO_OPTIONS - 4 {
                let ry = row0_y + i as f32 * row_h;

                let label = labels[i];

                if label == "On" {
                    draw_rectangle(
                        x + row_pad_x + 150.0 * scale,
                        ry - 11.0 * scale,
                        40.0 * scale,
                        row_h - 8.0,
                        Color::from_rgba(88, 94, 150, 235),
                    );
                    draw_text(
                        label,
                        x + row_pad_x + 160.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                        //Color::from_rgba(10, 163, 13, 1),
                    );
                } else {
                    draw_rectangle(
                        x + row_pad_x + 150.0 * scale,
                        ry - 11.0 * scale,
                        40.0 * scale,
                        row_h - 8.0,
                        Color::from_rgba(88, 94, 150, 235),
                    );
                    draw_text(
                        label,
                        x + row_pad_x + 157.5 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                        //Color::from_rgba(163, 10, 10, 1),
                    );
                }
            }
        }
        StartupState::VideoSettings => {
            draw_text(
                "Video",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );

            let row0_y = y + 92.0 * scale;

            const VIDEO_OPTIONS: usize = 3;

            let labels: [&str; VIDEO_OPTIONS] = [
                    "Resolution",
                    "Brightness",
                    "Back",
            ];

            for i in 0..VIDEO_OPTIONS {
                let ry = row0_y + i as f32 * row_h;

                // draw selected option highlight box
                if menu_role == i {

                    //println!("highlight box for menu_role {}: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, row_bg_w, row_h);

                    //

                    draw_rectangle(
                        x + row_pad_x,
                        ry - 15.0 * scale,
                        row_bg_w,
                        row_h,
                        Color::from_rgba(88, 94, 118, 235),
                    );
                }

                let label = labels[i];
                if i != VIDEO_OPTIONS - 1 {
                    // if not the Back option, draw text in primary text color
                    draw_text(
                        label,
                        x + row_pad_x + 10.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                    );
                } else {
                    // if the Back option, draw text in red color
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
        StartupState::ControlsSettings => {
            draw_text(
                "Controls - Keybinds",
                x + row_pad_x,
                y + row_h,
                ty.headline,
                palette.text_primary,
            );

            let row0_y = y + 92.0 * scale;

            const CONTROLS_OPTIONS_ROWS: usize = 5;

            let col_1_labels: [&str; CONTROLS_OPTIONS_ROWS] = [
                    "Move Left",
                    "Move Right",
                    "Look Up",
                    "Look Down",
                    "Jump",
            ];

            let col_2_labels: [&str; CONTROLS_OPTIONS_ROWS] = [
                    "Dash/Sprint",
                    "Melee Attack",
                    "Ranged Attack",
                    "Interact",
                    "Inventory",
            ];

            for i in 0..CONTROLS_OPTIONS_ROWS {
                let ry = row0_y + i as f32 * row_h;

                let labels = if controls_settings_col == 0 { col_1_labels } else { col_2_labels };

                // if not selecting the back option
                if controls_settings_col == 0 {
                    // if in the first column of options
                    let labels = col_1_labels;
                    if controls_settings_row == i {
                        // ith option in the first column is selected
                        //println!("highlight box for row {}, col 0: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x, ry - 15.0 * scale, row_bg_w / 2.0, row_h);
                        if !awaiting_kb_input {
                            draw_rectangle(
                                x + row_pad_x,
                                ry - 15.0 * scale,
                                row_bg_w / 2.0,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        } else {
                            draw_rectangle(
                                x + row_pad_x + 190.0 * scale,
                                ry - 15.0 * scale,
                                row_bg_w / 2.0 - 190.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    }
                } else {
                    // if in the second column of options
                    let labels = col_2_labels;
                    if controls_settings_row == i {
                        // ith option in the second column is selected
                        //println!("highlight box for row {}, col 1: top-left: ({}, {}), width: {}, height: {}", i, x + row_pad_x + row_bg_w / 2.0, ry - 15.0 * scale, row_bg_w / 2.0, row_h);
                        if !awaiting_kb_input {
                            draw_rectangle(
                                x + row_pad_x + row_bg_w / 2.0,
                                ry - 15.0 * scale,
                                row_bg_w / 2.0,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        } else {
                            draw_rectangle(
                                x + row_pad_x + row_bg_w / 2.0 + 190.0 * scale,
                                ry - 15.0 * scale,
                                row_bg_w / 2.0 - 190.0 * scale,
                                row_h,
                                Color::from_rgba(88, 94, 118, 235),
                            );
                        }
                    }
                }
                // draw keybind text
                let col_1_label = col_1_labels[i];
                let col_2_label = col_2_labels[i];
                draw_text(
                    col_1_label,
                    x + row_pad_x + 10.0 * scale,
                    ry + 8.0 * scale,
                    ty.body,
                    palette.text_primary,
                );
                draw_text(
                    col_2_label,
                    x + row_pad_x + 10.0 * scale + row_bg_w / 2.0,
                    ry + 8.0 * scale,
                    ty.body,
                    palette.text_primary,
                );

                // draw current keybinds
                let col_1_selected_kb_labels: [&str; CONTROLS_OPTIONS_ROWS] = [
                    &format!("{:?}", move_left_kb),
                    &format!("{:?}", move_right_kb),
                    &format!("{:?}", look_up_kb),
                    &format!("{:?}", look_down_kb),
                    &format!("{:?}", jump_kb),
                ];
                let col_2_selected_kb_labels: [&str; CONTROLS_OPTIONS_ROWS] = [
                    &format!("{:?}", dash_sprint_kb),
                    &format!("{:?}", melee_attack_kb),
                    &format!("{:?}", ranged_attack_kb),
                    &format!("{:?}", interact_kb),
                    &format!("{:?}", inventory_kb),
                ];
                let col_1_selected_kb_label = col_1_selected_kb_labels[i];
                let col_2_selected_kb_label = col_2_selected_kb_labels[i];
                if !awaiting_kb_input {
                    draw_text(
                        col_1_selected_kb_label,
                        x + row_pad_x + 10.0 * scale + 200.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                    );
                    draw_text(
                        col_2_selected_kb_label,
                        x + row_pad_x + 10.0 * scale + row_bg_w / 2.0 + 200.0 * scale,
                        ry + 8.0 * scale,
                        ty.body,
                        palette.text_primary,
                    );
                } else {
                    if controls_settings_row == i {
                        if controls_settings_col == 0 {
                            draw_text(
                                &format!("-{}-", col_1_selected_kb_label),
                                x + row_pad_x + 10.0 * scale + 200.0 * scale,
                                ry + 8.0 * scale,
                                ty.body,
                                Color::from_rgba(255, 180, 160, 255),
                            );
                            draw_text(
                                col_2_selected_kb_label,
                                x + row_pad_x + 10.0 * scale + row_bg_w / 2.0 + 200.0 * scale,
                                ry + 8.0 * scale,
                                ty.body,
                                palette.text_primary,
                            );
                        } else {
                            draw_text(
                                col_1_selected_kb_label,
                                x + row_pad_x + 10.0 * scale + 200.0 * scale,
                                ry + 8.0 * scale,
                                ty.body,
                                palette.text_primary,
                            );
                            draw_text(
                                &format!("-{}", col_2_selected_kb_label),
                                x + row_pad_x + 10.0 * scale + row_bg_w / 2.0 + 200.0 * scale,
                                ry + 8.0 * scale,
                                ty.body,
                                Color::from_rgba(255, 180, 160, 255),
                            );
                        }
                    } else {
                        draw_text(
                            col_1_selected_kb_label,
                            x + row_pad_x + 10.0 * scale + 200.0 * scale,
                            ry + 8.0 * scale,
                            ty.body,
                            palette.text_primary,
                        );
                        draw_text(
                            col_2_selected_kb_label,
                            x + row_pad_x + 10.0 * scale + row_bg_w / 2.0 + 200.0 * scale,
                            ry + 8.0 * scale,
                            ty.body,
                            palette.text_primary,
                        );
                    }
                }
            }

            let ry = row0_y + 5 as f32 * row_h;
            if controls_settings_row == 5 {
                draw_rectangle(
                    x + row_pad_x,
                    ry - 15.0 * scale,
                    row_bg_w,
                    row_h,
                    Color::from_rgba(88, 94, 118, 235),
                );
            }
            draw_text(
                "Back",
                x + row_pad_x + 10.0 * scale,
                ry + 8.0 * scale,
                ty.body,
                Color::from_rgba(255, 180, 160, 255),
            );
        }
        _ => {}
    }
}