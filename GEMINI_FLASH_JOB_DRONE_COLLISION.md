# GEMINI FLASH JOB #5: Debug Drone Loading + Implement Collision System

**Priority:** High (drones not interactive)
**Estimated Time:** 1-2 hours
**Blockers:** None (parallel to textures)
**Goal:** Make drones visible AND shootable

---

## The Problem

**Current State:**
- Red test cube spawns (good - positioning works)
- Red cube flies away at 40 m/s (movement works)
- But **drone.glb NOT loading** (should see drone model, not cube)
- Red cube disappears instantly (can't interact with it)
- **Need:** Drone visible AND collision system to shoot it

---

## Two-Part Job

### PART 1: Debug Drone Asset Loading

**Why drone.glb isn't showing:**

The red cube spawning means `spawn_beaver_drone()` runs, but drone.glb isn't loading.

**Possible causes:**
1. `assets/models/drone.glb` file doesn't exist
2. Asset path in code is wrong
3. SceneRoot isn't loading .glb properly (same issue as trees had)
4. Asset loading timing issue

**Investigation Steps:**

1. **Verify file exists:**
```bash
dir C:\Users\Box\plane_game\assets\models\
# Should show: drone.glb
```

If drone.glb missing:
```bash
# Check if it's in another location
dir C:\Users\Box\plane_game\assets\
# Or in downloads
dir E:\Downloads\*drone*
```

2. **Check asset path in code:**
In `src/drone.rs`, find `spawn_beaver_drone()` function (around line 15-30)

Look for:
```rust
SceneRoot(asset_server.load("models/drone.glb")),
```

**Issues to fix:**
- Path should be relative to `assets/` folder
- Should be `"models/drone.glb"` NOT `"assets/models/drone.glb"`
- Case sensitive (drone.glb vs DRONE.GLB)

3. **If SceneRoot broken:**
If drone.glb exists but still doesn't render, SceneRoot issue again.

Replace with fallback colored mesh:
```rust
// In spawn_beaver_drone, instead of SceneRoot:
commands.spawn((
    Mesh3d(meshes.add(Mesh::from(Cuboid::new(3.0, 2.0, 8.0)))),  // Drone-like shape
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),  // Dark gray drone
        emissive: LinearRgba::rgb(0.3, 0.3, 0.3),
        ..default()
    })),
    Transform {
        translation: position,
        scale: Vec3::splat(1.8),
        rotation: Quat::from_rotation_y(std::f32::consts::PI),
    },
    Drone { health: 50.0, speed: 40.0 },
    KamikazeBehavior,
));
```

This creates a dark gray drone-shaped box as fallback.

---

### PART 2: Implement Drone Collision/Health System

**Goal:** Make drones shootable (Space key = fire missile)

**Implementation:**

1. **Add damage system to Drone component:**

In `src/drone.rs`, the Drone struct should have health:
```rust
#[derive(Component)]
pub struct Drone {
    pub health: f32,
    pub speed: f32,
}
```

(Should already exist, verify it does)

2. **Create collision detection system:**

In `src/main.rs`, add new system:
```rust
fn drone_projectile_collision(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    mut drones: Query<(Entity, &mut Drone, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (proj_entity, proj_pos) in &projectiles {
        for (drone_entity, mut drone, drone_pos) in &mut drones {
            // Distance between projectile and drone
            let distance = proj_pos.translation.distance(drone_pos.translation);

            // Hit radius: 50m (generous for testing)
            if distance < 50.0 {
                println!("💥 HIT DRONE! Distance: {:.1}m", distance);

                // Deal damage
                drone.health -= 25.0;
                println!("🎯 Drone health: {:.1}", drone.health);

                // Despawn projectile
                commands.entity(proj_entity).despawn();

                // If drone dies
                if drone.health <= 0.0 {
                    println!("💀 DRONE DESTROYED!");

                    // Spawn explosion at drone location (simple: bright sphere)
                    commands.spawn((
                        Mesh3d(meshes.add(Mesh::from(Sphere { radius: 30.0 }))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 0.5, 0.0),  // Orange explosion
                            emissive: LinearRgba::rgb(5.0, 2.0, 0.0),
                            ..default()
                        })),
                        Transform::from_translation(drone_pos.translation),
                        // Despawn after 0.5 seconds (TODO: add timer)
                    ));

                    // Despawn drone
                    commands.entity(drone_entity).despawn();
                }
            }
        }
    }
}
```

3. **Register the system:**

In main function, add to app builder (near where other systems registered):
```rust
.add_systems(Update, drone_projectile_collision)
```

Place it AFTER projectile systems but before physics update.

4. **Add visual feedback:**

When you hit a drone, console prints:
- 💥 HIT DRONE! Distance: 45.2m
- 🎯 Drone health: 25.0
- 💀 DRONE DESTROYED!

When drone explodes:
- Orange glowing sphere spawns at drone location
- Drone disappears

---

## Testing Procedure

```bash
# Build
cd C:\Users\Box\plane_game
cargo build --release

# Copy assets
cp -r assets target/release/

# Run
target/release/plane_game.exe
```

**In Game:**
1. Spawn, look forward
2. See 3 drones (or red cubes if .glb broken)
3. Try to follow a drone
4. Press Space to fire missile
5. Watch console for collision messages
6. Try to hit drone with missile
7. When hit: see explosion orange sphere spawn
8. Drone disappears

---

## Expected Console Output (When Working)

```
💥 HIT DRONE! Distance: 45.2m
🎯 Drone health: 25.0
💥 HIT DRONE! Distance: 38.1m
🎯 Drone health: 0.0
💀 DRONE DESTROYED!
```

---

## Success Criteria

✅ Drones visible (either real model or gray mesh)
✅ Can fire missiles with Space key
✅ Missiles hit drones (console says "HIT DRONE")
✅ Drones take damage (health decreases)
✅ Drone explodes when health = 0 (orange sphere)
✅ Explosion fades/disappears
✅ FPS 60+

---

## Deliverables

When complete:

1. **Drone visibility:** Is it drone.glb or fallback mesh?
2. **Asset path:** Show console output (any "missing asset" errors?)
3. **Code changes:** Show drone_projectile_collision function
4. **System registration:** Show where system added to app
5. **Test results:** Did missiles hit drones? Did they die?
6. **Screenshot:** Drone + explosion visible in game
7. **Issues:** Any problems? How fixed?
8. **FPS:** Stable at 60+?

---

## Why This Matters

This is the **first interactive combat mechanic**. You can now:
- See drones
- Shoot them
- Get feedback (explosion)
- Feel like you're actually in combat

Next will be: Swarm AI (drones fly in formation) + Kamikaze AI (drones chase you)

---

**Time Estimate:** 1-2 hours (mostly debugging asset loading, then adding collision system)
**Complexity:** Medium (straightforward collision math, system registration straightforward)
**Risk:** Low (doesn't affect other systems)

