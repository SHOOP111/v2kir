use macroquad::{prelude::*, rand::gen_range};
use crate::{constants::*, math::*, model::*};

pub fn enemy_color(kind: EnemyKind) -> Color { match kind {
    EnemyKind::Grunt => Color::new(0.86,0.28,0.25,1.0), EnemyKind::Shooter => Color::new(0.75,0.38,0.94,1.0),
    EnemyKind::Dasher => Color::new(1.0,0.52,0.16,1.0), EnemyKind::Brute => Color::new(0.82,0.18,0.34,1.0),
    EnemyKind::Warden => Color::new(0.18,0.90,0.92,1.0), EnemyKind::Harvester => Color::new(0.88,0.78,0.22,1.0),
} }

pub fn update_enemies(game: &mut GameData, dt: f32) {
    let player_pos = game.player.pos;
    let mut damage_events: Vec<f32> = Vec::new();
    let mut enemy_shots: Vec<(Vec2, Vec2, f32, Color)> = Vec::new();
    let blood_moon = game.world.rift_event == Some(RiftEvent::BloodMoon);
    let time_snare = game.world.rift_event == Some(RiftEvent::TimeSnare);
    for e in game.enemies.iter_mut() {
        e.attack_timer -= dt; e.flash = (e.flash - dt * 5.0).max(0.0); e.stun = (e.stun - dt).max(0.0);
        if e.stun > 0.0 { e.velocity = damp(e.velocity, Vec2::ZERO, 16.0, dt); e.pos += e.velocity * dt; continue; }
        let to_player = player_pos - e.pos;
        let dist = to_player.length().max(0.001);
        let dir = to_player / dist;
        let tangent = Vec2::new(-dir.y, dir.x) * e.orbit_sign;
        let speed = e.speed * if time_snare { 0.58 } else { 1.0 };
        let desired = match e.kind {
            EnemyKind::Grunt => dir * speed,
            EnemyKind::Shooter => if dist > 560.0 { dir*speed } else if dist < 330.0 { -dir*speed } else { tangent*speed*0.85 },
            EnemyKind::Dasher => if e.attack_timer <= 0.0 { e.attack_timer = 1.5; dir * speed * 7.0 } else { damp(e.velocity, dir*speed, 5.0, dt) },
            EnemyKind::Brute => dir * speed,
            EnemyKind::Warden => if dist > 850.0 { dir*speed } else { tangent*speed*0.45 },
            EnemyKind::Harvester => if dist < 680.0 { -dir*speed } else { tangent*speed + dir*speed*0.35 },
        };
        e.velocity = damp(e.velocity, desired, 5.5, dt);
        e.pos += clamp_len(e.velocity, speed * 2.4) * dt;
        e.phase += dt * (1.2 + e.speed*0.002);
        if dist < e.radius + game.player.radius + 5.0 && e.attack_timer <= 0.0 {
            damage_events.push(e.damage * if blood_moon { 1.35 } else { 1.0 }); e.attack_timer = match e.kind { EnemyKind::Dasher=>1.8, EnemyKind::Brute=>1.25, _=>0.9 };
        }
        if matches!(e.kind, EnemyKind::Shooter|EnemyKind::Warden) && e.attack_timer <= 0.0 && dist < 1100.0 {
            e.attack_timer = if e.kind == EnemyKind::Warden { 1.15 } else { 1.75 };
            let spread = if e.elite { 0.11 } else { 0.05 };
            let shot_dir = rotate(dir, gen_range(-spread, spread));
            enemy_shots.push((e.pos, shot_dir, e.damage*0.65*if blood_moon { 1.35 } else { 1.0 }, enemy_color(e.kind)));
        }
    }
    for dmg in damage_events { apply_player_damage(game, dmg * if game.world.anomaly > 0.8 {1.12} else {1.0}); }
    for (pos,dir,dmg,color) in enemy_shots {
        game.projectiles.push(Projectile { pos, prev_pos: pos, velocity: dir * (550.0 + game.world.wave as f32*4.0), radius: 6.0,
            damage:dmg, lifetime:3.0, pierce:0, bounces:0, color, explosive:false, homing:0.0 });
        burst(game,pos,color,3,35.0,0.16);
    }
}

fn apply_player_damage(game:&mut GameData, dmg:f32) {
    if game.player.invulnerable > 0.0 { game.near_miss = game.near_miss.max(0.25); return; }
    game.player.hp -= dmg; game.player.invulnerable = 0.32; game.flash = 0.32; game.shake = 0.7;
    game.player.combo = 0; game.player.combo_timer = 0.0;
    floating(game, game.player.pos + vec2(0.0,-28.0), format!("-{}", dmg as i32), RED, 17.0);
    burst(game, game.player.pos, RED, 12, 180.0, 0.35);
    if game.player.hp <= 0.0 { game.player.hp = 0.0; game.state = RunState::Dead; game.banner = "THE CROWN CLAIMS ANOTHER".into(); game.banner_timer = 5.0; }
}

pub fn burst(game:&mut GameData,pos:Vec2,color:Color,count:usize,speed:f32,life:f32){
    let take=(MAX_PARTICLES-game.particles.len()).min(count); for _ in 0..take { let a=gen_range(0.0,PI2); let v=from_angle(a)*gen_range(speed*0.25,speed); game.particles.push(Particle{pos,velocity:v,life:gen_range(life*0.45,life),max_life:life,size:gen_range(2.0,6.0),color,gravity:gen_range(-20.0,35.0),drag:3.5}); }
}
pub fn floating(game:&mut GameData,pos:Vec2,text:String,color:Color,size:f32){ game.texts.push(FloatingText{pos,text,color,life:0.8,max_life:0.8,size}); }
