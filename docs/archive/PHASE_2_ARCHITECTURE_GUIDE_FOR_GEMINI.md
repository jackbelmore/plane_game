# Phase 2 Architecture Change Guide - Audio Integration

**For:** Gemini (working on Dynamic Impact System audio)
**Date:** 2026-02-06
**Status:** Phase 2 COMPLETE - This explains why SoundAssets no longer exists

---

## The Change: SoundAssets → GameAssets (Unified Loading)

### Before Phase 2 (What You May Remember)
```rust
// In src/main.rs - OLD PATTERN
struct SoundAssets {
    engine_sound: Handle<AudioSource>,
    missile_launch: Handle<AudioSource>,
    // ... other audio ...
}

impl FromWorld for SoundAssets {
    fn from_world(world: &mut World) -> Self {
        // Manual async loading - race conditions possible
        let asset_server = world.resource::<AssetServer>();
        SoundAssets {
            engine_sound: asset_server.load("sounds/engine.ogg"),
            // ...
        }
    }
}

// In main():
.init_resource::<SoundAssets>()
```

**Problem:** Audio loaded asynchronously during Startup. Could cause:
- First sound stutter (if not loaded yet)
- Race conditions between systems
- Unpredictable timing

### After Phase 2 (Current Architecture)
```rust
// In src/assets.rs - NEW PATTERN (UNIFIED)
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // --- Textures ---
    #[asset(path = "textures/grass/grass_BaseColor.png")]
    pub grass_texture: Handle<Image>,

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

**In src/main.rs:**
```rust
// NEW: State machine approach
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,  // Load ALL assets here
    Playing,  // Game runs here (assets guaranteed loaded)
}

// In main():
.init_state::<GameState>()
.add_loading_state(
    LoadingState::new(GameState::Loading)
        .continue_to_state(GameState::Playing)
        .load_collection::<GameAssets>()
)

// ALL gameplay systems moved to:
.add_systems(OnEnter(GameState::Playing), (
    setup_scene,
    spawn_realistic_clouds,
    spawn_objectives,
    spawn_turrets,
    spawn_player,
))
```

**Advantage:**
- ✅ All assets (textures + audio) load together
- ✅ Game only starts when EVERYTHING is ready
- ✅ No stutter, no race conditions
- ✅ Professional architecture

---

## How to Add Your New Explosion Sounds

### Step 1: Add Fields to GameAssets (src/assets.rs)

**Current GameAssets struct:**
```rust
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // ... existing fields ...

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

**Add your new explosions:**
```rust
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // ... existing fields ...

    #[asset(path = "sounds/explosion.ogg")]
    pub explosion: Handle<AudioSource>,

    // NEW: Add these lines
    #[asset(path = "sounds/explosion_standard.ogg")]
    pub explosion_standard: Handle<AudioSource>,

    #[asset(path = "sounds/explosion_heavy.ogg")]
    pub explosion_heavy: Handle<AudioSource>,

    #[asset(path = "sounds/warning.ogg")]
    pub warning: Handle<AudioSource>,

    // ... other fields ...
}
```

### Step 2: Ensure Audio Files Exist

Your audio files must be in the asset directory:
```
assets/
├── sounds/
│   ├── engine.ogg
│   ├── missile.ogg
│   ├── explosion.ogg
│   ├── explosion_standard.ogg      <- NEW
│   ├── explosion_heavy.ogg         <- NEW
│   ├── warning.ogg
│   ├── crash.ogg
│   └── wind.ogg
└── textures/
    └── grass/
        └── grass_BaseColor.png
```

### Step 3: Update Systems That Play Explosions

**Old pattern (pre-Phase 2):**
```rust
fn play_explosion_sound(
    sound_assets: Res<SoundAssets>,  // ← This no longer exists!
    audio: Res<Audio>,
) {
    audio.play(sound_assets.explosion.clone());
}
```

