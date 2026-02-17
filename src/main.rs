use bevy::{
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
    render::mesh::VertexAttributeValues,
    image::{ImageSampler, ImageAddressMode, ImageSamplerDescriptor},
    window::PrimaryWindow,
    winit::WinitWindows,
    core_pipeline::bloom::{Bloom, BloomCompositeMode},
};
use winit::window::Icon;
use avian3d::prelude::*;
use rand::prelude::*;
use std::io::Write;
use noise::{NoiseFn, Perlin}; // Added for terrain generation

mod drone;
mod ui; // NEW: HUD System
mod procedural_textures; // NEW: Procedural Grass Texture
mod assets; // NEW: Asset Loader
use bevy_asset_loader::prelude::*;
use assets::GameAssets;
use drone::{Drone, DronePlugin};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    Spawning, // One-time setup state
    Playing,
    Paused,
}

/// Generate terrain height using Perlin noise
fn get_terrain_height(world_x: f32, world_z: f32) -> f32 {
    let perlin = Perlin::new(42); // Fixed seed for deterministic terrain

    // Layer 1: Large rolling hills (500m wavelength, 100m amplitude - doubled for visibility)
    let scale_large = 0.002; // 1/500
    let height_large = perlin.get([world_x as f64 * scale_large, world_z as f64 * scale_large]) as f32 * 100.0;

    // Layer 2: Medium features (100m wavelength, 15m amplitude)
    let scale_medium = 0.01; // 1/100
    let height_medium = perlin.get([world_x as f64 * scale_medium + 100.0, world_z as f64 * scale_medium + 100.0]) as f32 * 15.0;

    // Layer 3: Small details (20m wavelength, 3m amplitude)
    let scale_small = 0.05; // 1/20
    let height_small = perlin.get([world_x as f64 * scale_small + 200.0, world_z as f64 * scale_small + 200.0]) as f32 * 3.0;

    // Combine layers (additive for natural-looking terrain)
    let total_height = height_large + height_medium + height_small;

    // Clamp to reasonable range (prevent crazy mountains)
    total_height.clamp(-50.0, 150.0) // Increased range for more dramatic terrain
}

// #region agent log
fn debug_log(location: &str, message: &str, data: &str, hypothesis_id: &str) {
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let line = format!(
        r#"{{"timestamp":{},"location":"{}","message":"{}","data":{},"sessionId":"debug-session","hypothesisId":"{}"}}"#,
        timestamp, esc(location), esc(message), data, hypothesis_id
    );
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(r"c:\Users\Box\plane_game\.cursor\debug.log") {
        let _ = writeln!(f, "{}", line);
    }
}
// #endregion

// ============================================================================
// COMPONENTS - Data containers for game entities
// ============================================================================

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

/// Chunk coordinate system
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
struct ChunkCoordinate {
    x: i32,
    z: i32,
}

impl ChunkCoordinate {
    fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / CHUNK_SIZE).floor() as i32,
            z: (pos.z / CHUNK_SIZE).floor() as i32,
        }
    }

    fn world_position(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE,
            0.0,
            self.z as f32 * CHUNK_SIZE,
        )
    }
}

#[derive(Component)]
struct ChunkEntity; // Tag for entities that belong to chunks

#[derive(Resource, Default)]
struct ChunkManager {
    loaded_chunks: std::collections::HashMap<ChunkCoordinate, Entity>,
    last_player_chunk: ChunkCoordinate,
}

// Constants
const CHUNK_SIZE: f32 = 1000.0; // 1km x 1km chunks
const LOAD_RADIUS_CHUNKS: i32 = 8; // Load chunks within 8km (fixes ground holes)
const UNLOAD_RADIUS_CHUNKS: i32 = 12; // Unload chunks beyond 12km

#[derive(Component)]
struct Tree;

#[derive(Component)]
struct LODLevel(u8); // 0=full detail, 1=medium, 2=low, 3=billboard

const TREES_PER_CHUNK_MIN: usize = 5;  // REDUCED (was 10)
const TREES_PER_CHUNK_MAX: usize = 10; // REDUCED (was 50)

/// Marker component for meteors
#[derive(Component)]
pub struct Meteor;

/// Marker component for mission objectives
#[derive(Component)]
struct Objective;

/// Enemy turret component
#[derive(Component)]
struct Turret {
    fire_timer: Timer,
}

/// Marker component to identify the player plane parent
#[derive(Component)]
pub struct PlayerPlane;

/// Marker for village buildings
#[derive(Component)]
struct VillageBuilding;

/// Marker for village roads
#[derive(Component)]
struct VillageRoad;

/// Marker for village decorations (fences, hedges, etc)
#[derive(Component)]
struct VillageDecoration;

/// Marker component for cloud entities
#[derive(Component)]
struct Cloud;

/// Marker component for the sky sphere
#[derive(Component)]
struct SkySphere;

/// Marker component for the infinite horizon disk
#[derive(Component)]
struct HorizonDisk;

/// Resource holding the shared ground material (loaded once at startup)
#[derive(Resource)]
struct GroundMaterial(Handle<StandardMaterial>);




/// Marker for the active engine sound entity
#[derive(Component)]
struct EngineSound;

/// Marker for the wind sound entity
#[derive(Component)]
struct WindSound;

/// Marker for the air rip / wing stress sound
#[derive(Component)]
struct WindStressSound;

/// Marker for the warning sound entity
#[derive(Component)]
struct WarningSound;

/// Component for manual audio attenuation (fading volume with distance)
/// Used to bypass Bevy's spatial audio limitations with stereo files
#[derive(Component)]
struct ManualAttenuation {
    max_distance: f32, // Not used for inverse square, but good for culling
    reference_distance: f32, // Distance where volume is 50%
    base_volume: f32,
    age: f32,
    doppler_ramp_time: f32, // Seconds to transition to full Doppler (preserves initial punch)
}

/// System to manually update volume based on distance to player
/// Solves "silent spatial audio" issues
fn update_manual_audio_attenuation(
    time: Res<Time>,
    player_query: Query<(&GlobalTransform, &LinearVelocity), With<PlayerPlane>>,
    mut audio_query: Query<(&GlobalTransform, &AudioSink, &mut ManualAttenuation, Option<&Parent>)>,
    velocity_query: Query<&LinearVelocity>,
) {
    let Ok((player_transform, player_velocity)) = player_query.get_single() else { return };
    let player_pos = player_transform.translation();

    let mut count = 0;
    for (transform, sink, mut settings, parent) in &mut audio_query {
        count += 1;
        settings.age += time.delta_secs();

        let sound_pos = transform.translation();
        let distance = player_pos.distance(sound_pos);
        
        // Skip processing if too far to save performance
        if distance > settings.max_distance {
            sink.set_volume(0.0);
            continue;
        }

        // 1. INVERSE SQUARE LAW (Realistic attenuation)
        // volume = ref / (ref + dist)
        // This ensures it never cuts off, but fades naturally forever
        let volume_fraction = settings.reference_distance / (settings.reference_distance + distance);
        sink.set_volume(settings.base_volume * volume_fraction);

        // 2. DOPPLER SHIFT
        // Need relative velocity along the line of sight
        let mut source_velocity = Vec3::ZERO;
        
        // Try to get velocity from parent (e.g. Missile)
        if let Some(parent_entity) = parent {
            if let Ok(vel) = velocity_query.get(parent_entity.get()) {
                source_velocity = vel.0;
            }
        }

        let direction_to_player = (player_pos - sound_pos).normalize_or_zero();
        let relative_velocity = source_velocity - player_velocity.0;
        let speed_towards_player = relative_velocity.dot(direction_to_player);

        // Speed of sound ~343 m/s
        // Pitch = 1.0 + (v / c)
        // Tune Doppler effect with a scaling factor (0.5 is standard "cinematic" value)
        const DOPPLER_SCALE: f32 = 0.5;
        let doppler_factor = (speed_towards_player / 343.0) * DOPPLER_SCALE;
        
        // Limit pitch shift to avoid craziness (0.5x to 2.0x) - Standard range
        // This prevents the sound from disappearing into sub-bass rumble
        let target_pitch = (1.0 + doppler_factor).clamp(0.5, 2.0);
        
        // Doppler Ramp: Blend from 1.0 (Normal) to Target (Doppler) based on age
        // This preserves the initial "Punch" of the sound before physics takes over
        let ramp_factor = (settings.age / settings.doppler_ramp_time).clamp(0.0, 1.0);
        
        // LERP: Start at 1.0, fade to target_pitch
        let current_pitch = 1.0 + (target_pitch - 1.0) * ramp_factor;
        
        sink.set_speed(current_pitch);

        // DEBUG: Print info for the first active sound only
        if count == 1 {
            println!("[AUDIO] Active: {} | Age: {:.2}s | Dist: {:.1}m | Pitch: {:.2}x (Tgt: {:.2}x)", 
                count, settings.age, distance, current_pitch, target_pitch);
        }
    }
}

/// System to handle high-G "Air Rip" sound during turns
fn update_maneuver_audio(
    player_query: Query<&AngularVelocity, With<PlayerPlane>>,
    mut audio_query: Query<&AudioSink, With<WindStressSound>>,
) {
    if let Ok(ang_vel) = player_query.get_single() {
        // High-G turn detection (rotation speed)
        let rotation_intensity = ang_vel.0.length();
        
        // Threshold: Starts becoming audible at 1.5 rad/s
        let target_volume = ((rotation_intensity - 1.5) * 0.6).clamp(0.0, 1.0);
        
        for sink in &mut audio_query {
            sink.set_volume(target_volume);
            // Increase pitch slightly as G-force increases
            sink.set_speed(0.8 + target_volume * 0.4);
        }
    }
}

/// Trigger cinematic "Boom" when afterburner kicks in
fn update_afterburner_audio(
    player_query: Query<&PlayerInput, With<PlayerPlane>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut prev_throttle: Local<f32>,
) {
    if let Ok(input) = player_query.get_single() {
        // If throttle just jumped into the "boost" zone (0.9+)
        if input.throttle > 0.9 && *prev_throttle <= 0.9 {
            commands.spawn((
                AudioPlayer(assets.afterburner_thump.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(1.5), // Loud thump
                    ..default()
                },
            ));
        }
        *prev_throttle = input.throttle;
    }
}

/// Marker for the container that holds the visual model
/// Allowing us to rotate the model (e.g. for banking) without affecting physics
#[derive(Component)]
struct ModelContainer;

/// Rocket mode state (Space travel)
#[derive(Component)]
struct RocketMode {
    enabled: bool,
}

impl Default for RocketMode {
    fn default() -> Self {
        Self { enabled: false }
    }
}

const ROCKET_THRUST_MULTIPLIER: f32 = 8.0;

/// Stores current player input state
#[derive(Component)]
struct PlayerInput {
    pitch: f32,
    roll: f32,
    yaw: f32,      // Added Yaw (Rudder)
    throttle: f32,
    _brake: f32,    // Airbrake
}

/// Timer for debug diagnostics
#[derive(Component)]
struct DiagnosticTimer(Timer);

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
            throttle: 0.0, // Start at 0 throttle
            _brake: 0.0,
        }
    }
}

/// Fly-By-Wire Flight Control Computer
/// Stabilizes the inherently unstable F-16 airframe using PID controllers
#[derive(Component)]
struct FlightControlComputer {
    // Target attitude (what player wants)
    _target_pitch: f32,  // Radians
    _target_roll: f32,   // Radians
    _target_yaw_rate: f32, // Reserved for future yaw control

    // PID state for pitch
    _pitch_error_integral: f32,
    _pitch_error_prev: f32,

    // PID state for roll
    _roll_error_integral: f32,
    _roll_error_prev: f32,

    // PID gains (tuned for F-16)
    _pitch_kp: f32,  // Proportional gain
    _pitch_ki: f32,  // Integral gain
    _pitch_kd: f32,  // Derivative gain

    _roll_kp: f32,
    _roll_ki: f32,
    _roll_kd: f32,

    // Control modes
    enabled: bool,      // Enable/disable FBW
    sas_enabled: bool,  // Stability Augmentation System (always recommended)
}

impl Default for FlightControlComputer {
    fn default() -> Self {
        Self {
            _target_pitch: 0.0,
            _target_roll: 0.0,
            _target_yaw_rate: 0.0,

            _pitch_error_integral: 0.0,
            _pitch_error_prev: 0.0,

            _roll_error_integral: 0.0,
            _roll_error_prev: 0.0,

            // VERY gentle PID gains (reduced 10x for stability)
            _pitch_kp: 0.15,  // Was 2.0
            _pitch_ki: 0.01,  // Was 0.1
            _pitch_kd: 0.08,  // Was 0.5

            _roll_kp: 0.12,   // Was 1.5
            _roll_ki: 0.005,  // Was 0.05
            _roll_kd: 0.06,   // Was 0.3

            enabled: true, // FBW ON by default - F-16 requires computer stabilization
            sas_enabled: true, // SAS ON by default - press K to disable (not recommended!)
        }
    }
}

#[derive(Component)]
struct FlightCamera {
    _local_offset: Vec3,
    _rotation_lag_speed: f32,
}

impl Default for FlightCamera {
    fn default() -> Self {
        Self {
            _local_offset: Vec3::new(0.0, 5.0, 15.0),
            _rotation_lag_speed: 5.0, // Stiffer camera for high speed
        }
    }
}

// ============================================================================
// AERODYNAMICS ENGINE (JSBSim Port)
// ============================================================================

/// Represents a 2D lookup table (x -> y)
#[derive(Clone)]
struct AeroCurve {
    _points: Vec<(f32, f32)>,
}

impl AeroCurve {
    fn new(points: Vec<(f32, f32)>) -> Self {
        Self { _points: points }
    }

    fn sample(&self, x: f32) -> f32 {
        if self._points.is_empty() { return 0.0; }
        if x <= self._points[0].0 { return self._points[0].1; }
        if x >= self._points.last().unwrap().0 { return self._points.last().unwrap().1; }

        for i in 0..self._points.len() - 1 {
            let (x0, y0) = self._points[i];
            let (x1, y1) = self._points[i+1];
            if x >= x0 && x <= x1 {
                let t = (x - x0) / (x1 - x0);
                return y0 + (y1 - y0) * t;
            }
        }
        0.0
    }
}

/// Stores the F-16 aerodynamic data (Curves extracted from f16.xml)
#[derive(Resource)]
struct F16AeroData {
    // Coefficients
    cl_alpha: AeroCurve, // Lift vs Alpha
    cd_alpha: AeroCurve, // Drag vs Alpha
    cy_beta: f32,        // Side force vs Beta (Scalar from XML: -1.146)
    
    // Stability Derivatives (Moments)
    cm_alpha: AeroCurve, // Pitch moment vs Alpha (Stability)
    cl_beta: AeroCurve,  // Roll moment vs Beta (Dihedral effect)
    cn_beta: AeroCurve,  // Yaw moment vs Beta (Weathercock stability)
    
    // Control Authorities
    cl_aileron: f32,     // Roll power (~0.05)
    cm_elevator: f32,    // Pitch power (~-0.8)
    cn_rudder: f32,      // Yaw power (~-0.05)
    
    // Damping
    cl_p: f32, // Roll damping
    cm_q: f32, // Pitch damping
    cn_r: f32, // Yaw damping

    // Physical Properties
    wing_area: f32,  // 300 sq ft -> 27.87 m^2
    wing_chord: f32, // 11.32 ft -> 3.45 m
    wing_span: f32,  // 30 ft -> 9.14 m
}

impl Default for F16AeroData {
    fn default() -> Self {
        Self {
            // Lift Coefficient (Alpha in Rads)
            // Stalls around 35 degrees (0.61 rad)
            cl_alpha: AeroCurve::new(vec![
                (-0.17, -0.65), (0.00, 0.18), (0.17, 0.80), (0.35, 1.39), (0.61, 1.90), (0.80, 1.50)
            ]),
            
            // Drag Coefficient (Increased for playability - prevents Mach 8 acceleration)
            // Original values too low, plane accelerates infinitely
            cd_alpha: AeroCurve::new(vec![
                (0.00, 0.15), (0.17, 0.30), (0.35, 0.50), (0.61, 1.20), (1.57, 2.50)
            ]),

            cy_beta: -1.14,

            // Pitch Moment (Cm) - More negative slope = More stable
            // Increased stability to prevent pitch oscillations
            cm_alpha: AeroCurve::new(vec![
                (-0.17, 0.10), (0.00, 0.00), (0.17, -0.15), (0.35, -0.30)
            ]),

            // Roll Moment due to Beta (Dihedral)
            // REDUCED by 10x to prevent uncontrollable roll divergence
            cl_beta: AeroCurve::new(vec![
                (-0.5, 0.005), (0.0, 0.0), (0.5, -0.005)
            ]),

            // Yaw Moment due to Beta (Weathercock)
            // REDUCED by 5x to prevent uncontrollable yaw divergence
            cn_beta: AeroCurve::new(vec![
                (-0.5, -0.02), (0.0, 0.0), (0.5, 0.02)
            ]),

            // Control Powers (Further reduced for stable manual flight)
            // Gentler inputs prevent overcontrol with increased damping
            cl_aileron: 0.03,  // Roll power (reduced for gentle control)
            cm_elevator: -0.25, // Pitch power (reduced for gentle control)
            cn_rudder: -0.02,  // Yaw power (reduced for gentle control)

            // PHASE 4: Damping Factors - MASSIVELY INCREASED for FBW stability
            // Even with FBW, need strong damping to prevent runaway divergence
            cl_p: -5.0,    // Roll damping (10x stronger - critical for preventing 100k deg/s spin)
            cm_q: -10.0,   // Pitch damping (10x stronger - prevents pitch oscillation)
            cn_r: -4.0,    // Yaw damping (8x stronger - prevents yaw departure)

            // Geometry (Converted to Metric)
            wing_area: 27.87,
            wing_chord: 3.45,
            wing_span: 9.14,
        }
    }
}

