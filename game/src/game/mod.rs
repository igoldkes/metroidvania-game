mod player;
mod screens;
mod ui;
mod story;
mod assets;

use macroquad::prelude::*;
use macroquad::audio::{load_sound, play_sound, play_sound_once, stop_sound, set_sound_volume, PlaySoundParams, Sound};

use std::collections::HashMap;

use player::Player;
use screens::startup_ui::draw_startup_overlay;
use screens::overlays_ui::draw_pause_menu_overlay;
use story::StoryPhase;

const TILE_SIZE: f32 = 32.0;

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

#[derive(Clone, Debug)]
enum Tile {
    None,
    BrownBrick,
    GrayBrick,
}

#[derive(Clone, Debug)]
pub struct Room {
    tiles: Vec<((i32, i32), Tile)>,
    tile_map: HashMap<(i32, i32), Tile>,
    width: i32,
    height: i32,
}

impl Room {
    pub fn load_room(path: &str) -> Self {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut tiles: Vec<((i32, i32), Tile)> = Vec::new();
        let mut tile_map: HashMap<(i32, i32), Tile> = HashMap::new();
        let room_file = std::fs::read_to_string(path).unwrap();
        for line in room_file.lines() {
            for c in line.chars() {
                let tile = match c {
                    '#' => Tile::GrayBrick,
                    '$' => Tile::BrownBrick,
                    _ => Tile::None,
                };
                tiles.push(((x, y), tile.clone()));
                tile_map.insert((x, y), tile.clone());
                x += 1;
            }
            x = 0;
            y += 1;
        }
        let height = y;
        let width = tiles.len() as i32 / (y + 1) - 1;

        Self { tiles, tile_map, width, height }
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x > self.width || y > self.height {
            return true;
        }
        if self.tile_map.get(&(x, y)).is_none() {
            eprintln!("is_solid called with ({}, {}) which is not in the map", x, y);
        }
        match *self.tile_map.get(&(x, y)).unwrap() {
            Tile::None => false,
            Tile::BrownBrick => true,
            Tile::GrayBrick => true,
        }
        //*self.tile_map.get(&(x, y)).unwrap_or(&false)
    }
}

pub struct GameState {
    startup_state: StartupState,
    pause_menu_state: PauseMenuState,
    player: Player,
    current_room: Room,
    cam: Camera2D,
    width: f32,
    height: f32,
    floor_y: f32,
    startup_menu_role: usize,
    pause_menu_role: usize,
    story: StoryPhase,
    paused: bool,
    // assets
    jackie_paper_right_texture: Texture2D,
    jackie_paper_left_texture: Texture2D,
    jackie_paper_up_right_texture: Texture2D,
    jackie_paper_up_left_texture: Texture2D,
    jackie_paper_down_right_texture: Texture2D,
    jackie_paper_down_left_texture: Texture2D,
    menu_click_sound: Sound,
    // settings toggles
    menu_clicks_settings_toggle: bool,

}

impl GameState {
    pub async fn new() -> Self {
        let width = screen_width();
        let height = screen_height();
        let floor_y = height - 48.0;

        let jackie_paper_right_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_right.png");
        let jackie_paper_left_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_left.png");
        let jackie_paper_up_right_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_up_right.png");
        let jackie_paper_up_left_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_up_left.png");
        let jackie_paper_down_right_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_down_right.png");
        let jackie_paper_down_left_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_down_left.png");

        let current_room = Room::load_room("assets/rooms/room3.txt");

        let player = Player::new(current_room.clone(), width / 2.0, height / 2.0, jackie_paper_right_texture.clone(), jackie_paper_left_texture.clone(), jackie_paper_up_right_texture.clone(), jackie_paper_up_left_texture.clone(), jackie_paper_down_right_texture.clone(), jackie_paper_down_left_texture.clone());

        let cam = Camera2D {
            target: vec2(width / 2.0, height / 2.0),
            zoom: vec2(2.0 / width, 2.0 / height),
            ..Default::default()
        };

        let menu_click_sound = load_sound("assets/audio_assets/menu_click_sound.wav").await.unwrap();

        Self {
            startup_state: StartupState::Splash,
            pause_menu_state: PauseMenuState::None,
            player,
            current_room,
            cam,
            width,
            height,
            floor_y,
            startup_menu_role: 0,
            pause_menu_role: 0,
            story: StoryPhase::new_game(),
            paused: false,
            jackie_paper_right_texture,
            jackie_paper_left_texture,
            jackie_paper_up_right_texture,
            jackie_paper_up_left_texture,
            jackie_paper_down_right_texture,
            jackie_paper_down_left_texture,
            menu_click_sound,
            menu_clicks_settings_toggle: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.width = screen_width();
        self.height = screen_height();

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

            let target = vec2(
                self.player.x + TILE_SIZE * self.player.pwidth / 2.0,
                self.player.y - TILE_SIZE * self.player.pheight / 2.0 - 100.0,
            );

            self.cam.target.x = self.cam.target.x.lerp(target.x, 0.1);
            self.cam.target.y = self.cam.target.y.lerp(target.y, 0.1);

            //let cam = Camera2D {
            //    target: vec2(
            //        self.player.x + TILE_SIZE * self.player.pwidth / 2.0,
            //        self.player.y - TILE_SIZE * self.player.pheight / 2.0,
            //    ),
            //    zoom: vec2(2.0 / self.width, 2.0 / self.height),
            //    ..Default::default()
            //};
            set_camera(&self.cam);

            self.player.draw();

            self.draw_tiles();

            //draw_rectangle(
            //    0.0,
            //    self.floor_y + 16.0,
            //    self.width,
            //    30.0,
            //    RED,
            //);
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

    fn draw_tiles(&self) {
        for tile in self.current_room.tiles.clone() {
            match tile.1 {
                Tile::BrownBrick => {
                    draw_rectangle(
                        tile.0.0 as f32 * TILE_SIZE,
                        tile.0.1 as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        BROWN,
                    );
                }
                Tile::GrayBrick => {
                    draw_rectangle(
                        tile.0.0 as f32 * TILE_SIZE,
                        tile.0.1 as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        GRAY,
                    );
                }
                Tile::None => {}
            }
        }
        //let size = self.current_room.tiles.len();
        //draw_text(
        //    &format!("{}", size),
        //    500.0,
        //    500.0,
        //    30.0,
        //    RED,
        //);
    }

    fn draw_checkered_background(&self) {
        for tile in self.current_room.tiles.clone() {
            todo!();
        }
    }
}