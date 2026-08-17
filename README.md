# Aetherfall: The Shattered Crown

A large-scale Rust action game prototype built around a deterministic simulation core, procedural arenas, data-driven combat, enemy behavior, persistence, and a presentation layer designed for iteration.

## Vision

Aetherfall is a top-down action roguelite with RPG progression, elite enemies, reactive encounters, procedural arenas, and a deliberately modular codebase. The goal is not a disposable demo: systems are separated so new weapons, enemies, abilities, biomes, and game modes can be added without rewriting the simulation.

## Current architecture

- `game`: fixed-step simulation, run state, clocks, difficulty scaling
- `world`: procedural arena generation and spatial queries
- `combat`: weapons, projectiles, damage, status effects, critical hits
- `ai`: lightweight state machines for pursuit, orbiting, and kiting
- `entities`: player, enemies, projectiles, pickups
- `render`: macroquad presentation and camera
- `ui`: HUD, menus, combat feedback
- `save`: versioned JSON persistence

## Controls

- WASD / Arrow keys: move
- Mouse: aim
- Left mouse: fire
- Space: dash
- Q: Nova ability
- E: temporary overdrive
- Esc: pause
- R: restart the run after death

## Run

```bash
cargo run --release
```

## Test

```bash
cargo test
```

The project intentionally ships with no third-party art or audio assets yet; the renderer uses procedural shapes and effects so the simulation can be exercised immediately.

## License

MIT