// ============================================================================
// PHASE 3 COMPONENTS - Combat System
// ============================================================================

#[derive(Component)]
struct Projectile {
    lifetime: f32,
}

#[derive(Component)]
struct LastShotTime {
    time: f32,
}

impl Default for LastShotTime {
    fn default() -> Self {
        Self { time: -10.0 }
    }
}

#[derive(Component)]
struct MuzzleFlash {
    lifetime: f32,
}

#[derive(Component)]
struct VisualDebris {
    velocity: Vec3,
    lifetime: f32,
}

#[derive(Component)]
struct MachineGunState {
    last_fired: f32,
    fire_side: bool, // Toggles Left/Right
    shot_count: u32,
}

impl Default for MachineGunState {
    fn default() -> Self {
        Self {
            last_fired: -10.0,
            fire_side: false,
            shot_count: 0,
        }
    }
}

#[derive(Component)]
struct Bullet {
    lifetime: f32,
    previous_translation: Vec3,
    is_tracer: bool,
}

// ============================================================================
// CONSTANTS
// ============================================================================

// F-16 Engine: F100-PW-229
// Reduced for playability - realistic thrust causes Mach 8 acceleration
// Target: Cruise at 200-300 m/s with manageable acceleration
const MAX_THRUST_NEWTONS: f32 = 35_000.0; // Reduced from 130,000 N
const MASS_KG: f32 = 9000.0; // Approx loaded weight

const BULLET_SPEED: f32 = 600.0; // Faster bullets for scale
const BULLET_LIFETIME: f32 = 3.0;
const FIRE_RATE: f32 = 10.125; // Reduced by 25% (was 13.5)
const FIRE_COOLDOWN: f32 = 1.0 / FIRE_RATE;

// Machine Gun Constants
const MG_FIRE_RATE: f32 = 0.08; // ~12 shots/sec
const MG_SPEED: f32 = 1200.0;   // Very fast
const MG_DAMAGE: f32 = 25.0;     // Buffed: 2 hits to kill a 50HP drone
const MG_OFFSET_LEFT: Vec3 = Vec3::new(-6.0, -2.5, -5.0);  // Further left (X: -4.5 → -6.0)
const MG_OFFSET_RIGHT: Vec3 = Vec3::new(-1.5, -2.5, -5.0); // Right position stays the same

// Offset for missile spawning (relative to player model)
// Adjusted to align with the F-16 nose cone (Shifted further LEFT to -3.0)
const GUN_OFFSET: Vec3 = Vec3::new(-3.0, -1.0, -5.0);
const _BULLET_RADIUS: f32 = 0.1; // Reserved for future use
const MUZZLE_FLASH_DURATION: f32 = 0.05;
const MUZZLE_FLASH_INTENSITY: f32 = 500.0;

// Missile visual dimensions
const MISSILE_LENGTH: f32 = 2.0;
const MISSILE_BODY_RADIUS: f32 = 0.15;
const MISSILE_FIN_SIZE: f32 = 0.3;

fn finish_spawning(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

// ============================================================================
// MAIN FUNCTION
// ============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ViperEye".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Spawning)
                .load_collection::<GameAssets>()
        )
        .insert_resource(ClearColor(Color::srgb(0.5, 0.6, 0.8))) // Skybox match
        .insert_resource(DirectionalLightShadowMap { size: 4096 }) // High-res shadows from Bevy example
        .init_resource::<F16AeroData>() // Load Aero Data
        // .init_resource::<SoundAssets>() // REMOVED: Now handled by GameAssets
        .init_resource::<ChunkManager>() // NEW: Chunk Manager
        .add_plugins(DronePlugin)
        .add_plugins(ui::UiPlugin) // NEW: HUD
        .add_systems(OnEnter(GameState::Spawning), (
            set_window_icon,
            configure_grass_texture_sampler,
            setup_scene,
            spawn_realistic_clouds,
            spawn_objectives,
            spawn_turrets,
            spawn_player,
            finish_spawning, // Transition to Playing immediately
        ).chain())
        .add_systems(PreUpdate, (
            safety_check_nan, // NEW: Global NaN protection
            check_ground_collision.run_if(in_state(GameState::Playing)), // Before physics so AABB never sees invalid state
        ))
        .add_systems(Update, (
            handle_pause_input, // NEW: Pause input runs always
            update_manual_audio_attenuation, // NEW: Manual audio attenuation runs always
        ))
        // Bevy has a 20-system tuple limit per add_systems; split to avoid overflow when adding more
        .add_systems(Update, (
            read_player_input,
            arcade_flight_physics, // ARCADE PHYSICS: Direct control, no FBW interference
            update_turrets, // NEW: Turret AI
            update_engine_audio, // NEW: Dynamic engine sound
            manage_chunks, // NEW: Infinite world chunk system
            update_altitude_visuals, // NEW: Sky->Space transition
            propagate_no_frustum_culling, // FIX: Ensure children don't get culled if parent is NoFrustumCulling
            // debug_flight_diagnostics, // REMOVED: Too noisy
            spawn_afterburner_particles, // Particle spawning based on throttle
            update_particles, // Update particle positions and fade
            update_cloud_billboards, // NEW: Make clouds face camera
            update_sky_sphere, // NEW: Keep sky sphere centered on camera
            update_horizon_disk, // NEW: Keep horizon disk centered on camera (XZ)
            update_flight_camera,
            debug_flight_data,
            update_maneuver_audio,   // NEW: Wind rip sound
            update_afterburner_audio, // NEW: Afterburner thump
            // debug_flight_dynamics, // REMOVED: Too noisy
        ).run_if(in_state(GameState::Playing)))
        .add_systems(PostUpdate, update_lod_levels.run_if(in_state(GameState::Playing))) // MOVED to PostUpdate so trees are spawned before LOD processes them
        // CRITICAL: Run NaN safety check BEFORE physics (FixedFirst runs before FixedUpdate physics)
        .add_systems(FixedFirst, detect_nan_early.run_if(in_state(GameState::Playing)))
        .add_systems(Update, (
            handle_quit,
            handle_restart, // R button to restart game
            debug_asset_loading, // Debug model loading
            // debug_tree_hierarchy, // REMOVED: False alarm with Direct Mesh Loading
            handle_shooting_input,
            handle_machine_gun_input,
            update_projectiles,
            update_bullets,
            update_visual_debris,
            handle_projectile_collisions,
            drone_projectile_collision,
            bullet_drone_collision,
            drone_player_collision, // NEW
            update_muzzle_flashes,
            update_explosion_effects, // Clean up explosion effects
        ).run_if(in_state(GameState::Playing)))
        .run();
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// Configure grass texture sampler for seamless tiling
///
/// The PNG texture uses Bevy's default ImageSampler (Clamp mode), but the ground
/// mesh UVs are multiplied by 10.0 for tiling. Clamp + tiling = stretched edges
/// at chunk boundaries. This system fixes it by setting sampler to Repeat mode.
fn configure_grass_texture_sampler(
    game_assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Some(image) = images.get_mut(&game_assets.grass_texture) {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            ..default()
        });
        eprintln!("✅ Grass texture sampler configured (Repeat mode)");
    } else {
        eprintln!("⚠️  Could not configure grass texture sampler");
    }

    // Configure normal map sampler (must also use Repeat for tiling)
    if let Some(image) = images.get_mut(&game_assets.grass_normal) {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            ..default()
        });
        eprintln!("✅ Grass normal map sampler configured (Repeat mode)");
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut _images: ResMut<Assets<Image>>, // Kept but unused for now
    game_assets: Res<GameAssets>,
) {
    // Load HDR Environment Map (now used for Sky Sphere)
    let skybox_handle = asset_server.load("textures/citrus_orchard_road_puresky_4k.hdr");

    commands.spawn((
        DirectionalLight {
            illuminance: 30000.0,
            shadows_enabled: true,
            ..default()
        },
        // Tuned shadow config for Flight Sim scale (large distances)
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.1,
            maximum_distance: 5000.0, // Increased to 5km to keep ground bright
            first_cascade_far_bound: 50.0, 
            overlap_proportion: 0.2,
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        GlobalTransform::default(),
    ));

    // Restore AmbientLight (so scene isn't black)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 80.0, // HDR-safe: 200.0 caused black screen with bloom (over-exposure)
    });

    // === CREATE SHARED GROUND MATERIAL WITH ASSET LOADER TEXTURE ===
    eprintln!("🌿 STARTUP: Using pre-loaded grass texture from GameAssets...");

    // Use loaded texture instead of generating
    let grass_texture_handle = game_assets.grass_texture.clone();
    
    // Create material with texture immediately
    let ground_material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE, // White so texture shows true colors
        base_color_texture: Some(grass_texture_handle),
        normal_map_texture: Some(game_assets.grass_normal.clone()), // Normal map for surface detail
        perceptual_roughness: 0.9,
        reflectance: 0.02,
        metallic: 0.0,
        unlit: true, // FIX: Unlit ensures grass shows full brightness regardless of shadow cascades
        ..default()
    });

    commands.insert_resource(GroundMaterial(ground_material_handle));
    eprintln!("🌿 STARTUP: Ground material resource created");

    // Spawn Sky Sphere
    commands.spawn((
        SkySphere,
        Mesh3d(meshes.add(Mesh::from(Sphere::new(20000.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(skybox_handle.clone()),
            base_color: Color::srgba(0.5, 0.6, 0.8, 1.0), // Tint to match fog
            unlit: true, // Sky should glow, not be lit by sun
            fog_enabled: false, // Don't let fog hide the sky
            cull_mode: None, // Render both sides (so we see it from inside)
            ..default()
        })),
        Transform::from_scale(Vec3::new(1.0, 1.0, -1.0)), // Invert sphere to show texture inside
        GlobalTransform::default(),
    ));

    // GLOBAL GROUND REMOVED - Replaced by Chunk System

    // Add a debug grid plane high above to show altitude/pitch reference
    let debug_grid_size = 20000.0;
    let debug_grid_mesh = meshes.add(Mesh::from(Plane3d::default().mesh().size(debug_grid_size, debug_grid_size)));
    let debug_grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.4, 0.4, 1.0, 0.15),  // Light blue transparent
        ..default()
    });

    commands.spawn((
        Mesh3d(debug_grid_mesh),
        MeshMaterial3d(debug_grid_material),
        Transform::from_xyz(0.0, 3000.0, 0.0),  // High above, acts as "sky" reference
        GlobalTransform::default(),
    ));

    // Add reference markers at cardinal directions
    let marker_positions = vec![
        (Vec3::new(5000.0, 50.0, 0.0), "X+"),      // Red: +X direction
        (Vec3::new(-5000.0, 50.0, 0.0), "X-"),     // Red: -X direction
        (Vec3::new(0.0, 50.0, 5000.0), "Z+"),      // Blue: +Z direction
        (Vec3::new(0.0, 50.0, -5000.0), "Z-"),     // Blue: -Z direction
    ];

    for (pos, _label) in marker_positions {
        let marker_mesh = meshes.add(Mesh::from(Sphere::new(100.0)));
        let marker_material = if pos.x > 0.0 || pos.x < 0.0 {
            materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),  // Red for X axis
                emissive: Color::srgb(1.0, 0.0, 0.0).into(),
                ..default()
            })
        } else {
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.0, 1.0),  // Blue for Z axis
                emissive: Color::srgb(0.0, 0.0, 1.0).into(),
                ..default()
            })
        };

        commands.spawn((
            Mesh3d(marker_mesh),
            MeshMaterial3d(marker_material),
            Transform::from_translation(pos),
            GlobalTransform::default(),
        ));
    }

    // Bevy 0.15: use DistanceFog
    // FIX: Linear fog keeps ground crisp for 10km, gentle fade to 30km (flight sim standard)
    commands.spawn((
        Camera3d::default(),
        Camera { hdr: true, ..default() }, // REQUIRED: Bloom needs HDR or all values are clamped to 1.0
        Msaa::Off, // HDR + MSAA causes black screen/artifacts in Bevy 0.15 - must disable
        // ReinhardLuminance does NOT require tonemapping_luts feature (safe fallback)
        // TonyMcMapface requires tonemapping_luts cargo feature - using Reinhard to isolate black screen
        bevy::core_pipeline::tonemapping::Tonemapping::ReinhardLuminance,
        // Bloom::NATURAL preset - proven working in official Bevy 0.15 bloom_3d example
        bevy::core_pipeline::bloom::Bloom::NATURAL,
        SpatialListener::default(), // FIXED: Enable spatial audio by attaching listener to camera
        Projection::Perspective(PerspectiveProjection {
            far: 100000.0,  // Increased further to cover massive distances
            ..default()
        }),
        DistanceFog {
            color: Color::srgba(0.5, 0.6, 0.8, 1.0), // Match skybox/ClearColor
            falloff: FogFalloff::Linear {
                start: 3000.0, // Fog starts at 3km (inside 8km chunk radius)
                end: 12000.0,  // Fully opaque at 12km (hides chunk edge)
            },
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    // FIX G: TEMPORARY - Bloom test sphere to verify bloom is working
    // TODO: DELETE after verifying bloom works (should see bright green glow halo)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            emissive: LinearRgba::rgb(20.0, 50.0, 20.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 510.0, -50.0)), // Near player spawn
    ));

    // Drones now spawn via the chunk system (infinite patrols)
}

/// Helper function to spawn the initial combat challenge
fn spawn_initial_drone_swarm(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut rng = rand::thread_rng();
    println!("🛸 SWARM INITIATED: Spawning 20 drones...");
    for i in 0..20 {
        let x = rng.gen_range(-2000.0..2000.0);
        let y = rng.gen_range(400.0..800.0);
        let z = rng.gen_range(-5000.0..-2000.0);
        crate::drone::spawn_beaver_drone(commands, asset_server, meshes, materials, Vec3::new(x, y, z));
        if i % 5 == 0 {
            println!("   > Drone group {} spawned", i / 5 + 1);
        }
    }
}


fn manage_chunks(
    mut commands: Commands,
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut chunk_manager: ResMut<ChunkManager>,
    chunk_entities: Query<(Entity, &ChunkCoordinate), With<ChunkEntity>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ground_material: Res<GroundMaterial>,  // Shared ground material
) {
    let Ok(player_transform) = player_query.get_single() else {
        eprintln!("❌ manage_chunks: NO PLAYER FOUND");
        return;
    };
    let player_chunk = ChunkCoordinate::from_world_pos(player_transform.translation);

    // Only update if player moved to new chunk or if this is the first run
    if player_chunk == chunk_manager.last_player_chunk && !chunk_manager.loaded_chunks.is_empty() {
        return;
    }
    chunk_manager.last_player_chunk = player_chunk;

    // DEBUG: Show chunk loading progress
    println!("📦 CHUNKS: Player at world({:.0},{:.0},{:.0}) = chunk({},{}), Loaded: {} chunks",
        player_transform.translation.x, player_transform.translation.y, player_transform.translation.z,
        player_chunk.x, player_chunk.z, chunk_manager.loaded_chunks.len());

    // 1. Unload distant chunks
    let mut to_unload = Vec::new();
    for (entity, chunk_coord) in &chunk_entities {
        let dx = player_chunk.x - chunk_coord.x;
        let dz = player_chunk.z - chunk_coord.z;
        if dx * dx + dz * dz > UNLOAD_RADIUS_CHUNKS * UNLOAD_RADIUS_CHUNKS {
            to_unload.push(*chunk_coord);
            commands.entity(entity).despawn_recursive();
        }
    }

    for coord in to_unload {
        chunk_manager.loaded_chunks.remove(&coord);
    }

    // 2. Load nearby chunks
    for x_offset in -LOAD_RADIUS_CHUNKS..=LOAD_RADIUS_CHUNKS {
        for z_offset in -LOAD_RADIUS_CHUNKS..=LOAD_RADIUS_CHUNKS {
            let chunk_coord = ChunkCoordinate {
                x: player_chunk.x + x_offset,
                z: player_chunk.z + z_offset,
            };

            if chunk_manager.loaded_chunks.contains_key(&chunk_coord) {
                continue;
            }
            
            let dx = x_offset;
            let dz = z_offset;
            if dx * dx + dz * dz > LOAD_RADIUS_CHUNKS * LOAD_RADIUS_CHUNKS {
                continue;
            }

            let chunk_entity = spawn_chunk(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                chunk_coord,
                ground_material.0.clone(),  // Pass shared material
            );
            chunk_manager.loaded_chunks.insert(chunk_coord, chunk_entity);
        }
    }
}

