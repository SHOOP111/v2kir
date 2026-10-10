use crate::constants::SAVE_VERSION;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Temporary world-scale modifiers that make each run feel less predictable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiftEvent {
    BloodMoon,
    Overcharge,
    TimeSnare,
    FortuneFlux,
}

impl RiftEvent {
    pub const ALL: [Self; 4] =
        [Self::BloodMoon, Self::Overcharge, Self::TimeSnare, Self::FortuneFlux];

    pub fn title(self) -> &'static str {
        match self {
            Self::BloodMoon => "BLOOD MOON",
            Self::Overcharge => "OVERCHARGE",
            Self::TimeSnare => "TIME SNARE",
            Self::FortuneFlux => "FORTUNE FLUX",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::BloodMoon => "Enemies hit harder",
            Self::Overcharge => "More energy + weapon damage",
            Self::TimeSnare => "Enemies move slower",
            Self::FortuneFlux => "Enemies drop more loot",
        }
    }
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
            radius: 18.0,
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
    pub phase: f32,
    pub flash: f32,
    pub stun: f32,
    pub elite: bool,
    pub orbit_sign: f32,
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub lifetime: f32,
    pub pierce: i32,
    pub bounces: i32,
    pub color: Color,
    pub explosive: bool,
    pub homing: f32,
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
pub struct SaveData {
    pub version: u32,
    pub best_wave: u32,
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
    pub rift_event: Option<RiftEvent>,
    pub rift_timer: f32,
    pub rift_cooldown: f32,
}

impl WorldState {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            time: 0.0,
            wave: 1,
            wave_progress: 0.0,
            spawn_budget: 3.0,
            next_enemy_id: 1,
            kills_for_wave: 0,
            target_kills: 14,
            anomaly: 0.0,
            arena_radius: 1100.0,
            rift_event: None,
            rift_timer: 0.0,
            rift_cooldown: 18.0,
        }
    }
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
}
