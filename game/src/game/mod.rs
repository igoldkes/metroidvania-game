mod player;
mod screens;
mod ui;
mod story;
mod assets;

use macroquad::prelude::*;
use macroquad::audio::{load_sound, play_sound, play_sound_once, stop_sound, set_sound_volume, PlaySoundParams, Sound};
use macroquad::experimental::animation::{AnimatedSprite, Animation};

use std::collections::HashMap;

use player::{Player, RoomChange};
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
    Door { identifier: char },
    Spikes,
}

#[derive(Clone, Debug)]
struct Door {
    pub room_path: String,
    pub spawn_x: i32,
    pub spawn_y: i32,
}

impl Door {
    fn new_door(room_path: &str, spawn_x: i32, spawn_y: i32) -> Self {
        Self {
            room_path: String::from(room_path),
            spawn_x,
            spawn_y,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Room {
    tiles: Vec<((i32, i32), Tile)>,
    tile_map: HashMap<(i32, i32), Tile>,
    width: i32,
    height: i32,
    door_map: HashMap<char, Door>,
}

impl Room {
    pub fn load_room(path: &str) -> Self {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut tiles: Vec<((i32, i32), Tile)> = Vec::new();
        let mut tile_map: HashMap<(i32, i32), Tile> = HashMap::new();
        let room_file = std::fs::read_to_string(path).unwrap();
        let mut lines = room_file.lines();
        let line1 = lines.next().unwrap();
        let line1_parts: Vec<&str> = line1.split(',').collect();
        let string = format!("{:?}", line1_parts.clone());
        //println!("{}", string);
        let width: i32 = line1_parts[0].parse().unwrap();
        let height: i32 = line1_parts[1].parse().unwrap();
        let line2 = lines.next().unwrap();
        let door_map: HashMap<char, Door> = door_parser(line2);
        for line in lines {
            for c in line.chars() {
                let tile = match c {
                    '#' => Tile::GrayBrick,
                    '$' => Tile::BrownBrick,
                    '^' => Tile::Spikes,
                    _ if door_map.contains_key(&c) => Tile::Door { identifier: c },
                    _ => Tile::None,
                };
                tiles.push(((x, y), tile.clone()));
                tile_map.insert((x, y), tile.clone());
                x += 1;
            }
            x = 0;
            y += 1;
        }
        let height1 = y;
        let width1 = tiles.len() as i32 / height1;

        //println!("width: {}, height: {}", width, height);
        //println!("width1: {}, height1: {}", width1, height1);

        Self { tiles, tile_map, width, height, door_map }
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x > self.width || y > self.height {
            return false;
        }
        if self.tile_map.get(&(x, y)).is_none() {
            eprintln!("is_solid called with ({}, {}) which is not in the map", x, y);
            return false;
        }
        match *self.tile_map.get(&(x, y)).unwrap() {
            Tile::None => false,
            Tile::BrownBrick => true,
            Tile::GrayBrick => true,
            Tile::Door { .. } => false,
            Tile::Spikes => true,
        }
        //*self.tile_map.get(&(x, y)).unwrap_or(&false)
    }

    pub fn is_door(&self, x: i32, y: i32) -> bool {
        match *self.tile_map.get(&(x, y)).unwrap() {
            Tile::Door { .. } => true,
            _ => false,
        }
    }

    pub fn is_spikes(&self, x: i32, y: i32) -> bool {
        match *self.tile_map.get(&(x, y)).unwrap() {
            Tile::Spikes => true,
            _ => false,
        }
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
    player_lives: usize,
    mouse_moved_buffer: f32,
    // assets
    jackie_paper_right_texture: Texture2D,
    jackie_paper_left_texture: Texture2D,
    jackie_paper_up_right_texture: Texture2D,
    jackie_paper_up_left_texture: Texture2D,
    jackie_paper_down_right_texture: Texture2D,
    jackie_paper_down_left_texture: Texture2D,
    jackie_paper_walking_texture: Texture2D,
    jackie_paper_walking_animation: AnimatedSprite,
    player_life_texture: Texture2D,
    background_texture: Texture2D,
    show_background: bool,
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
        let player_life_texture =  assets::load_player_life_texture("assets/graphics_assets/player_life.png");
        let background_texture = assets::load_background_texture("assets/graphics_assets/background.png");

        let jackie_paper_walking_texture = assets::load_jackie_paper_texture("assets/graphics_assets/sprite_sheet_walking.png");
        jackie_paper_walking_texture.set_filter(FilterMode::Linear);

        build_textures_atlas();

        let mut jackie_paper_walking_animation = AnimatedSprite::new(
            1179,
            1577,
            &[
                Animation {
                    name: "walking_left".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 4,
                },
                Animation {
                    name: "walking_right".to_string(),
                    row: 1,
                    frames: 2,
                    fps: 4,
                },
                Animation {
                    name: "jumping_left".to_string(),
                    row: 2,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "jumping_right".to_string(),
                    row: 3,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "falling_left".to_string(),
                    row: 4,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "falling_right".to_string(),
                    row: 5,
                    frames: 1,
                    fps: 1,
                },
            ],
            true,
        );

        let current_room = Room::load_room("assets/rooms/room3.txt");

        let player = Player::new(current_room.clone(), width / 2.0, height / 2.0, jackie_paper_right_texture.clone(), jackie_paper_left_texture.clone(), jackie_paper_up_right_texture.clone(), jackie_paper_up_left_texture.clone(), jackie_paper_down_right_texture.clone(), jackie_paper_down_left_texture.clone(), jackie_paper_walking_texture.clone(), jackie_paper_walking_animation.clone());

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
            player_lives: 5,
            mouse_moved_buffer: 0.0,
            jackie_paper_right_texture,
            jackie_paper_left_texture,
            jackie_paper_up_right_texture,
            jackie_paper_up_left_texture,
            jackie_paper_down_right_texture,
            jackie_paper_down_left_texture,
            jackie_paper_walking_texture,
            jackie_paper_walking_animation,
            player_life_texture,
            background_texture,
            show_background: false,
            menu_click_sound,
            menu_clicks_settings_toggle: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.width = screen_width();
        self.height = screen_height();

        if mouse_delta_position() != (0.0, 0.0).into() {
            self.mouse_moved_buffer = 0.15;
        }

        if self.mouse_moved_buffer > 0.0 {
            self.mouse_moved_buffer -= dt;
        }

        if matches!(self.story, StoryPhase::Playing) {
            self.player.update(self.width, self.height, self.floor_y, dt);

            if self.player_lives > self.player.lives {
                // player just lost a life
                self.player_lives = self.player.lives;
            } else if self.player_lives < self.player.lives {
                // player just gained a life
                self.player_lives = self.player.lives;
            }

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

        if let RoomChange::Change { door } = &self.player.room_change {
            self.change_room(&door.clone());
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
            clear_background(BLACK);

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

            if self.show_background {
                draw_texture_ex(
                    &self.background_texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(self.current_room.width as f32 * TILE_SIZE, self.current_room.height as f32 * TILE_SIZE)),
                        ..Default::default()
                    },
                );
            }

            self.player.draw2();

            self.draw_tiles();

            set_default_camera();

            self.draw_player_lives();

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

            if self.mouse_moved_buffer > 0.0 {
                let mouse_pos = mouse_position();
                if mouse_pos.0 > 338.0 && mouse_pos.0 < 942.0 {
                    if mouse_pos.1 > 294.5 && mouse_pos.1 < 342.0 {
                        self.pause_menu_role = 0;
                    } else if mouse_pos.1 > 342.0 && mouse_pos.1 < 389.5 {
                        self.pause_menu_role = 1;
                    } else if mouse_pos.1 > 389.5 && mouse_pos.1 < 427.5 {
                        self.pause_menu_role = 2;
                    }
                }
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

            if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
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
                if get_last_key_pressed().is_some() || is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                    }
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::MainMenu;
                }
            }
            StartupState::MainMenu => {
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                        if mouse_pos.1 > 327.0 && mouse_pos.1 < 365.0 {
                            self.startup_menu_role = 0;
                        } else if mouse_pos.1 > 365.0 && mouse_pos.1 < 403.0 {
                            self.startup_menu_role = 1;
                        }
                    }
                }
                

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

                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
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
                Tile::Spikes => {
                    draw_rectangle(
                        tile.0.0 as f32 * TILE_SIZE,
                        tile.0.1 as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        RED,
                    );
                }
                Tile::Door { .. } => {
                    //draw_rectangle(
                    //    tile.0.0 as f32 * TILE_SIZE,
                    //    tile.0.1 as f32 * TILE_SIZE,
                    //    TILE_SIZE,
                    //    TILE_SIZE,
                    //    BLACK,
                    //);
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

    fn change_room(&mut self, door: &Door) {
        self.player.room_change = RoomChange::None;
        let room_file = &door.room_path;
        let new_room = Room::load_room(room_file);
        self.current_room = new_room.clone();
        self.player.current_room = new_room;
        self.player.x = door.spawn_x as f32 * TILE_SIZE;
        self.player.y = door.spawn_y as f32 * TILE_SIZE;
        self.player.movement_blocked_buffer = 0.7;
    }

    fn draw_player_lives(&self) {
        for i in 0..self.player_lives {
            draw_texture_ex(
                &self.player_life_texture,
                self.width - 70.0 - i as f32 * 50.0,
                20.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(45.0, 45.0)),
                    ..Default::default()
                },
            )
        }
    }
}

fn door_parser(line: &str) -> HashMap<char, Door> {
    //todo!("parse a line from a room file: split on tabs to separate doors, split on commas to separate each door into 1) identifier 2) file path for connected room 3) x coordinate in tiles for the player's spawnpoint in the new room 4) y coordinate in tiles for the player's spawnpoint in the new room")
    let mut door_map: HashMap<char, Door> = HashMap::new();
    let doors: Vec<&str> = line.split('~').collect();
    for door in doors {
        let door_parts: Vec<&str> = door.split(',').collect();
        let string = format!("{:#?}", door_parts.clone());
        //println!("{}", string);
        let identifier: char = door_parts[0].chars().next().unwrap();
        let room_path = door_parts[1];
        let spawn_x: i32 = door_parts[2].parse().unwrap();
        let spawn_y: i32 = door_parts[3].parse().unwrap();
        let new_door = Door::new_door(room_path, spawn_x, spawn_y);
        door_map.insert(identifier, new_door);
    }
    door_map
}