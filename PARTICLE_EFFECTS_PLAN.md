# Particle Effects Implementation Plan

**Status:** Ready for implementation
**Priority:** Visual enhancement (non-critical for flight)
**Time Estimate:** 30-45 minutes

---

## Current Problem

The afterburner flame is currently:
- ❌ Orange rectangle on the RIGHT side of plane
- ❌ Not connected to the jet exhaust
- ❌ Static (doesn't scale with throttle)
- ❌ Geometry-based (not particle effects)

**Goal:** Replace with proper particle effects from Kenney particle pack that:
- ✅ Spawn at plane's rear exhaust (Z = +3 to +4 in model space)
- ✅ Scale size with throttle (0% = tiny/none, 100% = large burst)
- ✅ Look like fire/flame/smoke
- ✅ Fade with distance from plane

---

## Asset Strategy

### Flame Textures Available
From `E:\Downloads\kenney_particle-pack\PNG (Transparent)`:
- `flame_01.png` through `flame_05.png` - Good for primary flame
- `fire_01.png`, `fire_02.png` - Alternative flame style
- `smoke_01.png` through `smoke_10.png` - Trailing smoke

### Recommended Selection
1. **Primary Flame**: `flame_01.png` or `flame_02.png`
   - Use for hot, visible flames
   - Position at jet exhaust (rear)
   - Scale with throttle

2. **Secondary Smoke** (optional): `smoke_03.png` or `smoke_05.png`
   - Trails behind the flame
   - Appears when throttle > 50%
   - More subtle movement

3. **Intensity Flame** (optional): `flame_04.png` or `flame_05.png`
   - Appears only during boost (throttle > 80%)
   - Larger and more intense

---

## Implementation Plan

### Step 1: Copy Assets to Project

```bash
# Copy Kenney particle textures to project
mkdir -p C:\Users\Box\plane_game\assets\particles
copy "E:\Downloads\kenney_particle-pack\PNG (Transparent)\flame_*.png" C:\Users\Box\plane_game\assets\particles\
copy "E:\Downloads\kenney_particle-pack\PNG (Transparent)\smoke_*.png" C:\Users\Box\plane_game\assets\particles\
copy "E:\Downloads\kenney_particle-pack\PNG (Transparent)\fire_*.png" C:\Users\Box\plane_game\assets\particles\
```

**Result:**
```
assets/
  particles/
    flame_01.png
    flame_02.png
    flame_03.png
    flame_04.png
    flame_05.png
    smoke_01.png
    smoke_03.png
    smoke_05.png
    fire_01.png
    fire_02.png
```

---

### Step 2: Create Particle Emitter Component

**Replace old `AfterburnerFlame` component:**

```rust
#[derive(Component)]
struct AfterburnerParticles {
    /// How many particles to spawn per frame when throttle > threshold
    spawn_rate: f32,
    /// Minimum throttle before any particles spawn (0.0-1.0)
    spawn_threshold: f32,
    /// Time until particle dies (seconds)
    particle_lifetime: f32,
}

impl Default for AfterburnerParticles {
    fn default() -> Self {
        Self {
            spawn_rate: 5.0,              // 5 particles per frame at full throttle
            spawn_threshold: 0.2,         // Start showing at 20% throttle
            particle_lifetime: 0.8,       // 0.8 second life
        }
    }
}

#[derive(Component)]
struct Particle {
    /// Time until this particle dies
    lifetime_remaining: f32,
    /// Original lifetime (for fade calculation)
    lifetime_max: f32,
    /// Velocity of particle in world space (for movement)
    velocity: Vec3,
}
```

---

### Step 3: Particle Spawning System

```rust
fn spawn_afterburner_particles(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Transform, &PlayerInput), With<PlayerPlane>>,
    emitter_query: Query<(Entity, &Transform, &AfterburnerParticles), With<PlayerPlane>>,
) {
    if let Ok((player_transform, input)) = player_query.get_single() {
        for (_emitter_entity, _emitter_transform, emitter) in &emitter_query {
            // Only spawn if throttle is above threshold
            if input.throttle < emitter.spawn_threshold {
                continue;
            }

            // Calculate spawn rate based on throttle (higher throttle = more particles)
            let throttle_factor = (input.throttle - emitter.spawn_threshold) / (1.0 - emitter.spawn_threshold);
            let actual_spawn_rate = emitter.spawn_rate * throttle_factor;

            // Spawn particles this frame
            for _ in 0..(actual_spawn_rate as u32) {
                // Position: rear of plane (Z = +3.5 in model space, which is -3.5 in world after 180° rotation)
                let model_space_pos = Vec3::new(0.0, -0.2, 3.5);  // Slightly below center
                let world_pos = player_transform.transform_point(model_space_pos);

                // Velocity: backward velocity + some randomness
                let backward_velocity = player_transform.forward().as_vec3() * -20.0; // Moving backward
                let random_spread = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 5.0,
                    (rand::random::<f32>() - 0.5) * 5.0 + 3.0,  // Upward bias
                    (rand::random::<f32>() - 0.5) * 5.0,
                );
                let velocity = backward_velocity + random_spread;

                // Load flame texture (cycle through flame_01 to flame_03)
                let flame_index = ((time.elapsed_secs() * 5.0) as usize) % 3 + 1;
                let texture_path = format!("particles/flame_0{}.png", flame_index);
                let texture_handle = asset_server.load(&texture_path);

                // Create material with the flame texture
                let material = StandardMaterial {
                    base_color_texture: Some(texture_handle.clone()),
                    base_color: Color::srgba(1.0, 1.0, 1.0, 0.9),  // Start opaque
                    emissive: Color::srgb(2.0, 1.5, 0.5),           // Glow effect
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,  // Don't need complex lighting for particle
                    ..default()
                };

                let material_handle = materials.add(material);

                // Size: based on throttle (higher throttle = bigger particles)
                let size = 0.5 + throttle_factor * 0.8;  // 0.5 to 1.3 units
                let quad_mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::splat(size))));

                // Spawn the particle
                commands.spawn((
                    Particle {
                        lifetime_remaining: emitter.particle_lifetime,
                        lifetime_max: emitter.particle_lifetime,
                        velocity,
                    },
                    Transform::from_translation(world_pos)
                        .with_rotation(Quat::from_rotation_y(rand::random::<f32>() * std::f32::consts::TAU)),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    MeshMaterial3d(material_handle),
                    quad_mesh,
                ));
            }
        }
    }
}
```

---

### Step 4: Particle Update System

```rust
fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut transform, mut particle, material_handle) in &mut particle_query {
        // Update lifetime
        particle.lifetime_remaining -= time.delta_secs();

        // If dead, despawn
        if particle.lifetime_remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Update position (particle moves with velocity + slight upward drift)
        let drift = Vec3::Y * 5.0 * time.delta_secs();  // Upward drift
        transform.translation += particle.velocity * time.delta_secs() + drift;

        // Update opacity based on remaining lifetime (fade out)
        let opacity = particle.lifetime_remaining / particle.lifetime_max;
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color.set_alpha(opacity);
        }

        // Slight rotation for visual interest
        transform.rotate_y(2.0 * time.delta_secs());
    }
}
```

---

### Step 5: Update Player Spawn

**Remove the old flame geometry, add particle emitter:**

```rust
fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ... existing spawn code ...

    let player = commands.spawn((
        PlayerPlane,
        Transform::from_xyz(0.0, 500.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        RigidBody::Dynamic,
        Mass(MASS_KG),
        LinearVelocity(Vec3::new(0.0, 0.0, -100.0)),
        AngularVelocity::default(),
        ExternalForce::default(),
        ExternalTorque::default(),
        Collider::cuboid(2.0, 1.0, 4.0),
        PlayerInput::default(),
        FlightCamera::default(),
        AfterburnerParticles::default(),  // ← ADD THIS
    ))
    .insert(FlightControlComputer::default())
    .insert(DiagnosticTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
    .insert(LastShotTime::default())
    .id();

    // ... rest of spawn code ...

    // DELETE: The old geometry-based flame spawning code
    // Remove: parent.spawn((MeshMaterial3d, ...)) for the orange rectangle
}
```

---

### Step 6: Update System Schedule

**In `main()`, find the Update systems:**

```rust
.add_systems(Update, (
    read_player_input,
    arcade_flight_physics,
    check_ground_collision,
    update_flight_camera,
    spawn_afterburner_particles,  // ← ADD THIS
    update_particles,             // ← ADD THIS (must run AFTER spawn)
    update_missile_system,
    check_win_condition,
    debug_flight_diagnostics,
    // ... rest
))
```

**Order matters:**
1. `spawn_afterburner_particles` creates new particles
2. `update_particles` moves and fades existing particles

---

## Configuration Options

### Throttle Scaling Strategy

**Current approach:** More particles spawned as throttle increases

```rust
let actual_spawn_rate = emitter.spawn_rate * throttle_factor;
let size = 0.5 + throttle_factor * 0.8;
```

**Optional: Add color change for boost**

```rust
let material_color = if input.throttle > 0.8 {
    Color::srgba(1.0, 0.8, 0.3, opacity)  // Orange when boosting
} else {
    Color::srgba(1.0, 0.6, 0.2, opacity)  // Yellow-red normal
};
```

### Tuning Parameters (adjust for desired look)

```rust
// In AfterburnerParticles::default():
spawn_rate: 5.0,              // Particles per frame at full throttle
spawn_threshold: 0.2,         // Minimum throttle to show (0-1)
particle_lifetime: 0.8,       // How long each particle lasts (seconds)

// In spawn_afterburner_particles():
let backward_velocity = -20.0;    // How fast particles move back
let upward_bias = 3.0;            // How much they drift up
let size = 0.5 + throttle_factor * 0.8;  // Size range

// In update_particles():
let drift = Vec3::Y * 5.0;  // Upward drift speed
```

### Recommended Starting Values

| Parameter | Value | Effect |
|-----------|-------|--------|
| `spawn_rate` | 5.0 | How many per frame at full throttle |
| `spawn_threshold` | 0.2 (20%) | When flames start appearing |
| `particle_lifetime` | 0.8 sec | How long each particle lives |
| `backward_velocity` | -20.0 | Trail speed behind plane |
| `upward_bias` | 3.0 | How much particles drift up |
| `base_size` | 0.5 | Minimum particle size |
| `size_range` | 0.8 | How much size varies with throttle |

---

## Testing Checklist

After implementation, test:

- [ ] **Game launches** - No compilation errors
- [ ] **Zero throttle** - No particles visible
- [ ] **20% throttle** - Tiny flames appear at rear
- [ ] **50% throttle** - Moderate flame trail behind plane
- [ ] **100% throttle** - Large, bright flame burst
- [ ] **Movement** - Particles move away from plane and drift upward
- [ ] **Fade** - Particles fade as they age
- [ ] **Performance** - No lag with 50+ particles active
- [ ] **Boost** (Shift key) - Flames get larger/brighter
- [ ] **Camera** - Doesn't clip into flame particles
- [ ] **Rotation** - Flames appear at rear regardless of plane orientation

---

## Troubleshooting

### Particles don't appear
- Check: Are textures loaded? (`assets/particles/flame_*.png` exist?)
- Check: Is `spawn_rate > 0`?
- Check: Is `spawn_threshold` low enough?
- Check: Is `input.throttle > spawn_threshold`?

### Particles appear at wrong location
- Adjust `model_space_pos` vector (Z = 3.5 is rear of plane)
- Check: Model orientation (might need different offset)

### Particles move too fast/slow
- Adjust `backward_velocity` (negative = backward, larger = faster)
- Adjust `drift` upward movement

### Performance issues
- Reduce `spawn_rate` (fewer particles per frame)
- Reduce `particle_lifetime` (particles die faster)
- Reduce texture resolution (consider scaling down PNG files)

### Wrong colors/brightness
- Adjust `emissive` color (currently `Color::srgb(2.0, 1.5, 0.5)`)
- Adjust `base_color` alpha (currently `0.9`)
- Try different flame texture images

---

## File Dependencies

**Files to modify:**
- `src/main.rs` - Add components, systems, update schedule

**Files to copy:**
- `assets/particles/flame_01.png` through `flame_05.png`
- `assets/particles/smoke_03.png`, `smoke_05.png` (optional)
- `assets/particles/fire_01.png`, `fire_02.png` (optional)

**Files to delete:**
- Remove old `update_afterburner_flame()` system
- Remove old flame geometry spawning from `spawn_player()`
- Remove old `AfterburnerFlame` component

---

## Next Steps

1. **Copy particle textures** to `assets/particles/`
2. **Remove old flame code** (geometry, system, component)
3. **Add new particle components** (AfterburnerParticles, Particle)
4. **Implement spawn system** (spawn_afterburner_particles)
5. **Implement update system** (update_particles)
6. **Update schedule** to call both systems
7. **Test** with various throttle levels
8. **Tune** spawn_rate, lifetime, size, etc. for desired look

---

## Success Criteria

✅ Flames appear at rear of jet (not on side)
✅ Flames scale with throttle (0% = none, 100% = large)
✅ Particles fade out with age
✅ No significant performance impact
✅ Works during all flight maneuvers
✅ Camera doesn't clip into particles
