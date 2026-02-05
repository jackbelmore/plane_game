# Phase 3 Implementation Prompts - Combat System

**Status:** Phase 3 NOT STARTED (Waiting on drone 3D models)  
**Effort:** ~12 hours (2-3 days of focused work)  
**Blocker:** User must provide drone 3D model file(s) first

---

## 📋 Prerequisites

Before starting Phase 3, you need:

1. ✅ Phase 1 & 2 complete (chunk world + rocket mode)
2. ✅ Game running at 60 FPS (verified)
3. 🚫 **BLOCKING:** Drone 3D model file(s) in `.glb` or `.gltf` format

**Ask User:** "Can you provide the drone 3D models? Save to: `assets/models/drone.glb`"

---

## PROMPT 1: Gemini - Phase 3 Architecture Design (Planning)

**When User is Ready:** After drone models provided

```
Design the Phase 3 drone enemy system architecture for the flight simulator.

REQUIREMENTS:
1. Create Drone component and enum for AI variants (Swarm vs Kamikaze)
2. Design drone spawning logic:
   - 10-20 drones per chunk (same as trees)
   - 60% swarm AI, 40% kamikaze AI
   - Spawn in chunks around player (not in villages)
   - Deterministic seeding (use same pattern as trees)

3. Swarm AI behavior:
   - Groups of 3-5 drones
   - Leader-follower formation
   - Weaving flight pattern
   - Periodic firing at player
   - Fire rate: 1 shot per 2 seconds

4. Kamikaze AI behavior:
   - Direct pursuit of player
   - Accelerate toward target
   - Explode on contact or when destroyed
   - Spawn health: 50 HP
   - Player bullets: 10 damage each

5. Drone specs:
   - Health: 50 HP (5 bullets to kill)
   - Speed: 150 m/s (slower than player)
   - Fire rate: 1 shot/2 seconds
   - Detection range: 2km
   - Explosion radius: 100m damage zone

6. Integration points:
   - Drone spawning in manage_chunks() or separate spawn_drones_in_chunk()
   - Drone despawning when chunk unloads
   - Collision with player bullets
   - Explosion effect when destroyed
   - Damage system (player takes damage from collision/explosions)

Create a detailed design document with:
- Exact component definitions (with fields)
- System function signatures
- How drones interact with existing player/bullet systems
- Performance considerations (100+ drones at once)

This is planning/design only - don't write code yet.
```

---

## PROMPT 2: Gemini - Implement Drone Components & Spawning (Coding)

**After:** PROMPT 1 design approved

```
Implement Phase 3 Part 1: Drone components and spawning system.

FILE: src/main.rs

PART A: Add Drone Components (after RocketMode component, ~line 180):

1. Drone marker component:
   #[derive(Component)]
   struct Drone;

2. Drone AI enum:
   #[derive(Component)]
   enum DroneAI {
       Swarm { leader: Entity, formation_offset: Vec3 },
       Kamikaze { target: Entity },
   }

3. Drone health component:
   #[derive(Component)]
   struct DroneHealth(f32);

4. Constants:
   const DRONE_HEALTH: f32 = 50.0;
   const DRONE_SPEED: f32 = 150.0;
   const DRONES_PER_CHUNK_MIN: usize = 5;
   const DRONES_PER_CHUNK_MAX: usize = 15;

PART B: Add spawn_drones_in_chunk() function (after spawn_trees_in_chunk, ~line 700):

Function should:
1. Accept chunk_coord, chunk_pos like tree spawning
2. Use seeded RNG with chunk coordinates (same pattern as trees)
3. Randomly decide Swarm (60%) vs Kamikaze (40%)
4. For Swarm: Create 3-5 drones, pick leader, set formation offsets
5. For Kamikaze: Create single drone, set pursuit target (player if within 2km)
6. Spawn drones with:
   - SceneRoot(drone_model)
   - Transform (random position in chunk)
   - RigidBody::Dynamic
   - LinearVelocity (starting velocity)
   - Collider::sphere(5.0) (approximate size)
   - DroneHealth(50.0)
   - DroneAI variant
   - ChunkEntity tag (so they unload with chunk)

PART C: Integrate into manage_chunks() (modify around line 620):

Add after spawn_trees_in_chunk() call:
```rust
spawn_drones_in_chunk(
    &mut commands,
    &asset_server,
    chunk_coord,
    chunk_pos,
    // Pass player position for kamikaze detection:
    player_transform.translation,
);
```

PART D: Add drone model asset variable (in spawn_player or setup, ~line 400):

Add to SoundAssets or create new DronesAssets resource:
```rust
#[derive(Resource)]
struct DroneAssets {
    drone_model: Handle<Scene>,
}
```

TEST: After implementation:
- [ ] Compile without errors
- [ ] Game starts and chunks load
- [ ] Drones visible in loaded chunks (~10-15 per chunk)
- [ ] Drones have random spawn positions within chunks
- [ ] Drones persist while chunk is loaded

EXPECTED OUTPUT:
- Drones spawning but not moving (AI systems next phase)
- Drone models visible at chunk load
- No physics crashes
```

