# Phase 3: Combat Mechanics - Complete Implementation

## Component Definitions (Add to Components Section)

```rust
// ============================================================================
// PHASE 3 COMPONENTS - Combat System
// ============================================================================

/// Marker component for projectiles fired by planes
#[derive(Component)]
struct Projectile {
    /// Time remaining before bullet despawns (seconds)
    lifetime: f32,
}

/// Tracks when the player last fired to enforce cooldown
#[derive(Component)]
struct LastShotTime {
    /// Time in seconds when last shot was fired (from Time::elapsed_secs())
    time: f32,
}

impl Default for LastShotTime {
    fn default() -> Self {
        Self {
            time: -10.0, // Start at -10 so player can shoot immediately
        }
    }
}

/// Temporary light effect for muzzle flash
#[derive(Component)]
struct MuzzleFlash {
    /// How long the flash has existed (seconds)
    lifetime: f32,
}
```

## Constants (Add to Constants Section)

```rust
// ============================================================================
// PHASE 3 CONSTANTS - Combat Tuning
// ============================================================================

const BULLET_SPEED: f32 = 100.0;           // Forward speed (m/s)
const BULLET_LIFETIME: f32 = 5.0;          // Despawn after 5 seconds
const FIRE_RATE: f32 = 10.0;               // Bullets per second
const FIRE_COOLDOWN: f32 = 1.0 / FIRE_RATE; // 0.1 seconds between shots
const GUN_OFFSET: Vec3 = Vec3::new(0.0, 0.0, -3.0); // Gun position (front of plane)
const BULLET_RADIUS: f32 = 0.2;            // Visual size
const MUZZLE_FLASH_DURATION: f32 = 0.05;   // 50ms flash
const MUZZLE_FLASH_INTENSITY: f32 = 500.0; // Brightness
```

## System Functions (Add New Systems)

```rust
// ============================================================================
// PHASE 3 SYSTEMS - Combat
// ============================================================================

/// Handles player shooting input and spawns projectiles
/// 
/// This system:
/// 1. Checks if Space bar is pressed
/// 2. Verifies cooldown has elapsed (0.1s between shots)
/// 3. Spawns a bullet entity at the gun position
/// 4. Updates the last shot time
fn handle_shooting_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut LastShotTime), With<PlayerPlane>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the single player entity (unwrap safe - player always exists)
    if let Ok((player_transform, mut last_shot)) = player_query.get_single_mut() {
        // Check if Space is pressed AND enough time has passed
        let current_time = time.elapsed_secs();
        let can_shoot = keyboard.pressed(KeyCode::Space) 
            && (current_time - last_shot.time >= FIRE_COOLDOWN);
        
        if can_shoot {
            // Calculate gun position in world space
            // GUN_OFFSET is relative to player's local coordinate system
            let gun_position_world = player_transform.transform_point(GUN_OFFSET);
            
            // Get the direction the player is facing (Bevy uses -Z as forward)
            let forward = -player_transform.local_z();
            
            // Calculate bullet's initial velocity
            // Inherit player's forward direction at BULLET_SPEED
            let bullet_velocity = forward * BULLET_SPEED;
            
            // Create bullet mesh (small red sphere)
            let bullet_mesh = meshes.add(Sphere::new(BULLET_RADIUS));
            let bullet_material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0), // Bright red
                emissive: Color::srgb(0.5, 0.0, 0.0).into(), // Glowing effect
                ..default()
            });
            
            // Spawn the projectile entity
            commands.spawn((
                Projectile { lifetime: BULLET_LIFETIME },
                Mesh3d(bullet_mesh),
                MeshMaterial3d(bullet_material),
                Transform::from_translation(gun_position_world),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                // Avian3D physics components
                RigidBody::Dynamic,
                LinearVelocity(bullet_velocity),
                Collider::sphere(BULLET_RADIUS),
                // Make bullets light and not affected by gravity
                Mass::new(0.01),
                GravityScale(0.0), // No gravity - straight line trajectory
            ));
            
            // Spawn muzzle flash effect
            spawn_muzzle_flash(&mut commands, gun_position_world);
            
            // Update last shot time
            last_shot.time = current_time;
            
            // Debug output
            println!("FIRE! Bullet spawned at {:.1?}", gun_position_world);
        }
    }
}

/// Updates all projectiles - ages them and despawns expired ones
///
/// This system runs every frame for all bullets:
/// 1. Decreases lifetime by delta time
/// 2. Despawns bullets that have expired
fn update_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile)>,
) {
    for (entity, mut projectile) in &mut projectile_query {
        // Reduce lifetime by time since last frame (delta_secs)
        projectile.lifetime -= time.delta_secs();
        
        // If lifetime expired, despawn the bullet
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
            println!("Bullet expired (lifetime)");
        }
    }
}

/// Detects and handles projectile collisions using Avian3D events
///
/// Collision types:
/// - Bullet vs Ground: Despawn bullet
/// - Bullet vs Enemy: Despawn bullet, damage enemy (Phase 4)
/// - Bullet vs Bullet: Ignore for now
fn handle_projectile_collisions(
    mut collision_events: EventReader<Collision>,
    projectile_query: Query<Entity, With<Projectile>>,
    ground_query: Query<Entity, (With<Collider>, Without<Projectile>, Without<PlayerPlane>)>,
    mut commands: Commands,
) {
    // Iterate through all collision events this frame
    for Collision(contacts) in collision_events.read() {
        // Check if either entity is a projectile
        let projectile_entity = if projectile_query.contains(contacts.entity1) {
            Some(contacts.entity1)
        } else if projectile_query.contains(contacts.entity2) {
            Some(contacts.entity2)
        } else {
            None
        };
        
        if let Some(bullet) = projectile_entity {
            // Determine what the bullet hit
            let other_entity = if bullet == contacts.entity1 {
                contacts.entity2
            } else {
                contacts.entity1
            };
            
            // Check if bullet hit the ground or static geometry
            if ground_query.contains(other_entity) {
                commands.entity(bullet).despawn_recursive();
                println!("Bullet hit ground!");
                
                // TODO Phase 3.5: Spawn explosion/impact effect here
            }
            // TODO Phase 4: Check for enemy collision
            // else if enemy_query.contains(other_entity) {
            //     commands.entity(bullet).despawn_recursive();
            //     // Apply damage to enemy
            // }
        }
    }
}

/// Updates muzzle flash effects and despawns them after duration
fn update_muzzle_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flash_query: Query<(Entity, &mut MuzzleFlash)>,
) {
    for (entity, mut flash) in &mut flash_query {
        flash.lifetime += time.delta_secs();
        
        if flash.lifetime >= MUZZLE_FLASH_DURATION {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Helper function to spawn a temporary muzzle flash light
fn spawn_muzzle_flash(commands: &mut Commands, position: Vec3) {
    commands.spawn((
        MuzzleFlash { lifetime: 0.0 },
        PointLight {
            intensity: MUZZLE_FLASH_INTENSITY,
            color: Color::srgb(1.0, 0.9, 0.5), // Yellowish-white
            range: 10.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    ));
}
```

