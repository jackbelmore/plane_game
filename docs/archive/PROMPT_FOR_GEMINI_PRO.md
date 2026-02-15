# GEMINI PRO JOB: Implement Drone Combat System

**Priority:** HIGH (Core gameplay feature)
**Estimated Time:** 2-3 hours
**Status:** Ready to start
**Blocker:** None (drones already spawning)

---

## The Goal

Make drones **shootable and destructible**. When the player fires a missile (Space key), hitting a drone should:
1. Deal damage to the drone
2. Destroy the drone when health reaches 0
3. Spawn an explosion effect
4. Show console feedback for debugging

---

## Current State

✅ Drones spawn in formation (200m ahead)
✅ Red test cubes are visible (collision will work)
✅ Missile firing works (Space key creates projectiles)
❌ No collision detection between missiles and drones
❌ Drones have no health system yet
❌ No explosion effects on destruction

---

## What You Need to Implement

### PART 1: Ensure Drone Has Health (5 minutes)

**File:** `src/drone.rs`

Check if the Drone component has a `health` field. Should look like:

```rust
#[derive(Component)]
pub struct Drone {
    pub health: f32,
    pub speed: f32,
}
```

If `health` field is missing, add it. If health is already there, skip this step.

---

### PART 2: Create Collision Detection System (1-2 hours)

**File:** `src/main.rs`

Create a new system function to detect when projectiles hit drones:

```rust
fn drone_projectile_collision(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    mut drones: Query<(Entity, &mut Drone, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Collision detection loop
    for (proj_entity, proj_transform) in &projectiles {
        for (drone_entity, mut drone, drone_transform) in &mut drones {
            // Calculate distance between projectile and drone
            let distance = proj_transform.translation.distance(drone_transform.translation);

            // Hit radius: 50m (generous for testing)
            if distance < 50.0 {
                println!("💥 HIT DRONE! Distance: {:.1}m", distance);

                // Deal 25 damage per hit
                drone.health -= 25.0;
                println!("🎯 Drone health: {:.1}", drone.health);

                // Despawn the projectile (it exploded)
                commands.entity(proj_entity).despawn();

                // Check if drone died
                if drone.health <= 0.0 {
                    println!("💀 DRONE DESTROYED!");

                    // Spawn explosion effect
                    commands.spawn((
                        Mesh3d(meshes.add(Mesh::from(Sphere { radius: 30.0 }))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 0.5, 0.0),  // Orange
                            emissive: LinearRgba::rgb(5.0, 2.0, 0.0),  // Glow
                            ..default()
                        })),
                        Transform::from_translation(drone_transform.translation),
                    ));

                    // Despawn the drone
                    commands.entity(drone_entity).despawn();
                }
            }
        }
    }
}
```

**Key points:**
- Distance check: 50m hit radius (adjust if needed)
- Damage: 25 HP per hit (4 hits to kill drones with 100 HP)
- Explosion: Orange glowing sphere at drone location
- Console messages for debugging

---

### PART 3: Register the System (5 minutes)

**File:** `src/main.rs`

Find the line with `add_systems(Update, (` around line 512.

Add `drone_projectile_collision` to the system list:

```rust
.add_systems(Update, (
    handle_quit,
    handle_restart,
    debug_asset_loading,
    debug_tree_hierarchy,
    handle_shooting_input,
    update_projectiles,
    handle_projectile_collisions,
    drone_projectile_collision,  // ← ADD THIS LINE
    update_muzzle_flashes,
    update_explosion_effects,
))
```

**Important:** Add it AFTER `update_projectiles` so projectiles exist before collision check.

---

## Testing Procedure

```bash
# Build
cd /c/Users/Box/plane_game
cargo build --release

# Copy assets
cp -r assets target/release/

# Run
target/release/plane_game.exe
```

**In Game:**
1. Spawn at origin (0, 500, 0)
2. Look forward (you should see 3 red drones ~200m away)
3. Press Space to fire missile
4. Try to hit a drone
5. Watch console for collision messages:
   - `💥 HIT DRONE! Distance: 45.2m`
   - `🎯 Drone health: 75.0`
   - `💀 DRONE DESTROYED!`

**Visual feedback:**
- When drone dies, orange explosion sphere spawns at drone location
- Drone disappears
- Explosion sphere lasts briefly then despawns

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| No "HIT DRONE" messages | Hit radius might be too small (try 75m), or projectiles despawning too fast |
| Drones don't die | Check drone.health is decreasing (see console), might need more hits |
| No explosion | Check meshes/materials resources are available in function signature |
| Compilation error | Verify `Drone` struct has `health` field, check import for `Sphere` |

---

## Dependencies & Imports

Make sure these are imported at top of src/main.rs:
```rust
use bevy::prelude::*;
// Should already have:
// - Transform
// - Sphere
// - StandardMaterial
// - LinearRgba
```

---

## Success Criteria

✅ Projectiles hit drones (see distance in console)
✅ Drone health decreases on hit
✅ Drones explode when health = 0
✅ Orange explosion sphere visible
✅ Drone despawns after death
✅ Console shows all events
✅ Can fire multiple missiles in sequence
✅ Multiple drones can be damaged/destroyed
✅ FPS stays 60+ during collisions
✅ No crashes or panics

---

## Next Steps

After this is working:
1. Implement swarm formation AI (drones fly in patterns)
2. Implement kamikaze pursuit (drones chase player)
3. Add weapon variety (different missile types)
4. Add kill counter UI

---

## Code Location Reference

- **Drone struct:** `src/drone.rs` line ~20
- **Projectile struct:** `src/main.rs` line ~60 (find `struct Projectile`)
- **System registration:** `src/main.rs` line ~512
- **Sphere mesh example:** `src/main.rs` line ~143 (find `Sphere::new`)

---

**Ready to start?** Build and test, and let me know if you hit any issues!