---

## PROMPT 3: Gemini - Implement Swarm AI System (Coding)

**After:** PROMPT 2 completes (drones spawn)

```
Implement Phase 3 Part 2: Swarm AI formation flying.

FILE: src/main.rs

ADD NEW SYSTEM: update_swarm_ai() (after arcade_flight_physics, ~line 1450):

The system should:

1. Query all swarm drones with:
   - Transform
   - LinearVelocity
   - DroneAI (for leader detection)
   - ExternalForce

2. Query player Transform for reference

3. For each swarm drone:
   a) Find the leader drone (entity ID stored in formation_offset)
   b) Get leader position and velocity
   c) Calculate desired position: leader_pos + formation_offset
   d) Calculate heading toward desired position
   e) Apply thrust toward target:
      - Max speed: 150 m/s
      - Acceleration: Smooth pursuit (not instant)
   f) Add weaving pattern:
      - Oscillate position perpendicular to movement
      - Frequency: 2 oscillations per second
      - Amplitude: 50m deviation
   g) If player within 2km:
      - Shoot every 2 seconds (handled by separate fire system)

4. Velocity clamping:
   - Clamp speed to max 150 m/s
   - Maintain formation offset (± 50m)

PSEUDOCODE:
```rust
fn update_swarm_ai(
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut drone_query: Query<
        (&Transform, &mut LinearVelocity, &mut ExternalForce, &DroneAI),
        (With<Drone>, Without<PlayerPlane>),
    >,
) {
    let player_pos = player_query.single().translation;

    // First pass: Update all drones
    for (drone_tf, mut vel, mut force, ai) in &mut drone_query {
        if let DroneAI::Swarm { leader, formation_offset } = ai {
            // Find leader (query again or store in component)
            let leader_velocity = /* get from leader */;
            
            // Desired position = leader + offset
            let desired_pos = leader_pos + formation_offset;
            
            // Direction toward desired pos
            let direction = (desired_pos - drone_tf.translation).normalize();
            
            // Thrust toward target
            let max_speed = 150.0;
            let thrust = direction * max_speed;
            
            // Add weaving (sine wave perpendicular to motion)
            let weave = /* calculate perpendicular oscillation */;
            
            vel.0 = (thrust + weave).normalize() * vel.0.length();
        }
    }
}
```

INTEGRATION INTO MAIN:
Add to main() .add_systems(Update, ...):
```rust
.add_systems(Update, update_swarm_ai)
```

TEST: After implementation:
- [ ] Swarm drones move in formation
- [ ] Follow leader drone
- [ ] Weave pattern visible
- [ ] No collision between group members
- [ ] Drones stay within 50m of formation center

EXPECTED BEHAVIOR:
- Watch a swarm from distance
- See 3-5 drones flying together
- Formation maintains cohesion
- Weaving creates dynamic visual
- Drones don't collide with each other
```

---

## PROMPT 4: Gemini - Implement Kamikaze AI System (Coding)

**After:** PROMPT 3 completes (swarms flying)