fn update_altitude_visuals(
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut fog_query: Query<&mut DistanceFog, With<Camera3d>>,
    mut clear_color: ResMut<ClearColor>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let altitude = player_transform.translation.y;

    // Transition zone: 10km - 25km (Atmospheric exit)
    // GUARD: Don't darken fog until player is actually high (10km+)
    const TRANSITION_START: f32 = 10000.0;
    const TRANSITION_END: f32 = 25000.0;

    // Calculate transition factor (0.0 at 10km, 1.0 at 25km)
    let transition_factor = ((altitude - TRANSITION_START) / (TRANSITION_END - TRANSITION_START)).clamp(0.0, 1.0);

    // Smooth ease-in-out curve
    let t = transition_factor * transition_factor * (3.0 - 2.0 * transition_factor);

    // Base sky colors: Light blue/gray at low altitude, black in space
    let earth_sky = Color::srgb(0.5, 0.6, 0.8);
    let space_black = Color::srgb(0.0, 0.0, 0.0);

    // Interpolate colors based on altitude factor 't'
    let new_color = Color::srgb(
        earth_sky.to_linear().red * (1.0 - t),
        earth_sky.to_linear().green * (1.0 - t),
        earth_sky.to_linear().blue * (1.0 - t),
    );

    // Update background color
    clear_color.0 = new_color;

    // Update fog configuration
    if let Ok(mut fog) = fog_query.get_single_mut() {
        fog.color = new_color;

        // Keep Linear fog for consistent ground appearance
        // Only the color darkens with altitude, not the fog density
        fog.falloff = FogFalloff::Linear {
            start: 3000.0,
            end: 12000.0,
        };
    }
}

fn update_lod_levels(
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut tree_query: Query<(&GlobalTransform, &mut Visibility), With<Tree>>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation;

    // Very simple LOD: Hide trees beyond 20km to save on draw calls
    const HIDE_DISTANCE_SQ: f32 = 20000.0 * 20000.0;

    let mut visible_count = 0;
    let mut hidden_count = 0;

    for (tree_global_transform, mut visibility) in &mut tree_query {
        let dist_sq = player_pos.distance_squared(tree_global_transform.translation());
        if dist_sq > HIDE_DISTANCE_SQ {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            hidden_count += 1;
        } else {
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
            visible_count += 1;
        }
    }

    if visible_count > 0 || hidden_count > 0 {
        // Reduced logging frequency or removed for performance
        // eprintln!("📊 LOD: {} visible trees, {} hidden trees", visible_count, hidden_count);
    }
}

/// Create terrain mesh with heightmap applied
fn create_terrain_mesh(
    chunk_coord: ChunkCoordinate,
    meshes: &mut Assets<Mesh>,
) -> Handle<Mesh> {
    const SUBDIVISIONS: usize = 20; // 20x20 grid = 400 triangles per chunk
    // CHUNK_SIZE is already defined globally

    let chunk_world_x = chunk_coord.x as f32 * CHUNK_SIZE;
    let chunk_world_z = chunk_coord.z as f32 * CHUNK_SIZE;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices with heightmap
    for z in 0..=SUBDIVISIONS {
        for x in 0..=SUBDIVISIONS {
            let local_x = (x as f32 / SUBDIVISIONS as f32) * CHUNK_SIZE - CHUNK_SIZE / 2.0;
            let local_z = (z as f32 / SUBDIVISIONS as f32) * CHUNK_SIZE - CHUNK_SIZE / 2.0;

            let world_x = chunk_world_x + local_x;
            let world_z = chunk_world_z + local_z;

            let height = get_terrain_height(world_x, world_z);

            positions.push([local_x, height, local_z]);
            normals.push([0.0, 1.0, 0.0]); // Simple normal (could calculate from neighbors for lighting)
            
            // UV tiling: Multiply by 10.0 to match the previous Plane3d tiling
            let u = (x as f32 / SUBDIVISIONS as f32) * 10.0;
            let v = (z as f32 / SUBDIVISIONS as f32) * 10.0;
            uvs.push([u, v]);
        }
    }

    // Generate indices (two triangles per quad)
    for z in 0..SUBDIVISIONS {
        for x in 0..SUBDIVISIONS {
            let i0 = (z * (SUBDIVISIONS + 1) + x) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + (SUBDIVISIONS + 1) as u32;
            let i3 = i2 + 1;

            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    meshes.add(mesh)
}

fn spawn_chunk(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    chunk_coord: ChunkCoordinate,
    ground_material: Handle<StandardMaterial>,  // Use shared material from startup
) -> Entity {
    let chunk_pos = chunk_coord.world_position();

    // SAFETY: Ensure chunk position is valid
    if chunk_pos.is_nan() || !chunk_pos.is_finite() {
        eprintln!("❌ CHUNK SPAWN ERROR: Invalid chunk_pos for {:?}", chunk_coord);
        // Return a dummy entity to avoid breaking the caller, but this should be extremely rare
        return commands.spawn_empty().id();
    }

    println!("🌍 CHUNK SPAWN: coord=({},{}), world_pos=({:.0},{:.0},{:.0})",
        chunk_coord.x, chunk_coord.z, chunk_pos.x, chunk_pos.y, chunk_pos.z);

    let chunk_entity = commands.spawn((
        ChunkEntity,
        chunk_coord,
        Transform::from_translation(chunk_pos),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    )).id();

    // Ground material is now passed in (shared across all chunks)

    commands.entity(chunk_entity).with_children(|parent| {
        let half_size = CHUNK_SIZE / 2.0;
        let thickness = 0.5;

        // Final sanity check for collider dimensions
        if half_size > 0.0 && half_size.is_finite() && thickness > 0.0 {
            // Create ground mesh with UV tiling for texture detail
            // REPLACED: Use heightmap mesh instead of flat plane
            let mesh_handle = create_terrain_mesh(chunk_coord, meshes);

            parent.spawn((
                ChunkEntity,
                chunk_coord,
                Mesh3d(mesh_handle),
                MeshMaterial3d(ground_material),
                Transform::from_xyz(0.0, 0.0, 0.0), // No Y offset - terrain heights are baked into vertices
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                RigidBody::Static,
                Collider::cuboid(half_size, thickness, half_size),
            ));
        } else {
            eprintln!("❌ CHUNK SPAWN ERROR: Invalid collider dimensions for chunk {:?}", chunk_coord);
        }
    });

    spawn_trees_in_chunk(commands, asset_server, meshes, materials, chunk_coord, chunk_pos, chunk_entity);
    spawn_rocks_in_chunk(commands, meshes, materials, chunk_coord, chunk_pos, chunk_entity); // Added rocks
    spawn_meteors_in_chunk(commands, asset_server, chunk_coord, chunk_entity); // Added infinite sky litter
    
    // NEW: Occasionally spawn a drone "Patrol" in new chunks
    // 15% chance per chunk to spawn a drone
    let hash = ((chunk_coord.x.wrapping_mul(1234567)) ^ (chunk_coord.z.wrapping_mul(7654321))) as u32;
    if (hash % 100) < 15 {
        let spawn_pos = chunk_pos + Vec3::new(0.0, 500.0, 0.0);
        crate::drone::spawn_beaver_drone(commands, asset_server, meshes, materials, spawn_pos);
        println!("🛸 CHUNK PATROL: Drone spawned in chunk {:?}", chunk_coord);
    }

    if should_spawn_village(chunk_coord) {
        spawn_village_in_chunk(commands, asset_server, meshes, materials, chunk_coord, chunk_pos, chunk_entity);
    }

    println!("🌍 Chunk ({},{}) spawned with trees & village check", chunk_coord.x, chunk_coord.z);
    chunk_entity
}

fn propagate_no_frustum_culling(
    mut commands: Commands,
    trees: Query<Entity, (With<Tree>, With<bevy::render::view::NoFrustumCulling>)>,
    children_query: Query<&Children>,
    no_culling_query: Query<Entity, With<bevy::render::view::NoFrustumCulling>>,
) {
    for tree_entity in &trees {
        for child in children_query.iter_descendants(tree_entity) {
            if !no_culling_query.contains(child) {
                commands.entity(child).insert(bevy::render::view::NoFrustumCulling);
            }
        }
    }
}

fn spawn_trees_in_chunk(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _meshes: &mut Assets<Mesh>, // Unused _ removed
    materials: &mut Assets<StandardMaterial>, // Unused _ removed
    chunk_coord: ChunkCoordinate,
    _chunk_pos: Vec3,
    chunk_entity: Entity,
) {
    eprintln!("🌲 SPAWN_TREES_IN_CHUNK CALLED for chunk ({},{})", chunk_coord.x, chunk_coord.z);
    use rand::SeedableRng;
    let seed = ((chunk_coord.x as i64 * 73856093) ^ (chunk_coord.z as i64 * 19349663)) as u64;
    let mut chunk_rng = rand::rngs::StdRng::seed_from_u64(seed);

    let tree_count = chunk_rng.gen_range(TREES_PER_CHUNK_MIN..=TREES_PER_CHUNK_MAX);
    println!("🌲 Spawning {} trees in chunk ({},{})", tree_count, chunk_coord.x, chunk_coord.z);

    // Add #Mesh0/Primitive0 to target the mesh data directly inside the GLB
    let tree_models = vec![
        "fantasy_town/tree.glb",
        "fantasy_town/tree-crooked.glb",
        "fantasy_town/tree-high.glb",
        "fantasy_town/tree-high-crooked.glb",
        "fantasy_town/tree-high-round.glb",
    ];

    // Create a shared green material for all trees
    let tree_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.5, 0.1), // Grass green
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..default()
    });

    commands.entity(chunk_entity).with_children(|parent| {
        for _ in 0..tree_count {
            let x = chunk_rng.gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0);
            let z = chunk_rng.gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0);

            if should_spawn_village(chunk_coord) && (x*x + z*z < 400.0*400.0) {
                 continue;
            }

            let model_index = chunk_rng.gen_range(0..tree_models.len());
            let tree_model_path = format!("{}#Mesh0/Primitive0", tree_models[model_index]);
            let scale = chunk_rng.gen_range(3.0..6.0);

            // Use LOCAL coordinates because trees are now children of the chunk
            // Get terrain height for Y position
            let world_x = chunk_coord.x as f32 * CHUNK_SIZE + x;
            let world_z = chunk_coord.z as f32 * CHUNK_SIZE + z;
            let terrain_height = get_terrain_height(world_x, world_z);
            
            // Offset by +5.0 because chunk parent is at Y=-5.0
            let tree_local_pos = Vec3::new(x, terrain_height + 5.0, z);

            parent.spawn((
                Tree,
                ChunkEntity,
                chunk_coord,
                LODLevel(0),
                // DIRECT MESH LOADING (Option 1)
                Mesh3d(asset_server.load(tree_model_path)),
                MeshMaterial3d(tree_material.clone()), // Apply green material
                Transform {
                    translation: tree_local_pos,
                    rotation: Quat::from_rotation_y(chunk_rng.gen_range(0.0..std::f32::consts::TAU)),
                    scale: Vec3::splat(scale),
                },
                Visibility::default(),
                bevy::render::view::NoFrustumCulling,
            ));
        }
    });
}

fn spawn_rocks_in_chunk(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    chunk_coord: ChunkCoordinate,
    _chunk_pos: Vec3,
    chunk_entity: Entity,
) {
    use rand::SeedableRng;
    // Different seed than trees so rocks are in different spots
    let seed = ((chunk_coord.x as i64 * 19349663) ^ (chunk_coord.z as i64 * 73856093)) as u64;
    let mut chunk_rng = rand::rngs::StdRng::seed_from_u64(seed);

    let rock_count = chunk_rng.gen_range(2..=4);
    
    let rock_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.5, 0.4), // Gray-brown rock
        perceptual_roughness: 0.9,
        ..default()
    });

    let rock_mesh = meshes.add(Cuboid::new(20.0, 15.0, 20.0)); // 20m rocks

    commands.entity(chunk_entity).with_children(|parent| {
        for _ in 0..rock_count {
            let x = chunk_rng.gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0);
            let z = chunk_rng.gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0);

            // Avoid village center if necessary, but rocks are tough so maybe it's fine
            if should_spawn_village(chunk_coord) && (x*x + z*z < 400.0*400.0) {
                 continue;
            }

            let scale = chunk_rng.gen_range(0.8..1.5);
            let rotation = Quat::from_rotation_y(chunk_rng.gen_range(0.0..std::f32::consts::TAU));

            // Get terrain height
            let world_x = chunk_coord.x as f32 * CHUNK_SIZE + x;
            let world_z = chunk_coord.z as f32 * CHUNK_SIZE + z;
            let terrain_height = get_terrain_height(world_x, world_z);

            parent.spawn((
                ChunkEntity,
                chunk_coord,
                Mesh3d(rock_mesh.clone()),
                MeshMaterial3d(rock_material.clone()),
                Transform {
                    translation: Vec3::new(x, terrain_height + 5.0 + (7.5 * scale), z), // +5 for chunk offset, +half_height for pivot
                    rotation,
                    scale: Vec3::splat(scale),
                },
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                RigidBody::Static,
                Collider::cuboid(10.0, 7.5, 10.0), // Half-extents
            ));
        }
    });
}

/// Global safety system to prevent NaN values from crashing the physics engine
/// AGGRESSIVE: Catch NaN before it reaches physics engine and FIX them
fn detect_nan_early(
    mut player_query: Query<(
        Entity,
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
    ), With<PlayerPlane>>,
    mut projectiles: Query<(Entity, &mut Transform, &mut LinearVelocity), (With<Projectile>, Without<PlayerPlane>)>,
    mut drones: Query<(Entity, &mut Transform), (With<drone::Drone>, Without<PlayerPlane>, Without<Projectile>)>,
    mut commands: Commands,
) {
    // Check and FIX PLAYER
    if let Ok((_entity, mut transform, mut lin_vel, mut ang_vel)) = player_query.get_single_mut() {
        let mut needs_reset = false;

        // Transform check
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("🚨 EARLY: Player Transform.translation has NaN! {:?}", transform.translation);
            needs_reset = true;
        }
        if transform.scale.is_nan() || !transform.scale.is_finite() || transform.scale.x <= 0.0 || transform.scale.y <= 0.0 || transform.scale.z <= 0.0 {
            eprintln!("🚨 EARLY: Player Transform.scale invalid! {:?}", transform.scale);
            transform.scale = Vec3::ONE; // FIX: Reset scale to 1
        }
        if transform.rotation.is_nan() || !transform.rotation.is_normalized() {
            eprintln!("🚨 EARLY: Player Rotation has NaN or unnormalized!");
            transform.rotation = Quat::IDENTITY; // FIX: Reset rotation
        }

        // LinearVelocity check
        if lin_vel.0.is_nan() || !lin_vel.0.is_finite() {
            eprintln!("🚨 EARLY: Player LinearVelocity has NaN! {:?}", lin_vel.0);
            needs_reset = true;
        }

        // AngularVelocity check
        if ang_vel.0.is_nan() || !ang_vel.0.is_finite() {
            eprintln!("🚨 EARLY: Player AngularVelocity has NaN! {:?}", ang_vel.0);
            needs_reset = true;
        }

        // Full reset if needed
        if needs_reset {
            eprintln!("🚨 EARLY: RESETTING PLAYER TO SAFE STATE");
            transform.translation = Vec3::new(0.0, 500.0, 0.0);
            transform.rotation = Quat::IDENTITY;
            transform.scale = Vec3::ONE;
            *lin_vel = LinearVelocity(Vec3::new(0.0, 0.0, -100.0));
            *ang_vel = AngularVelocity::ZERO;
        }
    }

    // Check PROJECTILES - despawn if invalid
    for (entity, transform, velocity) in &projectiles {
        if transform.translation.is_nan() || !transform.translation.is_finite()
            || velocity.0.is_nan() || !velocity.0.is_finite()
            || transform.scale.is_nan() || !transform.scale.is_finite() {
            eprintln!("🚨 EARLY: Projectile {:?} has NaN! Despawning.", entity);
            commands.entity(entity).despawn_recursive();
        }
    }

    // Check DRONES - reset if invalid
    for (entity, mut transform) in &mut drones {
        if transform.translation.is_nan() || !transform.translation.is_finite() {
            eprintln!("🚨 EARLY: Drone {:?} position has NaN! Resetting.", entity);
            transform.translation = Vec3::new(0.0, 500.0, -200.0);
        }
        if transform.scale.is_nan() || !transform.scale.is_finite() || transform.scale.x <= 0.0 {
            eprintln!("🚨 EARLY: Drone {:?} scale invalid! Fixing.", entity);
            transform.scale = Vec3::splat(1.8);
        }
        if transform.rotation.is_nan() || !transform.rotation.is_normalized() {
            eprintln!("🚨 EARLY: Drone {:?} rotation invalid! Fixing.", entity);
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        }
    }
}

fn safety_check_nan(
    mut query: Query<(
        Entity, 
        &mut Transform, 
        Option<&mut LinearVelocity>, 
        Option<&mut AngularVelocity>,
        Option<&mut Position>,
        Option<&mut Rotation>,
    )>,
) {
    for (entity, mut transform, mut opt_lin_vel, mut opt_ang_vel, mut opt_pos, mut opt_rot) in &mut query {
        let mut needs_reset = false;
        
        if transform.translation.is_nan() || transform.rotation.is_nan() || transform.scale.is_nan() {
            needs_reset = true;
            eprintln!("⚠️ SAFETY: Detected NaN in Transform for entity {:?}.", entity);
        }

        if let Some(ref pos) = opt_pos {
            if pos.0.is_nan() {
                needs_reset = true;
                eprintln!("⚠️ SAFETY: Detected NaN in Position for entity {:?}.", entity);
            }
        }

        if let Some(ref rot) = opt_rot {
            if rot.0.is_nan() {
                needs_reset = true;
                eprintln!("⚠️ SAFETY: Detected NaN in Rotation for entity {:?}.", entity);
            }
        }
        
        if let Some(ref lin_vel) = opt_lin_vel {
            if lin_vel.is_nan() {
                needs_reset = true;
                eprintln!("⚠️ SAFETY: Detected NaN in LinearVelocity for entity {:?}.", entity);
            }
        }

        if let Some(ref ang_vel) = opt_ang_vel {
            if ang_vel.is_nan() {
                needs_reset = true;
                eprintln!("⚠️ SAFETY: Detected NaN in AngularVelocity for entity {:?}.", entity);
            }
        }

        if needs_reset {
            transform.translation = Vec3::ZERO;
            transform.rotation = Quat::IDENTITY;
            transform.scale = Vec3::ONE;
            
            if let Some(ref mut pos) = opt_pos {
                pos.0 = Vec3::ZERO;
            }
            if let Some(ref mut rot) = opt_rot {
                rot.0 = Quat::IDENTITY;
            }
            if let Some(ref mut lin_vel) = opt_lin_vel {
                **lin_vel = LinearVelocity::ZERO;
            }
            if let Some(ref mut ang_vel) = opt_ang_vel {
                **ang_vel = AngularVelocity::ZERO;
            }
        }
    }
}

