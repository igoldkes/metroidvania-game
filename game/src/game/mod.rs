mod player;
mod screens;
mod ui;
mod story;

use macroquad::prelude::*;
use macroquad::audio::{load_sound, play_sound, play_sound_once, stop_sound, set_sound_volume, PlaySoundParams, Sound};

use player::Player;
use screens::startup_ui::draw_startup_overlay;
use screens::overlays_ui::draw_pause_menu_overlay;
use story::StoryPhase;

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupState {
    Splash,
    MainMenu,
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PauseMenuState {
    None,
    Menu { pause_menu_role: usize },
}

pub struct GameState {
    startup_state: StartupState,
    pause_menu_state: PauseMenuState,
    player: Player,
    width: f32,
    height: f32,
    floor_y: f32,
    startup_menu_role: usize,
    pause_menu_role: usize,
    story: StoryPhase,
    paused: bool,
    // assets
    menu_click_sound: Sound,
    // settings toggles
    menu_clicks_settings_toggle: bool,

}

impl GameState {
    pub async fn new() -> Self {
        let width = screen_width();
        let height = screen_height();
        let floor_y = height - 100.0;

        let player = Player::new(width / 2.0, height / 2.0);

        let menu_click_sound = load_sound("assets/audio_assets/menu_click_sound.wav").await.unwrap();

        Self {
            startup_state: StartupState::Splash,
            pause_menu_state: PauseMenuState::None,
            player,
            width,
            height,
            floor_y,
            startup_menu_role: 0,
            pause_menu_role: 0,
            story: StoryPhase::new_game(),
            paused: false,
            menu_click_sound,
            menu_clicks_settings_toggle: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if matches!(self.story, StoryPhase::Playing) {
            self.player.update(self.width, self.height, self.floor_y, dt);

            if is_key_pressed(KeyCode::Escape) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                }
                self.paused = !self.paused;
                self.player.paused = self.paused;
                self.pause_menu_state = PauseMenuState::Menu { pause_menu_role: self.pause_menu_role };
            }
        }

        if !matches!(self.startup_state, StartupState::Done) {
            self.handle_startup_input();
            return;
        }
    }

    pub fn draw(&mut self) {
        if !matches!(self.startup_state, StartupState::Done) {
            draw_startup_overlay(
                &self.startup_state,
                self.startup_menu_role,
            );
            return;
        }

        if matches!(self.story, StoryPhase::Playing) {
            clear_background(DARKGRAY);

            self.player.draw();

            draw_rectangle(
                0.0,
                self.floor_y + 16.0,
                self.width,
                30.0,
                BROWN,
            );
        }

        if self.paused {
            draw_pause_menu_overlay( self.pause_menu_state.clone() );

            match self.pause_menu_state {
                PauseMenuState::Menu { .. } => {
                    self.pause_menu_state = PauseMenuState::Menu { pause_menu_role: self.pause_menu_role };
                }
                PauseMenuState::None => {}
            }

            if is_key_pressed(KeyCode::W) | is_key_pressed(KeyCode::Up) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                }
                if self.pause_menu_role == 0 {
                    self.pause_menu_role = 2;
                } else {
                    self.pause_menu_role -= 1;
                }
            }

            if is_key_pressed(KeyCode::S) | is_key_pressed(KeyCode::Down) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                }
                if self.pause_menu_role == 2 {
                    self.pause_menu_role = 0;
                } else {
                    self.pause_menu_role += 1;
                }
            }

            if is_key_pressed(KeyCode::Enter) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                }
                match self.pause_menu_role {
                    0 => {
                        // Resume game
                        self.paused = false;
                        self.player.paused = self.paused;
                    }
                    1 => {
                        // Return to main menu
                        self.startup_state = StartupState::MainMenu;
                        self.paused = false;
                        self.player.paused = self.paused;
                        self.pause_menu_state = PauseMenuState::None;
                        self.pause_menu_role = 0;
                        self.startup_menu_role = 0;
                    }
                    2 => {
                        // Exit to desktop
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_startup_input(&mut self) {
        match &self.startup_state {
            StartupState::Splash => {
                if get_last_key_pressed().is_some() {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                    }
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::MainMenu;
                }
            }
            StartupState::MainMenu => {
                if is_key_pressed(KeyCode::Up) | is_key_pressed(KeyCode::W) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                    }
                    if self.startup_menu_role == 0 {
                        self.startup_menu_role = 1;
                    } else {
                        self.startup_menu_role -= 1;
                    }
                }

                if is_key_pressed(KeyCode::S) | is_key_pressed(KeyCode::Down) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                    }
                    if self.startup_menu_role == 1 {
                        self.startup_menu_role = 0;
                    } else {
                        self.startup_menu_role += 1;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                    }
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::Splash;
                }

                if is_key_pressed(KeyCode::Enter) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                    }
                    match self.startup_menu_role {
                        0 => {
                            // Play
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::Done;
                            self.story = StoryPhase::Playing;
                        }
                        1 => {
                            // Exit game
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}