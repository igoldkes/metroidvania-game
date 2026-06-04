use macroquad::prelude::*;

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
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub is_jumping: bool,
    pub on_ground: bool,
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
    pub fn new(x: f32, y: f32, jackie_paper_right_texture: Texture2D, jackie_paper_left_texture: Texture2D, jackie_paper_up_right_texture: Texture2D, jackie_paper_up_left_texture: Texture2D, jackie_paper_down_right_texture: Texture2D, jackie_paper_down_left_texture: Texture2D) -> Self {
        Self {
            x,
            y,
            vel_x: 0.0,
            vel_y: 0.0,
            is_jumping: false,
            on_ground: true,
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
            if is_key_pressed(KeyCode::Right) | is_key_pressed(KeyCode::D) {
                self.x_direction = XDirection::Right;
            }
            if is_key_pressed(KeyCode::Left) | is_key_pressed(KeyCode::A) {
                self.x_direction = XDirection::Left;
            }
            if is_key_down(KeyCode::Up) | is_key_down(KeyCode::W) {
                self.y_direction = YDirection::Up;
            } else if is_key_down(KeyCode::Down) | is_key_down(KeyCode::S) {
                self.y_direction = YDirection::Down;
            } else {
                self.y_direction = YDirection::None;
            }

            if is_key_down(KeyCode::Right) | is_key_down(KeyCode::D) {
                self.vel_x = MOVE_SPEED;
                self.x_direction = XDirection::Right;
            } else if is_key_down(KeyCode::Left) | is_key_down(KeyCode::A) {
                self.vel_x = -MOVE_SPEED;
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

        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;

        if self.y >= floor_y {
            self.y = floor_y;
            self.vel_y = 0.0;
            self.on_ground = true;
            self.is_jumping = false;
        } else {
            self.on_ground = false;
        }

        self.x = clamp(self.x, 16.0, width - 16.0);
        self.y = clamp(self.y, 16.0, height - 16.0);
    }

    pub fn draw(&self) {
        match self.x_direction {
            XDirection::Right => {
                match self.y_direction {
                    YDirection::Up => {
                        draw_texture_ex(
                            &self.jackie_paper_up_right_texture,
                            self.x - 50.0,
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
                            self.x - 50.0,
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
                            self.x - 50.0,
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
                            self.x - 50.0,
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
                            self.x - 50.0,
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
                            self.x - 50.0,
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
        //draw_circle(self.x, self.y, 16.0, WHITE);
        
        if self.is_attacking {
            match self.attack_direction {
                AttackDirection::Right => {
                    draw_rectangle(
                        self.x + 16.0,
                        self.y - 6.0,
                        40.0,
                        12.0,
                        RED,
                    );
                }
                AttackDirection::Left => {
                    draw_rectangle(
                        self.x - 56.0,
                        self.y - 6.0,
                        40.0,
                        12.0,
                        RED,
                    );
                }
                AttackDirection::Up => {
                    draw_rectangle(
                        self.x - 6.0,
                        self.y - 56.0,
                        12.0,
                        40.0,
                        RED,
                    );
                }
                AttackDirection::Down => {
                    draw_rectangle(
                        self.x - 6.0,
                        self.y + 16.0,
                        12.0,
                        40.0,
                        RED,
                    );
                }
            }
        }
    }
}