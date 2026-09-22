mod player;
mod enemy;
mod screens;
mod ui;
mod story;
mod assets;

use macroquad::prelude::*;
use macroquad::audio::{load_sound, play_sound, play_sound_once, stop_sound, set_sound_volume, PlaySoundParams, Sound};
use macroquad::experimental::animation::{AnimatedSprite, Animation};

use std::collections::HashMap;

use crate::game::player::AttackDirection;

use player::{Player, RoomChange};
use enemy::{Enemy, EnemyType};
use screens::startup_ui::draw_startup_overlay;
use screens::overlays_ui::{draw_pause_menu_overlay, draw_inventory_overlay};
use story::StoryPhase;

const TILE_SIZE: f32 = 32.0;

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupState {
    // initial title splash screen
    Splash,
    // main menu
    MainMenu,
    // save files page
    //Saves,
    // general settings page
    Settings,
    // game settings page
    GameSettings,
    // audio settings page
    AudioSettings { audio_settings_state: AudioSettingsState },
    // video settings page
    VideoSettings,
    // controls settings page
    ControlsSettings,
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AudioSettingsState {
    Standard,
    MusicVolume,
    SFXVolume,
    MenuClicksVolume,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PauseMenuState {
    None,
    Menu,
    Settings,
    AudioSettings { audio_settings_state: AudioSettingsState },
    GameSettings,
    VideoSettings,
    ControlsSettings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InventoryState {
    Closed { last_inventory_page: InventoryPage },
    Open { inventory_page: InventoryPage },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InventoryPage {
    Items,
    Equipment,
    Map,
    EnemyLog,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Keybind {
    None,
    MoveLeft,
    MoveRight,
    LookUp,
    LookDown,
    Jump,
    DashSprint,
    MeleeAttack,
    RangedAttack,
    Interact,
    Inventory,
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
    enemies: Vec<Enemy>,
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

        let mut enemies: Vec<Enemy> = Vec::new();
        if line1_parts.len() > 2 {
            // room has enemies
            let enemy_line = line1_parts[2];
            let enemy_line_parts: Vec<&str> = enemy_line.split('~').collect();
            for enemy in enemy_line_parts {
                let enemy_data: Vec<&str> = enemy.split('/').collect();
                let enemy_type = match enemy_data[0] {
                    "test" => EnemyType::TestEnemy,
                    &_ => todo!(),
                };
                let enemy_x: f32 = enemy_data[1].parse().unwrap();
                let enemy_y: f32 = enemy_data[2].parse().unwrap();
                let new_enemy = Enemy::new(enemy_type, enemy_x * TILE_SIZE, enemy_y * TILE_SIZE);
                //println!("new enemy");
                enemies.push(new_enemy);
            }
        }

        //println!("width: {}, height: {}", width, height);
        //println!("width1: {}, height1: {}", width1, height1);

        Self { tiles, tile_map, width, height, door_map, enemies }
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
    pause_menu_controls_settings_row: usize,
    pause_menu_controls_settings_col: usize,
    story: StoryPhase,
    paused: bool,
    player_lives: usize,
    mouse_moved_buffer: f32,
    // assets
    //jackie_paper_right_texture: Texture2D,
    //jackie_paper_left_texture: Texture2D,
    //jackie_paper_up_right_texture: Texture2D,
    //jackie_paper_up_left_texture: Texture2D,
    //jackie_paper_down_right_texture: Texture2D,
    //jackie_paper_down_left_texture: Texture2D,
    jackie_paper_walking_texture: Texture2D,
    jackie_paper_walking_animation: AnimatedSprite,
    player_life_texture: Texture2D,
    background_texture: Texture2D,
    show_background: bool,
    menu_click_sound: Sound,
    // settings toggle
    menu_music_settings_toggle: bool,
    game_music_settings_toggle: bool,
    ambient_sounds_settings_toggle: bool,
    footsteps_settings_toggle: bool,
    menu_clicks_settings_toggle: bool,
    music_volume: usize,
    sfx_volume: usize,
    menu_clicks_volume: usize,
    potential_volume_lvl: usize,
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
    kb_map: HashMap<KeyCode, Keybind>,
    awaiting_kb_input: bool,
    kb_to_change: Keybind,
    inventory_state: InventoryState,
}

impl GameState {
    pub async fn new() -> Self {
        let width = screen_width();
        let height = screen_height();
        let floor_y = height - 48.0;

        //let jackie_paper_right_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_right.png");
        //let jackie_paper_left_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_left.png");
        //let jackie_paper_up_right_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_up_right.png");
        //let jackie_paper_up_left_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_up_left.png");
        //let jackie_paper_down_right_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_down_right.png");
        //let jackie_paper_down_left_texture = assets::load_jackie_paper_texture("assets/graphics_assets/jackie_paper_down_left.png");
        let player_life_texture =  assets::load_player_life_texture("assets/graphics_assets/player_life.png");
        let background_texture = assets::load_background_texture("assets/graphics_assets/background.png");

        let jackie_paper_walking_texture = assets::load_jackie_paper_texture("assets/graphics_assets/sprite_sheet_walking.png");
        jackie_paper_walking_texture.set_filter(FilterMode::Linear);

        build_textures_atlas();

        let mut jackie_paper_walking_animation = AnimatedSprite::new(
            96,
            128,
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

        let player = Player::new(current_room.clone(), width / 2.0, height / 2.0, jackie_paper_walking_texture.clone(), jackie_paper_walking_animation.clone());

        let cam = Camera2D {
            target: vec2(width / 2.0, height / 2.0),
            zoom: vec2(2.0 / width, 2.0 / height),
            ..Default::default()
        };

        let menu_click_sound = load_sound("assets/audio_assets/menu_click_sound.wav").await.unwrap();

        let mut kb_map: HashMap<KeyCode, Keybind> = HashMap::new();
        kb_map.insert(KeyCode::A, Keybind::MoveLeft);
        kb_map.insert(KeyCode::D, Keybind::MoveRight);
        kb_map.insert(KeyCode::W, Keybind::LookUp);
        kb_map.insert(KeyCode::S, Keybind::LookDown);
        kb_map.insert(KeyCode::Space, Keybind::Jump);
        kb_map.insert(KeyCode::LeftShift, Keybind::DashSprint);
        kb_map.insert(KeyCode::Semicolon, Keybind::MeleeAttack);
        kb_map.insert(KeyCode::Apostrophe, Keybind::RangedAttack);
        kb_map.insert(KeyCode::E, Keybind::Interact);
        kb_map.insert(KeyCode::T, Keybind::Inventory);

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
            pause_menu_controls_settings_row: 0,
            pause_menu_controls_settings_col: 0,
            story: StoryPhase::new_game(),
            paused: false,
            player_lives: 5,
            mouse_moved_buffer: 0.0,
            //jackie_paper_right_texture,
            //jackie_paper_left_texture,
            //jackie_paper_up_right_texture,
            //jackie_paper_up_left_texture,
            //jackie_paper_down_right_texture,
            //jackie_paper_down_left_texture,
            jackie_paper_walking_texture,
            jackie_paper_walking_animation,
            player_life_texture,
            background_texture,
            show_background: false,
            menu_click_sound,
            menu_music_settings_toggle: true,
            game_music_settings_toggle: true,
            ambient_sounds_settings_toggle: true,
            footsteps_settings_toggle: true,
            menu_clicks_settings_toggle: true,
            music_volume: 10,
            sfx_volume: 10,
            menu_clicks_volume: 10,
            potential_volume_lvl: 0,
            controls_settings_row: 0,
            controls_settings_col: 0,
            move_left_kb: KeyCode::A,
            move_right_kb: KeyCode::D,
            look_up_kb: KeyCode::W,
            look_down_kb: KeyCode::S,
            jump_kb: KeyCode::Space,
            dash_sprint_kb: KeyCode::LeftShift,
            melee_attack_kb: KeyCode::Semicolon,
            ranged_attack_kb: KeyCode::Apostrophe,
            interact_kb: KeyCode::E,
            inventory_kb: KeyCode::T,
            kb_map,
            awaiting_kb_input: false,
            kb_to_change: Keybind::None,
            inventory_state: InventoryState::Closed { last_inventory_page: InventoryPage::Items },
        }
    }

    pub fn update(&mut self, dt: f32) {
        //println!("{:?}, {}", self.pause_menu_state, self.paused);
        //println!("{:?}", KeyCode::LeftShift);
        //println!("{:?}", self.startup_state);

        // screen size checks
        self.width = screen_width();
        self.height = screen_height();

        // mouse movement checks
        if mouse_delta_position() != (0.0, 0.0).into() {
            // if mouse's position has moved at all, set mouse movement buffer to 0.15s
            self.mouse_moved_buffer = 0.15;
        }
        if self.mouse_moved_buffer > 0.0 {
            // if mouse movement buffer is greater than 0.0, decrement it
            self.mouse_moved_buffer -= dt;
        }

        if !matches!(self.startup_state, StartupState::Done) {
            self.handle_startup_input();
            return;
        }

        
        // STORYPHASE::PLAYING
        if matches!(self.story, StoryPhase::Playing) {
            // update player
            let inventory_open = if matches!(self.inventory_state, InventoryState::Open { .. }) { true } else { false };
            self.player.inventory_open = inventory_open;
            self.player.update(self.width, self.height, self.floor_y, dt, self.move_left_kb, self.move_right_kb, self.look_up_kb, self.look_down_kb, self.jump_kb, self.dash_sprint_kb, self.melee_attack_kb, self.ranged_attack_kb, self.interact_kb, self.inventory_kb);

            // enemy interactions
            if self.current_room.enemies.len() > 0 {
                // if there are not 0 enemies in the room, check enemy interactions
                let mut enemies = std::mem::take(&mut self.current_room.enemies);

                for enemy in &mut enemies {
                    // update enemy
                    enemy.update(dt, self.current_room.tile_map.clone(), self.current_room.width, self.current_room.height);

                    // for each enemy in the current room's vector of enemies...
                    if enemy.alive {
                        // if enemy is alive, check interactions
                        if !self.player.is_attacking {
                            // if player is not attacking, enemy is not being hit
                            enemy.being_hit = false;
                        }

                        // check enemy contact damage
                        if self.resolve_enemy_collisions(&enemy) && self.player.damage_blocked_buffer <= 0.0 {
                            // if player hitbox is overlapping with enemy hitbox and player can take damage...

                            // apply damage to player and make player immune to damage for 2.0s
                            self.player.lives -= 1;
                            self.player.damage_blocked_buffer = 2.0;

                            // apply knockback to player
                            let knockback_speed = 300.0;
                            let knockback_y = -150.0;
                            let knockback_direction = if self.player.x <= enemy.x {
                                // if player is to the left of enemy, knock back player leftward; knockback_vel_x < 0.0
                                -1.0
                            } else {
                                // if player is to the right of enemy, knock back player rightward; knockback_vel_x > 0.0
                                1.0
                            };
                            self.player.knockback_vel_x = knockback_speed * knockback_direction;
                            self.player.knockback_vel_y = knockback_y;
                        }

                        // check player attacks against enemy
                        if self.resolve_player_attack_collisions(&enemy) {
                            // if player attack hitbox is overlapping with enemy hitbox, then check if enemy has already been damaged by current attack
                            if !enemy.being_hit {
                                // if enemy has not been hit by current attack...

                                // apply damage to enemy
                                enemy.health -= 10.0;
                                enemy.being_hit = true;

                                // apply knockback to enemy and recoil knockback to player
                                let knockback_speed = 300.0;
                                let knockback_y = -150.0;
                                let recoil_knockback_speed = 125.0;
                                match self.player.attack_direction {
                                    AttackDirection::Right => {
                                        enemy.knockback_vel_x = knockback_speed;
                                        enemy.knockback_vel_y = knockback_y;

                                        self.player.knockback_vel_x = -recoil_knockback_speed;
                                    }
                                    AttackDirection::Left => {
                                        enemy.knockback_vel_x = -knockback_speed;
                                        enemy.knockback_vel_y = knockback_y;

                                        self.player.knockback_vel_x = recoil_knockback_speed;
                                    }
                                    AttackDirection::Up => {
                                        enemy.knockback_vel_x = 0.0;
                                        enemy.knockback_vel_y = -knockback_speed;
                                    }
                                    AttackDirection::Down => {
                                        enemy.knockback_vel_x = 0.0;
                                        enemy.knockback_vel_y = knockback_speed;

                                        // pogo player up off of enemy
                                        self.player.vel_y = -500.0;
                                        self.player.is_jumping = true;
                                        self.player.on_ground = false;
                                    }
                                }
                            }
                        }
                    }
                }

                self.current_room.enemies = enemies;
            }
            // enemy interactions done

            // player health updates
            if self.player_lives > self.player.lives {
                // if player_lives is greater than player's lives, then player has lost a life
                self.player_lives = self.player.lives;
            } else if self.player_lives < self.player.lives {
                // if player_lives is less than player's lives, then player has gained a life
                self.player_lives = self.player.lives;
            }
            // player health updates done

            // in-game menu escape checks
            if is_key_pressed(KeyCode::Escape) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                }
                match self.pause_menu_state {
                    PauseMenuState::None => {
                        match &self.inventory_state {
                            InventoryState::Closed { .. } => {
                                self.paused = true;
                                self.player.paused = self.paused;
                                self.pause_menu_role = 0;
                                self.pause_menu_state = PauseMenuState::Menu;
                            }
                            InventoryState::Open { inventory_page } => {
                                self.inventory_state = InventoryState::Closed { last_inventory_page: inventory_page.clone() };
                            }
                        }
                        
                    }
                    PauseMenuState::Menu => {
                        self.pause_menu_role = 0;
                        self.paused = false;
                        self.player.paused = self.paused;
                        self.pause_menu_state = PauseMenuState::None;
                    }
                    PauseMenuState::Settings => {
                        self.pause_menu_role = 1;
                        self.pause_menu_state = PauseMenuState::Menu;
                    }
                    PauseMenuState::GameSettings => {
                        self.pause_menu_role = 0;
                        self.pause_menu_state = PauseMenuState::Settings;
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard } => {
                        self.pause_menu_role = 1;
                        self.pause_menu_state = PauseMenuState::Settings;
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::MusicVolume } => {
                        self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::SFXVolume } => {
                        self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::MenuClicksVolume } => {
                        self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                    }
                    PauseMenuState::VideoSettings => {
                        self.pause_menu_role = 2;
                        self.pause_menu_state = PauseMenuState::Settings;
                    }
                    PauseMenuState::ControlsSettings => {
                        self.pause_menu_role = 3;
                        self.pause_menu_state = PauseMenuState::Settings;
                        self.pause_menu_controls_settings_row = 0;
                        self.pause_menu_controls_settings_row = 0;
                        self.kb_to_change = Keybind::None;
                    }
                }
            }
            // in-game menu escape checks done

            // in-game menu
            if self.paused {
                match self.pause_menu_state {
                    PauseMenuState::Menu => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            if mouse_pos.0 > 338.0 && mouse_pos.0 < 942.0 {
                                if mouse_pos.1 > 307.0 && mouse_pos.1 < 345.0 {
                                    self.pause_menu_role = 0;
                                } else if mouse_pos.1 > 345.0 && mouse_pos.1 < 383.0 {
                                    self.pause_menu_role = 1;
                                } else if mouse_pos.1 > 383.0 && mouse_pos.1 < 421.0 {
                                    self.pause_menu_role = 2;
                                } else if mouse_pos.1 > 421.0 && mouse_pos.1 < 459.0 {
                                    self.pause_menu_role = 3;
                                }
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.look_up_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 0 {
                                self.pause_menu_role = 3;
                            } else {
                                self.pause_menu_role -= 1;
                            }
                        }
                        if is_key_pressed(self.look_down_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 3 {
                                self.pause_menu_role = 0;
                            } else {
                                self.pause_menu_role += 1;
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.pause_menu_role = 0;
                            self.paused = false;
                            self.player.paused = self.paused;
                        }*/
                        // keyboard navigation done

                        // options handling
                        if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                            self.click_sound();
                            match self.pause_menu_role {
                                0 => {
                                    // Resume
                                    self.paused = false;
                                    self.player.paused = self.paused;
                                    self.pause_menu_state = PauseMenuState::None;
                                }
                                1 => {
                                    // Settings
                                    self.pause_menu_state = PauseMenuState::Settings;
                                    self.pause_menu_role = 0;
                                }
                                2 => {
                                    // Main Menu
                                    self.startup_state = StartupState::MainMenu;
                                    self.pause_menu_state = PauseMenuState::None;
                                    self.pause_menu_role = 0;
                                    self.startup_menu_role = 0;
                                    self.paused = false;
                                    self.player.paused = self.paused;
                                    self.story = StoryPhase::NotPlaying;
                                    //println!("{:?}", self.startup_state);
                                    //println!("{:?}", self.story);
                                }
                                3 => {
                                    // Exit to Desktop
                                    std::process::exit(0);
                                }
                                _ => {}
                            }
                        }
                        // options handling done
                    }
                    PauseMenuState::Settings => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            let mouse_x = mouse_pos.0;
                            let mouse_y = mouse_pos.1;

                            // check mouse's x position
                            if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                                // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                                if mouse_pos.1 > 287.0 && mouse_pos.1 < 325.0 {
                                    // if mouse's y position is between 287.0 and 325.0, then it is on menu option 0; menu_role = 0
                                    self.pause_menu_role = 0;
                                } else if mouse_pos.1 > 325.0 && mouse_pos.1 < 363.0 {
                                    // if mouse's y position is between 325.0 and 363.0, then it is on menu option 1; menu_role = 1
                                    self.pause_menu_role = 1;
                                } else if mouse_pos.1 > 363.0 && mouse_pos.1 < 401.0 {
                                    // if mouse's y position is between 363.0 and 401.0, then it is on menu option 2; menu_role = 2
                                    self.pause_menu_role = 2;
                                } else if mouse_pos.1 > 401.0 && mouse_pos.1 < 439.0 {
                                    // if mouse's y position is between 401.0 and 439.0, then it is on menu option 3; menu_role = 3
                                    self.pause_menu_role = 3;
                                } else if mouse_pos.1 > 439.0 && mouse_pos.1 < 477.0 {
                                    // if mouse's y position is between 439.0 and 477.0, then it is on menu option 4; menu_role = 4
                                    self.pause_menu_role = 4;
                                }
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.look_up_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 0 {
                                self.pause_menu_role = 4;
                            } else {
                                self.pause_menu_role -= 1;
                            }
                        }
                        if is_key_pressed(self.look_down_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 4 {
                                self.pause_menu_role = 0;
                            } else {
                                self.pause_menu_role += 1;
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.click_sound();
                            self.pause_menu_role = 1;
                            self.pause_menu_state = PauseMenuState::Menu;
                        }*/
                        // keyboard navigation done

                        // options handling
                        if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                            self.click_sound();
                            match self.pause_menu_role {
                                0 => {
                                    // Game Settings
                                    self.pause_menu_role = 0;
                                    self.pause_menu_state = PauseMenuState::GameSettings;
                                }
                                1 => {
                                    // Audio Settings
                                    self.pause_menu_role = 0;
                                    self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                                }
                                2 => {
                                    // Video Settings
                                    self.pause_menu_role = 0;
                                    self.pause_menu_state = PauseMenuState::VideoSettings;
                                }
                                3 => {
                                    // Controls Settings
                                    self.pause_menu_state = PauseMenuState::ControlsSettings;
                                }
                                4 => {
                                    // Back
                                    self.pause_menu_role = 1;
                                    self.pause_menu_state = PauseMenuState::Menu;
                                }
                                _ => {}
                            }
                        }
                        // options handling done
                    }
                    PauseMenuState::GameSettings => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            let mouse_x = mouse_pos.0;
                            let mouse_y = mouse_pos.1;

                            // check mouse's x position
                            if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                                // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                                if mouse_pos.1 > 287.0 && mouse_pos.1 < 325.0 {
                                    // if mouse's y position is between 287.0 and 325.0, then it is on menu option 0; menu_role = 0
                                    self.pause_menu_role = 0;
                                } else if mouse_pos.1 > 325.0 && mouse_pos.1 < 363.0 {
                                    // if mouse's y position is between 325.0 and 363.0, then it is on menu option 1; menu_role = 1
                                    self.pause_menu_role = 1;
                                } else if mouse_pos.1 > 363.0 && mouse_pos.1 < 401.0 {
                                    // if mouse's y position is between 363.0 and 401.0, then it is on menu option 2; menu_role = 2
                                    self.pause_menu_role = 2;
                                } else if mouse_pos.1 > 401.0 && mouse_pos.1 < 439.0 {
                                    // if mouse's y position is between 401.0 and 439.0, then it is on menu option 3; menu_role = 3
                                    self.pause_menu_role = 3;
                                } else if mouse_pos.1 > 439.0 && mouse_pos.1 < 477.0 {
                                    // if mouse's y position is between 439.0 and 477.0, then it is on menu option 4; menu_role = 4
                                    self.pause_menu_role = 4;
                                }
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.look_up_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 0 {
                                self.pause_menu_role = 4;
                            } else {
                                self.pause_menu_role -= 1;
                            }
                        }
                        if is_key_pressed(self.look_down_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 4 {
                                self.pause_menu_role = 0;
                            } else {
                                self.pause_menu_role += 1;
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.click_sound();
                            self.pause_menu_role = 0;
                            self.pause_menu_state = PauseMenuState::Settings;
                        }*/
                        // keyboard navigation done

                        // options handling
                        if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                            self.click_sound();
                            match self.pause_menu_role {
                                0 => {
                                    // Dash
                                    self.player.dash_enabled = !self.player.dash_enabled;
                                }
                                1 => {
                                    // Sprint
                                    self.player.sprint_enabled = !self.player.sprint_enabled;
                                }
                                2 => {
                                    // Wall Jump
                                    self.player.wall_jump_enabled = !self.player.wall_jump_enabled;
                                }
                                3 => {
                                    // Double Jump
                                    self.player.double_jump_enabled = !self.player.double_jump_enabled;
                                }
                                4 => {
                                    // Back
                                    self.pause_menu_role = 0;
                                    self.pause_menu_state = PauseMenuState::Settings;
                                }
                                _ => {}
                            }
                        }
                        // options handling done
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard } => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            let mouse_x = mouse_pos.0;
                            let mouse_y = mouse_pos.1;

                            // check volume options
                            // check mouse's x position
                            if mouse_pos.0 > 278.0 && mouse_pos.0 < 478.0 {
                                if mouse_pos.1 > 397.0 && mouse_pos.1 < 435.0 {
                                    // if mouse's y position is between 397.0 and 435.0, then it is on menu option 4; menu_role = 5
                                    self.pause_menu_role = 5;
                                } else if mouse_pos.1 > 435.0 && mouse_pos.1 < 473.0 {
                                    // if mouse's y position is between 435.0 and 473.0, then it is on menu option 4; menu_role = 6
                                    self.pause_menu_role = 6;
                                } else if mouse_pos.1 > 473.0 && mouse_pos.1 < 511.0 {
                                    // if mouse's y position is between 473.0 and 511.0, then it is on menu option 4; menu_role = 7
                                    self.pause_menu_role = 7;
                                } else if mouse_pos.1 > 511.0 && mouse_pos.1 < 549.0 {
                                    // if mouse's y position is between 511.0 and 549.0, then it is on menu option 4; menu_role = 8
                                    self.pause_menu_role = 8;
                                }
                            }

                            // check toggle options
                            // check mouse's x position
                            if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                                // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                                if mouse_pos.1 > 207.0 && mouse_pos.1 < 245.0 {
                                    // if mouse's y position is between 207.0 and 245.0, then it is on menu option 0; menu_role = 0
                                    self.pause_menu_role = 0;
                                } else if mouse_pos.1 > 245.0 && mouse_pos.1 < 283.0 {
                                    // if mouse's y position is between 245.0 and 283.0, then it is on menu option 1; menu_role = 1
                                    self.pause_menu_role = 1;
                                } else if mouse_pos.1 > 283.0 && mouse_pos.1 < 321.0 {
                                    // if mouse's y position is between 283.0 and 321.0, then it is on menu option 2; menu_role = 2
                                    self.pause_menu_role = 2;
                                } else if mouse_pos.1 > 321.0 && mouse_pos.1 < 359.0 {
                                    // if mouse's y position is between 321.0 and 359.0, then it is on menu option 3; menu_role = 3
                                    self.pause_menu_role = 3;
                                } else if mouse_pos.1 > 359.0 && mouse_pos.1 < 397.0 {
                                    // if mouse's y position is between 359.0 and 397.0, then it is on menu option 4; menu_role = 4
                                    self.pause_menu_role = 4;
                                } 
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.look_up_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 0 {
                                self.pause_menu_role = 8;
                            } else {
                                self.pause_menu_role -= 1;
                            }
                        }
                        if is_key_pressed(self.look_down_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 8 {
                                self.pause_menu_role = 0;
                            } else {
                                self.pause_menu_role += 1;
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.click_sound();
                            self.pause_menu_role = 1;
                            self.pause_menu_state = PauseMenuState::Settings;
                        }*/
                        // keyboard navigation done

                        // options handling
                        if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                            self.click_sound();
                            match self.pause_menu_role {
                                0 => {
                                    // Menu Music
                                    self.menu_music_settings_toggle = !self.menu_music_settings_toggle;
                                }
                                1 => {
                                    // Game Music
                                    self.game_music_settings_toggle = !self.game_music_settings_toggle;
                                }
                                2 => {
                                    // Ambient Sounds
                                    self.ambient_sounds_settings_toggle = !self.ambient_sounds_settings_toggle;
                                }
                                3 => {
                                    // Footsteps
                                    self.footsteps_settings_toggle = !self.footsteps_settings_toggle;
                                }
                                4 => {
                                    // Menu Clicks
                                    self.menu_clicks_settings_toggle = !self.menu_clicks_settings_toggle;
                                }
                                5 => {
                                    // Music Volume
                                    self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::MusicVolume };
                                }
                                6 => {
                                    // SFX Volume
                                    self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::SFXVolume };
                                }
                                7 => {
                                    // Menu Clicks Volume
                                    self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::MenuClicksVolume };
                                }
                                8 => {
                                    // Back
                                    self.pause_menu_role = 1;
                                    self.pause_menu_state = PauseMenuState::Settings;
                                }
                                _ => {}
                            }
                        }
                        // options handling done
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::MusicVolume } => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            let mouse_x = mouse_pos.0;
                            let mouse_y = mouse_pos.1;

                            // check mouse's y position
                            if mouse_pos.1 > 397.0 && mouse_pos.1 < 435.0 {
                                if mouse_pos.0 > 488.0 && mouse_pos.0 < 522.4 {
                                    // volume level 0
                                    self.potential_volume_lvl = 0;
                                } else if mouse_pos.0 > 522.4 && mouse_pos.0 < 522.4 + (48.8 * 1.0) {
                                    // volume level 1
                                    self.potential_volume_lvl = 1;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 1.0) && mouse_pos.0 < 522.4 + (48.8 * 2.0) {
                                    // volume level 2
                                    self.potential_volume_lvl = 2;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 2.0) && mouse_pos.0 < 522.4 + (48.8 * 3.0) {
                                    // volume level 3
                                    self.potential_volume_lvl = 3;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 3.0) && mouse_pos.0 < 522.4 + (48.8 * 4.0) {
                                    // volume level 4
                                    self.potential_volume_lvl = 4;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 4.0) && mouse_pos.0 < 522.4 + (48.8 * 5.0) {
                                    // volume level 5
                                    self.potential_volume_lvl = 5;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 5.0) && mouse_pos.0 < 522.4 + (48.8 * 6.0) {
                                    // volume level 6
                                    self.potential_volume_lvl = 6;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 6.0) && mouse_pos.0 < 522.4 + (48.8 * 7.0) {
                                    // volume level 7
                                    self.potential_volume_lvl = 7;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 7.0) && mouse_pos.0 < 522.4 + (48.8 * 8.0) {
                                    // volume level 8
                                    self.potential_volume_lvl = 8;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 8.0) && mouse_pos.0 < 522.4 + (48.8 * 9.0) {
                                    // volume level 9
                                    self.potential_volume_lvl = 9;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 9.0) && mouse_pos.0 < 1002.0 {
                                    // volume level 10
                                    self.potential_volume_lvl = 10;
                                }
                            }

                            if is_mouse_button_pressed(MouseButton::Left) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                if mouse_pos.0 > 488.0 && mouse_pos.0 < 1002.0 && mouse_pos.1 > 397.0 && mouse_pos.1 < 435.0 {
                                    self.music_volume = self.potential_volume_lvl;
                                } else {
                                    self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                                }
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            if self.music_volume < 10 {
                                self.music_volume += 1;
                                //set_sound_volume(&self.menu_music, self.music_volume as f32 / 10.0);
                                //set_sound_volume(&self.game_music, self.music_volume as f32 / 10.0);
                            }
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            if self.music_volume > 0 {
                                self.music_volume -= 1;
                                //set_sound_volume(&self.menu_music, self.music_volume as f32 / 10.0);
                                //set_sound_volume(&self.game_music, self.music_volume as f32 / 10.0);
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.click_sound();
                            self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }*/
                        if is_key_pressed(KeyCode::Enter) {
                            self.click_sound();
                            self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }
                        // keyboard navigation done
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::SFXVolume } => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            let mouse_x = mouse_pos.0;
                            let mouse_y = mouse_pos.1;

                            // check mouse's y position
                            if mouse_pos.1 > 435.0 && mouse_pos.1 < 473.0 {
                                if mouse_pos.0 > 488.0 && mouse_pos.0 < 522.4 {
                                    // volume level 0
                                    self.potential_volume_lvl = 0;
                                } else if mouse_pos.0 > 522.4 && mouse_pos.0 < 522.4 + (48.8 * 1.0) {
                                    // volume level 1
                                    self.potential_volume_lvl = 1;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 1.0) && mouse_pos.0 < 522.4 + (48.8 * 2.0) {
                                    // volume level 2
                                    self.potential_volume_lvl = 2;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 2.0) && mouse_pos.0 < 522.4 + (48.8 * 3.0) {
                                    // volume level 3
                                    self.potential_volume_lvl = 3;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 3.0) && mouse_pos.0 < 522.4 + (48.8 * 4.0) {
                                    // volume level 4
                                    self.potential_volume_lvl = 4;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 4.0) && mouse_pos.0 < 522.4 + (48.8 * 5.0) {
                                    // volume level 5
                                    self.potential_volume_lvl = 5;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 5.0) && mouse_pos.0 < 522.4 + (48.8 * 6.0) {
                                    // volume level 6
                                    self.potential_volume_lvl = 6;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 6.0) && mouse_pos.0 < 522.4 + (48.8 * 7.0) {
                                    // volume level 7
                                    self.potential_volume_lvl = 7;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 7.0) && mouse_pos.0 < 522.4 + (48.8 * 8.0) {
                                    // volume level 8
                                    self.potential_volume_lvl = 8;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 8.0) && mouse_pos.0 < 522.4 + (48.8 * 9.0) {
                                    // volume level 9
                                    self.potential_volume_lvl = 9;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 9.0) && mouse_pos.0 < 1002.0 {
                                    // volume level 10
                                    self.potential_volume_lvl = 10;
                                }
                            }

                            if is_mouse_button_pressed(MouseButton::Left) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                if mouse_pos.0 > 488.0 && mouse_pos.0 < 1002.0 && mouse_pos.1 > 435.0 && mouse_pos.1 < 473.0 {
                                    self.sfx_volume = self.potential_volume_lvl;
                                } else {
                                    self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                                }
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            if self.sfx_volume < 10 {
                                self.sfx_volume += 1;
                                //set_sound_volume(&self.ambient_sounds, self.sfx_volume as f32 / 10.0);
                                //set_sound_volume(&self.footsteps, self.sfx_volume as f32 / 10.0);
                            }
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            if self.sfx_volume > 0 {
                                self.sfx_volume -= 1;
                                //set_sound_volume(&self.ambient_sounds, self.sfx_volume as f32 / 10.0);
                                //set_sound_volume(&self.footsteps, self.sfx_volume as f32 / 10.0);
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.click_sound();
                            self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }*/
                        if is_key_pressed(KeyCode::Enter) {
                            self.click_sound();
                            self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }
                        // keyboard navigation done
                    }
                    PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::MenuClicksVolume } => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            let mouse_x = mouse_pos.0;
                            let mouse_y = mouse_pos.1;

                            // check mouse's y position
                            if mouse_pos.1 > 473.0 && mouse_pos.1 < 511.0 {
                                if mouse_pos.0 > 488.0 && mouse_pos.0 < 522.4 {
                                    // volume level 0
                                    self.potential_volume_lvl = 0;
                                } else if mouse_pos.0 > 522.4 && mouse_pos.0 < 522.4 + (48.8 * 1.0) {
                                    // volume level 1
                                    self.potential_volume_lvl = 1;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 1.0) && mouse_pos.0 < 522.4 + (48.8 * 2.0) {
                                    // volume level 2
                                    self.potential_volume_lvl = 2;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 2.0) && mouse_pos.0 < 522.4 + (48.8 * 3.0) {
                                    // volume level 3
                                    self.potential_volume_lvl = 3;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 3.0) && mouse_pos.0 < 522.4 + (48.8 * 4.0) {
                                    // volume level 4
                                    self.potential_volume_lvl = 4;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 4.0) && mouse_pos.0 < 522.4 + (48.8 * 5.0) {
                                    // volume level 5
                                    self.potential_volume_lvl = 5;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 5.0) && mouse_pos.0 < 522.4 + (48.8 * 6.0) {
                                    // volume level 6
                                    self.potential_volume_lvl = 6;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 6.0) && mouse_pos.0 < 522.4 + (48.8 * 7.0) {
                                    // volume level 7
                                    self.potential_volume_lvl = 7;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 7.0) && mouse_pos.0 < 522.4 + (48.8 * 8.0) {
                                    // volume level 8
                                    self.potential_volume_lvl = 8;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 8.0) && mouse_pos.0 < 522.4 + (48.8 * 9.0) {
                                    // volume level 9
                                    self.potential_volume_lvl = 9;
                                } else if mouse_pos.0 > 522.4 + (48.8 * 9.0) && mouse_pos.0 < 1002.0 {
                                    // volume level 10
                                    self.potential_volume_lvl = 10;
                                }
                            }

                            if is_mouse_button_pressed(MouseButton::Left) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                if mouse_pos.0 > 488.0 && mouse_pos.0 < 1002.0 && mouse_pos.1 > 473.0 && mouse_pos.1 < 511.0 {
                                    self.menu_clicks_volume = self.potential_volume_lvl;
                                } else {
                                    self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                                }
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            if self.menu_clicks_volume < 10 {
                                self.menu_clicks_volume += 1;
                                set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                            }
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            if self.menu_clicks_volume > 0 {
                                self.menu_clicks_volume -= 1;
                                set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.click_sound();
                            self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }*/
                        if is_key_pressed(KeyCode::Enter) {
                            self.click_sound();
                            self.pause_menu_state = PauseMenuState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }
                        // keyboard navigation done
                    }
                    PauseMenuState::VideoSettings => {
                        // mouse navigation
                        if self.mouse_moved_buffer > 0.0 {
                            let mouse_pos = mouse_position();
                            let mouse_x = mouse_pos.0;
                            let mouse_y = mouse_pos.1;

                            // check mouse's x position
                            if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                                // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                                if mouse_pos.1 > 327.0 && mouse_pos.1 < 365.0 {
                                    // if mouse's y position is between 327.0 and 365.0, then it is on menu option 0; menu_role = 0
                                    self.pause_menu_role = 0;
                                } else if mouse_pos.1 > 365.0 && mouse_pos.1 < 403.0 {
                                    // if mouse's y position is between 365.0 and 403.0, then it is on menu option 1; menu_role = 1
                                    self.pause_menu_role = 1;
                                } else if mouse_pos.1 > 403.0 && mouse_pos.1 < 441.0 {
                                    // if mouse's y position is between 403.0 and 441.0, then it is on menu option 2; menu_role = 2
                                    self.pause_menu_role = 2;
                                }
                            }
                        }
                        // mouse navigation done

                        // keyboard navigation
                        if is_key_pressed(self.look_up_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 0 {
                                self.pause_menu_role = 2;
                            } else {
                                self.pause_menu_role -= 1;
                            }
                        }
                        if is_key_pressed(self.look_down_kb) {
                            self.click_sound();
                            if self.pause_menu_role == 2 {
                                self.pause_menu_role = 0;
                            } else {
                                self.pause_menu_role += 1;
                            }
                        }
                        /*if is_key_pressed(KeyCode::Escape) {
                            self.click_sound();
                            self.pause_menu_role = 2;
                            self.pause_menu_state = PauseMenuState::Settings;
                        }*/
                        // keyboard navigation done

                        // options handling
                        if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                            self.click_sound();
                            match self.pause_menu_role {
                                0 => {
                                    // Resolution
                                }
                                1 => {
                                    // Brightness
                                }
                                2 => {
                                    // Back
                                    self.pause_menu_role = 2;
                                    self.pause_menu_state = PauseMenuState::Settings;
                                }
                                _ => {}
                            }
                        }
                        // options handling done
                    }
                    PauseMenuState::ControlsSettings => {
                        if !self.awaiting_kb_input {
                            // mouse navigation
                            if self.mouse_moved_buffer > 0.0 {
                                let mouse_pos = mouse_position();
                                let mouse_x = mouse_pos.0;
                                let mouse_y = mouse_pos.1;

                                // check mouse's x position
                                if mouse_pos.1 > 267.0 && mouse_pos.1 < 495.0 {
                                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 640.0 {
                                        // if mouse's x position is between 278.0 and 640.0, then it is in the width of the first column, so check its y position
                                        self.pause_menu_controls_settings_col = 0;
                                    } else if mouse_pos.0 >= 640.0 && mouse_pos.0 < 1002.0 {
                                        // if mouse's x position is between 640.0 and 1002.0, then it is in the width of the second column, so check its y position
                                        self.pause_menu_controls_settings_col = 1;
                                    }
                                }

                                // check mouse's y position
                                if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                                    if mouse_pos.1 > 267.0 && mouse_pos.1 < 305.0 {
                                        // if mouse's y position is between 267.0 and 305.0, then it is on menu option 0; menu_role = 0
                                        self.pause_menu_controls_settings_row = 0;
                                    } else if mouse_pos.1 > 305.0 && mouse_pos.1 < 343.0 {
                                        // if mouse's y position is between 305.0 and 343.0, then it is on menu option 1; menu_role = 1
                                        self.pause_menu_controls_settings_row = 1;
                                    } else if mouse_pos.1 > 343.0 && mouse_pos.1 < 381.0 {
                                        // if mouse's y position is between 343.0 and 381.0, then it is on menu option 2; menu_role = 2
                                        self.pause_menu_controls_settings_row = 2;
                                    } else if mouse_pos.1 > 381.0 && mouse_pos.1 < 419.0 {
                                        // if mouse's y position is between 381.0 and 419.0, then it is on menu option 2; menu_role = 3
                                        self.pause_menu_controls_settings_row = 3;
                                    } else if mouse_pos.1 > 419.0 && mouse_pos.1 < 457.0 {
                                        // if mouse's y position is between 419.0 and 457.0, then it is on menu option 2; menu_role = 4
                                        self.pause_menu_controls_settings_row = 4;
                                    } else if mouse_pos.1 > 457.0 && mouse_pos.1 < 495.0 {
                                        // if mouse's y position is between 457.0 and 495.0, then it is on menu option 2; menu_role = 4
                                        self.pause_menu_controls_settings_row = 5;
                                    }
                                }
                            }
                            // mouse navigation done

                            // keyboard navigation
                            if is_key_pressed(self.look_up_kb) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                if self.pause_menu_controls_settings_row == 0 {
                                    self.pause_menu_controls_settings_row = 5;
                                } else {
                                    self.pause_menu_controls_settings_row -= 1;
                                }
                            }
                            if is_key_pressed(self.look_down_kb) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                if self.pause_menu_controls_settings_row == 5 {
                                    self.pause_menu_controls_settings_row = 0;
                                } else {
                                    self.pause_menu_controls_settings_row += 1;
                                }
                            }
                            if self.pause_menu_controls_settings_row != 5 {
                                if is_key_pressed(self.move_right_kb) {
                                    if self.menu_clicks_settings_toggle {
                                        play_sound_once(&self.menu_click_sound);
                                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                    }
                                    if self.pause_menu_controls_settings_col == 1 {
                                        self.pause_menu_controls_settings_col = 0;
                                    } else {
                                        self.pause_menu_controls_settings_col += 1;
                                    }
                                }
                                if is_key_pressed(self.move_left_kb) {
                                    if self.menu_clicks_settings_toggle {
                                        play_sound_once(&self.menu_click_sound);
                                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                    }
                                    if self.pause_menu_controls_settings_col == 0 {
                                        self.pause_menu_controls_settings_col = 1;
                                    } else {
                                        self.pause_menu_controls_settings_col -= 1;
                                    }
                                }
                            }
                            /*if is_key_pressed(KeyCode::Escape) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                self.pause_menu_role = 3;
                                self.pause_menu_state = PauseMenuState::Settings;
                                self.pause_menu_controls_settings_row = 0;
                                self.pause_menu_controls_settings_col = 0;
                                self.kb_to_change = Keybind::None;
                            }*/
                            // keyboard navigation done

                            // options handling
                            if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                if self.pause_menu_controls_settings_col == 0 {
                                    // in first column
                                    match self.pause_menu_controls_settings_row {
                                        0 => {
                                            // Move Left
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::MoveLeft;
                                        }
                                        1 => {
                                            // Move Right
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::MoveRight;
                                        }
                                        2 => {
                                            // Look Up
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::LookUp;
                                        }
                                        3 => {
                                            // Look Down
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::LookDown;
                                        }
                                        4 => {
                                            // Jump
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::Jump;
                                        }
                                        5 => {
                                            // Back
                                            self.pause_menu_role = 3;
                                            self.pause_menu_state = PauseMenuState::Settings;
                                            self.pause_menu_controls_settings_row = 0;
                                            self.pause_menu_controls_settings_col = 0;
                                            self.kb_to_change = Keybind::None;
                                        }
                                        _ => {}
                                    }
                                } else {
                                    // in second column
                                    match self.pause_menu_controls_settings_row {
                                        0 => {
                                            // Dash/Sprint
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::DashSprint;
                                        }
                                        1 => {
                                            // Melee Attack
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::MeleeAttack;
                                        }
                                        2 => {
                                            // Ranged Attack
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::RangedAttack;
                                        }
                                        3 => {
                                            // Interact
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::Interact;
                                        }
                                        4 => {
                                            // Inventory
                                            self.awaiting_kb_input = true;
                                            self.kb_to_change = Keybind::Inventory;
                                        }
                                        5 => {
                                            // Back
                                            self.pause_menu_role = 3;
                                            self.pause_menu_state = PauseMenuState::Settings;
                                            self.pause_menu_controls_settings_row = 0;
                                            self.pause_menu_controls_settings_col = 0;
                                            self.kb_to_change = Keybind::None;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            // options handling done
                        } else {
                            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                                if self.menu_clicks_settings_toggle {
                                    play_sound_once(&self.menu_click_sound);
                                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                                }
                                self.awaiting_kb_input = false;
                            }

                            self.change_keybind();
                        }
                    }
                    PauseMenuState::None => {}
                }
            }
            // in-game menu done

            // inventory
            if is_key_pressed(self.inventory_kb) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                }
                match &self.inventory_state {
                    InventoryState::Closed { last_inventory_page } => {
                        self.inventory_state = InventoryState::Open { inventory_page: last_inventory_page.clone() };
                    }
                    InventoryState::Open { inventory_page } => {
                        self.inventory_state = InventoryState::Closed { last_inventory_page: inventory_page.clone() };
                    }
                }
            }

            if let InventoryState::Open { inventory_page } = &self.inventory_state {
                match inventory_page {
                    InventoryPage::Items => {
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::Equipment };
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::EnemyLog };
                        }
                    }
                    InventoryPage::Equipment => {
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::Map };
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::Items };
                        }
                    }
                    InventoryPage::Map => {
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::EnemyLog };
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::Equipment };
                        }
                    }
                    InventoryPage::EnemyLog => {
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::Items };
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            self.inventory_state = InventoryState::Open { inventory_page: InventoryPage::Map };
                        }
                    }
                }
            }
            // inventory done
        }
        // STORYPHASE::PLAYING DONE


        // room changes
        if let RoomChange::Change { door } = &self.player.room_change {
            self.change_room(&door.clone());
        }
    }

    pub fn draw(&mut self) {
        if !matches!(self.startup_state, StartupState::Done) {
            draw_startup_overlay(
                &self.startup_state,
                self.startup_menu_role,
                self.player.dash_enabled,
                self.player.sprint_enabled,
                self.player.wall_jump_enabled,
                self.player.double_jump_enabled,
                self.menu_music_settings_toggle,
                self.game_music_settings_toggle,
                self.ambient_sounds_settings_toggle,
                self.footsteps_settings_toggle,
                self.menu_clicks_settings_toggle,
                self.music_volume,
                self.sfx_volume,
                self.menu_clicks_volume,
                self.controls_settings_row,
                self.controls_settings_col,
                self.move_left_kb,
                self.move_right_kb,
                self.look_up_kb,
                self.look_down_kb,
                self.jump_kb,
                self.dash_sprint_kb,
                self.melee_attack_kb,
                self.ranged_attack_kb,
                self.interact_kb,
                self.inventory_kb,
                self.awaiting_kb_input,
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

            // update attack hitbox position
            match self.player.attack_direction {
                AttackDirection::Right => {
                    self.player.attack_x = self.player.x + self.player.pwidth * TILE_SIZE;
                    self.player.attack_y = self.player.y - (1.25 * TILE_SIZE);
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.25 * TILE_SIZE;
                }
                AttackDirection::Left => {
                    self.player.attack_x = self.player.x - (1.0 * TILE_SIZE);
                    self.player.attack_y = self.player.y - (1.25 * TILE_SIZE);
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.25 * TILE_SIZE;
                }
                AttackDirection::Up => {
                    self.player.attack_x = self.player.x;
                    self.player.attack_y = self.player.y - self.player.pheight * TILE_SIZE - (1.0 * TILE_SIZE);
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.0 * TILE_SIZE;
                }
                AttackDirection::Down => {
                    self.player.attack_x = self.player.x;
                    self.player.attack_y = self.player.y;
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.0 * TILE_SIZE;
                }
            }
            self.player.draw();

            for enemy in &self.current_room.enemies {
                enemy.draw();
            }

            self.draw_tiles();

            set_default_camera();

            self.draw_player_lives();
        }

        if self.paused {
            draw_pause_menu_overlay(
                self.pause_menu_state.clone(),
                self.pause_menu_role,
                self.pause_menu_controls_settings_row,
                self.pause_menu_controls_settings_col,
                self.player.dash_enabled,
                self.player.sprint_enabled,
                self.player.wall_jump_enabled,
                self.player.double_jump_enabled,
                self.menu_music_settings_toggle,
                self.game_music_settings_toggle,
                self.ambient_sounds_settings_toggle,
                self.footsteps_settings_toggle,
                self.menu_clicks_settings_toggle,
                self.music_volume,
                self.sfx_volume,
                self.menu_clicks_volume,
                self.move_left_kb,
                self.move_right_kb,
                self.look_up_kb,
                self.look_down_kb,
                self.jump_kb,
                self.dash_sprint_kb,
                self.melee_attack_kb,
                self.ranged_attack_kb,
                self.interact_kb,
                self.inventory_kb,
                self.awaiting_kb_input,
            );
            // blah
        }

        if let InventoryState::Open { inventory_page } = &self.inventory_state {
            draw_inventory_overlay(
                inventory_page.clone(),
            );
        }
    }

    /*pub fn draw1(&mut self) {
        if !matches!(self.startup_state, StartupState::Done) {
            draw_startup_overlay(
                &self.startup_state,
                self.startup_menu_role,
                self.player.dash_enabled,
                self.player.sprint_enabled,
                self.player.wall_jump_enabled,
                self.player.double_jump_enabled,
                self.menu_music_settings_toggle,
                self.game_music_settings_toggle,
                self.ambient_sounds_settings_toggle,
                self.footsteps_settings_toggle,
                self.menu_clicks_settings_toggle,
                self.music_volume,
                self.sfx_volume,
                self.menu_clicks_volume,
                self.controls_settings_row,
                self.controls_settings_col,
                self.move_left_kb,
                self.move_right_kb,
                self.look_up_kb,
                self.look_down_kb,
                self.jump_kb,
                self.dash_sprint_kb,
                self.melee_attack_kb,
                self.ranged_attack_kb,
                self.interact_kb,
                self.inventory_kb,
                self.awaiting_kb_input,
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

            // update attack hitbox position
            match self.player.attack_direction {
                AttackDirection::Right => {
                    self.player.attack_x = self.player.x + self.player.pwidth * TILE_SIZE;
                    self.player.attack_y = self.player.y - (1.25 * TILE_SIZE);
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.25 * TILE_SIZE;
                }
                AttackDirection::Left => {
                    self.player.attack_x = self.player.x - (1.0 * TILE_SIZE);
                    self.player.attack_y = self.player.y - (1.25 * TILE_SIZE);
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.25 * TILE_SIZE;
                }
                AttackDirection::Up => {
                    self.player.attack_x = self.player.x;
                    self.player.attack_y = self.player.y - self.player.pheight * TILE_SIZE - (1.0 * TILE_SIZE);
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.0 * TILE_SIZE;
                }
                AttackDirection::Down => {
                    self.player.attack_x = self.player.x;
                    self.player.attack_y = self.player.y;
                    self.player.attack_width = 1.0 * TILE_SIZE;
                    self.player.attack_height = 1.0 * TILE_SIZE;
                }
            }
            self.player.draw();

            // loop through enemies in the room and draw them
            //println!("{}", format!("{:?}", self.current_room.enemies));
            //println!("hello");
            for enemy in &self.current_room.enemies {
                //println!("hi");
                enemy.draw();
            }

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
            draw_pause_menu_overlay(
                self.pause_menu_state.clone(),
                self.pause_menu_role,
                self.pause_menu_controls_settings_row,
                self.pause_menu_controls_settings_col,
            );

            match self.pause_menu_state {
                PauseMenuState::Menu { .. } => {
                    self.pause_menu_state = PauseMenuState::Menu { pause_menu_role: self.pause_menu_role };
                }
                PauseMenuState::None => {}
            }

            match self.pause_menu_state {
                PauseMenuState::Menu { .. } => {
                    todo!();
                }
                PauseMenuState::Settings { .. } => {
                    todo!();
                }
                PauseMenuState::GameSettings { .. } => {
                    todo!();
                }
                PauseMenuState::AudioSettings { .. } => {
                    todo!();
                }
                PauseMenuState::VideoSettings { .. } => {
                    todo!();
                }
                PauseMenuState::ControlsSettings { .. } => {
                    todo!();
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
                

            if is_key_pressed(self.look_up_kb) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                }
                if self.pause_menu_role == 0 {
                    self.pause_menu_role = 2;
                } else {
                    self.pause_menu_role -= 1;
                }
            }

            if is_key_pressed(self.look_down_kb) {
                if self.menu_clicks_settings_toggle {
                    play_sound_once(&self.menu_click_sound);
                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
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
                    set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
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
    }*/

    fn handle_startup_input(&mut self) {
        match &self.startup_state {
            StartupState::Splash => {
                if get_last_key_pressed().is_some() || is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::MainMenu;
                }
            }
            StartupState::MainMenu => {
                // main menu navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check mouse's x position
                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                        // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                        if mouse_pos.1 > 327.0 && mouse_pos.1 < 365.0 {
                            // if mouse's y position is between 327.0 and 365.0, then it is on menu option 0; menu_role = 0
                            self.startup_menu_role = 0;
                        } else if mouse_pos.1 > 365.0 && mouse_pos.1 < 403.0 {
                            // if mouse's y position is between 365.0 and 403.0, then it is on menu option 1; menu_role = 1
                            self.startup_menu_role = 1;
                        } else if mouse_pos.1 > 403.0 && mouse_pos.1 < 441.0 {
                            // if mouse's y position is between 403.0 and 441.0, then it is on menu option 2; menu_role = 2
                            self.startup_menu_role = 2;
                        }
                    }
                }
                // main menu navigation with mouse done
                
                // main menu navigation with keyboard
                if is_key_pressed(self.look_up_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 0 {
                        self.startup_menu_role = 2;
                    } else {
                        self.startup_menu_role -= 1;
                    }
                }
                if is_key_pressed(self.look_down_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 2 {
                        self.startup_menu_role = 0;
                    } else {
                        self.startup_menu_role += 1;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::Splash;
                }
                // main menu navigation with keyboard done

                // main menu options handling
                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    match self.startup_menu_role {
                        0 => {
                            // Play
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::Done;
                            self.story = StoryPhase::Playing;
                        }
                        1 => {
                            // Settings
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::Settings;
                        }
                        2 => {
                            // Exit game
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                // main menu options handling done
            }
            StartupState::Settings => {
                // settings navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check mouse's x position
                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                        // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                        if mouse_pos.1 > 287.0 && mouse_pos.1 < 325.0 {
                            // if mouse's y position is between 287.0 and 325.0, then it is on menu option 0; menu_role = 0
                            self.startup_menu_role = 0;
                        } else if mouse_pos.1 > 325.0 && mouse_pos.1 < 363.0 {
                            // if mouse's y position is between 325.0 and 363.0, then it is on menu option 1; menu_role = 1
                            self.startup_menu_role = 1;
                        } else if mouse_pos.1 > 363.0 && mouse_pos.1 < 401.0 {
                            // if mouse's y position is between 363.0 and 401.0, then it is on menu option 2; menu_role = 2
                            self.startup_menu_role = 2;
                        } else if mouse_pos.1 > 401.0 && mouse_pos.1 < 439.0 {
                            // if mouse's y position is between 401.0 and 439.0, then it is on menu option 3; menu_role = 3
                            self.startup_menu_role = 3;
                        } else if mouse_pos.1 > 439.0 && mouse_pos.1 < 477.0 {
                            // if mouse's y position is between 439.0 and 477.0, then it is on menu option 4; menu_role = 4
                            self.startup_menu_role = 4;
                        }
                    }
                }
                // settings navigation with mouse done

                // settings navigation with keyboard
                if is_key_pressed(self.look_up_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 0 {
                        self.startup_menu_role = 4;
                    } else {
                        self.startup_menu_role -= 1;
                    }
                }
                if is_key_pressed(self.look_down_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 4 {
                        self.startup_menu_role = 0;
                    } else {
                        self.startup_menu_role += 1;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_menu_role = 1;
                    self.startup_state = StartupState::MainMenu;
                }
                // settings navigation with keyboard done

                // settings options handling
                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    match self.startup_menu_role {
                        0 => {
                            // Game
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::GameSettings;
                        }
                        1 => {
                            // Audio
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }
                        2 => {
                            // Video
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::VideoSettings;
                        }
                        3 => {
                            // Controls
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::ControlsSettings;
                        }
                        4 => {
                            // Back
                            self.startup_menu_role = 1;
                            self.startup_state = StartupState::MainMenu;
                        }
                        _ => {}
                    }
                }
                // settings options handling done
            }
            StartupState::GameSettings => {
                // game settings navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check mouse's x position
                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                        // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                        if mouse_pos.1 > 287.0 && mouse_pos.1 < 325.0 {
                            // if mouse's y position is between 287.0 and 325.0, then it is on menu option 0; menu_role = 0
                            self.startup_menu_role = 0;
                        } else if mouse_pos.1 > 325.0 && mouse_pos.1 < 363.0 {
                            // if mouse's y position is between 325.0 and 363.0, then it is on menu option 1; menu_role = 1
                            self.startup_menu_role = 1;
                        } else if mouse_pos.1 > 363.0 && mouse_pos.1 < 401.0 {
                            // if mouse's y position is between 363.0 and 401.0, then it is on menu option 2; menu_role = 2
                            self.startup_menu_role = 2;
                        } else if mouse_pos.1 > 401.0 && mouse_pos.1 < 439.0 {
                            // if mouse's y position is between 401.0 and 439.0, then it is on menu option 3; menu_role = 3
                            self.startup_menu_role = 3;
                        } else if mouse_pos.1 > 439.0 && mouse_pos.1 < 477.0 {
                            // if mouse's y position is between 439.0 and 477.0, then it is on menu option 4; menu_role = 4
                            self.startup_menu_role = 4;
                        }
                    }
                }
                // game settings navigation with mouse done

                // game settings navigation with keyboard
                if is_key_pressed(self.look_up_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 0 {
                        self.startup_menu_role = 4;
                    } else {
                        self.startup_menu_role -= 1;
                    }
                }
                if is_key_pressed(self.look_down_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 4 {
                        self.startup_menu_role = 0;
                    } else {
                        self.startup_menu_role += 1;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_menu_role = 0;
                    self.startup_state = StartupState::Settings;
                }
                // game settings navigation with keyboard done

                // game settings options handling
                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    match self.startup_menu_role {
                        0 => {
                            // Dash
                            self.player.dash_enabled = !self.player.dash_enabled;
                        }
                        1 => {
                            // Sprint
                            self.player.sprint_enabled = !self.player.sprint_enabled;
                        }
                        2 => {
                            // Wall Jump
                            self.player.wall_jump_enabled = !self.player.wall_jump_enabled;
                        }
                        3 => {
                            // Double Jump
                            self.player.double_jump_enabled = !self.player.double_jump_enabled;
                        }
                        4 => {
                            // Back
                            self.startup_menu_role = 0;
                            self.startup_state = StartupState::Settings;
                        }
                        _ => {}
                    }
                }
                // game settings options handling done
            }
            StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard } => {
                // audio settings navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check volume options
                    // check mouse's x position
                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 478.0 {
                        if mouse_pos.1 > 397.0 && mouse_pos.1 < 435.0 {
                            // if mouse's y position is between 397.0 and 435.0, then it is on menu option 4; menu_role = 5
                            self.startup_menu_role = 5;
                        } else if mouse_pos.1 > 435.0 && mouse_pos.1 < 473.0 {
                            // if mouse's y position is between 435.0 and 473.0, then it is on menu option 4; menu_role = 6
                            self.startup_menu_role = 6;
                        } else if mouse_pos.1 > 473.0 && mouse_pos.1 < 511.0 {
                            // if mouse's y position is between 473.0 and 511.0, then it is on menu option 4; menu_role = 7
                            self.startup_menu_role = 7;
                        } else if mouse_pos.1 > 511.0 && mouse_pos.1 < 549.0 {
                            // if mouse's y position is between 511.0 and 549.0, then it is on menu option 4; menu_role = 8
                            self.startup_menu_role = 8;
                        }
                    }

                    // check toggle options
                    // check mouse's x position
                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                        // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                        if mouse_pos.1 > 207.0 && mouse_pos.1 < 245.0 {
                            // if mouse's y position is between 207.0 and 245.0, then it is on menu option 0; menu_role = 0
                            self.startup_menu_role = 0;
                        } else if mouse_pos.1 > 245.0 && mouse_pos.1 < 283.0 {
                            // if mouse's y position is between 245.0 and 283.0, then it is on menu option 1; menu_role = 1
                            self.startup_menu_role = 1;
                        } else if mouse_pos.1 > 283.0 && mouse_pos.1 < 321.0 {
                            // if mouse's y position is between 283.0 and 321.0, then it is on menu option 2; menu_role = 2
                            self.startup_menu_role = 2;
                        } else if mouse_pos.1 > 321.0 && mouse_pos.1 < 359.0 {
                            // if mouse's y position is between 321.0 and 359.0, then it is on menu option 3; menu_role = 3
                            self.startup_menu_role = 3;
                        } else if mouse_pos.1 > 359.0 && mouse_pos.1 < 397.0 {
                            // if mouse's y position is between 359.0 and 397.0, then it is on menu option 4; menu_role = 4
                            self.startup_menu_role = 4;
                        } 
                    }
                }
                // audio settings navigation with mouse done

                // audio settings navigation with keyboard
                if is_key_pressed(self.look_up_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 0 {
                        self.startup_menu_role = 8;
                    } else {
                        self.startup_menu_role -= 1;
                    }
                }
                if is_key_pressed(self.look_down_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 8 {
                        self.startup_menu_role = 0;
                    } else {
                        self.startup_menu_role += 1;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_menu_role = 1;
                    self.startup_state = StartupState::Settings;
                }
                // audio settings navigation with keyboard done

                // audio settings options handling
                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    match self.startup_menu_role {
                        0 => {
                            // Menu Music
                            self.menu_music_settings_toggle = !self.menu_music_settings_toggle;
                        }
                        1 => {
                            // Game Music
                            self.game_music_settings_toggle = !self.game_music_settings_toggle;
                        }
                        2 => {
                            // Ambient Sounds
                            self.ambient_sounds_settings_toggle = !self.ambient_sounds_settings_toggle;
                        }
                        3 => {
                            // Footsteps
                            self.footsteps_settings_toggle = !self.footsteps_settings_toggle;
                        }
                        4 => {
                            // Menu Clicks
                            self.menu_clicks_settings_toggle = !self.menu_clicks_settings_toggle;
                        }
                        5 => {
                            // Music Volume
                            self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::MusicVolume };
                        }
                        6 => {
                            // SFX Volume
                            self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::SFXVolume };
                        }
                        7 => {
                            // Menu Clicks Volume
                            self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::MenuClicksVolume };
                        }
                        8 => {
                            // Back
                            self.startup_menu_role = 1;
                            self.startup_state = StartupState::Settings;
                        }
                        _ => {}
                    }
                }
                // audio settings options handling done
            }
            StartupState::AudioSettings { audio_settings_state: AudioSettingsState::MusicVolume } => {
                // audio settings music volume navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check mouse's y position
                    if mouse_pos.1 > 397.0 && mouse_pos.1 < 435.0 {
                        if mouse_pos.0 > 488.0 && mouse_pos.0 < 522.4 {
                            // volume level 0
                            self.potential_volume_lvl = 0;
                        } else if mouse_pos.0 > 522.4 && mouse_pos.0 < 522.4 + (48.8 * 1.0) {
                            // volume level 1
                            self.potential_volume_lvl = 1;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 1.0) && mouse_pos.0 < 522.4 + (48.8 * 2.0) {
                            // volume level 2
                            self.potential_volume_lvl = 2;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 2.0) && mouse_pos.0 < 522.4 + (48.8 * 3.0) {
                            // volume level 3
                            self.potential_volume_lvl = 3;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 3.0) && mouse_pos.0 < 522.4 + (48.8 * 4.0) {
                            // volume level 4
                            self.potential_volume_lvl = 4;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 4.0) && mouse_pos.0 < 522.4 + (48.8 * 5.0) {
                            // volume level 5
                            self.potential_volume_lvl = 5;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 5.0) && mouse_pos.0 < 522.4 + (48.8 * 6.0) {
                            // volume level 6
                            self.potential_volume_lvl = 6;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 6.0) && mouse_pos.0 < 522.4 + (48.8 * 7.0) {
                            // volume level 7
                            self.potential_volume_lvl = 7;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 7.0) && mouse_pos.0 < 522.4 + (48.8 * 8.0) {
                            // volume level 8
                            self.potential_volume_lvl = 8;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 8.0) && mouse_pos.0 < 522.4 + (48.8 * 9.0) {
                            // volume level 9
                            self.potential_volume_lvl = 9;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 9.0) && mouse_pos.0 < 1002.0 {
                            // volume level 10
                            self.potential_volume_lvl = 10;
                        }
                    }

                    if is_mouse_button_pressed(MouseButton::Left) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        if mouse_pos.0 > 488.0 && mouse_pos.0 < 1002.0 && mouse_pos.1 > 397.0 && mouse_pos.1 < 435.0 {
                            self.music_volume = self.potential_volume_lvl;
                        } else {
                            self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }
                    }
                }
                // audio settings music volume navigation with mouse done

                // audio settings music volume navigation with keyboard
                if is_key_pressed(self.move_right_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.music_volume < 10 {
                        self.music_volume += 1;
                        //set_sound_volume(&self.menu_music, self.music_volume as f32 / 10.0);
                        //set_sound_volume(&self.game_music, self.music_volume as f32 / 10.0);
                    }
                }
                if is_key_pressed(self.move_left_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.music_volume > 0 {
                        self.music_volume -= 1;
                        //set_sound_volume(&self.menu_music, self.music_volume as f32 / 10.0);
                        //set_sound_volume(&self.game_music, self.music_volume as f32 / 10.0);
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                }
                if is_key_pressed(KeyCode::Enter) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                }
                // audio settings music volume navigation with keyboard done
            }
            StartupState::AudioSettings { audio_settings_state: AudioSettingsState::SFXVolume } => {
                // audio settings sfx volume navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check mouse's y position
                    if mouse_pos.1 > 435.0 && mouse_pos.1 < 473.0 {
                        if mouse_pos.0 > 488.0 && mouse_pos.0 < 522.4 {
                            // volume level 0
                            self.potential_volume_lvl = 0;
                        } else if mouse_pos.0 > 522.4 && mouse_pos.0 < 522.4 + (48.8 * 1.0) {
                            // volume level 1
                            self.potential_volume_lvl = 1;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 1.0) && mouse_pos.0 < 522.4 + (48.8 * 2.0) {
                            // volume level 2
                            self.potential_volume_lvl = 2;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 2.0) && mouse_pos.0 < 522.4 + (48.8 * 3.0) {
                            // volume level 3
                            self.potential_volume_lvl = 3;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 3.0) && mouse_pos.0 < 522.4 + (48.8 * 4.0) {
                            // volume level 4
                            self.potential_volume_lvl = 4;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 4.0) && mouse_pos.0 < 522.4 + (48.8 * 5.0) {
                            // volume level 5
                            self.potential_volume_lvl = 5;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 5.0) && mouse_pos.0 < 522.4 + (48.8 * 6.0) {
                            // volume level 6
                            self.potential_volume_lvl = 6;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 6.0) && mouse_pos.0 < 522.4 + (48.8 * 7.0) {
                            // volume level 7
                            self.potential_volume_lvl = 7;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 7.0) && mouse_pos.0 < 522.4 + (48.8 * 8.0) {
                            // volume level 8
                            self.potential_volume_lvl = 8;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 8.0) && mouse_pos.0 < 522.4 + (48.8 * 9.0) {
                            // volume level 9
                            self.potential_volume_lvl = 9;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 9.0) && mouse_pos.0 < 1002.0 {
                            // volume level 10
                            self.potential_volume_lvl = 10;
                        }
                    }

                    if is_mouse_button_pressed(MouseButton::Left) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        if mouse_pos.0 > 488.0 && mouse_pos.0 < 1002.0 && mouse_pos.1 > 435.0 && mouse_pos.1 < 473.0 {
                            self.sfx_volume = self.potential_volume_lvl;
                        } else {
                            self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }
                    }
                }
                // audio settings sfx volume navigation with mouse done

                // audio settings sfx volume navigation with keyboard
                if is_key_pressed(self.move_right_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.sfx_volume < 10 {
                        self.sfx_volume += 1;
                        //set_sound_volume(&self.ambient_sounds, self.sfx_volume as f32 / 10.0);
                        //set_sound_volume(&self.footsteps, self.sfx_volume as f32 / 10.0);
                    }
                }
                if is_key_pressed(self.move_left_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.sfx_volume > 0 {
                        self.sfx_volume -= 1;
                        //set_sound_volume(&self.ambient_sounds, self.sfx_volume as f32 / 10.0);
                        //set_sound_volume(&self.footsteps, self.sfx_volume as f32 / 10.0);
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                }
                if is_key_pressed(KeyCode::Enter) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                }
                // audio settings sfx volume navigation with keyboard done
            }
            StartupState::AudioSettings { audio_settings_state: AudioSettingsState::MenuClicksVolume } => {
                // audio settings menu clicks volume navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check mouse's y position
                    if mouse_pos.1 > 473.0 && mouse_pos.1 < 511.0 {
                        if mouse_pos.0 > 488.0 && mouse_pos.0 < 522.4 {
                            // volume level 0
                            self.potential_volume_lvl = 0;
                        } else if mouse_pos.0 > 522.4 && mouse_pos.0 < 522.4 + (48.8 * 1.0) {
                            // volume level 1
                            self.potential_volume_lvl = 1;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 1.0) && mouse_pos.0 < 522.4 + (48.8 * 2.0) {
                            // volume level 2
                            self.potential_volume_lvl = 2;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 2.0) && mouse_pos.0 < 522.4 + (48.8 * 3.0) {
                            // volume level 3
                            self.potential_volume_lvl = 3;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 3.0) && mouse_pos.0 < 522.4 + (48.8 * 4.0) {
                            // volume level 4
                            self.potential_volume_lvl = 4;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 4.0) && mouse_pos.0 < 522.4 + (48.8 * 5.0) {
                            // volume level 5
                            self.potential_volume_lvl = 5;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 5.0) && mouse_pos.0 < 522.4 + (48.8 * 6.0) {
                            // volume level 6
                            self.potential_volume_lvl = 6;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 6.0) && mouse_pos.0 < 522.4 + (48.8 * 7.0) {
                            // volume level 7
                            self.potential_volume_lvl = 7;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 7.0) && mouse_pos.0 < 522.4 + (48.8 * 8.0) {
                            // volume level 8
                            self.potential_volume_lvl = 8;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 8.0) && mouse_pos.0 < 522.4 + (48.8 * 9.0) {
                            // volume level 9
                            self.potential_volume_lvl = 9;
                        } else if mouse_pos.0 > 522.4 + (48.8 * 9.0) && mouse_pos.0 < 1002.0 {
                            // volume level 10
                            self.potential_volume_lvl = 10;
                        }
                    }

                    if is_mouse_button_pressed(MouseButton::Left) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        if mouse_pos.0 > 488.0 && mouse_pos.0 < 1002.0 && mouse_pos.1 > 473.0 && mouse_pos.1 < 511.0 {
                            self.menu_clicks_volume = self.potential_volume_lvl;
                        } else {
                            self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                        }
                    }
                }
                // audio settings menu clicks volume navigation with mouse done

                // audio settings menu clicks volume navigation with keyboard
                if is_key_pressed(self.move_right_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.menu_clicks_volume < 10 {
                        self.menu_clicks_volume += 1;
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                }
                if is_key_pressed(self.move_left_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.menu_clicks_volume > 0 {
                        self.menu_clicks_volume -= 1;
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                }
                if is_key_pressed(KeyCode::Enter) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_state = StartupState::AudioSettings { audio_settings_state: AudioSettingsState::Standard };
                }
                // audio settings menu clicks volume navigation with keyboard done
            }
            StartupState::VideoSettings => {
                // video settings navigation with mouse
                if self.mouse_moved_buffer > 0.0 {
                    let mouse_pos = mouse_position();
                    let mouse_x = mouse_pos.0;
                    let mouse_y = mouse_pos.1;

                    // check mouse's x position
                    if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                        // if mouse's x position is between 278.0 and 1002.0, then it is in the width of the menu options, so check its y position
                        if mouse_pos.1 > 327.0 && mouse_pos.1 < 365.0 {
                            // if mouse's y position is between 327.0 and 365.0, then it is on menu option 0; menu_role = 0
                            self.startup_menu_role = 0;
                        } else if mouse_pos.1 > 365.0 && mouse_pos.1 < 403.0 {
                            // if mouse's y position is between 365.0 and 403.0, then it is on menu option 1; menu_role = 1
                            self.startup_menu_role = 1;
                        } else if mouse_pos.1 > 403.0 && mouse_pos.1 < 441.0 {
                            // if mouse's y position is between 403.0 and 441.0, then it is on menu option 2; menu_role = 2
                            self.startup_menu_role = 2;
                        }
                    }
                }
                // video settings navigation with mouse done

                // video settings navigation with keyboard
                if is_key_pressed(self.look_up_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 0 {
                        self.startup_menu_role = 2;
                    } else {
                        self.startup_menu_role -= 1;
                    }
                }
                if is_key_pressed(self.look_down_kb) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    if self.startup_menu_role == 2 {
                        self.startup_menu_role = 0;
                    } else {
                        self.startup_menu_role += 1;
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    self.startup_menu_role = 1;
                    self.startup_state = StartupState::Settings;
                }
                // video settings navigation with keyboard done

                // video settings options handling
                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    if self.menu_clicks_settings_toggle {
                        play_sound_once(&self.menu_click_sound);
                        set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                    }
                    match self.startup_menu_role {
                        0 => {
                            // Resolution
                        }
                        1 => {
                            // Brightness
                        }
                        2 => {
                            // Back
                            self.startup_menu_role = 2;
                            self.startup_state = StartupState::Settings;
                        }
                        _ => {}
                    }
                }
                // video settings options handling done
            }
            StartupState::ControlsSettings => {
                if !self.awaiting_kb_input {
                    // controls settings mouse navigation
                    if self.mouse_moved_buffer > 0.0 {
                        let mouse_pos = mouse_position();
                        let mouse_x = mouse_pos.0;
                        let mouse_y = mouse_pos.1;

                        // check mouse's x position
                        if mouse_pos.1 > 267.0 && mouse_pos.1 < 495.0 {
                            if mouse_pos.0 > 278.0 && mouse_pos.0 < 640.0 {
                                // if mouse's x position is between 278.0 and 640.0, then it is in the width of the first column, so check its y position
                                self.controls_settings_col = 0;
                            } else if mouse_pos.0 >= 640.0 && mouse_pos.0 < 1002.0 {
                                // if mouse's x position is between 640.0 and 1002.0, then it is in the width of the second column, so check its y position
                                self.controls_settings_col = 1;
                            }
                        }

                        // check mouse's y position
                        if mouse_pos.0 > 278.0 && mouse_pos.0 < 1002.0 {
                            if mouse_pos.1 > 267.0 && mouse_pos.1 < 305.0 {
                                // if mouse's y position is between 267.0 and 305.0, then it is on menu option 0; menu_role = 0
                                self.controls_settings_row = 0;
                            } else if mouse_pos.1 > 305.0 && mouse_pos.1 < 343.0 {
                                // if mouse's y position is between 305.0 and 343.0, then it is on menu option 1; menu_role = 1
                                self.controls_settings_row = 1;
                            } else if mouse_pos.1 > 343.0 && mouse_pos.1 < 381.0 {
                                // if mouse's y position is between 343.0 and 381.0, then it is on menu option 2; menu_role = 2
                                self.controls_settings_row = 2;
                            } else if mouse_pos.1 > 381.0 && mouse_pos.1 < 419.0 {
                                // if mouse's y position is between 381.0 and 419.0, then it is on menu option 2; menu_role = 3
                                self.controls_settings_row = 3;
                            } else if mouse_pos.1 > 419.0 && mouse_pos.1 < 457.0 {
                                // if mouse's y position is between 419.0 and 457.0, then it is on menu option 2; menu_role = 4
                                self.controls_settings_row = 4;
                            } else if mouse_pos.1 > 457.0 && mouse_pos.1 < 495.0 {
                                // if mouse's y position is between 457.0 and 495.0, then it is on menu option 2; menu_role = 4
                                self.controls_settings_row = 5;
                            }
                        }
                    }
                    // controls settings mouse navigation done

                    // controls settings keyboard navigation
                    if is_key_pressed(self.look_up_kb) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        if self.controls_settings_row == 0 {
                            self.controls_settings_row = 5;
                        } else {
                            self.controls_settings_row -= 1;
                        }
                    }
                    if is_key_pressed(self.look_down_kb) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        if self.controls_settings_row == 5 {
                            self.controls_settings_row = 0;
                        } else {
                            self.controls_settings_row += 1;
                        }
                    }
                    if self.controls_settings_row != 5 {
                        if is_key_pressed(self.move_right_kb) {
                            self.click_sound();
                            if self.controls_settings_col == 1 {
                                self.controls_settings_col = 0;
                            } else {
                                self.controls_settings_col += 1;
                            }
                        }
                        if is_key_pressed(self.move_left_kb) {
                            self.click_sound();
                            if self.controls_settings_col == 0 {
                                self.controls_settings_col = 1;
                            } else {
                                self.controls_settings_col -= 1;
                            }
                        }
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        self.startup_menu_role = 3;
                        self.startup_state = StartupState::Settings;
                        self.controls_settings_row = 0;
                        self.controls_settings_col = 0;
                        self.kb_to_change = Keybind::None;
                    }
                    // controls settings keyboard navigation done

                    // controls settings option handling
                    if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        if self.controls_settings_col == 0 {
                            // in first column
                            match self.controls_settings_row {
                                0 => {
                                    // Move Left
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::MoveLeft;
                                }
                                1 => {
                                    // Move Right
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::MoveRight;
                                }
                                2 => {
                                    // Look Up
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::LookUp;
                                }
                                3 => {
                                    // Look Down
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::LookDown;
                                }
                                4 => {
                                    // Jump
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::Jump;
                                }
                                5 => {
                                    // Back
                                    self.startup_menu_role = 3;
                                    self.startup_state = StartupState::Settings;
                                    self.controls_settings_row = 0;
                                    self.controls_settings_col = 0;
                                    self.kb_to_change = Keybind::None;
                                }
                                _ => {}
                            }
                        } else {
                            // in second column
                            match self.controls_settings_row {
                                0 => {
                                    // Dash/Sprint
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::DashSprint;
                                }
                                1 => {
                                    // Melee Attack
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::MeleeAttack;
                                }
                                2 => {
                                    // Ranged Attack
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::RangedAttack;
                                }
                                3 => {
                                    // Interact
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::Interact;
                                }
                                4 => {
                                    // Inventory
                                    self.awaiting_kb_input = true;
                                    self.kb_to_change = Keybind::Inventory;
                                }
                                5 => {
                                    // Back
                                    self.startup_menu_role = 3;
                                    self.startup_state = StartupState::Settings;
                                    self.controls_settings_row = 0;
                                    self.controls_settings_col = 0;
                                    self.kb_to_change = Keybind::None;
                                }
                                _ => {}
                            }
                        }
                    }
                    // controls settings option handling done
                } else {
                    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                        if self.menu_clicks_settings_toggle {
                            play_sound_once(&self.menu_click_sound);
                            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
                        }
                        self.awaiting_kb_input = false;
                    }

                    self.change_keybind();
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

    fn resolve_enemy_collisions(&self, enemy: &Enemy) -> bool {
        let colliding = {
            self.player.x < enemy.x + enemy.ewidth * TILE_SIZE &&
            self.player.x + self.player.pwidth * TILE_SIZE > enemy.x &&
            self.player.y - self.player.pheight * TILE_SIZE < enemy.y &&
            self.player.y > enemy.y - enemy.eheight * TILE_SIZE
        };
        colliding
    }

    fn resolve_player_attack_collisions(&self, enemy: &Enemy) -> bool {
        if !self.player.is_attacking {
            return false;
        }

        let colliding = {
            self.player.attack_x < enemy.x + enemy.ewidth * TILE_SIZE &&
            self.player.attack_x + self.player.attack_width > enemy.x &&
            self.player.attack_y < enemy.y &&
            self.player.attack_y + self.player.attack_height > enemy.y - enemy.eheight * TILE_SIZE
        };
        colliding
    }

    fn change_keybind(&mut self) {
        if get_last_key_pressed().is_none() {
            println!("no key pressed");
            return;
        }
        if let Some(key) = get_last_key_pressed() {
            self.awaiting_kb_input = false;
            if matches!(key, KeyCode::Enter) || matches!(key, KeyCode::Escape) {
                return;
            }
            let mut old_kb = Keybind::None;
            if self.kb_map.contains_key(&key) {
                old_kb = self.kb_map.get(&key).unwrap().clone();
            }
            let old_key;
            match self.kb_to_change {
                Keybind::MoveLeft => {
                    old_key = self.move_left_kb;
                    self.move_left_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::MoveRight => {
                    old_key = self.move_right_kb;
                    self.move_right_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::LookUp => {
                    old_key = self.look_up_kb;
                    self.look_up_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::LookDown => {
                    old_key = self.look_down_kb;
                    self.look_down_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::Jump => {
                    old_key = self.jump_kb;
                    self.jump_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::DashSprint => {
                    old_key = self.dash_sprint_kb;
                    self.dash_sprint_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::MeleeAttack => {
                    old_key = self.melee_attack_kb;
                    self.melee_attack_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::RangedAttack => {
                    old_key = self.ranged_attack_kb;
                    self.ranged_attack_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::Interact => {
                    old_key = self.interact_kb;
                    self.interact_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::Inventory => {
                    old_key = self.inventory_kb;
                    self.inventory_kb = key;
                    if self.kb_map.contains_key(&key) {
                        self.swap_keybinds(key, old_key, old_kb.clone());
                    }
                    self.kb_map.remove(&old_key);
                    self.kb_map.insert(old_key, old_kb);
                }
                Keybind::None => {
                    return;
                }
            }
        }
    }

    fn swap_keybinds(&mut self, key: KeyCode, old_key: KeyCode, old_kb: Keybind) {
        match old_kb {
            Keybind::MoveLeft => {
                self.move_left_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::MoveLeft);
            }
            Keybind::MoveRight => {
                self.move_right_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::MoveRight);
            }
            Keybind::LookUp => {
                self.look_up_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::LookUp);
            }
            Keybind::LookDown => {
                self.look_down_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::LookDown);
            }
            Keybind::Jump => {
                self.jump_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::Jump);
            }
            Keybind::DashSprint => {
                self.dash_sprint_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, self.kb_to_change.clone());
            }
            Keybind::MeleeAttack => {
                self.melee_attack_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::MeleeAttack);
            }
            Keybind::RangedAttack => {
                self.ranged_attack_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::RangedAttack);
            }
            Keybind::Interact => {
                self.interact_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::Interact);
            }
            Keybind::Inventory => {
                self.inventory_kb = old_key;
                //self.kb_map.remove(&old_key);
                //self.kb_map.insert(old_key, old_kb);
                self.kb_map.insert(key, Keybind::Inventory);
            }
            Keybind::None => {}
        }
    }

    fn click_sound(&self) {
        if self.menu_clicks_settings_toggle {
            play_sound_once(&self.menu_click_sound);
            set_sound_volume(&self.menu_click_sound, self.menu_clicks_volume as f32 / 10.0);
        }
    }
}

fn door_parser(line: &str) -> HashMap<char, Door> {
    //todo!("parse a line from a room file: split on tabs to separate doors, split on commas to separate each door into 1) identifier 2) file path for connected room 3) x coordinate in tiles for the player's spawnpoint in the new room 4) y coordinate in tiles for the player's spawnpoint in the new room")
    let mut door_map: HashMap<char, Door> = HashMap::new();
    if !line.is_empty() {
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
    }
    door_map
}