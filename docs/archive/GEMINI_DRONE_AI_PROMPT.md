# GEMINI: Implement Drone Combat AI

**Priority:** Phase 3 Feature
**Status:** Ready to implement
**Estimated Time:** 2-3 hours

---

## Current State

Drones are spawned and move forward at constant speed. They have:
- Health system (50 HP)
- Collision detection with missiles (working)
- Explosion effects on death (working)
- Basic forward movement at 150 m/s

**What's Missing:** Drones don't pursue the player - they just fly in a straight line.

---

## Your Task: Implement Kamikaze Pursuit AI

Make drones actively chase the player plane like Ukrainian military drones.

### Requirements

1. **Target Acquisition**
   - Drones should detect player position
   - Calculate direction to player
   - Smoothly turn toward player

2. **Pursuit Behavior**
   - Gradually rotate toward player (not instant snap)
   - Maintain pursuit even when player maneuvers
   - Speed up when closing in (optional)

3. **Attack Pattern**
   - Fly directly at player (kamikaze style)
   - If within 20m of player, trigger explosion/damage

4. **Swarm Coordination (Optional)**
   - Multiple drones approach from different angles
   - Don't all follow exact same path

---

## Code Location

**File:** `src/drone.rs`

**Current move_drones function (lines 60-122):**
```rust
fn move_drones(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Drone)>,
) {
    for (entity, mut transform, drone) in &mut query {
        // ... validation code ...

        let forward = transform.forward();
        let move_vec = forward * drone.speed * time.delta_secs();
        transform.translation += move_vec;
    }
}
```

---

## Implementation Guide

### Step 1: Add Player Query to move_drones

Change the system signature to also query the player:

```rust
fn move_drones(
    time: Res<Time>,
    mut drone_query: Query<(Entity, &mut Transform, &Drone), Without<PlayerPlane>>,
    player_query: Query<&Transform, With<PlayerPlane>>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation;

    for (entity, mut transform, drone) in &mut drone_query {
        // ... pursuit logic here ...
    }
}
```

**Note:** You'll need to import `PlayerPlane` from main.rs or pass it as a marker.

### Step 2: Calculate Direction to Player

```rust
// Direction from drone to player
let to_player = (player_pos - transform.translation).normalize_or_zero();

// Current forward direction
let current_forward = transform.forward().as_vec3();

// Smoothly interpolate toward player (adjust 2.0 for turn speed)
let turn_speed = 2.0 * time.delta_secs();
let new_forward = current_forward.lerp(to_player, turn_speed).normalize_or_zero();
```

### Step 3: Update Rotation to Face New Direction

```rust
// Only update if we have a valid direction
if new_forward.length() > 0.5 {
    // Calculate rotation that looks in new_forward direction
    let up = Vec3::Y;
    let look_rotation = Transform::IDENTITY.looking_to(new_forward, up).rotation;

    // Smoothly rotate toward target
    transform.rotation = transform.rotation.slerp(look_rotation, turn_speed);
}
```

### Step 4: Move Forward (existing code)

```rust
let forward = transform.forward();
let move_vec = forward * drone.speed * time.delta_secs();
transform.translation += move_vec;
```

---

## Testing

1. **Build:** `cargo build --release`
2. **Copy assets:** `cp -r assets target/release/`
3. **Run:** `target/release/plane_game.exe`

**Verify:**
- Drones should turn toward player
- Drones should chase player when you fly past them
- Turn rate should be smooth, not instant
- No crashes or NaN errors

---

## Bonus Features (If Time Permits)

### Speed Boost When Close
```rust
let distance = (player_pos - transform.translation).length();
let speed_mult = if distance < 500.0 { 1.5 } else { 1.0 };
let move_vec = forward * drone.speed * speed_mult * time.delta_secs();
```

### Collision Damage to Player
Add a system that checks if drone is within 20m of player and damages them:
```rust
fn drone_player_collision(
    drone_query: Query<&Transform, With<Drone>>,
    player_query: Query<&Transform, With<PlayerPlane>>,
) {
    // Check distance, apply damage if < 20m
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| src/drone.rs | Update move_drones() with pursuit logic |
| src/main.rs | May need to export PlayerPlane marker or add collision system |

---

## Success Criteria

- [ ] Drones turn toward player smoothly
- [ ] Drones chase player across the map
- [ ] No NaN/physics crashes
- [ ] Game runs at 60+ FPS with 6 pursuing drones
- [ ] Pursuit feels threatening but dodgeable

---

## Reference: Current Drone Spawn Positions

From main.rs (lines 695-706):
```rust
// Close formation (visible immediately)
spawn_beaver_drone(..., Vec3::new(0.0, 520.0, -200.0));
spawn_beaver_drone(..., Vec3::new(-150.0, 500.0, -100.0));
spawn_beaver_drone(..., Vec3::new(150.0, 500.0, -100.0));

// Long range (player will encounter later)
spawn_beaver_drone(..., Vec3::new(500.0, 1500.0, -2000.0));
spawn_beaver_drone(..., Vec3::new(-500.0, 1200.0, -3000.0));
spawn_beaver_drone(..., Vec3::new(0.0, 2000.0, -5000.0));
```

Good luck! The player is counting on you to make these drones dangerous.
