# Plane Game - Claude Development Context

## Project Overview
F-16 flight simulator with infinite procedurally-generated terrain, chunk-based world loading, and arcade physics. Built with Bevy 0.15 + Avian3D physics.

**Status:** Phase 2 COMPLETE + Phase 3 STARTED - Chunk-based infinite world with professional asset loader, rocket mode, seamless sky→space, physics hardened. **Phase 3 IN PROGRESS:** Drone combat system with advanced swarm AI (lead pursuit, obstacle avoidance, flocking), missile-drone collision detection, kamikaze mechanics working. **Recent fixes:** Grass texture sampler configuration (fixed UV tiling artifacts), UI/Drone system scheduling (all systems properly gated by GameState). **Known issues:** Building .glb models not rendering (Bevy 0.15 SceneRoot limitation, Phase 4+).

## User Design Goals (Multi-Session Coordination)

**High Priority:**
- World size: As large as possible without crashing
- Asset distribution: Tons of medieval villages and forest sections scattered throughout
- Performance: Must run on RTX 3080 Ti + Ryzen 7 5800X without lag
- Visual style: Mix of Kenney assets (currently used for terrain)

**Phase 2 Features (IMPLEMENTED):**
- ✅ Professional asset loader: bevy_asset_loader with GameState machine (Loading → Playing)
- ✅ Grass texture loading: PNG asset with proper UV sampler configuration (fixes tiling artifacts)
- ✅ System scheduling: All gameplay systems gated by GameState::Playing (prevents race conditions)
- ✅ Rocket booster mode (R key toggle): 8x thrust multiplier, reaches 25km in ~12 seconds
- ✅ Sky→space transition: Gradual color change from blue to black (15-25km range)

**Phase 3 Features (IN PROGRESS):**
- ✅ Drone spawning: Infinite chunk-based spawning (15% per chunk) with advanced swarm AI (lead pursuit, obstacle avoidance, flocking)
- ✅ Missile system: Fire with Space, collision detection with drones
- ✅ Kamikaze mechanics: Drones collide at 20m proximity, explosion effects
- ✅ Dynamic drone speed: Warp pursuit (5.0x) when player >5km away, sprint (3.0x) when >2km, combat (2.2x) when <1km
- ✅ Infinite world patrol: Drones automatically despawn at 15km+ distance, new ones spawn ahead
- ⚠️ **REGRESSION:** Drone 3D model rendering - visual fallback removed in latest work, may be invisible now
- ⏳ Speed counter UI: Show velocity numerically (next task)

**Phase 4+ Features (Not Yet Started):**
- Cockpit view: Future addition (steal code from open-source plane game if available)
- Day/night cycle: Currently always ambient lighting on
- Ground texture/grid: Make speed visible through ground pattern
- Building collisions: Currently villages don't block player

**Design Constraints:**
- Keep current camera system for now (3rd person view behind plane)
- Use OGG audio only (rodio crashes on cover art metadata)
- Performance target: 60+ FPS during normal flight

## Gameplay Features (Already Implemented - From Gemini Session)

### Game States
- **Playing:** Normal flight, altitude > 50m, fuel available
- **GameOver:** Crash when altitude < 50m or velocity > 50m/s impact

### Controls
- **W/S:** Pitch (nose up/down)
- **A/D:** Roll (barrel roll)
- **Q/E:** Yaw (turn left/right)
- **Shift:** Throttle/Boost (increases fuel burn rate)
- **Ctrl:** Brake
- **R:** Toggle Rocket Mode (8x thrust multiplier)
- **Space:** Fire (weapons - fires sprite particles)

### Systems Implemented
- **Fuel System:** Resource logic with idle burn rate (normal flight) and boost burn rate (afterburner)
- **UI:** Retro "Gas Canister" icon + dynamic fuel bar (color changes with fuel level)
- **Audio:** Engine loop (pitch varies with throttle), wind noise (airspeed), altitude alarms, crash impact sound
- **Physics:** Hybrid system - Avian3D AABB collisions + custom arcade flight physics (bank-to-turn model)
- **Particles:** Afterburner flame inherits 20% of plane velocity for realistic trails, scales 3x with boost

