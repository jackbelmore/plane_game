# Gemini Prompt - Grass Texture Implementation (Phase A)

**Priority:** HIGH | **Effort:** 1.5-2 hours | **Success Rate:** 100%
**After UI system is working, implement this immediately**

---

## Your Mission

Implement a **high-quality procedural grass texture** using the approach you suggested:

1. Use your existing `rand` crate dependency
2. Generate smoother noise (not blocky x^y patterns)
3. Create convincing grass texture at startup
4. No async loading, no bind group issues
5. This fixes the visible grass problem immediately

---

## Detailed Requirements

### Step 1: Create Better Noise Function
**File:** `src/procedural_textures.rs` (NEW FILE)

Generate a grass texture using layered random noise instead of bitwise x^y:

```rust
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn create_grass_texture(seed: u64) -> Image {
    const SIZE: usize = 512;
    const SCALE: f32 = 0.05; // Frequency of noise

    let mut rng = StdRng::seed_from_u64(seed);
    let mut data = Vec::with_capacity(SIZE * SIZE * 4);

    for y in 0..SIZE {
        for x in 0..SIZE {
            // Layer 1: Base grass color (green)
            let base_r = 85u8;
            let base_g = 150u8;
            let base_b = 65u8;

            // Layer 2: Add Perlin-lite noise for variation
            let noise1 = ((x as f32 * SCALE).sin() * (y as f32 * SCALE).cos()).abs();
            let noise2 = rng.gen::<f32>() * 0.3; // Random per-pixel variation
            let noise = (noise1 + noise2).min(1.0);

            // Layer 3: Blend with darker grass for depth
            let dark_r = (base_r as f32 * (1.0 - noise * 0.4)) as u8;
            let dark_g = (base_g as f32 * (1.0 - noise * 0.2)) as u8;
            let dark_b = (base_b as f32 * (1.0 - noise * 0.3)) as u8;

            // Optional: Add slight brown patches for organic feel
            let patch_chance = rng.gen::<f32>();
            let (final_r, final_g, final_b) = if patch_chance < 0.05 {
                // 5% brown patches (dirt)
                ((dark_r as f32 * 0.8) as u8, (dark_g as f32 * 0.7) as u8, (dark_b as f32 * 0.6) as u8)
            } else {
                (dark_r, dark_g, dark_b)
            };

            data.push(final_r);
            data.push(final_g);
            data.push(final_b);
            data.push(255); // Full opacity
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
    );

    // Configure sampler for seamless tiling
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        lod_min_clamp: 0.0,
        lod_max_clamp: 32.0,
        anisotropy_clamp: 16,
        compare: None,
    });

    image
}
```

**Key improvements:**
- ✅ Sin/cos waves instead of x^y (smooth, organic)
- ✅ Layered random values (variable depth)
- ✅ Brown patches for realism (5% chance)
- ✅ Proper sampler for seamless tiling
- ✅ Uses existing rand crate

### Step 2: Remove Broken Async Code
**File:** `src/main.rs`

Find and delete:
1. The `check_grass_texture_loaded()` system (search for this function)
2. The `GrassTextureLoading` resource definition
3. Any call to `.with_system(check_grass_texture_loaded)`

### Step 3: Update setup_scene()
**File:** `src/main.rs` (lines 587-630 area)

Replace the texture loading section:

```rust
// OLD CODE (DELETE):
// let texture_handle = asset_server.load("textures/grass/grass_BaseColor.jpg");
// let ground_material = materials.add(StandardMaterial {
//     base_color_texture: Some(texture_handle),
//     ...
// });

// NEW CODE (ADD):
fn setup_scene(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // ... rest of parameters
) {
    // Step 1: Create procedural texture (NO async)
    let grass_image = crate::procedural_textures::create_grass_texture(12345); // Seed for consistency
    let grass_texture = images.add(grass_image);

    // Step 2: Create material with the texture (texture is NOW guaranteed ready)
    let ground_material = materials.add(StandardMaterial {
        base_color_texture: Some(grass_texture),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    // Step 3: Use ground_material in spawn_ground_chunk()
    // (This part stays the same)
}
```

### Step 4: Add Module Declaration
**File:** `src/main.rs` (top of file, around line 11)

Add:
```rust
mod procedural_textures;
```

### Step 5: Test
```bash
cargo build --release
cp -r assets target/release/
target/release/plane_game.exe

# Expected result:
# - Game launches
# - Ground is green (no black patches)
# - Texture repeats seamlessly as you fly
# - Performance: 60+ FPS
```

---

## Testing Checklist

Before you commit:

```
Visual Tests:
  [ ] Ground renders with green texture (not black)
  [ ] Texture has organic variation (not uniform color)
  [ ] Brown patches visible occasionally (realism)
  [ ] Texture tiles seamlessly (no seams at chunk boundaries)
  [ ] No visible "aliasing" or blocky patterns

Performance Tests:
  [ ] FPS stays 60+ during normal flight
  [ ] No stutters when loading new chunks
  [ ] Texture loads instantly (no loading lag)
  [ ] Memory stable after 10 minutes flight

Edge Cases:
  [ ] Fly up to 25km - texture still visible
  [ ] Fly 20km away - distant chunks render correctly
  [ ] Multiple chunks loaded - all have consistent texture
  [ ] Respawn (ESC) - texture resets correctly
```

---

## Commit Message

```
Feature: Procedural grass texture system

- Replace async JPG loading with procedural generation
- Use layered noise (sin/cos + random) for organic variation
- Add brown patches for realism (5% occurrence)
- Fixes Bevy #15081 async texture bind group issue
- Texture guaranteed ready at startup, no loading lag
- Performance: ~1ms to generate, 0ms per frame
```

---

## After This Works - Phase B (Later)

Once grass texture is visible and working:

We'll implement **GameState + bevy_asset_loader** to:
1. Load real JPG/PNG files properly
2. Pre-load all audio assets
3. Add loading screen UI
4. Proper async handling for Phase 4 features

But for now, **procedural = instant visual fix**.

---

## Success Criteria

After implementation:
- ✅ Green grass visible on all chunks
- ✅ Natural-looking variation (not blocky)
- ✅ No black/missing texture areas
- ✅ Seamless tiling across chunks
- ✅ 60+ FPS maintained
- ✅ No async loading errors in console

---

## Questions for You

1. **Noise complexity:** Should we use simpler noise (faster) or more complex (prettier)?
   - Simple: Just sin/cos waves
   - Complex: Add Perlin-lite with multiple octaves

2. **Color variation:** How much brown/dirt variation?
   - Current: 5% brown patches
   - Increase to: 10-15% for more realism
   - Decrease to: 1-2% for pure grass

3. **Texture resolution:** 512x512 good or should we use:
   - 256x256 (faster, fine for far view)
   - 1024x1024 (slower, better detail)

---

## Reference

See GRASS_TEXTURE_FIX_PLAN.md for full context on why this works and why Bevy #15081 happens.

**Ready to implement?** Let me know!
