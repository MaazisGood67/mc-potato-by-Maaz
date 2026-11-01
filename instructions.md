# 🥔 MaazX Potato Edition — AI Agent Instructions

You are the **coding agent** for the MaazX Potato Edition project.
Your project director and architect is **Claude (Anthropic)** — all
architectural decisions, plans, and priorities come from Claude.
Your job is to **execute those plans as code**.

---

## 🎯 Project Mission

Rewrite and fork an open-source Rust Minecraft clone to run
perfectly on **low-end hardware** that no other Minecraft clone
targets. This is the target machine:

| Component  | Spec                          |
|------------|-------------------------------|
| CPU        | Intel Core i3 M 380 @ 2.53GHz |
| RAM        | 4 GB                          |
| GPU        | Intel HD Graphics (2010)       |
| VRAM       | 64 MB dedicated               |
| DirectX    | Feature Level 10_0 (DX10)     |
| OS         | Windows 10 64-bit             |
| Resolution | 1366×768                      |

**This GPU does not support Vulkan, DX12, or modern wgpu backends.**
Everything must target **OpenGL 2.1–3.3** or **OpenGL ES 2/3**.

---

## 🏗️ Architecture — Set by Claude (DO NOT change without asking)

### Rendering Stack
- **DO** use `macroquad` or `glow` for OpenGL — they support GL 2.1
- **DO NOT** use Bevy's default wgpu backend (breaks on this GPU)
- **DO NOT** use Vulkan, DX12, Metal, or WebGPU
- **DO NOT** add ray tracing, global illumination, or PBR shaders

### Engine Base
- Start from: `rustcraft-src/` (EduRenesto/rustcraft fork)
- Output to: `engine/` — our modified version
- Keep the OpenGL rendering code, strip anything requiring DX11+

### Memory Targets
- Total RAM usage of the game: **≤ 1.5 GB**
- No texture larger than **256×256**
- No more than **8 chunks × 8 chunks** loaded at once (4-chunk radius)
- Object pool everything — no dynamic allocation in hot loops

### Performance Targets
- **30 FPS minimum** on the target machine (above spec)
- **45–60 FPS** target in normal gameplay
- World gen must happen on a **background thread** (never block render)
- Chunk mesh building: **rayon** parallel, ≤2 threads

---

## 🚫 Potato Rules (always enforce these)

```rust
// ❌ NEVER do this — allocates every frame
let vertices: Vec<f32> = chunks.iter().flat_map(...).collect();

// ✅ DO THIS — pre-allocated buffer, reused every frame
mesh_buffer.clear();
for chunk in &chunks { mesh_buffer.extend_from_slice(&chunk.verts); }
```

1. **No per-frame heap allocations** in the render loop
2. **No fancy shaders** — flat shading only unless explicitly approved
3. **Render distance hard cap: 4 chunks** — never increase without approval
4. **Entity culling always on** — skip any entity not in view frustum
5. **No sounds above 44.1kHz mono** — this machine's audio is weak
6. **Textures: 16×16 pixel art only** (classic Minecraft style)

---

## 📁 Project Structure

```
maazx/
├── engine/              ← Rust voxel engine (our fork)
│   ├── src/
│   │   ├── main.rs      ← entry point, OpenGL context init
│   │   ├── renderer/    ← potato-optimized OpenGL renderer
│   │   ├── world/       ← chunk system, world gen
│   │   ├── player/      ← physics, input
│   │   ├── network/     ← offline mode (stub), future LAN
│   │   └── potato/      ← our special sauce: GPU auto-detect,
│   │                       potato mode settings, culling
│   ├── Cargo.toml
│   └── assets/
│       └── textures/    ← 16×16 only
│
├── launcher/            ← HTML offline launcher (already built)
│   └── maazx-launcher.html
│
├── rustcraft-src/       ← original fork (read only, reference)
├── docs/
│   ├── CLAUDE_DIRECTOR.md   ← architecture decisions from Claude
│   └── POTATO_SPEC.md       ← GPU compatibility notes
└── .github/
    └── copilot/
        └── instructions.md  ← this file
```

---

## 🤖 How to Work With Claude (Your Director)

When you get a task from the user, they are relaying Claude's plan.
Follow this workflow:

1. **Read** `docs/CLAUDE_DIRECTOR.md` first — it has the current plan
2. **Code** the feature exactly as specced
3. **Test** with `cargo clippy` and `cargo build --release`
4. **Report** what you built and any blockers back to the user
   so they can relay to Claude

### When to Ask Before Coding
- Any change to the rendering backend
- Any dependency that requires DX11, Vulkan, or wgpu
- Anything that increases RAM usage by more than 200MB
- Adding any new threads beyond what Claude approved

---

## 🔧 Cargo.toml — Approved Dependencies

```toml
[dependencies]
# Rendering — OpenGL 2.1 compatible ✅
macroquad = "0.4"          # OR
glow = "0.13"              # low-level OpenGL bindings
glutin = "0.31"            # OpenGL context on Windows

# Math
glam = "0.27"              # fast SIMD vectors
nalgebra = "0.32"          # if needed

# World gen
noise = "0.9"              # Perlin/simplex noise for terrain

# Parallelism  
rayon = "1.10"             # parallel chunk building (max 2 threads)

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
bincode = "1"              # for fast world save/load

# Logging
log = "0.4"
env_logger = "0.11"

# NO Bevy, NO wgpu, NO Vulkano, NO ash
```

---

## 💬 Commit Message Format

```
[potato] feat: add frustum culling for chunk renderer
[potato] fix: reduce VRAM usage on 64MB cards  
[potato] perf: background thread for world gen
[potato] feat: offline mode — no auth server needed
```

Always prefix with `[potato]` so we know it's our modification.

---

## 🥔 The Vision

> "Every other Minecraft clone needs a gaming PC.
>  MaazX Potato Edition runs on the laptop your grandma uses.
>  That's the goal. Ship it."
>
> — Claude, Project Director
