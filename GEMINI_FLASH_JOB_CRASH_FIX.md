# 🚨 GEMINI FLASH URGENT JOB: Fix avian3d Collider Crash

**Priority:** CRITICAL - Blocks all testing
**Assigned To:** Gemini Flash (fresh context)
**Status:** Ready to execute
**Time Estimate:** 1-2 hours

---

## The Problem

**Error Message:**
```
thread 'main' panicked at avian3d-0.2.1/src/collision/collider/mod.rs:378:9:
assertion failed: b.min.cmple(b.max).all()
```

**What This Means:**
Avian3D physics engine detected an invalid bounding box (AABB):
- `b.min` > `b.max` in one or more axes
- This happens with NaN, Infinity, or corrupted collider dimensions
- Typically caused by: physics state corruption, chunk loading/unloading issues, or extreme transforms

**Trigger:**
- Flying away from initial meteor cluster
- Turning plane around
- Moving to chunks far from spawn
- Game crashes when physics updates the bad collider

---

## Investigation Plan for Gemini Flash

### Step 1: Understand the Codebase
1. Read CLAUDE.md (focusing on: World System, Asset Loading Gotchas, Physics Safety)
2. Read src/main.rs focusing on:
   - Lines 650-750: `spawn_chunk()` function (creates ground colliders)
   - Lines 2400-2500: `check_ground_collision()` (physics updates)
   - Look for where colliders are spawned and despawned

### Step 2: Identify Root Cause
Search for potential issues:

**Option A: Chunk Despawn Leaves Bad Collider**
- Check if despawn_recursive properly cleans up colliders
- Verify physics world doesn't reference deleted entities
- Look for orphaned Collider components

**Option B: Physics Calculation Overflow**
- Check arcade_flight_physics for extreme values
- Verify rotation calculations don't produce NaN/Infinity
- Look for velocity clamping before physics update

**Option C: Collider Dimension Issue**
- Check spawn_chunk() ground collider size calculation
- Verify CHUNK_SIZE (1000m) doesn't overflow in collider math
- Look for division by zero or negative dimensions

**Option D: Transform Propagation Issue**
- Check if extreme rotations corrupt child entity physics
- Verify GlobalTransform calculations are bounded
- Look for physics state mutation during transform sync

### Step 3: Add Defensive Checks

Add safety checks in these locations:

**In spawn_chunk() (around line 670):**
```rust
// SAFETY: Verify collider dimensions are valid before spawning
let collider_size = CHUNK_SIZE / 2.0;
assert!(collider_size > 0.0, "Collider size is invalid!");
assert!(collider_size.is_finite(), "Collider size is NaN/Infinity!");

commands.spawn((
    Collider::cuboid(collider_size, 10.0, collider_size),
    // ... rest of spawn
));
```

**In check_ground_collision() (around line 2450):**
```rust
// SAFETY: Detect and prevent NaN collider dimensions
for (entity, collider) in &collider_query {
    // If collider exists but has invalid AABB, despawn it
    if !collider_bounds_valid(collider) {
        eprintln!("⚠️ SAFETY: Invalid collider on entity {:?}", entity);
        commands.entity(entity).despawn_recursive();
    }
}

fn collider_bounds_valid(collider: &Collider) -> bool {
    // Check that dimensions are positive and finite
    match collider {
        Collider::cuboid(x, y, z) => {
            x.is_finite() && *x > 0.0 &&
            y.is_finite() && *y > 0.0 &&
            z.is_finite() && *z > 0.0
        }
        _ => true, // Assume other shapes are valid
    }
}
```

**In arcade_flight_physics() (around line 1350):**
```rust
// SAFETY: Clamp transform before applying physics
if !transform.translation.is_finite() {
    eprintln!("⚠️ SAFETY: Non-finite transform detected!");
    transform.translation = Vec3::new(0.0, 500.0, 0.0);
}

if !velocity.x.is_finite() || !velocity.y.is_finite() || !velocity.z.is_finite() {
    eprintln!("⚠️ SAFETY: Non-finite velocity detected!");
    *velocity = LinearVelocity::ZERO;
}
```

### Step 4: Test the Fix
```bash
# Build with fixes
cd /c/Users/Box/plane_game
cargo build --release

# Copy assets
cp -r assets target/release/

# Run and test
target/release/plane_game.exe

# Test sequence:
# 1. Fly away from spawn in straight line (5km+ away)
# 2. Turn plane around 180 degrees
# 3. Fly back toward spawn
# 4. Repeat in different directions
# 5. Monitor console for safety warnings
# 6. If no crash after 10 minutes, fix is successful
```

### Step 5: Document the Fix
Before completing:
1. Note what the root cause was
2. Explain which safety check caught it
3. Update CLAUDE.md "Common Errors & Fixes" table
4. Add comment in code explaining the fix

---

## Files to Check

| File | Lines | Check For |
|------|-------|-----------|
| src/main.rs | 650-750 | spawn_chunk() collider spawning |
| src/main.rs | 2400-2500 | check_ground_collision() physics |
| src/main.rs | 1350-1450 | arcade_flight_physics NaN checks |
| src/main.rs | 600-650 | manage_chunks despawn logic |

---

## Expected Outcome

After fix:
- ✅ No assertion panics in avian3d
- ✅ Can fly 20km+ away and return safely
- ✅ Can turn in any direction without crash
- ✅ Safety checks print to console (showing they're working)
- ✅ Game runs for 5+ minutes without crash

---

## Success Criteria (For You to Verify)

```
Does the game:
1. Launch without errors? [ ]
2. Fly away from spawn 20km? [ ]
3. Turn around completely? [ ]
4. Fly back toward spawn? [ ]
5. Repeat 3 times without crash? [ ]
6. No console warnings about "SAFETY" checks? [ ] (optional - they can appear if legitimate)
7. FPS stays 60+? [ ]

All checked = SUCCESS ✅
```

---

## If Fix Doesn't Work

**Fallback Plan:**
1. Reduce LOAD_RADIUS to 4 chunks (4km) temporarily
2. Reduce UNLOAD_RADIUS to 6 chunks (6km)
3. This limits colliders in memory, might prevent overflow
4. Then investigate further

---

## Important Context for Gemini

**What Gemini Should Know:**
- Bevy 0.15 physics (Avian3D 0.2)
- Game is flight simulator with chunk-based world
- CHUNK_SIZE = 1000m, LOAD_RADIUS_CHUNKS = 8
- Physics runs in FixedUpdate schedule
- Ground colliders spawned dynamically per chunk

**What Not to Change:**
- Don't refactor main.rs structure
- Don't remove working systems
- Only add safety checks and assertions
- Keep Cuboid geometry for ground (not changing to other shapes)

---

## Gemini's Output Format

When Gemini Flash is done, ask for:
1. ✅ "Fixed: Here's what the problem was"
2. ✅ "Code changes: [paste modified functions]"
3. ✅ "Test results: [describe test flight]"
4. ✅ "Next job: [what Claude should do next]"

---

## After This Fix

Once crash is fixed:
1. Claude will implement Asset Loading (tree .glb models)
2. Then test Drone spawning/movement
3. Then implement Swarm AI
4. Then add Combat mechanics

---

**Created:** 2026-02-05
**Status:** Ready for Gemini Flash to execute
**Linked to:** ULTIMATE_PRIORITY_PLAN.md