fn should_spawn_village(chunk_coord: ChunkCoordinate) -> bool {
    let hash = ((chunk_coord.x.wrapping_mul(73856093)) ^ (chunk_coord.z.wrapping_mul(19349663))) as u32;
    (hash % 100) < 15  // 15% spawn rate (increased from 5% for better visibility)
}

fn spawn_village_in_chunk(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    chunk_coord: ChunkCoordinate,
    chunk_pos: Vec3,
    _chunk_entity: Entity,
) {
    println!("🏘️  Spawning village in chunk ({},{})", chunk_coord.x, chunk_coord.z);
    let village_center = chunk_pos;
    const NUM_BUILDINGS: usize = 8;
    const BUILDING_DISTANCE: f32 = 150.0;

    // Load texture atlas for village (Kenney assets share one texture)
    let texture_handle = asset_server.load("fantasy_town/Textures/colormap.png");

    // Wall material (darker brown)
    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.8,
        reflectance: 0.2,
        unlit: false,
        ..default()
    });

    // Roof material (brighter red-brown for visibility)
    let roof_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.3, 0.2), // Bright red-brown
        base_color_texture: Some(texture_handle),
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        unlit: false,
        ..default()
    });

    for i in 0..NUM_BUILDINGS {
        let angle = (i as f32 / NUM_BUILDINGS as f32) * std::f32::consts::TAU;
        let building_x = village_center.x + angle.cos() * BUILDING_DISTANCE;
        let building_z = village_center.z + angle.sin() * BUILDING_DISTANCE;
        let rotation = Quat::from_rotation_y(angle + std::f32::consts::PI);

        let terrain_height = get_terrain_height(building_x, building_z);

        // Building Base (Wall)
        commands.spawn((
            VillageBuilding,
            ChunkEntity,
            chunk_coord,
            Mesh3d(asset_server.load("fantasy_town/wall.glb#Mesh0/Primitive0")),
            MeshMaterial3d(wall_material.clone()),
            Transform {
                translation: Vec3::new(building_x, terrain_height - 0.5, building_z),
                rotation,
                scale: Vec3::splat(6.0),
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            RigidBody::Static,
            Collider::cuboid(3.0, 5.0, 3.0),
        ));

        // Building Roof (with fallback cube for visibility)
        println!("  🏠 Spawning roof at ({}, {}, {})", building_x, terrain_height + 32.0, building_z);
        commands.spawn((
            VillageBuilding,
            ChunkEntity,
            chunk_coord,
            SceneRoot(asset_server.load("fantasy_town/roof-gable.glb#Scene0")),
            Transform {
                translation: Vec3::new(building_x, terrain_height + 32.0, building_z),
                rotation,
                scale: Vec3::splat(6.0),
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ))
        .with_children(|parent| {
            // Fallback visual cube (bright red for debugging)
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 2.0))),
                MeshMaterial3d(roof_material.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
    }

    // Central Tower
    let tower_height = get_terrain_height(village_center.x, village_center.z);
    commands.spawn((
        VillageBuilding,
        ChunkEntity,
        chunk_coord,
        Mesh3d(asset_server.load("fantasy_town/wall.glb#Mesh0/Primitive0")),
        MeshMaterial3d(wall_material.clone()),
        Transform {
            translation: Vec3::new(village_center.x, tower_height - 0.5, village_center.z),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(20.0),
        },
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        RigidBody::Static,
        Collider::cuboid(10.0, 20.0, 10.0),
    ));
}

/// Spawn realistic cloud layer using FX cloud alpha textures
fn spawn_realistic_clouds(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();

    // Load all 10 cloud textures
    let cloud_textures = vec![
        "textures/clouds/FX_CloudAlpha01.png",
        "textures/clouds/FX_CloudAlpha02.png",
        "textures/clouds/FX_CloudAlpha03.png",
        "textures/clouds/FX_CloudAlpha04.png",
        "textures/clouds/FX_CloudAlpha05.png",
        "textures/clouds/FX_CloudAlpha06.png",
        "textures/clouds/FX_CloudAlpha07.png",
        "textures/clouds/FX_CloudAlpha08.png",
        "textures/clouds/FX_CloudAlpha09.png",
        "textures/clouds/FX_CloudAlpha10.png",
    ];

    // Cloud layer configuration
    const CLOUD_COUNT: usize = 50;
    const CLOUD_ALTITUDE_MIN: f32 = 800.0;
    const CLOUD_ALTITUDE_MAX: f32 = 1500.0;
    const CLOUD_SPREAD: f32 = 40000.0;

    for _ in 0..CLOUD_COUNT {
        let x = rng.gen_range(-CLOUD_SPREAD..CLOUD_SPREAD);
        let y = rng.gen_range(CLOUD_ALTITUDE_MIN..CLOUD_ALTITUDE_MAX);
        let z = rng.gen_range(-CLOUD_SPREAD..CLOUD_SPREAD);

        let texture_index = rng.gen_range(0..cloud_textures.len());
        let texture_path = cloud_textures[texture_index];

        let base_size = 150.0;
        let size = base_size + rng.gen_range(-50.0..100.0);

        let cloud_material = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(texture_path)),
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.8),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });

        let cloud_mesh = meshes.add(Mesh::from(Rectangle::new(size, size * 0.7)));

        commands.spawn((
            Cloud,
            Mesh3d(cloud_mesh),
            MeshMaterial3d(cloud_material),
            Transform::from_xyz(x, y, z)
                .with_rotation(Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::TAU))),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
    }

    println!("☁️  Realistic cloud layer spawned - {} clouds at 800-1500m altitude", CLOUD_COUNT);
}

/// Update clouds to face camera (billboarding)
fn update_cloud_billboards(
    mut cloud_query: Query<&mut Transform, (With<Cloud>, Without<Camera3d>)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Cloud>)>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        for mut cloud_transform in &mut cloud_query {
            cloud_transform.look_at(camera_transform.translation, Vec3::Y);
        }
    }
}

/// Keep the sky sphere centered on the camera to create an infinite sky effect
fn update_sky_sphere(
    mut sky_sphere_query: Query<&mut Transform, With<SkySphere>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<SkySphere>)>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        for mut sky_transform in &mut sky_sphere_query {
            sky_transform.translation = camera_transform.translation;
        }
    }
}

/// Keep the horizon disk centered on the camera (XZ only)
fn update_horizon_disk(
    mut horizon_query: Query<&mut Transform, With<HorizonDisk>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<HorizonDisk>)>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        for mut transform in &mut horizon_query {
            transform.translation.x = camera_transform.translation.x;
            transform.translation.z = camera_transform.translation.z;
            // Y remains fixed at -5.0
        }
    }
}

/// NEW: Spawn random meteor obstacles in the sky per chunk
fn spawn_meteors_in_chunk(
    commands: &mut Commands,
    asset_server: &AssetServer,
    chunk_coord: ChunkCoordinate,
    chunk_entity: Entity,
) {
    use rand::SeedableRng;
    let seed = ((chunk_coord.x as i64 * 1234567) ^ (chunk_coord.z as i64 * 7654321)) as u64;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    
    // Paths to meteor assets
    let meteor_paths = [
        "models/obstacles/meteor.glb#Scene0",
        "models/obstacles/meteor_detailed.glb#Scene0",
        "models/obstacles/meteor_half.glb#Scene0",
    ];

    let meteor_models: Vec<Handle<Scene>> = meteor_paths
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    // High density: 40 meteors per 1km chunk
    const METEOR_COUNT: usize = 40;
    
    commands.entity(chunk_entity).with_children(|parent| {
        for _ in 0..METEOR_COUNT {
            // Local position within the 1km chunk
            let pos = Vec3::new(
                rng.gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0),
                rng.gen_range(200.0..4000.0), // Scattered from 200m to 4km altitude
                rng.gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0),
            );

            // Random rotation
            let rotation = Quat::from_euler(
                EulerRot::XYZ,
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
            );
            
            // Size: Mostly small (scale 2-8), rare large ones (up to 20)
            let scale_roll = rng.gen_range(0.0..1.0);
            let scale = if scale_roll > 0.9 {
                rng.gen_range(12.0..25.0) // Big ones
            } else {
                rng.gen_range(2.0..8.0) // Small litter
            };

            let model_handle = meteor_models[rng.gen_range(0..meteor_models.len())].clone();

            parent.spawn((
                Meteor,
                ChunkEntity,
                SceneRoot(model_handle),
                Transform {
                    translation: pos,
                    rotation,
                    scale: Vec3::splat(scale),
                },
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                RigidBody::Static,
                Collider::sphere(0.8), 
            ));
        }
    });
}

/// NEW: Spawn destructible mission objectives (Satellite Dishes)
fn spawn_objectives(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let dish_handle = asset_server.load("models/satelliteDish_large.glb#Scene0");
    
    // Position 3 objectives in a triangle around the world
    let positions = [
        Vec3::new(500.0, 0.0, -500.0),
        Vec3::new(-500.0, 0.0, -500.0),
        Vec3::new(0.0, 0.0, 800.0),
    ];

    for (i, pos) in positions.iter().enumerate() {
        commands.spawn((
            Objective,
            SceneRoot(dish_handle.clone()),
            Transform {
                translation: *pos,
                rotation: Quat::from_rotation_y(i as f32 * 2.0),
                scale: Vec3::splat(15.0), // Scale up to be a good target
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            RigidBody::Static,
            Collider::cuboid(5.0, 8.0, 5.0), // Match model approx
        ));
        
        println!("📡 Objective {} spawned at {:?}", i + 1, pos);
    }
}

/// NEW: Spawn enemy turrets
fn spawn_turrets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let turret_handle = asset_server.load("models/turret_double.glb#Scene0");
    
    // Position turrets near objectives
    let positions = [
        Vec3::new(450.0, 0.0, -450.0),
        Vec3::new(-450.0, 0.0, -450.0),
        Vec3::new(50.0, 0.0, 750.0),
    ];

    for pos in positions {
        commands.spawn((
            Turret {
                fire_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            },
            SceneRoot(turret_handle.clone()),
            Transform {
                translation: pos,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(15.0),
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            RigidBody::Static,
            Collider::cuboid(4.0, 6.0, 4.0),
        ));
    }
}

/// NEW: Update turret AI: rotate and fire at player
fn update_turrets(
    time: Res<Time>,
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut turret_query: Query<(&mut Transform, &mut Turret), Without<PlayerPlane>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut transform, mut turret) in &mut turret_query {
            // Face the player
            let target_pos = player_transform.translation;
            transform.look_at(target_pos, Vec3::Y);
            
            // Fire every 2 seconds
            turret.fire_timer.tick(time.delta());
            if turret.fire_timer.just_finished() {
                let muzzle_pos = transform.translation + transform.up().as_vec3() * 5.0;
                let direction = (target_pos - muzzle_pos).normalize();
                let velocity = direction * 300.0; // Slower than player bullets
                
                spawn_missile(&mut commands, &mut meshes, &mut materials, muzzle_pos, transform.rotation, velocity);
                spawn_muzzle_flash(&mut commands, &mut meshes, &mut materials, muzzle_pos, None);
            }
        }
    }
}

/// Dynamic engine and environmental audio system
fn update_engine_audio(
    game_state: Res<State<GameState>>,
    player_query: Query<(&PlayerInput, &LinearVelocity, &Transform), With<PlayerPlane>>,
    mut engine_audio_query: Query<&AudioSink, (With<EngineSound>, Without<WindSound>, Without<WarningSound>)>,
    mut wind_audio_query: Query<&AudioSink, (With<WindSound>, Without<EngineSound>, Without<WarningSound>)>,
    mut warning_audio_query: Query<&AudioSink, (With<WarningSound>, Without<EngineSound>, Without<WindSound>)>,
) {
    // Pause all audio when game is paused
    if game_state.get() == &GameState::Paused {
        for sink in &mut engine_audio_query {
            sink.pause();
        }
        for sink in &mut wind_audio_query {
            sink.pause();
        }
        return;
    }

    if let Ok((input, velocity, transform)) = player_query.get_single() {
        let speed = velocity.0.length();
        
        // Engine Sound: Scales with throttle
        for sink in &mut engine_audio_query {
            sink.set_volume(0.1 + input.throttle * 0.7);
            sink.set_speed(0.8 + input.throttle * 0.7);
        }

        // Wind/Airflow Sound: Scales with actual speed (max volume at 500 m/s)
        let wind_volume = (speed / 500.0).min(1.0) * 0.5;
        for sink in &mut wind_audio_query {
            sink.set_volume(wind_volume);
            sink.set_speed(0.9 + (speed / 500.0) * 0.3);
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sounds: Res<GameAssets>,
) {
    // Print controls on startup
    println!("\n╔══════════════════════════════════════════════╗");
    println!("║       F-16 FIGHTER JET - CONTROLS           ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║  W/S        - Pitch Up/Down                  ║");
    println!("║  A/D        - Roll Left/Right                ║");
    println!("║  Q/E        - Yaw Left/Right                 ║");
    println!("║  Shift      - Increase Throttle (Boost)      ║");
    println!("║  Ctrl       - Decrease Throttle              ║");
    println!("║  SPACE      - Fire Missiles                  ║");
    println!("║  R          - Restart Game                   ║");
    println!("║  ESC        - Quit                           ║");
    println!("╚══════════════════════════════════════════════╝\n");

    // Load the F-16 Template (High Quality)
    let model_handle = asset_server.load("models/f16_template/scene.gltf#Scene0");
    // Alternative: let model_handle = asset_server.load("models/low_poly_f16/scene.gltf#Scene0");
    // Alternative: let model_handle = asset_server.load("models/fighter_jet_enhanced.gltf#Scene0");

    let player = commands.spawn((
        PlayerPlane,
        Transform::from_xyz(0.0, 500.0, 0.0), // Start high up
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        RigidBody::Dynamic,
        Mass(MASS_KG), // REAL MASS
        LinearVelocity(Vec3::new(0.0, 0.0, -100.0)), // Start at 100 m/s - gentler start
        AngularVelocity::default(),
        ExternalForce::default(),
        ExternalTorque::default(),
        Collider::cuboid(2.0, 1.0, 4.0),
        PlayerInput::default(),
        FlightCamera::default(),
    ))
    .insert(FlightControlComputer::default())
    .insert(DiagnosticTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
    .insert(AfterburnerParticles::default())
    .insert(LastShotTime::default())
    .insert(MachineGunState::default())
    .insert(RocketMode::default())
    .id();

    commands.entity(player)
    .with_children(|parent| {
        parent.spawn((
            ModelContainer,
            // Scale down model to fit game (typical GLTF exports are oversized)
            // Rotate 180 degrees (PI radians) around Y to face forward (-Z)
            Transform::from_scale(Vec3::splat(0.08))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            SceneRoot(model_handle),
        ));

        // Looping engine sound
        parent.spawn((
            EngineSound,
            AudioPlayer(sounds.engine_loop.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.2), // Start quiet
                ..default()
            },
        ));

        // Wind/Airflow loop
        parent.spawn((
            WindSound,
            AudioPlayer(sounds.wind.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.0), // Starts silent
                ..default()
            },
        ));

        // Warning alarm loop
        parent.spawn((
            WarningSound,
            AudioPlayer(sounds.warning.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.0), // Starts silent
                paused: true, // Only plays in danger
                ..default()
            },
        ));

        // Air Rip / Wing Stress loop (Cinematic)
        parent.spawn((
            WindStressSound,
            AudioPlayer(sounds.air_rip.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::new(0.0), // Silent until high-G turn
                ..default()
            },
        ));
    });
}

