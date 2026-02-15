# Gemini Knowledge Update & Action Plan
**Date:** 2026-02-15
**Session Context:** Claude identified that Gemini's suggestions duplicate existing work. This plan corrects knowledge gaps and provides clear next steps.

---

## ❌ DO NOT IMPLEMENT - Already Complete

### 1. Lighting System ✅ DONE
**Your suggestion:** "Add DirectionalLight and AmbientLight to reveal F-16 detail"
**Reality:** Both already exist and working perfectly.

**Evidence:**
- **File:** `src/main.rs`
- **Function:** `setup_scene()` (around line 511-602)
- **Code exists:**
  ```rust
  // Directional Light (Sun)
  commands.spawn((
      DirectionalLight {
          illuminance: 30000.0,
          shadows_enabled: true,
          ..default()
      },
      Transform::from_rotation(Quat::from_euler(
          EulerRot::XYZ,
          -std::f32::consts::FRAC_PI_4,
          std::f32::consts::FRAC_PI_4,
          0.0,
      )),
  ));

  // Ambient Light (Essential for model visibility)
  commands.insert_resource(AmbientLight {
      color: Color::WHITE,
      brightness: 200.0,
  });
  ```

**Status:** Production-ready, DO NOT TOUCH.

---

### 2. Fog System ✅ DONE
**Your suggestion:** "Add fog to hide sharp horizon"
**Reality:** Advanced fog system implemented with sky transition sync.

**Evidence:**
- **File:** `src/main.rs`
- **System:** `update_fog_based_on_altitude()` (around line 1490-1540)
- **Features:**
  - Fog range: 3000m start → 6000m end (optimized for 8km chunk load radius)
  - Sky-to-space gradient: 15-25km altitude transition
  - Rocket mode support: Fog fades at high altitude
  - Color synced with horizon disk (seamless infinite world effect)

**Status:** Hardened in Phase 2, performance-tested. DO NOT MODIFY.

---

### 3. Asset Loading System ✅ DONE
**Your suggestion:** "Replace dark cubes with fantasy_town assets"
**Reality:** Professional asset loader with bevy_asset_loader, trees and buildings spawning.

**Evidence:**
- **Files:** `src/assets.rs`, `src/main.rs`
- **System:** `GameState::Loading` → `GameState::Playing` with proper async loading
- **Current assets spawning:**
  - **Trees:** 5-10 per chunk, using `tree.glb`, `tree-crooked.glb`, `tree-high.glb` (line 1103)
  - **Buildings:** 15% of chunks get villages (8 buildings + 1 tower per village)
  - **Grass texture:** Procedural + PNG fallback (Bevy 0.15 async texture bug workaround)
  - **Meteors, turrets, objectives:** All using SceneRoot loading

