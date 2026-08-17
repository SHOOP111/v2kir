pub const WORLD_W: f32 = 5200.0;
pub const WORLD_H: f32 = 5200.0;
pub const FIXED_DT: f32 = 1.0 / 120.0;
pub const PLAYER_RADIUS: f32 = 18.0;
pub const PLAYER_SPEED: f32 = 330.0;
pub const DASH_SPEED: f32 = 980.0;
pub const BULLET_SPEED: f32 = 920.0;
pub const MAX_PARTICLES: usize = 3200;
pub const MAX_ENEMIES: usize = 900;
pub const SAVE_VERSION: u32 = 1;
pub const ARENA_CELL: f32 = 260.0;
pub const ARENA_MIN_RADIUS: f32 = 780.0;
pub const ARENA_MAX_RADIUS: f32 = 2100.0;
pub const PI2: f32 = std::f32::consts::PI * 2.0;

pub fn difficulty_wave(wave: u32) -> f32 {
    1.0 + (wave.saturating_sub(1) as f32) * 0.075 + ((wave / 8) as f32).sqrt() * 0.18
}

pub fn xp_to_next(level: u32) -> u32 {
    let n = level.max(1) as f32;
    (85.0 + n.powf(1.55) * 42.0) as u32
}
