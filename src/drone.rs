use bevy::prelude::*;
use avian3d::prelude::*;
use crate::{PlayerPlane, GameState, Meteor};

// ============================================================================
// RESOURCES
// ============================================================================

/// Tracks active missiles in flight to prevent overwhelming the player
#[derive(Resource)]
pub struct CombatDirector {
    pub active_missiles: usize,
    pub max_missiles: usize,
}

impl Default for CombatDirector {
    fn default() -> Self {
        Self {
            active_missiles: 0,
            max_missiles: 2,
        }
    }
}

// ============================================================================
// COMPONENTS
// ============================================================================

/// Drone component with health and speed stats
#[derive(Component, Reflect)]
pub struct Drone {
    pub health: f32,
    pub speed: f32,  // Base speed in m/s (e.g., 130.0)
}

impl Default for Drone {
    fn default() -> Self {
        Self {
            health: 100.0,
            speed: 130.0,  // Restored from old working version
        }
    }
}

/// Marker for kamikaze behavior (old AI pattern)
#[derive(Component)]
pub struct KamikazeBehavior;

/// State machine for drone AI behavior
#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DroneState {
    Patrol,
    Intercept,
    AttackRun,
    Evasive,
}

/// Physics tuning parameters for drone flight control
#[derive(Component)]
pub struct DronePhysics {
    pub pid_p: f32,
    pub pid_d: f32,
    pub max_speed: f32,
}

impl Default for DronePhysics {
    fn default() -> Self {
        Self { pid_p: 5.0, pid_d: 2.0, max_speed: 130.0 }
    }
}

/// Weapon system for drones
#[derive(Component, Reflect)]
pub struct DroneWeapons {
    pub missile_cooldown: Timer,
    pub gun_cooldown: Timer,
}

impl Default for DroneWeapons {
    fn default() -> Self {
        Self {
            missile_cooldown: Timer::from_seconds(2.0, TimerMode::Once),
            gun_cooldown: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

/// Marker component for missile projectiles
#[derive(Component, Reflect)]
pub struct Missile {
    pub remaining_lifetime: f32,
}

/// Marker component for bullet projectiles
#[derive(Component, Reflect)]
pub struct Bullet {
    pub remaining_lifetime: f32,
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatDirector>()
            .register_type::<Drone>()
            .register_type::<DroneWeapons>()
            .register_type::<Missile>()
            .register_type::<Bullet>()
            .add_systems(
                Update,
                (
                    move_drones,              // Old working AI (restored)
                    drone_weapon_system,      // Weapon firing
                    drone_projectile_system,  // Projectile lifetime
                    missile_cleanup,          // Sync active missile count
                )
                    .chain(),
            );
    }
}

/// Helper function to spawn a drone entity with all required components
pub fn spawn_drone(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
) {
    let drone_scene_handle = asset_server.load("models/drone.glb#Scene0");
    
    commands.spawn((
        Drone::default(),
        KamikazeBehavior,
        DroneWeapons::default(),
        DroneState::Patrol,
        DronePhysics::default(),
        Transform {
            translation: position,
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            scale: Vec3::splat(1.0),
        },
        InheritedVisibility::default(),
        RigidBody::Dynamic,
        Collider::sphere(2.0),
        GravityScale(0.0),
        LinearVelocity::default(),
        AngularVelocity::ZERO,
        ExternalForce::default(),
        ExternalTorque::default(),
    ))
    .with_children(|parent| {
        // SceneRoot for 3D model
        parent.spawn((
            SceneRoot(drone_scene_handle),
            Transform::from_scale(Vec3::splat(15.0)),
        ));
        
        // VISUAL FALLBACK: Smaller red cube (semi-transparent to see model through)
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(3.0, 1.5, 4.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.8, 0.1, 0.1, 0.5), // Semi-transparent red
                emissive: LinearRgba::rgb(2.0, 0.0, 0.0),
                ..default()
            })),
            Transform::IDENTITY,
        ));
    });
}

/// Alias for compatibility with existing code that calls spawn_beaver_drone
pub fn spawn_beaver_drone(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
) {
    spawn_drone(commands, asset_server, meshes, materials, position);
}

