# Copilot Instructions for Plane Game

## Build & Run

```bash
# Debug build (faster compile, slower runtime)
cargo run

# Release build (slower compile, optimized runtime - RECOMMENDED)
cargo run --release

# Just compile check
cargo check

# Copy assets to release build after code changes
cp -r assets target/release/
```

**Build Optimization**: Custom Cargo.toml profile with `opt-level = 1` for dev, dependencies at `opt-level = 3`. Release uses `lto = "thin"` with 16 codegen units for parallel compilation.

**Performance Note**: Release builds are essential for stable physics at 60+ FPS. Dev builds use reasonable optimizations for faster iteration without terrible runtime performance.

## Architecture Overview

**Bevy 0.15 ECS-based F-16 flight simulator** with Avian3D physics and infinite procedurally-generated terrain. Code is organized across multiple files:

### Module Structure
- **`src/main.rs`** (2500+ lines) - Core flight systems, chunk world generation, player spawn
- **`src/drone.rs`** - Drone AI with lead pursuit, swarm behavior, collision detection
- **`src/ui.rs`** - HUD display (altitude, speed, fuel, velocity readout)
- **`src/assets.rs`** - bevy_asset_loader setup with GameState machine (Loading → Playing)
- **`src/procedural_textures.rs`** - Runtime texture generation for grass, preventing async bind group issues

### Game States
Game uses a `GameState` enum to gate systems:
- **Loading**: Asset loading in progress, gameplay systems inactive
- **Playing**: Normal flight, all systems active
- **GameOver**: Crash occurred, physics frozen

All gameplay systems must be in the `OnEnter(GameState::Playing)` or `Update` with `run_if(in_state(GameState::Playing))` to prevent race conditions during asset loading.

### World Generation
- **Chunk system**: 1000m × 1000m tiles, load radius 8km, unload radius 12km
- **Chunk coordinates**: `ChunkCoordinate::from_world_pos(pos) = (pos.x / 1000).floor() as i32`
- **Per-chunk spawning**: Trees (5-10), villages (15% chance), drones (15% chance with deterministic hash)
- **Terrain**: Perlin noise multi-biome heightmap via `get_terrain_height()`:
  - **Flatlands** (s < 0.35): Gentle rolling plains, 30m amplitude
  - **Canyons** (0.35 < s < 0.65): Ridged valleys, 500m depth
  - **Mountains** (s > 0.65): Massive peaks up to 1.2km+ with craggy detail

### Drone AI (Phase 3)
**Tactical Speed Zones** - distance-based behavior adaptation:
- **>5km**: Warp pursuit (8.0x speed, ~1040 m/s) - instant catch-up
- **3-5km**: Sprint (5.0x speed, ~650 m/s) - close distance fast
- **800-3000m**: Combat maneuvering (2.2x speed, ~286 m/s) - positioning for shots
- **300-800m**: Attack/aim (1.5x speed, ~195 m/s) - slow for accuracy
- **<300m**: Danger zone (3.0x speed, ~390 m/s) - avoid collision

**Lead Pursuit**: Predicts player position 1.2s ahead for intercept angle

**Weapon Firing Conditions**:
- **Missiles**: 800-2000m range, <45° angle, 2s cooldown, max 2 active (CombatDirector)
- **Guns**: <1000m range, <10° angle, 0.5s cooldown

**Altitude Matching**: Gradual 10% approach when <3km from player, safety limits at <100m (climb) and >10km (descend)

### Player Entity Hierarchy
1. **PlayerPlane (parent)** - Physics container with RigidBody, Collider, all velocity components
   - Components: `PlayerInput`, `FlightState`, `FlightCamera`, `GameState`
2. **ModelContainer (child)** - Allows visual model rotation independent of physics
3. **Visual mesh (grandchild)** - F-16 GLTF model via `SceneRoot` with fallback cube

**Important**: Physics operates on parent's **global** transform; children use **local** coordinates.