## Integration Instructions

### 1. Add Components to Player Spawn

In the `spawn_player()` function, add `LastShotTime::default()` to the player entity:

```rust
// In spawn_player(), add to player entity components:
commands.spawn((
    PlayerPlane,
    Transform::from_xyz(0.0, 50.0, 0.0),
    // ... existing components ...
    LastShotTime::default(), // ADD THIS LINE
))
```

### 2. Register Systems

In `main()`, add the Phase 3 systems to the update schedule:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(ClearColor(Color::srgb(0.5, 0.8, 1.0)))
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, (
            read_player_input,
            apply_flight_physics,
            apply_angular_damping,
            update_flight_camera,
            debug_player_state,
            handle_quit,
            // PHASE 3: Combat Systems
            handle_shooting_input,      // ADD THIS
            update_projectiles,         // ADD THIS
            handle_projectile_collisions, // ADD THIS
            update_muzzle_flashes,      // ADD THIS
        ))
        .run();
}
```

### 3. Add Event Registration

Avian3D collision events need to be registered. Add this BEFORE `.add_systems()`:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_event::<Collision>() // ADD THIS LINE
        .insert_resource(ClearColor(Color::srgb(0.5, 0.8, 1.0)))
        // ... rest of setup
```

### 4. Update Imports

At the top of `main.rs`, ensure you have:

```rust
use bevy::prelude::*;
use avian3d::prelude::*;
```

## Testing Checklist

### Basic Functionality
- [ ] Press Space - bullet appears in front of plane
- [ ] Hold Space - bullets fire at ~10/second (not continuous stream)
- [ ] Bullets move forward in straight line at constant speed
- [ ] Bullets despawn after 5 seconds
- [ ] White flash appears briefly at gun position when firing

