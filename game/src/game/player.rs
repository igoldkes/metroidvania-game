use macroquad::prelude::*;
use macroquad::experimental::animation::{AnimatedSprite, Animation};

use super::{Room, Tile, Door};

const TILE_SIZE: f32 = 32.0;

#[derive(Clone, Debug)]
pub enum XDirection {
    Right,
    Left,
}

#[derive(Clone, Debug)]
pub enum YDirection {
    None,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub enum AttackDirection {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub enum RoomChange {
    None,
    Change { door: Door },
}

pub struct Player {
    pub movement_blocked_buffer: f32,
    pub damage_blocked_buffer: f32,
    pub current_room: Room,
    pub room_change: RoomChange,
    pub x: f32,
    pub y: f32,
    pub pwidth: f32, // in tiles
    pub pheight: f32, // in tiles
    pub vel_x: f32,
    pub vel_y: f32,
    pub is_jumping: bool,
    pub on_ground: bool,
    pub hitting_wall_right: bool,
    pub hitting_wall_left: bool,
    pub jump_buffer_time: f32,
    pub paused: bool,
    pub is_attacking: bool,
    pub x_direction: XDirection,
    pub y_direction: YDirection,
    pub attack_direction: AttackDirection,
    pub attack_buffer_time: f32,
    //jackie_paper_right_texture: Texture2D,
    //jackie_paper_left_texture: Texture2D,
    //jackie_paper_up_right_texture: Texture2D,
    //jackie_paper_up_left_texture: Texture2D,
    //jackie_paper_down_right_texture: Texture2D,
    //jackie_paper_down_left_texture: Texture2D,
    jackie_paper_walking_texture: Texture2D,
    jackie_paper_walking_animation: AnimatedSprite,
    pub lives: usize,
}

impl Player {
    pub fn new(current_room: Room, x: f32, y: f32, jackie_paper_walking_texture: Texture2D, jackie_paper_walking_animation: AnimatedSprite) -> Self {
        Self {
            movement_blocked_buffer: 0.0,
            damage_blocked_buffer: 0.0,
            current_room,
            room_change: RoomChange::None,
            x,
            y,
            pwidth: 1.0,
            pheight: 2.0,
            vel_x: 0.0,
            vel_y: 0.0,
            is_jumping: false,
            on_ground: true,
            hitting_wall_right: false,
            hitting_wall_left: false,
            jump_buffer_time: 0.0,
            paused: false,
            is_attacking: false,
            x_direction: XDirection::Right,
            y_direction: YDirection::None,
            attack_direction: AttackDirection::Right,
            attack_buffer_time: 0.0,
            //jackie_paper_right_texture,
            //jackie_paper_left_texture,
            //jackie_paper_up_right_texture,
            //jackie_paper_up_left_texture,
            //jackie_paper_down_right_texture,
            //jackie_paper_down_left_texture,
            jackie_paper_walking_texture,
            jackie_paper_walking_animation,
            lives: 5,
        }
    }

    pub fn update(&mut self, width: f32, height: f32, floor_y: f32, dt: f32) {
        const GRAVITY_UP: f32 = 1050.0;
        const GRAVITY_DOWN: f32 = 1500.0;
        const JUMP_FORCE: f32 = -650.0;
        const JUMP_CUT: f32 = 0.1;
        const MOVE_SPEED: f32 = 300.0;

        if self.movement_blocked_buffer > 0.0 {
            self.movement_blocked_buffer -= dt;
        }

        if self.damage_blocked_buffer > 0.0 {
            self.damage_blocked_buffer -= dt;
        }

        if !self.paused && self.movement_blocked_buffer <= 0.0 {
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                self.x_direction = XDirection::Right;
            }
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                self.x_direction = XDirection::Left;
            }
            if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                self.y_direction = YDirection::Up;
            } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                self.y_direction = YDirection::Down;
            } else {
                self.y_direction = YDirection::None;
            }

