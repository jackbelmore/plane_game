use bevy::prelude::*;
use crate::{PlayerPlane, GameState};
use avian3d::prelude::*;

#[derive(Component)]
pub struct Drone {
    pub health: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct KamikazeBehavior;

pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_drones.run_if(in_state(GameState::Playing)));
    }
}

/// Spawns the Kamikaze drone using the drone model mesh, with a dark gray fallback if it fails.
pub fn spawn_beaver_drone(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
) {
    println!("DEBUG: Spawning drone at {:?}", position);

    // Load drone model using SceneRoot (most compatible with Bevy 0.15 GLB)
    let drone_scene_handle = asset_server.load("models/drone.glb#Scene0");

    // Parent entity with physics and AI
    commands.spawn((
        Drone {
            health: 50.0,
            speed: 130.0, 
        },
        KamikazeBehavior,
        Transform {
            translation: position,
            scale: Vec3::splat(1.0), // Standard scale for now
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
        },
        InheritedVisibility::default(),
        RigidBody::Dynamic,
        Collider::cuboid(18.0, 9.0, 24.0), // Doubled again (was 9, 4.5, 12)
    ))
    .with_children(|parent| {
        // The actual 3D model (SceneRoot)
        parent.spawn((
            SceneRoot(drone_scene_handle),
            Transform::from_scale(Vec3::splat(15.0)), // Doubled again (was 7.5)
        ));

        // VISUAL FALLBACK: Simple Red Cube (guarantees visibility)
        // This is crucial because Bevy 0.15 SceneRoot loading can be invisible if the GLB hierarchy isn't perfect
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 3.0))), // Drone-sized box
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.1, 0.1), // Dark red
                emissive: LinearRgba::rgb(2.0, 0.0, 0.0), // Red glow
                ..default()
            })),
            Transform::IDENTITY,
        ));
    });
}

/// Movement system: Drones pursue the player using advanced Swarm Intelligence
fn move_drones(
    mut commands: Commands,
    time: Res<Time>,
    mut drone_query: Query<(Entity, &mut Transform, &Drone), (With<KamikazeBehavior>, Without<PlayerPlane>)>,
    player_query: Query<(&Transform, &LinearVelocity), With<PlayerPlane>>,
    meteor_query: Query<&Transform, (With<crate::Meteor>, Without<Drone>)>,
) {
    let Ok((player_transform, player_velocity)) = player_query.get_single() else { return };
    let player_pos = player_transform.translation;
    let delta_secs = time.delta_secs();
    let elapsed = time.elapsed_secs();

    // SAFETY: Validate delta time
    if !delta_secs.is_finite() || delta_secs <= 0.0 {
        return;
    }

    // Pre-collect drone and meteor data for efficient calculation
    let drone_data: Vec<(Entity, Vec3, Quat)> = drone_query
        .iter()
        .map(|(e, t, _)| (e, t.translation, t.rotation))
        .collect();
        
    // Only check nearby meteors (within 1km of player) to save performance
    let nearby_meteors: Vec<Vec3> = meteor_query
        .iter()
        .filter(|t| t.translation.distance(player_pos) < 2000.0)
        .map(|t| t.translation)
        .collect();

    for (entity, mut transform, drone) in &mut drone_query {
        if !transform.translation.is_finite() { continue; }

        let mut steering_force = Vec3::ZERO;

        // --- 1. LEAD PURSUIT (The Goal) ---
        let lead_time = 1.8; // Increased lead time for better interception
        let target_pos = player_pos + (player_velocity.0 * lead_time);
        let pursuit_dir = (target_pos - transform.translation).normalize_or_zero();
        
        // Increase pursuit weight significantly to overcome other forces
        steering_force += pursuit_dir * 3.5; 

        // Add a secondary force directly towards the player for persistence
        let direct_dir = (player_pos - transform.translation).normalize_or_zero();
        steering_force += direct_dir * 1.5;

        // --- 2. OBSTACLE AVOIDANCE (Meteors) ---
        for meteor_pos in &nearby_meteors {
            let dist = transform.translation.distance(*meteor_pos);
            if dist < 250.0 { // Avoidance radius
                // Repel from meteor
                let repel_dir = (transform.translation - *meteor_pos).normalize_or_zero();
                let intensity = (250.0 - dist) / 250.0;
                steering_force += repel_dir * intensity * 4.0; // Strong desire: don't crash
            }
        }

        // --- 3. SWARM BEHAVIOR (Flocking) ---
        let mut separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut neighbors = 0;

        for (other_entity, other_pos, other_rot) in &drone_data {
            if entity == *other_entity { continue; }
            
            let dist = transform.translation.distance(*other_pos);
            if dist < 400.0 { // Swarm radius
                neighbors += 1;
                // Separation: Keep personal space
                if dist < 120.0 {
                    separation += (transform.translation - *other_pos).normalize_or_zero() * (120.0 - dist) / 120.0;
                }
                // Alignment: Face same way as group
                alignment += other_rot.mul_vec3(Vec3::Z);
                // Cohesion: Move toward group center
                cohesion += *other_pos;
            }
        }

        if neighbors > 0 {
            steering_force += separation * 2.5;
            steering_force += (alignment / neighbors as f32).normalize_or_zero() * 0.5;
            
            let center = cohesion / neighbors as f32;
            steering_force += (center - transform.translation).normalize_or_zero() * 0.3;
        }

        // --- 4. TACTICAL WEAVING ---
        let weave_freq = 0.8 + (entity.index() % 5) as f32 * 0.1;
        let weave_amp = 15.0;
        let weave_offset = transform.right() * (elapsed * weave_freq).sin() * weave_amp;
        
        // --- 5. FINAL DIRECTION & ROTATION ---
        if steering_force != Vec3::ZERO {
            let final_dir = steering_force.normalize_or_zero();
            // SIGNIFICANTLY REDUCED TURN SPEED for realism (was 4.5)
            let turn_speed = 1.2 * delta_secs;
            
            // Calculate desired rotation (Pitch and Yaw)
            let mut target_transform = transform.clone();
            target_transform.look_at(transform.translation + final_dir + weave_offset * 0.01, Vec3::Y);
            
            // Calculate Banking (Roll) based on horizontal turning intensity
            let local_steering = transform.rotation.inverse().mul_vec3(final_dir);
            let banking_amount = -local_steering.x * 0.8; // Tilt into the turn
            let banking_quat = Quat::from_rotation_z(banking_amount);
            
            let final_target_rot = target_transform.rotation * banking_quat;
            transform.rotation = transform.rotation.slerp(final_target_rot, turn_speed);
        }

        // --- 6. MOVEMENT & CATCH-UP ---
        let forward = transform.forward();
        let distance_to_player = transform.translation.distance(player_pos);
        
        // DYNAMIC SPEED MULTIPLIER
        // If player flies off, drones enter "Warp Pursuit"
        let speed_mult = if distance_to_player > 5000.0 {
            8.0 // Warp: ~1040 m/s
        } else if distance_to_player > 2000.0 {
            5.0 // Sprint: ~650 m/s
        } else if distance_to_player < 1200.0 {
            3.2 // Combat: ~416 m/s (F-16 Combat Speed)
        } else {
            2.0 // Standard
        };
        
        let move_vec = forward * drone.speed * speed_mult * delta_secs;
        let new_pos = transform.translation + move_vec;

        // --- 7. LIFETIME MANAGEMENT ---
        // Despawn if way too far (15km) to save performance
        if distance_to_player > 15000.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        if new_pos.is_finite() {
            transform.translation = new_pos;
        }
    }
}