**What you might see as "cubes":**
- **Red roof cubes:** Intentional fallback visual (we're debugging `roof-gable.glb` loading)
- **Wall buildings:** Brown boxes ARE the wall.glb meshes (correct, just simple geometry)

**Status:** Phase 2 complete, DO NOT REDO ASSET SYSTEM.

---

### 4. VFX System ✅ EXISTS (Partial)
**Your suggestion:** "Add contrails and engine glow"
**Reality:** Afterburner system exists, contrails are Phase 4.

**Evidence:**
- **File:** `src/main.rs`
- **System:** `spawn_afterburner_particles()` (around line 2190-2250)
- **Features:**
  - Particle size scales with throttle (0.3-3.0 range)
  - Velocity inheritance (20% of plane velocity for realistic trails)
  - Boost mode: 3x particle scale
  - NaN-safe (clamped size 0.1-10.0)
  - Color: Orange glow with emissive lighting

**What's missing:** Wingtip contrails at high speed/altitude (valid suggestion for Phase 4).

**Status:** Core VFX done, enhanced VFX is optional polish.

---

## ✅ RECOMMENDED WORK - Phase 3 Priorities

### Priority 1: Verify Roof Fix (IMMEDIATE)
**Context:** Claude just implemented roof material separation + fallback cubes. Needs visual confirmation.

**What was changed:**
- **File:** `src/main.rs:1340-1410`
- **Change 1:** Separated `wall_material` (brown) from `roof_material` (red-brown tint)
- **Change 2:** Added fallback `Cuboid(2.0, 1.0, 2.0)` with bright red material
- **Change 3:** Changed roof loading from `Mesh3d` to `SceneRoot(roof-gable.glb#Scene0)`
- **Change 4:** Village spawn rate: 5% → 15% (better visibility without lag)

**Your task:**
1. Run game: `cargo run --release`
2. Fly around and find villages (15% of chunks)
3. **Expected:** Brown walls + bright red cubes on top
4. **Report back:**
   - Can you see red roof cubes? (Yes/No)
   - Are walls brown or yellow/white? (Color check)
   - FPS during flight over villages? (Performance check)

**If red cubes NOT visible:**
- Position issue: Try adjusting Y from 5.5 to 8.0 or 10.0
- Scale issue: Try increasing scale from 6.0 to 10.0
- Material issue: Check if `roof_material` is being applied

**If red cubes ARE visible:**
- Test if `roof-gable.glb` loads underneath (SceneRoot might work)
- If GLB loads, remove fallback cubes
- Tune `roof_material` color to be less red, more brown

**Acceptance criteria:**
- [ ] Roofs visible on all buildings
- [ ] Distinct from walls (different material/color)
- [ ] FPS > 60 with 15% village spawn
- [ ] No visual artifacts (z-fighting, flickering)

---

### Priority 2: Terrain Heightmap (HIGH IMPACT)
**Your suggestion:** "Add noise-based heightmap for hills"
**Status:** ✅ VALID - This is genuinely missing and would improve the game significantly.

**Why this matters:**
- Flat ground makes flight feel arcade-y and boring
- No visual feedback for altitude changes
- Hills would create tactical challenges (terrain avoidance)
- Performance cost: Low (just Y-offset in ground mesh generation)

**Implementation Plan:**

#### Step 1: Add Noise Library
**File:** `Cargo.toml`
```toml
[dependencies]
noise = "0.9"  # Perlin/Simplex noise for terrain generation
```

#### Step 2: Create Heightmap Function
**File:** `src/main.rs` (add near chunk system, around line 800)
```rust
use noise::{NoiseFn, Perlin};

/// Generate terrain height using Perlin noise
fn get_terrain_height(world_x: f32, world_z: f32) -> f32 {
    let perlin = Perlin::new(42); // Fixed seed for deterministic terrain

    // Layer 1: Large rolling hills (500m wavelength, 50m amplitude)
    let scale_large = 0.002; // 1/500
    let height_large = perlin.get([world_x as f64 * scale_large, world_z as f64 * scale_large]) as f32 * 50.0;

    // Layer 2: Medium features (100m wavelength, 15m amplitude)
    let scale_medium = 0.01; // 1/100
    let height_medium = perlin.get([world_x as f64 * scale_medium + 100.0, world_z as f64 * scale_medium + 100.0]) as f32 * 15.0;

    // Layer 3: Small details (20m wavelength, 3m amplitude)
    let scale_small = 0.05; // 1/20
    let height_small = perlin.get([world_x as f64 * scale_small + 200.0, world_z as f64 * scale_small + 200.0]) as f32 * 3.0;

    // Combine layers (additive for natural-looking terrain)
    let total_height = height_large + height_medium + height_small;

    // Clamp to reasonable range (prevent crazy mountains)
    total_height.clamp(-20.0, 80.0)
}
```

#### Step 3: Modify Ground Chunk Spawning
**File:** `src/main.rs`
**Function:** `spawn_chunk()` (around line 987-1030)

**Find this code:**
```rust
// Ground plane for this chunk
let ground_mesh = meshes.add(Plane3d::new(
    Vec3::Y,
    Vec2::splat(CHUNK_SIZE / 2.0),
));
```

**Replace with:**
```rust
// Generate heightmap-based terrain mesh
let ground_mesh = create_terrain_mesh(chunk_coord, meshes);
```

**Add this new function** (around line 1100):
```rust
/// Create terrain mesh with heightmap applied
fn create_terrain_mesh(
    chunk_coord: ChunkCoordinate,
    meshes: &mut Assets<Mesh>,
) -> Handle<Mesh> {
    const SUBDIVISIONS: usize = 20; // 20x20 grid = 400 triangles per chunk
    const CHUNK_SIZE: f32 = 1000.0;

    let chunk_world_x = chunk_coord.x as f32 * CHUNK_SIZE;
    let chunk_world_z = chunk_coord.z as f32 * CHUNK_SIZE;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices with heightmap
    for z in 0..=SUBDIVISIONS {
        for x in 0..=SUBDIVISIONS {
            let local_x = (x as f32 / SUBDIVISIONS as f32) * CHUNK_SIZE - CHUNK_SIZE / 2.0;
            let local_z = (z as f32 / SUBDIVISIONS as f32) * CHUNK_SIZE - CHUNK_SIZE / 2.0;

            let world_x = chunk_world_x + local_x;
            let world_z = chunk_world_z + local_z;

            let height = get_terrain_height(world_x, world_z);

            positions.push([local_x, height, local_z]);
            normals.push([0.0, 1.0, 0.0]); // Simple normal (could calculate from neighbors for lighting)
            uvs.push([x as f32 / SUBDIVISIONS as f32, z as f32 / SUBDIVISIONS as f32]);
        }
    }

    // Generate indices (two triangles per quad)
    for z in 0..SUBDIVISIONS {
        for x in 0..SUBDIVISIONS {
            let i0 = (z * (SUBDIVISIONS + 1) + x) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + (SUBDIVISIONS + 1) as u32;
            let i3 = i2 + 1;

            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    meshes.add(mesh)
}
```

#### Step 4: Update Tree/Building Placement
**File:** `src/main.rs`
**Function:** `spawn_trees_in_chunk()` (around line 1080)

**Change tree Y position from 0.0 to terrain height:**
```rust
// OLD:
let tree_local_pos = Vec3::new(x, 0.0, z);

// NEW:
let world_x = chunk_pos.x + x;
let world_z = chunk_pos.z + z;
let terrain_height = get_terrain_height(world_x, world_z);
let tree_local_pos = Vec3::new(x, terrain_height, z);
```

**Do the same for villages** in `spawn_village_in_chunk()` (line 1366):
```rust
// Update building Y position:
let terrain_height = get_terrain_height(building_x, building_z);
Transform {
    translation: Vec3::new(building_x, terrain_height - 0.5, building_z),
    // ... rest unchanged
}
```

#### Step 5: Update Ground Collision Check
**File:** `src/main.rs`
**Function:** `check_ground_collision()` (around line 2267)

**Change from fixed Y=50 to terrain-aware:**
```rust
// OLD:
const GROUND_LEVEL: f32 = 50.0;
if transform.translation.y < GROUND_LEVEL {
    // crash
}

// NEW:
let terrain_height = get_terrain_height(
    transform.translation.x,
    transform.translation.z,
);
const CRASH_MARGIN: f32 = 50.0; // Still need some clearance
if transform.translation.y < terrain_height + CRASH_MARGIN {
    // crash
}
```

#### Step 6: Testing Checklist
- [ ] Cargo build succeeds with `noise` dependency
- [ ] Ground chunks render with hills visible
- [ ] Trees spawn on terrain surface (not floating/buried)
- [ ] Buildings follow terrain height
- [ ] Ground collision triggers at correct height
- [ ] FPS impact minimal (< 5% drop)
- [ ] No z-fighting or mesh artifacts
- [ ] Horizon still seamless (fog hides chunk edges)

**Performance targets:**
- **Subdivisions: 20** = 400 triangles/chunk (good balance)
- If FPS drops > 10%, reduce to subdivisions: 15 (225 triangles)
- If FPS stable, could increase to 25 (625 triangles) for smoother hills

**Visual tuning:**
- **Too spiky?** Reduce small detail amplitude (3m → 1m)
- **Too flat?** Increase large hill amplitude (50m → 100m)
- **Wrong scale?** Adjust wavelengths (500m → 1000m for gentler slopes)

---

### Priority 3: Phase 3 Combat Polish (MEDIUM IMPACT)

**Context:** Drone combat system exists but has issues identified by Claude.

**Current status (from CLAUDE.md):**
- ✅ Drone spawning: 15% per chunk with swarm AI
- ✅ Missile system: Space key fires, collision detection working
- ✅ Kamikaze mechanics: Drones collide at 20m proximity
- ✅ Dynamic speed: Warp pursuit (5x) when player >5km away
- ⚠️ **REGRESSION:** Drone 3D model rendering - visual fallback removed, may be invisible

**Your task:**
1. **Verify drones are visible:**
   - Run game, fly around
   - Check console for "CHUNK PATROL" messages
   - If drones invisible, add fallback cube (same pattern as roof fix)

2. **Test combat loop:**
   - Fire missiles (Space key)
   - Do missiles hit drones?
   - Do drones explode?
   - Does player take damage from kamikaze?

3. **If drones invisible, fix in `src/drone.rs:22-59`:**
   ```rust
   // Add fallback visual (same pattern as roofs)
   .with_children(|parent| {
       parent.spawn((
           Mesh3d(meshes.add(Cuboid::new(2.0, 0.5, 3.0))), // Drone-shaped
           MeshMaterial3d(materials.add(StandardMaterial {
               base_color: Color::srgb(0.8, 0.1, 0.1), // Dark red
               emissive: LinearRgba::rgb(2.0, 0.0, 0.0), // Red glow
               ..default()
           })),
           Transform::IDENTITY,
       ));
   });
   ```

**Acceptance criteria:**
- [ ] Drones visible (either GLB model or fallback cube)
- [ ] Missiles spawn and travel correctly
- [ ] Collision detection works (missile + drone)
- [ ] Kamikaze mechanic triggers at 20m
- [ ] Console shows no entity despawn errors

---

### Priority 4: HUD Evolution (LOW IMPACT - Optional Polish)

**Your suggestion:** "Fighter pilot style HUD with crosshair"
**Status:** Valid but low priority (Phase 4+).

**Current HUD (from `src/ui.rs`):**
- Fuel bar with gas canister icon
- Text-based flight data

**What would improve it:**
1. Center crosshair (aiming reticle)
2. Altitude ladder (vertical scale)
3. Horizon line (artificial horizon)
4. Speed tape (vertical speed indicator)
5. Compass rose (heading indicator)

**Defer this until:**
- Core gameplay loop finalized
- Drone combat balanced
- Performance optimized

**Reason:** HUD changes don't affect gameplay feel, just presentation. Terrain heightmap and drone visibility have much higher ROI.

---

## 🚫 DO NOT DO

1. **Don't fix compiler warnings** - They're just unused variables (cosmetic only)
2. **Don't add new lighting** - DirectionalLight + AmbientLight already perfect
3. **Don't implement new fog** - Current fog system is production-hardened
4. **Don't rebuild asset loader** - bevy_asset_loader integration complete
5. **Don't create new .md files** - Update existing GEMINI.md only

---

## 📋 Recommended Execution Order

### Session 1: Verify Current State (30 min)
1. Read CLAUDE.md fully (understand what exists)
2. Read `src/main.rs:511-602` (lighting system)
3. Read `src/main.rs:1340-1410` (village/roof system)
4. Run game, verify:
   - Lighting looks good
   - Fog visible at horizon
   - Trees spawning
   - Villages spawning (15% of chunks)
   - Roofs visible (red cubes expected)
5. Update GEMINI.md with current state

### Session 2: Roof Fix Validation (1 hour)
1. Fly around, find 3-5 villages
2. Take screenshots of roofs
3. Report findings to user:
   - Red cubes visible? (Yes/No + screenshot)
   - Wall color? (Brown/Yellow/White)
   - FPS? (Number)
4. If cubes NOT visible:
   - Debug position/scale
   - Try different Y heights (5.5 → 10.0)
5. If cubes visible:
   - Test if `roof-gable.glb` loaded underneath
   - Tune material color if needed

### Session 3: Terrain Heightmap (3-4 hours)
1. Add `noise` crate to Cargo.toml
2. Implement `get_terrain_height()` function
3. Implement `create_terrain_mesh()` function
4. Update `spawn_chunk()` to use new mesh
5. Update tree/building placement with terrain height
6. Update ground collision with terrain-aware check
7. Test thoroughly:
   - Visual check (hills visible?)
   - Performance check (FPS impact?)
   - Collision check (crashes at right height?)
8. Tune parameters for best look

### Session 4: Drone Visibility (1 hour)
1. Check if drones visible in current build
2. If not, add fallback cube to `drone.rs`
3. Test combat loop (missiles, collisions)
4. Fix entity despawn errors if present

---

## 🎯 Success Metrics

**After completing this plan, the game should have:**
1. ✅ Roofs visible on all buildings (red cubes or GLB models)
2. ✅ Terrain with hills/valleys (not flat)
3. ✅ Drones visible and combat working
4. ✅ FPS > 60 during normal flight
5. ✅ No entity despawn errors in console
6. ✅ All existing features still working (lighting, fog, assets)

**Performance budget:**
- Terrain heightmap: < 5% FPS impact
- Village spawning (15%): < 10% FPS impact
- Total FPS: > 60 on RTX 3080 Ti

---

## 📁 Critical Files Reference

| File | Purpose | Lines | Status |
|------|---------|-------|--------|
| `src/main.rs:511-602` | Lighting system (setup_scene) | DO NOT MODIFY |
| `src/main.rs:1340-1410` | Village spawning + roof fix | VERIFY WORKING |
| `src/main.rs:987-1030` | Chunk spawning | MODIFY for heightmap |
| `src/main.rs:1080-1115` | Tree spawning | MODIFY for terrain Y |
| `src/main.rs:2267-2295` | Ground collision | MODIFY for terrain check |
| `src/drone.rs:22-59` | Drone spawning | FIX if invisible |
| `src/ui.rs` | HUD system | DEFER changes |
| `CLAUDE.md` | Full project context | READ FIRST |
| `GEMINI.md` | Your session notes | UPDATE with findings |

---

## 🔍 Knowledge Verification Questions

**Before starting work, answer these to confirm you understand:**

1. Does the game need new lighting? (Answer: NO - already exists)
2. Does the game need fog? (Answer: NO - already exists at 3000-6000m)
3. Are assets loading? (Answer: YES - trees and buildings spawn from fantasy_town)
4. What's the current village spawn rate? (Answer: 15%, was 5%, was 100% causing lag)
5. What color should roofs be? (Answer: Red cubes as fallback, brown when GLB loads)
6. What's the main missing feature? (Answer: Terrain heightmap - ground is flat)
7. Should you fix compiler warnings? (Answer: NO - low priority cosmetic issue)
8. What's the FPS target? (Answer: 60+ on RTX 3080 Ti)

**If you can't answer these, re-read CLAUDE.md before proceeding.**

---

## 📞 Communication Protocol

**When reporting progress:**
1. State what you tested/implemented
2. Provide evidence (console output, screenshots, code snippets)
3. Report metrics (FPS, entity counts, performance impact)
4. List any blockers or questions
5. Suggest next steps

**When asking questions:**
1. Reference specific file + line number
2. Explain what you tried already
3. Show relevant code snippets
4. State expected vs actual behavior

**When updating GEMINI.md:**
1. Add findings to "Development Status" section
2. Update "Known Issues" if you find new problems
3. Remove resolved issues
4. Keep it concise (< 50 lines total)

---

**End of plan. Begin with Session 1: Verify Current State.**
