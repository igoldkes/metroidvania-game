use macroquad::prelude::*;

use std::fs;

pub fn load_jackie_paper_texture(path: &str) -> Texture2D {
    match fs::read(path) {
        Ok(bytes) => Texture2D::from_file_with_format(&bytes, None),
        Err(_) => build_fallback_jackie_paper_texture(),
    }
}

pub fn load_background_texture(path: &str) -> Texture2D {
    match fs::read(path) {
        Ok(bytes) => Texture2D::from_file_with_format(&bytes, None),
        Err(_) => build_fallback_background_texture(),
    }
}

pub fn load_player_life_texture(path: &str) -> Texture2D {
    match fs::read(path) {
        Ok(bytes) => Texture2D::from_file_with_format(&bytes, None),
        Err(_) => build_fallback_player_life_texture(),
    }
}

fn build_fallback_jackie_paper_texture() -> Texture2D {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut img = Image::gen_image_color(W as u16, H as u16, Color::from_rgba(0, 0, 0, 0));
    let cx = W as f32 * 0.5;
    let cy = H as f32 * 0.5;
    let fill = Color::from_rgba(255, 0, 0, 255);
    let edge = Color::from_rgba(255, 130, 130, 255);
    for y in 0..H {
        for x in 0..W {
            let dx = (x as f32 - cx).abs();
            let dy = (y as f32 - cy).abs();
            if dx + dy <= 18.0 {
                let c = if dx + dy >= 15.0 { edge } else { fill };
                img.set_pixel(x, y, c);
            }
        }
    }
    Texture2D::from_image(&img)
}

fn build_fallback_background_texture() -> Texture2D {
    const W: u32 = 420;
    const H: u32 = 300;
    let mut img = Image::gen_image_color(W as u16, H as u16, Color::from_rgba(0, 0, 0, 0));
    let base = Color::from_rgba(212, 188, 142, 255);
    let dark = Color::from_rgba(178, 150, 110, 255);
    for y in 0..H {
        for x in 0..W {
            let edge = x.min(W - 1 - x).min(y.min(H - 1 - y)) as f32;
            let t = (edge / 28.0).clamp(0.0, 1.0);
            let r = dark.r + (base.r - dark.r) * t;
            let g = dark.g + (base.g - dark.g) * t;
            let b = dark.b + (base.b - dark.b) * t;
            img.set_pixel(x, y, Color::new(r, g, b, 1.0));
        }
    }
    Texture2D::from_image(&img)
}

fn build_fallback_player_life_texture() -> Texture2D {
    const W: u32 = 64;
    const H: u32 = 64;
    let mut img = Image::gen_image_color(W as u16, H as u16, Color::from_rgba(0, 0, 0, 0));
    let cx = W as f32 * 0.5;
    let cy = H as f32 * 0.5;
    let fill = Color::from_rgba(255, 0, 0, 255);
    let edge = Color::from_rgba(255, 130, 130, 255);
    for y in 0..H {
        for x in 0..W {
            let dx = (x as f32 - cx).abs();
            let dy = (y as f32 - cy).abs();
            if dx + dy <= 18.0 {
                let c = if dx + dy >= 15.0 { edge } else { fill };
                img.set_pixel(x, y, c);
            }
        }
    }
    Texture2D::from_image(&img)
}