// ============================================================================
// AI SYSTEM (Restored from git 7de359c)
// ============================================================================

/// Movement system: Drones pursue the player using advanced Swarm Intelligence
fn move_drones(
    mut commands: Commands,
    time: Res<Time>,
    mut drone_query: Query<(Entity, &mut Transform, &Drone), (With<KamikazeBehavior>, Without<PlayerPlane>)>,
    player_query: Query<(&Transform, &LinearVelocity), With<PlayerPlane>>,
    meteor_query: Query<&Transform, (With<Meteor>, Without<Drone>)>,
) {
    let Ok((player_transform, player_velocity)) = player_query.get_single() else { return };
    let player_pos = player_transform.translation;
    let delta_secs = time.delta_secs();
    let elapsed = time.elapsed_secs();

    if !delta_secs.is_finite() || delta_secs <= 0.0 { return; }

    let drone_data: Vec<(Entity, Vec3, Quat)> = drone_query
        .iter()
        .map(|(e, t, _)| (e, t.translation, t.rotation))
        .collect();

    let nearby_meteors: Vec<Vec3> = meteor_query
        .iter()
        .filter(|t| t.translation.distance(player_pos) < 2000.0)
        .map(|t| t.translation)
        .collect();

    for (entity, mut transform, drone) in &mut drone_query {
        if !transform.translation.is_finite() { continue; }

        let mut steering_force = Vec3::ZERO;

        // Lead pursuit with altitude matching (Phase 2: Tactical Intelligence)
        let lead_time = 1.8;
        let mut target_pos = player_pos + (player_velocity.0 * lead_time);
        
        // ALTITUDE TACTICS (Harfang3D research)
        // When in combat range (< 3km), gradually match player altitude
        let distance_to_player = transform.translation.distance(player_pos);
        if distance_to_player < 3000.0 {
            let altitude_delta = (target_pos.y - transform.translation.y) / 10.0;  // Gradual approach (10% per frame)
            target_pos.y = transform.translation.y + altitude_delta;
        }
        
        let pursuit_dir = (target_pos - transform.translation).normalize_or_zero();
        steering_force += pursuit_dir * 3.5;

        let direct_dir = (player_pos - transform.translation).normalize_or_zero();
        steering_force += direct_dir * 1.5;

        // Obstacle avoidance
        for meteor_pos in &nearby_meteors {
            let dist = transform.translation.distance(*meteor_pos);
            if dist < 250.0 {
                let repel_dir = (transform.translation - *meteor_pos).normalize_or_zero();
                let intensity = (250.0 - dist) / 250.0;
                steering_force += repel_dir * intensity * 4.0;
            }
        }

        // Swarm behavior
        let mut separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut neighbors = 0;

        for (other_entity, other_pos, other_rot) in &drone_data {
            if entity == *other_entity { continue; }
            let dist = transform.translation.distance(*other_pos);
            if dist < 400.0 {
                neighbors += 1;
                if dist < 120.0 {
                    separation += (transform.translation - *other_pos).normalize_or_zero() * (120.0 - dist) / 120.0;
                }
                alignment += other_rot.mul_vec3(Vec3::Z);
                cohesion += *other_pos;
            }
        }

        if neighbors > 0 {
            steering_force += separation * 2.5;
            steering_force += (alignment / neighbors as f32).normalize_or_zero() * 0.5;
            let center = cohesion / neighbors as f32;
            steering_force += (center - transform.translation).normalize_or_zero() * 0.3;
        }

        // Tactical weaving
        let weave_freq = 0.8 + (entity.index() % 5) as f32 * 0.1;
        let weave_amp = 15.0;
        let weave_offset = transform.right() * (elapsed * weave_freq).sin() * weave_amp;

        // ALTITUDE SAFETY (Phase 2: Harfang3D research)
        // Emergency climb if too low, emergency descend if too high
        let current_altitude = transform.translation.y;
        if current_altitude < 100.0 {
            // DANGER: Too low - emergency climb
            let climb_force = Vec3::Y * 5.0;
            steering_force += climb_force;
        } else if current_altitude > 10000.0 {
            // DANGER: Too high - descend
            let descend_force = Vec3::NEG_Y * 3.0;
            steering_force += descend_force;
        }

        // Rotation
        if steering_force != Vec3::ZERO {
            let final_dir = steering_force.normalize_or_zero();
            let turn_speed = 1.2 * delta_secs;
            let mut target_transform = transform.clone();
            target_transform.look_at(transform.translation + final_dir + weave_offset * 0.01, Vec3::Y);

            let local_steering = transform.rotation.inverse().mul_vec3(final_dir);
            let banking_amount = -local_steering.x * 0.8;
            let banking_quat = Quat::from_rotation_z(banking_amount);

            let final_target_rot = target_transform.rotation * banking_quat;
            transform.rotation = transform.rotation.slerp(final_target_rot, turn_speed);
        }

        // Movement with tactical speed control (Phase 2: Safety + Tactics)
        let forward = transform.forward();
        let distance_to_player = transform.translation.distance(player_pos);

        // TACTICAL ZONES (based on Harfang3D research)
        // Long-range (>3km): Boost to intercept
        // Combat range (800m-3km): Match speed for maneuvering  
        // Attack range (300m-800m): Slow down to aim weapons
        // Danger zone (<300m): Speed up to avoid collision
        
        let speed_mult = if distance_to_player > 5000.0 { 
            8.0  // Warp pursuit - catch up fast
        } else if distance_to_player > 3000.0 { 
            5.0  // Sprint - close distance
        } else if distance_to_player > 800.0 { 
            2.2  // Combat speed - maneuver for shots
        } else if distance_to_player > 300.0 { 
            1.5  // Attack speed - slow for accuracy
        } else { 
            3.0  // Danger zone - speed up to avoid ram
        };

        let move_vec = forward * drone.speed * speed_mult * delta_secs;
        transform.translation += move_vec;

        if distance_to_player > 15000.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}


// WEAPON SYSTEM
// ============================================================================

/// Handle drone weapon firing logic
fn drone_weapon_system(
    mut drone_query: Query<
        (
            Entity,
            &GlobalTransform,
            &Transform,
            &LinearVelocity,
            &mut DroneWeapons,
        ),
        With<Drone>,
    >,
    player_query: Query<&GlobalTransform, With<PlayerPlane>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut combat_director: ResMut<CombatDirector>,
    time: Res<Time>,
) {
    // DIAGNOSTIC: Confirm system runs
    static mut FIRST_RUN: bool = true;
    unsafe {
        if FIRST_RUN {
            eprintln!("✅ WEAPON SYSTEM ACTIVATED");
            FIRST_RUN = false;
        }
    }
    
    let Ok(player_transform) = player_query.get_single() else { 
        // More visible error message
        eprintln!("❌ WEAPON: No player found (query returned {} results)", player_query.iter().count());
        return 
    };
    let player_pos = player_transform.translation();

    let mut drones_checked = 0;
    let mut missiles_fired = 0;
    let mut guns_fired = 0;
    
    for (entity, drone_transform, transform, drone_velocity, mut weapons) in drone_query.iter_mut() {
        drones_checked += 1;
        
        let drone_pos = drone_transform.translation();
        let distance = (player_pos - drone_pos).length();
        
        let drone_forward = transform.forward();
        let to_player = (player_pos - drone_pos).normalize_or_zero();
        let angle_rad = drone_forward.dot(to_player).acos().abs();
        let angle_deg = angle_rad.to_degrees();

        // DIAGNOSTIC: Detailed logging every second for first drone
        let should_log = entity.index() == 0 && (time.elapsed_secs() as u32) != (time.elapsed_secs() - time.delta_secs()) as u32;
        
        if should_log {
            eprintln!("\n━━━ DRONE DIAGNOSTIC ━━━");
            eprintln!("🎯 Drone #{} status:", entity.index());
            eprintln!("   Distance: {:.0}m (missile range: 800-2000m, gun range: <1000m)", distance);
            eprintln!("   Angle: {:.1}° (missile max: 45°, gun max: 10°)", angle_deg);
            eprintln!("   Missile cooldown: {:.2}s remaining (fires at 0.0s)", weapons.missile_cooldown.remaining_secs());
            eprintln!("   Gun cooldown: {:.2}s remaining (fires at 0.0s)", weapons.gun_cooldown.remaining_secs());
            eprintln!("   Combat Director: {}/{} missiles active", combat_director.active_missiles, combat_director.max_missiles);
        }

        // Update cooldown timers
        weapons.missile_cooldown.tick(time.delta());
        weapons.gun_cooldown.tick(time.delta());

        // MISSILE FIRING (medium-long range, relaxed angle)
        let missile_range_ok = distance > 800.0 && distance < 2000.0;
        let missile_angle_ok = angle_deg < 45.0;
        let missile_cooldown_ok = weapons.missile_cooldown.finished();
        let missile_slots_ok = combat_director.active_missiles < combat_director.max_missiles;
        
        // DIAGNOSTIC: Log why missiles DON'T fire
        if should_log {
            eprintln!("   Missile checks:");
            eprintln!("      Range (800-2000m): {} {}", 
                if missile_range_ok { "✅ PASS" } else { "❌ FAIL" },
                if !missile_range_ok { format!("({:.0}m is outside range)", distance) } else { String::new() }
            );
            eprintln!("      Angle (<45°): {} {}", 
                if missile_angle_ok { "✅ PASS" } else { "❌ FAIL" },
                if !missile_angle_ok { format!("({:.1}° too wide)", angle_deg) } else { String::new() }
            );
            eprintln!("      Cooldown: {} {}", 
                if missile_cooldown_ok { "✅ PASS" } else { "❌ FAIL" },
                if !missile_cooldown_ok { format!("({:.2}s remaining)", weapons.missile_cooldown.remaining_secs()) } else { String::new() }
            );
            eprintln!("      Slots available: {} {}", 
                if missile_slots_ok { "✅ PASS" } else { "❌ FAIL" },
                if !missile_slots_ok { format!("({}/{} full)", combat_director.active_missiles, combat_director.max_missiles) } else { String::new() }
            );
        }
        
        if missile_range_ok && missile_angle_ok && missile_cooldown_ok && missile_slots_ok {
            eprintln!("🚀 FIRING MISSILE [Drone {}] at {:.0}m, {:.1}°", entity.index(), distance, angle_deg);
            spawn_missile(&mut commands, &mut meshes, &mut materials, drone_pos, *drone_forward, drone_velocity.0);
            combat_director.active_missiles += 1;
            weapons.missile_cooldown.reset();
            missiles_fired += 1;
        }
        // GUN FIRING (close range, tight angle)
        else {
            let gun_range_ok = distance < 1000.0;
            let gun_angle_ok = angle_deg < 10.0;
            let gun_cooldown_ok = weapons.gun_cooldown.finished();
            
            // DIAGNOSTIC: Log why guns DON'T fire
            if should_log {
                eprintln!("   Gun checks:");
                eprintln!("      Range (<1000m): {} {}", 
                    if gun_range_ok { "✅ PASS" } else { "❌ FAIL" },
                    if !gun_range_ok { format!("({:.0}m too far)", distance) } else { String::new() }
                );
                eprintln!("      Angle (<10°): {} {}", 
                    if gun_angle_ok { "✅ PASS" } else { "❌ FAIL" },
                    if !gun_angle_ok { format!("({:.1}° too wide)", angle_deg) } else { String::new() }
                );
                eprintln!("      Cooldown: {} {}", 
                    if gun_cooldown_ok { "✅ PASS" } else { "❌ FAIL" },
                    if !gun_cooldown_ok { format!("({:.2}s remaining)", weapons.gun_cooldown.remaining_secs()) } else { String::new() }
                );
            }
            
            if gun_range_ok && gun_angle_ok && gun_cooldown_ok {
                eprintln!("🔫 FIRING GUN [Drone {}] at {:.0}m, {:.1}°", entity.index(), distance, angle_deg);
                spawn_bullet(&mut commands, &mut meshes, &mut materials, drone_pos, *drone_forward, drone_velocity.0);
                weapons.gun_cooldown.reset();
                guns_fired += 1;
            }
        }
    }
    
    // Summary log every 5 seconds
    if (time.elapsed_secs() as u32) % 5 == 0 && time.delta_secs() < 0.2 {
        eprintln!(
            "\n📊 WEAPON SUMMARY: {} drones, {} missiles fired this cycle, {} guns fired",
            drones_checked, missiles_fired, guns_fired
        );
    }
}

/// Spawn a missile projectile
fn spawn_missile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    origin: Vec3,
    direction: Vec3,
    launch_velocity: Vec3,
) {
    let missile_velocity = direction * 100.0 + launch_velocity * 0.2;

    commands.spawn((
        Missile {
            remaining_lifetime: 10.0,
        },
        Transform {
            translation: origin + direction * 5.0,
            rotation: Quat::from_rotation_arc(Vec3::Z, direction),
            scale: Vec3::splat(0.5),
        },
        GlobalTransform::default(),
        // Capsule shape for missile
        Mesh3d(meshes.add(Capsule3d::new(0.2, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.5, 0.0), // Orange
            emissive: LinearRgba::rgb(1.0, 0.3, 0.0),
            ..default()
        })),
        RigidBody::Dynamic,
        LinearVelocity(missile_velocity),
        AngularVelocity::ZERO,
        Collider::sphere(0.3),
        ExternalForce::default(),
        ExternalTorque::default(),
    ));
}