### Collision Detection
- [ ] Fly low and shoot ground - bullets disappear on contact
- [ ] Console prints "Bullet hit ground!" message
- [ ] Console prints "FIRE! Bullet spawned at..." when shooting
- [ ] Console prints "Bullet expired (lifetime)" after 5 seconds

### Edge Cases
- [ ] Can shoot immediately on game start (no initial cooldown)
- [ ] Rapid Space tapping respects 0.1s cooldown
- [ ] Bullets don't slow down over time (no drag)
- [ ] Bullets don't curve (no gravity)

### Performance
- [ ] Game runs smoothly with 20+ bullets on screen
- [ ] Muzzle flashes despawn quickly (no buildup)
- [ ] No memory leaks (bullets despawn properly)

## Debug Commands

Add these temporary debug systems to verify behavior:

```rust
// Temporary debug system - counts active bullets
fn debug_bullet_count(
    mut counter: Local<u32>,
    projectile_query: Query<&Projectile>,
) {
    *counter += 1;
    if *counter % 60 == 0 {
        println!("Active bullets: {}", projectile_query.iter().count());
    }
}
```

Add to update systems for testing, remove once verified.

## Tuning Guide

### Fire Rate Too Slow/Fast
Adjust `FIRE_RATE` constant (bullets per second)

### Bullets Too Fast/Slow
Adjust `BULLET_SPEED` constant (m/s)

### Bullets Disappear Too Soon
Increase `BULLET_LIFETIME` (seconds)

### Gun Position Wrong
Modify `GUN_OFFSET` Vec3:
- X: Left/right (-/+)
- Y: Up/down (-/+)  
- Z: Forward/back (-/+) - negative is forward in Bevy

### Muzzle Flash Too Dim/Bright
Adjust `MUZZLE_FLASH_INTENSITY` constant

## Known Limitations

1. **No bullet trails** - Phase 3.5 can add particle effects
2. **No recoil** - Could add in Phase 3.5 as camera shake
3. **No bullet spread** - Perfectly accurate for now
4. **No ammo limit** - Infinite bullets
5. **No bullet-bullet collision** - Bullets pass through each other
6. **No ricochet** - Bullets just despawn on impact

## Phase 4 Integration Points

The collision system is designed to easily add enemy detection:

```rust
// In handle_projectile_collisions, add this after ground check:
else if let Ok(mut enemy) = enemy_query.get_mut(other_entity) {
    commands.entity(bullet).despawn_recursive();
    enemy.health -= 10.0; // Or whatever damage system you use
    println!("Enemy hit!");
}
```

## Architecture Decisions

### Q: Gun position - relative or absolute?
**A:** Relative to player transform using `transform_point(GUN_OFFSET)`. This ensures bullets spawn correctly regardless of plane orientation.

### Q: Bullets with gravity or straight line?
**A:** Straight line (`GravityScale(0.0)`). Arcade feel prioritizes predictability over realism.

### Q: Visual recoil/screen shake?
**A:** Not in Phase 3. Can add in Phase 3.5 polish pass. Current design supports it via camera system.

### Q: Collision detection method?
**A:** Avian3D collision events. More performant than raycasts for this use case, and provides exact collision information.

### Q: Why `Mass::new(0.01)` for bullets?
**A:** Light mass prevents bullets from pushing heavy objects (ground, planes). They should disappear on impact, not bounce.

### Q: Why separate muzzle flash system?
**A:** Decouples visual effect from bullet logic. Flash can be tweaked independently and doesn't affect gameplay.

## Common Issues & Fixes

**Issue:** Bullets spawn inside player and immediately collide
- **Fix:** Increase `GUN_OFFSET.z` magnitude (more negative = further forward)

**Issue:** Bullets fire backward
- **Fix:** Check you're using `-player_transform.local_z()` (negative Z is forward in Bevy)

**Issue:** Can't shoot at all
- **Fix:** Verify `LastShotTime::default()` is on player entity

**Issue:** Collision events not firing
- **Fix:** Ensure `.add_event::<Collision>()` is registered in `main()`

**Issue:** Bullets curve downward
- **Fix:** Check `GravityScale(0.0)` is set on projectile

**Issue:** Muzzle flash stays forever
- **Fix:** Verify `update_muzzle_flashes` system is in update schedule

## Performance Notes

- Each bullet is ~10 components (lightweight)
- Collision events are efficient (O(n) where n = collisions, not bullets)
- Muzzle flashes auto-cleanup in 50ms
- Lifetime-based despawning prevents bullet buildup
- At 10 bullets/sec, ~50 bullets max before oldest despawn

Expected performance: 60 FPS with 100+ active bullets on modest hardware.