fn read_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut PlayerInput, &AngularVelocity, &Transform, &mut FlightControlComputer, &mut RocketMode), With<PlayerPlane>>,
) {
    for (mut input, _ang_vel, _transform, mut fbw, mut rocket_mode) in &mut player_query {
        // Toggle Rocket Mode with R key
        if keyboard_input.just_pressed(KeyCode::KeyR) {
            rocket_mode.enabled = !rocket_mode.enabled;
            println!("🚀 ROCKET MODE: {}", if rocket_mode.enabled { "ENABLED" } else { "DISABLED" });
        }

        // Toggle SAS with K key (for legacy FBW, not used with arcade physics)
        if keyboard_input.just_pressed(KeyCode::KeyK) {
            fbw.sas_enabled = !fbw.sas_enabled;
            println!("⚙️  SAS: {}", if fbw.sas_enabled { "ENABLED ✓" } else { "DISABLED ⚠️" });
        }
        
        // ARCADE PHYSICS: Full authority control (no SAS limiting)
        // Pitch (W/S) - Direct control (inverted: W pitches down, S pitches up)
        let target_pitch = if keyboard_input.pressed(KeyCode::KeyW) { -1.0 }
                          else if keyboard_input.pressed(KeyCode::KeyS) { 1.0 }
                          else { 0.0 };
        input.pitch = input.pitch.lerp(target_pitch, 0.1);

        // Roll (A/D) - Direct control
        let target_roll = if keyboard_input.pressed(KeyCode::KeyA) { -1.0 }
                         else if keyboard_input.pressed(KeyCode::KeyD) { 1.0 }
                         else { 0.0 };
        input.roll = input.roll.lerp(target_roll, 0.1);

        // Yaw (Q/E) - Direct control
        let target_yaw = if keyboard_input.pressed(KeyCode::KeyQ) { 1.0 } 
                        else if keyboard_input.pressed(KeyCode::KeyE) { -1.0 } 
                        else { 0.0 };
        input.yaw = input.yaw.lerp(target_yaw, 0.1);

        // Throttle (Shift/Ctrl)
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            input.throttle = (input.throttle + 0.01).min(1.0); // Afterburner
        } else if keyboard_input.pressed(KeyCode::ControlLeft) {
            input.throttle = (input.throttle - 0.01).max(0.0);
        }
    }
}

/// Fly-By-Wire Flight Control System
/// Implements PID controllers to stabilize the inherently unstable F-16
/// Adjusts PlayerInput values to match desired attitude
#[allow(dead_code)]
fn fly_by_wire_control(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (
            &Transform,
            &AngularVelocity,
            &mut PlayerInput,
            &mut FlightControlComputer,
        ),
        With<PlayerPlane>,
    >,
    mut debug_counter: Local<u32>,
) {
    for (transform, ang_vel, mut input, mut fbw) in &mut player_query {
        // Toggle FBW with L key
        if keyboard.just_pressed(KeyCode::KeyL) {
            fbw.enabled = !fbw.enabled;
            println!("🔧 FBW: {}", if fbw.enabled { "ENABLED" } else { "DISABLED" });
        }

        if !fbw.enabled {
            return; // FBW disabled, pass through raw inputs
        }

        let dt = time.delta_secs();
        if dt == 0.0 || dt > 0.1 { return; } // Skip if dt is 0 or suspiciously large

        // Get current attitude (Euler angles)
        let (current_roll, current_pitch, _current_yaw) = transform.rotation.to_euler(EulerRot::XYZ);

        // Get current rotation rates
        let omega = transform.rotation.inverse() * ang_vel.0;
        let pitch_rate = omega.x;
        let roll_rate = omega.z;

        // SIMPLIFIED: Direct attitude command with gentle scaling
        // Player input directly sets target attitude (not rate)
        fbw._target_pitch = input.pitch * 0.15;  // ±0.15 rad = ±8.6° max
        fbw._target_roll = input.roll * 0.25;    // ±0.25 rad = ±14° max

        // If no input, auto-level aggressively
        if input.pitch.abs() < 0.01 {
            fbw._target_pitch = 0.0; // Level flight
        }
        if input.roll.abs() < 0.01 {
            fbw._target_roll = 0.0; // Wings level
        }

        // ==========================
        // PITCH PID CONTROLLER
        // ==========================
        let pitch_error = fbw._target_pitch - current_pitch;
        fbw._pitch_error_integral += pitch_error * dt;
        fbw._pitch_error_integral = fbw._pitch_error_integral.clamp(-1.0, 1.0); // Anti-windup

        let pitch_error_derivative = (pitch_error - fbw._pitch_error_prev) / dt;
        let pitch_error_derivative = pitch_error_derivative.clamp(-10.0, 10.0); // Prevent spikes
        fbw._pitch_error_prev = pitch_error;

        let pitch_correction =
            fbw._pitch_kp * pitch_error +
            fbw._pitch_ki * fbw._pitch_error_integral +
            fbw._pitch_kd * pitch_error_derivative;

        // PHASE 3: Normalize rates to [-1, 1] range before applying damping gains
        // Real F-16 max pitch rate: ~300°/s = 5.24 rad/s
        const MAX_PITCH_RATE: f32 = 5.0;  // rad/s
        let normalized_pitch_rate = (pitch_rate / MAX_PITCH_RATE).clamp(-1.0, 1.0);

        // Add active rate damping (fight unwanted rotation)
        // Apply damping as normalized control input (max ±1.0)
        // CRITICAL: Must be strong enough to overcome instability
        let pitch_damping = -normalized_pitch_rate * 2.5;  // Was 0.5 - much stronger now

        // ==========================
        // ROLL PID CONTROLLER
        // ==========================
        let roll_error = fbw._target_roll - current_roll;
        fbw._roll_error_integral += roll_error * dt;
        fbw._roll_error_integral = fbw._roll_error_integral.clamp(-1.0, 1.0); // Anti-windup

        let roll_error_derivative = (roll_error - fbw._roll_error_prev) / dt;
        let roll_error_derivative = roll_error_derivative.clamp(-10.0, 10.0); // Prevent spikes
        fbw._roll_error_prev = roll_error;

        let roll_correction =
            fbw._roll_kp * roll_error +
            fbw._roll_ki * fbw._roll_error_integral +
            fbw._roll_kd * roll_error_derivative;

        // PHASE 3: Normalize roll rate for damping
        const MAX_ROLL_RATE: f32 = 5.0;   // rad/s
        let normalized_roll_rate = (roll_rate / MAX_ROLL_RATE).clamp(-1.0, 1.0);

        // Add active rate damping (fight unwanted rotation)
        let roll_damping = -normalized_roll_rate * 3.0;  // Was 0.5 - MUCH stronger to prevent divergence

        // Apply corrections + damping to control inputs
        // CRITICAL: When rates are extreme, damping MUST dominate over position correction
        // PITCH: Negate correction (for control mapping) but keep damping sign correct
        // Damping must always oppose rotation, so we add it directly without negation
        input.pitch = (-pitch_correction * 0.3 + pitch_damping).clamp(-1.0, 1.0);

        // ROLL: Reduce correction strength so damping can dominate when needed
        input.roll = (roll_correction * 0.3 + roll_damping).clamp(-1.0, 1.0);

        // Debug FBW every 30 frames (~0.5 seconds)
        *debug_counter += 1;
        if *debug_counter % 30 == 0 {
            println!(
                "🔧 FBW | Pitch: {:.2}° (tgt:{:.2}° rate:{:.2}rad/s corr:{:.3}) | Roll: {:.2}° (tgt:{:.2}° rate:{:.2}rad/s corr:{:.3})",
                current_pitch.to_degrees(),
                fbw._target_pitch.to_degrees(),
                pitch_rate,
                pitch_correction + pitch_damping,
                current_roll.to_degrees(),
                fbw._target_roll.to_degrees(),
                roll_rate,
                roll_correction + roll_damping,
            );
        }

        // Yaw is still direct (rudder coordination can be added later)
    }
}

/// The Heart of the Beast: JSBSim-style Physics
#[allow(dead_code)]
fn apply_aerodynamics(
    mut player_query: Query<
        (
            &PlayerInput,
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
            &mut ExternalForce,
            &mut ExternalTorque,
        ),
        With<PlayerPlane>,
    >,
    aero: Res<F16AeroData>,
    _time: Res<Time>,
) {
    let air_density = 1.225; // Sea Level density (kg/m^3)

    for (input, transform, velocity, ang_vel, mut ext_force, mut ext_torque) in &mut player_query {
        // 1. Get Velocity in Local Body Frame
        // Bevy: -Z = Forward, Y = Up, X = Right
        // Aero: X = Forward, Z = Down, Y = Right (We must map carefully)

        let mut v_world = velocity.0;

        // SPEED LIMITER: Cap max speed to prevent hypersonic runaway
        const MAX_SPEED: f32 = 350.0; // m/s (~783 mph, above target for margin)
        let current_speed = v_world.length();
        if current_speed > MAX_SPEED {
            v_world = v_world.normalize() * MAX_SPEED;
            // Note: This doesn't update the actual velocity component,
            // it just prevents forces from accelerating beyond this point
        }

        let v_body = transform.rotation.inverse() * v_world;
        let speed_sq = v_world.length_squared();
        let speed = v_world.length();

        if speed < 1.0 {
            // Apply simple thrust if stopped
            let thrust = transform.forward() * input.throttle * MAX_THRUST_NEWTONS;
            ext_force.apply_force(thrust);
            continue;
        }

        // 2. Calculate Alpha (AoA) and Beta (Sideslip)
        // Body Frame Mapping for calculation:
        // Forward speed (u) = -v_body.z
        // Side speed (v) = v_body.x
        // Vertical speed (w) = v_body.y (Positive = Up, so flow coming from down)

        // Alpha = atan2(w, u) -> atan2(Vertical, Forward)
        // Note: If plane is falling flat, v_body.y is negative (falling).
        // Flow is coming UP. Alpha should be positive.
        // Alpha = atan2(-v_body.y, -v_body.z)
        let alpha = (-v_body.y).atan2(-v_body.z);

        // Beta = atan2(v, u) -> atan2(Side, Forward)
        let beta = (v_body.x).atan2(-v_body.z);

        // 3. Dynamic Pressure (Q-Bar)
        let q_bar = 0.5 * air_density * speed_sq;
        let s = aero.wing_area;
        let b = aero.wing_span;
        let c = aero.wing_chord;

        // 4. Calculate Coefficients
        let cl = aero.cl_alpha.sample(alpha);

        // Drag: Base from alpha + airbrake + speed penalty
        // Add exponential drag at high speeds to limit max velocity naturally
        let speed_drag_factor = (speed / 200.0).powi(2) * 0.3; // Ramps up aggressively
        let cd = aero.cd_alpha.sample(alpha) + 0.05 * input._brake + speed_drag_factor;

        let cy = aero.cy_beta * beta;

        // 5. Calculate Forces (Wind Frame)
        // Lift acts Perpendicular to Velocity
        // Drag acts Opposite to Velocity
        let lift_mag = q_bar * s * cl;
        let drag_mag = q_bar * s * cd;
        let side_mag = q_bar * s * cy;

        // Direction Vectors
        let forward_dir = v_world.normalize(); // Direction of flight
        let right_dir = transform.right().as_vec3(); // Wing axis
        // Lift direction: Perpendicular to velocity and right wing.
        // Effective "Up" relative to airflow.
        let lift_dir = right_dir.cross(forward_dir).normalize();
        let drag_dir = -forward_dir;
        let side_dir = -right_dir; // Side force pushes opposite to slip

        let lift_force = lift_dir * lift_mag;
        let drag_force = drag_dir * drag_mag;
        let side_force = side_dir * side_mag;

        let thrust_force = transform.forward().as_vec3() * input.throttle * MAX_THRUST_NEWTONS;

        let total_force = lift_force + drag_force + side_force + thrust_force;

        // 6. Calculate Moments (Torque)
        // M = Q * S * Length * Coeff

        // Get Angular Rates in Body Frame
        let mut omega = transform.rotation.inverse() * ang_vel.0;

        // PHASE 1: CRITICAL SAFETY - Clamp angular velocity magnitude to prevent explosion
        // Real F-16 max control rate: ~300°/s = 5.24 rad/s
        // Allow 2x for aerodynamic effects: 10 rad/s = 573°/s
        const MAX_ANGULAR_VELOCITY: f32 = 10.0; // rad/s
        let omega_magnitude = omega.length();
        if omega_magnitude > MAX_ANGULAR_VELOCITY {
            omega = omega.normalize() * MAX_ANGULAR_VELOCITY;
        }

        // Bevy Axes:
        // X = Right. Rot around X = Pitch.
        // Y = Up. Rot around Y = Yaw.
        // Z = Back. Rot around Z = Roll.

        let pitch_rate = omega.x;
        let yaw_rate = omega.y;
        let roll_rate = omega.z;

        // PHASE 2: Calculate normalized rates with protection against division by small speed
        // and large rate values
        const MIN_SPEED_FOR_DAMPING: f32 = 5.0; // m/s - below this, disable rate damping
        let speed_for_damping = speed.max(MIN_SPEED_FOR_DAMPING);

        let norm_p = (roll_rate * b) / (2.0 * speed_for_damping);
        let norm_q = (pitch_rate * c) / (2.0 * speed_for_damping);
        let norm_r = (yaw_rate * b) / (2.0 * speed_for_damping);

        // Clamp normalized rates to reasonable bounds
        // Real F-16: these should be in [-0.3, 0.3] range typically
        const MAX_NORMALIZED_RATE: f32 = 1.0;
        let norm_p = norm_p.clamp(-MAX_NORMALIZED_RATE, MAX_NORMALIZED_RATE);
        let norm_q = norm_q.clamp(-MAX_NORMALIZED_RATE, MAX_NORMALIZED_RATE);
        let norm_r = norm_r.clamp(-MAX_NORMALIZED_RATE, MAX_NORMALIZED_RATE);

        // Coefficients
        let cm = aero.cm_alpha.sample(alpha)
               + aero.cm_elevator * input.pitch
               + aero.cm_q * norm_q;

        // EMERGENCY FIX: Scale down roll torque by 90% to prevent constant spinning
        // Aerodynamic roll forces are overpowering FBW control authority
        let cl_roll = (aero.cl_beta.sample(beta)
                     + aero.cl_aileron * input.roll
                     + aero.cl_p * norm_p) * 0.1;

        let cn_yaw = aero.cn_beta.sample(beta)
                   + aero.cn_rudder * input.yaw
                   + aero.cn_r * norm_r;

        // Torque Magnitude
        let pitch_torque = q_bar * s * c * cm;
        let roll_torque = q_bar * s * b * cl_roll;
        let yaw_torque = q_bar * s * b * cn_yaw;

        // Apply Torques (Bevy Frame)
        // Pitch = X axis (Right) -> +X is Pitch Up
        // Yaw = Y axis (Up) -> +Y is Yaw Left (Bevy is Right-Handed Y-Up) -> Aero +Cn is Yaw Right
        // Roll = Z axis (Back) -> +Z is Roll Left (Bevy is Right-Handed Y-Up) -> Aero +Cl is Roll Right

        // CORRECTION: Negate Yaw and Roll to match Bevy's coordinate system
        let torque_local = Vec3::new(pitch_torque, -yaw_torque, -roll_torque);
        let torque_world = transform.rotation * torque_local;

        // APPLY PHYSICS
        *ext_force = ExternalForce::default();
        ext_force.apply_force(total_force);

        *ext_torque = ExternalTorque::default();
        ext_torque.apply_torque(torque_world);
    }
}

// ============================================================================
// ARCADE FLIGHT PHYSICS (DISABLED - Experimental alternative to JSBSim)
// ============================================================================
// Kept for reference - uncomment to use arcade physics instead of JSBSim
/// ARCADE FLIGHT PHYSICS
/// Direct control like Ace Combat / StarFox - 100% stable and playable
/// Based on: F117A-remake (Bevy) and brihernandez's ArcadeJetFlightExample
///
/// This replaces the JSBSim aerodynamics with simple, fun, stable controls.
/// Player input directly controls rotation rates - no complex feedback loops.
fn arcade_flight_physics(
    mut player_query: Query<
        (
            &PlayerInput,
            &Transform,
            &LinearVelocity,
            &mut AngularVelocity,
            &mut ExternalForce,
            &RocketMode,
        ),
        With<PlayerPlane>,
    >,
) {
    // ===== TUNING CONSTANTS =====
    const ROLL_RATE: f32 = 2.5;
    const PITCH_RATE: f32 = 1.8;
    const YAW_RATE: f32 = 1.2;
    const DRAG_COEFFICIENT: f32 = 0.1;
    const MAX_THRUST_NEWTONS: f32 = 100000.0;  // INCREASED: Was 50000, now 100000 for sustained flight
    const SMOOTHING_FACTOR: f32 = 0.15;
    const BOOST_MULTIPLIER: f32 = 3.5;
    const BOOST_THRESHOLD: f32 = 0.8;

    for (input, transform, velocity, mut ang_vel, mut ext_force, rocket_mode) in &mut player_query {
        ext_force.clear();

        // ===== 1. LOCAL-SPACE ROTATION (Gemini's elegant approach) =====
        // Use transform basis vectors for proper 3D rotation in aircraft's local frame
        let right = transform.right().as_vec3();
        let up = transform.up().as_vec3();
        let forward = transform.forward().as_vec3();

        // Target rotation rates in LOCAL space (around plane's own axes)
        let target_omega = right * input.pitch * PITCH_RATE +
                          up * input.yaw * YAW_RATE +
                          forward * input.roll * ROLL_RATE;

        // Smooth interpolation for natural feel - NaN PROTECTION
        if !target_omega.is_nan() && target_omega.is_finite() {
             ang_vel.0 = ang_vel.0.lerp(target_omega, SMOOTHING_FACTOR);
        }

        // ===== 2. DRAG =====
        let speed = velocity.length();
        if speed > 1.0 && speed.is_finite() {
            // SAFE NORMALIZATION: Prevent division by zero if velocity is tiny
            let drag_force = -velocity.0.normalize_or_zero() * speed * speed * DRAG_COEFFICIENT;
            if !drag_force.is_nan() && drag_force.is_finite() {
                ext_force.apply_force(drag_force);
            }
        }

        // ===== 3. THRUST WITH VERTICAL COMPONENT (My approach, simplified) =====
        // GET PITCH ANGLE - More defensive
        let (_, pitch_angle, _) = transform.rotation.to_euler(EulerRot::XYZ);

        // VALIDATE PITCH IMMEDIATELY
        let safe_pitch = if pitch_angle.is_nan() || !pitch_angle.is_finite() {
            eprintln!("⚠️ Pitch angle is invalid! Resetting to 0.");
            0.0
        } else {
            pitch_angle
        };

        // CLAMP PITCH to reasonable range (prevents Euler angle singularities at ±π/2)
        let clamped_pitch = safe_pitch.clamp(-std::f32::consts::PI/2.0 + 0.01, std::f32::consts::PI/2.0 - 0.01);

        // Decompose thrust into forward and vertical components based on pitch
        // CLAMP INPUTS to prevent runaway values
        let safe_throttle = input.throttle.clamp(0.0, 1.0);
        
        let vertical_component = safe_throttle * MAX_THRUST_NEWTONS * clamped_pitch.sin();
        let forward_component = safe_throttle * MAX_THRUST_NEWTONS * clamped_pitch.cos();

        // Apply boost multiplier when throttle is high
        let mut boost_mult = if safe_throttle > BOOST_THRESHOLD { BOOST_MULTIPLIER } else { 1.0 };
        
        // Rocket mode overrides everything with massive thrust
        if rocket_mode.enabled {
            boost_mult = ROCKET_THRUST_MULTIPLIER;
        }
        
        // Final safety clamp on boost
        boost_mult = boost_mult.clamp(1.0, 20.0);

        let thrust_force = (forward * forward_component + up * vertical_component) * boost_mult;
        
        if !thrust_force.is_nan() && thrust_force.is_finite() {
            ext_force.apply_force(thrust_force);
        }

        // ===== 4. GRAVITY =====
        // Handled by Avian3D - no manual application needed
    }
}

