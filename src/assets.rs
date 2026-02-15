use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // --- UI Assets ---
    #[asset(path = "ui/logo.png")]
    pub logo: Handle<Image>,

    #[asset(path = "ui/icon.png")]
    pub icon: Handle<Image>,

    // --- Textures ---
    #[asset(path = "textures/grass/grass_BaseColor.png")]
    pub grass_texture: Handle<Image>,

    // Normal map for surface detail
    #[asset(path = "textures/grass/grass_Normal.png")]
    pub grass_normal: Handle<Image>,

    // --- Audio ---
    #[asset(path = "sounds/engine.ogg")]
    pub engine_loop: Handle<AudioSource>,

    #[asset(path = "sounds/missile_hero.ogg")]
    pub missile_hero: Handle<AudioSource>,

    #[asset(path = "sounds/missile_light.ogg")]
    pub missile_light: Handle<AudioSource>,
    
    #[asset(path = "sounds/explosion.ogg")]
    pub explosion: Handle<AudioSource>,

    #[asset(path = "sounds/explosion_standard.ogg")]
    pub explosion_standard: Handle<AudioSource>,

    #[asset(path = "sounds/explosion_heavy.ogg")]
    pub explosion_heavy: Handle<AudioSource>,
    
    #[asset(path = "sounds/warning.ogg")]
    pub warning: Handle<AudioSource>,
    
    #[asset(path = "sounds/crash.ogg")]
    pub crash: Handle<AudioSource>,
    
    #[asset(path = "sounds/wind.ogg")]
    pub wind: Handle<AudioSource>,

    #[asset(path = "sounds/machine_gun.ogg")]
    pub machine_gun: Handle<AudioSource>,
}
