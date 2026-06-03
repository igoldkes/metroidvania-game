use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub enum StoryPhase {
    NotPlaying,
    Playing,
}

impl StoryPhase {
    pub fn new_game() -> Self {
        StoryPhase::NotPlaying
    }

    pub fn update(&mut self, dt: f32) {
        match self {
            StoryPhase::NotPlaying => {}
            StoryPhase::Playing => {}
        }
    }
}