use macroquad::prelude::*;

use super::Room;

const TILE_SIZE: f32 = 32.0;

enum XDirection {
    Right,
    Left,
}

enum YDirection {
    None,
    Up,
    Down,
}

enum AttackDirection {
    Right,
    Left,
    Up,
    Down,
}

pub struct Player {
    pub current_room: Room,
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
    jackie_paper_right_texture: Texture2D,
    jackie_paper_left_texture: Texture2D,
    jackie_paper_up_right_texture: Texture2D,
    jackie_paper_up_left_texture: Texture2D,
    jackie_paper_down_right_texture: Texture2D,
    jackie_paper_down_left_texture: Texture2D,
}

impl Player {
    pub fn new(current_room: Room, x: f32, y: f32, jackie_paper_right_texture: Texture2D, jackie_paper_left_texture: Texture2D, jackie_paper_up_right_texture: Texture2D, jackie_paper_up_left_texture: Texture2D, jackie_paper_down_right_texture: Texture2D, jackie_paper_down_left_texture: Texture2D) -> Self {
        Self {
            current_room,
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
            jackie_paper_right_texture,
            jackie_paper_left_texture,
            jackie_paper_up_right_texture,
            jackie_paper_up_left_texture,
            jackie_paper_down_right_texture,
            jackie_paper_down_left_texture,
        }
    }

    pub fn update(&mut self, width: f32, height: f32, floor_y: f32, dt: f32) {
        const GRAVITY_UP: f32 = 800.0;
        const GRAVITY_DOWN: f32 = 1500.0;
        const JUMP_FORCE: f32 = -600.0;
        const JUMP_CUT: f32 = 0.1;
        const MOVE_SPEED: f32 = 300.0;
        
        if !self.paused {
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

    pub fn draw(&self) {
        

        draw_rectangle(
            self.x,
            self.y - self.pheight as f32 * TILE_SIZE,
            self.pwidth as f32 * TILE_SIZE,
            self.pheight as f32 * TILE_SIZE,
            Color::from_rgba(255, 0, 0, 80),
        );
    }

    pub fn draw1(&self) {
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
    }

    fn check_collisions(&mut self, room: &mut Room) {
        let bottom_y = ((self.y - 0.0) / TILE_SIZE - 1.0) as i32;
        let top_y = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE) as i32;
        let left_x = (self.x / TILE_SIZE) as i32;
        let right_x = ((self.x + self.pwidth * TILE_SIZE) / TILE_SIZE) as i32;

        if self.vel_y >= 0.0 {
            // falling
            let next_bottom = ((self.y) / TILE_SIZE) as i32;
            if room.is_solid(left_x, next_bottom) || room.is_solid(right_x, next_bottom) {
                self.y = next_bottom as f32 * TILE_SIZE;
                self.vel_y = 0.0;
                self.on_ground = true;
            } else {
                self.on_ground = false;
            }
        } else if self.vel_y < 0.0 {
            // jumping
            let next_top = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE) as i32;
            if room.is_solid(left_x, next_top) || room.is_solid(right_x, next_top) {
                self.y = ((next_top + 1) as f32 + self.pheight) * TILE_SIZE + 1.0;
                self.vel_y = 0.0;
                self.is_jumping = false;
            }
        }

        if self.vel_x > 0.0 {
            // moving right
            let next_right = ((self.x + self.pwidth * TILE_SIZE) / TILE_SIZE) as i32;
            if room.is_solid(next_right, top_y) || room.is_solid(next_right, bottom_y) {
                self.x = (next_right as f32 - self.pwidth) * TILE_SIZE;
                self.vel_x = 0.0;
                self.hitting_wall_right = true;
            }
        } else if self.vel_x < 0.0 {
            // moving left
            let next_left = ((self.x) / TILE_SIZE) as i32;
            if room.is_solid(next_left, top_y) || room.is_solid(next_left, bottom_y) {
                self.x = (next_left + 1) as f32 * TILE_SIZE;
                self.vel_x = 0.0;
                self.hitting_wall_left = true;
            }
        }
    }

    fn resolve_horizontal_collisions(&mut self) {
        let top_y = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
        let bottom_y = (self.y / TILE_SIZE - 1.0).floor() as i32;
        let middle_y = ((self.y - self.pheight / 2.0 * TILE_SIZE) / TILE_SIZE).floor() as i32;

        if self.vel_x > 0.0 {
            // moving right
            
            let next_right = ((self.x + self.pwidth * TILE_SIZE) / TILE_SIZE).floor() as i32;
            draw_circle(
                next_right as f32 * TILE_SIZE,
                top_y as f32 * TILE_SIZE,
                16.0,
                RED,
            );
            draw_circle(
                next_right as f32 * TILE_SIZE,
                bottom_y as f32 * TILE_SIZE,
                16.0,
                RED,
            );
            draw_circle(
                next_right as f32 * TILE_SIZE,
                middle_y as f32 * TILE_SIZE,
                16.0,
                RED,
            );
            if self.current_room.is_solid(next_right, top_y) || self.current_room.is_solid(next_right, bottom_y) || self.current_room.is_solid(next_right, middle_y) {
                self.x = (next_right as f32) * TILE_SIZE - self.pwidth * TILE_SIZE;
                self.vel_x = 0.0;
                self.hitting_wall_right = true;
            } else {
                self.hitting_wall_right = false;
            }
        } else if self.vel_x < 0.0 {
            // moving left
            let next_left = (self.x / TILE_SIZE).floor() as i32;
            if self.current_room.is_solid(next_left, top_y) || self.current_room.is_solid(next_left, bottom_y) || self.current_room.is_solid(next_left, middle_y) {
                self.x = (next_left as f32 + 1.0) * TILE_SIZE;
                self.vel_x = 0.0;
                self.hitting_wall_left = true;
            } else {
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
            } else {
                self.on_ground = false;
            }
        } else if self.vel_y < 0.0 {
            // moving up (jumping)
            let next_top = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE - 0.5).floor() as i32;
            if self.current_room.is_solid(left_x, next_top) || self.current_room.is_solid(right_x, next_top) {
                self.y = ((next_top + 1) as f32 + self.pheight) * TILE_SIZE;
                self.vel_y = 0.0;
                //self.is_jumping = false;
            }
        }
    }
}