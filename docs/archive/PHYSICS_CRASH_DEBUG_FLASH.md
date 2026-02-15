# GEMINI FLASH: Physics Crash Debugging - Avian3D AABB Assertion Failure

**Priority:** CRITICAL (Game-breaking crash)
**Status:** Ready to diagnose and fix
**Error:** `assertion failed: b.min.cmple(b.max).all()` in avian3d collider

---

## The Problem

Game crashes with Vulkan semaphore validation error + avian3d AABB assertion after flying for a bit. The assertion failure means a collider's bounding box has `min > max` in at least one dimension, which only happens when NaN or Infinity gets into physics calculations.

---

## What We Know

### Current NaN Safety (Already In Place)
- **check_ground_collision()** at src/main.rs:2643 has NaN checks
- **safety_check_nan()** at src/main.rs:1078 monitors all physics entities
- **arcade_flight_physics()** at src/main.rs:2000 checks many calculations

BUT: These checks run AFTER physics engine has already tried to validate colliders. We need to catch NaN EARLIER.

---

## Your Debugging Mission

### STEP 1: Find the NaN Source (30 minutes)

Create a new aggressive NaN detection system that logs BEFORE physics runs:

**File:** src/main.rs

**Add this new system function** (around line 1070, before safety_check_nan):

```rust
/// AGGRESSIVE: Catch NaN before it reaches physics engine
fn detect_nan_early(
    mut player_query: Query<(
        Entity,
        &Transform,
        &LinearVelocity,
        &AngularVelocity,
        &ExternalForce,
    ), With<PlayerPlane>>,
    projectiles: Query<(&Transform, &LinearVelocity), With<Projectile>>,
    drones: Query<(&Transform, &Drone)>,
) {
    // Check PLAYER
    if let Ok((entity, transform, lin_vel, ang_vel, ext_force)) = player_query.get_single() {
        // Transform check
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("🚨 EARLY: Player Transform.translation has NaN! {:?}", transform.translation);
        }
        if transform.scale.is_nan() || !transform.scale.is_finite() {
            eprintln!("🚨 EARLY: Player Transform.scale has NaN! {:?}", transform.scale);
        }
        if transform.rotation.is_nan() {
            eprintln!("🚨 EARLY: Player Rotation has NaN!");
        }

        // LinearVelocity check
        if lin_vel.0.is_nan() || !lin_vel.0.is_finite() {
            eprintln!("🚨 EARLY: Player LinearVelocity has NaN! {:?}", lin_vel.0);
        }

        // AngularVelocity check
        if ang_vel.0.is_nan() || !ang_vel.0.is_finite() {
            eprintln!("🚨 EARLY: Player AngularVelocity has NaN! {:?}", ang_vel.0);
        }

        // ExternalForce check
        if ext_force.force.is_nan() || !ext_force.force.is_finite() {
            eprintln!("🚨 EARLY: Player ExternalForce has NaN! {:?}", ext_force.force);
        }

        // Check collider bounds (derived from scale and local collider size)
        let collider_half_extents = Vec3::new(2.0, 1.0, 4.0); // Match Collider::cuboid(2.0, 1.0, 4.0)
        let scaled_extents = collider_half_extents * transform.scale;
        if scaled_extents.is_nan() || !scaled_extents.is_finite() {
            eprintln!("🚨 EARLY: Collider extents would be NaN! scale={:?}, extents={:?}",
                     transform.scale, scaled_extents);
        }
    }

    // Check PROJECTILES
    for (transform, velocity) in &projectiles {
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("🚨 EARLY: Projectile position has NaN! {:?}", transform.translation);
        }
        if velocity.0.is_nan() || !velocity.0.is_finite() {
            eprintln!("🚨 EARLY: Projectile velocity has NaN! {:?}", velocity.0);
        }
    }

    // Check DRONES
    for (transform, _drone) in &drones {
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("🚨 EARLY: Drone position has NaN! {:?}", transform.translation);
        }
    }
}
```

**Add to system registration** (around line 489, FIRST in the list before physics):

```rust
.add_systems(Update, (
    detect_nan_early,  // ← ADD THIS FIRST
    handle_quit,
    handle_restart,
    // ... rest of systems
))
```

**Why this matters:** This will print WHICH entity and WHICH field has NaN before the crash.

---

### STEP 2: Check Explosion Debris Physics (15 minutes)

The explosion spawning code (around src/main.rs:2774) spawns debris with RigidBody but NO Collider.

**Issue:** Avian3D expects all RigidBody entities to have Colliders.

**Fix in spawn_huge_explosion()** (around line 2774):

**BEFORE (broken):**
```rust
commands.spawn((
    Mesh3d(debris_mesh),
    MeshMaterial3d(debris_material),
    Transform::from_translation(position),
    GlobalTransform::default(),
    Visibility::default(),
    InheritedVisibility::default(),
    RigidBody::Dynamic,  // ← Physics body
    LinearVelocity(debris_velocity),
    GravityScale(1.0),
    ExplosionEffect { lifetime: 0.0, max_lifetime: 5.0 },
));
```

**AFTER (fixed):**
```rust
commands.spawn((
    Mesh3d(debris_mesh),
    MeshMaterial3d(debris_material),
    Transform::from_translation(position),
    GlobalTransform::default(),
    Visibility::default(),
    InheritedVisibility::default(),
    RigidBody::Dynamic,
    Collider::sphere(0.5),  // ← ADD THIS
    LinearVelocity(debris_velocity),
    GravityScale(1.0),
    ExplosionEffect { lifetime: 0.0, max_lifetime: 5.0 },
));
```

