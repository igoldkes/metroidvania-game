use macroquad::prelude::*;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub is_jumping: bool,
    pub on_ground: bool,
    pub jump_buffer_time: f32,
    pub paused: bool,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vel_x: 0.0,
            vel_y: 0.0,
            is_jumping: false,
            on_ground: true,
            jump_buffer_time: 0.0,
            paused: false,
        }
    }

    pub fn update(&mut self, width: f32, height: f32, floor_y: f32, dt: f32) {
        const GRAVITY_UP: f32 = 800.0;
        const GRAVITY_DOWN: f32 = 1500.0;
        const JUMP_FORCE: f32 = -600.0;
        const JUMP_CUT: f32 = 0.1;
        const MOVE_SPEED: f32 = 300.0;
        
        if !self.paused {
            if is_key_down(KeyCode::Right) | is_key_down(KeyCode::D) {
                self.vel_x = MOVE_SPEED;
            } else if is_key_down(KeyCode::Left) | is_key_down(KeyCode::A) {
                self.vel_x = -MOVE_SPEED;
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
        draw_circle(self.x, self.y, 16.0, WHITE);
    }
}