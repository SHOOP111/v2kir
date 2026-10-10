use crate::{constants::*, model::*};
use macroquad::prelude::*;

pub fn draw_hud(game: &GameData) {
    if game.state != RunState::Playing {
        return;
    }
    let w = screen_width();
    let h = screen_height();
    draw_panel(18.0, 18.0, 295.0, 86.0, Color::new(0.02, 0.035, 0.06, 0.82));
    draw_text(
        &format!("LVL {}   WAVE {}", game.player.level, game.world.wave),
        30.0,
        43.0,
        20.0,
        WHITE,
    );
    draw_bar(30.0, 55.0, 235.0, 10.0, game.player.hp / game.player.max_hp, RED);
    draw_text(
        &format!("{:.0}/{:.0}", game.player.hp, game.player.max_hp),
        272.0,
        64.0,
        11.0,
        WHITE,
    );
    draw_bar(30.0, 73.0, 235.0, 8.0, game.player.energy / game.player.max_energy, SKYBLUE);
    draw_text(&format!("E {:.0}", game.player.energy), 272.0, 81.0, 11.0, WHITE);
    draw_bar(30.0, 89.0, 235.0, 7.0, game.player.fury / 100.0, GOLD);
    draw_text("FURY", 272.0, 96.0, 10.0, GOLD);

    let xp_req = xp_to_next(game.player.level);
    let xp_ratio = game.player.xp as f32 / xp_req as f32;
    draw_bar(18.0, h - 22.0, w - 36.0, 5.0, xp_ratio, VIOLET);
    let right_x = w - 250.0;
    draw_panel(right_x, 18.0, 232.0, 76.0, Color::new(0.02, 0.035, 0.06, 0.78));
    draw_text(&format!("SCORE {:09}", game.score), right_x + 14.0, 42.0, 18.0, GOLD);
    draw_text(&format!("KILLS {:05}", game.player.kills), right_x + 14.0, 66.0, 16.0, WHITE);
    draw_text(&format!("DMG {:.0}", game.player.damage_done), right_x + 122.0, 66.0, 14.0, SKYBLUE);

    let event_x = (w - 326.0) * 0.5;
    draw_panel(event_x, 18.0, 326.0, 68.0, Color::new(0.02, 0.035, 0.06, 0.82));
    let (event_title, event_color, event_status) = if let Some(event) = game.world.rift_event {
        (
            event.title(),
            match event {
                RiftEvent::BloodMoon => RED,
                RiftEvent::Overcharge => SKYBLUE,
                RiftEvent::TimeSnare => VIOLET,
                RiftEvent::FortuneFlux => GOLD,
            },
            format!("{}  //  {:.1}s", event.description(), game.world.rift_timer),
        )
    } else {
        (
            "DORMANT",
            Color::new(0.42, 0.50, 0.62, 1.0),
            format!("NEXT EVENT IN {:.0}s", game.world.rift_cooldown),
        )
    };
    let title = format!("RIFT // {}", event_title);
    draw_text(&title, event_x + 14.0, 43.0, 17.0, event_color);
    draw_text(&event_status, event_x + 14.0, 67.0, 13.0, WHITE);

    let weapon = match game.player.weapon {
        WeaponKind::Dawnblade => "DAWNBLADE",
        WeaponKind::Repeater => "REPEATER",
        WeaponKind::ArcCannon => "ARC CANNON",
        WeaponKind::VoidLance => "VOID LANCE",
    };
    let wm = measure_text(weapon, None, 17, 1.0);
    draw_text(weapon, w * 0.5 - wm.width * 0.5, h - 42.0, 17.0, WHITE);
    if game.player.overdrive > 0.0 {
        let t = measure_text("OVERDRIVE", None, 18, 1.0);
        draw_text("OVERDRIVE", w * 0.5 - t.width * 0.5, 122.0, 18.0, GOLD);
    }
}

fn draw_panel(x: f32, y: f32, w: f32, h: f32, c: Color) {
    draw_rectangle(x + 4.0, y + 4.0, w, h, Color::new(0.0, 0.0, 0.0, 0.25));
    draw_rectangle(x, y, w, h, c);
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(0.18, 0.30, 0.46, 0.65));
}
fn draw_bar(x: f32, y: f32, w: f32, h: f32, p: f32, c: Color) {
    draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.65));
    draw_rectangle(x, y, w * p.clamp(0.0, 1.0), h, c);
}