**New pattern (Phase 2):**
```rust
fn play_explosion_sound(
    game_assets: Res<GameAssets>,  // ← Use unified GameAssets instead
    audio: Res<Audio>,
) {
    // Pick random explosion type
    let is_heavy = rand::random::<f32>() < 0.2;  // 20% chance
    let sound = if is_heavy {
        game_assets.explosion_heavy.clone()
    } else {
        game_assets.explosion_standard.clone()
    };

    audio.play(sound);
}
```

### Step 4: Example - Dynamic Impact System

```rust
fn play_impact_explosion(
    game_assets: Res<GameAssets>,
    audio: Res<Audio>,
    impact_query: Query<&ImpactEvent, Changed<ImpactEvent>>,
) {
    for impact in impact_query.iter() {
        // Pick explosion based on impact type
        let sound = match impact.target_type {
            TargetType::Drone => {
                // Drones: 70% light, 30% heavy
                if rand::random::<f32>() < 0.7 {
                    game_assets.explosion_standard.clone()
                } else {
                    game_assets.explosion_heavy.clone()
                }
            }
            TargetType::Building => {
                // Buildings: Always heavy
                game_assets.explosion_heavy.clone()
            }
            _ => game_assets.explosion.clone(),
        };

        audio.play(sound);
    }
}

// Add to main() in OnEnter(GameState::Playing):
.add_systems(OnEnter(GameState::Playing), (
    setup_scene,
    spawn_realistic_clouds,
    spawn_objectives,
    spawn_turrets,
    spawn_player,
    play_impact_explosion,  // <- NEW: Your explosion system
))
```

---

## Important Notes

### ✅ What Changed
- `struct SoundAssets` no longer exists as separate resource
- All assets (textures + audio) unified under `GameAssets` in `src/assets.rs`
- Systems can only run in `OnEnter(GameState::Playing)` or later (guaranteed assets loaded)

### ✅ What Stays the Same
- Audio playback logic unchanged (still use `audio.play()`)
- File formats unchanged (OGG files)
- Audio quality unchanged
- Physics/collision unchanged

### ✅ Why This is Better
- No more "first sound stutter" (all audio ready before gameplay)
- Cleaner code (single GameAssets struct vs multiple resource init)
- Easier to add new assets (just add `#[asset(path = "...")]` field)
- Professional architecture (matches industry standards)

---

## Common Mistakes to Avoid

❌ **DON'T** try to find SoundAssets struct
```rust
// This won't compile - it doesn't exist anymore!
fn my_system(sound_assets: Res<SoundAssets>) { }
```

✅ **DO** use GameAssets instead
```rust
// This is correct
fn my_system(game_assets: Res<GameAssets>) {
    audio.play(game_assets.explosion_standard.clone());
}
```

---

❌ **DON'T** add systems to Startup schedule
```rust
// Wrong - assets might not be loaded yet!
.add_systems(Startup, play_explosion_sound)
```

✅ **DO** add to OnEnter(GameState::Playing) or Update
```rust
// Correct - runs after GameState::Playing entered
.add_systems(OnEnter(GameState::Playing), play_explosion_sound)

// Also correct - runs every frame while Playing
.add_systems(Update, play_explosion_sound)
```

---

## Summary

**For your Dynamic Impact System:**

1. ✅ Add `explosion_standard` and `explosion_heavy` fields to GameAssets struct
2. ✅ Place audio files in `assets/sounds/`
3. ✅ Update explosion playback systems to use `game_assets` instead of `sound_assets`
4. ✅ Keep systems in `Update` or `OnEnter(GameState::Playing)` schedule
5. ✅ Done! No other changes needed

The architecture change is complete and working perfectly. Just follow this pattern and you're good to go!

---

**Questions?** See PHASE_2_COMPLETION_REPORT.md or CODE_REVIEW_PHASE_2_IMPLEMENTATION.md for full details.
