# GEMINI FLASH JOB #4: Fix Drone Spawning - Make Them Visible

**Priority:** High (can't test drones if not visible)
**Estimated Time:** 30 minutes
**Blocker:** No (but needed before combat testing)
**Goal:** Spawn drones near player so they're visible and testable

---

## The Problem

**Current Situation:**
- Drones spawning at (100, 200, -500) and (-100, 200, -500)
- That's 500m directly ahead of player
- Drones move forward at 40 m/s
- Player can't easily see them (too far, moving away)
- Can't verify drone models are loading
- Can't test if they can be shot

**Solution:**
Spawn drones much closer to player in visible formations:
1. **One drone:** 200m directly ahead (chase target)
2. **Two drones:** 150m to left and right (flanking formation)
3. **All near camera:** Easy to see and aim at

---

## Implementation

### Option A: Simple Close Formation (RECOMMENDED)

Modify drone spawning in `src/main.rs` - find where `spawn_beaver_drone()` is called (likely in `setup_scene()`):

**OLD CODE:**
```rust
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(100.0, 200.0, -500.0));
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(-100.0, 200.0, -500.0));
```

**NEW CODE (Close formation):**
```rust
// Spawn 3 test drones in visible formation near player
// Player starts at (0, 500, 0) facing forward (-Z)

// Drone 1: Directly ahead, slightly above (chase target)
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(0.0, 520.0, -200.0));

// Drone 2: Left side, slightly behind and above (flanking)
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(-150.0, 500.0, -100.0));

// Drone 3: Right side, slightly behind and above (flanking)
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(150.0, 500.0, -100.0));
```

**Why these positions:**
- -200m to -100m ahead (in view range)
- 520m-500m altitude (slightly above/level with player at 500m)
- ±150m left/right (visible but not blocking view)
- Spaced so you can see all 3 easily

---

### Option B: Single Drone for Testing (FASTEST)

If you just want ONE drone to verify it works:

```rust
// Single test drone directly ahead, visible
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(0.0, 510.0, -150.0));
```

Then add more later once you confirm it's working.

---

### Option C: Dynamic Formation (ADVANCED)

Create a function that spawns drones in different patterns:

```rust
fn spawn_test_drones(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let player_pos = Vec3::new(0.0, 500.0, 0.0);
    let forward = Vec3::new(0.0, 0.0, -1.0);  // -Z is forward

    // Drone 1: 200m ahead, slightly above
    let pos1 = player_pos + forward * 200.0 + Vec3::new(0.0, 20.0, 0.0);
    spawn_beaver_drone(commands, asset_server, pos1);

    // Drone 2: 150m ahead, left flank
    let pos2 = player_pos + forward * 150.0 + Vec3::new(-150.0, 0.0, 0.0);
    spawn_beaver_drone(commands, asset_server, pos2);

    // Drone 3: 150m ahead, right flank
    let pos3 = player_pos + forward * 150.0 + Vec3::new(150.0, 0.0, 0.0);
    spawn_beaver_drone(commands, asset_server, pos3);
}

// Then call this in setup_scene():
spawn_test_drones(&mut commands, &asset_server);
```

---

## Testing Procedure

**After making changes:**

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
1. Spawn at origin (0, 500, 0)
2. Look forward (-Z direction)
3. **Should immediately see 1-3 drones ahead**
4. Note their appearance:
   - ✅ Can you see the drone model?
   - ✅ What do they look like?
   - ✅ Are they moving?
   - ✅ Can you get closer to them?
5. Try following one drone
6. Observe movement behavior

---

## Verification Checklist

After spawning near player:

- [ ] Drones visible immediately on game start
- [ ] Can see all 3 (or 1) drones in viewport
- [ ] Drones are moving forward smoothly
- [ ] Can chase/follow them with plane
- [ ] Can see drone model details
- [ ] Drones don't clip through plane or terrain
- [ ] FPS stays 60+
- [ ] No console errors about missing assets

---

## Answering Your Questions

### Q: "Can I shoot them down once missile system is fixed?"

**Current Status of Missile System:**
- ✅ Space key fires (implemented)
- ✅ Creates sprite projectiles (visible orange particles)
- ⚠️ Missile collision with drones: **NOT YET IMPLEMENTED**

**What needs to happen for drones to be shootable:**
1. Add collision detection: Projectile → Drone
2. Subtract health from drone on hit
3. Despawn drone when health = 0
4. Spawn explosion effect on death

**Code location (to be added later):**
```rust
fn handle_projectile_drone_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    mut drones: Query<(Entity, &mut Drone, &Transform)>,
) {
    for (proj_entity, proj_transform) in &projectiles {
        for (drone_entity, mut drone, drone_transform) in &mut drones {
            let distance = proj_transform.translation.distance(drone_transform.translation);

            // If close enough, drone takes damage
            if distance < 20.0 {  // 20m hit radius
                drone.health -= 25.0;  // Deal 25 damage per hit
                commands.entity(proj_entity).despawn();  // Projectile explodes

                // If drone dies
                if drone.health <= 0.0 {
                    commands.entity(drone_entity).despawn();
                    // TODO: Spawn explosion effect
                }
            }
        }
    }
}
```

**When will this be implemented?**
- After drones are visible and moving correctly
- Part of Phase 3 Combat implementation
- Estimated: 1-2 hours after drone AI working

### Q: "Are they flying etc?"

**Current Drone Behavior:**
- Move forward in local space at constant 40 m/s
- No weaving or maneuvering yet
- No targeting or pursuit
- Just linear movement

**What's coming (Phase 3 AI):**
- Swarm formations (circles around leader)
- Kamikaze pursuit (chase player)
- Evasive weaving
- Collision with projectiles

---

## Expected Appearance

**What drone model looks like:**
- Filename: `80_followers_iranian_shahed-136_drone.glb`
- Type: Military reconnaissance/attack drone
- Should look like a quadcopter/fixed-wing hybrid
- Scaled to 1.8x (from spawn_beaver_drone)
- Dark colored (not bright)

**If drone doesn't appear:**
- Check if `assets/models/drone.glb` exists
- Verify asset loading in console output
- May need to check scale (1.8x might be too small/large)
- Could adjust scale in spawn function if needed

---

## Quick Win Option

If you want to see results IMMEDIATELY:

Replace the drone spawn with visible test geometry:

```rust
// Quick test: spawn visible cube instead of drone
commands.spawn((
    Mesh3d(meshes.add(Mesh::from(Cuboid {
        half_size: Vec3::splat(20.0),  // 40m cube (visible!)
    }))),
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),  // Bright red
        emissive: Color::srgb(0.5, 0.0, 0.0),
        ..default()
    })),
    Transform::from_translation(Vec3::new(0.0, 510.0, -150.0)),
    Drone { health: 50.0, speed: 40.0 },
    KamikazeBehavior,
));
```

This spawns a big red cube so you can:
- See drone position clearly
- Test movement
- Test collision/shooting once that's implemented

Then swap back to real drone.glb model later.

---

## Deliverables

When done, provide:

1. ✅ **Code changes:** Show the modified drone spawn calls
2. ✅ **Test results:** Did drones appear close to player?
3. ✅ **Description:** What do the drones look like?
4. ✅ **Movement:** Are they moving forward?
5. ✅ **Screenshot:** Optional - show drones in viewport
6. ✅ **Issues:** Any problems? How fixed?
7. ✅ **Next recommendation:** What should be worked on next?

---

## Success Criteria

✅ Drones spawn very close to player (within 200m)
✅ All drones visible on game start
✅ Can see drone model(s) clearly
✅ Drones moving smoothly forward
✅ Can chase/follow drones with plane
✅ No console errors
✅ FPS 60+
✅ Ready to test missile/shooting system next

---

**Time Estimate:** 30 minutes (mostly testing)
**Complexity:** Simple (just change spawn coordinates)
**Risk:** None (doesn't affect other systems)