/// Spawn a bullet projectile
fn spawn_bullet(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    origin: Vec3,
    direction: Vec3,
    launch_velocity: Vec3,
) {
    let bullet_velocity = direction * 150.0 + launch_velocity * 0.1;

    commands.spawn((
        Bullet {
            remaining_lifetime: 5.0,
        },
        Transform {
            translation: origin + direction * 3.0,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(0.2),
        },
        GlobalTransform::default(),
        // Sphere shape for bullet
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0), // Yellow
            emissive: LinearRgba::rgb(1.0, 1.0, 0.0),
            ..default()
        })),
        RigidBody::Dynamic,
        LinearVelocity(bullet_velocity),
        AngularVelocity::ZERO,
        Collider::sphere(0.1),
        ExternalForce::default(),
        ExternalTorque::default(),
    ));
}

// ============================================================================
// PROJECTILE LOGIC
// ============================================================================

/// Update projectile behavior and handle cleanup
fn drone_projectile_system(
    mut missile_query: Query<(Entity, &GlobalTransform, &mut Missile, &mut LinearVelocity), Without<Bullet>>,
    mut bullet_query: Query<(Entity, &mut Bullet), Without<Missile>>,
    player_query: Query<&GlobalTransform, With<PlayerPlane>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    // === Missiles (Pure Pursuit) ===
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation();
    let delta = time.delta().as_secs_f32();

    for (missile_entity, missile_transform, mut missile, mut velocity) in missile_query.iter_mut() {
        let missile_pos = missile_transform.translation();

        // Pure pursuit: steer directly at player
        let to_player = (player_pos - missile_pos).normalize_or_zero();
        let steering_force = to_player * 5.0; // Aggressive steering

        // Update velocity
        velocity.0 = (velocity.0 + steering_force).clamp_length_max(150.0);

        // Update lifetime
        missile.remaining_lifetime -= delta;

        // Despawn on contact or timeout (contact handled by Avian3D collision events)
        if missile.remaining_lifetime <= 0.0 {
            commands.entity(missile_entity).despawn();
        }
    }

    // === Bullets (Straight Line) ===
    for (bullet_entity, mut bullet) in bullet_query.iter_mut() {
        bullet.remaining_lifetime -= delta;

        if bullet.remaining_lifetime <= 0.0 {
            commands.entity(bullet_entity).despawn();
        }
    }
}

/// Clean up missiles when they despawn, decrementing the active missile counter
fn missile_cleanup(
    mut missile_query: Query<&Missile>,
    mut combat_director: ResMut<CombatDirector>,
) {
    // Count remaining active missiles
    let active_count = missile_query.iter().count();

    // If count changed, update the resource
    if active_count < combat_director.active_missiles {
        combat_director.active_missiles = active_count;
    }
}
