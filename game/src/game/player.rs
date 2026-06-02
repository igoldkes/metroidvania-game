use macroquad::prelude::*;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub is_jumping: bool,
    pub on_ground: bool,
    pub jump_buffer_time: f32,
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
        }
    }
}