## Critical Architecture

### World System
- **Chunk size:** 1000m × 1000m
- **Load radius:** 8 chunks (8km buffer) - CRITICAL: prevents ground holes
- **Unload radius:** 12 chunks (12km)
- **Trees per chunk:** 5-10 (was 50, reduced for performance)
- **Tree Y position:** 0.0 (ground level, not 1.0)
- **Chunk coordinates:** Calculated via `ChunkCoordinate::from_world_pos(pos) = (pos.x / 1000).floor() as i32`

### Asset Loading Gotchas
- **SceneRoot with `#Scene0` selector DOESN'T WORK** - ~~Bevy 0.15 doesn't load scene children properly~~ **[OBSOLETE - FIXED 2026-02-06]**
  - ~~`.glb` files load but don't spawn visible children~~
  - ✅ **RESOLVED:** Now using `Mesh3d + MeshMaterial3d` with `#Mesh0/Primitive0` selector (see line 1103)
  - Direct mesh loading pattern working in production
- **Assets must be in `target/release/assets/`** - game runs from executable, not source
  - Copy assets after every code change: `cp -r assets target/release/`
- **All asset paths are case-sensitive** (even on Windows with Bevy)

### Lighting & Rendering
- **Ambient light is ESSENTIAL** - without it, all models render pitch black
  - Added: `AmbientLight { color: WHITE, brightness: 200.0 }` in setup_scene()
  - Without this, trees were invisible despite being spawned
- **Camera far clip plane MUST exceed fog distance** - prevents jagged edges at horizon
  - Default Camera3d has far clip of 1000m (too short for large worlds)
  - Current fog ends at 30,000m (30km)
  - **FIX:** Set `Projection::Perspective { far: 100000.0, ..default() }` (100km)
  - Geometry was being clipped by camera before fading into fog
- **NoFrustumCulling component** - used to prevent frustum culling issues with loaded scenes
  - Added to tree entities to guarantee visibility
- **DirectionalLight** - illuminance 30000.0 with shadows enabled

### Physics Safety (From Gemini Hardening Session)
- **NaN crash protection (Multiple layers):**
  - `arcade_flight_physics`: Uses `normalize_or_zero()` instead of `normalize()`, clamps throttle/boost safely
  - `spawn_afterburner_particles`: Early return if throttle input is NaN, particle size clamped to 0.1-10.0
  - `update_particles`: Despawns particles with NaN transforms immediately
  - `check_ground_collision`: Hard reset if Y > 100,000 or Y < -1,000 (prevents "dark bar" glitch)
  - **Result:** Game is stable, no physics crashes or warped flames

### Rendering & Sky System (From Gemini Hardening Session)
- **Jagged Horizon Problem SOLVED:**
  - **Root cause:** Horizon Disk was blue (sky color) but ground chunks green, creating sharp contrast
  - **Solution:** Infinite Earth technique - make HorizonDisk green (0.25, 0.3, 0.25) to match chunks exactly
  - **Fog Sync:** Tightened fog to 3000m start, 6000m end (ensures 100% opacity before 8000m chunk edge)
  - **Disk properties:** 100km radius, enabled fog, matches ground material properties
  - **Result:** Seamless infinite-looking world with no stair-step artifacts
- **Warped Afterburner Particle Fix:**
  - Root cause: Particle size could reach extreme values (NaN), creating giant streaks
  - Solution: Clamped size to 0.1-10.0 range in `spawn_afterburner_particles`
  - Result: Clean flame effects, no visual glitches
- **Clear Color & Fog Color Sync:**
  - All sky/fog/horizon elements use Color::srgba(0.5, 0.6, 0.8, 1.0) consistently
  - Creates seamless sky-to-horizon-to-ground visual continuity

## Debugging Techniques That Worked

1. **Entity hierarchy inspection** - Add system that checks `if children.len() > 0` to verify SceneRoot spawned children
2. **LOD visibility tracking** - Print visible/hidden tree counts every frame to spot culling issues
3. **Chunk position logging** - Print chunk coordinates and world positions when spawning/unloading
4. **Test mesh substitution** - Replace problematic assets with Cuboid to test rendering pipeline
5. **System ordering matters** - Move LOD to PostUpdate instead of Update so newly-spawned trees are visible

