# GEMINI PRO: Physics Crash Fixes - Implementation Guide

**Priority:** CRITICAL (Game-breaking crash)
**Status:** Awaiting Flash's NaN source identification
**Blocker:** Need output from PHYSICS_CRASH_DEBUG_FLASH.md

---

## Overview

Flash will run diagnostics and report WHERE the NaN is coming from. Your job is to IMPLEMENT THE FIX based on the source location.

---

## Probable NaN Sources & Fixes

### Case 1: NaN in Explosion Debris (Most Likely)

**Symptom:** Crash happens shortly after destroying a drone (explosion spawned)

**Root Cause:** Debris spawned with RigidBody::Dynamic but no Collider. Avian3D panics when trying to create AABB.

**Fix Location:** src/main.rs, function spawn_huge_explosion() around line 2774

**Implementation:**

Find this code:
```rust
commands.spawn((
    Mesh3d(debris_mesh),
    MeshMaterial3d(debris_material),
    Transform::from_translation(position),
    GlobalTransform::default(),
    Visibility::default(),
    InheritedVisibility::default(),
    RigidBody::Dynamic,
    LinearVelocity(debris_velocity),
    GravityScale(1.0),
    ExplosionEffect { lifetime: 0.0, max_lifetime: 5.0 },
));
```

Change to:
```rust
commands.spawn((
    Mesh3d(debris_mesh),
    MeshMaterial3d(debris_material),
    Transform::from_translation(position),
    GlobalTransform::default(),
    Visibility::default(),
    InheritedVisibility::default(),
    RigidBody::Dynamic,
    Collider::sphere(0.5),  // ← ADD THIS LINE
    LinearVelocity(debris_velocity),
    GravityScale(1.0),
    ExplosionEffect { lifetime: 0.0, max_lifetime: 5.0 },
));
```

**Why:** Avian3D requires all RigidBody entities to have a Collider. Without it, collider AABB creation fails.

---

### Case 2: NaN in Drone Movement

**Symptom:** Crash happens when drones move, especially at long distances from origin

**Root Cause:** Transform.forward() or position calculation produces NaN, likely due to invalid rotation or extreme coordinates

**Fix Location:** src/drone.rs, function move_drones() around line 60

**Implementation:**

Replace entire move_drones() function with:

```rust
/// Simple movement system: moves the drone forward based on its local rotation
fn move_drones(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Drone)>,
) {
    for (entity, mut transform, drone) in &mut query {
        // SAFETY: Validate transform state before processing
        if !transform.translation.is_finite() {
            eprintln!("⚠️ Drone {:?} has invalid translation! {:?}", entity, transform.translation);
            transform.translation = Vec3::new(0.0, 500.0, -200.0);
            continue;
        }

        if transform.rotation.is_nan() || !transform.rotation.is_normalized() {
            eprintln!("⚠️ Drone {:?} has invalid rotation! {:?}", entity, transform.rotation);
            transform.rotation = Quat::IDENTITY;
        }

        // Calculate forward vector with safety check
        let forward = transform.forward();
        if !forward.is_finite() {
            eprintln!("⚠️ Drone {:?} forward() produced invalid vector! {:?}", entity, forward);
            continue;
        }

        // Calculate movement vector
        let delta_secs = time.delta_secs();
        if !delta_secs.is_finite() || delta_secs <= 0.0 {
            eprintln!("⚠️ Invalid delta_secs: {}", delta_secs);
            continue;
        }

        let move_vec = forward * drone.speed * delta_secs;

        // Validate movement before applying
        if !move_vec.is_finite() {
            eprintln!("⚠️ Drone {:?} movement is invalid! speed={}, delta={}",
                     entity, drone.speed, delta_secs);
            continue;
        }

        // Apply movement
        let new_pos = transform.translation + move_vec;

        // Validate final position
        if !new_pos.is_finite() {
            eprintln!("⚠️ Drone {:?} new position is invalid! {:?}", entity, new_pos);
            continue;
        }

        // Clamp position to prevent extreme coordinates that could cause overflow
        if new_pos.length() > 100_000.0 {
            eprintln!("⚠️ Drone {:?} too far from origin! Resetting.", entity);
            transform.translation = Vec3::new(0.0, 500.0, -200.0);
            continue;
        }

        transform.translation = new_pos;

        // Log movement occasionally for debugging
        if time.elapsed_secs() as i32 % 5 == 0 && entity.index() % 2 == 0 {
             println!("DEBUG: Drone {:?} moving, pos: {:?}", entity, transform.translation);
        }
    }
}
```

**Key improvements:**
- Validates transform before forward() calculation
- Normalizes rotation if it's NaN
- Checks delta_secs validity
- Validates all intermediate calculations
- Clamps position to prevent extreme coordinates

---

### Case 3: NaN in Projectile Updates

**Symptom:** Crash happens when firing missiles or during projectile movement

**Root Cause:** Projectile velocity or position calculation produces NaN

**Fix Location:** src/main.rs, function update_projectiles() around line 2377

**Implementation:**

Find the projectile movement code and wrap with validation:

