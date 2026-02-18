# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
# Release build (required for normal play — debug is too slow)
cargo build --release
target/release/plane_game.exe

# Forced rebuild when Cargo thinks nothing changed
touch src/main.rs && cargo build --release

# After ANY code change, copy assets to build output
cp -r assets target/release/

# Compile check only (fast)
cargo check
```

Build time: ~2 minutes full, <5s incremental. If it completes in <1s, the cache was hit and changes weren't compiled — use `touch src/main.rs`.

## Architecture

**Bevy 0.15 ECS flight simulator** — Avian3D physics, infinite procedural terrain, arcade flight model.

### Module Structure
- `src/main.rs` (~4000 lines) — core systems: world/chunk generation, flight physics, lighting, player spawn
- `src/drone.rs` — drone AI (lead pursuit, swarm flocking, kamikaze), spawning, missile collision
- `src/ui.rs` — HUD: altitude, speed, fuel, velocity readout
- `src/assets.rs` — `bevy_asset_loader` `GameAssets` struct; defines `GameState` (Loading → Playing)
- `src/procedural_textures.rs` — generates grass texture at startup to bypass Bevy async bind group bug (#15081)

### Game States
`GameState` gates all gameplay systems:
- **Loading** — asset loading; gameplay systems inactive
- **Playing** — normal flight
- **GameOver** — crash; physics frozen

All gameplay systems must use `.run_if(in_state(GameState::Playing))` or `OnEnter(GameState::Playing)`.

### System Scheduling (order matters)
1. `FixedFirst` — `detect_nan_early`: validates transform scale/velocity before Avian3D runs
2. `Update` — input, flight physics, drone AI, chunk management, particles
3. `PostUpdate` — camera, LOD updates (must be PostUpdate so newly spawned entities are visible)
4. `FixedPostUpdate` — Avian3D physics resolution

### Chunk World
- Chunk size: 1000m × 1000m
- Load radius: 8 chunks (8km) · Unload radius: 12km
- `ChunkCoordinate::from_world_pos(pos)`: `(pos / 1000.0).floor() as i32`
- Per-chunk: 5–10 trees, 15% village, 15% drone patrol (deterministic hash — same chunk = same spawn)

### Player Entity Hierarchy
```
PlayerPlane (RigidBody, physics, PlayerInput, FlightState)
  └─ ModelContainer (visual rotation independent of physics)
       └─ Mesh/SceneRoot (F-16 .glb)
```
Physics operates on the **parent** global transform. Children use local coordinates.

### Flight Physics
- Thrust along `-transform.local_z()` (Bevy forward is -Z)
- Bank-to-turn: roll angle creates yaw via `TURN_FACTOR`
- Rocket mode (R key): 8× thrust, reaches 25 km in ~12 s
- All tuning constants in the `CONSTANTS` section of `main.rs`

---

## Critical Technical Constraints

### Lighting — Physical Values Required
Bevy 0.15 uses physical camera exposure (EV100). Values must match:
```rust
DirectionalLight { illuminance: 100_000.0, .. }  // lux::DIRECT_SUNLIGHT
AmbientLight { brightness: 2500.0, color: Color::srgb(0.8, 0.85, 1.0) }  // sky fill
// On the camera entity:
Exposure { ev100: 15.0 }          // Exposure::SUNLIGHT — must match illuminance scale
EnvironmentMapLight {              // REQUIRED for metallic specular reflections
    diffuse_map:  asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
    specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
    intensity: 900.0,
    ..default()
}
```
Without `EnvironmentMapLight`, metallic surfaces (F-16 fuselage) appear flat/black from most angles. Without `AmbientLight`, shadow areas are pitch black. Without matching `Exposure`, the scene is over- or under-exposed regardless of illuminance values. `Exposure` is in `bevy::render::camera::Exposure` (not in prelude — must be imported explicitly).

### NaN Crash Protection
`detect_nan_early` runs in `FixedFirst` (BEFORE physics). Never move it to `Update`. It:
- Resets `transform.scale` if any dimension ≤ 0 or NaN
- Resets position/velocity if NaN
- Hard-resets if Y > 100,000 or Y < -1,000

### Asset Loading Patterns
```rust
// Simple geometry — reliable in Bevy 0.15
Mesh3d(asset_server.load("models/tree.glb#Mesh0/Primitive0"))
MeshMaterial3d(materials.add(StandardMaterial { .. }))

// Complex scenes with hierarchy
SceneRoot(asset_server.load("models/building.glb#Scene0"))
```
`SceneRoot` with simple `.glb` files (no scene hierarchy) can silently render nothing — use the `Mesh3d` pattern instead, and add a bright-colored `Cuboid` fallback child to verify spawn before debugging mesh paths.

### Rendering Requirements
- Camera far clip **must** exceed fog distance: `Projection::Perspective { far: 100_000.0 }`
- `Msaa::Off` required — HDR + MSAA causes black screen in Bevy 0.15
- Ground material uses `unlit: true` — shadows/lighting don't affect the ground plane (intentional)
- Sky sphere and horizon disk follow the camera each frame (`update_sky_sphere`, `update_horizon_disk`, `update_sun_position`)
- Add `NoFrustumCulling` to large scene entities if they disappear at odd camera angles
- `ktx2` and `zstd` Cargo features required to load `.ktx2` environment maps

### Audio
- **OGG only.** Files with embedded cover art crash rodio at runtime.
- Strip metadata before use: `ffmpeg -i input.ogg -vn -c:a copy output.ogg`

---

## Asset Locations

**In-repo:** `assets/`
- `assets/fantasy_town/` — Kenney medieval buildings/trees (167 .glb files)
- `assets/models/` — drone.glb, sun.glb, F-16 model
- `assets/textures/` — HDR sky (`citrus_orchard_road_puresky_4k.hdr`), grass PNG
- `assets/environment_maps/` — IBL cubemaps for specular (pisa_diffuse/specular `.ktx2`)
- `assets/sounds/` — OGG audio (engine, wind, crash, weapons)

**External asset drive:** `F:\Plane_Game Assets\`
- `kenney_fantasy-town-kit_2.0/` — medieval buildings (source for `assets/fantasy_town/`)
- `future_fighterjet/` — F-16 model source
- `Grass004_1K-PNG/`, `Grass004_4K-PNG/` — high-res grass textures
- `Sonniss.com - GDC 2019 - Game Audio Bundle/` — sound effects source

---

## Common Failure Modes

| Symptom | Cause | Fix |
|---------|-------|-----|
| Build in <1s, changes ignored | Cargo cache hit | `touch src/main.rs` |
| `assets/` not found at runtime | Assets not copied to build output | `cp -r assets target/release/` |
| Model spawns but invisible | Zero ambient light or SceneRoot failure | Check `AmbientLight`; try `Mesh3d` with fallback `Cuboid` |
| Ground has holes/gaps | Load radius too small | Keep `LOAD_RADIUS` at 8km |
| Physics crash (NaN panic) | Invalid transform before Avian3D | Verify `detect_nan_early` is in `FixedFirst` |
| Trees visible then disappear | LOD system in Update schedule | Move LOD to `PostUpdate` |
| OGG audio crash on load | Cover art metadata in file | Strip with ffmpeg |
| F-16 looks flat/dark (no specular) | Missing `EnvironmentMapLight` on camera | Add IBL cubemap to camera entity |
| Scene over/under-exposed | `Exposure` EV100 mismatched to `illuminance` | Use `ev100: 15.0` with `illuminance: 100_000.0` |
