# Prompt: Make the plane game environment look more real

Use this prompt when asking Claude (or another AI) to improve the environment in `src/main.rs` so the game looks more realistic. The game is a Bevy 0.15 + avian3d flight game with a player jet, meteors, objectives, and turrets.

---

## Context

- **Tech:** Bevy 0.15, avian3d physics, Rust. Scene is set up in `setup_scene()` and related spawn functions.
- **Current state:** Flat lime-green ground plane, solid dark blue-grey clear color (no sky), directional light with shadows, orange-red exhaust on the plane. There is a high-altitude debug grid and cardinal direction markers (red/blue spheres). A blue sphere and other objects may appear (objectives, meteors).
- **Goal:** Improve the environment so it feels like a real sky/ground flight scene—better sky, ground, lighting, and optional props—without breaking physics or existing gameplay.

---

## What to add or change

### 1. Sky and atmosphere

- Replace the solid `ClearColor` with a more realistic sky:
  - **Option A:** Use a skybox (cubemap or panoramic texture) for a day or dusk sky. If Bevy 0.15 has a skybox or background plugin, use it; otherwise use a large inverted sphere/dome with a sky texture or gradient.
  - **Option B:** If no skybox is available, use a vertical gradient clear color or a large dome mesh with a gradient material (lighter blue near horizon, darker blue up) and no back-face culling so the inside looks like sky.
- Optionally add a subtle fog (distance-based fade) so far terrain and objects fade for depth. Use Bevy’s fog if available, or a simple distance-based alpha on distant objects.
- Keep performance in mind: avoid expensive per-pixel work if the scene is large.

### 2. Ground

- Replace the flat lime-green plane with something that reads as real ground:
  - **Texture:** Use a tiled ground texture (e.g. grass, dirt, or tarmac) via `StandardMaterial::base_color_texture` and scale UVs so the repeat looks good at flight scale.
  - **Color:** If no texture is added yet, at least move from bright lime to a more natural green/brown/grey and keep it a single plane so collision stays simple.
- Do not change the ground collider size or position without adjusting `check_ground_collision` (e.g. `GROUND_LEVEL`) so the plane still lands correctly.
- Optional: add a simple “runway” strip (different material or color) in the middle of the map for visual reference.

### 3. Lighting

- Keep `DirectionalLight` with `shadows_enabled: true`.
- Tune rotation so shadows fall in a consistent direction (e.g. sun from one side and slightly above) and match the sky (e.g. warmer light for dusk).
- Optionally add a second, weaker light (e.g. fill or ambient) so the underside of the plane and ground aren’t pitch black, or slightly increase ambient in the environment.
- Ensure light color and intensity suit the new sky (e.g. orange-ish for sunset, neutral for overcast).

### 4. Debug and optional cleanup

- Consider making the debug grid (high transparent blue plane) and cardinal markers (red/blue spheres) optional: e.g. spawn them only in a debug mode or behind a compile-time flag, so “real” mode has a clean sky and no floating markers.
- If there are small brown/dark blobs in the sky (e.g. meteors or debris), either improve their materials and scale so they read as intentional (rocks, clouds) or move them so they don’t look like artifacts.

### 5. Optional environment props

- Only if it fits the current design: add a few simple static props (e.g. trees as rotated cones or cubes, or a couple of box buildings) near the ground for scale and variety. Use simple meshes and materials; avoid changing physics or colliders unless needed.
- Keep the play area clear so the plane, meteors, objectives, and turrets remain the focus.

### 6. Camera and exhaust

- Do not change the camera follow logic or the exhaust spawn position/offset; those are already tuned. Only adjust environment (sky, ground, lighting, props).

---

## Constraints

- Keep `setup_scene()` and spawn logic in `main.rs` (or refactor into a separate module only if it stays clear and easy to find).
- Do not remove or rename existing components/systems (e.g. `PlayerPlane`, `check_ground_collision`, collision queries). Only add or adjust visuals and optional debug toggles.
- Preserve ground collision: `GROUND_LEVEL` and ground entity position/collider must stay consistent so the plane still hits “ground” and respawns correctly.
- Use only Bevy 0.15 and existing dependencies (e.g. no new crates unless the user agrees). Prefer built-in nodes (meshes, materials, lights, fog) over custom shaders if possible.

---

## Summary checklist for Claude

- [ ] Sky: gradient, skybox, or dome so the background is no longer a flat dark blue.
- [ ] Ground: texture and/or more natural color; keep collider and height consistent.
- [ ] Lighting: directional light direction/color tuned to sky; shadows on.
- [ ] Optional: fog, debug-only grid/markers, runway strip, a few simple ground props.
- [ ] No changes to camera follow or exhaust position; no breaking changes to physics or gameplay.

Use this prompt when asking Claude to “add to the environment to make this look more real” so it has clear, actionable tasks and constraints.