```
Implement Phase 3 Part 3: Kamikaze AI pursuit and explosion.

FILE: src/main.rs

ADD NEW SYSTEM: update_kamikaze_ai() (after update_swarm_ai, ~line 1550):

The system should:

1. Query kamikaze drones with:
   - Transform
   - LinearVelocity
   - ExternalForce
   - DroneAI
   - DroneHealth

2. Query player Transform

3. For each kamikaze drone:
   a) Calculate vector to player
   b) Calculate distance to player
   c) If distance > 2000m: No pursuit (out of range)
   d) If distance <= 2000m and < 100m: EXPLODE
   e) If distance 100m-2000m:
      - Full acceleration toward player
      - Max speed: 150 m/s (same as swarms)
      - Direct pursuit (no weaving)

4. Explosion handling (on collision with player):
   - Spawn explosion effect at drone position
   - Play explosion sound
   - Remove drone entity
   - Apply damage to player (25 HP damage)
   - Notify player they got hit

PSEUDOCODE:
```rust
fn update_kamikaze_ai(
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut drone_query: Query<
        (Entity, &Transform, &mut LinearVelocity, &mut ExternalForce, &DroneAI),
        (With<Drone>, Without<PlayerPlane>),
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_pos = player_query.single().translation;

    for (entity, drone_tf, mut vel, mut force, ai) in &mut drone_query {
        if let DroneAI::Kamikaze { target: _ } = ai {
            let to_player = player_pos - drone_tf.translation;
            let distance = to_player.length();

            if distance < 100.0 {
                // EXPLODE
                spawn_drone_explosion(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    drone_tf.translation,
                );
                commands.entity(entity).despawn_recursive();
            } else if distance < 2000.0 {
                // PURSUE
                let direction = to_player.normalize();
                let max_speed = 150.0;
                vel.0 = direction * max_speed;
            }
            // else: out of range, no movement
        }
    }
}
```

ADD NEW FUNCTION: spawn_drone_explosion() (after spawn_huge_explosion, ~line 2100):

Creates visual explosion effect:
- Small fireball (radius 20m)
- Orange/red emissive material
- Explosion particle burst (10-20 particles)
- Light effect (short duration)
- Explosion sound

INTEGRATION INTO MAIN:
Add to main() .add_systems(Update, ...):
```rust
.add_systems(Update, update_kamikaze_ai)
```

ADD DAMAGE SYSTEM: Create player_health tracking (if not exists):
- Add component to player: struct PlayerHealth(f32) with default 100.0
- Apply 25 damage on kamikaze explosion
- Handle death (respawn at sea level)

TEST: After implementation:
- [ ] Kamikaze drones detect player within 2km
- [ ] Pursue player directly (no weaving)
- [ ] Explode when reaching player (<100m)
- [ ] Explosion effect visible and sounds
- [ ] Player takes damage (25 HP)
- [ ] Drones out of range don't pursue

EXPECTED BEHAVIOR:
- Fly around, see drones detecting you
- Kamikaze drones beeline toward you
- They explode on impact
- You take damage and must avoid/destroy them
```

---

## PROMPT 5: Gemini - Implement Drone Firing System (Optional Combat Feature)

**After:** PROMPT 4 completes (kamikazes work)

