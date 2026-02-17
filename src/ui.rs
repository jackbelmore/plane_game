use bevy::prelude::*;
use crate::{PlayerPlane, drone::Drone, GameState, assets::GameAssets};
use avian3d::prelude::LinearVelocity;

#[derive(Component)]
pub struct SpeedText;

#[derive(Component)]
pub struct AltText;

#[derive(Component)]
pub struct ThreatText;

#[derive(Component)]
pub struct PauseText;

#[derive(Component)]
pub struct AltitudeWarningState {
    pub is_warning: bool,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Spawning), setup_hud)
           .add_systems(Update, (
               update_hud.run_if(in_state(GameState::Playing)),
               update_pause_visibility, // Runs always to toggle visibility
           ));
    }
}

fn setup_hud(
    mut commands: Commands, 
    _asset_server: Res<AssetServer>,
    _game_assets: Res<GameAssets>,
) {
    // NOTE: Camera2d removed - caused multi-camera HDR black screen bug (Bevy 0.15)
    // UI (Node components) renders through the main Camera3d automatically in Bevy 0.15

    // Top Left: Flight Data - SPLIT INTO TWO TEXT COMPONENTS
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ))
    .with_children(|parent| {
        // Speed text (always green)
        parent.spawn((
            Text::new("SPD: 0 m/s"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)), // HUD Green
            SpeedText,
        ));

        // Altitude text (changes to red when low)
        parent.spawn((
            Text::new("ALT: 0 m"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)), // Start green
            AltText,
            AltitudeWarningState { is_warning: false },
        ));
    });

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

    // Center: Pause Indicator (initially hidden)
    commands.spawn((
        Text::new("PAUSED\n\nPress P to Resume"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // Yellow warning
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(50.0),
            ..default()
        },
        Visibility::Hidden, // Start hidden
        PauseText,
    ));
}

fn update_hud(
    mut speed_query: Query<&mut Text, (With<SpeedText>, Without<AltText>, Without<ThreatText>)>,
    mut alt_query: Query<(&mut Text, &mut TextColor, &mut AltitudeWarningState), (With<AltText>, Without<SpeedText>, Without<ThreatText>)>,
    mut threat_query: Query<&mut Text, (With<ThreatText>, Without<SpeedText>, Without<AltText>)>,
    player_query: Query<(&Transform, &LinearVelocity), With<PlayerPlane>>,
    drone_query: Query<&Drone>,
) {
    // Update Flight Data
    if let Ok((transform, velocity)) = player_query.get_single() {
        let speed = velocity.0.length();
        let altitude = transform.translation.y;

        // Update speed text
        if let Ok(mut text) = speed_query.get_single_mut() {
            text.0 = format!("SPD: {:.0} m/s", speed);
        }

        // Update altitude text AND color
        if let Ok((mut text, mut color, mut warning_state)) = alt_query.get_single_mut() {
            text.0 = format!("ALT: {:.0} m", altitude);

            // WARNING THRESHOLD: Same as old audio trigger
            const ALTITUDE_WARNING_THRESHOLD: f32 = 100.0;
            let should_warn = altitude < ALTITUDE_WARNING_THRESHOLD;

            // Only update color if state changed (performance)
            if should_warn != warning_state.is_warning {
                warning_state.is_warning = should_warn;
                color.0 = if should_warn {
                    Color::srgb(1.0, 0.0, 0.0) // Red warning
                } else {
                    Color::srgb(0.0, 1.0, 0.0) // Green normal
                };
            }
        }
    }

    // Update Threat Count
    let threat_count = drone_query.iter().count();
    if let Ok(mut text) = threat_query.get_single_mut() {
        text.0 = format!("THREATS: {}", threat_count);
    }
}

fn update_pause_visibility(
    game_state: Res<State<GameState>>,
    mut pause_query: Query<&mut Visibility, With<PauseText>>,
) {
    if let Ok(mut visibility) = pause_query.get_single_mut() {
        *visibility = if game_state.get() == &GameState::Paused {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
