# Aetherfall: The Shattered Crown

A top-down Rust action roguelite prototype built with Macroquad. The playable game is consolidated into **one standalone source file: `src/main.rs`**. Its internal modules keep simulation, combat, AI, rendering, UI, and persistence logically separated while still allowing the file to be copied or reviewed as a single deliverable.

## Gameplay

- Fixed-step simulation with buffered input edges so quick key presses are not lost between render frames.
- Six enemy archetypes, elite variants, wave-based pressure, arena-bound movement, enemy projectiles, and a growing threat curve.
- Four weapons: Repeater (starting weapon), Dawnblade (wave 3), Arc Cannon (wave 6), and Void Lance (wave 10). Unlocks persist across runs.
- Dash invulnerability, Nova area damage, Fury/Overdrive, XP and levels, combo tracking, pickups, critical hits, and explosive splash damage.
- Procedural visual effects and camera shake, with bounded particle/projectile/pickup populations.
- Version-tolerant profile loading, input validation, best-wave and best-score records, lifetime statistics, and temporary-file save writes.

## Controls

- **WASD / arrows** — move
- **Mouse** — aim
- **Left mouse** — fire
- **Space** — dash
- **Q** — Nova (45 energy, 5-second cooldown)
- **E** — Overdrive when Fury reaches 100
- **Tab** — cycle unlocked weapons
- **1–4** — select an unlocked weapon directly
- **Esc** — pause/resume
- **R** — restart after defeat

## Run

Requires the Rust stable toolchain.

```bash
cargo run --release
```

## Test

```bash
cargo test --all-targets
cargo check --release
```

The project intentionally uses procedural graphics and no external art or audio assets. Player data is stored in `save/profile.json`; set `AETHERFALL_SAVE_DIR` to change the save directory.