## Common Errors & Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| Cargo build instant (< 1s) but changes not applied | Build cache, file mtime not updated | `touch src/main.rs` then rebuild |
| Physics feel broken, plane unstable | Performance lag from too many entities (100% village spawn) | Reduce spawn rates, check FPS in console |
| "Path not found: assets/..." | Assets in src/assets, not target/release/assets | Copy: `cp -r assets target/release/` |
| Jagged edges at horizon | Camera far clip (1000m) < fog distance (30km) | Set Projection::Perspective { far: 100000.0 } |
| Trees exist but invisible | Zero ambient light | Add AmbientLight resource in setup_scene() |
| Ground has holes/gaps | LOAD_RADIUS too small (5km) | Increase to 8km for 3km cushion |
| Trees appearing but then disappearing | LOD system hiding them | Move LOD to PostUpdate schedule |
| SceneRoot children not visible | Bevy 0.15 visibility bug | Use Mesh3d + MeshMaterial3d instead |
| Player crash (NaN panic) | Invalid physics state | Add NaN check in check_ground_collision() |

## Asset Library Location

**External Asset Drive:** `F:\Plane_Game Assets\`
- `kenney_fantasy-town-kit_2.0/` - Medieval buildings (currently in use)
- `kenney_city-kit-commercial_2.1/` - Modern city buildings
- `kenney_city-kit-suburban_20/` - Suburban houses
- `kenney_city-kit-roads/` - Road system
- `kenney_space-kit/` - Space assets
- `future_fighterjet/` - Fighter jet models
- `Grass004_1K-PNG/`, `Grass004_4K-PNG/` - High-res grass textures
- `Resource Boy - Cloud Textures/` - Cloud atlas
- `Sonniss.com - GDC 2019 - Game Audio Bundle/` - Professional sound effects
- `Ultimate Stylized Nature - May 2022/` - Nature assets
- `kenney_blocky-characters_20/` - Character models
- `kenney_particle-pack/` - Particle effects
- `Sci-Fi Essentials Kit[Standard]/` - Sci-fi props

**Current Usage:**
- ✅ `kenney_fantasy-town-kit_2.0/` copied to `assets/fantasy_town/`
- ⏳ Other assets available but not yet integrated

## File Locations

- **Core:** `src/main.rs` (2540+ lines)
  - Chunk system: lines 587-652 (manage_chunks)
  - Tree spawning: lines 803-870 (spawn_trees_in_chunk)
  - LOD system: lines 701-735 (update_lod_levels)
  - Physics safety: lines 2267-2295 (check_ground_collision)
  - Lighting & Camera: lines 511-602 (setup_scene) - includes AmbientLight + far clip fix

- **Assets:** `assets/fantasy_town/` (167 files)
  - Trees: tree.glb, tree-crooked.glb, tree-high.glb, tree-high-crooked.glb, tree-high-round.glb
  - Buildings: wall.glb, roof-gable.glb (5 variants tested)
  - Building components: 160+ decoration assets

## Performance Baseline (Current)

- **Chunk load radius:** 8km
- **Trees per chunk:** 5-10
- **Total entities:** ~360-500 trees + ground meshes + UI
- **FPS target:** 60+ (current: playable, not measured)
- **GPU:** RTX 3080 Ti
- **CPU:** Ryzen 7 5800X

**Scaling notes:** Reducing tree count works, increasing radius causes lag. The 8km/12km ratio provides good balance.

## Next Steps (Priority Order)

### Priority 1: FIX ASSET LOADING (BLOCKER) ✅ **COMPLETED 2026-02-06**
**Status:** ~~Main visual blocker - trees/buildings still rendered as green cubes~~ **RESOLVED**
**Problem:** ~~SceneRoot doesn't spawn visible .glb children in Bevy 0.15~~ **FIXED**
**Solution Implemented:** Direct Mesh loading with `#Mesh0/Primitive0` selector:
1. ✅ Load glTF Mesh directly via `Mesh3d(asset_server.load(tree_model_path))`
2. ✅ Apply StandardMaterial manually: `MeshMaterial3d(tree_material.clone())`
3. ✅ Spawn as Mesh3d entity (not SceneRoot wrapper)
4. ✅ Updated in: `spawn_trees_in_chunk()` function (lines 1103-1104)
5. ✅ Trees now render with proper mesh loading

