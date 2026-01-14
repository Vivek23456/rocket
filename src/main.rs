mod joystick;
mod player;
mod game;

use macroquad::prelude::*;
use game::GameState;

fn window_conf() -> Conf {
    Conf {
        window_title: "🕹️ Dual Joystick Game".to_owned(),
        window_width: 1280,
        window_height: 720,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = GameState::new();

    loop {
        let dt = get_frame_time();

        // Update game state
        game.update(dt);

        // Draw everything
        game.draw();

        next_frame().await
    }
}
