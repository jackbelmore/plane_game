# Plane Game Physics Crash - Analysis & Next Steps

**Date:** 2026-02-05
**Issue:** Game crashes during gameplay with avian3d AABB assertion error
**Status:** Diagnostic prompts created - ready to share with Gemini agents

---

## What Happened

You've successfully implemented:
✅ Ground textures (1K PNG, solid green fallback)
✅ Drone model loading (3D models visible)
✅ Drone combat system (missiles, collisions, explosions)
✅ 6 drones spawning in formations

But then the game crashes with:
```
assertion failed: b.min.cmple(b.max).all() in avian3d collider
Vulkan semaphore validation error: VUID-vkQueueSubmit-pSignalSemaphores-00067
```

This happens after flying for a bit, suggesting NaN (not-a-number) or Infinity values are being generated somewhere and reaching the physics system.

---

## Root Cause Analysis

The avian3d physics engine validates collider bounding boxes before using them. If a collider's min > max in any dimension (which indicates NaN/Infinity), it asserts and crashes.

**Problem:** Current NaN safety checks run AFTER the physics engine tries to use the colliders.

**Solution:** Add aggressive NaN detection BEFORE physics runs, identify the source, and fix it.

---

## Likely Culprits (Most to Least Likely)

### #1: Explosion Debris (70% probability)
In `spawn_huge_explosion()`, debris particles are spawned with:
- RigidBody::Dynamic ✅
- LinearVelocity ✅
- **Collider::sphere()** ❌ MISSING

Avian3D can't create AABB for physics bodies without colliders.

**Fix:** Add `Collider::sphere(0.5)` to debris spawn

### #2: Drone Movement (20% probability)
The `move_drones()` system directly modifies Transform without validation:
- Might produce NaN from invalid rotation calculations
- Might overflow at extreme coordinates (100km+)

**Fix:** Validate rotation, clamp position bounds, check movement vector

### #3: Projectile Updates (8% probability)
`update_projectiles()` might calculate invalid positions:
- Velocity × time calculation produces Infinity
- Position accumulates floating-point errors
- Coordinate overflow at large distances

**Fix:** Validate movement vectors, bound positions

### #4: Physics Calculations (2% probability)
`arcade_flight_physics()` might produce NaN from Euler angle conversions

**Fix:** More aggressive pitch angle clamping

---

## What I've Created for You

I've prepared THREE detailed documents with specific instructions for Gemini agents:

### 1. **PHYSICS_CRASH_DEBUG_FLASH.md**
For Gemini Flash - Diagnostic phase

**Contents:**
- Step 1: Add aggressive NaN detection system (NEW: `detect_nan_early()`)
- Step 2: Check explosion debris (add missing Collider)
- Step 3: Validate drone movement function
- Step 4: Validate projectile updates
- Step 5: Run game and collect logs

**Output:** Flash reports which entity and which field has NaN

**Time:** 1-2 hours

---

### 2. **PHYSICS_CRASH_FIX_PRO.md**
For Gemini Pro - Fix implementation phase

**Contents:**
- Case 1: If debris has no collider → add it
- Case 2: If drone movement corrupted → add validation
- Case 3: If projectile movement corrupted → add validation
- Case 4: If physics math failing → improve angle handling

**Output:** Pro implements targeted fix and tests it

**Time:** 1-2 hours

---

### 3. **PHYSICS_CRASH_STRATEGY.md**
Overview document explaining the two-phase approach

---

## Your Next Steps

### Immediate (Choose One)

**Option A: Send prompts to both agents in parallel**
```
To Gemini Flash:
"We have a physics crash. Read PHYSICS_CRASH_DEBUG_FLASH.md and execute the 5-step diagnostic procedure. Run the game and report any 🚨 EARLY: NaN messages."

To Gemini Pro:
"Prepare to implement fixes. Read PHYSICS_CRASH_FIX_PRO.md and wait for Flash's diagnostic output, then implement the corresponding case fix."
```

**Option B: Send to Flash first, then Pro**
```
To Flash: "Execute PHYSICS_CRASH_DEBUG_FLASH.md diagnostic"
(Wait for output)
To Pro: "Flash found NaN at [location]. Implement [Case X] from PHYSICS_CRASH_FIX_PRO.md"
```

### Timeline

- **Phase 1 (Flash):** 1-2 hours → identifies NaN source
- **Phase 2 (Pro):** 1-2 hours → implements and tests fix
- **Total:** 2-4 hours

### Success Criteria

✅ Game runs for 5+ minutes without crash
✅ Can spawn/destroy drones without crash
✅ Explosions clean up properly
✅ No `🚨 EARLY:` NaN warnings in console

---

## Files to Share with Agents

Send these three files to your Gemini agents:

```
/c/Users/Box/plane_game/PHYSICS_CRASH_DEBUG_FLASH.md      → Gemini Flash
/c/Users/Box/plane_game/PHYSICS_CRASH_FIX_PRO.md          → Gemini Pro
/c/Users/Box/plane_game/PHYSICS_CRASH_STRATEGY.md         → Reference
```

---

## If Flash Can't Reproduce the Crash

If Flash runs the game and doesn't get a crash:
1. Have them fly longer (5+ minutes)
2. Have them destroy more drones (create more explosions)
3. Have them fly to extreme coordinates (test bounds)
4. Have them check if solid green ground helps or hurts (disable textures)

The crash IS reproducible based on your earlier testing, so extended testing should trigger it.

---

## Current State Summary

| Component | Status | Issue |
|-----------|--------|-------|
| Ground | ✅ Rendering | 1K texture loading asynchronously (not visually displayed yet) |
| Trees | ✅ Working | 3D models visible |
| Drones | ✅ Visible | 3D models, 6 per spawn, movement working |
| Combat | ✅ Functional | Missiles, collisions, explosions all working |
| Physics | ❌ Crashing | NaN values in unknown location causing AABB assertion |

---

## Next Session Goals

1. **Fix physics crash** (with Flash/Pro) - 2-4 hours
2. **Test combat loop** - Can fly 10+ minutes without crash
3. **Implement terrain jaggedness fix** - Expand horizon or add fog
4. **Phase 3 AI:** Swarm formations + kamikaze pursuit

---

## Questions to Monitor

As Flash and Pro work, watch for:

**From Flash:**
- Which 🚨 EARLY message appears?
- Does the crash still happen after diagnostic additions?
- Do any ⚠️ SAFETY messages appear?

**From Pro:**
- Did the build succeed?
- Does game run longer without crash?
- Any new warnings in console?

---

## Contingency Plan

If the two-phase approach doesn't work:

1. Have Flash extend the diagnostic to check:
   - Player transform scale and rotation validity
   - Chunk entity collider dimensions
   - System ordering issues (is physics running before NaN checks?)

2. Have Pro consider:
   - Increasing NaN tolerance/clamping
   - Reducing physics engine frequency
   - Adding entity deserialization checks

3. Report to Claude with full console logs for deeper investigation

---

**Next action:** Choose Option A or B above and send the prompts to your Gemini agents.

The crash is fixable - we just need to identify the exact source first.

