# Current Work Plan - 3-AI Coordinated Development
**Date:** 2026-02-05
**Status:** Ground textures improved → Ready for combat features
**Next Priority:** Drone collision/combat system + terrain jaggedness fix

---

## Known Issues to Address

### 1. 🟡 GROUND TEXTURE COLOR (JUST FIXED)
**Status:** ✅ Fixed - Changed base_color from white to neutral beige
**What was wrong:** Base color at 1.0,1.0,1.0 (pure white) was washing out the grass texture to yellow
**Fix applied:** Changed to Color::srgb(0.9, 0.9, 0.85) - neutral beige that lets texture shine through
**Next test:** Run game and verify ground looks like grass green, not yellow

### 2. 🔴 TERRAIN JAGGEDNESS (PRIORITY #1 - CLAUDE)
**Problem:** When flying further away, chunk edges become visible as jagged lines
**Root cause:** Horizon disk (infinite green floor) at Y=-500.0 works but chunks above/below still show edges
**Solution options:**
- **Option A (Quick):** Increase horizon disk size from 100km to 200km radius
- **Option B (Better):** Implement proper fog-based fadeout at chunk boundaries
- **Option C (Best):** Generate intermediate LOD chunks between loaded and unloaded areas

**Recommendation:** Start with Option B (fog blending) - professional flight sim technique

---

## Work Distribution Plan

### CLAUDE (You) - PRIORITY 1: Terrain Jaggedness Fix
**Time estimate:** 1-2 hours
**Complexity:** Medium
**What to do:**
1. Investigate horizon disk implementation (lines 581-601 in src/main.rs)
2. Check if fog system can blend chunk boundaries
3. Test expanding horizon disk radius vs fog vs LOD approach
4. Implement chosen solution and verify no visual artifacts

**Success criteria:**
- ✅ No jagged chunk edges visible when flying 10km+ away
- ✅ Smooth visual transition at chunk boundaries
- ✅ FPS stays 60+ during flight

---

### GEMINI PRO - PRIORITY 2: Drone Collision/Combat System
**Time estimate:** 2-3 hours
**Complexity:** High
**What to implement:**

1. **Drone health/damage system** (in src/drone.rs):
   - Add health depletion on projectile hit
   - Spawn explosion effect on death
   - Despawn drone when health <= 0

2. **Projectile-drone collision detection** (in src/main.rs):
   - Create system: `drone_projectile_collision()`
   - Check distance between each projectile and each drone
   - Hit radius: ~50m (generous for testing)
   - Apply damage: 25 HP per hit
   - Despawn projectile on hit

3. **Visual feedback**:
   - Console messages when drone hit (show health remaining)
   - Orange explosion sphere at death location
   - Optional: Rumble/screen shake effect

**Code location hints:**
- Lines 1350-1450: arcade_flight_physics() - understand velocity/position
- Lines 2400-2500: check_ground_collision() - similar collision pattern
- Lines 180-200: Drone component definition (add health if missing)

**Success criteria:**
- ✅ Can fire missile with Space bar
- ✅ Missile hits drone (see hit distance message)
- ✅ Drone health decreases each hit
- ✅ Drone explodes at 0 health
- ✅ Explosion is visible and despawns

---

### GEMINI FLASH - PRIORITY 3: Drone Model Loading Fix
**Time estimate:** 1-2 hours
**Complexity:** Medium
**Current issue:**
- Drones spawn as RED TEST CUBE instead of drone.glb model
- Similar to tree loading issue that was already fixed

**What to investigate:**
1. Check if drone.glb exists: `ls -la C:\Users\Box\plane_game\assets\models\`
2. Look at spawn_beaver_drone() function (src/drone.rs, around line 15-30)
3. Compare with working tree loading pattern (src/main.rs, lines 963-1010)
4. Tree loading uses: `asset_server.load("models/tree.glb#Mesh0/Primitive0")`
5. Drone likely needs same #Mesh0/Primitive0 selector

**What to fix:**
- Update spawn_beaver_drone() to use direct Mesh loading (not SceneRoot)
- Test with multiple drone model files if needed
- Add fallback to gray mesh if .glb fails (like existing red cube does)

**Debugging steps:**
```bash
# Verify drone.glb exists
ls -lh /c/Users/Box/plane_game/assets/models/drone.glb

# Check if it's actually a valid glb
file /c/Users/Box/plane_game/assets/models/drone.glb