            if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                if !self.hitting_wall_right {
                    self.vel_x = MOVE_SPEED;
                }
                self.x_direction = XDirection::Right;
            } else if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                if !self.hitting_wall_left {
                    self.vel_x = -MOVE_SPEED;
                }
                self.x_direction = XDirection::Left;
            } else {
                self.vel_x = 0.0;
            }

            if is_key_pressed(KeyCode::Space) {
                self.jump_buffer_time = 0.15;
            }
            if self.jump_buffer_time > 0.0 {
                self.jump_buffer_time -= dt;
            }
            if self.jump_buffer_time > 0.0 && self.on_ground {
                self.vel_y = JUMP_FORCE;
                self.is_jumping = true;
                self.on_ground = false;
                self.jump_buffer_time = 0.0;
            }
            if is_key_released(KeyCode::Space) && self.vel_y < 0.0 {
                self.vel_y *= JUMP_CUT;
            }

            if is_key_pressed(KeyCode::Semicolon) && !self.is_attacking {
                self.is_attacking = true;
                self.attack_buffer_time = 0.3;
                self.attack_direction = match self.y_direction {
                    YDirection::Up => {
                        AttackDirection::Up
                    }
                    YDirection::Down => {
                        AttackDirection::Down
                    }
                    YDirection::None => {
                        match self.x_direction {
                            XDirection::Right => {
                                AttackDirection::Right
                            }
                            XDirection::Left => {
                                AttackDirection::Left
                            }
                        }
                    }
                }
            }
            if self.attack_buffer_time > 0.0 {
                self.is_attacking = true;
                self.attack_buffer_time -= dt;
            } else {
                self.is_attacking = false;
            }
        } else {
            if self.on_ground {
                self.vel_x = 0.0;
                self.vel_y = 0.0;
            }
        }

        let gravity = if self.vel_y < 0.0 { GRAVITY_UP } else { GRAVITY_DOWN };
        
        self.vel_y += gravity * dt;
        
        
        self.on_ground = false;
        self.y += self.vel_y * dt;
        // resolve vertical collisions
        self.resolve_vertical_collisions();
        
        self.hitting_wall_right = false;
        self.hitting_wall_left = false;
        self.x += self.vel_x * dt;
        // resolve horizontal collisions
        self.resolve_horizontal_collisions();

        // update and set animations
        let walking = self.vel_x != 0.0;
        match self.x_direction {
            XDirection::Left => {
                if walking {
                    // moving left
                    if self.on_ground {
                        self.jackie_paper_walking_animation.set_animation(0);
                        self.jackie_paper_walking_animation.update();
                    } else if self.vel_y < 0.0 {
                        // moving up
                        self.jackie_paper_walking_animation.set_animation(2);
                        self.jackie_paper_walking_animation.update();
                    } else if self.vel_y > 0.0 {
                        // moving down
                        self.jackie_paper_walking_animation.set_animation(4);
                        self.jackie_paper_walking_animation.update();
                    }
                } else {
                    // standing still facing left

                }
            }
            XDirection::Right => {
                if walking {
                    // moving right
                    if self.on_ground {
                        self.jackie_paper_walking_animation.set_animation(1);
                        self.jackie_paper_walking_animation.update();
                    } else if self.vel_y < 0.0 {
                        // moving up
                        self.jackie_paper_walking_animation.set_animation(3);
                        self.jackie_paper_walking_animation.update();
                    } else if self.vel_y > 0.0 {
                        // moving down
                        self.jackie_paper_walking_animation.set_animation(5);
                        self.jackie_paper_walking_animation.update();
                    }
                } else {
                    // standing still facing right

                }
            }
        }

        //self.on_ground = false;
        //self.hitting_wall_right = false;
        //self.hitting_wall_left = false;
        //let mut room = self.current_room.clone();
        //self.check_collisions(&mut room);

        //if self.y >= floor_y {
        //    self.y = floor_y;
        //    self.vel_y = 0.0;
        //    self.on_ground = true;
        //    self.is_jumping = false;
        //} else {
        //    self.on_ground = false;
        //}

        //self.x = clamp(self.x, 16.0, width - 16.0);
        //self.y = clamp(self.y, 16.0, height - 16.0);
    }

