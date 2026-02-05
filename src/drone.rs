use bevy::prelude::*;

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
        app.add_systems(Update, move_drones);
    }
}

/// Spawns the Kamikaze drone using the drone model mesh, with a dark gray fallback if it fails.
pub fn spawn_beaver_drone(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    println!("DEBUG: Spawning drone at {:?}", position);
    
    // Attempt to load the actual drone model mesh (using same pattern as trees)
    let drone_handle: Handle<Mesh> = asset_server.load("models/drone.glb#Mesh0/Primitive0");
    
    // Material: Dark gray to match the expected drone look
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        perceptual_roughness: 0.8,
        metallic: 0.1,
        emissive: LinearRgba::rgb(0.3, 0.3, 0.3),
        ..default()
    });

    commands.spawn((
        Drone {
            health: 50.0,
            speed: 150.0, // Keeping the high speed for visibility/combat testing
        },
        KamikazeBehavior,
        Mesh3d(drone_handle),
        MeshMaterial3d(material),
        Transform {
            translation: position,
            scale: Vec3::splat(1.8),
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
        },
        InheritedVisibility::default(),
    ));
}

/// Simple movement system: moves the drone forward based on its local rotation
fn move_drones(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Drone)>,
) {
    for (entity, mut transform, drone) in &mut query {
        let forward = transform.forward();
        let move_vec = forward * drone.speed * time.delta_secs();
        transform.translation += move_vec;
        
        // Log movement occasionally for debugging
        if time.elapsed_secs() as i32 % 5 == 0 && entity.index() % 2 == 0 {
             println!("DEBUG: Drone {:?} moving, pos: {:?}", entity, transform.translation);
        }
    }
}