```
Implement Phase 3 Part 4: Drone firing system.

FILE: src/main.rs

ADD DRONE FIRING: update_drone_firing() system (after update_kamikaze_ai, ~line 1600):

Requirements:
1. Swarm drones fire at player if within 2km (once every 2 seconds)
2. Each drone fires one projectile per shot
3. Projectile is simple sphere with missile speed
4. Player can shoot drones (bullets do 10 damage each)

Implementation:
```rust
fn update_drone_firing(
    time: Res<Time>,
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut drone_query: Query<
        (&Transform, &DroneAI, &mut LastShotTime),
        (With<Drone>, Without<PlayerPlane>),
    >,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_pos = player_query.single().translation;
    let current_time = time.elapsed_secs();

    for (drone_tf, ai, mut last_shot) in &mut drone_query {
        // Only swarm drones fire (kamikazes don't need to)
        if !matches!(ai, DroneAI::Swarm { .. }) {
            continue;
        }

        let distance = drone_tf.translation.distance(player_pos);
        if distance > 2000.0 {
            continue; // Out of range
        }

        // Fire every 2 seconds
        if current_time - last_shot.time >= 2.0 {
            let direction = (player_pos - drone_tf.translation).normalize();
            let projectile_pos = drone_tf.translation + direction * 3.0;
            
            spawn_drone_missile(
                &mut commands,
                &mut meshes,
                &mut materials,
                projectile_pos,
                direction * 200.0, // Drone projectile speed
            );
            
            last_shot.time = current_time;
        }
    }
}
```

ADD DRONE DAMAGE COMPONENT:
- Track damage dealt to drones
- 10 damage per player bullet hit
- 50 HP = 5 bullets to kill
- Handle drone death/explosion

HANDLE PLAYER BULLET COLLISION WITH DRONES:
- Modify handle_projectile_collisions() to check for Drone entities
- Apply damage to drone on hit
- Remove bullet
- Check if drone health <= 0
- Spawn drone explosion if killed

TEST: After implementation:
- [ ] Swarm drones fire at you (every 2 seconds)
- [ ] Drone projectiles are visible
- [ ] You can shoot drones with bullets (Space key)
- [ ] 5 bullets kill one drone
- [ ] Killed drones explode and disappear

EXPECTED BEHAVIOR:
- Swarm drones shooting at you while flying together
- You must aim and shoot to destroy them
- Combat becomes interactive and challenging
```

---

## PROMPT 6: Copilot - Phase 3 Integration Testing

**After:** All drone systems implemented

```
Test Phase 3 combat system comprehensively:

TEST SCENARIO 1: Swarm Drones
1. Start game and fly to altitude 1000m
2. Observe for 30 seconds
   - Expected: See swarms of 3-5 drones flying together
   - Behavior: Drones maintain formation and weave
3. Get close to swarm (within 500m)
   - Expected: Drones start firing at you
   - Observe: Projectiles coming toward you
4. Try to shoot drones (Space key)
   - Expected: Drones take damage (1 bullet = 10 damage)
   - 5 bullets per drone to destroy
5. Destroy one swarm completely
   - Expected: All drones explode on kill

TEST SCENARIO 2: Kamikaze Drones
1. Fly at low altitude
2. Look for kamikaze drones (should look different or have AI label)
3. Let one get close
   - Expected: Kamikaze pursues directly at you
   - No weaving, just straight pursuit
4. Let kamikaze reach you (collision)
   - Expected: Massive explosion effect
   - Health decreases (25 damage?)
5. Shoot kamikaze before it reaches
   - Expected: 5 bullets destroy it
   - Explosion at destruction point

TEST SCENARIO 3: Mixed Combat
1. Fly around for 2 minutes
2. Engage with multiple drone groups
3. Verify:
   - [ ] Multiple swarms don't interfere with each other
   - [ ] Kamikazes don't stack formations
   - [ ] Explosions are visually distinct (drone vs kamikaze)
   - [ ] FPS stays above 60 (report if drops)
   - [ ] No physics crashes
4. Try to destroy all visible drones
   - Expected: Combat feels challenging but fair
   - Drones dangerous but defeatable

TEST SCENARIO 4: Performance
1. Count visible drones (should show in diagnostics or estimate visually)
2. Monitor FPS:
   - With 0 drones: _____ FPS
   - With 20 drones: _____ FPS
   - With 50 drones: _____ FPS
   - With 100 drones: _____ FPS
3. Report performance impact

TEST SCENARIO 5: Edge Cases
1. Destroy all drones in current chunk
   - Expected: New drones spawn when moving to next chunk
2. Fly very fast (rocket mode) through drone group
   - Expected: No collision explosions, clean pass-through
3. Let player health reach 0
   - Expected: Respawn at sea level or damage feedback

PASS CRITERIA:
✅ Swarms fly in formation and fire
✅ Kamikazes pursue and explode
✅ Combat feels engaging
✅ FPS stays 60+ with 50+ drones
✅ No physics crashes
✅ Damage system works

REPORT:
- Any combat feels unfair (too easy/hard)
- Performance drops
- Visual/audio issues
- Suggestions for balance adjustments
```

