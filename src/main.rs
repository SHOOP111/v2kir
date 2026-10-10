use aetherfall::{constants::FIXED_DT, game::GameData, render, ui};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Aetherfall: The Shattered Crown".into(),
        window_width: 1440,
        window_height: 900,
        window_resizable: true,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = GameData::new();
    loop {
        let mut frame_dt = get_frame_time().min(0.05);
        game.accumulator += frame_dt;
        while game.accumulator >= FIXED_DT {
            game.update(FIXED_DT);
            game.accumulator -= FIXED_DT;
        }
        render::draw(&game);
        render::draw_overlay(&game);
        ui::draw_hud(&game);
        frame_dt = frame_dt.max(0.0);
        let _ = frame_dt;
        next_frame().await;
    }
}
