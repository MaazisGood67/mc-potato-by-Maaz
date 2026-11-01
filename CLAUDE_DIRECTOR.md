# 📋 MaazX — Director's Plan
**Director:** Claude (Anthropic)  
**Coding Agent:** GPT-6 Sol / Luna (GitHub Copilot in VS Code)  
**Developer:** Maaz

---

## Current Phase: PHASE 1 — Fork & Strip

### ✅ Completed
- [x] HTML Launcher UI (MaazX Potato Edition)
- [x] Codespace dev environment
- [x] GPT-6 Sol agent instructions

### 🔨 Active Tasks for Agent

#### TASK 001 — Audit the base engine
**File:** `rustcraft-src/`  
**Agent instruction:**
```
Read through rustcraft-src/src/main.rs and rustcraft-src/Cargo.toml.
List every dependency that requires OpenGL 3.3+, Vulkan, or DX11.
Output a markdown table: Dependency | GL version required | Can we keep it?
Save the result to docs/DEPENDENCY_AUDIT.md
```

#### TASK 002 — Create engine/ scaffold
**Agent instruction:**
```
Create the engine/ directory with this structure:
engine/src/main.rs          — empty main with println!("MaazX starting...")
engine/src/renderer/mod.rs  — empty mod
engine/src/world/mod.rs     — empty mod
engine/src/player/mod.rs    — empty mod
engine/src/potato/mod.rs    — empty mod

engine/Cargo.toml with ONLY these deps:
  macroquad = "0.4"
  glam = "0.27"
  noise = "0.9"
  rayon = "1.10"
  log = "0.4"
  env_logger = "0.11"

Run: cargo build --release in engine/
Report if it compiles.
```

#### TASK 003 — OpenGL window test
**Agent instruction:**
```
In engine/src/main.rs write a macroquad app that:
1. Opens a 1366x768 window titled "MaazX Potato Edition"
2. Clears to a dark green color (#0D1F0E)
3. Draws "MaazX Potato Edition" text in the center in white
4. Prints to console: "GPU: " + the OpenGL renderer string
5. Exits cleanly on ESC

This tests that our OpenGL context works.
Run it and report what GPU string it prints.
```

#### TASK 004 — Port chunk data structure
**Agent instruction:**
```
From rustcraft-src/src/ copy the chunk/world data structures 
(NOT the renderer) into engine/src/world/chunk.rs

A chunk is 16x256x16 blocks.
Use a flat array: [u8; 16*256*16] for block IDs
Block ID 0 = air, 1 = stone, 2 = grass, 3 = dirt, 4 = wood

Add a ChunkPos struct: { x: i32, z: i32 }
Add a World struct that holds a HashMap<ChunkPos, Chunk>

No rendering code here — data only.
```

---

## Architecture Decisions Log

### Decision 001 — Rendering Backend
**Date:** Project start  
**Decision:** macroquad over Bevy/wgpu  
**Reason:** Bevy 0.13+ dropped DX11 and some Intel HD chips fall back
to CPU rendering. macroquad uses miniquad which targets OpenGL 2.1.
Our GPU (Intel HD Core i3, 2010) only supports DX10/GL3.0.  
**Status:** FINAL — do not revisit

### Decision 002 — No Vulkan
**Date:** Project start  
**Decision:** Zero Vulkan in this project  
**Reason:** Intel HD Graphics (2010) does not have a Vulkan driver.
Any crate that pulls in Vulkan as a dependency is banned.  
**Status:** FINAL

### Decision 003 — Chunk size
**Date:** Project start  
**Decision:** 16×256×16 chunks, max 8×8 loaded (4 chunk radius)  
**Reason:** 64MB VRAM means we cannot buffer many meshes.
Each chunk mesh is ~2MB on GPU. 64 chunks × 2MB = 128MB — too much.
32 chunks × 2MB = 64MB — fits, barely.  
**Status:** FINAL, may reduce to 6×6 if VRAM issues arise

### Decision 004 — Offline Mode
**Date:** Project start  
**Decision:** No authentication server, no Microsoft login  
**Reason:** This is an independent open-source Rust rewrite,
not a Minecraft client. It does not connect to Mojang servers.  
**Status:** FINAL

---

## What Makes MaazX Different

| Feature | Every Other Rust MC Clone | MaazX Potato Edition |
|---------|--------------------------|---------------------|
| GPU req | Vulkan / DX12 / wgpu | OpenGL 2.1 ✅ |
| RAM req | 8GB+ | 4GB ✅ |
| VRAM   | 512MB+ | 64MB ✅ |
| Auth   | Microsoft login | Offline ✅ |
| Target | Gaming PC | Potato PC ✅ |
| Year   | 2024+ hardware | 2010+ hardware ✅ |

---

## Next Message to Agent

When Maaz opens Codespace and types to the Copilot agent,
paste this as the first message:

```
You are working on MaazX Potato Edition.
Read .github/copilot/instructions.md first, then docs/CLAUDE_DIRECTOR.md.
Start with TASK 001: audit the dependencies in rustcraft-src/Cargo.toml.
List every crate and whether it works on OpenGL 2.1 / Intel HD Graphics 2010.
Save output to docs/DEPENDENCY_AUDIT.md
```
