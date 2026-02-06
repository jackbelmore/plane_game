# Gemini Prompt - Phase 2: Professional Asset Loading (IMPROVED)

**Priority:** MEDIUM | **Effort:** 2-3 hours | **Complexity:** Medium
**After missiles + sounds are working, implement this**

---

## Key Improvements from Gemini's Analysis

This revised version addresses three critical issues:

1. ✅ **Asset Conversion First** - Convert JPG to PNG before code changes
2. ✅ **Audio Assets Included** - Migrate sound loading to asset loader (prevents stutter)
3. ✅ **All Startup Systems** - Move EVERYTHING to OnEnter(GameState::Playing) not just setup_scene

---

## Your Mission

Upgrade to professional asset loading with **bevy_asset_loader**, handling:
- ✅ Grass texture (PNG, not JPG)
- ✅ Sound assets (engine, missiles, explosions)
- ✅ All game initialization (proper state machine)
- ✅ No more async bind group bugs
- ✅ No stutter on first sound
- ✅ Clean, extensible architecture

---

## Implementation Steps (Revised Order)

### STEP 0: Asset Preparation (DO THIS FIRST!)

**Why first:** If PNG doesn't exist when code tries to load it, the game will panic.

```bash
# Copy assets if not already there
cp -r assets target/release/

# Navigate to grass textures
cd target/release/assets/textures/grass/

# Convert JPG to PNG using ffmpeg
ffmpeg -i grass_BaseColor.jpg -pngfilter 0 grass_BaseColor.png
ffmpeg -i grass_Normal.jpg -pngfilter 0 grass_Normal.png

# Verify files created
ls -lh grass_BaseColor.png grass_Normal.png

# Optional: Check if original JPG is still there (can delete after confirming PNG works)
ls -lh grass_BaseColor.jpg
```

**Success criteria:**
- ✅ `grass_BaseColor.png` exists in `assets/textures/grass/`
- ✅ File size >100KB (not corrupted)
- ✅ No errors from ffmpeg

---

### STEP 1: Add Dependency

```bash
cargo add bevy_asset_loader
```

---

### STEP 2: Create Asset Collections

**File:** `src/assets.rs` (NEW FILE)

```rust
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/grass/grass_BaseColor.png")]
    pub grass_color: Handle<Image>,

    #[asset(path = "textures/grass/grass_Normal.png")]
    pub grass_normal: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct SoundAssets {
    #[asset(path = "sounds/engine_loop.ogg")]
    pub engine_sound: Handle<AudioSource>,

    #[asset(path = "sounds/wind.ogg")]
    pub wind_sound: Handle<AudioSource>,

    #[asset(path = "sounds/crash.ogg")]
    pub crash_sound: Handle<AudioSource>,

    #[asset(path = "sounds/missile_fire.ogg")]
    pub missile_fire_sound: Handle<AudioSource>,

    // Add other sounds as needed
    // #[asset(path = "sounds/explosion.ogg")]
    // pub explosion_sound: Handle<AudioSource>,
}
```

---

### STEP 3: Define GameState Enum

**File:** `src/main.rs` (add near top, after imports, before main function)

```rust
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
```

---

### STEP 4: Initialize LoadingState in App

**File:** `src/main.rs` (in main function)

Find:
```rust
let mut app = App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        // ...
    }))
```

Add IMMEDIATELY AFTER `.add_plugins()`:

```rust
    .init_state::<GameState>()
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Playing)
            .load_collection::<TextureAssets>()
            .load_collection::<SoundAssets>()
    )
```

Add module declaration at top of main.rs:
```rust
mod assets;
```

---

### STEP 5: Move ALL Startup Systems to OnEnter(GameState::Playing)

**Critical:** Find ALL `.add_systems(Startup, ...)` and change to `.add_systems(OnEnter(GameState::Playing), ...)`