**Result:** Asset loading pipeline working. Trees load as direct meshes with materials applied.
**Completed:** 2026-02-06

### Priority 2: Verify Phase 1 & 2 Complete
- Run game and check:
  - Ground continuous, no holes ✓ (8km radius working)
  - Chunks load/unload smoothly ✓
  - Physics stable, no crashes ✓ (NaN protection added)
  - Horizon seamless ✓ (green disk + fog sync done)
  - Rocket mode 25km reachable in ~12 sec ✓
  - FPS 60+ during normal flight (untested)

### Priority 3: Phase 3 Combat System
**Blocker:** Waiting on drone 3D models from user
**When ready:** PHASE3_IMPLEMENTATION_PROMPTS.md has full design
**Effort:** ~12 hours once assets provided

### Optional Polish
- Ground grid texture for speed feedback
- Proper LOD mesh variants (instead of visibility toggle)
- Building colliders in villages
- Enhanced particle effects

## Bash Commands Useful for This Project

```bash
# Copy assets to release build
cp -r /c/Users/Box/plane_game/assets /c/Users/Box/plane_game/target/release/

# Build and run (release mode)
cd /c/Users/Box/plane_game && cargo build --release && target/release/plane_game.exe

# Force rebuild when cargo caches changes (verify with compile time >1m)
touch src/main.rs && cargo build --release

# Test console output for specific features (villages, roofs, drones)
timeout 10 target/release/plane_game.exe 2>&1 | grep -E "(🏘️|🏠|CHUNK PATROL)"

# Quick test (8 second timeout)
cd /c/Users/Box/plane_game && timeout 8 target/release/plane_game.exe 2>&1 | grep -E "(LOD|CHUNK|SPAWN|children)"

# Check asset file types
file /c/Users/Box/plane_game/assets/fantasy_town/*.glb

# List all .glb files
ls -lh /c/Users/Box/plane_game/assets/fantasy_town/*.glb

# Sync code to Google Drive (from Gemini workflow)
robocopy "C:\Users\Box\plane_game" "G:\My Drive\plane_game" /MIR /XD target

# Fix OGG audio files (remove cover art that crashes rodio)
ffmpeg -i input.ogg -vn -c:a copy output.ogg
```

## Known Limitations

- **SceneRoot broken in Bevy 0.15** for simple .glb files (no scene hierarchy) **[PARTIALLY RESOLVED]**
  - ✅ **Trees FIXED:** Now using direct Mesh3d loading with `#Mesh0/Primitive0` selector (line 1103)
  - ⚠️ **Buildings PARTIALLY WORKING (2026-02-15):**
    - ✅ Walls rendering successfully with Mesh3d + MeshMaterial3d
    - ❌ Roofs NOT visible (roof-gable.glb asset loading issue)
    - **Current fix:** Using SceneRoot + fallback red cube for roof visibility
    - **Material fix:** Separated wall_material (brown) from roof_material (red-brown tint)
    - **Issue:** Direct Mesh3d loading may not work for complex roof geometry
    - **Affected code:** main.rs lines 1340-1410 (spawn_village_in_chunk)
    - **Testing needed:** Verify fallback cubes appear, then debug GLB scene path
- **No multi-GPU support** - single-threaded asset loading can stall frames
- **Collision system** - only checks Y position, not actual AABB collision
- **No night mode** - AmbientLight always on, no day/night cycle
- **Player starts at (0, 500, 0)** - no spawn point variation
- **Audio:** OGG files with cover art crash rodio - must remove metadata with ffmpeg first
- **Line Endings:** Windows Git uses CRLF by default, but Rust/Config files need LF only

## Code Patterns

