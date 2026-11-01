# MaazX Potato Edition

An original offline Rust voxel-survival game targeting older Windows PCs with Intel integrated graphics and a desktop OpenGL 2.1 baseline.

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/YOUR_USERNAME/maazx)

---

## Project Goal

The long-term benchmark is the broad single-player feature scope of Minecraft Java 1.20.1. MaazX is a clean-room game, not a Minecraft client; it uses original code and assets and does not promise Java multiplayer or world-file compatibility.

### Target Machine
| Spec | Value |
|------|-------|
| CPU | Intel Core i3 @ 2.53GHz |
| RAM | 4 GB |
| GPU | Intel HD Graphics (2010) |
| VRAM | 64 MB |
| API | OpenGL 2.1 / DirectX 10 |

## Current Build

- Seeded 16x256x16 chunk storage in a small four-by-four loaded region
- Hills, caves, lowland water, sandy areas, and sparse trees
- First-person mouse look, WASD movement, sprint, jump, gravity, and block collision
- Block targeting, breaking, and placement
- Nine-slot hotbar, basic health/hunger, and a day/night sky cycle
- Main/pause/settings screens, configurable render distance/mouse sensitivity/window mode, and local saves
- Generated 16x16 pixel-art-style texture atlas

This is an early game build. Crafting, full survival systems, broad biome variety, and a connected launcher are not complete. Performance must be measured on the actual laptop; no 30 or 100 FPS result is guaranteed.

## Run From Source

Install Rust with [rustup](https://rustup.rs), then from PowerShell in a clone of this repository:

```powershell
cd .\engine
cargo run --release
```

Prebuilt Windows test builds are published in GitHub Releases when a bundled test build is ready. A prebuilt executable does not require Rust to run.

## Controls

- Click in the game window to capture the mouse; Escape opens the pause menu or releases the mouse
- WASD to move, Shift to sprint, Space to jump
- Mouse to look; left-click to break and right-click to place
- Number keys 1-9 or the mouse wheel to select a hotbar slot

## Project Structure
```
engine/      Rust game and OpenGL-compatible renderer
engine/src/world.rs  Chunk storage, block IDs, and seeded terrain
engine/vendor/miniquad/  Legacy Windows OpenGL 2.1 compatibility patch
rustcraft-src/  Reference source only; not the current game engine
docs/GAME_PLAN.md  Product scope, hardware limits, and milestones
```

## Development Checks

From `engine/`:

```sh
cargo test --release
cargo check --target x86_64-pc-windows-gnu
cargo clippy --release
cargo build --release
```

## Reference

The `rustcraft-src/` directory contains the inspected EduRenesto/rustcraft reference. The current MaazX engine is separate and does not use its renderer.
