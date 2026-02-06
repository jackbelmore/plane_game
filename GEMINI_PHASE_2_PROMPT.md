# Gemini Prompt - Phase 2: Professional Asset Loading System

**Priority:** MEDIUM | **Effort:** 2-3 hours | **Complexity:** Medium
**After missiles + sounds are working, implement this**

---

## Your Mission

Upgrade from procedural textures to a **professional asset loading system** using `bevy_asset_loader`. This enables:
- ✅ Loading real JPG/PNG texture files
- ✅ Proper async handling (no more bind group bugs)
- ✅ Integrated audio asset loading
- ✅ Clean LoadingState architecture
- ✅ Easy to extend for future assets

---

## Current State

You've implemented Phase 1 (procedural grass). Now Phase 2 makes it production-grade.

**Before (Procedural):**
```rust
// Texture generated at runtime
let grass_image = procedural_textures::create_grass_texture();
```

**After (Professional):**
```rust
// Texture loaded from file with proper async
#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "textures/grass/grass_BaseColor.png")]
    pub grass_texture: Handle<Image>,
}
```

---

## Implementation Steps

### STEP 1: Add Dependency

```bash
cargo add bevy_asset_loader
```

### STEP 2: Create GameState Enum

**File:** `src/main.rs` (add near the top, after imports)

```rust
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
```

### STEP 3: Create Asset Collection

**File:** `src/assets.rs` (NEW FILE)

```rust
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "textures/grass/grass_BaseColor.png")]
    pub grass_texture: Handle<Image>,

    // Optional: Add more textures here as we expand
    // #[asset(path = "textures/grass/grass_Normal.png")]
    // pub grass_normal: Handle<Image>,
}

// For now, keep procedural as backup
#[derive(Resource)]
pub struct ProceduralTextureAsset(pub Handle<Image>);
```

### STEP 4: Update `src/main.rs`

#### 4a. Add module declaration (top of file, line ~12)
```rust
mod assets;
```

#### 4b. Update imports (after existing use statements)
```rust
use bevy_asset_loader::prelude::*;
use assets::GameAssets;
```

#### 4c. Add LoadingState to App builder (in main function)

Find where you have:
```rust
let mut app = App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        // ... window config ...
    }))
```

And add these lines AFTER `.add_plugins()` and BEFORE `.add_systems()`:

```rust
app.init_state::<GameState>()
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Playing)
            .load_collection::<GameAssets>()
    );
```

#### 4d. Move setup_scene to Playing state

Find:
```rust
.add_systems(Startup, setup_scene)
```

Change to:
```rust
.add_systems(OnEnter(GameState::Playing), setup_scene)
```

(This ensures setup_scene runs AFTER assets are loaded)

#### 4e. Update setup_scene function signature

**Before:**
```rust
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // ...
    let grass_image = procedural_textures::create_grass_texture();
    let grass_texture_handle = images.add(grass_image);
}
```

**After:**
```rust
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,  // NEW: Get from loader
) {
    // ... rest of setup ...

    // Use loaded texture instead of generating
    let grass_texture_handle = game_assets.grass_texture.clone();

    let ground_material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(grass_texture_handle),
        perceptual_roughness: 0.9,
        reflectance: 0.02,
        metallic: 0.0,
        ..default()
    });

    // ... rest stays the same ...
}
```

### STEP 5: Convert JPG to PNG

The JPG files have metadata issues in Bevy. Convert them:

```bash
# From target/release directory (where the game runs)
cd target/release/assets/textures/grass/

# Convert JPG to PNG
ffmpeg -i grass_BaseColor.jpg grass_BaseColor.png
ffmpeg -i grass_Normal.jpg grass_Normal.png

# Verify files exist
ls -lh grass_BaseColor.png
```

**Or manually:**
- Open `grass_BaseColor.jpg` in any image editor
- Export as PNG (File → Export As → grass_BaseColor.png)
- Save in `assets/textures/grass/`

### STEP 6: Optional - Add Loading Screen UI

If you want to show a loading message while assets load:

**Add to setup_scene or in a new system:**

```rust
fn show_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // This runs during GameState::Loading
    commands.spawn((
        Text::new("LOADING..."),
        TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

// Add to app:
// .add_systems(OnEnter(GameState::Loading), show_loading_screen)
```

---

## Testing Checklist

After implementation:

```
Build:
  [ ] cargo check passes
  [ ] cargo build --release succeeds
  [ ] No compilation errors

Runtime:
  [ ] Game launches
  [ ] Shows "Loading..." briefly (or just starts)
  [ ] Enters GameState::Playing
  [ ] setup_scene runs after assets loaded
  [ ] No "Path not found" errors for grass_BaseColor.png

Visual:
  [ ] Grass texture visible
  [ ] Texture looks similar to procedural version
  [ ] No visible difference from Phase 1
  [ ] Seamless tiling across chunks

Performance:
  [ ] FPS still 60+
  [ ] No stutter during asset load
  [ ] Load time <2 seconds
```

---

## Troubleshooting

### Error: "Path not found: textures/grass/grass_BaseColor.png"
**Solution:**
- Ensure PNG file exists in `assets/textures/grass/`
- Run: `cp -r assets target/release/`
- Check file case (Linux is case-sensitive)

### Error: "Could not find an asset loader"
**Solution:**
- Ensure `bevy_asset_loader` added: `cargo add bevy_asset_loader`
- Ensure GameState is declared and `.init_state::<GameState>()` is in app

### Game runs but shows procedural texture still
**Solution:**
- Check if `setup_scene` is using `game_assets.grass_texture`
- Verify `OnEnter(GameState::Playing)` is set correctly
- Add debug: `eprintln!("Loading assets...");` in setup_scene

---

## Commit Message

```
Feature: Professional asset loading with bevy_asset_loader

- Add bevy_asset_loader dependency for proper async asset handling
- Create GameState enum (Loading → Playing)
- Implement AssetCollection in src/assets.rs
- Move setup_scene to OnEnter(GameState::Playing)
- Load grass texture from PNG file instead of procedural generation
- Keep procedural texture as fallback in case load fails
- Convert JPG to PNG (fixes metadata issues)
- Add LoadingState configuration
- No visual change (texture looks same as procedural)
- Ready for Phase 3: Easy to add more assets (audio, models, etc)
```

---

## After This Works

**Phase 2 Complete!**

Now you can:
1. ✅ Load any texture file (not just procedural)
2. ✅ Add audio assets to GameAssets collection
3. ✅ Add drone/player models to asset loader
4. ✅ Extend with splat maps in Phase 3

**Phase 3+ Future Work:**
- Add multiple terrain textures to GameAssets
- Implement splat map system
- Load audio files via asset loader (instead of FromWorld)

---

## Questions for You

1. **JPG vs PNG:** Should we keep the JPG files or fully delete them?
   - Recommend: Delete (PNG is cleaner)

2. **Loading screen:** Do you want a visible "LOADING..." message?
   - Recommend: No (load is fast enough, 1-2 seconds)

3. **Audio assets:** Should we add audio files to GameAssets now?
   - Recommend: Wait until you confirm textures work first

---

## Timeline

- **Implementation:** 1.5-2 hours
- **Testing:** 30 minutes
- **Total:** 2-2.5 hours

After missiles + sounds are done, this is the natural next step!

---

## Success Criteria

When complete:
- ✅ Game loads assets properly (no async issues)
- ✅ Grass texture loads from PNG file
- ✅ No visual difference from procedural (intentional)
- ✅ Architecture ready for Phase 3+ features
- ✅ Code is production-quality
- ✅ One clean commit

**Ready to implement?** Let me know when you start!
