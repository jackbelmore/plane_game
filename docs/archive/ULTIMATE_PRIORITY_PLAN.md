# Ultimate Priority Plan - Flight Simulator

**Date:** 2026-02-05
**Current State:** Phase 1&2 complete (rendering hardened), Drone module started, CRITICAL CRASH to fix
**Goal:** Get to playable Phase 3 combat with drones

---

## 🚨 CRITICAL BLOCKER: Avian3D Collider Crash

**Error:** `assertion failed: b.min.cmple(b.max).all()`
**Location:** avian3d collision system AABB validation
**Trigger:** Flying away from meteor cluster, turning around
**Root Cause:** Collider has NaN/Infinity/invalid dimensions

### Why This Matters
- ❌ Game crashes when flying to certain areas
- ❌ Blocks all testing and development
- ❌ Likely related to chunk loading/unloading physics state

### Investigation Needed
1. Check if chunk despawn leaves colliders behind
2. Verify physics values don't overflow during rotation
3. Check ground collider dimensions in spawn_chunk()
4. Examine if extreme transforms propagate to physics

---

## Priority 1: FIX THE CRASH ⚠️ (BLOCKER)

**Estimated Time:** 1-2 hours
**Assigned To:** Gemini Flash (fresh context)
**Deliverable:** Crash-free flight in any direction

### What Gemini Flash Should Do
```
1. Read the crash error details
2. Analyze chunk_system collider spawning (line ~670)
3. Check if physics state corrupts on chunk unload
4. Add safety checks for:
   - NaN in chunk collider dimensions
   - Physics values during extreme rotations
   - Collider cleanup when chunks despawn
5. Test: Fly 20km in all directions without crash
```

### Success Criteria
- ✅ No assertions in avian3d
- ✅ Can fly away from spawn and return safely
- ✅ No physics warnings in console
- ✅ Game remains stable for 5+ minutes of flight

---

## Priority 2: Asset Loading Fix (MAIN VISUAL)

**Status:** Plan ready in ASSET_LOADING_FIX_PLAN.md
**Estimated Time:** 1-2 hours
**Blocker For:** Seeing actual trees instead of green cubes

### What Needs to Happen
1. Replace SceneRoot with direct Mesh loading in spawn_trees_in_chunk()
2. Load tree.glb → StandardMaterial → Mesh3d
3. Test with cargo run --release
4. Verify trees render with proper lighting

### Why After Crash Fix
Can't test asset changes if game keeps crashing mid-flight

---

## Priority 3: Drone Integration & Movement

**Status:** Gemini Flash just created src/drone.rs
**Assets:** drone.glb ready at assets/models/
**Estimated Time:** 30-60 minutes

### Current State
- ✅ Drone module created (src/drone.rs)
- ✅ SceneRoot spawning logic implemented
- ✅ Movement system added (moves forward by speed)
- ❌ Not yet spawned or tested

### What Needs to Happen
1. Move asset: E:\Downloads\80_followers_iranian_shahed-136_drone.glb → assets/models/drone.glb
2. Test spawn_beaver_drone() function
3. Verify drone appears in game world
4. Verify drone moves forward smoothly
5. Test scaling/rotation look correct

### Success Criteria
- ✅ Drone visible in game
- ✅ Moves in predictable direction
- ✅ Scale/rotation look right
- ✅ No visual glitches

---

## Priority 4: Swarm & Kamikaze AI

**Status:** PHASE3_IMPLEMENTATION_PROMPTS.md has design
**Estimated Time:** 4-6 hours
**Depends On:** Drones spawning correctly

### Architecture (From Phase 3 Prompts)
```rust
#[derive(Component)]
enum DroneAI {
    Swarm {
        formation_offset: Vec3,
        leader: Entity,
        weave_timer: f32,
    },
    Kamikaze {
        target: Entity,
        last_seen_pos: Vec3,
    },
}
```

### Implementation Steps
1. Create swarm formation logic (circles around leader)
2. Add weaving patterns (dodge-like movement)
3. Implement kamikaze pursuit (direct line to player)
4. Add collision callbacks for hits
5. Balance difficulty (60% swarm, 40% kamikaze per chunk)