fn update_flight_camera(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<PlayerPlane>)>,
    player_query: Query<(&Transform, &FlightCamera, &LinearVelocity), With<PlayerPlane>>,
) {
    if let (Ok(mut camera_transform), Ok((player_transform, _flight_camera, _velocity))) =
        (camera_query.get_single_mut(), player_query.get_single()) {

        // === STEP 1: Set camera position directly (no smoothing) ===
        // Position: 15 units behind plane, 5 units above plane center
        let local_offset = Vec3::new(0.0, 5.0, 15.0);
        camera_transform.translation = player_transform.transform_point(local_offset);

        // === STEP 2: Calculate desired camera rotation ===
        // Camera looks at plane center, using plane's up vector as camera up
        let look_target = player_transform.translation;
        let camera_up = player_transform.up().as_vec3();

        let temp_transform = Transform::IDENTITY
            .with_translation(camera_transform.translation)
            .looking_at(look_target, camera_up);

        let target_rotation = temp_transform.rotation;

        // === STEP 3: Smoothly rotate camera to follow plane ===
        // Smooth rotation prevents jerky head movement (15 rad/s interpolation speed)
        let t_rot = (15.0 * time.delta_secs()).min(1.0);
        camera_transform.rotation =
            camera_transform.rotation.slerp(target_rotation, t_rot);
    }
}

/// SYSTEM: Print flight diagnostics every 0.5 seconds
fn debug_flight_diagnostics(
    time: Res<Time>,
    mut timer_query: Query<&mut DiagnosticTimer>,
    player_query: Query<
        (
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
            &PlayerInput,
        ),
        With<PlayerPlane>,
    >,
) {
    let mut timer = match timer_query.get_single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    if let Ok((transform, velocity, ang_vel, input)) = player_query.get_single() {
        let altitude = transform.translation.y;
        let climb_rate = velocity.0.y;
        let speed = velocity.length();
        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        
        let roll_deg = roll.to_degrees();
        let pitch_deg = pitch.to_degrees();
        let yaw_deg = yaw.to_degrees();
        
        let turning = roll.abs() > 0.01 && pitch.abs() > 0.01;
        let yaw_rate_deg_per_sec = (transform.rotation.inverse() * ang_vel.0).y.to_degrees();
        let throttle_percent = (input.throttle * 100.0) as i32;

        println!("═════════════════════════════════════════════════════════════════════");
        println!(" VIPER EYE - TACTICAL FLIGHT INTERFACE");
        println!("─────────────────────────────────────────────────────────────────────");
        println!("ALT: {:>5.0} m  |  CLIMB: {:>+6.1} m/s  |  SPEED: {:>3.0} m/s  |  THR: {:>2}%",
            altitude, climb_rate, speed, throttle_percent);
        println!("ROLL: {:>6.1}°  |  PITCH: {:>6.1}°  |  YAW: {:>6.1}°",
            roll_deg, pitch_deg, yaw_deg);
        println!("INPUTS: [Pitch: {:>4.1}][Roll: {:>4.1}][Yaw: {:>4.1}][Throttle: {:>4.1}]",
            input.pitch, input.roll, input.yaw, input.throttle);
        println!("─────────────────────────────────────────────────────────────────────");
        println!("TURNING: {}  Yaw Rate: {:>5.1}°/s",
            if turning { "YES" } else { "NO " }, yaw_rate_deg_per_sec);
        println!("═════════════════════════════════════════════════════════════════════\n");
    }
}

// ===================================================================
// DEBUG SYSTEM - Flight Dynamics Monitor
// ===================================================================
fn debug_flight_dynamics(
    time: Res<Time>,
    mut last_debug: Local<f32>,
    player_query: Query<(
        &Transform,
        &AngularVelocity,
        &LinearVelocity,
        &PlayerInput,
        &FlightControlComputer
    ), With<PlayerPlane>>,
) {
    // Only print every 0.5 seconds to avoid spam
    *last_debug += time.delta_secs();
    if *last_debug < 0.5 {
        return;
    }
    *last_debug = 0.0;

    if let Ok((transform, ang_vel, lin_vel, input, fbw)) = player_query.get_single() {
        // Convert quaternion to Euler angles (in degrees)
        let (yaw, pitch, roll) = transform.rotation.to_euler(bevy::math::EulerRot::YXZ);
        let pitch_deg = pitch.to_degrees();
        let roll_deg = roll.to_degrees();
        let yaw_deg = yaw.to_degrees();

        // Convert angular velocity to body frame (rad/s)
        let omega = transform.rotation.inverse() * ang_vel.0;
        let pitch_rate = omega.x; // rad/s
        let roll_rate = omega.z;  // rad/s
        let yaw_rate = omega.y;   // rad/s

        // Convert to deg/s for readability
        let pitch_rate_deg = pitch_rate.to_degrees();
        let roll_rate_deg = roll_rate.to_degrees();
        let yaw_rate_deg = yaw_rate.to_degrees();

        // Speed
        let speed = lin_vel.length();
        let speed_kph = speed * 3.6;

        // Print comprehensive flight state
        println!("╔═══════════════════════════════════════════════════════════════════╗");
        println!("║ FLIGHT DYNAMICS DEBUG                                             ║");
        println!("╠═══════════════════════════════════════════════════════════════════╣");
        println!("║ ATTITUDE:  Pitch: {:>6.1}°  Roll: {:>6.1}°  Yaw: {:>6.1}°         ║",
                 pitch_deg, roll_deg, yaw_deg);
        println!("║ RATES:     Pitch: {:>6.1}°/s  Roll: {:>6.1}°/s  Yaw: {:>6.1}°/s   ║",
                 pitch_rate_deg, roll_rate_deg, yaw_rate_deg);
        println!("║ SPEED:     {:>5.0} m/s ({:>4.0} kph)                                ║",
                 speed, speed_kph);
        println!("║ INPUTS:    Pitch: {:>5.2}  Roll: {:>5.2}  Yaw: {:>5.2}  Thr: {:>4.1}%  ║",
                 input.pitch, input.roll, input.yaw, input.throttle * 100.0);
        println!("║ SYSTEMS:   FBW: {}  SAS: {}                                ║",
                 if fbw.enabled { "ON ✓" } else { "OFF " },
                 if fbw.sas_enabled { "ON ✓" } else { "OFF⚠" });
        println!("╚═══════════════════════════════════════════════════════════════════╝");
    }
}

// ===================================================================
// CRITICAL SAFETY: Hard limit angular velocity to prevent physics explosion
// ===================================================================
/// This runs AFTER aerodynamics applies torques, clamping the resulting velocity
/// Without this, the unstable F-16 aerodynamics cause exponential divergence
#[allow(dead_code)]
fn clamp_angular_velocity(
    mut query: Query<&mut AngularVelocity, With<PlayerPlane>>,
) {
    // Maximum safe angular velocity: 10 rad/s = ~573°/s
    // Real F-16 max roll rate: ~270°/s, we allow 2x for arcade feel
    const MAX_ANGULAR_VELOCITY: f32 = 10.0; // rad/s

    for mut ang_vel in &mut query {
        let magnitude = ang_vel.0.length();
        if magnitude > MAX_ANGULAR_VELOCITY {
            // Preserve direction, limit magnitude
            // This physically prevents the component from exceeding safe values
            ang_vel.0 = ang_vel.0.normalize() * MAX_ANGULAR_VELOCITY;
        }
    }
}


fn debug_flight_data(
    mut counter: Local<u32>,
    player_query: Query<(&Transform, &LinearVelocity, &PlayerInput), With<PlayerPlane>>,
) {
    *counter += 1;
    if *counter % 60 == 0 { 
        if let Ok((transform, velocity, input)) = player_query.get_single() {
            let speed = velocity.length();
            let altitude = transform.translation.y;
            let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
            println!(
                "[FLIGHT] ALT: {:.0} m | SPD: {:.0} m/s | P: {:.1}° R: {:.1}° Y: {:.1}° | THR: {:.0}%",
                altitude,
                speed,
                pitch.to_degrees(),
                roll.to_degrees(),
                yaw.to_degrees(),
                input.throttle * 100.0
            );
        }
    }
}

/// Handle global pause toggle (P key)
/// Runs in all states to allow unpausing
fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        match state.get() {
            GameState::Playing => {
                next_state.set(GameState::Paused);
                physics_time.pause();
                println!("⏸️  PAUSED - Physics Frozen");
            }
            GameState::Paused => {
                next_state.set(GameState::Playing);
                physics_time.unpause();
                println!("▶️  RESUMED - Physics Active");
            }
            _ => {}
        }
    }
}

fn handle_quit(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    // Use F10 to quit (ESC is now respawn)
    if keyboard_input.pressed(KeyCode::F10) {
        exit.send(AppExit::Success);
    }
}

fn handle_restart(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut PlayerInput,
            &mut FlightControlComputer,
        ),
        With<PlayerPlane>,
    >,
    mut commands: Commands,
    drone_query: Query<Entity, With<Drone>>,
    projectile_query: Query<Entity, With<Projectile>>, // Added Projectile query
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // F5 to respawn (ESC removed to avoid accidental restarts)
    if keyboard_input.just_pressed(KeyCode::F5) {
        if let Ok((mut transform, mut lin_vel, mut ang_vel, mut input, mut fbw)) =
            player_query.get_single_mut()
        {
            println!("🔄 RESPAWNING PLAYER AND RESETTING SWARM");
            
            // 1. Clear existing drones
            for drone_entity in &drone_query {
                commands.entity(drone_entity).despawn_recursive();
            }

            // 2. Clear existing projectiles (and their attached sounds)
            for proj_entity in &projectile_query {
                commands.entity(proj_entity).despawn_recursive();
            }

            // 3. Reset position to spawn point
            transform.translation = Vec3::new(0.0, 500.0, 0.0);
            transform.rotation = Quat::IDENTITY;

            // 3. Reset velocity and rotation
            lin_vel.0 = Vec3::new(0.0, 0.0, -100.0);
            ang_vel.0 = Vec3::ZERO;

            // 4. Reset input state
            *input = PlayerInput::default();

            // 5. Reset FBW state
            *fbw = FlightControlComputer::default();

            // 6. Spawn fresh fresh swarm
            spawn_initial_drone_swarm(&mut commands, &*asset_server, &mut *meshes, &mut *materials);

            println!("\n🔄 GAME RESTARTED\n");
        }
    }
}

fn debug_asset_loading(
    mut loaded: Local<bool>,
    scenes: Query<&SceneRoot>,
    asset_server: Res<AssetServer>,
) {
    if !*loaded && !scenes.is_empty() {
        for scene in &scenes {
            let load_state = asset_server.load_state(&scene.0);
            println!("Model load state: {:?}", load_state);
            if matches!(load_state, bevy::asset::LoadState::Loaded) {
                println!("✓ Fighter jet model loaded successfully!");
                *loaded = true;
            }
        }
    }
}

fn handle_shooting_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>, // Added mouse input
    time: Res<Time>,
    mut player_query: Query<(Entity, &Transform, &LinearVelocity, &mut LastShotTime), With<PlayerPlane>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sounds: Res<GameAssets>,
) {
    if let Ok((player_entity, player_transform, player_velocity, mut last_shot)) = player_query.get_single_mut() {
        let current_time = time.elapsed_secs();
        let can_shoot = (keyboard.pressed(KeyCode::Space) || mouse.pressed(MouseButton::Right))
            && (current_time - last_shot.time >= FIRE_COOLDOWN);

        if can_shoot {
            let gun_position_world = player_transform.transform_point(GUN_OFFSET);
            let forward = -player_transform.local_z();
            
            // FIX: Inherit player velocity so missiles don't fly backward in Rocket Mode
            // Missile speed is now RELATIVE to the plane (Plane Speed + Missile Speed)
            let bullet_velocity = player_velocity.0 + (forward * BULLET_SPEED);

            // Spawn missile with proper orientation and visuals
            let missile_entity = spawn_missile(&mut commands, &mut meshes, &mut materials, gun_position_world, player_transform.rotation, bullet_velocity);
            
            // Pass local GUN_OFFSET and parent to plane
            spawn_muzzle_flash(&mut commands, &mut meshes, &mut materials, GUN_OFFSET, Some(player_entity));

            // Play missile launch sound
            // Logic: Play heavy "Hero" sound MORE often (1 in 5)
            let mut rng = rand::thread_rng();
            let (sound, volume) = if rng.gen_bool(1.0 / 5.0) { 
                (sounds.missile_hero.clone(), 1.5) // 1.5x (Balanced loudness)
            } else {
                (sounds.missile_light.clone(), 1.0) // 1.0x (Standard loudness)
            };

            // Add slight pitch variation to everything
            let pitch_speed = rng.gen_range(0.9..1.1);

            // Attach audio to missile with MANUAL attenuation (fixes stereo file issues)
            commands.entity(missile_entity).with_children(|parent| {
                parent.spawn((
                    AudioPlayer(sound),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::new(volume),
                        speed: pitch_speed,
                        spatial: false, // DISABLED: Handled manually
                        ..default()
                    },
                    ManualAttenuation {
                        max_distance: 10000.0, // 10km audible range
                        reference_distance: 400.0, // 400m for long cinematic fade tail
                        base_volume: volume,
                        age: 0.0, // Start age at 0
                        doppler_ramp_time: 0.5, // 0.5s ramp to full Doppler (keeps initial punch)
                    },
                    Transform::IDENTITY, // REQUIRED: Allow position inheritance
                    Visibility::default(),
                ));
            });

            last_shot.time = current_time;
        }
    }
}