- **Chunk spawning:** Always spawn as children of chunk entity via `with_children(|parent| { parent.spawn(...) })`
- **Visibility toggle:** Use `LODLevel(0)` component + `Visibility::Inherited` on all renderable entities
- **Transform hierarchy:** Child entities use LOCAL coordinates; chunk parent handles world positioning
- **Component naming:** Use clear prefixes (Tree, VillageBuilding, ChunkEntity) for system queries
- **Debug logging:** Use eprintln!() for console output (survives release builds)
- **Fallback visual debug:** For asset loading issues, use `.with_children(|parent| { parent.spawn(Mesh3d(Cuboid)) })` with bright contrasting color
- **Spawn rate tuning:** Probabilistic spawns (villages, drones) - start at 15% for testing visibility, reduce to 5% for production performance

## Testing Checklist

- [ ] Ground renders without holes at all altitudes
- [ ] Trees appear with correct lighting (visible, not black)
- [ ] No crashes when flying fast (NaN protection working)
- [ ] Assets load from target/release/assets/ correctly
- [ ] Frame rate stable at 60+ FPS when flying
- [ ] Chunk loading/unloading smooth (no sudden pops)
- [ ] Code changes compiled (verify build time >1m, not cached <1s)
- [ ] Probabilistic spawns working (check console for 🏘️ village, 🛸 drone messages)
- [ ] Performance acceptable (villages at 15% spawn = <10% FPS impact)

## Multi-Chat Coordination

**Sessions Working on This Project:**
1. **Gemini Chat:** Gameplay implementation (controls, fuel system, audio, particles, physics hybrid model)
2. **Claude Chat (2026-02-05):** Phase 1 asset rendering debugging - fixed invisible trees via 7 emergency fixes
3. **Copilot Chat:** (Referenced, current context unknown)
4. **User Notes:** Design goals and preferences documented in HomeBrain Sync

**Shared Context Location:**
- `C:\Users\Box\Documents\HomeBrain SYNC\Inbox\Plane Game Notes.md` - Central hub for all chat summaries
- `C:\Users\Box\plane_game\CLAUDE.md` - Team documentation (this file)
- `C:\Users\Box\plane_game\.claude.local.md` - Local development notes
- `C:\Users\Box\plane_game\FIXES_APPLIED.md` - Emergency fixes applied in latest session

---

## Session 2026-02-05 Learnings (Claude Session)

### Fixed Issues
1. **Physics Crash (AABB assertion)** ✅
   - Root cause: NaN in collider dimensions (transform.scale could be zero or NaN)
   - Solution: `detect_nan_early()` system in `FixedFirst` schedule (runs BEFORE physics)
   - Now catches AND FIXES invalid values before physics engine sees them
   - Added scale validation: reject scale with any dimension ≤ 0 or NaN

2. **ESC Key Respawn** ✅
   - Changed: ESC now respawns player (was quit game)
   - F10 to quit instead
   - Convenient for testing combat repeatedly

3. **Drone Spawn Distance & Speed** ✅
   - Moved spawn from 200m (instant death) to 1km+ away
   - Speed reduced to 80-130 m/s to allow player reaction time
   - Drones spawn in formations for tactical variety

4. **Drone Pursuit AI** ✅
   - Implemented: Lead pursuit (predicting player movement)
   - Swarm behavior: separation, alignment, cohesion
   - Obstacle avoidance (meteors)
   - Tactical weaving for visual interest
   - Dynamic speed multiplier (warp pursuit if >5km away)

