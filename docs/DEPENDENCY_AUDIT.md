# Dependency Audit

**Target:** OpenGL 2.1 on 2010-era Intel HD Graphics
**Scope:** Direct dependencies declared in `rustcraft-src/Cargo.toml` (six crates). A crate's own compatibility is separate from the renderer's requirements.

| Crate | Version | Role | OpenGL 2.1 / Intel HD 2010 | Can we keep it? |
|---|---:|---|---|---|
| `glutin` | `0.19.0` | Window and OpenGL context creation | The crate does not inherently require a modern OpenGL version. Compatibility depends on creating a legacy-compatible context and the installed driver supporting it. | **Yes, conditionally.** Request/use an OpenGL 2.1 context and verify creation on the target driver. |
| `gl` | `0.11.0` | OpenGL function bindings and constants | The bindings do not impose a minimum version, but functions are only usable when exposed by the context or extensions. The current source uses vertex arrays and framebuffer objects, which require OpenGL 3.0 or their extensions. | **Yes, conditionally.** Gate newer calls and provide extension checks or GL 2.1-compatible fallbacks. |
| `cgmath` | `0.16.1` | CPU-side vector and matrix math | No OpenGL dependency or GPU requirement. | **Yes.** |
| `stb_image` | `0.2.2` | CPU-side image decoding | No OpenGL dependency or GPU requirement. | **Yes.** |
| `noise` | `0.5.1` | CPU-side procedural noise | No OpenGL dependency or GPU requirement. | **Yes.** |
| `rayon` | `1.3` | CPU parallelism | No OpenGL dependency or GPU requirement. Its thread count affects CPU/RAM use, not graphics API compatibility. | **Yes.** Keep the project's two-thread limit. |

## Important renderer blocker

The dependency list itself contains no Vulkan, wgpu, or DirectX 11 crate. However, the checked-in renderer is **not currently compatible with an OpenGL 2.1-only context**:

- All six shaders under `rustcraft-src/res/shaders/` declare `#version 430 core`, requiring OpenGL 4.3 / GLSL 4.30. A 2.1 driver cannot compile them as written.
- `rustcraft-src/src/shader.rs` accepts compute and geometry shader types. Compute shaders require OpenGL 4.3; geometry shaders require OpenGL 3.2. Merely having those constants in the `gl` crate does not make the features available on a 2.1 context.
- `rustcraft-src/src/vertex_buffer.rs` uses vertex array objects, and `rustcraft-src/src/render_target.rs` uses framebuffer objects. These require OpenGL 3.0 core or the corresponding extensions; a 2.1 implementation must be checked for those extensions or the code needs fallbacks.

**Conclusion:** Keep the six crates from a dependency standpoint, but do not treat that as proof that the existing engine runs on the target GPU. To meet the OpenGL 2.1 target, rewrite shaders for GLSL 1.20 and guard or replace APIs newer than 2.1, then verify against the actual Intel driver. This audit covers direct `Cargo.toml` dependencies, not every transitive crate in `Cargo.lock`.