fn update_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile, &Transform, Option<&LinearVelocity>)>,
) {
    let delta = time.delta_secs();

    // SAFETY: Validate delta time
    if !delta.is_finite() || delta <= 0.0 {
        return;
    }

    for (entity, mut projectile, transform, velocity) in &mut projectile_query {
        // SAFETY: Validate projectile state
        if !transform.translation.is_finite() {
            eprintln!("⚠️ Projectile {:?} has invalid position! {:?}", entity, transform.translation);
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn_recursive();
            }
            continue;
        }

        if let Some(vel) = velocity {
            if !vel.0.is_finite() {
                eprintln!("⚠️ Projectile {:?} has invalid velocity! {:?}", entity, vel.0);
                if commands.get_entity(entity).is_some() {
                    commands.entity(entity).despawn_recursive();
                }
                continue;
            }
        }

        // SAFETY: Clamp position to reasonable bounds
        if transform.translation.length() > 200_000.0 {
            eprintln!("⚠️ Projectile {:?} traveled too far! Despawning.", entity);
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn_recursive();
            }
            continue;
        }

        // NOTE: No manual translation update here. 
        // Projectiles use RigidBody::Dynamic + LinearVelocity, so Avian3D moves them.

        projectile.lifetime -= delta;
        if projectile.lifetime <= 0.0 {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn handle_machine_gun_input(
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut player_query: Query<(Entity, &Transform, &LinearVelocity, &mut MachineGunState), With<PlayerPlane>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sounds: Res<GameAssets>,
) {
    if let Ok((player_entity, player_transform, player_velocity, mut mg_state)) = player_query.get_single_mut() {
        let current_time = time.elapsed_secs();
        let can_shoot = mouse.pressed(MouseButton::Left)
            && (current_time - mg_state.last_fired >= MG_FIRE_RATE);

        if can_shoot {
            mg_state.shot_count += 1;
            let is_tracer = mg_state.shot_count % 5 == 0;

            eprintln!("🔫 SHOT #{}: is_tracer={}", mg_state.shot_count, is_tracer);

            // Toggle between left and right wingtips
            let offset = if mg_state.fire_side {
                MG_OFFSET_RIGHT
            } else {
                MG_OFFSET_LEFT
            };
            mg_state.fire_side = !mg_state.fire_side;

            let bullet_position_world = player_transform.transform_point(offset);
            let forward = -player_transform.local_z();

            // Bullets inherit player velocity + muzzle velocity
            // Tracers are 0.75X speed for visual clarity (they appear slower than regular bullets)
            let speed_mult = if is_tracer { 0.75 } else { 1.0 };
            let bullet_velocity = player_velocity.0 + (forward * MG_SPEED * speed_mult);

            // SAFETY: Validate calculated values before spawning
            if !bullet_position_world.is_finite() {
                eprintln!("❌ MG SPAWN ABORTED: Invalid position (player transform may be corrupt)");
                eprintln!("   player_pos={:?}, offset={:?}", player_transform.translation, offset);
                return; // Skip this shot
            }

            if !forward.is_finite() || !bullet_velocity.is_finite() {
                eprintln!("❌ MG SPAWN ABORTED: Invalid velocity calculation");
                eprintln!("   forward={:?}, bullet_vel={:?}", forward, bullet_velocity);
                return; // Skip this shot
            }

            // DEBUG: Print spawn info for ALL bullets
            eprintln!("   SPAWN: pos={:.1?}, player_pos={:.1?}, offset={:?}",
                bullet_position_world, player_transform.translation, offset);

            // Spawn the tracer bullet with slight randomization
            let mut rng = rand::thread_rng();
            // Randomized length for "streaking" effect (tracers are longer but not massive)
            // FIX A: Reduced ranges to prevent capsules from extending far above spawn point
            let length_mult = if is_tracer {
                rng.gen_range(1.2..1.8)  // Was 2.5..4.0 (7.5-12m total) → now 3.6-5.4m total
            } else {
                rng.gen_range(0.8..1.2)  // Was 1.5..3.0 (4.5-9m total) → now 2.4-3.6m total
            };
            
            // Randomized brightness for "pulsing" effect (tracers are brighter)
            let glow_mult = if is_tracer {
                rng.gen_range(4.0..8.0)
            } else {
                rng.gen_range(2.0..5.0)
            };
            
            spawn_bullet(&mut commands, &mut meshes, &mut materials, bullet_position_world, player_transform.rotation, bullet_velocity, length_mult, glow_mult, is_tracer);
            
            // Pass local offset and parent to plane
            spawn_muzzle_flash(&mut commands, &mut meshes, &mut materials, offset, Some(player_entity));

            // Play machine gun sound (Non-spatial for "punch")
            // Pitch shift tracers slightly higher for feedback
            let pitch_speed = if is_tracer {
                rng.gen_range(1.05..1.15)
            } else {
                rng.gen_range(0.95..1.05)
            };

            commands.spawn((
                AudioPlayer(sounds.machine_gun.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(0.5),
                    speed: pitch_speed,
                    spatial: false,
                    ..default()
                },
            ));

            mg_state.last_fired = current_time;
        }
    }
}

fn update_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Bullet, &Transform, Option<&LinearVelocity>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, mut bullet, transform, _velocity) in &mut bullet_query {
        // Safety check for NaN
        if !transform.translation.is_finite() {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn_recursive();
            }
            continue;
        }

        // --- Heat Trail for Super Tracers ---
        if bullet.is_tracer {
            // Spawn 2 tiny red lingering sparks per frame for density
            for i in 0..2 {
                let mesh = meshes.add(Sphere::new(0.12)); // Buffed from 0.04
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.0, 0.0),
                    emissive: LinearRgba::rgb(300.0, 0.0, 0.0), // Massive trail brightness
                    ..default()
                });

                // Offset trail slightly in LOCAL space
                let local_offset = Vec3::new(
                    (i as f32 * 0.2) - 0.1,
                    0.0, 
                    (i as f32 * 0.2) - 0.1
                );
                let world_offset = transform.rotation.mul_vec3(local_offset);

                commands.spawn((
                    VisualDebris {
                        velocity: Vec3::ZERO, // Linger in air
                        lifetime: 0.2,
                    },
                    Transform::from_translation(transform.translation + world_offset),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                ));
            }
        }

        bullet.lifetime -= delta;
        if bullet.lifetime <= 0.0 {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn bullet_drone_collision(
    mut commands: Commands,
    mut bullets: Query<(Entity, &Transform, &mut Bullet)>,
    mut drones: Query<(Entity, &Transform, &mut drone::Drone), Without<Bullet>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    for (bullet_entity, bullet_transform, mut bullet) in &mut bullets {
        let current_pos = bullet_transform.translation;
        let prev_pos = bullet.previous_translation;
        
        for (drone_entity, drone_transform, mut drone_comp) in &mut drones {
            let drone_pos = drone_transform.translation;
            
            // --- CCD: Segment-to-Point Distance ---
            // A = prev_pos, B = current_pos, C = drone_pos
            let ab = current_pos - prev_pos;
            let ac = drone_pos - prev_pos;
            let bc = drone_pos - current_pos;
            
            let dot_ac_ab = ac.dot(ab);
            let dot_ab_ab = ab.dot(ab);
            
            let distance = if dot_ab_ab <= 0.0 {
                // Point segment (bullet hasn't moved yet)
                ac.length()
            } else {
                let t = (dot_ac_ab / dot_ab_ab).clamp(0.0, 1.0);
                let projection = prev_pos + ab * t;
                (drone_pos - projection).length()
            };
            
            // Hit radius: 15.0m (Bullet is small, but we check the path)
            if distance < 15.0 { 
                // Apply damage to drone (Tracers do double damage)
                let damage = if bullet.is_tracer { MG_DAMAGE * 2.0 } else { MG_DAMAGE };
                drone_comp.health -= damage;
                
                // Spawn hit sparks at the closest point on the path
                let hit_visual_pos = if dot_ab_ab > 0.0 {
                    let t = (dot_ac_ab / dot_ab_ab).clamp(0.0, 1.0);
                    prev_pos + ab * t
                } else {
                    current_pos
                };
                spawn_hit_spark(&mut commands, &mut meshes, &mut materials, hit_visual_pos);

                // Despawn bullet on hit
                if commands.get_entity(bullet_entity).is_some() {
                    commands.entity(bullet_entity).despawn_recursive();
                }
                
                // Check if drone died
                if drone_comp.health <= 0.0 {
                    println!("💀 DRONE DESTROYED BY GUNFIRE!");

                    // Play explosion sound (mimic missile death logic)
                    let mut rng = rand::thread_rng();
                    let explosion_sound = if rng.gen_bool(0.3) {
                        game_assets.explosion_heavy.clone()
                    } else {
                        game_assets.explosion_standard.clone()
                    };

                    commands.spawn((
                        AudioPlayer(explosion_sound),
                        PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::new(1.0),
                            spatial: true,
                            ..default()
                        },
                        Transform::from_translation(drone_transform.translation),
                    ));

                    // Spawn explosion visual effect
                    commands.spawn((
                        Mesh3d(meshes.add(Mesh::from(Sphere { radius: 30.0 }))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 0.5, 0.0),
                            emissive: LinearRgba::rgb(5.0, 2.0, 0.0),
                            ..default()
                        })),
                        Transform::from_translation(drone_transform.translation),
                        ExplosionEffect { lifetime: 0.0, max_lifetime: 1.0 },
                    ));

                    commands.entity(drone_entity).despawn_recursive();
                }
                break; 
            }
        }
        
        // After collision check, update previous translation for next frame
        bullet.previous_translation = current_pos;
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    orientation: Quat, // Plane's orientation - needed for forward offset
    velocity: Vec3,
    length_mult: f32,
    glow_mult: f32,
    is_tracer: bool,
) {
    // PHYSICAL THICKNESS BOOST: 0.05 -> 0.12 for regular, 0.08 -> 0.18 for tracer
    // At high flight speeds, we need physical width to avoid pixel-thin lines
    // FIX C: Reduced tracer radius 0.25→0.18 (bloom glow provides visual size)
    let radius = if is_tracer { 0.18 } else { 0.12 };
    let capsule_half_length = 1.5 * length_mult;
    let mesh = meshes.add(Capsule3d::new(radius, capsule_half_length));
    
    let (base_color, emissive) = if is_tracer {
        eprintln!("🔴 CREATING RED TRACER MATERIAL");
        // FIX F: Reduced emissive for new bloom settings (500→40)
        // With intensity: 0.3 bloom, this creates visible soft red glow (~1-2m halo)
        // Added green/blue tint (2.0, 2.0) for warmer glow
        (Color::srgb(1.0, 0.0, 0.0), LinearRgba::rgb(40.0, 2.0, 2.0))
    } else {
        eprintln!("🟡 Creating yellow bullet material");
        // FIX F: Reduced emissive for new bloom settings (200/150→15/12)
        // Subtle warm yellow glow, doesn't compete with tracers
        (Color::srgb(1.0, 1.0, 0.0), LinearRgba::rgb(15.0, 12.0, 0.0))
    };

    eprintln!("   Material: base_color={:?}, emissive={:?}", base_color, emissive);

    let material = materials.add(StandardMaterial {
        base_color,
        emissive,
        unlit: true, // CRITICAL: Emissive materials should be unlit to show properly
        ..default()
    });

    // Capsule3d is Y-axis aligned by default — rotate Y axis to point along velocity
    // (Previous "fix" used Vec3::NEG_Z which made bullets fly sideways — now corrected)
    let rotation = if velocity.length_squared() > 0.001 {
        let vel_normalized = velocity.normalize();
        let dot = vel_normalized.dot(Vec3::Y).abs();
        if dot > 0.999 {
            Quat::IDENTITY // Near-vertical velocity — avoid degenerate arc (returns NaN)
        } else {
            let rot = Quat::from_rotation_arc(Vec3::Y, vel_normalized);
            if rot.is_finite() && rot.is_normalized() { rot } else { Quat::IDENTITY }
        }
    } else {
        Quat::IDENTITY
    };

    // FIX: Offset spawn position forward by half the capsule length
    // Use plane's forward direction (not velocity) to prevent upward offset when climbing

    // CRITICAL: Validate orientation quaternion before using it
    if !orientation.is_finite() || !orientation.is_normalized() {
        eprintln!("❌ SPAWN ERROR: Invalid orientation quaternion!");
        eprintln!("   orientation={:?}", orientation);
        return; // Abort spawn - corrupt plane transform
    }

    // FIX: Spawn bullets directly at gun position without additional forward offset
    // The wingtip offsets (MG_OFFSET_LEFT/RIGHT) already position bullets correctly
    // Adding plane_forward offset causes upward displacement when pitching up
    //
    // Previous bug: plane_forward * 1.5 added +1.48m Y when pitched 80° up,
    // making tracers appear at top of screen due to their high emissive/bloom
    //
    // FIX B: Spawn bullets directly at gun position - no offset needed
    // The wingtip offsets (MG_OFFSET_LEFT/RIGHT) already position correctly
    // With capsule length fixed (Fix A), this offset is unnecessary and causes
    // issues when pitched due to world-space Y being wrong when oriented
    let adjusted_position = position;

    eprintln!("   SPAWN: pos={:.1?}, is_tracer={}", adjusted_position, is_tracer);

    // CRITICAL: Validate position before spawning to prevent NaN bullets
    if !adjusted_position.is_finite() {
        eprintln!("❌ BULLET SPAWN ERROR: Invalid position detected!");
        eprintln!("   position={:?}, is_tracer={}", position, is_tracer);
        return; // Abort spawn - do not create invalid bullet
    }

    // Additional validation: Check velocity is valid
    if !velocity.is_finite() || velocity.length_squared() < 0.1 {
        eprintln!("❌ BULLET SPAWN ERROR: Invalid velocity detected!");
        eprintln!("   velocity={:?}, length={:?}", velocity, velocity.length());
        return; // Abort spawn
    }

    commands.spawn((
        Bullet {
            lifetime: 2.0,
            previous_translation: adjusted_position,
            is_tracer,
        },
        Transform::from_translation(adjusted_position)
            .with_rotation(rotation),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        RigidBody::Dynamic,
        LinearVelocity(velocity),
        Collider::capsule(radius, 1.5),
        GravityScale(0.0),
        Mesh3d(mesh),
        MeshMaterial3d(material),
    ));
}

fn spawn_hit_spark(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let mut rng = rand::thread_rng();
    
    // Spawn 8-12 tiny debris sparks for a "shattering" effect
    for _ in 0..rng.gen_range(8..13) {
        let length = rng.gen_range(0.2..1.5);
        // Using a thin capsule for sparks to look like motion-blurred debris
        let mesh = meshes.add(Capsule3d::new(0.03, length));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.6, 0.1),
            emissive: LinearRgba::rgb(40.0, 15.0, 0.0),
            ..default()
        });

        // Give sparks a random "pop" velocity
        let velocity = Vec3::new(
            rng.gen_range(-40.0..40.0),
            rng.gen_range(-40.0..40.0),
            rng.gen_range(-40.0..40.0)
        );

        // SAFETY: Handle near-zero velocity to prevent NaN rotation
        let rotation = if velocity.length_squared() > 0.0001 {
            Quat::from_rotation_arc(Vec3::Z, velocity.normalize())
        } else {
            Quat::IDENTITY
        };

        commands.spawn((
            VisualDebris { 
                velocity,
                lifetime: rng.gen_range(0.2..0.6) 
            }, 
            Transform::from_translation(position)
                .with_rotation(rotation),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            Mesh3d(mesh),
            MeshMaterial3d(material),
        ));
    }
}

fn drone_projectile_collision(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    mut drones: Query<(Entity, &mut Drone, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
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

                    // Dynamic Impact System: Randomize explosion sound
                    let mut rng = rand::thread_rng();
                    let explosion_sound = if rng.gen_bool(0.3) {
                        game_assets.explosion_heavy.clone() // 30% Chance: Heavy Rumble
                    } else {
                        game_assets.explosion_standard.clone() // 70% Chance: Standard Snap
                    };

                    // Play 3D Explosion Sound
                    commands.spawn((
                        AudioPlayer(explosion_sound),
                        PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::new(1.0),
                            spatial: true, // Enable 3D audio
                            ..default()
                        },
                        Transform::from_translation(drone_transform.translation),
                    ));

                    // Spawn explosion visual effect
                    commands.spawn((
                        Mesh3d(meshes.add(Mesh::from(Sphere { radius: 30.0 }))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 0.5, 0.0),  // Orange
                            emissive: LinearRgba::rgb(5.0, 2.0, 0.0),  // Glow
                            ..default()
                        })),
                        Transform::from_translation(drone_transform.translation),
                        // Add explosion effect component to auto-despawn
                        ExplosionEffect { lifetime: 0.0, max_lifetime: 1.0 },
                    ));

                    // Despawn the drone
                    commands.entity(drone_entity).despawn();
                }
            }
        }
    }
}

fn drone_player_collision(
    mut commands: Commands,
    drone_query: Query<(Entity, &Transform), With<Drone>>,
    player_query: Query<&Transform, With<PlayerPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sounds: Res<GameAssets>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation;

    for (drone_entity, drone_transform) in &drone_query {
        let distance = drone_transform.translation.distance(player_pos);
        
        if distance < 20.0 {
            println!("💥 KAMIKAZE HIT! Drone exploded on player!");
            
            // Spawn explosion at collision point
            spawn_huge_explosion(&mut commands, &mut meshes, &mut materials, drone_transform.translation);

            // Play explosion sound
            commands.spawn((
                AudioPlayer(sounds.explosion.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::new(1.0),
                    ..default()
                },
            ));

            // Despawn drone
            commands.entity(drone_entity).despawn_recursive();
        }
    }
}

fn handle_projectile_collisions(
    mut collision_events: EventReader<Collision>,
    projectile_query: Query<Entity, With<Projectile>>,
    ground_query: Query<Entity, (With<Collider>, Without<Projectile>, Without<PlayerPlane>, Without<Objective>)>,
    objective_query: Query<(Entity, &Transform), With<Objective>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sounds: Res<GameAssets>,
) {
    for Collision(contacts) in collision_events.read() {
        let projectile_entity = if projectile_query.contains(contacts.entity1) {
            Some(contacts.entity1)
        } else if projectile_query.contains(contacts.entity2) {
            Some(contacts.entity2)
        } else {
            None
        };
        
        if let Some(bullet) = projectile_entity {
            let other_entity = if bullet == contacts.entity1 {
                contacts.entity2
            } else {
                contacts.entity1
            };
            
            // Check if bullet still exists (prevent B0003 warning)
            if commands.get_entity(bullet).is_none() {
                continue;
            }

            if ground_query.contains(other_entity) {
                commands.entity(bullet).despawn_recursive();
            } else if let Ok((target_entity, target_transform)) = objective_query.get(other_entity) {
                // Check if target still exists
                if commands.get_entity(target_entity).is_some() {
                    println!("🎯 TARGET DESTROYED!");
                    
                    let explosion_pos = target_transform.translation;
                    commands.entity(target_entity).despawn_recursive();
                    commands.entity(bullet).despawn_recursive();
                    
                    spawn_huge_explosion(&mut commands, &mut meshes, &mut materials, explosion_pos);

                    // Play explosion sound
                    commands.spawn((
                        AudioPlayer(sounds.explosion.clone()),
                        PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            volume: bevy::audio::Volume::new(1.0),
                            ..default()
                        },
                    ));
                }
            }
        }
    }
}

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

