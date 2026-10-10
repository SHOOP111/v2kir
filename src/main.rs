use macroquad::prelude::*;

mod constants {
    pub const WORLD_W: f32 = 5_200.0;
    pub const WORLD_H: f32 = 5_200.0;
    pub const FIXED_DT: f32 = 1.0 / 120.0;
    pub const PLAYER_RADIUS: f32 = 18.0;
    pub const PLAYER_SPEED: f32 = 330.0;
    pub const DASH_SPEED: f32 = 980.0;
    pub const BULLET_SPEED: f32 = 940.0;
    pub const MAX_PARTICLES: usize = 3_200;
    pub const MAX_ENEMIES: usize = 900;
    pub const MAX_ACTIVE_ENEMIES: usize = 65;
    pub const MAX_PROJECTILES: usize = 2_000;
    pub const MAX_PICKUPS: usize = 320;
    pub const SAVE_VERSION: u32 = 2;
    pub const ARENA_CELL: f32 = 260.0;
    pub const ARENA_MIN_RADIUS: f32 = 780.0;
    pub const ARENA_MAX_RADIUS: f32 = 2_100.0;
    pub const PI2: f32 = std::f32::consts::PI * 2.0;

    pub fn difficulty_wave(wave: u32) -> f32 {
        1.0 + wave.saturating_sub(1) as f32 * 0.075
            + (wave / 8) as f32 .sqrt() * 0.18
    }

    pub fn xp_to_next(level: u32) -> u32 {
        let level = level.max(1) as f32;
        (85.0 + level.powf(1.55) * 42.0)
            .min((u32::MAX - 1) as f32) as u32
    }
}

mod math {
    use macroquad::prelude::*;

    pub fn clamp_len(v: Vec2, max: f32) -> Vec2 {
        if !v.x.is_finite() || !v.y.is_finite() || !max.is_finite() {
            return Vec2::ZERO;
        }
        let max = max.max(0.0);
        let len2 = v.length_squared();
        if len2 > max * max && len2 > 0.00001 {
            v / len2.sqrt() * max
        } else {
            v
        }
    }

    pub fn safe_normalize(v: Vec2) -> Vec2 {
        if !v.x.is_finite() || !v.y.is_finite() {
            return Vec2::ZERO;
        }
        let len2 = v.length_squared();
        if len2 > 0.00001 {
            v / len2.sqrt()
        } else {
            Vec2::ZERO
        }
    }

    pub fn damp(current: Vec2, target: Vec2, rate: f32, dt: f32) -> Vec2 {
        if !current.x.is_finite()
            || !current.y.is_finite()
            || !target.x.is_finite()
            || !target.y.is_finite()
        {
            return Vec2::ZERO;
        }
        let t = 1.0 - (-rate.max(0.0) * dt.max(0.0)).exp();
        current.lerp(target, t.clamp(0.0, 1.0))
    }

    pub fn rotate(v: Vec2, angle: f32) -> Vec2 {
        let (s, c) = angle.sin_cos();
        Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
    }

    pub fn from_angle(angle: f32) -> Vec2 {
        Vec2::new(angle.cos(), angle.sin())
    }

    pub fn circle_hit(a: Vec2, ar: f32, b: Vec2, br: f32) -> bool {
        let radius = ar.max(0.0) + br.max(0.0);
        a.distance_squared(b) <= radius * radius
    }