# Compare with tree.glb structure
file /c/Users/Box/plane_game/assets/fantasy_town/tree.glb
```

**Success criteria:**
- ✅ Drones appear as 3D models (not red cubes)
- ✅ Drones spawn in formation near player
- ✅ Drones move smoothly forward
- ✅ Model is visible and not black/invisible

---

## Parallel Work Strategy

**Timeline:**
1. **Today (Session 1):**
   - Claude: Start terrain jaggedness fix (1-2 hours)
   - Gemini Pro: Start drone collision system (2-3 hours in parallel)
   - Gemini Flash: Fix drone.glb loading (1-2 hours in parallel)

2. **Next session:**
   - Integrate completed systems
   - Test full combat loop (shoot drones, see explosions)
   - Fix any integration issues

**Coordination notes:**
- Pro and Flash work in parallel (no dependencies)
- Claude's terrain fix is independent
- All three can work simultaneously without conflicts
- Test each subsystem before integration

---

## Success Milestones

### Milestone 1 (This session): ✅ Visual Improvements
- ✅ Ground textures (4K PNG grass - DONE)
- ⏳ Ground color corrected (awaiting test)
- ⏳ Terrain jaggedness reduced (Claude)
- ⏳ Drone models visible (Flash)

### Milestone 2 (Next session): Combat Ready
- ⏳ Drones can be shot (Pro)
- ⏳ Explosions visible on hit
- ⏳ Full combat loop working

### Milestone 3 (Phase 3): AI & Polish
- Swarm formations
- Kamikaze pursuit behavior
- Weapon variety (different missile types)
- Score/kill counter UI

---

## Code Structure Reference

```
src/main.rs (2922 lines)
├── Lines 511-575: Lighting & horizon setup
├── Lines 587-652: Chunk management (manage_chunks)
├── Lines 868-924: spawn_chunk() - ground mesh + texture
├── Lines 941-1010: spawn_trees_in_chunk() - tree models
├── Lines 1350-1450: arcade_flight_physics() - plane movement
├── Lines 2400-2500: check_ground_collision() - collision detection
└── Lines 2267-2295: NaN safety checks

src/drone.rs
├── Lines 1-30: Drone component & spawn function
├── Lines 50-80: KamikazeBehavior component
└── Lines 100-150: Drone movement system

assets/
├── textures/grass/ - 4K PNG (UPDATED)
├── models/drone.glb - Iranian drone (needs debug)
└── fantasy_town/ - Tree models (working)
```

---

## Prompts for AI Team

### For Gemini Pro:
```
TASK: Implement drone collision/combat system

GOAL: Make drones shootable and explode when hit by missiles

CURRENT STATE:
- Drones spawn in close formation (200m ahead)
- Red test cube visible (collision will work on this)
- Space key fires missile (creates sprite projectiles)
- Need: Hit detection + damage + explosions

TODO:
1. Create drone_projectile_collision system
2. Check distance between projectiles and drones (hit radius 50m)
3. Decrease drone.health by 25 per hit
4. Despawn projectile on hit
5. Spawn orange explosion sphere on death
6. Despawn drone when health <= 0
7. Add console messages for debugging

FILES TO MODIFY:
- src/main.rs: Add drone_projectile_collision() system and register it
- src/drone.rs: Ensure Drone struct has health field

TESTING:
- Spawn, fly forward, see drones ahead
- Fire missile with Space
- Watch for collision messages in console
- Verify drones explode
```

### For Gemini Flash:
```
TASK: Fix drone.glb model loading

GOAL: Replace red test cube with actual 3D drone model

CURRENT STATE:
- spawn_beaver_drone() uses SceneRoot (broken in Bevy 0.15)
- Drones spawn as red test cubes instead of models
- Trees already fixed using direct Mesh loading

TODO:
1. Check if drone.glb exists and is valid
2. Look at tree loading code (working example)
3. Update spawn_beaver_drone() to use Mesh3d + MeshMaterial3d pattern
4. Load drone.glb using asset_server.load("models/drone.glb#Mesh0/Primitive0")
5. Test with fallback gray cuboid if .glb fails

REFERENCE CODE (from trees, working):
asset_server.load("models/tree.glb#Mesh0/Primitive0")

FILES TO MODIFY:
- src/drone.rs: spawn_beaver_drone() function

TESTING:
- Build and run
- Verify drones appear as 3D models
- Check console for no asset errors
- Confirm 3 drones visible on startup
```

### For Claude (You):
```
TASK: Fix terrain jaggedness when flying far from origin

GOAL: Eliminate visible chunk edges and jagged boundaries

CURRENT STATE:
- Ground renders fine near origin
- When flying 10km+ away, chunk boundaries become visible as jagged lines
- Horizon disk helps but doesn't fully solve problem
- Need: Smooth fadeout or LOD system

OPTIONS:
A) Increase horizon disk size from 100km to 200km radius
B) Implement fog-based blending at chunk boundaries
C) Add LOD intermediate chunks between loaded/unloaded

RECOMMENDED: Option B (fog blending)

TODO:
1. Check current fog settings in setup_scene()
2. Extend fog to hide chunk boundaries (fade distance tuning)
3. Or: implement gradient fade on chunk edges
4. Test at various distances

FILE TO MODIFY:
- src/main.rs lines 581-601 (HorizonDisk)

TESTING:
- Fly away 20km in all directions
- Verify no jagged edges visible
- Check FPS stays 60+
```

---

## Expected Outcome (By Next Session)

✅ **Visual Polish:**
- Ground with realistic grass texture
- No texture color issues
- Smooth chunk transitions (no jagged edges)
- Drone models visible instead of cubes

✅ **Combat System:**
- Fire missiles with Space
- Missiles hit drones (distance-based)
- Drones take damage and die
- Explosions visible on impact
- Console feedback for all events

✅ **Ready for Phase 3:**
- Combat loop functional
- Can proceed with AI behavior (swarms, pursuit)

---

**Next review:** After all three systems completed, integrate and test full combat sequence.
