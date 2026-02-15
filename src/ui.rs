use bevy::prelude::*;
use crate::{PlayerPlane, drone::Drone, GameState, assets::GameAssets};
use avian3d::prelude::LinearVelocity;

#[derive(Component)]
pub struct SpeedText;

#[derive(Component)]
pub struct AltText;

#[derive(Component)]
pub struct ThreatText;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
           .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)));
    }
}

fn setup_hud(
    mut commands: Commands, 
    _asset_server: Res<AssetServer>,
    _game_assets: Res<GameAssets>,
) {
    // REQUIRED: 2D Camera for UI to render on top of 3D
    commands.spawn((
        Camera2d,
        Camera {
            order: 1, // Render after 3D (default is 0)
            ..default()
        },
    ));

    // Top Left: Flight Data
    commands.spawn((
        Text::new("SPD: 0 m/s\nALT: 0 m"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)), // HUD Green
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        SpeedText,
    ));

    // Top Right: Threat Counter
    commands.spawn((
        Text::new("THREATS: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)), // Threat Red
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        ThreatText,
    ));
}

fn update_hud(
    mut speed_query: Query<&mut Text, (With<SpeedText>, Without<ThreatText>)>,
    mut threat_query: Query<&mut Text, (With<ThreatText>, Without<SpeedText>)>,
    player_query: Query<(&Transform, &LinearVelocity), With<PlayerPlane>>,
    drone_query: Query<&Drone>,
) {
    // Update Flight Data
    if let Ok((transform, velocity)) = player_query.get_single() {
        let speed = velocity.0.length();
        let altitude = transform.translation.y;
        
        if let Ok(mut text) = speed_query.get_single_mut() {
            text.0 = format!("SPD: {:.0} m/s\nALT: {:.0} m", speed, altitude);
        }
    }

    // Update Threat Count
    let threat_count = drone_query.iter().count();
    if let Ok(mut text) = threat_query.get_single_mut() {
        text.0 = format!("THREATS: {}", threat_count);
    }
}
