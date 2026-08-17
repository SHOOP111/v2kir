use macroquad::{prelude::*, rand::gen_range};
use crate::{constants::*, math::*, model::*};

pub fn reset_run(game: &mut GameData) {
    let seed = (get_time() as u64).wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(0xA37E_F411);
    game.state = RunState::Playing;
    game.player = Player::default();
    game.enemies.clear(); game.projectiles.clear(); game.pickups.clear();
    game.particles.clear(); game.texts.clear();
    game.world = WorldState::new(seed);
    game.score = 0; game.elapsed = 0.0; game.shake = 0.0; game.flash = 0.0;
    game.near_miss = 0.0; game.banner = "THE SHATTERED CROWN AWAKENS".into(); game.banner_timer = 3.2;
}

pub fn spawn_position(world: &WorldState, player: Vec2) -> Vec2 {
    let edge = world.arena_radius * gen_range(0.86_f32, 1.0_f32);
    let angle = gen_range(0.0_f32, PI2);
    let mut p = player + from_angle(angle) * edge;
    p.x = p.x.clamp(-WORLD_W * 0.5 + 120.0, WORLD_W * 0.5 - 120.0);
    p.y = p.y.clamp(-WORLD_H * 0.5 + 120.0, WORLD_H * 0.5 - 120.0);
    p
}

pub fn spawn_enemy(game: &mut GameData, kind: EnemyKind, elite: bool) {
    if game.enemies.len() >= MAX_ENEMIES { return; }
    let p = spawn_position(&game.world, game.player.pos);
    let scale = difficulty_wave(game.world.wave);
    let (hp, speed, dmg, radius) = match kind {
        EnemyKind::Grunt => (42.0 * scale, 105.0, 12.0 * scale.sqrt(), 17.0),
        EnemyKind::Shooter => (58.0 * scale, 84.0, 10.0 * scale.sqrt(), 18.0),
        EnemyKind::Dasher => (52.0 * scale, 132.0, 18.0 * scale.sqrt(), 16.0),
        EnemyKind::Brute => (280.0 * scale, 48.0, 29.0 * scale.sqrt(), 31.0),
        EnemyKind::Warden => (760.0 * scale, 62.0, 22.0 * scale.sqrt(), 44.0),
        EnemyKind::Harvester => (125.0 * scale, 158.0, 17.0 * scale.sqrt(), 22.0),
    };
    let elite_scale = if elite { 2.15 } else { 1.0 };
    game.enemies.push(Enemy { id: game.world.next_enemy_id, kind, pos: p, velocity: Vec2::ZERO,
        radius: radius * if elite { 1.16 } else { 1.0 }, hp: hp * elite_scale, max_hp: hp * elite_scale,
        speed: speed * if elite { 1.12 } else { 1.0 }, damage: dmg * elite_scale,
        attack_timer: gen_range(0.2, 1.2), phase: gen_range(-PI2, PI2), flash: 0.0, stun: 0.0,
        elite, orbit_sign: if gen_range(0, 2) == 0 { -1.0 } else { 1.0 } });
    game.world.next_enemy_id += 1;
}

pub fn update_world(game: &mut GameData, dt: f32) {
    game.elapsed += dt;
    game.world.time += dt;
    game.world.anomaly = ((game.world.time * 0.07).sin() * 0.5 + 0.5) * 0.7;
    game.world.arena_radius = ARENA_MIN_RADIUS + (ARENA_MAX_RADIUS - ARENA_MIN_RADIUS) *
        (0.5 + 0.5 * (game.world.wave as f32 / 30.0).min(1.0));
    game.shake = (game.shake - dt * 3.8).max(0.0);
    game.flash = (game.flash - dt * 2.5).max(0.0);
    game.near_miss = (game.near_miss - dt * 2.0).max(0.0);
    game.banner_timer = (game.banner_timer - dt).max(0.0);

    game.world.spawn_budget += dt * (0.62 + game.world.wave as f32 * 0.027);
    let alive_pressure = game.enemies.len() as f32 / 65.0;
    if game.world.spawn_budget > 1.0 && alive_pressure < 1.0 {
        let budget = game.world.spawn_budget.floor().min(4.0) as i32;
        for _ in 0..budget {
            let r = gen_range(0.0_f32, 100.0_f32);
            let kind = if game.world.wave >= 18 && r < 5.0 { EnemyKind::Warden }
                else if game.world.wave >= 9 && r < 13.0 { EnemyKind::Brute }
                else if game.world.wave >= 6 && r < 27.0 { EnemyKind::Harvester }
                else if r < 52.0 { EnemyKind::Grunt }
                else if r < 78.0 { EnemyKind::Shooter }
                else { EnemyKind::Dasher };
            let elite = game.world.wave >= 5 && gen_range(0.0_f32, 100.0_f32) < (3.0 + game.world.wave as f32 * 0.22).min(14.0);
            spawn_enemy(game, kind, elite);
        }
        game.world.spawn_budget -= budget as f32;
    }

    if game.world.kills_for_wave >= game.world.target_kills {
        game.world.wave += 1;
        game.world.kills_for_wave = 0;
        game.world.target_kills = 10 + game.world.wave * 7;
        game.player.energy = game.player.max_energy;
        game.player.hp = (game.player.hp + game.player.max_hp * 0.12).min(game.player.max_hp);
        if game.world.wave % 5 == 0 { game.banner = format!("ELITE TIDE // WAVE {}", game.world.wave); }
        else { game.banner = format!("WAVE {} // THREAT RATING +{}%", game.world.wave, game.world.wave * 7); }
        game.banner_timer = 2.7;
        game.shake = 1.0;
    }
}

pub fn resolve_world_bounds(game: &mut GameData) {
    let p = &mut game.player.pos;
    let lim = game.world.arena_radius - 34.0;
    let d = p.length();
    if d > lim {
        let n = safe_normalize(*p);
        *p = n * lim;
        if game.player.velocity.dot(n) > 0.0 { game.player.velocity -= n * game.player.velocity.dot(n); }
        game.flash = game.flash.max(0.08);
    }
}
