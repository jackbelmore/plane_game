# GEMINI TEXTURE IMPLEMENTATION PROMPT

**Role:** Senior Rust Graphics Engineer
**Task:** Fix the invisible ground texture bug in `plane_game` (Bevy 0.15).

## Context
The game currently attempts to load a JPG texture asynchronously in `setup_scene` and assign it to a `StandardMaterial`. Due to Bevy bug #15081 (bind group not refreshing on async load), the ground remains invisible (or uses a 1x1 placeholder).

We will implement a **Two-Phase Fix**:
1.  **Immediate Visual Fix:** Generate a procedural grass texture at startup. This bypasses async loading entirely and guarantees visible terrain.
2.  **Architecture Fix:** Introduce a `GameState` machine and `bevy_asset_loader`. This is the "correct" way to handle assets in Bevy, ensuring everything is loaded *before* the game starts.

---

## STEP 1: Better Procedural Grass (Immediate Fix)

Instead of a simple XOR pattern (which looks digital/aliased), we will use the `rand` crate to generate a noisy "grass-like" texture.

### 1. Create `src/procedural_textures.rs`

```rust
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    },
};
use rand::Rng;

pub fn create_grass_texture() -> Image {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 1024;
    
    // 1. Generate Noise Data
    // We'll use a simple "white noise" with a green tint, but at a higher resolution
    // to simulate grass blades when viewed from a distance.
    let mut data = Vec::with_capacity(WIDTH * HEIGHT * 4);
    let mut rng = rand::thread_rng();

    for _y in 0..HEIGHT {
        for _x in 0..WIDTH {
            // Base Green: R=30-50, G=100-160, B=20-40
            // We vary the intensity to create "blades"
            let intensity = rng.gen_range(0.8..1.2);
            
            let r = (40.0 * intensity) as u8;
            let g = (130.0 * intensity) as u8;
            let b = (30.0 * intensity) as u8;
            
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(255); // Alpha
        }
    }

    // 2. Create Image
    let mut image = Image::new(
        Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    // 3. Set Sampler to Repeat (Tiling)
    // Critical for terrain: The texture must tile seamlessly
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        ..default()
    });

    image
}
```

### 2. Update `src/main.rs`

*   **Add Module:** `mod procedural_textures;`
*   **Modify `setup_scene`:**
    *   REMOVE the async JPG load (`asset_server.load(...)`).
    *   REMOVE `GrassTextureLoading` resource insertion.
    *   ADD call to `create_grass_texture()`:
        ```rust
        let grass_image = procedural_textures::create_grass_texture();
        let grass_handle = images.add(grass_image);
        ```
    *   Assign `grass_handle` to the `StandardMaterial`.
*   **Cleanup:** Remove the `check_grass_texture_loaded` system entirely.

---

## STEP 2: Introduce Game State (Architecture Prep)

The current code runs everything in `Startup` or `Update`. To use `bevy_asset_loader` later, we need a proper state machine.

### 1. Define State Enum (in `src/main.rs`)

```rust
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
    // Paused, // Future use
    // GameOver, // Future use
}
```

### 2. Refactor App Building

*   Change `setup_scene` to run `OnEnter(GameState::Playing)` instead of `Startup`.
*   Temporarily, for Step 1, make `Loading` transition immediately to `Playing`:
    ```rust
    // Temporary scaffolding system
    fn transition_to_game(mut next_state: ResMut<NextState<GameState>>) {
        next_state.set(GameState::Playing);
    }
    ```

---

## STEP 3: Asset Loader Implementation (The Real Fix)

Once Step 1 and 2 are working, we implement the robust solution.

### 1. Add Dependencies
`cargo add bevy_asset_loader`

### 2. Define Asset Collections
Create `src/assets.rs`:

```rust
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "textures/grass/grass_BaseColor.png")] // NOTE: User must convert JPG->PNG
    pub grass_texture: Handle<Image>,
    
    #[asset(path = "models/f16.glb")]
    pub player_model: Handle<Scene>,
}
```

### 3. Configure Loading State in `main.rs`

```rust
App::new()
    .init_state::<GameState>()
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .target_state(GameState::Playing)
            .load_collection::<GameAssets>()
    )
    // ...
```

---

## Execution Order

1.  **Execute Step 1 FIRST.** This gives us a playable game immediately.
2.  **Verify** the grass looks okay (it will be "noisy" green, but visible).
3.  **Execute Step 2 & 3** to professionalize the codebase.

## Command for AI Agent
"Implement Step 1 from GEMINI_TEXTURE_IMPLEMENTATION_PROMPT.md. Create the procedural texture module, wire it into main.rs, and remove the broken async loading code. Ensure the project compiles and runs."