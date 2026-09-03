use macroquad::prelude::*;
use macroquad::experimental::animation::{AnimatedSprite, Animation};

use std::collections::HashMap;

use super::player::{XDirection, YDirection, AttackDirection};

use super::{Room, Tile, Door};

const TILE_SIZE: f32 = 32.0;

#[derive(Clone, Debug)]
pub enum EnemyType {
    TestEnemy,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub aggrivated: bool,
    pub x: f32,
    pub y: f32,
    pub ewidth: f32, // in tiles
    pub eheight: f32, // in tiles
    pub vel_x: f32,
    pub vel_y: f32,
    pub is_jumping: bool,
    pub on_ground: bool,
    pub hitting_wall_right: bool,
    pub hitting_wall_left: bool,
    pub x_direction: XDirection,
    pub y_direction: YDirection,
    pub behavior_loop_buffer: f32,
    pub damaging_player: bool,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, x: f32, y: f32) -> Self {
        let ewidth = match enemy_type {
            EnemyType::TestEnemy => 1.0,
        };
        let eheight = match enemy_type {
            EnemyType::TestEnemy => 1.0,
        };

        Self {
            enemy_type,
            aggrivated: false,
            x,
            y,
            ewidth,
            eheight,
            vel_x: 0.0,
            vel_y: 0.0,
            is_jumping: false,
            on_ground: true,
            hitting_wall_right: false,
            hitting_wall_left: false,
            x_direction: XDirection::Right,
            y_direction: YDirection::None,
            behavior_loop_buffer: 0.0,
            damaging_player: false,
        }
    }

    pub fn update(&mut self, dt: f32, current_room_tile_map: HashMap<(i32, i32), Tile>, current_room_width: i32, current_room_height: i32) {
        const GRAVITY_UP: f32 = 1050.0;
        const GRAVITY_DOWN: f32 = 1500.0;

        self.damaging_player = false;

        if self.aggrivated {
            // engaged; go after player
            self.enemy_aggrivated_behavior(dt);
        } else {
            // not engaged; perform idle behavior
            self.enemy_idle_behavior(dt);
        }

        let gravity = if self.vel_y < 0.0 { GRAVITY_UP } else { GRAVITY_DOWN };
        self.vel_y += gravity * dt;

        self.on_ground = false;
        self.y += self.vel_y * dt;
        self.resolve_vertical_collisions(current_room_tile_map.clone(), current_room_width, current_room_height);
        
        self.hitting_wall_right = false;
        self.hitting_wall_left = false;
        self.x += self.vel_x * dt;
        self.resolve_horizontal_collisions(current_room_tile_map, current_room_width, current_room_height);
    }

    pub fn draw(&self) {
        draw_rectangle(
            self.x,
            self.y - self.eheight * TILE_SIZE,
            self.ewidth * TILE_SIZE,
            self.eheight * TILE_SIZE,
            Color::from_rgba(255, 255, 255, 255),
        );
    }

    fn enemy_idle_behavior(&mut self, dt: f32) {
        match self.enemy_type {
            EnemyType::TestEnemy => {
                self.behavior_loop_buffer -= dt;
                if self.behavior_loop_buffer <= 0.0 {
                    // reset behavior loop
                    self.behavior_loop_buffer = 4.0;
                } else if self.behavior_loop_buffer <= 2.0 {
                    // walk left for 2 seconds or until hitting an obstacle
                    self.x_direction = XDirection::Left;
                    if !self.hitting_wall_left {
                        self.vel_x = -100.0;
                    } else {
                        // hit a wall moving left; move right
                        self.behavior_loop_buffer = 0.0;
                    }
                } else {
                    // walk right for 2 seconds or until hitting an obstacle
                    self.x_direction = XDirection::Right;
                    if !self.hitting_wall_right {
                        self.vel_x = 100.0;
                    } else {
                        // hit a wall moving right; move left
                        self.behavior_loop_buffer = 2.0;
                    }
                }
            }
        }
    }

    fn enemy_aggrivated_behavior(&mut self, dt: f32) {
        match self.enemy_type {
            EnemyType::TestEnemy => {
                todo!();
            }
        }
    }

    fn resolve_horizontal_collisions(&mut self, current_room_tile_map: HashMap<(i32, i32), Tile>, current_room_width: i32, current_room_height: i32) {
        match self.eheight {
            1.0 => {
                // enemy is 1 tile tall
                let top_y = ((self.y - self.eheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
                let bottom_y = ((self.y - 1.0) / TILE_SIZE).floor() as i32;

                if self.vel_x > 0.0 {
                    // moving right
                    let next_right = ((self.x + self.ewidth * TILE_SIZE) / TILE_SIZE).floor() as i32;
                    if self.is_solid(next_right, top_y, current_room_tile_map.clone(), current_room_width, current_room_height) || self.is_solid(next_right, bottom_y, current_room_tile_map.clone(), current_room_width, current_room_height) || self.is_spikes(next_right, top_y, current_room_tile_map.clone()) || self.is_spikes(next_right, bottom_y, current_room_tile_map.clone()) {
                        self.x = (next_right as f32) * TILE_SIZE - self.ewidth * TILE_SIZE;
                        self.vel_x = 0.0;
                        self.hitting_wall_right = true;
                    } else {
                        self.hitting_wall_right = false;
                    }
                } else if self.vel_x < 0.0 {
                    // moving left
                    let next_left = (self.x / TILE_SIZE).floor() as i32;
                    if self.is_solid(next_left, top_y, current_room_tile_map.clone(), current_room_width, current_room_height) || self.is_solid(next_left, bottom_y, current_room_tile_map.clone(), current_room_width, current_room_height) || self.is_spikes(next_left, top_y, current_room_tile_map.clone()) || self.is_spikes(next_left, bottom_y, current_room_tile_map.clone()) {
                        self.x = (next_left as f32 + 1.0) * TILE_SIZE;
                        self.vel_x = 0.0;
                        self.hitting_wall_left = true;
                    } else {
                        self.hitting_wall_left = false;
                    }
                }
            },
            _ => todo!()
        }
    }

    fn resolve_vertical_collisions(&mut self, current_room_tile_map: HashMap<(i32, i32), Tile>, current_room_width: i32, current_room_height: i32) {
        match self.ewidth {
            1.0 => {
                let left_x = (self.x / TILE_SIZE).floor() as i32;
                let right_x = ((self.x + self.ewidth * TILE_SIZE - 1.0) / TILE_SIZE).floor() as i32;

                if self.vel_y >= 0.0 {
                    // enemy is falling
                    let next_bottom = (self.y / TILE_SIZE).floor() as i32;
                    if self.is_solid(left_x, next_bottom, current_room_tile_map.clone(), current_room_width, current_room_height) || self.is_solid(right_x, next_bottom, current_room_tile_map.clone(), current_room_width, current_room_height) {
                        self.y = next_bottom as f32 * TILE_SIZE;
                        self.vel_y = 0.0;
                        self.on_ground = true;
                        self.is_jumping = false;
                        // todo!("spike interaction")
                    }
                } else if self.vel_y < 0.0 {
                    // enemy is jumping
                    self.is_jumping = true;
                    let next_top = ((self.y - self.eheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
                    if self.is_solid(left_x, next_top, current_room_tile_map.clone(), current_room_width, current_room_height) || self.is_solid(right_x, next_top, current_room_tile_map.clone(), current_room_width, current_room_height) {
                        self.y = ((next_top + 1) as f32 + self.eheight) * TILE_SIZE;
                        self.vel_y = 0.0;
                    }
                }
            },
            _ => todo!()
        }
    }

    fn is_solid(&self, x: i32, y: i32, current_room_tile_map: HashMap<(i32, i32), Tile>, current_room_width: i32, current_room_height: i32) -> bool {
        if x < 0 || y < 0 || x > current_room_width || y > current_room_height {
            return false;
        }
        if current_room_tile_map.get(&(x, y)).is_none() {
            eprintln!("is_solid called with ({}, {}) which is not in the map", x, y);
            return false;
        }
        match *current_room_tile_map.get(&(x, y)).unwrap() {
            Tile::None => false,
            Tile::BrownBrick => true,
            Tile::GrayBrick => true,
            Tile::Door { .. } => false,
            Tile::Spikes => true,
        }
    }

    pub fn is_spikes(&self, x: i32, y: i32, current_room_tile_map: HashMap<(i32, i32), Tile>) -> bool {
        match *current_room_tile_map.get(&(x, y)).unwrap() {
            Tile::Spikes => true,
            _ => false,
        }
    }
}