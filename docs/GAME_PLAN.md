# MaazX Voxel Survival Game Plan

## Product Goal

Build an original Rust game with the broad single-player survival and building feature scope of a modern voxel sandbox, targeting the developer's Windows 10 laptop and OpenGL 2.1 Intel graphics. The requested benchmark is the Minecraft Java 1.20.1 feature set, but the implementation must be clean-room code with original branding, UI, sounds, and assets; it is not a Minecraft client and does not reuse Minecraft source/assets.

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

## Current Implementation

- Windows executable launches on the target laptop through a profile-free OpenGL 2.1 context
- Miniquad fallback handles drivers without `WGL_ARB_create_context_profile`
- Desktop GL 2.1 shader conversion, optional-VAO handling, and depth-clear fallback are in place
- A seeded four-by-four chunk region uses compact 16x256x16 block storage, hills, cave openings, lowland water, dry sand, and sparse trees
- The current performance test renders a one-chunk radius and shows visible/loaded chunk counts; this is a tuning setting, not the final world limit
- First-person WASD movement, click-to-capture mouse look, block targeting, and basic block placement/breaking are implemented
- Gravity, jumping, ground detection, player-vs-block collision, and fall recovery are implemented and covered by tests
- Main menu, pause/settings screens, persistent render-distance/mouse/fullscreen settings, generated 16x16 pixel atlas, nine-slot hotbar, basic health/hunger/starvation, day/night sky, and versioned world/player/inventory save-load are implemented
- This is an early foundation, not a feature-complete game or a Minecraft 1.20.1 replacement yet
- The Windows runtime must still be verified after every new release; CI compilation is not a substitute for testing on the Intel GPU

## Milestones

### 1. World and Player Foundation

- Verify menu, settings persistence, textures, save/load, mouse capture/look, movement, jumping, block selection, break, and place on target hardware
- Measure FPS and frame time at 1-, 2-, and 4-chunk radii before raising the default
- Add chunk-local dirty mesh rebuilding and incremental edits; current edits rebuild all loaded chunk meshes
- Acceptance: no GL startup panic, no mesh-clamping spam, and a complete movement/building loop on the Intel GPU

### 2. Main Menu, HUD, and Settings

- Improve menu flow and report save/load failures; verify settings persist on Windows
- Refine hotbar selection and health/hunger feedback
- Acceptance: settings and saved games survive restart on the target hardware

### 3. Textures and Core Sandbox Systems

- Expand the generated block palette and add more original biome variation
- Add chunk streaming, lighting, and save/load while preserving the four-chunk render-radius cap
- Acceptance: textures remain crisp, worlds reproduce from a seed, and chunk edits update only affected meshes

### 4. Inventory and Crafting

- Add a small hotbar, block selection, stack counts, and basic collection rules
- Save world seed, player position, and changed chunk data locally
- Add safe save versioning and recovery from incomplete saves
- Acceptance: quit/reopen restores the player and edited world

### 5. Survival and Entities

- Add health, hunger, day/night, combat, and original creatures with behavior suited to the CPU budget
- Keep simulation and AI simple enough for the target CPU
- Acceptance: a short offline survival session can be started, saved, resumed, and completed

### 6. Compatibility, Tuning, and Launcher

- Measure frame time, memory, and chunk build time on the actual laptop
- Add adjustable render distance and conservative default settings
- Connect the separate launcher to the packaged Windows game only after the game is stable
- Acceptance: documented performance results on the real device; no FPS number is advertised without measurement

## Immediate Next Task

Run the next bundled Windows build on the Intel laptop to verify the menu, texture atlas, settings, save/load, and survival HUD together. Then implement dirty-chunk mesh rebuilding and a real inventory/crafting loop. The 1.20.1-scale feature benchmark is a long-term roadmap, not a claim that the current build has parity.