    pub fn circle_segment_hit(center: Vec2, radius: f32, start: Vec2, end: Vec2) -> bool {
        let d = end - start;
        let len2 = d.length_squared();
        let t = if len2 > 0.00001 {
            ((center - start).dot(d) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let closest = start + d * t;
        center.distance_squared(closest) <= radius.max(0.0).powi(2)
    }

    pub fn hash2(mut x: i32, mut y: i32, seed: u64) -> u32 {
        x = x.wrapping_mul(0x27d4_eb2d);
        y = y.wrapping_mul(0x1656_67b1);
        let mut h = seed as u32 ^ x as u32 ^ y as u32;
        h ^= h >> 16;
        h = h.wrapping_mul(0x85eb_ca6b);
        h ^= h >> 13;
        h = h.wrapping_mul(0xc2b2_ae35);
        h ^ (h >> 16)
    }

    pub fn hash01(x: i32, y: i32, seed: u64) -> f32 {
        ((hash2(x, y, seed) >> 8) as f32) * (1.0 / 16_777_216.0)
    }

    pub fn wrap_angle(angle: f32) -> f32 {
        if !angle.is_finite() {
            return 0.0;
        }
        (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::PI * 2.0)
            - std::f32::consts::PI
    }

    pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
        a + wrap_angle(b - a) * t.clamp(0.0, 1.0)
    }
}

mod model {
    use macroquad::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::constants::SAVE_VERSION;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum RunState {
        Title,
        Playing,
        Paused,
        Dead,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum EnemyKind {
        Grunt,
        Shooter,
        Dasher,
        Brute,
        Warden,
        Harvester,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum WeaponKind {
        Dawnblade,
        Repeater,
        ArcCannon,
        VoidLance,
    }

    impl WeaponKind {
        pub const ALL: [Self; 4] = [
            Self::Repeater,
            Self::Dawnblade,
            Self::ArcCannon,
            Self::VoidLance,
        ];

        pub fn key(self) -> &'static str {
            match self {
                Self::Dawnblade => "dawnblade",
                Self::Repeater => "repeater",
                Self::ArcCannon => "arc_cannon",
                Self::VoidLance => "void_lance",
            }
        }

        pub fn label(self) -> &'static str {
            match self {
                Self::Dawnblade => "DAWNBLADE",
                Self::Repeater => "REPEATER",
                Self::ArcCannon => "ARC CANNON",
                Self::VoidLance => "VOID LANCE",
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ProjectileOwner {
        Player,
        Enemy,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum PickupKind {
        Heal,
        Energy,
        Xp,
        Fury,
    }

    #[derive(Clone, Debug)]
    pub struct Player {
        pub pos: Vec2,
        pub velocity: Vec2,
        pub radius: f32,
        pub hp: f32,
        pub max_hp: f32,
        pub energy: f32,
        pub max_energy: f32,
        pub fury: f32,
        pub xp: u32,
        pub level: u32,
        pub kills: u64,
        pub damage_done: f64,
        pub weapon: WeaponKind,
        pub fire_timer: f32,
        pub dash_timer: f32,
        pub dash_time: f32,
        pub invulnerable: f32,
        pub nova_timer: f32,
        pub overdrive: f32,
        pub combo: u32,
        pub combo_timer: f32,
    }

    impl Default for Player {
        fn default() -> Self {
            Self {
                pos: Vec2::ZERO,
                velocity: Vec2::ZERO,
                radius: crate::constants::PLAYER_RADIUS,
                hp: 100.0,
                max_hp: 100.0,
                energy: 100.0,
                max_energy: 100.0,
                fury: 0.0,
                xp: 0,
                level: 1,
                kills: 0,
                damage_done: 0.0,
                weapon: WeaponKind::Repeater,
                fire_timer: 0.0,
                dash_timer: 0.0,
                dash_time: 0.0,
                invulnerable: 0.0,
                nova_timer: 0.0,
                overdrive: 0.0,
                combo: 0,
                combo_timer: 0.0,
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Enemy {
        pub id: u64,
        pub kind: EnemyKind,
        pub pos: Vec2,
        pub velocity: Vec2,
        pub radius: f32,
        pub hp: f32,
        pub max_hp: f32,
        pub speed: f32,
        pub damage: f32,
        pub attack_timer: f32,
        pub shoot_timer: f32,
        pub phase: f32,
        pub flash: f32,
        pub stun: f32,
        pub elite: bool,
        pub orbit_sign: f32,
    }

    #[derive(Clone, Debug)]
    pub struct Projectile {
        pub owner: ProjectileOwner,
        pub pos: Vec2,
        pub prev_pos: Vec2,
        pub velocity: Vec2,
        pub radius: f32,
        pub damage: f32,
        pub lifetime: f32,
        pub pierce: i32,
        pub color: Color,
        pub explosive: bool,
        pub homing: f32,
        pub hit_ids: Vec<u64>,
    }

    #[derive(Clone, Debug)]
    pub struct Pickup {
        pub pos: Vec2,
        pub velocity: Vec2,
        pub kind: PickupKind,
        pub radius: f32,
        pub ttl: f32,
        pub spin: f32,
    }

    #[derive(Clone, Debug)]
    pub struct Particle {
        pub pos: Vec2,
        pub velocity: Vec2,
        pub life: f32,
        pub max_life: f32,
        pub size: f32,
        pub color: Color,
        pub gravity: f32,
        pub drag: f32,
    }

    #[derive(Clone, Debug)]
    pub struct FloatingText {
        pub pos: Vec2,
        pub text: String,
        pub color: Color,
        pub life: f32,
        pub max_life: f32,
        pub size: f32,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(default)]
    pub struct SaveData {
        pub version: u32,
        pub best_wave: u32,
        pub best_score: u64,
        pub lifetime_kills: u64,
        pub lifetime_xp: u64,
        pub unlocked_weapons: Vec<String>,
        pub volume: f32,
    }

    impl Default for SaveData {
        fn default() -> Self {
            Self {
                version: SAVE_VERSION,
                best_wave: 0,
                best_score: 0,
                lifetime_kills: 0,
                lifetime_xp: 0,
                unlocked_weapons: vec!["repeater".into()],
                volume: 0.75,
            }
        }
    }

    pub struct WorldState {
        pub seed: u64,
        pub time: f32,
        pub wave: u32,
        pub wave_progress: f32,
        pub spawn_budget: f32,
        pub next_enemy_id: u64,
        pub kills_for_wave: u32,
        pub target_kills: u32,
        pub anomaly: f32,
        pub arena_radius: f32,
    }

    impl WorldState {
        pub fn new(seed: u64) -> Self {
            Self {
                seed,
                time: 0.0,
                wave: 1,
                wave_progress: 0.0,
                spawn_budget: 2.0,
                next_enemy_id: 1,
                kills_for_wave: 0,
                target_kills: 14,
                anomaly: 0.0,
                arena_radius: 1_100.0,
            }
        }
    }

    #[derive(Clone, Copy, Default)]
    pub struct InputFrame {
        pub movement: Vec2,
        pub aim: Vec2,
        pub fire: bool,
        pub start: bool,
        pub pause: bool,
        pub restart: bool,
        pub dash: bool,
        pub nova: bool,
        pub overdrive: bool,
        pub next_weapon: bool,
        pub selected_weapon: Option<WeaponKind>,
    }

    #[derive(Default)]
    pub struct InputEdges {
        pub start: bool,
        pub pause: bool,
        pub restart: bool,
        pub dash: bool,
        pub nova: bool,
        pub overdrive: bool,
        pub next_weapon: bool,
        pub selected_weapon: Option<WeaponKind>,
    }

    pub struct GameData {
        pub state: RunState,
        pub player: Player,
        pub enemies: Vec<Enemy>,
        pub projectiles: Vec<Projectile>,
        pub pickups: Vec<Pickup>,
        pub particles: Vec<Particle>,
        pub texts: Vec<FloatingText>,
        pub world: WorldState,
        pub save: SaveData,
        pub score: u64,
        pub accumulator: f32,
        pub elapsed: f32,
        pub shake: f32,
        pub flash: f32,
        pub near_miss: f32,
        pub banner: String,
        pub banner_timer: f32,
        pub save_timer: f32,
        pub run_finalized: bool,
        pub current_input: InputFrame,
        pub pending_input: InputEdges,
    }
}

mod save {
    use std::{fs, path::PathBuf};

    use crate::model::SaveData;
    use crate::constants::SAVE_VERSION;

    fn path() -> PathBuf {
        let base = std::env::var_os("AETHERFALL_SAVE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("save")
            });
        base.join("profile.json")
    }

    pub fn load() -> SaveData {
        let raw = match fs::read_to_string(path()) {
            Ok(raw) => raw,
            Err(_) => return SaveData::default(),
        };

        let mut save = serde_json::from_str::<SaveData>(&raw).unwrap_or_default();
        save.version = SAVE_VERSION;
        save.best_wave = save.best_wave.min(1_000_000);
        save.volume = if save.volume.is_finite() {
            save.volume.clamp(0.0, 1.0)
        } else {
            0.75
        };
        save.unlocked_weapons.retain(|key| {
            matches!(
                key.as_str(),
                "repeater" | "dawnblade" | "arc_cannon" | "void_lance"
            )
        });
        if !save.unlocked_weapons.iter().any(|key| key == "repeater") {
            save.unlocked_weapons.push("repeater".to_owned());
        }
        save
    }

    pub fn store(save: &SaveData) -> Result<(), String> {
        let target = path();
        let parent = target.parent().ok_or("Invalid save path")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let temporary = target.with_extension("json.tmp");
        let raw = serde_json::to_vec_pretty(save).map_err(|error| error.to_string())?;

        fs::write(&temporary, raw).map_err(|error| error.to_string())?;
        if let Err(first_error) = fs::rename(&temporary, &target) {
            if target.exists() {
                fs::remove_file(&target).map_err(|error| error.to_string())?;
                fs::rename(&temporary, &target).map_err(|error| error.to_string())?;
            } else {
                let _ = fs::remove_file(&temporary);
                return Err(first_error.to_string());
            }
        }
        Ok(())
    }
}

mod ai {
    use macroquad::{
        prelude::*,
        rand::gen_range,
    };

    use crate::{
        combat,
        constants::*,
        math::{clamp_len, damp, from_angle, safe_normalize},
        model::*,
    };

    pub fn enemy_color(kind: EnemyKind) -> Color {
        match kind {
            EnemyKind::Grunt => Color::new(0.86, 0.28, 0.25, 1.0),
            EnemyKind::Shooter => Color::new(0.75, 0.38, 0.94, 1.0),
            EnemyKind::Dasher => Color::new(1.0, 0.52, 0.16, 1.0),
            EnemyKind::Brute => Color::new(0.82, 0.18, 0.34, 1.0),
            EnemyKind::Warden => Color::new(0.18, 0.90, 0.92, 1.0),
            EnemyKind::Harvester => Color::new(0.88, 0.78, 0.22, 1.0),
        }
    }

    pub fn update_enemies(game: &mut GameData, dt: f32) {
        let player_pos = game.player.pos;
        let player_radius = game.player.radius;
        let arena_radius = game.world.arena_radius;
        let wave = game.world.wave;
        let mut damage_events = Vec::new();
        let mut enemy_shots: Vec<(Vec2, Vec2, f32, Color)> = Vec::new();

        for enemy in &mut game.enemies {
            enemy.attack_timer = (enemy.attack_timer - dt).max(0.0);
            enemy.shoot_timer = (enemy.shoot_timer - dt).max(0.0);
            enemy.flash = (enemy.flash - dt * 5.0).max(0.0);
            enemy.stun = (enemy.stun - dt).max(0.0);

            if enemy.stun > 0.0 {
                enemy.velocity = damp(enemy.velocity, Vec2::ZERO, 16.0, dt);
                enemy.pos += enemy.velocity * dt;
                continue;
            }

            let to_player = player_pos - enemy.pos;
            let distance = to_player.length().max(0.001);
            let direction = to_player / distance;
            let tangent = Vec2::new(-direction.y, direction.x) * enemy.orbit_sign;

            let desired = match enemy.kind {
                EnemyKind::Grunt | EnemyKind::Brute => direction * enemy.speed,
                EnemyKind::Shooter => {
                    if distance > 560.0 {
                        direction * enemy.speed
                    } else if distance < 330.0 {
                        -direction * enemy.speed
                    } else {
                        tangent * enemy.speed * 0.85
                    }
                }
                EnemyKind::Dasher => {
                    if enemy.attack_timer <= 0.0 && distance < 720.0 {
                        enemy.attack_timer = 1.5;
                        direction * enemy.speed * 2.4
                    } else {
                        damp(enemy.velocity, direction * enemy.speed, 5.0, dt)
                    }
                }
                EnemyKind::Warden => {
                    if distance > 850.0 {
                        direction * enemy.speed
                    } else {
                        tangent * enemy.speed * 0.45
                    }
                }
                EnemyKind::Harvester => {
                    if distance < 680.0 {
                        -direction * enemy.speed
                    } else {
                        tangent * enemy.speed + direction * enemy.speed * 0.35
                    }
                }
            };

            enemy.velocity = damp(enemy.velocity, desired, 5.5, dt);
            enemy.pos += clamp_len(enemy.velocity, enemy.speed * 2.4) * dt;
            enemy.phase += dt * (1.2 + enemy.speed * 0.002);

            let radial_distance = enemy.pos.length();
            let allowed_radius = (arena_radius - enemy.radius).max(40.0);
            if radial_distance > allowed_radius {
                let outward = safe_normalize(enemy.pos);
                enemy.pos = outward * allowed_radius;
                let outward_speed = enemy.velocity.dot(outward);
                if outward_speed > 0.0 {
                    enemy.velocity -= outward * outward_speed;
                }
            }

            let actual_distance = enemy.pos.distance(player_pos);
            if actual_distance < enemy.radius + player_radius + 5.0
                && enemy.attack_timer <= 0.0
            {
                damage_events.push(enemy.damage);
                enemy.attack_timer = match enemy.kind {
                    EnemyKind::Dasher => 1.8,
                    EnemyKind::Brute => 1.25,
                    _ => 0.9,
                };
            }

            if matches!(enemy.kind, EnemyKind::Shooter | EnemyKind::Warden)
                && enemy.shoot_timer <= 0.0
                && actual_distance < 1_100.0
                && game.projectiles.len() + enemy_shots.len() < MAX_PROJECTILES
            {
                enemy.shoot_timer = if enemy.kind == EnemyKind::Warden {
                    1.15
                } else {
                    1.75
                };
                let spread = if enemy.elite { 0.11 } else { 0.05 };
                if enemy.kind == EnemyKind::Warden {
                    for offset in [-0.18_f32, 0.0, 0.18] {
                        let shot_dir = crate::math::rotate(direction, offset + gen_range(-spread, spread));
                        enemy_shots.push((
                            enemy.pos,
                            shot_dir,
                            enemy.damage * 0.55,
                            enemy_color(enemy.kind),
                        ));
                    }
                } else {
                    let shot_dir = crate::math::rotate(direction, gen_range(-spread, spread));
                    enemy_shots.push((
                        enemy.pos,
                        shot_dir,
                        enemy.damage * 0.65,
                        enemy_color(enemy.kind),
                    ));
                }
            }
        }

        for damage in damage_events {
            apply_player_damage(game, damage * if game.world.anomaly > 0.8 { 1.12 } else { 1.0 });
        }

        for (position, direction, damage, color) in enemy_shots {
            if game.projectiles.len() >= MAX_PROJECTILES {
                break;
            }
            game.projectiles.push(Projectile {
                owner: ProjectileOwner::Enemy,
                pos: position,
                prev_pos: position,
                velocity: direction * (550.0 + wave as f32 * 4.0).min(1_300.0),
                radius: 6.0,
                damage,
                lifetime: 3.0,
                pierce: 0,
                color,
                explosive: false,
                homing: 0.0,
                hit_ids: Vec::new(),
            });
            burst(game, position, color, 3, 35.0, 0.16);
        }
    }

    pub fn apply_player_damage(game: &mut GameData, damage: f32) {
        if game.player.invulnerable > 0.0 || game.state == RunState::Dead {
            game.near_miss = game.near_miss.max(0.25);
            return;
        }

        let damage = damage.max(0.0);
        game.player.hp = (game.player.hp - damage).max(0.0);
        game.player.invulnerable = 0.32;
        game.flash = 0.32;
        game.shake = 0.7;
        game.player.combo = 0;
        game.player.combo_timer = 0.0;
        floating(
            game,
            game.player.pos + vec2(0.0, -28.0),
            format!("-{}", damage.round() as u32),
            RED,
            17.0,
        );
        burst(game, game.player.pos, RED, 12, 180.0, 0.35);

        if game.player.hp <= 0.0 {
            game.player.hp = 0.0;
            game.state = RunState::Dead;
            game.banner = "THE CROWN CLAIMS ANOTHER".to_owned();
            game.banner_timer = 5.0;
            crate::game::finish_run(game);
        }
    }

    pub fn burst(
        game: &mut GameData,
        position: Vec2,
        color: Color,
        count: usize,
        speed: f32,
        life: f32,
    ) {
        let count = (MAX_PARTICLES.saturating_sub(game.particles.len())).min(count);
        for _ in 0..count {
            let angle = gen_range(0.0_f32, PI2);
            let velocity = from_angle(angle) * gen_range(speed * 0.25, speed.max(speed * 0.25 + 0.01));
            game.particles.push(Particle {
                pos: position,
                velocity,
                life: gen_range(life * 0.45, life.max(0.01)),
                max_life: life.max(0.01),
                size: gen_range(2.0_f32, 6.0),
                color,
                gravity: gen_range(-20.0_f32, 35.0),
                drag: 3.5,
            });
        }
    }

    pub fn floating(
        game: &mut GameData,
        position: Vec2,
        text: String,
        color: Color,
        size: f32,
    ) {
        if game.texts.len() >= 500 {
            game.texts.remove(0);
        }
        game.texts.push(FloatingText {
            pos: position,
            text,
            color,
            life: 0.8,
            max_life: 0.8,
            size,
        });
    }
}

mod combat {
    use macroquad::{prelude::*, rand::gen_range};

    use crate::{
        ai::{self, burst, floating},
        constants::*,
        math::{circle_segment_hit, damp, rotate, safe_normalize},
        model::*,
    };

    fn weapon_stats(weapon: WeaponKind) -> (f32, f32, f32, i32, bool) {
        match weapon {
            WeaponKind::Dawnblade => (42.0, 0.12, 720.0, 1, false),
            WeaponKind::Repeater => (15.0, 0.085, BULLET_SPEED, 0, false),
            WeaponKind::ArcCannon => (58.0, 0.55, 690.0, 0, true),
            WeaponKind::VoidLance => (115.0, 0.9, 1_160.0, 5, false),
        }
    }

    pub fn fire_weapon(game: &mut GameData, aim: Vec2) {
        if game.player.fire_timer > 0.0
            || game.state != RunState::Playing
            || game.projectiles.len() >= MAX_PROJECTILES
            || game.player.energy < 2.5
        {
            return;
        }

        let aim = if aim.length_squared() > 0.00001 {
            safe_normalize(aim)
        } else {
            Vec2::X
        };
        let (damage, cooldown, speed, pierce, explosive) = weapon_stats(game.player.weapon);
        let overdrive = if game.player.overdrive > 0.0 { 0.74 } else { 1.0 };
        game.player.fire_timer = cooldown * overdrive;
        game.player.energy = (game.player.energy - 2.5).max(0.0);

        let shots = match game.player.weapon {
            WeaponKind::Dawnblade => 2,
            WeaponKind::VoidLance => 3,
            _ => 1,
        };
        for shot_index in 0..shots {
            if game.projectiles.len() >= MAX_PROJECTILES {
                break;
            }
            let spread = match game.player.weapon {
                WeaponKind::Dawnblade => 0.16,
                WeaponKind::VoidLance => 0.035,
                _ => 0.012,
            };
            let offset = (shot_index as f32 - (shots - 1) as f32 * 0.5) * spread;
            let direction = rotate(aim, offset);
            let position = game.player.pos + direction * 25.0;
            let critical = gen_range(0.0_f32, 100.0) < (7.0 + game.player.level as f32 * 0.7).min(45.0);
            let dealt = damage
                * if critical { 2.0 } else { 1.0 }
                * (1.0 + game.player.level.min(200) as f32 * 0.045);
            let color = match game.player.weapon {
                WeaponKind::Dawnblade => GOLD,
                WeaponKind::Repeater => WHITE,
                WeaponKind::ArcCannon => SKYBLUE,
                WeaponKind::VoidLance => VIOLET,
            };
            game.projectiles.push(Projectile {
                owner: ProjectileOwner::Player,
                pos: position,
                prev_pos: position,
                velocity: direction * speed,
                lifetime: 1.65,
                radius: if explosive { 9.0 } else { 5.0 },
                damage: dealt,
                pierce,
                color,
                explosive,
                homing: if game.player.weapon == WeaponKind::VoidLance { 0.8 } else { 0.0 },
                hit_ids: Vec::new(),
            });
            if critical {
                floating(
                    game,
                    position,
                    format!("CRIT {}", dealt.round() as u32),
                    GOLD,
                    15.0,
                );
                game.shake = game.shake.max(0.18);
            }
        }

        burst(
            game,
            game.player.pos + aim * 24.0,
            if explosive { SKYBLUE } else { WHITE },
            4,
            90.0,
            0.13,
        );
    }

    pub fn update_projectiles(game: &mut GameData, dt: f32) {
        let player_position = game.player.pos;
        {
            let (projectiles, enemies) = (&mut game.projectiles, &game.enemies);
            for projectile in projectiles.iter_mut() {
                projectile.prev_pos = projectile.pos;

                if projectile.owner == ProjectileOwner::Player
                    && projectile.homing > 0.0
                    && !enemies.is_empty()
                {
                    let target = enemies
                        .iter()
                        .filter(|enemy| !projectile.hit_ids.contains(&enemy.id))
                        .min_by(|a, b| {
                            projectile
                                .pos
                                .distance_squared(a.pos)
                                .total_cmp(&projectile.pos.distance_squared(b.pos))
                        });
                    if let Some(target) = target {
                        let direction = safe_normalize(target.pos - projectile.pos);
                        let speed = projectile.velocity.length();
                        projectile.velocity = damp(
                            projectile.velocity,
                            direction * speed,
                            projectile.homing * 8.0,
                            dt,
                        );
                    }
                }

                projectile.pos += projectile.velocity * dt;
                projectile.lifetime -= dt;
                if projectile.pos.x.abs() > WORLD_W || projectile.pos.y.abs() > WORLD_H {
                    projectile.lifetime = -1.0;
                }
            }
        }

        let mut player_hits = Vec::new();
        let mut enemy_hits = Vec::new();
        for (projectile_index, projectile) in game.projectiles.iter().enumerate() {
            if projectile.lifetime <= 0.0 {
                continue;
            }

            match projectile.owner {
                ProjectileOwner::Player => {
                    if let Some((enemy_index, _)) = game
                        .enemies
                        .iter()
                        .enumerate()
                        .filter(|(_, enemy)| !projectile.hit_ids.contains(&enemy.id))
                        .find(|(_, enemy)| {
                            circle_segment_hit(
                                enemy.pos,
                                enemy.radius + projectile.radius,
                                projectile.prev_pos,
                                projectile.pos,
                            )
                        })
                    {
                        player_hits.push((projectile_index, enemy_index));
                    }
                }
                ProjectileOwner::Enemy => {
                    if circle_segment_hit(
                        player_position,
                        game.player.radius + projectile.radius,
                        projectile.prev_pos,
                        projectile.pos,
                    ) {
                        enemy_hits.push((projectile_index, projectile.damage));
                    }
                }
            }
        }

        let mut removed = vec![false; game.projectiles.len()];
        for (projectile_index, damage) in enemy_hits {
            removed[projectile_index] = true;
            ai::apply_player_damage(game, damage);
        }

        for (projectile_index, enemy_index) in player_hits {
            if removed[projectile_index] || enemy_index >= game.enemies.len() {
                continue;
            }
            let (damage, explosive, remove_projectile, target_id, impact_position, impact_kind) = {
                let projectile = &mut game.projectiles[projectile_index];
                let target = &game.enemies[enemy_index];
                if projectile.hit_ids.contains(&target.id) {
                    continue;
                }
                projectile.hit_ids.push(target.id);
                let remove_projectile = projectile.explosive || projectile.pierce <= 0;
                if !remove_projectile {
                    projectile.pierce -= 1;
                }
                (
                    projectile.damage,
                    projectile.explosive,
                    remove_projectile,
                    target.id,
                    target.pos,
                    target.kind,
                )
            };
            removed[projectile_index] |= remove_projectile;

            if let Some(enemy) = game.enemies.get_mut(enemy_index) {
                enemy.hp -= damage;
                enemy.flash = 0.18;
                game.player.damage_done += damage as f64;
                game.player.fury = (game.player.fury + damage * 0.12).min(100.0);
            }

            floating(
                game,
                impact_position + vec2(0.0, -32.0),
                format!("{}", damage.round() as u32),
                if damage > 70.0 { GOLD } else { WHITE },
                14.0,
            );

            if explosive {
                burst(game, impact_position, SKYBLUE, 18, 230.0, 0.32);
                for other in &mut game.enemies {
                    if other.id == target_id {
                        continue;
                    }
                    let distance = other.pos.distance(impact_position);
                    if distance < 150.0 {
                        let splash = damage * (1.0 - distance / 150.0) * 0.35;
                        other.hp -= splash;
                        other.stun = other.stun.max(0.12);
                        game.player.damage_done += splash as f64;
                        game.player.fury = (game.player.fury + splash * 0.12).min(100.0);
                    }
                }
            } else {
                burst(
                    game,
                    impact_position,
                    ai::enemy_color(impact_kind),
                    5,
                    80.0,
                    0.18,
                );
            }
        }

        for index in (0..game.projectiles.len()).rev() {
            if removed[index] || game.projectiles[index].lifetime <= 0.0 {
                game.projectiles.swap_remove(index);
            }
        }

        let dead: Vec<usize> = game
            .enemies
            .iter()
            .enumerate()
            .filter_map(|(index, enemy)| (enemy.hp <= 0.0).then_some(index))
            .collect();
        for index in dead.into_iter().rev() {
            let enemy = game.enemies.swap_remove(index);
            kill_enemy(game, enemy);
        }
    }

    fn enemy_tint(kind: EnemyKind) -> Color {
        match kind {
            EnemyKind::Grunt => RED,
            EnemyKind::Shooter => VIOLET,
            EnemyKind::Dasher => ORANGE,
            EnemyKind::Brute => MAROON,
            EnemyKind::Warden => SKYBLUE,
            EnemyKind::Harvester => YELLOW,
        }
    }

    fn kill_enemy(game: &mut GameData, enemy: Enemy) {
        game.player.kills = game.player.kills.saturating_add(1);
        game.player.combo = game.player.combo.saturating_add(1);
        game.player.combo_timer = 2.2;
        game.world.kills_for_wave = game.world.kills_for_wave.saturating_add(1);
        game.save.lifetime_kills = game.save.lifetime_kills.saturating_add(1);

        let xp = (14.0 + enemy.max_hp * 0.17)
            * if enemy.elite { 2.8 } else { 1.0 }
            * (1.0 + game.world.wave as f32 * 0.016);
        grant_xp(game, xp.max(1.0) as u32);
        game.score = game
            .score
            .saturating_add(enemy.max_hp.max(0.0) as u64 * if enemy.elite { 7 } else { 1 });

        burst(
            game,
            enemy.pos,
            enemy_tint(enemy.kind),
            if enemy.elite { 38 } else { 18 },
            if enemy.elite { 340.0 } else { 210.0 },
            if enemy.elite { 0.55 } else { 0.34 },
        );

        if game.pickups.len() < MAX_PICKUPS
            && (enemy.elite
                || gen_range(0.0_f32, 100.0) < 8.0 + game.world.anomaly * 12.0)
        {
            let kind = if game.player.hp < 55.0 && gen_range(0, 2) == 0 {
                PickupKind::Heal
            } else if gen_range(0, 3) == 0 {
                PickupKind::Fury
            } else {
                PickupKind::Energy
            };
            game.pickups.push(Pickup {
                pos: enemy.pos,
                velocity: Vec2::ZERO,
                kind,
                radius: 12.0,
                ttl: 18.0,
                spin: 0.0,
            });
        }

        if game.pickups.len() < MAX_PICKUPS && gen_range(0.0_f32, 100.0) < 4.5 {
            game.pickups.push(Pickup {
                pos: enemy.pos + vec2(gen_range(-8.0_f32, 8.0), gen_range(-8.0_f32, 8.0)),
                velocity: Vec2::ZERO,
                kind: PickupKind::Xp,
                radius: 9.0,
                ttl: 25.0,
                spin: 0.0,
            });
        }
    }

    pub fn grant_xp(game: &mut GameData, xp: u32) {
        game.player.xp = game.player.xp.saturating_add(xp);
        game.save.lifetime_xp = game.save.lifetime_xp.saturating_add(xp as u64);
        loop {
            let requirement = xp_to_next(game.player.level);
            if game.player.xp < requirement || game.player.level >= 9_999 {
                break;
            }
            game.player.xp -= requirement;
            game.player.level = game.player.level.saturating_add(1);
            game.player.max_hp = (game.player.max_hp + 8.0).min(1_000.0);
            game.player.max_energy = (game.player.max_energy + 4.0).min(500.0);
            game.player.hp = game.player.max_hp;
            game.player.energy = game.player.max_energy;
            game.banner = format!("LEVEL {} // POWER ASCENDED", game.player.level);
            game.banner_timer = 1.8;
            game.shake = 0.45;
        }
    }

    pub fn update_pickups(game: &mut GameData, dt: f32) {
        let player_position = game.player.pos;
        for pickup in &mut game.pickups {
            pickup.ttl -= dt;
            pickup.spin += dt * 3.2;
            if player_position.distance(pickup.pos) < 170.0 {
                let direction = safe_normalize(player_position - pickup.pos);
                pickup.velocity = damp(pickup.velocity, direction * 260.0, 7.0, dt);
            }
            pickup.pos += pickup.velocity * dt;
        }

        let mut collected = Vec::new();
        for (index, pickup) in game.pickups.iter().enumerate() {
            if pickup.ttl <= 0.0 {
                collected.push((index, None));
            } else if pickup.pos.distance(player_position) < pickup.radius + game.player.radius {
                collected.push((index, Some((pickup.kind, pickup.pos))));
            }
        }

        for (_, effect) in &collected {
            let Some((kind, position)) = effect else {
                continue;
            };
            match kind {
                PickupKind::Heal => {
                    game.player.hp = (game.player.hp + 28.0).min(game.player.max_hp);
                }
                PickupKind::Energy => {
                    game.player.energy = (game.player.energy + 35.0).min(game.player.max_energy);
                }
                PickupKind::Fury => {
                    game.player.fury = (game.player.fury + 30.0).min(100.0);
                }
                PickupKind::Xp => grant_xp(game, 44),
            }
            let color = match kind {
                PickupKind::Heal => GREEN,
                PickupKind::Energy => SKYBLUE,
                PickupKind::Fury => ORANGE,
                PickupKind::Xp => GOLD,
            };
            burst(game, *position, color, 12, 130.0, 0.22);
        }

        for (index, _) in collected.into_iter().rev() {
            if index < game.pickups.len() {
                game.pickups.swap_remove(index);
            }
        }
    }
}

mod world {
    use macroquad::{
        prelude::*,
        rand::{gen_range, srand},
    };

    use crate::{
        combat,
        constants::*,
        math::{from_angle, safe_normalize},
        model::*,
        save,
    };

    pub fn reset_run(game: &mut GameData) {
        let seed = ((get_time() * 1_000_000.0) as u64)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(0xA37E_F411);
        srand(seed);

        game.state = RunState::Playing;
        game.player = Player::default();
        game.enemies.clear();
        game.projectiles.clear();
        game.pickups.clear();
        game.particles.clear();
        game.texts.clear();
        game.world = WorldState::new(seed);
        game.score = 0;
        game.elapsed = 0.0;
        game.shake = 0.0;
        game.flash = 0.0;
        game.near_miss = 0.0;
        game.banner = "THE SHATTERED CROWN AWAKENS".to_owned();
        game.banner_timer = 3.2;
        game.save_timer = 10.0;
        game.run_finalized = false;
        game.accumulator = game.accumulator.max(0.0);
        let _ = save::store(&game.save);
    }

    pub fn spawn_position(world: &WorldState, player: Vec2, radius: f32) -> Vec2 {
        let limit = (world.arena_radius - radius - 12.0).max(80.0);
        for _ in 0..18 {
            let angle = gen_range(0.0_f32, PI2);
            let distance = world.arena_radius * gen_range(0.68_f32, 0.96_f32);
            let candidate = player + from_angle(angle) * distance;
            if candidate.length() < limit && candidate.distance(player) > 420.0 {
                return candidate;
            }
        }

        let angle = gen_range(0.0_f32, PI2);
        from_angle(angle) * limit
    }

    pub fn spawn_enemy(game: &mut GameData, kind: EnemyKind, elite: bool) {
        if game.enemies.len() >= MAX_ENEMIES {
            return;
        }

        let (hp, speed, damage, radius) = match kind {
            EnemyKind::Grunt => (42.0, 105.0, 12.0, 17.0),
            EnemyKind::Shooter => (58.0, 84.0, 10.0, 18.0),
            EnemyKind::Dasher => (52.0, 132.0, 18.0, 16.0),
            EnemyKind::Brute => (280.0, 48.0, 29.0, 31.0),
            EnemyKind::Warden => (760.0, 62.0, 22.0, 44.0),
            EnemyKind::Harvester => (125.0, 158.0, 17.0, 22.0),
        };
        let scale = difficulty_wave(game.world.wave);
        let elite_scale = if elite { 2.15 } else { 1.0 };
        let position = spawn_position(&game.world, game.player.pos, radius);
        let id = game.world.next_enemy_id;
        game.world.next_enemy_id = game.world.next_enemy_id.saturating_add(1);

        game.enemies.push(Enemy {
            id,
            kind,
            pos: position,
            velocity: Vec2::ZERO,
            radius: radius * if elite { 1.16 } else { 1.0 },
            hp: hp * scale * elite_scale,
            max_hp: hp * scale * elite_scale,
            speed: speed * if elite { 1.12 } else { 1.0 },
            damage: damage * scale.sqrt() * elite_scale,
            attack_timer: gen_range(0.2_f32, 1.2),
            shoot_timer: gen_range(0.45_f32, 1.6),
            phase: gen_range(-PI2, PI2),
            flash: 0.0,
            stun: 0.0,
            elite,
            orbit_sign: if gen_range(0, 2) == 0 { -1.0 } else { 1.0 },
        });
    }

    fn unlock_for_wave(game: &mut GameData) {
        let unlock = match game.world.wave {
            3 => Some((WeaponKind::Dawnblade, "WAVE 3 REWARD // DAWNBLADE UNLOCKED")),
            6 => Some((WeaponKind::ArcCannon, "WAVE 6 REWARD // ARC CANNON UNLOCKED")),
            10 => Some((WeaponKind::VoidLance, "WAVE 10 REWARD // VOID LANCE UNLOCKED")),
            _ => None,
        };
        if let Some((weapon, message)) = unlock {
            if !game.save.unlocked_weapons.iter().any(|key| key == weapon.key()) {
                game.save.unlocked_weapons.push(weapon.key().to_owned());
                game.banner = message.to_owned();
                game.banner_timer = 3.0;
                let _ = save::store(&game.save);
            }
        }
    }

    pub fn update_world(game: &mut GameData, dt: f32) {
        game.elapsed += dt;
        game.world.time += dt;
        game.world.anomaly = ((game.world.time * 0.07).sin() * 0.5 + 0.5) * 0.7;

        let wave_factor = (game.world.wave as f32 / 30.0).clamp(0.0, 1.0);
        game.world.arena_radius = ARENA_MIN_RADIUS
            + (ARENA_MAX_RADIUS - ARENA_MIN_RADIUS) * (0.5 + 0.5 * wave_factor);
        game.shake = (game.shake - dt * 3.8).max(0.0);
        game.flash = (game.flash - dt * 2.5).max(0.0);
        game.near_miss = (game.near_miss - dt * 2.0).max(0.0);
        game.banner_timer = (game.banner_timer - dt).max(0.0);

        let active_pressure = game.enemies.len();
        if active_pressure < MAX_ACTIVE_ENEMIES {
            let rate = 0.62 + game.world.wave.min(250) as f32 * 0.027;
            game.world.spawn_budget = (game.world.spawn_budget + dt * rate).min(4.0);
            let count = game.world.spawn_budget.floor().min(4.0) as usize;
            for _ in 0..count {
                if game.enemies.len() >= MAX_ACTIVE_ENEMIES {
                    break;
                }
                let roll = gen_range(0.0_f32, 100.0);
                let kind = if game.world.wave >= 18 && roll < 5.0 {
                    EnemyKind::Warden
                } else if game.world.wave >= 9 && roll < 13.0 {
                    EnemyKind::Brute
                } else if game.world.wave >= 6 && roll < 27.0 {
                    EnemyKind::Harvester
                } else if roll < 52.0 {
                    EnemyKind::Grunt
                } else if roll < 78.0 {
                    EnemyKind::Shooter
                } else {
                    EnemyKind::Dasher
                };
                let elite_chance = (3.0 + game.world.wave.min(50) as f32 * 0.22).min(14.0);
                let elite = game.world.wave >= 5 && gen_range(0.0_f32, 100.0) < elite_chance;
                spawn_enemy(game, kind, elite);
            }
            game.world.spawn_budget = (game.world.spawn_budget - count as f32).max(0.0);
        }

        game.world.wave_progress = (game.world.kills_for_wave as f32
            / game.world.target_kills.max(1) as f32)
            .clamp(0.0, 1.0);

        if game.world.kills_for_wave >= game.world.target_kills {
            game.world.wave = game.world.wave.saturating_add(1);
            game.world.kills_for_wave = 0;
            game.world.target_kills = 10_u32
                .saturating_add(game.world.wave.saturating_mul(7))
                .min(500);
            game.world.wave_progress = 0.0;
            game.player.energy = game.player.max_energy;
            game.player.hp =
                (game.player.hp + game.player.max_hp * 0.12).min(game.player.max_hp);
            game.save.best_wave = game.save.best_wave.max(game.world.wave);
            if game.world.wave % 5 == 0 {
                game.banner = format!("ELITE TIDE // WAVE {}", game.world.wave);
            } else {
                game.banner = format!(
                    "WAVE {} // THREAT RATING +{}%",
                    game.world.wave,
                    game.world.wave.saturating_mul(7)
                );
            }
            game.banner_timer = 2.7;
            game.shake = 1.0;
            unlock_for_wave(game);
            let _ = save::store(&game.save);
        }
    }

    pub fn resolve_world_bounds(game: &mut GameData) {
        let limit = (game.world.arena_radius - game.player.radius - 6.0).max(40.0);
        let distance = game.player.pos.length();
        if distance > limit {
            let normal = safe_normalize(game.player.pos);
            game.player.pos = normal * limit;
            let outward_speed = game.player.velocity.dot(normal);
            if outward_speed > 0.0 {
                game.player.velocity -= normal * outward_speed;
            }
            game.flash = game.flash.max(0.08);
        }
    }

    use macroquad::rand::gen_range;
}

mod game {
    use macroquad::{prelude::*, rand::gen_range};

    use crate::{
        ai,
        combat,
        constants::*,
        math::{damp, safe_normalize},
        model::*,
        save,
        world,
    };

    impl GameData {
        pub fn new() -> Self {
            Self {
                state: RunState::Title,
                player: Player::default(),
                enemies: Vec::new(),
                projectiles: Vec::new(),
                pickups: Vec::new(),
                particles: Vec::new(),
                texts: Vec::new(),
                world: WorldState::new(0xA37E_F411),
                save: save::load(),
                score: 0,
                accumulator: 0.0,
                elapsed: 0.0,
                shake: 0.0,
                flash: 0.0,
                near_miss: 0.0,
                banner: String::new(),
                banner_timer: 0.0,
                save_timer: 10.0,
                run_finalized: false,
                current_input: InputFrame::default(),
                pending_input: InputEdges::default(),
            }
        }

        pub fn capture_input(&mut self) {
            let movement = Vec2::new(
                (is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)) as u8 as f32
                    - (is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)) as u8 as f32,
                (is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)) as u8 as f32
                    - (is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)) as u8 as f32,
            );
            let (mouse_x, mouse_y) = mouse_position();
            let aim_vector = vec2(
                mouse_x - screen_width() * 0.5,
                mouse_y - screen_height() * 0.5,
            );
            let aim = if aim_vector.length_squared() > 0.0001 {
                safe_normalize(aim_vector)
            } else {
                Vec2::X
            };

            self.current_input = InputFrame {
                movement: safe_normalize(movement),
                aim,
                fire: is_mouse_button_down(MouseButton::Left),
                ..InputFrame::default()
            };

            self.pending_input.start |= is_key_pressed(KeyCode::Enter)
                || is_mouse_button_pressed(MouseButton::Left);
            self.pending_input.pause |= is_key_pressed(KeyCode::Escape);
            self.pending_input.restart |= is_key_pressed(KeyCode::R);
            self.pending_input.dash |= is_key_pressed(KeyCode::Space);
            self.pending_input.nova |= is_key_pressed(KeyCode::Q);
            self.pending_input.overdrive |= is_key_pressed(KeyCode::E);
            self.pending_input.next_weapon |= is_key_pressed(KeyCode::Tab);
            if is_key_pressed(KeyCode::Key1) {
                self.pending_input.selected_weapon = Some(WeaponKind::Repeater);
            } else if is_key_pressed(KeyCode::Key2) {
                self.pending_input.selected_weapon = Some(WeaponKind::Dawnblade);
            } else if is_key_pressed(KeyCode::Key3) {
                self.pending_input.selected_weapon = Some(WeaponKind::ArcCannon);
            } else if is_key_pressed(KeyCode::Key4) {
                self.pending_input.selected_weapon = Some(WeaponKind::VoidLance);
            }
        }

        pub fn next_input(&mut self) -> InputFrame {
            let mut input = self.current_input;
            input.start = self.pending_input.start;
            input.pause = self.pending_input.pause;
            input.restart = self.pending_input.restart;
            input.dash = self.pending_input.dash;
            input.nova = self.pending_input.nova;
            input.overdrive = self.pending_input.overdrive;
            input.next_weapon = self.pending_input.next_weapon;
            input.selected_weapon = self.pending_input.selected_weapon;
            self.pending_input = InputEdges::default();
            input
        }

        pub fn update(&mut self, dt: f32, input: InputFrame) {
            match self.state {
                RunState::Title => {
                    if input.start {
                        world::reset_run(self);
                    }
                }
                RunState::Paused => {
                    if input.pause || input.start {
                        self.state = RunState::Playing;
                    }
                }
                RunState::Dead => {
                    if input.restart {
                        world::reset_run(self);
                    }
                }
                RunState::Playing => {
                    if input.pause {
                        self.state = RunState::Paused;
                        return;
                    }
                    self.simulate(dt, input);
                }
            }
        }

        fn simulate(&mut self, dt: f32, input: InputFrame) {
            let dt = dt.clamp(0.0, FIXED_DT * 2.0);
            self.player.fire_timer = (self.player.fire_timer - dt).max(0.0);
            self.player.dash_timer = (self.player.dash_timer - dt).max(0.0);
            self.player.dash_time = (self.player.dash_time - dt).max(0.0);
            self.player.invulnerable = (self.player.invulnerable - dt).max(0.0);
            self.player.nova_timer = (self.player.nova_timer - dt).max(0.0);
            self.player.overdrive = (self.player.overdrive - dt).max(0.0);
            self.player.combo_timer = (self.player.combo_timer - dt).max(0.0);
            if self.player.combo_timer <= 0.0 {
                self.player.combo = 0;
            }

            if let Some(weapon) = input.selected_weapon {
                self.equip_weapon(weapon);
            } else if input.next_weapon {
                self.cycle_weapon();
            }

            if input.dash && self.player.dash_timer <= 0.0 {
                let direction = if input.movement.length_squared() > 0.0 {
                    input.movement
                } else {
                    input.aim
                };
                self.player.velocity = direction * DASH_SPEED;
                self.player.dash_time = 0.14;
                self.player.dash_timer = 0.95;
                self.player.invulnerable = 0.22;
                ai::burst(self, self.player.pos, SKYBLUE, 24, 300.0, 0.28);
                self.shake = self.shake.max(0.35);
            }

            if input.nova && self.player.nova_timer <= 0.0 && self.player.energy >= 45.0 {
                self.player.energy -= 45.0;
                self.player.nova_timer = 5.0;
                let origin = self.player.pos;
                for enemy in &mut self.enemies {
                    let distance = enemy.pos.distance(origin);
                    if distance < 360.0 {
                        enemy.hp -= 125.0 * (1.0 - distance / 360.0);
                        enemy.stun = enemy.stun.max(0.6);
                    }
                }
                ai::burst(self, origin, VIOLET, 70, 420.0, 0.65);
                self.shake = self.shake.max(0.8);
            }

            if input.overdrive && self.player.fury >= 100.0 {
                self.player.fury = 0.0;
                self.player.overdrive = 8.0;
                self.banner = "OVERDRIVE // TIME TO BREAK THE HORDE".to_owned();
                self.banner_timer = 1.8;
                ai::burst(self, self.player.pos, GOLD, 35, 260.0, 0.45);
            }

            if input.fire {
                combat::fire_weapon(self, input.aim);
            }

            let max_speed = if self.player.dash_time > 0.0 {
                DASH_SPEED
            } else {
                PLAYER_SPEED * if self.player.overdrive > 0.0 { 1.18 } else { 1.0 }
            };
            let desired_velocity = input.movement * max_speed;
            if self.player.dash_time <= 0.0 {
                self.player.velocity = damp(self.player.velocity, desired_velocity, 16.0, dt);
            }
            self.player.pos += self.player.velocity * dt;
            self.player.energy =
                (self.player.energy + dt * 11.0).min(self.player.max_energy);
            world::resolve_world_bounds(self);

            world::update_world(self, dt);
            ai::update_enemies(self, dt);
            combat::update_projectiles(self, dt);
            combat::update_pickups(self, dt);
            update_fx(self, dt);

            if self.player.fury >= 100.0 && self.player.overdrive <= 0.0 {
                self.banner = "FURY PRIMED // [E] OVERDRIVE".to_owned();
                self.banner_timer = self.banner_timer.max(0.1);
            }

            self.save_timer -= dt;
            if self.save_timer <= 0.0 {
                self.save_timer = 10.0;
                let _ = save::store(&self.save);
            }
        }

        fn equip_weapon(&mut self, weapon: WeaponKind) {
            if self.save.unlocked_weapons.iter().any(|key| key == weapon.key())
                && self.player.weapon != weapon
            {
                self.player.weapon = weapon;
                self.banner = format!("WEAPON // {}", weapon.label());
                self.banner_timer = 1.2;
            }
        }

        fn cycle_weapon(&mut self) {
            let weapons = WeaponKind::ALL;
            let start = weapons
                .iter()
                .position(|weapon| *weapon == self.player.weapon)
                .unwrap_or(0);
            for offset in 1..=weapons.len() {
                let candidate = weapons[(start + offset) % weapons.len()];
                if self.save.unlocked_weapons.iter().any(|key| key == candidate.key()) {
                    self.equip_weapon(candidate);
                    return;
                }
            }
        }
    }

    pub fn finish_run(game: &mut GameData) {
        if game.run_finalized {
            return;
        }
        game.run_finalized = true;
        game.save.best_wave = game.save.best_wave.max(game.world.wave);
        game.save.best_score = game.save.best_score.max(game.score);
        let _ = save::store(&game.save);
    }

    fn update_fx(game: &mut GameData, dt: f32) {
        for particle in &mut game.particles {
            particle.life -= dt;
            particle.velocity *= 1.0 / (1.0 + particle.drag * dt);
            particle.velocity.y += particle.gravity * dt;
            particle.pos += particle.velocity * dt;
        }
        game.particles.retain(|particle| particle.life > 0.0);
        if game.particles.len() > MAX_PARTICLES {
            let extra = game.particles.len() - MAX_PARTICLES;
            game.particles.drain(0..extra);
        }

        for text in &mut game.texts {
            text.life -= dt;
            text.pos.y -= dt * 28.0;
        }
        game.texts.retain(|text| text.life > 0.0);
    }

    use macroquad::rand::gen_range;
}

mod render {
    use macroquad::{prelude::*, rand::gen_range};

    use crate::{
        ai::enemy_color,
        constants::*,
        math::from_angle,
        model::*,
    };

    pub struct View {
        pub center: Vec2,
        pub zoom: f32,
        pub shake: Vec2,
    }

    impl View {
        pub fn world_to_screen(&self, position: Vec2) -> Vec2 {
            (position - self.center) * self.zoom
                + vec2(screen_width() * 0.5, screen_height() * 0.5)
                + self.shake
        }

        pub fn scale(&self, value: f32) -> f32 {
            value * self.zoom
        }
    }

    fn fade(color: Color, alpha: f32) -> Color {
        Color::new(color.r, color.g, color.b, (color.a * alpha).clamp(0.0, 1.0))
    }

    pub fn draw(game: &GameData) {
        clear_background(Color::new(0.014, 0.018, 0.028, 1.0));
        let screen_min = screen_width().min(screen_height()).max(1.0);
        let zoom = ((screen_min * 0.47) / game.world.arena_radius.max(1.0)).clamp(0.38, 0.9)
            * (1.0 - (game.enemies.len() as f32 / 380.0).clamp(0.0, 0.22));
        let shake = if game.shake > 0.0 {
            vec2((game.elapsed * 43.0).sin(), (game.elapsed * 57.0).cos())
                * game.shake
                * 18.0
        } else {
            Vec2::ZERO
        };
        let view = View {
            center: game.player.pos,
            zoom,
            shake,
        };

        draw_arena(game, &view);
        draw_pickups(game, &view);
        draw_projectiles(game, &view);
        draw_enemies(game, &view);
        draw_player(game, &view);
        draw_particles(game, &view);
        draw_texts(game, &view);
        draw_world_effects(game, &view);
        draw_crosshair(game);
    }

    fn draw_arena(game: &GameData, view: &View) {
        let center = view.world_to_screen(Vec2::ZERO);
        let radius = view.scale(game.world.arena_radius);
        draw_circle(
            center.x,
            center.y,
            radius,
            Color::new(0.025, 0.034, 0.055, 1.0),
        );

        let steps = (game.world.arena_radius / ARENA_CELL).ceil() as i32;
        for grid in -steps..=steps {
            let coordinate = grid as f32 * ARENA_CELL;
            let start_x = view.world_to_screen(vec2(coordinate, -game.world.arena_radius));
            let end_x = view.world_to_screen(vec2(coordinate, game.world.arena_radius));
            draw_line(
                start_x.x,
                start_x.y,
                end_x.x,
                end_x.y,
                1.0,
                fade(Color::new(0.10, 0.16, 0.26, 1.0), 0.35),
            );
            let start_y = view.world_to_screen(vec2(-game.world.arena_radius, coordinate));
            let end_y = view.world_to_screen(vec2(game.world.arena_radius, coordinate));
            draw_line(
                start_y.x,
                start_y.y,
                end_y.x,
                end_y.y,
                1.0,
                fade(Color::new(0.10, 0.16, 0.26, 1.0), 0.35),
            );
        }

        draw_circle_lines(
            center.x,
            center.y,
            radius,
            2.5,
            fade(SKYBLUE, 0.42 + game.world.anomaly * 0.25),
        );
        for ring in 0..7 {
            let ring_radius = radius * (0.16 + ring as f32 * 0.12);
            draw_circle_lines(center.x, center.y, ring_radius, 1.0, fade(VIOLET, 0.10));
        }
    }

    fn draw_player(game: &GameData, view: &View) {
        let position = view.world_to_screen(game.player.pos);
        let radius = view.scale(game.player.radius);
        let blink = game.player.invulnerable > 0.0 && (game.elapsed * 40.0) as i32 % 2 == 0;
        let core = if blink { fade(WHITE, 0.25) } else { WHITE };
        draw_circle(position.x, position.y, radius * 1.7, fade(SKYBLUE, 0.08));
        draw_circle(position.x, position.y, radius, core);

        let aim = if game.current_input.aim.length_squared() > 0.0 {
            game.current_input.aim
        } else {
            Vec2::X
        };
        let tip = position + aim * radius * 1.75;
        draw_line(
            position.x,
            position.y,
            tip.x,
            tip.y,
            view.scale(5.0).max(1.5),
            if game.player.overdrive > 0.0 { GOLD } else { SKYBLUE },
        );
        draw_circle(
            position.x,
            position.y,
            radius * 0.56,
            Color::new(0.05, 0.12, 0.20, 1.0),
        );
        draw_circle_lines(position.x, position.y, radius * 1.35, 2.0, fade(SKYBLUE, 0.55));
        if game.player.dash_time > 0.0 {
            draw_circle_lines(position.x, position.y, radius * 2.1, 2.0, fade(WHITE, 0.7));
        }
    }

    fn draw_enemies(game: &GameData, view: &View) {
        for enemy in &game.enemies {
            let position = view.world_to_screen(enemy.pos);
            let radius = view.scale(enemy.radius);
            let color = if enemy.flash > 0.0 { WHITE } else { enemy_color(enemy.kind) };
            draw_circle(
                position.x,
                position.y,
                radius * 1.45,
                fade(enemy_color(enemy.kind), 0.08),
            );
            match enemy.kind {
                EnemyKind::Warden => {
                    draw_poly(position.x, position.y, 6, radius, enemy.phase * 0.12, color);
                    draw_circle_lines(position.x, position.y, radius * 1.4, 2.0, fade(color, 0.7));
                }
                EnemyKind::Brute => {
                    draw_rectangle(
                        position.x - radius,
                        position.y - radius,
                        radius * 2.0,
                        radius * 2.0,
                        color,
                    );
                    draw_rectangle_lines(
                        position.x - radius,
                        position.y - radius,
                        radius * 2.0,
                        radius * 2.0,
                        1.5,
                        fade(WHITE, 0.45),
                    );
                }
                _ => {
                    draw_circle(position.x, position.y, radius, color);
                    draw_circle_lines(position.x, position.y, radius, 1.5, fade(WHITE, 0.35));
                }
            }
            if enemy.elite {
                draw_circle_lines(position.x, position.y, radius * 1.7, 2.0, GOLD);
            }
            let health = (enemy.hp / enemy.max_hp.max(0.001)).clamp(0.0, 1.0);
            draw_rectangle(
                position.x - radius,
                position.y - radius - 8.0,
                radius * 2.0,
                3.0,
                fade(BLACK, 0.8),
            );
            draw_rectangle(
                position.x - radius,
                position.y - radius - 8.0,
                radius * 2.0 * health,
                3.0,
                if enemy.elite { GOLD } else { color },
            );
        }
    }

    fn draw_projectiles(game: &GameData, view: &View) {
        for projectile in &game.projectiles {
            let start = view.world_to_screen(projectile.prev_pos);
            let end = view.world_to_screen(projectile.pos);
            let color = if projectile.owner == ProjectileOwner::Enemy {
                Color::new(1.0, 0.28, 0.32, 1.0)
            } else {
                projectile.color
            };
            draw_line(
                start.x,
                start.y,
                end.x,
                end.y,
                view.scale(projectile.radius * 1.35).max(1.0),
                fade(color, 0.28),
            );
            draw_circle(end.x, end.y, view.scale(projectile.radius).max(1.4), color);
        }
    }

    fn draw_pickups(game: &GameData, view: &View) {
        for pickup in &game.pickups {
            let position = view.world_to_screen(pickup.pos);
            let radius = view.scale(pickup.radius)
                * (1.0 + 0.12 * (pickup.spin * 3.0).sin());
            let color = match pickup.kind {
                PickupKind::Heal => GREEN,
                PickupKind::Energy => SKYBLUE,
                PickupKind::Fury => ORANGE,
                PickupKind::Xp => GOLD,
            };
            draw_circle(
                position.x,
                position.y,
                radius * 1.8,
                fade(color, 0.08),
            );
            draw_circle(position.x, position.y, radius.max(2.0), color);
            draw_circle_lines(
                position.x,
                position.y,
                radius * 1.45,
                1.5,
                fade(WHITE, 0.45),
            );
        }
    }

    fn draw_particles(game: &GameData, view: &View) {
        for particle in &game.particles {
            let position = view.world_to_screen(particle.pos);
            let alpha = (particle.life / particle.max_life.max(0.001)).clamp(0.0, 1.0);
            let radius = view.scale(particle.size * (0.55 + 0.45 * alpha)).max(0.5);
            draw_circle(position.x, position.y, radius, fade(particle.color, alpha));
        }
    }

    fn draw_texts(game: &GameData, view: &View) {
        for floating in &game.texts {
            let position = view.world_to_screen(floating.pos);
            let alpha = (floating.life / floating.max_life.max(0.001)).clamp(0.0, 1.0);
            draw_text(
                &floating.text,
                position.x,
                position.y,
                (floating.size * view.zoom).max(9.0),
                fade(floating.color, alpha),
            );
        }
    }

    fn draw_world_effects(game: &GameData, view: &View) {
        let time = game.world.time;
        for index in 0..12 {
            let angle = time * 0.12 + index as f32 * 0.53;
            let distance = game.world.arena_radius * 0.75
                + ((time + index as f32 * 2.1).sin() * 120.0);
            let position = game.player.pos + from_angle(angle) * distance;
            let screen = view.world_to_screen(position);
            let direction = from_angle(angle);
            let length = view.scale(45.0);
            draw_line(
                screen.x,
                screen.y,
                screen.x + length * direction.x,
                screen.y + length * direction.y,
                1.0,
                fade(VIOLET, 0.08 + game.world.anomaly * 0.06),
            );
        }
    }

    fn draw_crosshair(game: &GameData) {
        let (x, y) = mouse_position();
        let color = if game.player.overdrive > 0.0 { GOLD } else { SKYBLUE };
        draw_circle_lines(x, y, 7.0, 1.5, fade(color, 0.85));
        draw_line(x - 12.0, y, x - 4.0, y, 1.0, color);
        draw_line(x + 4.0, y, x + 12.0, y, 1.0, color);
        draw_line(x, y - 12.0, x, y - 4.0, 1.0, color);
        draw_line(x, y + 4.0, x, y + 12.0, 1.0, color);
    }

    pub fn draw_overlay(game: &GameData) {
        let width = screen_width();
        let height = screen_height();
        if game.flash > 0.0 {
            draw_rectangle(0.0, 0.0, width, height, fade(RED, game.flash * 0.35));
        }

        if game.banner_timer > 0.0 && !game.banner.is_empty() {
            let alpha = (game.banner_timer.min(0.5) * 2.0).min(1.0);
            let metrics = measure_text(&game.banner, None, 28, 1.0);
            draw_text(
                &game.banner,
                width * 0.5 - metrics.width * 0.5,
                84.0,
                28.0,
                fade(WHITE, alpha),
            );
        }

        match game.state {
            RunState::Title => title_screen(game),
            RunState::Paused => pause_screen(),
            RunState::Dead => dead_screen(game),
            RunState::Playing => {}
        }
    }

    fn title_screen(game: &GameData) {
        let width = screen_width();
        let height = screen_height();
        draw_rectangle(0.0, 0.0, width, height, Color::new(0.01, 0.012, 0.02, 0.88));
        let title = "AETHERFALL";
        let subtitle = "THE SHATTERED CROWN";
        let title_metrics = measure_text(title, None, 72, 1.0);
        draw_text(
            title,
            width * 0.5 - title_metrics.width * 0.5,
            height * 0.34,
            72.0,
            SKYBLUE,
        );
        let subtitle_metrics = measure_text(subtitle, None, 24, 1.0);
        draw_text(
            subtitle,
            width * 0.5 - subtitle_metrics.width * 0.5,
            height * 0.34 + 42.0,
            24.0,
            VIOLET,
        );
        let hint = "PRESS ENTER OR CLICK TO DESCEND";
        let hint_metrics = measure_text(hint, None, 18, 1.0);
        draw_text(
            hint,
            width * 0.5 - hint_metrics.width * 0.5,
            height * 0.70,
            18.0,
            WHITE,
        );
        let record = format!(
            "BEST WAVE {}    BEST SCORE {}    LIFETIME KILLS {}",
            game.save.best_wave, game.save.best_score, game.save.lifetime_kills
        );
        let metrics = measure_text(&record, None, 16, 1.0);
        draw_text(
            &record,
            width * 0.5 - metrics.width * 0.5,
            height * 0.77,
            16.0,
            Color::new(0.64, 0.72, 0.82, 1.0),
        );
        let footer = "WASD MOVE  •  MOUSE FIRE/AIM  •  SPACE DASH  •  Q NOVA  •  ESC PAUSE";
        let footer_metrics = measure_text(footer, None, 15, 1.0);
        draw_text(
            footer,
            width * 0.5 - footer_metrics.width * 0.5,
            height * 0.85,
            15.0,
            Color::new(0.70, 0.75, 0.84, 1.0),
        );
    }

    fn pause_screen() {
        let width = screen_width();
        let height = screen_height();
        draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.02, 0.68));
        centered_text("PAUSED", height * 0.40, 54, WHITE);
        centered_text("ESC / ENTER — RESUME", height * 0.50, 20, SKYBLUE);
        centered_text("TAB OR 1–4 — CHANGE WEAPON", height * 0.55, 18, WHITE);
    }

