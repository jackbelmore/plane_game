# Particle Effects - Implementation Code

Due to file formatting issues during automated edits, here's the exact code to add manually to `src/main.rs`.

## Step 1: Components (Replace lines 9-15)

**Find and delete:**
```rust
/// Afterburner flame effect component
#[derive(Component)]
struct AfterburnerFlame {
    flame_textures: Vec<Handle<Image>>,
    timer: Timer,
    current_frame: usize,
}
```

**Replace with:**
```rust
/// Afterburner particle emitter component
#[derive(Component)]
struct AfterburnerParticles {
    spawn_rate: f32,
    spawn_threshold: f32,
    particle_lifetime: f32,
}

impl Default for AfterburnerParticles {
    fn default() -> Self {
        Self {
            spawn_rate: 5.0,
            spawn_threshold: 0.2,
            particle_lifetime: 0.8,
        }
    }
}

/// Individual particle component
#[derive(Component)]
struct Particle {
    lifetime_remaining: f32,
    lifetime_max: f32,
    velocity: Vec3,
}
```

## Step 2: Update Schedule (Replace old flame system)

**Find this line in main():**
```rust
update_afterburner_flame, // Flame visual effect based on throttle
```

**Replace with:**
```rust
spawn_afterburner_particles, // Particle spawning based on throttle
update_particles, // Update particle positions and fade
```

## Step 3: Update Player Spawn

**In `spawn_player()` function, find:**
```rust
.insert(LastShotTime::default())
.id();

commands.entity(player)
.with_children(|parent| {
    parent.spawn((
        ModelContainer,
        // ...model code...
    ));

    // OLD FLAME GEOMETRY - DELETE THIS SECTION:
    let flame_mesh = meshes.add(Rectangle::new(0.4, 0.6));
    let flame_material = materials.add(StandardMaterial {
        // ... material code ...
    });
    // ... two parent.spawn calls for flames ...
});
```

**Replace with:**
```rust
.insert(LastShotTime::default())
.insert(AfterburnerParticles::default())
.id();

commands.entity(player)
.with_children(|parent| {
    parent.spawn((
        ModelContainer,
        // Scale down model to fit game
        Transform::from_scale(Vec3::splat(0.08))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        SceneRoot(model_handle),
    ));
});
```

## Step 4: Add Particle Systems

**At the END of the file, before the last closing brace, add:**

```rust
/// Spawn particle effects from jet exhaust based on throttle
fn spawn_afterburner_particles(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Transform, &PlayerInput), With<PlayerPlane>>,
    emitter_query: Query<&AfterburnerParticles, With<PlayerPlane>>,
) {
    if let (Ok((player_transform, input)), Ok(emitter)) = (player_query.get_single(), emitter_query.get_single()) {
        if input.throttle < emitter.spawn_threshold {
            return;
        }

        let throttle_factor = (input.throttle - emitter.spawn_threshold) / (1.0 - emitter.spawn_threshold);
        let actual_spawn_rate = emitter.spawn_rate * throttle_factor;

        for _ in 0..(actual_spawn_rate as u32) {
            let model_space_pos = Vec3::new(0.0, -0.2, 3.5);
            let world_pos: Vec3 = player_transform.transform_point(model_space_pos);

            let backward_velocity = player_transform.forward().as_vec3() * -20.0;
            let random_spread = Vec3::new(
                (rand::random::<f32>() - 0.5) * 5.0,
                (rand::random::<f32>() - 0.5) * 5.0 + 3.0,
                (rand::random::<f32>() - 0.5) * 5.0,
            );
            let velocity = backward_velocity + random_spread;

            let flame_index = ((time.elapsed_secs() * 5.0) as usize) % 3 + 1;
            let texture_path = format!("particles/flame_0{}.png", flame_index);
            let texture_handle = asset_server.load(&texture_path);

            let material = StandardMaterial {
                base_color_texture: Some(texture_handle),
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.9),
                emissive: LinearRgba::rgb(2.0, 1.5, 0.5),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            };

            let material_handle = materials.add(material);
            let size = 0.5 + throttle_factor * 0.8;
            let quad_mesh = meshes.add(Mesh::from(Rectangle::new(size, size)));

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
                Mesh3d(quad_mesh),
            ));
        }
    }
}

/// Update particles: movement, fade, despawn
fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut transform, mut particle, material_handle) in &mut particle_query {
        particle.lifetime_remaining -= time.delta_secs();

        if particle.lifetime_remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let drift = Vec3::Y * 5.0 * time.delta_secs();
        transform.translation += particle.velocity * time.delta_secs() + drift;

        let opacity = particle.lifetime_remaining / particle.lifetime_max;
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color.set_alpha(opacity);
        }

        transform.rotate_y(2.0 * time.delta_secs());
    }
}
```

## Step 5: Delete Old Function

**Find and DELETE the entire `update_afterburner_flame` function** (search for "fn update_afterburner_flame")

## Build & Test

```bash
cd C:\Users\Box\plane_game
cargo build --release
cargo run --release
```

Test:
1. Hold Shift to increase throttle
2. At 20%+ throttle - small flames appear at rear
3. At 100% throttle - large flames with boost effect
4. Flames trail behind plane and fade away
5. Scale smoothly with throttle

---

## Summary

✅ Camera fix: DONE (direct position, smooth rotation)
✅ Particle effects: READY FOR MANUAL IMPLEMENTATION
✅ Boost multiplier: Already working (3.5x at 80%+ throttle)

Next priority: Manual implementation or full code rebuild if automated edits continue failing.