---

### STEP 3: Validate Drone Movement (20 minutes)

The drone movement system in **src/drone.rs:60-74** directly modifies Transform without validation.

**Add NaN checks** to move_drones() function:

**BEFORE:**
```rust
fn move_drones(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Drone)>,
) {
    for (entity, mut transform, drone) in &mut query {
        let forward = transform.forward();
        let move_vec = forward * drone.speed * time.delta_secs();
        transform.translation += move_vec;

        // Log movement occasionally for debugging
        if time.elapsed_secs() as i32 % 5 == 0 && entity.index() % 2 == 0 {
             println!("DEBUG: Drone {:?} moving, pos: {:?}", entity, transform.translation);
        }
    }
}
```

**AFTER (with validation):**
```rust
fn move_drones(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Drone)>,
) {
    for (entity, mut transform, drone) in &mut query {
        // SAFETY: Check transform is valid before movement
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("⚠️ Drone {:?} has invalid translation! Resetting.", entity);
            transform.translation = Vec3::new(0.0, 500.0, -200.0);
            continue;
        }

        let forward = transform.forward();

        // SAFETY: Check forward vector is valid
        if forward.is_nan() || !forward.is_finite() {
            eprintln!("⚠️ Drone {:?} has invalid forward vector! {:?}", entity, forward);
            continue;
        }

        let move_vec = forward * drone.speed * time.delta_secs();

        // SAFETY: Check movement vector is valid
        if move_vec.is_nan() || !move_vec.is_finite() {
            eprintln!("⚠️ Drone {:?} calculated invalid movement! speed={}, delta={:?}",
                     entity, drone.speed, time.delta_secs());
            continue;
        }

        transform.translation += move_vec;

        // SAFETY: Validate final position
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("⚠️ Drone {:?} position became NaN after movement! {:?}", entity, transform.translation);
            transform.translation = Vec3::new(0.0, 500.0, -200.0);
            continue;
        }

        // Log movement occasionally for debugging
        if time.elapsed_secs() as i32 % 5 == 0 && entity.index() % 2 == 0 {
             println!("DEBUG: Drone {:?} moving, pos: {:?}", entity, transform.translation);
        }
    }
}
```

---

### STEP 4: Validate Projectile Updates (15 minutes)

Find the **update_projectiles()** function (src/main.rs:2377) and add similar NaN checks:

**Add validation:**
```rust
fn update_projectiles(
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &LinearVelocity), With<Projectile>>,
) {
    for (entity, mut transform, velocity) in &mut projectile_query {
        // SAFETY: Validate starting state
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("⚠️ Projectile {:?} has invalid position! {:?}", entity, transform.translation);
            // Despawn corrupted projectile
            // (handled in next section)
            continue;
        }

        let movement = velocity.0 * time.delta_secs();

        // SAFETY: Check movement
        if movement.is_nan() || !movement.is_finite() {
            eprintln!("⚠️ Projectile {:?} movement is NaN! velocity={:?}", entity, velocity.0);
            continue;
        }

        transform.translation += movement;

        // SAFETY: Validate final position
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("⚠️ Projectile {:?} position became NaN! {:?}", entity, transform.translation);
            // Position might be corrupted - should be despawned
            continue;
        }

        // Rest of projectile update logic
        // ...
    }
}
```

---

### STEP 5: Test & Collect Logs (30 minutes)

**Build and run:**
```bash
cd /c/Users/Box/plane_game
cargo build --release 2>&1 | tee build.log

# If build succeeds:
cp -r assets target/release/
target/release/plane_game.exe 2>&1 | tee gameplay.log
```

**In game:**
1. Spawn at origin
2. Fly forward for 30+ seconds
3. Fire missiles and destroy drones
4. Watch for console messages:
   - If you see `🚨 EARLY:` messages, that's the NaN source
   - Check gameplay.log for patterns

**Analyze logs for:**
- Which entity has NaN (player/projectile/drone)?
- Which field is NaN (position/velocity/rotation)?
- When does it happen (after how many frames/impacts)?

---

## If You Find NaN, Report:

```
🚨 FOUND NaN:
- Entity type: [Player/Projectile/Drone/Debris]
- Field: [Translation/Velocity/Rotation/Scale]
- Value: [what it was]
- When: [after X impacts/during drone movement/at chunk load]
```

Then send this to Pro or Claude for the fix.

---

## Critical Code Sections to Check

| Section | File | Lines | Purpose |
|---------|------|-------|---------|
| Explosion debris spawn | src/main.rs | 2767-2778 | Add Collider to physics debris |
| Drone movement | src/drone.rs | 60-74 | Add NaN validation before/after movement |
| Projectile updates | src/main.rs | 2377-? | Add NaN validation to position updates |
| Early NaN detection | src/main.rs | ~1070 | NEW: Add detect_nan_early() system |

---

## Success Criteria

✅ Game compiles without errors
✅ Early NaN detection system runs (no crashes on startup)
✅ Can fly for 60+ seconds without crash
✅ Can fire missiles and destroy drones without crash
✅ If crash occurs, console shows `🚨 EARLY:` message with NaN location
✅ No Vulkan semaphore validation errors during normal gameplay

---

**Next step:** If you successfully identify the NaN source and build succeeds, test for 2+ minutes of gameplay. If still crashes, report the 🚨 EARLY message location and we'll escalate to Pro.