```rust
fn update_projectiles(
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &LinearVelocity), With<Projectile>>,
) {
    let delta = time.delta_secs();

    // SAFETY: Validate delta time
    if !delta.is_finite() || delta <= 0.0 {
        eprintln!("⚠️ Invalid delta_secs in update_projectiles: {}", delta);
        return;
    }

    for (entity, mut transform, velocity) in &mut projectile_query {
        // SAFETY: Validate projectile state
        if !transform.translation.is_finite() {
            eprintln!("⚠️ Projectile {:?} has invalid position! {:?}", entity, transform.translation);
            // This projectile should be despawned in next iteration
            continue;
        }

        if !velocity.0.is_finite() {
            eprintln!("⚠️ Projectile {:?} has invalid velocity! {:?}", entity, velocity.0);
            continue;
        }

        // Calculate new position
        let movement = velocity.0 * delta;

        // SAFETY: Validate movement vector
        if !movement.is_finite() {
            eprintln!("⚠️ Projectile {:?} movement is invalid! vel={:?}, delta={}",
                     entity, velocity.0, delta);
            continue;
        }

        let new_pos = transform.translation + movement;

        // SAFETY: Validate final position
        if !new_pos.is_finite() {
            eprintln!("⚠️ Projectile {:?} new position is invalid! {:?}", entity, new_pos);
            continue;
        }

        // SAFETY: Clamp position to reasonable bounds
        if new_pos.length() > 200_000.0 {
            eprintln!("⚠️ Projectile {:?} traveled too far! Despawning.", entity);
            continue; // Will be despawned in next section
        }

        transform.translation = new_pos;

        // Rest of projectile logic (collision checks, despawn, etc.)
        // ...
    }
}
```

---

### Case 4: NaN in Arcade Flight Physics

**Symptom:** Crash during normal flight, especially during fast maneuvers or extreme angles

**Root Cause:** Rotation calculations produce NaN when converting to/from Euler angles

**Fix Location:** src/main.rs, function arcade_flight_physics() around line 2000

**Current code already has checks, but may need to be more aggressive:**

```rust
// GET PITCH ANGLE - More defensive
let (_, pitch_angle, _) = transform.rotation.to_euler(EulerRot::XYZ);

// VALIDATE PITCH IMMEDIATELY
let safe_pitch = if pitch_angle.is_nan() || !pitch_angle.is_finite() {
    eprintln!("⚠️ Pitch angle is invalid! Resetting to 0.");
    0.0
} else {
    pitch_angle
};

// CLAMP PITCH to reasonable range (prevents Euler angle singularities)
let clamped_pitch = safe_pitch.clamp(-std::f32::consts::PI/2.0, std::f32::consts::PI/2.0);

// Use clamped pitch in all thrust calculations
let vertical_component = safe_throttle * MAX_THRUST_NEWTONS * clamped_pitch.sin();
let forward_component = safe_throttle * MAX_THRUST_NEWTONS * clamped_pitch.cos();
```

**Why:** Euler angles have singularities at ±π/2. Clamping prevents edge cases.

---

## Implementation Checklist

**Before you start:**
- [ ] Flash has reported the NaN source location
- [ ] You have the exact field and entity type with NaN
- [ ] You've read the corresponding "Case X" section above

**During implementation:**
- [ ] Make targeted fixes only - don't refactor
- [ ] Add console warnings with entity/field information
- [ ] Test each fix individually before combining
- [ ] Preserve existing functionality

**After implementation:**
- [ ] Build: `cargo build --release 2>&1 | tee build.log`
- [ ] Copy assets: `cp -r assets target/release/`
- [ ] Test: `target/release/plane_game.exe`
- [ ] Fly for 60+ seconds with drones spawned
- [ ] Fire missiles and destroy drones
- [ ] Watch console for `⚠️` warnings

---

## Testing Protocol

**Test 1: Basic Gameplay (2 minutes)**
```
1. Start game
2. Fly forward for 30 seconds
3. Watch for crashes or ⚠️ warnings
4. Expected: No crashes, no NaN warnings
```

**Test 2: Combat (2 minutes)**
```
1. Fly forward toward drones
2. Fire missile (Space)
3. Destroy 2-3 drones
4. Watch for crashes during explosion
5. Expected: Orange explosions, drones destroyed, no crash
```

**Test 3: Extended Flight (3 minutes)**
```
1. Fly in all directions (avoid ground)
2. Fire missiles occasionally
3. Destroy more drones if possible
4. Expected: Stable FPS, no crashes
```

**Total test time:** ~7 minutes

---

## What to Do If Still Crashing

1. **Check build output** for compilation warnings
2. **Review console log** for `⚠️` messages
3. **Note exact conditions** when crash happens:
   - Right after explosion?
   - During missile fire?
   - After flying X seconds?
   - At specific location?
4. **Report to Flash** with exact console output for deeper investigation

---

## Imports Needed

Make sure these are at top of src/main.rs (likely already present):
```rust
use bevy::prelude::*;
use avian3d::prelude::*;
```

No new imports needed for this fix.

---

## Success Criteria

✅ Code compiles without errors
✅ Game launches without immediate crash
✅ Can fly for 60+ seconds continuously
✅ Drones spawn and move without crash
✅ Missiles fire and hit drones
✅ Explosions spawn and despawn cleanly
✅ No Vulkan validation errors during gameplay
✅ No `🚨 EARLY:` NaN warnings in console
✅ Console shows normal debug messages (drone movement, etc.)

---

## Code Reference

| File | Lines | Component |
|------|-------|-----------|
| src/main.rs | 2767-2778 | spawn_huge_explosion() - debris spawn |
| src/main.rs | 2377-? | update_projectiles() - movement logic |
| src/main.rs | 2000-2081 | arcade_flight_physics() - thrust/rotation |
| src/drone.rs | 60-74 | move_drones() - drone movement |
| src/main.rs | 489 | System registration (physics order) |

---

**You'll know you succeeded when:** The user can fly around, shoot drones, create explosions, and the game doesn't crash for 5+ minutes of continuous play.