fn spawn_muzzle_flash(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    parent: Option<Entity>,
) {
    // 1. Point Light (Brighter and larger range)
    let light_entity = commands.spawn((
        MuzzleFlash { lifetime: 0.0 },
        PointLight {
            intensity: MUZZLE_FLASH_INTENSITY * 10.0, 
            color: Color::srgb(1.0, 0.8, 0.4),
            range: 15.0, // Restored range
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    )).id();

    // 2. Visual "Flash" (Increased for visibility)
    let mesh = meshes.add(Sphere::new(0.25)); 
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.5),
        emissive: LinearRgba::rgb(150.0, 100.0, 30.0), // High intensity
        ..default()
    });

    let mesh_entity = commands.spawn((
        MuzzleFlash { lifetime: 0.0 }, // Re-use MuzzleFlash component for cleanup
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position).with_scale(Vec3::splat(1.0)), 
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    )).id();

    // If a parent is provided (the jet), attach the flash so it moves with the jet
    if let Some(p) = parent {
        commands.entity(p).add_child(light_entity);
        commands.entity(p).add_child(mesh_entity);
    }
}

/// Spawn a realistic-looking missile (AIM-120 style)
fn spawn_missile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    orientation: Quat,
    velocity: Vec3,
) -> Entity {
    // Create missile body (elongated cylinder pointing along -Z)
    let body_mesh = meshes.add(Cylinder::new(MISSILE_BODY_RADIUS, MISSILE_LENGTH));
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8), // Light gray
        metallic: 0.8,
        perceptual_roughness: 0.2,
        ..default()
    });

    // Nose cone material (red tip)
    let nose_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.1, 0.1), // Dark red
        metallic: 0.5,
        ..default()
    });

    // Exhaust glow material
    let exhaust_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.5, 0.0), // Orange
        emissive: LinearRgba::rgb(5.0, 2.0, 0.0), // Bright orange glow
        ..default()
    });

    // Spawn missile parent entity with physics
    commands.spawn((
        Projectile { lifetime: BULLET_LIFETIME },
        Transform::from_translation(position)
            .with_rotation(orientation),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        RigidBody::Dynamic,
        LinearVelocity(velocity),
        Collider::capsule(MISSILE_BODY_RADIUS, MISSILE_LENGTH),
        GravityScale(0.0),
    ))
    .with_children(|parent| {
        // Missile body (rotated 90° because Bevy's Cylinder is Y-axis aligned)
        parent.spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_material),
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ));

        // Nose cone (small sphere at front)
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(MISSILE_BODY_RADIUS * 1.2))),
            MeshMaterial3d(nose_material.clone()),
            Transform::from_xyz(0.0, 0.0, -MISSILE_LENGTH * 0.6),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ));

        // Exhaust glow (small sphere at back)
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(MISSILE_BODY_RADIUS * 0.8))),
            MeshMaterial3d(exhaust_material),
            Transform::from_xyz(0.0, 0.0, MISSILE_LENGTH * 0.6),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ));

        // Add 4 fins (small boxes around the body)
        let fin_mesh = meshes.add(Cuboid::new(MISSILE_FIN_SIZE, 0.02, MISSILE_FIN_SIZE));
        let fin_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3), // Dark gray
            metallic: 0.9,
            ..default()
        });

        // Fins at 90° intervals
        for i in 0..4 {
            let angle = i as f32 * std::f32::consts::FRAC_PI_2;
            let offset = Vec3::new(
                angle.cos() * MISSILE_BODY_RADIUS * 1.5,
                angle.sin() * MISSILE_BODY_RADIUS * 1.5,
                0.0,
            );
            
            parent.spawn((
                Mesh3d(fin_mesh.clone()),
                MeshMaterial3d(fin_material.clone()),
                Transform::from_translation(offset),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
            ));
        }
    })
    .id()
}
/// Component for explosion effects that despawn after a time
#[derive(Component)]
struct ExplosionEffect {
    lifetime: f32,
    max_lifetime: f32,
}

/// Check for ground collision and create explosion effect.
/// Resets rotation and angular velocity on respawn to avoid physics AABB panic (invalid bounds).
fn check_ground_collision(
    mut commands: Commands,
    mut player_query: Query<(
        Entity,
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
    ), With<PlayerPlane>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sounds: Res<GameAssets>,
) {
    const SOFT_CEILING: f32 = 0.5; // Don't let physics see us below this (avoids AABB edge cases)

    for (_entity, mut transform, mut velocity, mut ang_vel) in &mut player_query {
        // SAFETY FIX: Check for NaN values that crash avian3d physics
        if transform.translation.is_nan() || transform.translation.x.is_nan() || transform.translation.y.is_nan() || transform.translation.z.is_nan()
            || velocity.x.is_nan() || velocity.y.is_nan() || velocity.z.is_nan() {
            eprintln!("⚠️ SAFETY: Detected NaN in player transform/velocity! Resetting to safe position.");
            transform.translation = Vec3::new(0.0, 500.0, 0.0);
            *velocity = LinearVelocity::ZERO;
            *ang_vel = AngularVelocity::ZERO;
            continue;
        }

        // SAFETY FIX: Check for Extreme Values (Dark Bar Glitch)
        if transform.translation.y > 100_000.0 || transform.translation.y < -1000.0 {
             eprintln!("⚠️ SAFETY: Detected Extreme Y Position! Resetting.");
             transform.translation = Vec3::new(0.0, 500.0, 0.0);
             *velocity = LinearVelocity::ZERO;
             *ang_vel = AngularVelocity::ZERO;
             continue;
        }

        // Calculate dynamic terrain height at player position
        let terrain_height = get_terrain_height(transform.translation.x, transform.translation.z);
        // Collision threshold is terrain height + safety margin (need clearance above ground)
        const CRASH_MARGIN: f32 = 5.0; // Reduced from 50.0 to fix mid-air collisions
        let ground_level = terrain_height + CRASH_MARGIN;

        if transform.translation.y <= ground_level {
            let crash_speed = velocity.length();

            if crash_speed > 50.0 {
                println!("💥 MASSIVE EXPLOSION! Speed: {:.0} m/s", crash_speed);
                spawn_huge_explosion(&mut commands, &mut meshes, &mut materials, transform.translation);

                // Play crash sound
                commands.spawn((
                    AudioPlayer(sounds.crash.clone()),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Despawn,
                        volume: bevy::audio::Volume::new(1.0),
                        ..default()
                    },
                ));

                // Full reset so physics never sees invalid rotation (avoids AABB panic)
                transform.translation = Vec3::new(0.0, 500.0, 0.0);
                transform.rotation = Quat::IDENTITY;
                *velocity = LinearVelocity(Vec3::new(0.0, 0.0, -200.0));
                *ang_vel = AngularVelocity::default();
            } else {
                // Soft landing: keep above ground and zero downward velocity
                transform.translation.y = ground_level + SOFT_CEILING;
                velocity.0.y = velocity.0.y.max(0.0);

                // FIX: Reset rotation if plane is severely tilted (prevents stuck on side)
                // Check roll angle - if more than 45° tilted, reset to upright
                let (pitch, yaw, roll) = transform.rotation.to_euler(bevy::math::EulerRot::XYZ);
                if roll.abs() > 0.785 || pitch.abs() > 1.57 {  // 45° roll or 90° pitch
                    eprintln!("⚠️ SOFT LANDING: Resetting rotation (roll={:.1}°, pitch={:.1}°)",
                        roll.to_degrees(), pitch.to_degrees());

                    // Reset to level flight orientation
                    transform.rotation = Quat::IDENTITY;

                    // Give small forward velocity to help unstick
                    *velocity = LinearVelocity(Vec3::new(0.0, 0.0, -50.0));
                    *ang_vel = AngularVelocity::ZERO;
                }
            }
        } else if transform.translation.y < ground_level + SOFT_CEILING {
            // Clamp just above ground so we never penetrate (helps collision/physics stability)
            transform.translation.y = ground_level + SOFT_CEILING;
        }
    }
}

/// Spawn a huge explosion with particles and light
fn spawn_huge_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let fireball_mesh = meshes.add(Sphere::new(15.0));
    let fireball_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.0),
        emissive: LinearRgba::rgb(10.0, 3.0, 0.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(fireball_mesh),
        MeshMaterial3d(fireball_material),
        Transform::from_translation(position + Vec3::Y * 10.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ExplosionEffect { lifetime: 0.0, max_lifetime: 2.0 },
    ));

    commands.spawn((
        PointLight {
            intensity: 50000.0,
            color: Color::srgb(1.0, 0.6, 0.2),
            range: 200.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(position + Vec3::Y * 10.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ExplosionEffect { lifetime: 0.0, max_lifetime: 1.0 },
    ));

    for i in 0..20 {
        let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
        let speed = 50.0 + (i as f32 * 5.0);
        let debris_velocity = Vec3::new(
            angle.cos() * speed,
            30.0 + (i as f32 * 2.0),
            angle.sin() * speed,
        );

        let debris_mesh = meshes.add(Sphere::new(0.5));
        let debris_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            metallic: 0.8,
            ..default()
        });

        commands.spawn((
            Mesh3d(debris_mesh),
            MeshMaterial3d(debris_material),
            Transform::from_translation(position),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            RigidBody::Dynamic,
            Collider::sphere(0.5), // FIX: Add collider to dynamic rigid body
            LinearVelocity(debris_velocity),
            GravityScale(1.0),
            ExplosionEffect { lifetime: 0.0, max_lifetime: 5.0 },
        ));
    }
}

/// Update and despawn explosion effects
fn update_explosion_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ExplosionEffect)>,
) {
    for (entity, mut effect) in &mut query {
        effect.lifetime += time.delta_secs();
        if effect.lifetime >= effect.max_lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Spawn particle effects from jet exhaust based on throttle
fn spawn_afterburner_particles(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&Transform, &PlayerInput, &LinearVelocity), With<PlayerPlane>>,
    emitter_query: Query<&AfterburnerParticles, With<PlayerPlane>>,
) {
    // #region agent log
    let (player_ok, emitter_ok) = (player_query.get_single(), emitter_query.get_single());
    if player_ok.is_err() || emitter_ok.is_err() {
        debug_log("spawn_afterburner_particles", "query get_single failed", &format!(r#"{{"player_ok":{},"emitter_ok":{}}}"#, player_ok.is_ok(), emitter_ok.is_ok()), "D");
        return;
    }
    let (player_transform, input, player_velocity) = player_ok.unwrap();
    let emitter = emitter_ok.unwrap();
    // #endregion

    // NaN SAFETY: Don't spawn particles if throttle is corrupted
    if input.throttle.is_nan() {
        return;
    }

    if input.throttle < emitter.spawn_threshold {
        // #region agent log
        debug_log("spawn_afterburner_particles", "throttle below threshold", &format!(r#"{{"throttle":{},"spawn_threshold":{}}}"#, input.throttle, emitter.spawn_threshold), "A");
        // #endregion
        return;
    }

    let throttle_factor = (input.throttle - emitter.spawn_threshold) / (1.0 - emitter.spawn_threshold);
    let actual_spawn_rate = emitter.spawn_rate * throttle_factor;
    let spawn_count = actual_spawn_rate as u32;
    // #region agent log
    debug_log("spawn_afterburner_particles", "spawn rate computed", &format!(r#"{{"throttle":{},"throttle_factor":{},"actual_spawn_rate":{},"spawn_count_u32":{}}}"#, input.throttle, throttle_factor, actual_spawn_rate, spawn_count), "A");
    // #endregion

    // Exhaust always behind plane in world space (avoids flame appearing at wing when banked)
    let back = (-player_transform.forward().as_vec3()).normalize_or_zero();
    const EXHAUST_DISTANCE: f32 = 2.8; // Further back so flame is outside exhaust, not inside fuselage
    const EXHAUST_SHIFT_LEFT: f32 = 2.6; // Nudge left so flame is at center exhaust
    let world_pos: Vec3 = player_transform.translation
        + back * EXHAUST_DISTANCE
        - player_transform.right().as_vec3() * EXHAUST_SHIFT_LEFT;
    
    // #region agent log
    if spawn_count > 0 {
        debug_log("spawn_afterburner_particles", "spawn world_pos", &format!(r#"{{"world_pos":[{},{},{}],"player_translation":[{},{},{}],"back":[{},{},{}]}}"#, world_pos.x, world_pos.y, world_pos.z, player_transform.translation.x, player_transform.translation.y, player_transform.translation.z, back.x, back.y, back.z), "B");
    }
    // #endregion

    for _ in 0..spawn_count {

            let backward_velocity = player_transform.forward().as_vec3() * -20.0;
            // Inherit 20% of player velocity for "drag" effect (smoke trail)
            // If player is moving fast forward, particles shouldn't stop instantly
            let inherited_velocity = player_velocity.0 * 0.2;

            let random_spread = Vec3::new(
                (rand::random::<f32>() - 0.5) * 5.0,
                (rand::random::<f32>() - 0.5) * 5.0,
                (rand::random::<f32>() - 0.5) * 5.0,
            );
            let velocity = backward_velocity + inherited_velocity + random_spread;

            let flame_index = ((time.elapsed_secs() * 15.0) as usize) % 6 + 1;
            let texture_path = format!("particles/flame_0{}.png", flame_index);
            let texture_handle = asset_server.load(&texture_path);

            let material = StandardMaterial {
                base_color_texture: Some(texture_handle),
                base_color: Color::srgba(1.0, 0.35, 0.05, 0.95), // Red-orange flame
                emissive: LinearRgba::rgb(8.0, 2.5, 0.2),      // Bright orange flame glow
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true, // Visible from both sides
                cull_mode: None,
                ..default()
            };

            let material_handle = materials.add(material);
            
            // Dynamic scaling: Boost makes flame HUGE
            let size = if input.throttle > 0.8 {
                2.5 + (input.throttle - 0.8) * 5.0
            } else {
                1.2 + input.throttle * 1.5
            };
            
            // Safety Clamp to prevent giant needles
            let size = size.clamp(0.1, 10.0);
            
            let quad_mesh = meshes.add(Mesh::from(Rectangle::new(size, size)));

            commands.spawn((
                Particle {
                    lifetime_remaining: emitter.particle_lifetime,
                    lifetime_max: emitter.particle_lifetime,
                    velocity,
                },
                Transform::from_translation(world_pos),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                MeshMaterial3d(material_handle),
                Mesh3d(quad_mesh),
            ));
        }
}

/// Update particles: movement, fade, despawn
fn update_visual_debris(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut VisualDebris)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut debris) in &mut query {
        transform.translation += debris.velocity * delta;
        debris.lifetime -= delta;
        if debris.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Particle>)>,
    mut log_acc: Local<f32>,
) {
    let camera_transform = camera_query.get_single().ok();

    // #region agent log
    *log_acc += time.delta_secs();
    let should_log = *log_acc >= 1.0;
    if should_log {
        *log_acc = 0.0;
    }
    let mut log_count: u32 = 0;
    let mut log_first_pos = Vec3::ZERO;
    let mut log_first_life = -1.0_f32;
    // #endregion

    for (entity, mut transform, mut particle, material_handle) in &mut particle_query {
        // NaN CHECK
        if transform.translation.is_nan() {
            commands.entity(entity).despawn();
            continue;
        }

        // #region agent log
        log_count += 1;
        if log_count == 1 {
            log_first_pos = transform.translation;
            log_first_life = particle.lifetime_remaining;
        }
        // #endregion
        particle.lifetime_remaining -= time.delta_secs();

        if particle.lifetime_remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let drift = Vec3::Y * 5.0 * time.delta_secs();
        transform.translation += particle.velocity * time.delta_secs() + drift;

        // Face the camera (billboarding)
        if let Some(cam_transform) = camera_transform {
            transform.look_at(cam_transform.translation, Vec3::Y);
        }

        let opacity = particle.lifetime_remaining / particle.lifetime_max;
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color.set_alpha(opacity);
        }
    }

    // #region agent log
    if should_log {
        debug_log("update_particles", "particle count and first", &format!(r#"{{"particle_count":{},"first_pos":[{},{},{}],"first_lifetime_remaining":{}}}"#, log_count, log_first_pos.x, log_first_pos.y, log_first_pos.z, log_first_life), "E");
    }
    // #endregion
}

/// Diagnostic system to check if Tree SceneRoot children are being spawned
fn debug_tree_hierarchy(
    query: Query<(Entity, &Children), With<Tree>>,
) {
    let mut total_trees_with_children = 0;
    let mut total_children_count = 0;

    for (entity, children) in query.iter().take(5) {
        if children.len() > 0 {
            total_trees_with_children += 1;
            total_children_count += children.len();
            eprintln!("🔍 Tree {:?} has {} children (Scene children spawned!)", entity, children.len());
        }
    }

    if total_trees_with_children == 0 {
        eprintln!("⚠️  DEBUG: Sampled 5 trees, NONE have children! Scene loading might be broken.");
        eprintln!("⚠️  If this persists, try removing #Scene0 from asset paths or checking file format.");
    } else {
        eprintln!("✓ Trees have children - Scene loading works! (avg {} children per tree)", total_children_count / total_trees_with_children.max(1));
    }
}

fn set_window_icon(
    winit_windows: NonSend<WinitWindows>,
    game_assets: Res<GameAssets>,
    images: Res<Assets<Image>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let Some(primary) = winit_windows.get_window(primary_entity) else {
        return;
    };

    if let Some(image) = images.get(&game_assets.icon) {
        let width = image.width();
        let height = image.height();
        let rgba = image.data.clone();

        match Icon::from_rgba(rgba, width, height) {
            Ok(icon) => {
                primary.set_window_icon(Some(icon));
                eprintln!("✅ Window icon set successfully!");
            }
            Err(e) => {
                eprintln!("❌ Failed to set window icon: {:?}", e);
            }
        }
    }
}