/*    pub fn draw2(&self) {
        draw_rectangle(
            self.x,
            self.y - self.pheight as f32 * TILE_SIZE,
            self.pwidth as f32 * TILE_SIZE,
            self.pheight as f32 * TILE_SIZE,
            Color::from_rgba(255, 0, 0, 80),
        );

        match self.x_direction {
            XDirection::Right => {
                match self.y_direction {
                    YDirection::Up => {
                        draw_texture_ex(
                            &self.jackie_paper_up_right_texture,
                            self.x - 33.0,
                            self.y - 86.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(90.0, 90.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::Down => {
                        draw_texture_ex(
                            &self.jackie_paper_down_right_texture,
                            self.x - 33.0,
                            self.y - 86.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(90.0, 90.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::None => {
                        draw_texture_ex(
                            &self.jackie_paper_right_texture,
                            self.x - 33.0,
                            self.y - 86.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(90.0, 90.0)), // 67.5, 90.0
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            XDirection::Left => {
                match self.y_direction {
                    YDirection::Up => {
                        draw_texture_ex(
                            &self.jackie_paper_up_left_texture,
                            self.x - 26.0,
                            self.y - 86.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(90.0, 90.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::Down => {
                        draw_texture_ex(
                            &self.jackie_paper_down_left_texture,
                            self.x - 26.0,
                            self.y - 86.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(90.0, 90.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::None => {
                        draw_texture_ex(
                            &self.jackie_paper_left_texture,
                            self.x - 26.0,
                            self.y - 86.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(90.0, 90.0)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        if self.is_attacking {
            match self.attack_direction {
                AttackDirection::Right => {
                    draw_rectangle(
                        self.x + self.pwidth * TILE_SIZE,
                        self.y - self.pheight / 2.0 * TILE_SIZE - 6.0,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Left => {
                    draw_rectangle(
                        self.x - 40.0,
                        self.y - self.pheight / 2.0 * TILE_SIZE - 6.0,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Up => {
                    draw_rectangle(
                        self.x + self.pwidth / 2.0 * TILE_SIZE - 6.0,
                        self.y - self.pheight * TILE_SIZE - 40.0,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Down => {
                    draw_rectangle(
                        self.x + self.pwidth / 2.0 * TILE_SIZE - 6.0,
                        self.y,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
            }
        }
    }*/

    pub fn draw(&self) {
        if self.damage_blocked_buffer <= 0.0 {
            draw_rectangle(
                self.x,
                self.y - self.pheight as f32 * TILE_SIZE,
                self.pwidth as f32 * TILE_SIZE,
                self.pheight as f32 * TILE_SIZE,
                Color::from_rgba(255, 0, 0, 80),
            );
        } else {
            draw_rectangle(
                self.x,
                self.y - self.pheight as f32 * TILE_SIZE,
                self.pwidth as f32 * TILE_SIZE,
                self.pheight as f32 * TILE_SIZE,
                Color::from_rgba(0, 0, 255, 80),
            );
        }
            
        if self.is_attacking {
            match self.attack_direction {
                AttackDirection::Right => {
                    draw_rectangle(
                        self.x + self.pwidth * TILE_SIZE,
                        self.y - self.pheight / 2.0 * TILE_SIZE - 6.0,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Left => {
                    draw_rectangle(
                        self.x - 40.0,
                        self.y - self.pheight / 2.0 * TILE_SIZE - 6.0,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Up => {
                    draw_rectangle(
                        self.x + self.pwidth / 2.0 * TILE_SIZE - 6.0,
                        self.y - self.pheight * TILE_SIZE - 40.0,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Down => {
                    draw_rectangle(
                        self.x + self.pwidth / 2.0 * TILE_SIZE - 6.0,
                        self.y,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
            }
        }
    }

    pub fn draw1(&mut self) {
        draw_rectangle(
            self.x,
            self.y - self.pheight as f32 * TILE_SIZE,
            self.pwidth as f32 * TILE_SIZE,
            self.pheight as f32 * TILE_SIZE,
            Color::from_rgba(255, 0, 0, 80),
        );

        // check movement
        

        let walking_frame = self.jackie_paper_walking_animation.frame();
        draw_texture_ex(
            &self.jackie_paper_walking_texture,
            self.x - 26.0,
            self.y - 86.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(90.0, 90.0)),
                source: Some(walking_frame.source_rect),
                ..Default::default()
            },
        );

        if self.is_attacking {
            match self.attack_direction {
                AttackDirection::Right => {
                    draw_rectangle(
                        self.x + self.pwidth * TILE_SIZE,
                        self.y - self.pheight / 2.0 * TILE_SIZE - 6.0,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Left => {
                    draw_rectangle(
                        self.x - 40.0,
                        self.y - self.pheight / 2.0 * TILE_SIZE - 6.0,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Up => {
                    draw_rectangle(
                        self.x + self.pwidth / 2.0 * TILE_SIZE - 6.0,
                        self.y - self.pheight * TILE_SIZE - 40.0,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Down => {
                    draw_rectangle(
                        self.x + self.pwidth / 2.0 * TILE_SIZE - 6.0,
                        self.y,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
            }
        }

    }

/*    pub fn draw1(&self) {
        match self.x_direction {
            XDirection::Right => {
                match self.y_direction {
                    YDirection::Up => {
                        draw_texture_ex(
                            &self.jackie_paper_up_right_texture,
                            self.x - 55.0,
                            self.y - 80.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(100.0, 100.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::Down => {
                        draw_texture_ex(
                            &self.jackie_paper_down_right_texture,
                            self.x - 55.0,
                            self.y - 80.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(100.0, 100.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::None => {
                        draw_texture_ex(
                            &self.jackie_paper_right_texture,
                            self.x - 55.0,
                            self.y - 80.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(100.0, 100.0)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            XDirection::Left => {
                match self.y_direction {
                    YDirection::Up => {
                        draw_texture_ex(
                            &self.jackie_paper_up_left_texture,
                            self.x - 45.0,
                            self.y - 80.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(100.0, 100.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::Down => {
                        draw_texture_ex(
                            &self.jackie_paper_down_left_texture,
                            self.x - 45.0,
                            self.y - 80.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(100.0, 100.0)),
                                ..Default::default()
                            },
                        );
                    }
                    YDirection::None => {
                        draw_texture_ex(
                            &self.jackie_paper_left_texture,
                            self.x - 45.0,
                            self.y - 80.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(100.0, 100.0)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }
        
        draw_rectangle(
            self.x - 26.0,
            self.y - 69.0,
            52.0,
            85.0,
            Color::from_rgba(255, 0, 0, 80),
        );
        draw_circle(self.x, self.y, 16.0, WHITE);

        if self.is_attacking {
            match self.attack_direction {
                AttackDirection::Right => {
                    draw_rectangle(
                        self.x + 26.0,
                        self.y - 26.5,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Left => {
                    draw_rectangle(
                        self.x - 66.0,
                        self.y - 26.5,
                        40.0,
                        12.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Up => {
                    draw_rectangle(
                        self.x - 6.0,
                        self.y - 109.0,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
                AttackDirection::Down => {
                    draw_rectangle(
                        self.x - 6.0,
                        self.y + 16.0,
                        12.0,
                        40.0,
                        Color::from_rgba(0, 255, 0, 80),
                    );
                }
            }
        }
    }*/

    fn resolve_horizontal_collisions(&mut self) {
        let top_y = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
        let bottom_y = ((self.y - 1.0) / TILE_SIZE).floor() as i32;
        let middle_y = ((self.y - self.pheight / 2.0 * TILE_SIZE) / TILE_SIZE).floor() as i32;

        if self.vel_x > 0.0 {
            // moving right
            
            let next_right = ((self.x + self.pwidth * TILE_SIZE) / TILE_SIZE).floor() as i32;
            if self.current_room.is_solid(next_right, top_y) || self.current_room.is_solid(next_right, bottom_y) /*|| self.current_room.is_solid(next_right, middle_y)*/ {
                self.x = (next_right as f32) * TILE_SIZE - self.pwidth * TILE_SIZE;
                self.vel_x = 0.0;
                self.hitting_wall_right = true;
                if self.damage_blocked_buffer <= 0.0 {
                    if self.current_room.is_spikes(next_right, top_y) || self.current_room.is_spikes(next_right, bottom_y) /*|| self.current_room.is_spikes(next_right, middle_y)*/ {
                        println!("spikes!");
                        self.lives -= 1;
                        self.damage_blocked_buffer = 2.0;
                    }
                }
            } else {
                if self.on_ground {
                    if self.current_room.is_door(next_right - 1, bottom_y) && self.current_room.is_door(next_right - 1, middle_y) {
                        //println!("hi");
                        //std::process::exit(0);
                        //todo!("load new room");
                        let identifier = match self.current_room.tile_map.get(&(next_right - 1, bottom_y)).unwrap() {
                            Tile::Door { identifier: c } => c,
                            _ => &'z',
                        };
                        //println!("made it");
                        let door = self.current_room.door_map.get(identifier).unwrap();
                        let string = format!("{:?}", door);
                        println!("{}, identifier: {}", string, identifier);
                        self.room_change = RoomChange::Change { door: door.clone() };
                    }
                } else {
                    if self.current_room.is_door(next_right - 1, bottom_y) && self.current_room.is_door(next_right - 1, middle_y) && self.current_room.is_door(next_right - 1, top_y) {
                        //std::process::exit(0);
                        //todo!("load new room");
                        let identifier = match self.current_room.tile_map.get(&(next_right - 1, bottom_y)).unwrap() {
                            Tile::Door { identifier: c } => c,
                            _ => &'z',
                        };
                        //println!("made it");
                        let door = self.current_room.door_map.get(identifier).unwrap();
                        let string = format!("{:?}", door);
                        println!("{}, identifier: {}", string, identifier);
                        self.room_change = RoomChange::Change { door: door.clone() };
                    }
                }
                self.hitting_wall_right = false;
            }
        } else if self.vel_x < 0.0 {
            // moving left

            let next_left = (self.x / TILE_SIZE).floor() as i32;
            if self.current_room.is_solid(next_left, top_y) || self.current_room.is_solid(next_left, bottom_y) || self.current_room.is_solid(next_left, middle_y) {
                self.x = (next_left as f32 + 1.0) * TILE_SIZE;
                self.vel_x = 0.0;
                self.hitting_wall_left = true;
                if self.damage_blocked_buffer <=0.0 {
                    if self.current_room.is_spikes(next_left, top_y) || self.current_room.is_spikes(next_left, bottom_y) || self.current_room.is_spikes(next_left, middle_y) {
                        println!("spikes!");
                        self.lives -= 1;
                        self.damage_blocked_buffer = 2.0;
                    }
                }
            } else {
                if self.on_ground {
                    if self.current_room.is_door(next_left + 1, bottom_y) && self.current_room.is_door(next_left + 1, middle_y) {
                        println!("hello");
                        //std::process::exit(0);
                        //todo!("load new room");
                        let identifier = match self.current_room.tile_map.get(&(next_left + 1, bottom_y)).unwrap() {
                            Tile::Door { identifier: c } => c,
                            _ => &'z',
                        };
                        let door = self.current_room.door_map.get(identifier).unwrap();
                        let string = format!("{:?}", door);
                        println!("{}", string);
                        self.room_change = RoomChange::Change { door: door.clone() };
                    }
                } else {
                    if self.current_room.is_door(next_left + 1, bottom_y) && self.current_room.is_door(next_left + 1, middle_y) && self.current_room.is_door(next_left + 1, top_y) {
                        //println!("hello");
                        //std::process::exit(0);
                        //todo!("load new room");
                        let identifier = match self.current_room.tile_map.get(&(next_left + 1, bottom_y)).unwrap() {
                            Tile::Door { identifier: c } => c,
                            _ => &'z',
                        };
                        let door = self.current_room.door_map.get(identifier).unwrap();
                        let string = format!("{:?}", door);
                        println!("{}", string);
                        self.room_change = RoomChange::Change { door: door.clone() };
                    }
                }
                self.hitting_wall_left = false;
            }
        }
    }

    fn resolve_vertical_collisions(&mut self) {
        let left_x = (self.x / TILE_SIZE).floor() as i32;
        let right_x = ((self.x + self.pwidth * TILE_SIZE - 1.0) / TILE_SIZE).floor() as i32;

        if self.vel_y >= 0.0 {
            // moving down (falling)
            let next_bottom = (self.y / TILE_SIZE).floor() as i32;
            if self.current_room.is_solid(left_x, next_bottom) || self.current_room.is_solid(right_x, next_bottom) {
                self.y = next_bottom as f32 * TILE_SIZE;
                self.vel_y = 0.0;
                self.on_ground = true;
                if self.current_room.is_spikes(left_x, next_bottom) && self.current_room.is_spikes(right_x, next_bottom) && self.damage_blocked_buffer <= 0.0 {
                    //println!("spikes!");
                    self.lives -= 1;
                    self.damage_blocked_buffer = 2.0;
                }
            } else {
                if self.current_room.is_door(left_x, next_bottom - 1) && self.current_room.is_door(right_x, next_bottom - 1) {
                    //println!("heya");
                    //std::process::exit(0);
                    //todo!("load new room");
                    let identifier = match self.current_room.tile_map.get(&(left_x, next_bottom - 1)).unwrap() {
                        Tile::Door { identifier: c } => c,
                        _ => &'z',
                    };
                    let door = self.current_room.door_map.get(identifier).unwrap();
                    let string = format!("{:?}", door);
                    println!("{}", string);
                    self.room_change = RoomChange::Change { door: door.clone() };
                }
                self.on_ground = false;
            }
        } else if self.vel_y < 0.0 {
            // moving up (jumping)
            let next_top = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
            if self.current_room.is_solid(left_x, next_top) || self.current_room.is_solid(right_x, next_top) {
                self.y = ((next_top + 1) as f32 + self.pheight) * TILE_SIZE;
                self.vel_y = 0.0;
                //self.is_jumping = false;
                if self.current_room.is_spikes(left_x, next_top) && self.current_room.is_spikes(right_x, next_top) && self.damage_blocked_buffer <= 0.0 {
                    println!("spikes!");
                    self.lives -= 1;
                    self.damage_blocked_buffer = 2.0;
                }
            } else {
                if self.current_room.is_door(left_x, next_top + 1) && self.current_room.is_door(right_x, next_top + 1) {
                    println!("bello");
                    //std::process::exit(0);
                    //todo!("load new room");
                    let identifier = match self.current_room.tile_map.get(&(left_x, next_top + 1)).unwrap() {
                        Tile::Door { identifier: c } => c,
                        _ => &'z',
                    };
                    let door = self.current_room.door_map.get(identifier).unwrap();
                    let string = format!("{:?}", door);
                    println!("{}", string);
                    self.room_change = RoomChange::Change { door: door.clone() };
                }
            }
        }
    }
}