    fn dead_screen(game: &GameData) {
        let height = screen_height();
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            height,
            Color::new(0.05, 0.0, 0.01, 0.72),
        );
        centered_text("RUN ENDED", height * 0.38, 54, RED);
        centered_text(
            &format!("WAVE {}    SCORE {}", game.world.wave, game.score),
            height * 0.46,
            22,
            GOLD,
        );
        centered_text("PRESS R TO RESTART", height * 0.55, 20, WHITE);
        centered_text(
            &format!(
                "PERSONAL BEST — WAVE {} / SCORE {}",
                game.save.best_wave, game.save.best_score
            ),
            height * 0.61,
            16,
            SKYBLUE,
        );
    }

    fn centered_text(text: &str, y: f32, size: u16, color: Color) {
        let metrics = measure_text(text, None, size, 1.0);
        draw_text(
            text,
            screen_width() * 0.5 - metrics.width * 0.5,
            y,
            size as f32,
            color,
        );
    }
}

mod ui {
    use macroquad::prelude::*;

    use crate::{constants::xp_to_next, model::*};

    pub fn draw_hud(game: &GameData) {
        if game.state != RunState::Playing {
            return;
        }
        let width = screen_width();
        let height = screen_height();
        panel(18.0, 18.0, 310.0, 101.0, Color::new(0.02, 0.035, 0.06, 0.84));

        draw_text(
            &format!("LVL {}   WAVE {}", game.player.level, game.world.wave),
            30.0,
            43.0,
            20.0,
            WHITE,
        );
        bar(30.0, 55.0, 235.0, 10.0, game.player.hp / game.player.max_hp.max(1.0), RED);
        draw_text(
            &format!("{:.0}/{:.0}", game.player.hp, game.player.max_hp),
            272.0,
            64.0,
            11.0,
            WHITE,
        );
        bar(
            30.0,
            73.0,
            235.0,
            8.0,
            game.player.energy / game.player.max_energy.max(1.0),
            SKYBLUE,
        );
        draw_text(&format!("E {:.0}", game.player.energy), 272.0, 81.0, 11.0, WHITE);
        bar(30.0, 89.0, 235.0, 7.0, game.player.fury / 100.0, GOLD);
        draw_text("FURY", 272.0, 96.0, 10.0, GOLD);

        let right_x = (width - 250.0).max(18.0);
        panel(right_x, 18.0, 232.0, 76.0, Color::new(0.02, 0.035, 0.06, 0.82));
        draw_text(&format!("SCORE {:09}", game.score), right_x + 14.0, 42.0, 18.0, GOLD);
        draw_text(
            &format!("KILLS {:05}", game.player.kills),
            right_x + 14.0,
            66.0,
            16.0,
            WHITE,
        );
        draw_text(
            &format!("DMG {:.0}", game.player.damage_done),
            right_x + 122.0,
            66.0,
            14.0,
            SKYBLUE,
        );

        let progress = game.world.wave_progress.clamp(0.0, 1.0);
        bar(18.0, height - 29.0, width - 36.0, 5.0, progress, SKYBLUE);
        let requirement = xp_to_next(game.player.level).max(1);
        bar(
            18.0,
            height - 20.0,
            width - 36.0,
            4.0,
            game.player.xp as f32 / requirement as f32,
            VIOLET,
        );
        centered_text(
            &format!(
                "{}   |   COMBO x{}   |   WAVE ELIMINATIONS {}/{}",
                game.player.weapon.label(),
                game.player.combo,
                game.world.kills_for_wave,
                game.world.target_kills
            ),
            height - 42.0,
            16,
            WHITE,
        );
        if game.player.overdrive > 0.0 {
            centered_text("OVERDRIVE", 122.0, 18, GOLD);
        } else if game.player.nova_timer > 0.0 {
            centered_text(&format!("NOVA {:.1}s", game.player.nova_timer), 122.0, 16, SKYBLUE);
        }
        let controls = "SPACE DASH  •  Q NOVA  •  E OVERDRIVE  •  TAB / 1–4 WEAPON  •  ESC PAUSE";
        draw_text(controls, 20.0, height - 50.0, 13.0, Color::new(0.66, 0.73, 0.82, 1.0));
    }