---

## Priority 5: Combat Mechanics

**Status:** Prompts ready, needs drone collision system
**Estimated Time:** 3-4 hours
**Depends On:** Drones with AI working

### What's Needed
1. Projectile → Drone collision detection
2. Drone health system (currently 50.0 hp)
3. Drone destruction/despawn on death
4. Explosion effects (particles)
5. Score/feedback to player

---

## Priority 6: Polish & Optimization

**Status:** Optional, can skip if Phase 3 works
**Estimated Time:** 2-3 hours

### Options
- Add ground grid texture for speed feedback
- Implement proper LOD (not just visibility toggle)
- Add building colliders to villages
- Improve particle effects
- Add HUD combat indicators

---

## Timeline Estimate

| Phase | Task | Hours | Blocker |
|-------|------|-------|---------|
| 1 | Fix avian3d crash | 1-2 | None (CRITICAL) |
| 2 | Asset loading (trees) | 1-2 | Phase 1 |
| 3 | Drone spawning/movement | 0.5-1 | Phase 1 |
| 4 | Swarm AI implementation | 4-6 | Phase 3 |
| 5 | Kamikaze AI + combat | 3-4 | Phase 4 |
| 6 | Polish (optional) | 2-3 | Phases 4-5 |
| **Total** | **All to Phase 5** | **10-15** | |

---

## File Organization

### Core Architecture
- **src/main.rs** - Main game loop, physics, flight controls (2758 lines)
- **src/drone.rs** - NEW: Drone components, spawning, movement (created)

### Critical Modules Needed
- **Swarm AI system** (in main.rs or new module)
- **Kamikaze AI system** (in main.rs or new module)
- **Combat collision handler** (in main.rs or new module)

### Documentation
- **CLAUDE.md** - Source of Truth (keeps UPDATED)
- **ASSET_LOADING_FIX_PLAN.md** - Tree asset fix
- **PHASE3_IMPLEMENTATION_PROMPTS.md** - Combat AI design
- **ULTIMATE_PRIORITY_PLAN.md** - This file

---

## Parallel Work Possible

While Gemini Flash fixes the crash:
- Can review PHASE3_IMPLEMENTATION_PROMPTS.md
- Can manually move drone.glb asset to assets/models/
- Can prepare test spawn points for drones

---

## Risk Management

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Crash blocks all work | Severe | Gemini Flash priority #1 |
| Asset loading fails | Visual blocker | Fallback: keep green cubes |
| Drone glTF doesn't load | Phase 3 blocker | Use test Cuboid instead |
| AI pathfinding breaks | Gameplay blocker | Start with simple pursuit |
| Physics performance tanks | FPS drop | Reduce drone count or complexity |

---

## Decision Points

### After Phase 1 (Crash Fix)
- **Decision A:** Continue to asset loading immediately
- **Decision B:** Test drone spawning first (faster feedback)
- **Recommendation:** B (5 min test, validates gemini/drone integration)

### After Phase 3 (Drone Movement)
- **Decision A:** Implement swarm AI (complex, requires careful design)
- **Decision B:** Test simple kamikaze first (easier to debug)
- **Recommendation:** B (prove combat works before adding formations)

### After Phase 5 (Combat Basics)
- **Decision A:** Add polish (ground grid, LOD, etc.)
- **Decision B:** Submit game as Phase 3 complete, call it done
- **Recommendation:** B (feature-complete is better than polished but broken)

---

## Success = Playable Drone Combat

**Definition:**
- ✅ Drones spawn in chunks
- ✅ Swarm AI circles and weaves
- ✅ Kamikaze AI pursues player
- ✅ Player can shoot drones (Space key)
- ✅ Drones explode on hit
- ✅ Game stays stable at 60+ FPS
- ✅ 5+ minute flight without crash

---

**Owner:** User + Gemini Flash (fresh context)
**Last Updated:** 2026-02-05
**Status:** Ready for execution
