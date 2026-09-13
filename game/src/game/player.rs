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
    pub attack_x: f32,
    pub attack_y: f32,
    pub attack_width: f32,
    pub attack_height: f32,
    pub knockback_vel_x: f32,
    pub knockback_vel_y: f32,
    pub double_jump_enabled: bool,
    pub wall_jump_enabled: bool,
    pub dash_enabled: bool,
    pub sprint_enabled: bool,
    pub dash_buffer: f32,
    pub dashed: bool,
    pub sprinting: bool,
    pub on_wall: bool,
    pub wall_jump_buffer: f32,
    pub on_wall_buffer: f32,
    pub wall_slide_timer: f32,
    pub dash_direction: f32,
    pub double_jumped: bool,
    pub is_double_jumping: bool,
    pub dash_cooldown: f32,
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
            attack_x: x + TILE_SIZE,
            attack_y: y - TILE_SIZE - 6.0,
            attack_width: 1.0 * TILE_SIZE,
            attack_height: 1.25 * TILE_SIZE,
            knockback_vel_x: 0.0,
            knockback_vel_y: 0.0,
            double_jump_enabled: true,
            wall_jump_enabled: true,
            dash_enabled: true,
            sprint_enabled: true,
            dash_buffer: 0.0,
            dashed: false,
            sprinting: false,
            on_wall: false,
            wall_jump_buffer: 0.0,
            on_wall_buffer: 0.0,
            wall_slide_timer: 0.0,
            dash_direction: 1.0,
            double_jumped: false,
            is_double_jumping: false,
            dash_cooldown: 0.0,
        }
    }

    pub fn update(&mut self, width: f32, height: f32, floor_y: f32, dt: f32) {
        const GRAVITY_UP: f32 = 1050.0;
        const GRAVITY_DOWN: f32 = 1500.0;
        const JUMP_FORCE: f32 = -650.0;
        const JUMP_CUT: f32 = 0.1;
        const MOVE_SPEED: f32 = 300.0;
        const SPRINT_SPEED: f32 = 450.0;
        const DASH_SPEED: f32 = 800.0;
        const KNOCKBACK_DECAY: f32 = 400.0;

        // BUFFER DECREMENTS
        if self.movement_blocked_buffer > 0.0 {
            self.movement_blocked_buffer -= dt;
        }

        if self.damage_blocked_buffer > 0.0 {
            self.damage_blocked_buffer -= dt;
        }
        // BUFFER DECREMENTS DONE

        if !self.paused && self.movement_blocked_buffer <= 0.0 {
            // if game is not paused and player can move, then run the corresponding logic

            // INITIAL DIRECTION SET
            if is_key_pressed(KeyCode::D) && self.wall_jump_buffer <= 0.0 {
                // if D is pressed and x_direction is not briefly locked because player just jumped off of a wall, player is facing right
                self.x_direction = XDirection::Right;
            }
            if is_key_pressed(KeyCode::A) && self.wall_jump_buffer <= 0.0 {
                // if A is pressed and x_direction is not briefly locked because player just jumped off of a wall, player is facing left
                self.x_direction = XDirection::Left;
            }
            if is_key_down(KeyCode::W) {
                // if W is being held down, player is looking up
                self.y_direction = YDirection::Up;
            } else if is_key_down(KeyCode::S) {
                // if W is not being held down, and S is, player is looking down
                self.y_direction = YDirection::Down;
            } else {
                // if neither W nor S are being held down, player is neither looking up nor down
                self.y_direction = YDirection::None;
            }
            // INITIAL DIRECTIONS SET DONE


            // HORIZONTAL MOVEMENT INPUT DETECTION
            if is_key_down(KeyCode::D) {
                if !self.hitting_wall_right && self.wall_jump_buffer <= 0.0 {
                    // self.on_wall = false;
                    if self.sprinting {
                        self.vel_x = SPRINT_SPEED;
                    } else {
                        self.vel_x = MOVE_SPEED;
                    }
                }
                if self.wall_jump_buffer <= 0.0 {
                    self.x_direction = XDirection::Right;
                }
            } else if is_key_down(KeyCode::A) {
                if !self.hitting_wall_left && self.wall_jump_buffer <= 0.0 {
                    // self.on_wall = false;
                    if self.sprinting {
                        self.vel_x = -SPRINT_SPEED;
                    } else {
                        self.vel_x = -MOVE_SPEED;
                    }
                }
                if self.wall_jump_buffer <= 0.0 {
                    self.x_direction = XDirection::Left;
                }
            } else {
                // self.on_wall = false;
                self.vel_x = 0.0;
            }
            // HORIZONTAL MOVEMENT INPUT DETECTION DONE


            // VERTICAL MOVEMENT INPUT DETECTION
            if is_key_pressed(KeyCode::Space) {
                // jump-queuing: if spacebar is pressed, count down a timer from 0.15s
                self.jump_buffer_time = 0.15;
            }
            if self.jump_buffer_time > 0.0 {
                // if jump-queue timer is greater than 0.0, then register an attempted jump input and decrement the timer
                if self.on_ground && self.dash_buffer <= 0.0 {
                    // if player is on the ground and not currently dashing, then allow the jump and set the jump-queue timer to 0.0
                    self.vel_y = JUMP_FORCE;
                    self.is_jumping = true;
                    self.on_ground = false;
                    self.jump_buffer_time = 0.0;
                // todo!("add code for the cases that player is not on the ground and dashing, and the player is on the ground and dashing, and not on the ground and not dashing")
                } else if !self.on_ground && self.dash_buffer <= 0.0 && self.double_jump_enabled && !self.double_jumped {
                    self.vel_y = JUMP_FORCE;
                    self.is_double_jumping = true;
                    self.double_jumped = true;
                    self.jump_buffer_time = 0.0;
                }
            }
            if is_key_released(KeyCode::Space) && self.vel_y < 0.0 {
                // if spacebar is released and player is moving upward, apply jump cut
                self.vel_y *= JUMP_CUT;
                self.is_double_jumping = false;
            } else if self.vel_y > 0.0 {
                self.is_double_jumping = false;
            }

            if self.on_ground || self.on_wall {
                self.double_jumped = false;
                self.is_double_jumping = false;
            }
            // VERTICAL MOVEMENT INPUT DETECTION DONE


            // DASH INPUT DETECTION
            if is_key_pressed(KeyCode::LeftShift) && self.dash_buffer <= 0.0 && !self.dashed && self.dash_cooldown <= 0.0 {
                // if left shift is pressed and player is not currently dashing and player has not already dashed, then dash for 0.15s
                self.dash_buffer = 0.15;
                self.dash_cooldown = 0.65;
                self.dash_direction = match self.x_direction {
                    XDirection::Right => {
                        if self.on_wall {
                            // if player is facing right and on a wall, then dash leftward away from the wall; vel_x < 0.0
                            self.on_wall = false;
                            -1.0
                        } else {
                            // if player is facing right and not on a wall, dash rightward; vel_x > 0.0
                            1.0
                        }
                    }
                    XDirection::Left => {
                        if self.on_wall {
                            // if player is facing left and on a wall, then dash rightward away from the wall; vel_x > 0.0
                            self.on_wall = false;
                            1.0
                        } else {
                            // if player is facing left and not on a wall, then dash leftward; vel_x < 0.0
                            -1.0
                        }
                    }
                };
            }
            if self.on_ground || self.on_wall {
                // if player is on the groudn or on a wall, then player can dash again
                self.dashed = false;
            }
            if self.dash_buffer > 0.0 {
                // if player is currently dashing, then decrement the dash buffer, set dashed to true, stop any current attacks, and set horizontal movement speed to dash speed
                self.is_attacking = false;
                self.attack_buffer_time = 0.0;
                self.dash_buffer -= dt;
                let dash_speed = DASH_SPEED;
                if self.knockback_vel_x == 0.0 {
                    // if player is not being knocked back, then perform the dash (i.e., knockback interrupts dashing)
                    self.vel_x = dash_speed * self.dash_direction
                } else {
                    self.dash_cooldown = 0.0;
                }
                self.x_direction = match self.dash_direction {
                    1.0 => {
                        XDirection::Right
                    }
                    -1.0 => {
                        XDirection::Left
                    }
                    _ => {
                        XDirection::Right
                    }
                };
            }
            if self.dash_cooldown > 0.0 {
                self.dash_cooldown -= dt;
            }
            // DASH INPUT DETECTION DONE


            // SPRINT INPUT DETECTION
            if is_key_down(KeyCode::LeftShift) && self.sprint_enabled {
                // if left shift is being held down, and can sprint, then set sprinting to true
                self.sprinting = true;
            } else {
                // if either left shift is not being held down, or player cannot sprint, then set sprinting to false
                self.sprinting = false;
            }
            // SPRINT INPUT DETECTION DONE

            // WALL JUMP INPUT DETECTION AND LOGIC
            if self.wall_jump_buffer > 0.0 {
                self.wall_jump_buffer -= dt;
                let direction = match self.x_direction {
                    XDirection::Right => {
                        1.0
                    }
                    XDirection::Left => {
                        -1.0
                    }
                };
                self.vel_x = direction * MOVE_SPEED;
            }
            if self.on_wall_right() || self.on_wall_left() && self.wall_jump_enabled {
                // if player is not on the ground, can wall jump, and is up against a wall either to the right or the left, then set on_wall to true
                self.on_wall = true;
            } else {
                // if player either cannot jump or both on_wall_right() and on_wall_left() return false, then set on_wall to false
                self.on_wall = false;
            }
            
            if self.on_wall {
                // if player is on a wall, increment the wall slide timer
                self.wall_slide_timer += dt;
            } else {
                // if player is not on a wall, set the wall slide timer to 0.0
                self.wall_slide_timer = 0.0;
            }

            if self.wall_jump_enabled && self.on_wall {
                // if player is on a wall and can wall jump, detect input for wall jumps
                if is_key_pressed(KeyCode::Space) {
                    // if spacebar is pressed, perform the wall jump
                    self.vel_y = JUMP_FORCE;
                    self.is_jumping = true;
                    self.on_wall = false;
                    self.jump_buffer_time = 0.0;
                    self.wall_jump_buffer = 0.15;
                    self.x_direction = match self.x_direction {
                        XDirection::Right => {
                            XDirection::Left
                        }
                        XDirection::Left => {
                            XDirection::Right
                        }
                    };
                }
                if is_key_released(KeyCode::Space) && self.vel_y < 0.0 {
                    // if spacebar is released and player is moving upward, apply jump cut
                    self.vel_y *= JUMP_CUT;
                }
            }
            // WALL JUMP INPUT DETECTION AND LOGIC DONE


            // ATTACK INPUT DETECTION AND LOGIC
            if is_key_pressed(KeyCode::Semicolon) && !self.is_attacking && self.dash_buffer <= 0.0 {
                // if smeicolon is pressed and player is not currently attacking or dashing, then perform an attack
                self.is_attacking = true;
                self.attack_buffer_time = 0.3;
                self.attack_direction = match self.y_direction {
                    YDirection::Up => {
                        // if player is facing up, attack upward
                        AttackDirection::Up
                    }
                    YDirection::Down => {
                        // if player is facing down, attack downward
                        AttackDirection::Down
                    }
                    YDirection::None => {
                        // if player is neither facing up nor down, check x_direction
                        match self.x_direction {
                            XDirection::Right => {
                                if self.on_wall {
                                    // if player is on a wall and facing right, then attack left (away from the wall)
                                    AttackDirection::Left
                                } else {
                                    // if player is not on a wall, attack right
                                    AttackDirection::Right
                                }
                            }
                            XDirection::Left => {
                                if self.on_wall {
                                    // if player is on a wall and facing left, then attack right
                                    AttackDirection::Right
                                } else {
                                    // if player is ot on a wall, attack left
                                    AttackDirection::Left
                                }
                                
                            }
                        }
                    }
                };
            }
            if self.attack_buffer_time > 0.0 {
                // if player is attacking, decrement it and keep is_attacking as true
                self.is_attacking = true;
                self.attack_buffer_time -= dt;
            } else {
                // if player is not attacking, set is_attacking to false
                self.is_attacking = false;
            }
            // ATTACK INPUT DETECTION AND LOGIC DONE

        } else {
            // if either game is paused or player's movement is blocked, then wait until player is on the ground and then stop moving
            if self.on_ground {
                self.vel_x = 0.0;
                self.vel_y = 0.0;
            }
        }
        //  notes:  on_wall checks seem to either be redundantly done in the WALL JUMP INPUT DETECTION AND LOGIC section or the HORIZONTAL MOVEMENT INPUT DETECTION section
        //          wall_slide_timer logic seems like it can be put inside of the if blocks that set on_wall accordingly, instead of being in a separate conditional that itself checks on_wall
        //          if wall_jump_enabled is checked any time on_wall would be set to true, then it seems like it would not need to be checked again in the WALL JUMP INPUT DETECTION AND LOGIC section as well


        // KNOCKBACK LOGIC
        if self.knockback_vel_x != 0.0 {
            // if player is being knocked back, then apply knockback to horizontal velocity
            self.vel_x = self.knockback_vel_x;
        }
        if self.knockback_vel_y != 0.0 {
            // if player is being knocked back, then apply knockback to the vertical velocity
            self.vel_y = self.knockback_vel_y;
        }

        if self.knockback_vel_x > 0.0 {
            self.knockback_vel_x = (self.knockback_vel_x - KNOCKBACK_DECAY * dt).max(0.0);
        } else if self.knockback_vel_x < 0.0 {
            self.knockback_vel_x = (self.knockback_vel_x + KNOCKBACK_DECAY * dt).min(0.0);
        }
        if self.knockback_vel_y > 0.0 {
            self.knockback_vel_y = (self.knockback_vel_y - KNOCKBACK_DECAY * dt).max(0.0);
        } else if self.knockback_vel_y < 0.0 {
            self.knockback_vel_y = (self.knockback_vel_y + KNOCKBACK_DECAY * dt).min(0.0);
        }
        // KNOCKBACK LOGIC DONE


        // POSITION UPDATES AND COLLISION RESOLUTION
        let gravity = if self.vel_y < 0.0 {
            GRAVITY_UP
        } else if self.vel_y >= 0.0 && !self.on_wall {
            GRAVITY_DOWN
        } else {
            GRAVITY_DOWN / 3.0 * power(self.wall_slide_timer, 3).min(1.0)
        };

        if self.dash_buffer <= 0.0 || self.on_wall_buffer > 0.0 {
            self.vel_y += gravity * dt;
        }

        self.on_ground = false;
        if self.dash_buffer <= 0.0 {
            self.y += self.vel_y * dt;
        } else {
            // falling momentum resets when dash is complete and player resumes falling
            self.vel_y = 0.0;
        }
        self.resolve_vertical_collisions();

        self.hitting_wall_right = false;
        self.hitting_wall_left = false;
        self.x += self.vel_x * dt;
        self.resolve_horizontal_collisions();
        // POSITION UPDATES AND COLLISION RESOLUTION DONE
    }

    pub fn update1(&mut self, width: f32, height: f32, floor_y: f32, dt: f32) {
        const GRAVITY_UP: f32 = 1050.0;
        const GRAVITY_DOWN: f32 = 1500.0;
        const JUMP_FORCE: f32 = -650.0;
        const JUMP_CUT: f32 = 0.1;
        const MOVE_SPEED: f32 = 300.0;
        const SPRINT_SPEED: f32 = 450.0;

        println!("{}", self.on_wall_left());

        if self.on_wall {
            self.on_wall_buffer = 0.05;
        } else if self.on_wall_buffer > 0.0 {
            self.on_wall_buffer -= dt;
        }

        if self.movement_blocked_buffer > 0.0 {
            self.movement_blocked_buffer -= dt;
        }

        if self.damage_blocked_buffer > 0.0 {
            self.damage_blocked_buffer -= dt;
        }

        if self.knockback_vel_x > 0.0 {
            self.knockback_vel_x = (self.knockback_vel_x - 400.0 * dt).max(0.0);
        } else if self.knockback_vel_x < 0.0 {
            self.knockback_vel_x = (self.knockback_vel_x + 400.0 * dt).min(0.0);
        }
        if self.knockback_vel_y > 0.0 {
            self.knockback_vel_y = (self.knockback_vel_y - 400.0 * dt).max(0.0);
        } else if self.knockback_vel_y < 0.0 {
            self.knockback_vel_y = (self.knockback_vel_y + 400.0 * dt).min(0.0);
        }

        if !self.paused && self.movement_blocked_buffer <= 0.0 {
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) && self.wall_jump_buffer <= 0.0 {
                self.x_direction = XDirection::Right;
            }
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) && self.wall_jump_buffer <= 0.0 {
                self.x_direction = XDirection::Left;
            }
            if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                self.y_direction = YDirection::Up;
            } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                self.y_direction = YDirection::Down;
            } else {
                self.y_direction = YDirection::None;
            }

            if self.wall_jump_buffer > 0.0 {
                self.wall_jump_buffer -= dt;
                let direction = match self.x_direction {
                    XDirection::Right => {
                        1.0
                    }
                    XDirection::Left => {
                        -1.0
                    }
                };
                self.vel_x = direction * MOVE_SPEED;
            }

            /*if self.on_wall {
                println!("on wall");
                if self.wall_jump_buffer > 0.0 {
                    println!("jumped");
                }
            }*/

            if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                if !self.hitting_wall_right && self.wall_jump_buffer <= 0.0 && !self.on_wall_right() {
                    self.on_wall = false;
                    if self.sprinting {
                        self.vel_x = SPRINT_SPEED;
                    } else {
                        self.vel_x = MOVE_SPEED;
                    }
                } else if self.wall_jump_enabled && !self.on_ground {
                    self.on_wall = true;
                    //self.vel_x = 0.0;
                }
                if self.wall_jump_buffer <= 0.0 {
                    self.x_direction = XDirection::Right;
                }
            } else if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                if !self.hitting_wall_left && self.wall_jump_buffer <= 0.0 {
                    self.on_wall = false;
                    if self.sprinting {
                        self.vel_x = -SPRINT_SPEED;
                    } else {
                        self.vel_x = -MOVE_SPEED;
                    }
                } else if self.wall_jump_enabled && !self.on_ground {
                    self.on_wall = true;
                    //self.vel_x = 0.0;
                }
                if self.wall_jump_buffer <= 0.0 {
                    self.x_direction = XDirection::Left;
                }
            } else {
                self.on_wall = false;
                self.vel_x = 0.0;
            }

            if is_key_pressed(KeyCode::Space) {
                self.jump_buffer_time = 0.15;
            }
            if self.jump_buffer_time > 0.0 && self.dash_buffer <= 0.0 {
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
            
            // sprinting
            if is_key_down(KeyCode::LeftShift) {
                self.sprinting = true;
            }
            if is_key_released(KeyCode::LeftShift) {
                self.sprinting = false;
            }

            // dashing
            if self.on_ground || self.on_wall_buffer > 0.0 {
                self.dashed = false;
            }
            if self.dash_buffer > 0.0 {
                self.dash_buffer -= dt;
                self.dash();
            }
            if is_key_pressed(KeyCode::LeftShift) && self.dash_buffer <= 0.0 && !self.dashed {
                self.dash_buffer = 0.15;
            }

            // wall jumping

            if self.hitting_wall_right && !self.on_ground && self.wall_jump_enabled {
                self.on_wall = true;
                self.on_wall_buffer = 0.05;
                self.x_direction = XDirection::Left;
            }
            if self.hitting_wall_left && !self.on_ground && self.wall_jump_enabled {
                self.on_wall = true;
                self.on_wall_buffer = 0.05;
                self.x_direction = XDirection::Right;
            }

            if self.on_wall_buffer > 0.0 {
                self.wall_slide_timer += dt;
            } else {
                self.wall_slide_timer = 0.0;
            }

            if self.wall_jump_enabled && self.on_wall_buffer > 0.0 {
                if is_key_pressed(KeyCode::Space) {
                    self.vel_y = JUMP_FORCE;
                    self.is_jumping = true;
                    self.on_wall = false;
                    self.on_wall_buffer = 0.0;
                    self.jump_buffer_time = 0.0;
                    self.wall_jump_buffer = 0.15;
                    /*if self.hitting_wall_right {
                        self.x_direction = XDirection::Left;
                    }
                    if self.hitting_wall_left {
                        self.x_direction = XDirection::Right;
                    }*/
                    self.x_direction = match self.x_direction {
                        XDirection::Right => {
                            XDirection::Left
                        }
                        XDirection::Left => {
                            XDirection::Right
                        }
                    }
                }
                if is_key_released(KeyCode::Space) && self.vel_y < 0.0 {
                    self.vel_y *= JUMP_CUT;
                }
            }

            let mut n = 0;
            if self.is_attacking {
                n += 1;
                println!("{:?}, {:?}, {}", self.x_direction, self.attack_direction, n);
            }

            // attacking
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

        if self.knockback_vel_x != 0.0 {
            self.vel_x = self.knockback_vel_x;
        }
        if self.knockback_vel_y != 0.0 {
            self.vel_y = self.knockback_vel_y;
        }

        let gravity = if self.vel_y < 0.0 {
            GRAVITY_UP
        } else if self.vel_y >= 0.0 && self.on_wall_buffer <= 0.0 {
            GRAVITY_DOWN
        } else {
            GRAVITY_DOWN / 3.0 * power(self.wall_slide_timer, 3).min(1.0)
        };

        if self.dash_buffer <= 0.0 {
            self.vel_y += gravity * dt;
        } else if self.on_wall_buffer > 0.0 {
            self.vel_y += gravity * dt;
        }

        self.on_ground = false;
        if self.dash_buffer <= 0.0 {
            self.y += self.vel_y * dt;
        }
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
    }

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
        if self.on_wall {
            draw_rectangle(
                self.x,
                self.y - self.pheight as f32 * TILE_SIZE,
                self.pwidth as f32 * TILE_SIZE,
                self.pheight as f32 * TILE_SIZE,
                Color::from_rgba(255, 255, 255, 80),
            );
        }
        if self.is_double_jumping {
            draw_rectangle(
                self.x,
                self.y - self.pheight as f32 * TILE_SIZE,
                self.pwidth as f32 * TILE_SIZE,
                self.pheight as f32 * TILE_SIZE,
                Color::from_rgba(0, 255, 0, 80),
            );
        }
        
        if self.is_attacking {
            draw_rectangle(
                self.attack_x,
                self.attack_y,
                self.attack_width,
                self.attack_height,
                Color::from_rgba(0, 255, 0, 80),
            );
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

    fn resolve_horizontal_collisions(&mut self) {
        let top_y = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
        let bottom_y = ((self.y - 1.0) / TILE_SIZE).floor() as i32;
        let middle_y = ((self.y - self.pheight / 2.0 * TILE_SIZE) / TILE_SIZE).floor() as i32;

        if self.vel_x > 0.0 {
            // moving right
            
            let next_right = ((self.x + self.pwidth * TILE_SIZE) / TILE_SIZE).floor() as i32;
            if self.current_room.is_solid(next_right, top_y) || self.current_room.is_solid(next_right, bottom_y) || self.current_room.is_solid(next_right, middle_y) {
                self.x = (next_right as f32) * TILE_SIZE - self.pwidth * TILE_SIZE;
                self.vel_x = 0.0;
                self.hitting_wall_right = true;
                if self.damage_blocked_buffer <= 0.0 {
                    if self.current_room.is_spikes(next_right, top_y) || self.current_room.is_spikes(next_right, bottom_y) || self.current_room.is_spikes(next_right, middle_y) {
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

    fn dash(&mut self) {
        self.dashed = true;
        let dash_speed = 800.0;
        let dash_direction = match self.x_direction {
            XDirection::Left => {
                -1.0
            }
            XDirection::Right => {
                1.0
            }
        };
        if self.knockback_vel_x == 0.0 {
            self.vel_x = dash_speed * dash_direction;
        }
    }

    fn on_wall_right(&self) -> bool {
        let top_y = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
        let bottom_y = ((self.y - 1.0) / TILE_SIZE).floor() as i32;
        let middle_y = ((self.y - self.pheight / 2.0 * TILE_SIZE) / TILE_SIZE).floor() as i32;

        if self.on_ground {
            return false;
        }

        let right_x = ((self.x + self.pwidth * TILE_SIZE) / TILE_SIZE).floor() as i32;

        return self.current_room.is_solid(right_x, top_y) || self.current_room.is_solid(right_x, bottom_y) || self.current_room.is_solid(right_x, middle_y);
    }

    fn on_wall_left(&self) -> bool {
        let top_y = ((self.y - self.pheight * TILE_SIZE) / TILE_SIZE).floor() as i32;
        let bottom_y = ((self.y - 1.0) / TILE_SIZE).floor() as i32;
        let middle_y = ((self.y - self.pheight / 2.0 * TILE_SIZE) / TILE_SIZE).floor() as i32;

        if self.on_ground {
            return false;
        }

        let left_x = ((self.x - self.pwidth * TILE_SIZE) / TILE_SIZE).floor() as i32;
        
        return self.current_room.is_solid(left_x, top_y) || self.current_room.is_solid(left_x, bottom_y) || self.current_room.is_solid(left_x, middle_y);
    }
}

fn power(base: f32, exponent: i32) -> f32 {
    let mut x = 1.0;
    for i in 0..exponent {
        x *= base;
    }
    x
}