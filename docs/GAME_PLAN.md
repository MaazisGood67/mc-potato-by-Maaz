# MaazX Voxel Survival Game Plan

## Product Goal

Build an original, offline, single-player Rust voxel survival game for the developer's Windows 10 laptop. The game can share broad genre conventions such as block terrain, first-person movement, collecting, building, crafting, and survival, but it will use original code, names, UI, sounds, and textures. It will not reproduce Minecraft 1.20.1 source, assets, branding, or exact presentation.

## Target Hardware and Engineering Limits

- Intel Core i3 M 380, 4 GB RAM, Intel HD Graphics, Windows 10 x64
- Baseline graphics target: desktop OpenGL 2.1 / GLSL 1.20
- Native window target: 1366x768
- Keep render distance at or below four 16x256x16 chunks
- No Vulkan, DX12, wgpu, ray tracing, or modern-only shader features
- Generate worlds away from the render loop; limit background work to two threads
- Rebuild only changed chunk meshes and render exposed faces only
- Keep textures at 16x16; keep per-frame allocations out of gameplay/render loops
- Treat 30 FPS as a measured minimum goal and 100 FPS as an unverified stretch goal, not a promise

## Current Prototype

- Windows executable launches on the target laptop through a profile-free OpenGL 2.1 context
- Miniquad fallback handles drivers without `WGL_ARB_create_context_profile`
- Desktop GL 2.1 shader conversion, optional-VAO handling, and depth-clear fallback are in place
- A 64x64 flat grass-block world is meshed in draw-call-safe batches with visible side faces
- First-person WASD movement, click-to-capture mouse look, block targeting, and basic block placement/breaking are implemented
- Gravity, jumping, ground detection, player-vs-block collision, and fall recovery are implemented and covered by tests
- The Windows runtime must still be verified after every new release; CI compilation is not a substitute for testing on the Intel GPU

## Milestones

### 1. Stable Playable Core

- Verify mouse capture/look, movement, block selection, break, and place on target hardware
- Verify mouse capture/look, movement, jumping, block selection, break, and place on target hardware
- Acceptance: no GL startup panic, no mesh-clamping spam, controls work, and the player cannot pass through blocks

### 2. World Data and Terrain

- Replace the prototype `HashSet` with compact chunk block storage and stable block IDs
- Add deterministic seeded heightmap terrain, then caves and simple tree structures
- Use dirty-chunk mesh rebuilds, exposed-face culling, and bounded chunk loading
- Acceptance: same seed recreates the same world, edits update only affected chunks, memory stays within budget

### 3. Building and Persistence

- Add a small hotbar, block selection, stack counts, and basic collection rules
- Save world seed, player position, and changed chunk data locally
- Add safe save versioning and recovery from incomplete saves
- Acceptance: quit/reopen restores the player and edited world

### 4. Survival Loop

- Add original crafting recipes, health, hunger, day/night, and a small set of original creatures
- Keep simulation and AI simple enough for the target CPU
- Acceptance: a short offline survival session can be started, saved, resumed, and completed

### 5. Potato Tuning and Launcher

- Measure frame time, memory, and chunk build time on the actual laptop
- Add adjustable render distance and conservative default settings
- Connect the separate launcher to the packaged Windows game only after the game is stable
- Acceptance: documented performance results on the real device; no FPS number is advertised without measurement

## Immediate Next Task

Run the new Windows build on the Intel laptop to verify mouse input, collision, jump, and block edits together. Then replace the prototype flat-world `HashSet` with compact chunk block storage and add deterministic seeded heightmap terrain.