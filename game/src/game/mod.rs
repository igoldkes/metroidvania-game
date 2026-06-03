mod player;
mod screens;
mod ui;
mod story;

use macroquad::prelude::*;

use player::Player;
use screens::startup_ui::draw_startup_overlay;
use story::StoryPhase;

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupState {
    Splash,
    MainMenu,
    Done,
}

pub struct GameState {
    startup_state: StartupState,
    player: Player,
    width: f32,
    height: f32,
    floor_y: f32,
    startup_menu_role: usize,
    story: StoryPhase,
}

impl GameState {
    pub fn new() -> Self {
        let width = screen_width();
        let height = screen_height();
        let floor_y = height - 100.0;

        let player = Player::new(width / 2.0, height / 2.0);

        Self {
            startup_state: StartupState::Splash,
            player,
            width,
            height,
            floor_y,
            startup_menu_role: 0,
            story: StoryPhase::new_game(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        if matches!(self.story, StoryPhase::Playing) {
            self.player.update(self.width, self.height, self.floor_y, dt);
        }

        if !matches!(self.startup_state, StartupState::Done) {
            self.handle_startup_input();
            return;
        }
    }

    pub fn draw(&self) {
        if !matches!(self.startup_state, StartupState::Done) {
            draw_startup_overlay(
                &self.startup_state,
                self.startup_menu_role,
                self.width,
                self.height,
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
    }

    fn handle_startup_input(&mut self) {
        match &self.startup_state {
            StartupState::Splash => {
                if get_last_key_pressed().is_some() {
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::MainMenu;
                }
            }
            StartupState::MainMenu => {
                if is_key_pressed(KeyCode::Up) | is_key_pressed(KeyCode::W) {
                    if self.startup_menu_role == 0 {
                        self.startup_menu_role = 1;
                    } else {
                        self.startup_menu_role -= 1;
                    }
                }

                if is_key_pressed(KeyCode::S) | is_key_pressed(KeyCode::Down) {
                    if self.startup_menu_role == 1 {
                        self.startup_menu_role = 0;
                    } else {
                        self.startup_menu_role += 1;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::Splash;
                }

                if is_key_pressed(KeyCode::Enter) {
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