mod player;

use macroquad::prelude::*;

use player::Player;

pub struct GameState {
    player: Player,
    width: f32,
    height: f32,
    floor_y: f32,
}

impl GameState {
    pub fn new() -> Self {
        let width = screen_width();
        let height = screen_height();
        let floor_y = height - 100.0;

        let player = Player::new(width / 2.0, height / 2.0);

        Self {
            player,
            width,
            height,
            floor_y,
        }
    }

    pub fn update(&mut self, dt: f32) {
        const GRAVITY_UP: f32 = 800.0;
        const GRAVITY_DOWN: f32 = 1500.0;
        const JUMP_FORCE: f32 = -600.0;
        const JUMP_CUT: f32 = 0.1;
        const MOVE_SPEED: f32 = 300.0;
        
        if is_key_down(KeyCode::Right) | is_key_down(KeyCode::D) {
            self.player.vel_x = MOVE_SPEED;
        } else if is_key_down(KeyCode::Left) | is_key_down(KeyCode::A) {
            self.player.vel_x = -MOVE_SPEED;
        } else {
            self.player.vel_x = 0.0;
        }

        if is_key_pressed(KeyCode::Space) {
            self.player.jump_buffer_time = 0.15;
        }
        if self.player.jump_buffer_time > 0.0 {
            self.player.jump_buffer_time -= dt;
        }
        if self.player.jump_buffer_time > 0.0 && self.player.on_ground {
            self.player.vel_y = JUMP_FORCE;
            self.player.is_jumping = true;
            self.player.on_ground = false;
            self.player.jump_buffer_time = 0.0;
        }
        if is_key_released(KeyCode::Space) && self.player.vel_y < 0.0 {
            self.player.vel_y *= JUMP_CUT;
        }

        let gravity = if self.player.vel_y < 0.0 { GRAVITY_UP } else { GRAVITY_DOWN };
        
        self.player.vel_y += gravity * dt;

        self.player.x += self.player.vel_x * dt;
        self.player.y += self.player.vel_y * dt;

        if self.player.y >= self.floor_y {
            self.player.y = self.floor_y;
            self.player.vel_y = 0.0;
            self.player.on_ground = true;
            self.player.is_jumping = false;
        } else {
            self.player.on_ground = false;
        }

        self.player.x = clamp(self.player.x, 16.0, self.width - 16.0);
        self.player.y = clamp(self.player.y, 16.0, self.height - 16.0);
    }

    pub fn draw(&self) {
        clear_background(DARKGRAY);

        draw_circle(self.player.x, self.player.y, 16.0, WHITE);

        draw_rectangle(
            0.0,
            self.floor_y + 16.0,
            self.width,
            30.0,
            BROWN,
        );
    }
}