---

## PROMPT 7: Gemini - Phase 3 Polish & Balance (Final)

**After:** Integration testing completes

```
Polish Phase 3 combat system for release:

TUNING PARAMETERS:
Review and adjust these based on playtesting feedback:

1. Drone difficulty:
   - Swarm fire rate: 1 shot / 2 seconds (adjustable?)
   - Kamikaze pursuit speed: 150 m/s (too fast/slow?)
   - Drone health: 50 HP (5 bullets) - balanced?

2. Visual feedback:
   - Add console message when drone killed:
     "💥 Drone destroyed! [Swarm/Kamikaze] (X destroyed total)"
   - Add drone damage indicator:
     When drone takes damage, show health status?

3. Audio:
   - Drone fire sound (different from player bullet)
   - Drone explosion sound (different from player explosion)
   - Damage/hit feedback sound

4. Drone spawn balance:
   - Too many drones? Reduce DRONES_PER_CHUNK_MAX
   - Too few? Increase it
   - Adjust mix (60% swarm, 40% kamikaze)

5. AI tuning:
   - Swarm weaving too aggressive? Reduce amplitude
   - Kamikaze pursuit too slow? Increase speed
   - Formation too tight? Increase offset variance

IMPLEMENTATION:
Add tuning constants at top of file:
```rust
const DRONE_FIRE_RATE: f32 = 2.0; // seconds between shots
const DRONE_MAX_SPEED: f32 = 150.0; // m/s
const KAMIKAZE_EXPLOSION_RANGE: f32 = 100.0; // meters
const PLAYER_DAMAGE_PER_EXPLOSION: f32 = 25.0; // HP
const SWARM_WEAVE_AMPLITUDE: f32 = 50.0; // meters
const SWARM_WEAVE_FREQUENCY: f32 = 2.0; // oscillations/second
```

Allow tweaking these without recompiling (save to config.json in future update).

FINAL QUALITY CHECKLIST:
- [ ] Combat feels fair and engaging
- [ ] Drones are visible and audible
- [ ] Explosions look impressive
- [ ] FPS stable at 60+ (even with 100+ drones)
- [ ] No physics crashes or glitches
- [ ] Drone AI behavior realistic and varied
- [ ] Player feedback clear (hit, damage, kill)
```

---

## Execution Order

| Step | Prompt | Agent | Duration | Prerequisite |
|------|--------|-------|----------|--------------|
| 0 | User provides drone model | User | - | BLOCKER |
| 1 | Architecture Design | Gemini | 1 hr | Drone model ready |
| 2 | Components & Spawning | Gemini | 2 hrs | PROMPT 1 approved |
| 3 | Swarm AI | Gemini | 2 hrs | PROMPT 2 compiles |
| 4 | Kamikaze AI | Gemini | 2 hrs | PROMPT 3 working |
| 5 | Drone Firing | Gemini | 2 hrs | PROMPT 4 working |
| 6 | Integration Testing | Copilot | 1 hr | PROMPTS 2-5 complete |
| 7 | Polish & Balance | Gemini | 1 hr | Testing feedback |

**Total:** ~12 hours over 2-3 days

---

## Success Criteria - Phase 3 Complete When

✅ **Drone spawning works:**
- Drones appear in chunks
- Mix of swarm and kamikaze
- Deterministic placement

✅ **Swarm AI functional:**
- Drones fly in formations
- Maintain cohesion
- Weave pattern visible
- Fire at player periodically

✅ **Kamikaze AI functional:**
- Pursue player aggressively
- Explode on contact
- Deal damage to player

✅ **Combat engaging:**
- Can shoot and destroy drones
- Combat feels challenging but fair
- FPS remains 60+

✅ **Polish complete:**
- Visual/audio feedback
- Balance tuning done
- No crashes or glitches

---

## Notes for User

- Phase 3 is **completely optional** for MVP
- Game is fully playable without combat (explore world in Phases 1-2)
- Consider Phase 3 an expansion/upgrade
- User must provide drone 3D models to start
- Combat will be one of most complex systems - allow 2-3 days for full implementation