### System Scheduling
Systems run in strict order each frame:
1. **FixedFirst** (before physics): `detect_nan_early` - NaN safety checks on scale/velocity
2. **Update**: Input reading, flight physics, drone AI, missile updates
3. **PostUpdate**: Camera update (slerp), chunk loading, LOD updates
4. **FixedPostUpdate**: Avian3D physics resolution

**Physics Safety**: Multiple layers of NaN protection:
- `normalize_or_zero()` instead of `normalize()`
- Scale validation (reject ≤0 or NaN)
- Particle size clamped 0.1-10.0
- Throttle/boost input clamped [0, 1]

### Flight Physics (Arcade Model)
- **Thrust**: Along `-transform.local_z()` (Bevy's forward is -Z). Toggle Rocket Mode (R key) for 8x multiplier.
- **Lift**: Altitude gain only above `min_speed` (~20 m/s), scales with speed squared
- **Bank-to-turn**: Roll angle creates yaw torque via `TURN_FACTOR`
- **Pitch limiting**: Reduces torque as pitch approaches `MAX_TILT_ANGLE` (~80°)
- **Angular damping**: `0.75` multiplier per frame creates realistic spindown
- **All constants** in `CONSTANTS` section of main.rs

### Coordinate System
- Bevy: **Y-up, -Z forward, +X right**
- Forward is `-transform.local_z()` (note the negative)
- Pitch/Roll/Yaw match Euler angles (local space)

## Asset Loading & Rendering

### Critical Rendering Requirements
1. **Ambient light ESSENTIAL**: Without it, all models render black. Set in `setup_scene()`.
2. **Camera far clip > fog distance**: Default 1000m clips geometry. Current: 100,000m far plane.
3. **Asset paths case-sensitive**: Even on Windows (e.g., `assets/models/` not `assets/Models/`)
4. **Assets copied to build output**: `cp -r assets target/release/` after code changes

### Asset Loading Pattern (Critical for Bevy 0.15)

**DO NOT use SceneRoot for simple GLB files** - it often fails to spawn visible children:
```rust
// ❌ BROKEN in Bevy 0.15 (children don't render)
SceneRoot(asset_server.load("tree.glb#Scene0"))

// ✅ WORKING pattern - direct mesh loading
let mesh = asset_server.load("tree.glb#Mesh0/Primitive0");
let material = materials.add(StandardMaterial {
    base_color: Color::srgb(0.4, 0.3, 0.2),
    ..default()
});
commands.spawn((
    Mesh3d(mesh),
    MeshMaterial3d(material),
    Transform::default(),
    GlobalTransform::default(),
));
```

**Known Bevy 0.15 Issues**:
- **Async texture bind group bug** (Issue #15081): Materials with JPG/PNG textures never update bind groups → Use procedural textures created at startup (see `procedural_textures.rs`)
- **SceneRoot visibility**: Simple `.glb` files often render as invisible → Always provide fallback cube mesh for debugging

### Visibility & LOD
- **LODLevel(0)** component with `Visibility::Inherited` gates all renderable entities
- LOD system runs in PostUpdate to guarantee newly-spawned trees visible next frame
- **NoFrustumCulling** component prevents culling glitches on large loaded scenes

## Key Conventions

### Component Design
- **Marker components** (e.g., `PlayerPlane`, `TreeEntity`) identify entity types for queries
- **Components are data**: Pure structs without logic
- **Systems query components**: Functions that mutate component state each frame

### Procedural Spawning
- **Deterministic hashing** for reproducible chunk generation: `hash(chunk_x, chunk_y, seed) % 100`
- **Probabilistic distribution**: 15% spawn rate for drones/villages = `rand() % 100 < 15`
- **Chunk entity parent pattern**: Always spawn children via `with_children()` for transform hierarchy

### Physics & Collisions
- **AABB colliders** for all physics objects (Avian3D handles 3D boxes)
- **Collision resolution**: Check Y position < 50m for ground impact (simplistic but stable)
- **NaN crash prevention**: Validate all Vec3/Quat values before physics step (see `detect_nan_early`)
- **Constraint**: NEVER modify `Transform.translation` or `Transform.rotation` directly on entities with `RigidBody` - use `ExternalForce`/`ExternalTorque` instead
  - **Exception**: Drones intentionally use Transform manipulation (not physics forces) for arcade AI behavior

### Audio
- **OGG format only**: MP3/WAV crash rodio. Remove cover art metadata with: `ffmpeg -i input.ogg -vn -c:a copy output.ogg`
- **Audio pitching**: Engine sound pitch varies with throttle percentage
- **Spatial audio**: Not implemented; all sounds are 2D mix

## Current Phase Status

✅ **Phase 1** - Flight mechanics, basic world
✅ **Phase 2** - Infinite chunks, villages, asset loader, rocket mode, sky→space transition, terrain heightmap (Perlin noise)
⏳ **Phase 3** - Drone combat AI (ACTIVE) - weapons debugging in progress
  - ✅ Lead pursuit navigation
  - ✅ Swarm flocking (separation, alignment, cohesion)
  - ✅ Tactical speed zones (warp/sprint/combat/attack/danger)
  - ✅ Altitude matching
  - ⚠️ Weapon firing system (missiles/guns) - **currently debugging** (see weapon diagnostics in drone.rs)
⏳ **Phase 4+** - Cockpit view, day/night cycle, building collisions, enhanced maneuvers

**Known Issues**:
- Weapons not firing despite diagnostic logging added (active debugging in progress)
- `.glb` visual fallbacks sometimes removed during refactors - always restore red cube if mesh missing
- Building roof models occasionally don't render - use SceneRoot with fallback cube approach

**User Hardware Target**: RTX 3080 Ti + Ryzen 7 5800X, 60+ FPS at 1080p

## Critical Debugging Patterns

**Physics crashes (NaN panic)**:
1. Check `detect_nan_early` is in FixedFirst schedule (BEFORE physics)
2. Verify scale is never 0 or NaN (collider dimensions)
3. Look for velocity/position becoming infinite
4. Pattern for safe interpolation:
   ```rust
   if !value.is_finite() || value <= 0.0 {
       eprintln!("⚠️ Invalid value detected!");
       value = SAFE_DEFAULT;
   }
   ```

**Rendering issues (black models)**:
1. Verify AmbientLight added in setup_scene() with brightness 200+
2. Check camera far clip plane is > 100,000 (prevents jagged horizon)
3. Add `NoFrustumCulling` if needed
4. **Never use SceneRoot for simple GLB files** - use Mesh3d + MeshMaterial3d pattern

**Chunking/LOD issues**:
1. Verify load radius ≥ 8km (prevents ground holes with 3km safety buffer)
2. Confirm LOD system runs PostUpdate (not Update)
3. Check chunk spawn messages in console: "CHUNK SPAWN..."
4. Use deterministic RNG seeding: `StdRng::seed_from_u64(chunk_hash)` for reproducible spawns

**Asset loading**:
1. Verify assets in `target/release/assets/` (not just `assets/`)
2. Check asset paths are case-sensitive (even on Windows)
3. Use procedural fallback textures if async bind group fails
4. Always provide fallback cube mesh when loading GLTF models for debugging

**Build cache false positives**:
- Symptom: Cargo builds in <1s but code changes not applied
- Fix: `touch src/main.rs && cargo build --release`

**Drone AI / Weapons issues**:
- Check console for "✅ WEAPON SYSTEM ACTIVATED" message
- Look for "━━━ DRONE DIAGNOSTIC ━━━" sections showing condition checks
- Common problems:
  - Angle requirements too strict (<10° for guns, <45° for missiles)
  - Drones flying through range too fast (tactical speed zones need tuning)
  - CombatDirector `max_missiles` limit reached (default: 2)
