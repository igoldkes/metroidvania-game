use macroquad::prelude::*;

mod game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Metroidvania Game".to_string(),
        window_width: 1280,
        window_height: 720,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = game::GameState::new().await;

    loop {
        let dt = get_frame_time();

        state.update(dt);

        state.draw();

        next_frame().await
    }
}