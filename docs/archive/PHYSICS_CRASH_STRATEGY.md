# Physics Crash - Debugging & Fix Strategy

**Issue:** Game crashes with `assertion failed: b.min.cmple(b.max).all()` in avian3d after flying for a bit
**Root Cause:** NaN or Infinity values reaching the physics engine's collider AABB calculations
**Status:** Requires two-phase investigation and fix

---

## The Problem

The avian3d physics engine has safety checks that validate collider bounding boxes. When a collider's min > max in any dimension (which means NaN/Infinity got into the calculations), it panics with the assertion error.

The current NaN safety checks in `check_ground_collision()` and `safety_check_nan()` run AFTER the physics engine has already tried to validate colliders. We need to catch NaN values BEFORE they reach the physics system.

---

## Two-Phase Solution

### Phase 1: FLASH - Diagnose the NaN Source

**What Flash will do:**
1. Add aggressive NaN detection that logs BEFORE physics runs
2. Add missing Collider to explosion debris (most likely culprit)
3. Add validation to drone movement system
4. Add validation to projectile updates
5. Run the game and look for `🚨 EARLY:` messages in console
6. Report WHICH entity and WHICH field has NaN

**Expected output:**
```
🚨 EARLY: Projectile position has NaN! [x, y, z]
OR
🚨 EARLY: Collider extents would be NaN! scale=[x, y, z], extents=[x, y, z]
OR
🚨 EARLY: Drone position has NaN! [x, y, z]
```

**File:** PHYSICS_CRASH_DEBUG_FLASH.md
**Time estimate:** 1-2 hours

---

### Phase 2: PRO - Implement the Actual Fix

**What Pro will do:**
Based on Flash's NaN location report, implement targeted fixes:

- **If debris:** Add Collider::sphere(0.5) to explosion debris spawn
- **If drones:** Add rotation normalization and position bounds checking to move_drones()
- **If projectiles:** Add movement validation and position bounds checking to update_projectiles()
- **If physics:** Improve Euler angle handling in arcade_flight_physics()

**File:** PHYSICS_CRASH_FIX_PRO.md
**Time estimate:** 1-2 hours

---

## Why This Two-Phase Approach?

1. **Diagnostic phase identifies the exact culprit** - Without knowing which entity/field has NaN, fixes are just guesses
2. **Targeted fixes are safer** - Rather than adding NaN checks everywhere, we fix the actual source
3. **Parallelizable** - Flash can run diagnostics while Pro prepares fixes
4. **Faster overall** - 2 hours diagnosis + 1-2 hours fix = Done before trying random fixes

---

## Workflow

```
USER: Game crashes with avian3d assertion
  ↓
FLASH: Add NaN detection + run game
  ↓
FLASH: Reports "NaN in Projectile position" OR "NaN in Drone movement" etc.
  ↓
PRO: Implements fix based on Flash's report
  ↓
PRO: Tests and confirms crash is fixed
  ↓
USER: Can play without crashes ✅
```

---

## Most Likely NaN Sources (Ranked)

**1. Explosion Debris (70% probability)**
- Spawned with RigidBody::Dynamic but NO Collider
- Avian3D can't create AABB for entity without collider
- FIX: Add `Collider::sphere(0.5)` to debris spawn

**2. Drone Movement (20% probability)**
- `transform.forward()` might return NaN on invalid rotations
- Position calculations might overflow at extreme distances
- FIX: Validate rotation, clamp position bounds

**3. Projectile Updates (8% probability)**
- Velocity × delta_secs calculation produces NaN
- Position might accumulate floating-point errors
- FIX: Validate movement vector before applying

**4. Physics Calculations (2% probability)**
- Euler angle singularities in arcade_flight_physics
- Unlikely given existing NaN checks
- FIX: More aggressive angle clamping

---

## Testing After Fix

**Quick test (2 minutes):**
```bash
cargo build --release
cp -r assets target/release/
target/release/plane_game.exe
# Fly around, don't crash in 60 seconds? ✅
```

**Full test (5 minutes):**
```bash
# Run game and do:
# 1. Fly forward 30 seconds (normal flight)
# 2. Fire 3-4 missiles and destroy drones
# 3. Fly in all directions for 2+ minutes
# 4. Watch console for ⚠️ warnings (none expected)
# 5. Verify FPS stays 60+
```

---

## How to Run These Prompts

### For Flash:
1. Copy content from `PHYSICS_CRASH_DEBUG_FLASH.md`
2. Send to Gemini Flash with context
3. Ask Flash to follow the 5-step debugging mission
4. Have Flash run the game and report console output

### For Pro:
1. Wait for Flash's NaN report
2. Copy content from `PHYSICS_CRASH_FIX_PRO.md`
3. Send to Gemini Pro with Flash's diagnostic output
4. Ask Pro to implement the corresponding "Case X" fix
5. Have Pro test the fix

---

## If Fix Doesn't Work

If the crash still happens after both phases:

1. **Collect more detailed logs:**
   - Run game with both `🚨 EARLY:` detection AND `⚠️ SAFETY:` checks
   - Let game run until crash
   - Share full console output

2. **Profile the crash:**
   - Note exact conditions: after how many frames? During what action?
   - Check if crash is reproducible or random

3. **Escalate to Claude:**
   - Provide full console logs + crash conditions
   - May need deeper physics engine investigation
   - Could require changes to system ordering or physics update frequency

---

## Summary of Files

| File | Purpose | Owner |
|------|---------|-------|
| PHYSICS_CRASH_DEBUG_FLASH.md | 5-step diagnostic procedure | Flash |
| PHYSICS_CRASH_FIX_PRO.md | Fix implementations by case | Pro |
| PHYSICS_CRASH_STRATEGY.md | This file - overview + workflow | User |

---

## Success Metrics

✅ Game launches and runs
✅ Can fly for 60+ seconds without crash
✅ Can fire missiles and destroy drones without crash
✅ Explosions spawn and clean up correctly
✅ No Vulkan validation errors in console
✅ No `🚨 EARLY:` NaN warnings appear

**Total time estimate:** 2-4 hours (diagnosis + fix + testing)

