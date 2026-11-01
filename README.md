# 🥔 MaazX — Minecraft Potato Edition

> A Rust voxel game that actually runs on your grandma's laptop.

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/YOUR_USERNAME/maazx)

---

## What is this?

MaazX is a fork and heavy modification of open-source Rust Minecraft clones,
**specifically optimized for low-end hardware** that every other project ignores.

### Target Machine
| Spec | Value |
|------|-------|
| CPU | Intel Core i3 @ 2.53GHz |
| RAM | 4 GB |
| GPU | Intel HD Graphics (2010) |
| VRAM | 64 MB |
| API | OpenGL 2.1 / DirectX 10 |

## Why this is unique

Every Rust Minecraft clone requires Vulkan, DX12, or wgpu — none of which
work on 2010-era integrated GPUs. MaazX uses **macroquad** with an OpenGL 2.1
backend, meaning it runs on hardware that's over 15 years old.

## Features
- 🥔 **Potato Mode** — auto-detects weak GPU and adjusts settings
- 🔌 **100% Offline** — no Microsoft account, no login server
- ⚡ **Optimized** — 4-chunk render distance, aggressive culling
- 🚀 **Targets 30–60 FPS** on Intel HD Graphics
- 🦀 **Written in Rust** — fast, safe, no JVM overhead

## Getting Started (GitHub Codespaces)

1. Click **"Open in GitHub Codespaces"** above
2. Wait ~2 minutes for setup
3. Open Copilot Chat (sidebar) — select **GPT-6 Sol** model
4. Read `docs/CLAUDE_DIRECTOR.md` to see the full plan
5. Paste the first task prompt from the bottom of that file

## Project Structure
```
engine/      ← Rust game engine (OpenGL 2.1)
launcher/    ← HTML offline launcher  
docs/        ← Architecture decisions by Claude
```

## Architecture

- **Rendering:** macroquad (OpenGL 2.1 — no Vulkan, no wgpu)
- **World:** 16×256×16 chunks, 4-chunk radius max
- **Threads:** rayon for chunk gen (max 2 threads)
- **Auth:** none — fully offline
- **Director:** Claude (Anthropic)
- **Coding Agent:** GPT-6 Sol (GitHub Copilot)

## Credits

- Base engine forked from [EduRenesto/rustcraft](https://github.com/EduRenesto/rustcraft)
- Architecture & direction: Claude (Anthropic)  
- Built by: Maaz (monuson)
