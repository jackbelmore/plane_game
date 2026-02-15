# Session Handoff - 2026-02-05

## Current Issues to Fix

### 1. Grass Texture Not Rendering
**Status:** Implemented load-wait system but texture still not showing
**What we tried:**
- Procedural texture (worked but reverted)
- JPG async loading with `check_grass_texture_loaded` system
- Load state monitoring with `GrassTextureLoading` resource

**Root cause:** Bevy 0.15 bug - [Issue #15081](https://github.com/bevyengine/bevy/issues/15081) - materials with slow-loading textures never update bind group

**Current code location:**
- `setup_scene()` creates material with texture handle
- `check_grass_texture_loaded()` monitors load state and updates material when ready
- Resource: `GrassTextureLoading` tracks texture handle and loaded state

**To fix:** Either revert to procedural texture OR implement proper asset loading with bevy_asset_loader

### 2. Drones Rendering as Yellow Spheres
**Status:** Drone model not loading, showing fallback or explosion mesh
**What we know:**
- Drones spawn at correct positions (1km+ away now)
- Model path: `models/drone.glb#Mesh0/Primitive0`
- Yellow spheres = likely explosion effects OR model load failure

**Check:**
- Does `assets/models/drone.glb` exist?
- Is the mesh path correct?
- Are explosions being spawned incorrectly?

**Current drone spawn:** `src/drone.rs` line 32
```rust
let drone_handle: Handle<Mesh> = asset_server.load("models/drone.glb#Mesh0/Primitive0");
```

---

## Recent Changes (This Session)

### Fixed - Physics Crash
- Added `detect_nan_early` system in `FixedFirst` schedule (before physics)
- System now FIXES invalid values, not just logs them
- Added scale validation (zero/negative scale causes AABB crash)
- Debris now has `Collider::sphere(0.5)`

### Fixed - ESC Key
- ESC now respawns player (was quit)
- F10 to quit game
- Code in `handle_quit()` and `handle_restart()`

### Fixed - Drone Spawn Distance
- Drones now spawn 1km+ away (was 200m - instant death)
- Speed reduced to 80 m/s (was 150)
- Positions in `setup_scene()` lines ~720-731

### Implemented - Drone Pursuit AI
- `move_drones()` in `src/drone.rs` now uses `look_at()` + `slerp()` for smooth pursuit
- Speed boost (1.3x) when within 500m
- All NaN safety checks included

---

## Key Files

| File | Purpose |
|------|---------|
| src/main.rs | Main game (~3000 lines) |
| src/drone.rs | Drone component + pursuit AI |
| CLAUDE.md | Project documentation |
| assets/models/drone.glb | Drone 3D model |
| assets/textures/grass/ | Grass textures (JPG/PNG) |

---

## Controls

- **WASD** - Pitch/Roll
- **QE** - Yaw
- **Shift** - Throttle
- **Ctrl** - Brake
- **R** - Rocket mode toggle
- **Space** - Fire missiles
- **ESC** - Respawn
- **F10** - Quit

---

## Build & Run

```bash
cd /c/Users/Box/plane_game
cargo build --release
cp -r assets target/release/
target/release/plane_game.exe
```

---

## Next Steps (Priority Order)

1. **Fix drone rendering** - Check if model loads, add fallback cube if not
2. **Fix grass texture** - Either use procedural OR implement proper load waiting
3. **Test combat loop** - Drones chase, player shoots, explosions work
4. **Add speed UI** - Show velocity on screen

---

## Console Messages to Watch

- `🌿 TEXTURE LOADED!` - Grass texture ready
- `DEBUG: Drone pursuing player at dist: XXXm` - Drone AI working
- `🚨 EARLY:` - NaN detection caught something
- `⚠️ Drone` - Drone validation issue