Systems to move (search for these in main.rs):
- `setup_scene` ← Most important
- `spawn_player`
- `spawn_turrets`
- `spawn_objectives`
- `spawn_realistic_clouds`
- `spawn_afterburner_particles` (if it's Startup)
- Any other gameplay initialization

**Before:**
```rust
.add_systems(Startup, (
    setup_scene,
    spawn_player,
    spawn_realistic_clouds,
))
```

**After:**
```rust
.add_systems(OnEnter(GameState::Playing), (
    setup_scene,
    spawn_player,
    spawn_realistic_clouds,
))
```

**Why:** These systems need TextureAssets and SoundAssets to be ready. Waiting for GameState::Playing ensures assets are loaded first.

---

### STEP 6: Update setup_scene() to Use Assets

**File:** `src/main.rs`

**Update function signature:**

```rust
// OLD
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

// NEW - Add these two parameters:
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_assets: Res<TextureAssets>,  // ADD THIS
    sound_assets: Res<SoundAssets>,      // ADD THIS
) {
```

**Replace procedural texture code:**

```rust
// OLD (remove this):
// let grass_image = procedural_textures::create_grass_texture();
// let grass_texture_handle = images.add(grass_image);

// NEW (add this):
let grass_texture_handle = texture_assets.grass_color.clone();
```

**Create material with loaded texture:**

```rust
let ground_material_handle = materials.add(StandardMaterial {
    base_color: Color::WHITE,
    base_color_texture: Some(grass_texture_handle),
    perceptual_roughness: 0.9,
    reflectance: 0.02,
    metallic: 0.0,
    ..default()
});
```

---

### STEP 7: Update Sound Loading

**Remove FromWorld SoundAssets resource creation** (search for it in main.rs)

Find something like:
```rust
impl FromWorld for SoundAssets {
    fn from_world(world: &mut World) -> Self {
        // Old sound loading code
    }
}
```

**Delete this entire impl block** - sounds now load via bevy_asset_loader.

**Update any code that uses SoundAssets:**

```rust
// OLD
let sound_assets = world.resource::<SoundAssets>();

// NEW
let sound_assets = sound_assets_res;  // Now passed as parameter
```

---

### STEP 8: Test Build & Run

```bash
cargo build --release 2>&1 | tail -20
cp -r assets target/release/
target/release/plane_game.exe
```

**Expected output:**
```
# Should see loading happen (fast, <2 seconds)
# Then game state transitions to Playing
# Then you see the game
```

---

## Testing Checklist

```
Pre-Build:
  [ ] PNG files exist in assets/textures/grass/
  [ ] grass_BaseColor.png file size >100KB
  [ ] grass_Normal.png file size >100KB

Compilation:
  [ ] cargo build --release succeeds
  [ ] No "Path not found" errors
  [ ] No "Could not find asset loader" errors

Runtime:
  [ ] Game launches
  [ ] Briefly shows Loading state
  [ ] Transitions to Playing
  [ ] setup_scene runs after assets loaded
  [ ] setup_player runs after assets loaded

Visual:
  [ ] Grass texture visible
  [ ] Looks similar to procedural version
  [ ] No black/placeholder areas
  [ ] Seamless tiling

Audio:
  [ ] Engine sound plays smoothly (no stutter on startup)
  [ ] Missile fire sound works
  [ ] Crash sound works
  [ ] No audio errors in console

Performance:
  [ ] FPS stable at 60+
  [ ] Load time <2 seconds
  [ ] No stutters during asset load
```

---

## Troubleshooting

### Error: "Path not found: textures/grass/grass_BaseColor.png"
**Cause:** PNG file doesn't exist or wrong path
**Fix:**
```bash
# Verify file exists
ls -la target/release/assets/textures/grass/grass_BaseColor.png

# If missing, convert again
cd target/release/assets/textures/grass/
ffmpeg -i grass_BaseColor.jpg -pngfilter 0 grass_BaseColor.png
```

### Error: "Could not find an asset loader"
**Cause:** bevy_asset_loader not added or GameState not initialized
**Fix:**
```bash
cargo add bevy_asset_loader
# Verify .init_state::<GameState>() is in app builder
```

### Game runs but still shows procedural texture
**Cause:** Code still calling procedural function
**Fix:**
- Remove `let grass_image = procedural_textures::create_grass_texture();`
- Use `let grass_texture_handle = texture_assets.grass_color.clone();` instead

### Game panics during setup_scene
**Cause:** Resources not available (should be in OnEnter, not Startup)
**Fix:**
- Verify setup_scene is `.add_systems(OnEnter(GameState::Playing), setup_scene)`
- Not `.add_systems(Startup, setup_scene)`

### Audio stutters on first missile
**Cause:** SoundAssets still using FromWorld
**Fix:**
- Delete the old FromWorld impl block
- Verify SoundAssets parameter is in all systems that need it

---

## Commit Message

```
Feature: Professional asset loading system with bevy_asset_loader

Major Changes:
- Add bevy_asset_loader dependency for proper async asset management
- Create GameState enum (Loading → Playing)
- Implement TextureAssets and SoundAssets collections in src/assets.rs
- Initialize LoadingState configuration (loads all assets before game starts)
- Move ALL Startup systems to OnEnter(GameState::Playing):
  * setup_scene
  * spawn_player
  * spawn_realistic_clouds
  * spawn_turrets
  * spawn_objectives
  * And any other gameplay initialization
- Migrate sound loading from FromWorld to asset loader (fixes stutter)
- Convert grass textures from JPG to PNG (avoids metadata issues)
- Remove procedural texture generation (now loading from file)

Benefits:
- ✅ Professional asset pipeline (industry standard)
- ✅ No more Bevy #15081 async bind group bugs
- ✅ Audio loads properly (no stutter on first sound)
- ✅ Easy to extend (add more textures/audio/models)
- ✅ Ready for Phase 3 features
- ✅ Same visual quality as before (textures look identical)

Performance:
- Load time: <2 seconds
- Frame time: No per-frame overhead
- Memory: Same as procedural (4.19 MB for texture)
```

---

## Important Notes

### Why Move ALL Startup Systems?
If only setup_scene moves to Playing:
- ❌ spawn_player runs during Loading (might crash, assets not ready)
- ❌ spawn_clouds runs before textures loaded
- ❌ Timing bugs from systems running out of order

Moving all gameplay systems to Playing ensures:
- ✅ Everything runs AFTER assets loaded
- ✅ No race conditions
- ✅ Predictable initialization order

### Why Move Audio Now?
Current approach (FromWorld):
- ❌ Loads immediately in background
- ⚠️ First sound has stutter (loading finishes during playback)

bevy_asset_loader approach:
- ✅ Waits for all assets before game starts
- ✅ No stutter on first missile/crash sound
- ✅ Professional pipeline

### Why PNG Instead of JPG?
- JPG can have embedded EXIF metadata
- Bevy's image loader sometimes fails on this metadata
- PNG is uncompressed, no metadata issues
- Same visual quality, more reliable

---

## Success Criteria

When complete:
- ✅ All assets load before game starts
- ✅ No more procedural texture generation
- ✅ Grass texture loads from PNG file
- ✅ Audio loads without stutter
- ✅ All systems run in correct order
- ✅ Code is production-quality
- ✅ One clean commit
- ✅ Ready for Phase 3

---

## Questions for You

1. **Should we keep procedural texture as fallback?**
   - Recommend: No (asset loader is reliable enough)

2. **Any other sounds to add to SoundAssets?**
   - Check if there are explosion, damage, or other sounds

3. **Other textures to load?**
   - Recommend: Just grass for now, add more in Phase 3

---

## Timeline

- **PNG Conversion:** 5 minutes
- **Code implementation:** 1.5-2 hours
- **Testing:** 30 minutes
- **Total:** 2-2.5 hours

---

## Ready to Implement?

This is a thorough, bulletproof approach. All edge cases covered. Enjoy! 🚀