### Known Blockers (Bevy 0.15 Issues)
1. **JPG Texture Not Rendering** ⚠️
   - Issue: [Bevy #15081](https://github.com/bevyengine/bevy/issues/15081)
   - Problem: Materials with async-loaded textures never update bind group after load
   - Workaround: Use procedural textures (working but less detailed)
   - Attempted fix: `check_grass_texture_loaded()` system with `AssetServer::get_load_state()` - doesn't fix bind group update bug
   - Better solution: Implement `bevy_asset_loader` crate OR wait for Bevy fix

2. **Drone 3D Model (.glb) Not Rendering** ⚠️
   - File exists: `assets/models/drone.glb` (12MB)
   - Issue: Using `SceneRoot(asset_server.load("models/drone.glb#Scene0"))`
   - Workaround: Using visual red cube fallback mesh
   - Root cause: Bevy 0.15 SceneRoot mesh loading complexity (known limitation)
   - Fix attempted: Direct mesh loading with `#Mesh0/Primitive0` selector - didn't work
   - Better approach: Confirm correct scene path with `#Scene0` or use different asset format
   - **Note:** Tree loading now works with Mesh3d pattern (see line 1103) - same approach could work for drones

### Code Patterns That Worked Well
1. **NaN Detection Pattern:**
   ```rust
   // In FixedFirst schedule (before physics)
   if !value.is_finite() || value <= 0.0 {
       eprintln!("⚠️ Invalid!");
       value = safe_default;
   }
   ```

2. **Procedural Texture Pattern:**
   ```rust
   // Create texture at startup (no async issues)
   let texture_data = vec![...];
   let image = Image::new(extent, format, texture_data, ...);
   image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
       address_mode_u: ImageAddressMode::Repeat,
       address_mode_v: ImageAddressMode::Repeat,
       ..default()
   });
   ```

3. **Drone Query Pattern:**
   ```rust
   let Ok((player_transform, player_velocity)) = player_query.get_single() else { return };
   for (entity, mut transform, drone) in &mut drone_query {
       // Can safely access player without checking every iteration
   }
   ```

### Testing Insights
- Game needs 5+ minutes of flight to reveal async loading issues
- Drones spawning close (200m) causes instant collisions - bad for testing
- Procedural texture acceptable visual quality for now
- Red cube drones highly visible - good for debugging AI behavior
- 60+ FPS maintained with 6 drones + combat system

### Critical Fixes to Keep
- **NaN detection in FixedFirst** - do NOT move to Update schedule (physics runs before Update)
- **detect_nan_early() MUST check scale** - zero/negative scale causes AABB assertion
- **Drone spawn 1km+** - player needs time to see and react
- **ESC respawn** - much better UX for testing

---

## AI Workflow & Maintenance

**Memory File Status:** CLAUDE.md is the Source of Truth
- Updated whenever major features complete or blockers identified.
- **Documentation Hub:** Located in the `docs/` folder.
  - `docs/TECHNICAL_KNOWLEDGE.md`: Consolidates physics safety, asset workarounds, and Bevy 0.15 specifics.
  - `docs/FEATURE_PLANS.md`: Active plans for Phase 3 combat and future UI/Environment polish.
- **Archive:** Outdated session logs and prompts are moved to `docs/archive/`.

**Project Files:** Cleaned and consolidated (2026-02-15)
- **KEEP:** README.md, CLAUDE.md, GAME_DESIGN.md.
- **Git Strategy:** Run `git commit` before major refactors as safety checkpoint.
- **Code Management:** Keep main.rs under 1500-2000 lines before splitting.

**Last Updated:** 2026-02-15 (Documentation consolidated into /docs)
**Next Review:** Fix drone visibility regression FIRST (add fallback cube)
**Priority Blockers:** Drone visual fallback removed (must restore), Bevy 0.15 async texture loading (#15081)

---

## Session 2026-02-06 Learnings (Gemini + Claude Handoff)

### Gemini's Work Completed ✅
1. **Infinite Chunk-Based Drone Spawning:**
   - 15% chance per chunk to spawn a drone patrol
   - Drones spawn at chunk center + 500m altitude
   - Follows deterministic hash (same chunk = same spawn)
   - Code: `src/main.rs` lines 1012-1018

2. **Warp Pursuit AI Enhancement:**
   - `>5km away:` 5.0x speed (650 m/s) - instant catch-up
   - `>2km away:` 3.0x speed (390 m/s) - sprint mode
   - `<1km away:` 2.2x speed (280 m/s) - combat mode
   - `<1km away:` 1.5x speed (190 m/s) - standard
   - Code: `src/drone.rs` lines 167-175

3. **Advanced Swarm Intelligence:**
   - Lead pursuit (predicts player 1.2s ahead)
   - Obstacle avoidance (250m radius from meteors)
   - Flocking behavior (separation/alignment/cohesion within 400m)
   - Tactical weaving (sine wave pattern)
   - Code: `src/drone.rs` lines 98-144

4. **Lifetime Management:**
   - Drones despawn at 15km+ distance (performance)
   - Prevents infinite spawning memory leak
   - Code: `src/drone.rs` lines 181-185

### Critical Regression Found ⚠️
**Issue:** Visual fallback removed when switching to SceneRoot
- **Before:** Direct mesh load with StandardMaterial (dark gray, always visible)
- **After:** SceneRoot only, no fallback → drones may be invisible
- **Impact:** Drones spawn and run AI but players can't see them
- **Status:** MUST FIX in next Gemini session
- **Solution:** Add red cube fallback in `spawn_beaver_drone()` (lines 22-59 of drone.rs)

### Claude Session Actions
1. Built and tested game - confirmed startup works
2. Analyzed Gemini's changes - found regression
3. Verified infinite spawning works (28+ drones across chunks in 8s test)
4. Created comprehensive prompt for Gemini: `GEMINI_NEXT_SESSION_PROMPT.md`
5. Updated CLAUDE.md with current status

### Code Issues to Address
| File | Issue | Fix |
|------|-------|-----|
| drone.rs:22-59 | No visual fallback | Add Cuboid mesh + dark gray/red material |
| drone.rs:33 | SceneRoot path unknown | Verify `#Scene0` is correct (was `#Mesh0/Primitive0`) |
| main.rs:1012-1018 | Infinite spawn works ✅ | No changes needed |
| drone.rs:98-175 | AI works perfectly ✅ | No changes needed |

### Test Results
- Game launches cleanly
- Chunks spawn with proper loading radius
- Drones spawn in ~28 chunks immediately
- No crashes or NaN errors
- Console shows proper "CHUNK PATROL" messages
- **BUT:** Unknown if drones are visible (need visual test)

---

## Session 2026-02-15 Learnings (Claude Session)

### Issue Identified: Village Roofs Not Rendering
**User reported via screenshot:**
- ✅ Building walls rendering (brown boxes)
- ❌ Roofs completely missing (should have gabled red/brown roofs)
- ❌ Distant buildings showing yellow/white color (wrong material)

### Root Causes Found
1. **Material blending** - Single brown material used for both walls AND roofs made roofs invisible
2. **Mesh loading method** - Direct `Mesh3d` with `#Mesh0/Primitive0` may not work for complex roof geometry
3. **No visual fallback** - If GLB fails to load, nothing renders

### Fix Applied (main.rs:1340-1410)
1. **Separated materials:**
   - `wall_material` - Darker brown for building bases
   - `roof_material` - Bright red-brown tint (0.8, 0.3, 0.2) for visibility
   - Both use `colormap.png` texture but roof has color tint overlay

2. **Changed roof loading method:**
   - **Before:** `Mesh3d(asset_server.load("roof-gable.glb#Mesh0/Primitive0"))`
   - **After:** `SceneRoot(asset_server.load("roof-gable.glb#Scene0"))`
   - Reasoning: SceneRoot may handle complex geometry better (works for meteors/turrets)

3. **Added fallback visual:**
   - Red `Cuboid(2.0, 1.0, 2.0)` spawned as child of roof entity
   - Guarantees roofs visible even if GLB fails
   - Can be removed once GLB loading confirmed working

4. **Debug logging:**
   - Added `println!("🏠 Spawning roof at ...")` to track roof entity creation

### Testing Status
- ✅ Build successful (1m 42s compile time)
- ⚠️ Runtime testing needed to verify:
  - [ ] Red fallback cubes visible on buildings
  - [ ] Walls remain brown (not yellow/white)
  - [ ] Console shows roof spawn messages
  - [ ] FPS stable with new rendering approach

### Next Steps
1. **Run game and verify fallback cubes visible**
2. **If cubes visible:** GLB loading confirmed broken, try alternative roof assets
3. **If cubes NOT visible:** Position/scale issue, need to adjust Transform values
4. **Once working:** Remove fallback cubes, tune roof material color

### Files Modified
- `src/main.rs:1340-1410` - spawn_village_in_chunk() material separation + roof fallback
- `CLAUDE.md` - Updated Known Limitations section
- Created `/c/Users/Box/VILLAGE_ROOF_FIX.md` - Detailed fix documentation