    fn panel(x: f32, y: f32, width: f32, height: f32, color: Color) {
        draw_rectangle(x + 4.0, y + 4.0, width, height, Color::new(0.0, 0.0, 0.0, 0.25));
        draw_rectangle(x, y, width, height, color);
        draw_rectangle_lines(
            x,
            y,
            width,
            height,
            1.0,
            Color::new(0.18, 0.30, 0.46, 0.65),
        );
    }

    fn bar(x: f32, y: f32, width: f32, height: f32, progress: f32, color: Color) {
        draw_rectangle(x, y, width, height, Color::new(0.0, 0.0, 0.0, 0.65));
        draw_rectangle(x, y, width * progress.clamp(0.0, 1.0), height, color);
    }

    fn centered_text(text: &str, y: f32, size: u16, color: Color) {
        let metrics = measure_text(text, None, size, 1.0);
        draw_text(
            text,
            screen_width() * 0.5 - metrics.width * 0.5,
            y,
            size as f32,
            color,
        );
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Aetherfall: The Shattered Crown".to_owned(),
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
    use constants::FIXED_DT;
    use game::GameData;

    let mut game = GameData::new();
    loop {
        let frame_dt = get_frame_time().clamp(0.0, 0.05);
        game.capture_input();
        game.accumulator += frame_dt;

        while game.accumulator >= FIXED_DT {
            let input = game.next_input();
            game.update(FIXED_DT, input);
            game.accumulator -= FIXED_DT;
        }

        render::draw(&game);
        render::draw_overlay(&game);
        ui::draw_hud(&game);
        next_frame().await;
    }
}
