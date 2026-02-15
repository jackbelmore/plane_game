# GEMINI PHASE 2 IMPROVED PROMPT

**Priority:** MEDIUM | **Effort:** 2-2.5 hours | **Complexity:** Medium
**Goal:** Professionalize asset loading architecture using `bevy_asset_loader`.

---

## 🚀 The Upgrade Plan

We are moving from a "procedural prototype" and "async startup chaos" to a **Production-Grade State Machine**.

**Key Changes:**
1.  **Strict State Machine:** Game starts in `Loading` → transitions to `Playing` only when ready.
2.  **Unified Asset Loading:** Textures AND Audio load together. No more "pop" or "stutter" on first sound.
3.  **Startup Cleanup:** All gameplay spawning systems move from `Startup` to `OnEnter(GameState::Playing)`.

---

## 📋 Implementation Steps

### STEP 1: Add Dependency
```bash
cargo add bevy_asset_loader
```

### STEP 2: Define Game State
**File:** `src/main.rs` (Top of file, near imports)

```rust
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
```

### STEP 3: Create Asset Collections
**File:** `src/assets.rs` (Create New File)

```rust
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // --- Textures ---
    #[asset(path = "textures/grass/grass_BaseColor.png")]
    pub grass_texture: Handle<Image>,

    // Future: Add Normal Map
    // #[asset(path = "textures/grass/grass_Normal.png")]
    // pub grass_normal: Handle<Image>,

    // --- Audio ---
    #[asset(path = "sounds/engine.ogg")]
    pub engine_loop: Handle<AudioSource>,
    
    #[asset(path = "sounds/missile.ogg")]
    pub missile_launch: Handle<AudioSource>,
    
    #[asset(path = "sounds/explosion.ogg")]
    pub explosion: Handle<AudioSource>,
    
    #[asset(path = "sounds/warning.ogg")]
    pub warning: Handle<AudioSource>,
    
    #[asset(path = "sounds/crash.ogg")]
    pub crash: Handle<AudioSource>,
    
    #[asset(path = "sounds/wind.ogg")]
    pub wind: Handle<AudioSource>,
}
```

### STEP 4: Refactor `src/main.rs`

#### 4a. Module & Imports
```rust
mod assets; // Add module
use bevy_asset_loader::prelude::*;
use assets::GameAssets;
```

#### 4b. Remove Old `SoundAssets` Resource
*   Delete the `struct SoundAssets` definition and its `FromWorld` implementation.
*   We will use `GameAssets` directly instead.

#### 4c. Initialize Loading State (in `main()`)
Replace the standard setup with the loader configuration:

```rust
// Inside main()
App::new()
    .add_plugins(DefaultPlugins)
    // ... plugins ...
    .init_state::<GameState>() // Initialize State
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .target_state(GameState::Playing)
            .load_collection::<GameAssets>()
    )
    // ...
```

#### 4d. MIGRATE SYSTEMS (Crucial!)
Move **ALL** gameplay initialization from `Startup` to `OnEnter(GameState::Playing)`.

**Find these lines:**
```rust
.add_systems(Startup, setup_scene)
.add_systems(Startup, spawn_realistic_clouds)
.add_systems(Startup, spawn_objectives)
.add_systems(Startup, spawn_turrets)
.add_systems(Startup, spawn_player)
```

**Change to:**
```rust
.add_systems(OnEnter(GameState::Playing), (
    setup_scene,
    spawn_realistic_clouds,
    spawn_objectives,
    spawn_turrets,
    spawn_player
))
```

#### 4e. Update `setup_scene`
Use the pre-loaded assets.

```rust
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>, // Inject Assets
) {
    // ... (Light & Camera setup remains same) ...

    // === GROUND ===
    // Use the handle from GameAssets (Already loaded!)
    let grass_texture_handle = game_assets.grass_texture.clone();

    let ground_material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(grass_texture_handle),
        perceptual_roughness: 0.9,
        reflectance: 0.02,
        ..default()
    });
    
    // ... (Rest of function) ...
}
```

#### 4f. Update Sound Usage
Any system that used `Res<SoundAssets>` (like `update_engine_audio`) needs to be updated to use `Res<GameAssets>` and access fields directly (e.g., `game_assets.engine_loop`).

---

## ✅ Verification Checklist

1.  **Assets Exist:** Verified `assets/textures/grass/grass_BaseColor.png` exists.
2.  **Compilation:** Run `cargo check` to ensure all `SoundAssets` references are updated to `GameAssets`.
3.  **Runtime:** Run `cargo run --release`.
    *   Game should start.
    *   Brief (milliseconds) black screen/loading.
    *   Game appears with Grass Texture visible.
    *   Audio plays immediately (no stutter).

---

## 🔧 Command for AI
"Execute the plan in GEMINI_PHASE_2_IMPROVED_PROMPT.md. This involves adding the dependency, creating the asset collection, and heavily refactoring main.rs to use the new GameState and GameAssets system."