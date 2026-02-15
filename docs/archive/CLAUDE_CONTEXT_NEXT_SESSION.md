# Claude Code - Next Session Context

**Session Date:** 2026-02-05
**Status:** Game playable, needs model loading fixes
**Last Known State:** Flight works, drones visible but not loading 3D model, grass textured with procedural pattern

---

## QUICK START

```bash
cd /c/Users/Box/plane_game

# Build
cargo build --release 2>&1 | tail -20

# Copy assets and run
cp -r assets target/release/
target/release/plane_game.exe
```

---

## Current Known Issues

### 1. Drone 3D Model Not Loading ⚠️
- **Visible:** Red cubes instead of drone models
- **File:** `src/drone.rs` lines 30-32
- **Model path:** `models/drone.glb#Mesh0/Primitive0` (12MB file exists)
- **Root cause:** Bevy 0.15 SceneRoot mesh loading issue (known limitation)
- **Workaround:** Currently using `Mesh::from(Cuboid)` fallback
- **To fix:** Either confirm mesh path OR load as SceneRoot properly

### 2. JPG Textures Not Rendering ⚠️
- **Visible:** Ground shows procedural green instead of JPG texture
- **File:** `src/main.rs` lines ~587-612
- **Issue:** Bevy 0.15 bug [#15081](https://github.com/bevyengine/bevy/issues/15081)
  - Materials with async-loading textures never update bind group after load
  - Placeholder texture remains bound indefinitely
- **Current solution:** Procedural texture (working, but less detailed)
- **To fix:** Implement bevy_asset_loader OR wait for Bevy fix

### 3. Drones Easy to Outrun
- **Issue:** Drones spawn 1km away at 80 m/s, player starts with -100 m/s
- **File:** `src/main.rs` lines ~720-731
- **To fix:** Increase drone speed OR adjust initial player velocity

---

## What's Working Well ✅

### Physics System
- NaN detection in `FixedFirst` schedule (before physics)
- Catches and fixes invalid scale/rotation/velocity
- No crashes for 5+ minutes of flight

### Flight Controls
- W/S pitch, A/D roll, Q/E yaw
- Shift throttle, Ctrl brake, R rocket mode
- ESC respawn (was quit)
- F10 to quit

### Drone AI
- Pursuit behavior: `look_at()` + `slerp()` smooth rotation
- Speed boost (1.3x) when within 500m
- All NaN safety validated

### Ground
- Chunk system spawning correctly (no holes)
- Collision detection working
- Procedural texture tiling properly

---

## Architecture Overview

### Key Systems
| System | File | Purpose |
|--------|------|---------|
| `manage_chunks` | main.rs:750 | Load/unload terrain chunks |
| `move_drones` | drone.rs:59 | Drone pursuit AI |
| `arcade_flight_physics` | main.rs:2000 | Flight physics |
| `check_ground_collision` | main.rs:2742 | Crash detection |
| `detect_nan_early` | main.rs:1127 | NaN safety (FixedFirst) |
| `check_grass_texture_loaded` | main.rs:750 | Texture load monitoring |
| `drone_projectile_collision` | main.rs:2607 | Missile-drone hits |
| `drone_player_collision` | main.rs:2656 | Kamikaze collisions |

### Resources
| Resource | Purpose |
|----------|---------|
| `GroundMaterial` | Shared ground material |
| `GrassTextureLoading` | Texture load state tracking |
| `ChunkManager` | Track loaded chunks |

### Components
| Component | File | Purpose |
|-----------|------|---------|
| `Drone` | drone.rs:5 | Health + speed |
| `PlayerPlane` | main.rs:120 | Player marker |
| `KamikazeBehavior` | drone.rs:10 | Drone behavior |

---

## Testing Checklist

- [ ] Game launches without crash
- [ ] Can fly for 5+ minutes without NaN errors
- [ ] Drones spawn and move toward player
- [ ] Can shoot drones (Space)
- [ ] Explosions appear on drone destruction
- [ ] Ground renders without holes
- [ ] FPS stays 60+ during normal flight

---

## Code Patterns Used

### NaN Safety Pattern
```rust
if !value.is_finite() {
    eprintln!("⚠️ Invalid value! {:?}", value);
    value = safe_default;
}
```

### Drone Query Pattern
```rust
let Ok(player_transform) = player_query.get_single() else { return };
for (entity, mut transform, drone) in &mut drone_query {
    // pursuit logic
}
```

### Procedural Texture Pattern
```rust
let mut texture_data = Vec::new();
for _y in 0..size {
    for _x in 0..size {
        texture_data.extend_from_slice(&[r, g, b, 255]);
    }
}
let image = Image::new(..., texture_data, ...);
```

---

## Console Messages to Monitor

| Message | Meaning |
|---------|---------|
| `🚨 EARLY:` | NaN detected, being fixed |
| `⚠️ Drone:` | Drone validation warning |
| `🌿 STARTUP:` | Texture/material initialization |
| `DEBUG: Drone pursuing` | AI working correctly |
| `💥 KAMIKAZE HIT` | Drone reached player |
| `🌍 CHUNK SPAWN` | New terrain chunk loaded |

---

## Important File Locations

```
src/main.rs              - 3000+ lines, all core systems
src/drone.rs            - Drone component + pursuit AI
assets/models/drone.glb - 12MB drone model (NOT rendering)
assets/textures/grass/  - Procedural texture (JPG async loading broken)
CLAUDE.md              - Project documentation
SESSION_HANDOFF.md     - Previous session state
```

---

## Next Priority Tasks

1. **Fix drone model loading** (HIGH IMPACT)
   - Check if `drone.glb#Mesh0/Primitive0` path is correct
   - May need to use SceneRoot instead of direct Mesh
   - Or confirm path and debug async loading

2. **Fix JPG texture async loading** (MEDIUM IMPACT)
   - Implement bevy_asset_loader workaround
   - OR wait for Bevy 0.15 fix
   - OR accept procedural texture as permanent solution

3. **Improve drone difficulty** (LOW IMPACT)
   - Increase drone speed to 120+ m/s
   - Spawn closer initially
   - Add multiple waves

4. **Add speed UI** (NICE TO HAVE)
   - Display velocity on screen
   - Show altitude
   - Show damage indicator

---

## Debugging Tips

### Check System Order
- NaN detection runs in `FixedFirst` (before physics)
- Physics runs in `FixedUpdate` (automatic)
- Game logic runs in `Update`

### Asset Loading Debug
```rust
eprintln!("Load state: {:?}", asset_server.get_load_state(&handle));
```

### Drone Visibility Debug
- Add `println!` in `move_drones` distance logging
- Check if drones are spawning at correct positions
- Verify pursuit rotation is working

### Texture Debug
```rust
eprintln!("🌿 TEXTURE LOADED!"); // Should print once
```

---

## One-Line Fixes to Try

**Drone model not rendering:**
- Try: Load as SceneRoot instead of direct Mesh
- Try: Check if `#Mesh0/Primitive0` selector is correct (might be `#Scene0`)
- Try: Print load state with `asset_server.get_load_state()`

**JPG texture not showing:**
- Try: Reduce JPG size (2MB → 1MB)
- Try: Use TGA format instead (uncompressed)
- Try: Implement `bevy_asset_loader` crate

**Drones too slow to catch:**
- Change: `speed: 80.0` → `speed: 120.0` in drone.rs:44
- Change: Spawn distance from 1km to 500m

---

## Session History

- **2026-02-05:** Physics crash fixed, ESC respawn, drones added, grass procedural
- **Previous:** Chunk system, flight physics, rocket mode, combat system

---

## Resources

- [Bevy 0.15 Docs](https://docs.rs/bevy/0.15/)
- [Issue #15081 - Async texture loading](https://github.com/bevyengine/bevy/issues/15081)
- [bevy_asset_loader crate](https://crates.io/crates/bevy_asset_loader)
- [CLAUDE.md](./CLAUDE.md) - Full project documentation
