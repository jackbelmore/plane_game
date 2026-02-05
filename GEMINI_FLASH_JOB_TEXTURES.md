# GEMINI FLASH JOB #3: Implement Realistic Ground Textures

**Priority:** High (visual improvement)
**Estimated Time:** 1-2 hours
**Blocker:** No (can work in parallel)
**Deliverable:** Textured ground with normal maps + rocks

---

## The Problem

**Current Ground:**
- Plain flat green color (boring)
- No texture detail
- Unrealistic appearance
- Doesn't match Kenney art style

**Goal:**
- Add grass texture with normal maps
- Make ground look bumpy/realistic
- Scatter rocks as obstacles
- Maintain physics collision

---

## Job Tasks (In Order)

### TASK 1: Download Textures from Poly Haven

**Website:** https://polyhaven.com/textures

1. **Search for grass textures:**
   - Go to polyhaven.com/textures
   - Search "grass"
   - Look for textures with tags: "PBR", "tileable"
   - Download recommended: "Ground Grass 001" or similar
   - Format: Download the **1K resolution** with all maps

2. **Download these texture sets:**
   - **Grass** (for main ground) - with Normal and Roughness maps
   - **Dirt/Earth** (for exposed areas) - same format
   - **Rock** (optional, for variety) - same format

3. **What to download:**
   Each texture should include:
   - `*_BaseColor.jpg` (color map)
   - `*_Normal.jpg` (normal map for bumps)
   - `*_Roughness.jpg` (roughness/shine map)

4. **Save location:**
   - Download ZIP file
   - Extract to: `C:\Users\Box\plane_game\assets\textures\`
   - Should have subdirectory: `assets/textures/ground_grass_001/` etc.

**Alternative if Poly Haven blocked:**
- Go to ambientcg.com
- Search "grass"
- Download same format files

---

### TASK 2: Fix Blue Cube Issue

In `src/main.rs`, find the HorizonDisk spawning code (around line 575):

```rust
// Current code shows Y = -2.0
Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
    .with_translation(Vec3::new(0.0, -2.0, 0.0)),
```

**Change Y value:**
```rust
// Move horizon disk further down (below visible area)
Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
    .with_translation(Vec3::new(0.0, -500.0, 0.0)),  // Much lower
```

**Why:** If blue cube is horizon disk, moving it down hides it below terrain.

Test this first - if blue cube disappears, that was the issue!

---

### TASK 3: Update spawn_chunk() to Use Textures

Find `spawn_chunk()` function in `src/main.rs` (around line 650-700).

**Current code probably looks like:**
```rust
let ground_material = materials.add(StandardMaterial {
    base_color: Color::srgb(0.25, 0.3, 0.25),  // Plain green
    ..default()
});
```

**Replace with:**
```rust
let ground_material = materials.add(StandardMaterial {
    base_color_texture: Some(asset_server.load("textures/ground_grass_001/ground_grass_001_BaseColor.jpg")),
    normal_map_texture: Some(asset_server.load("textures/ground_grass_001/ground_grass_001_Normal.jpg")),
    perceptual_roughness: 0.8,
    reflectance: 0.2,
    ..default()
});
```

**Important:**
- Adjust texture paths to match where you saved files
- Keep `perceptual_roughness: 0.8` (makes it look earthy)
- Change `reflectance: 0.2` (grass isn't shiny)

---

### TASK 4: Add Rocks as Props

Create a new function `spawn_rocks_in_chunk()`:

```rust
fn spawn_rocks_in_chunk(
    chunk_coord: ChunkCoordinate,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    // Deterministic seeding so rocks are same each time
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        ((chunk_coord.x as u64 * 73856093) ^ (chunk_coord.z as u64 * 19349663)) ^ 0x12345678,
    );

    // Spawn 2-4 rocks per chunk
    let rock_count = rng.gen_range(2..5);

    for _ in 0..rock_count {
        let x = rng.gen_range(0.0..CHUNK_SIZE);
        let z = rng.gen_range(0.0..CHUNK_SIZE);
        let y = 5.0;  // Height above ground

        let world_pos = chunk_coord.world_position();
        let pos = Vec3::new(
            world_pos.x + x - CHUNK_SIZE / 2.0,
            world_pos.y + y,
            world_pos.z + z - CHUNK_SIZE / 2.0,
        );

        // Spawn rock as cuboid obstacle
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid {
                half_size: Vec3::splat(10.0),  // 20m diameter rock
            }))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.5, 0.4),  // Gray-brown
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_translation(pos),
            Collider::cuboid(10.0, 10.0, 10.0),
            RigidBody::Static,
        ));
    }
}
```

**Add to spawn_chunk() function:**
In the same place where trees are spawned, add:
```rust
spawn_rocks_in_chunk(chunk_coord, &mut commands, &asset_server);
```

---

### TASK 5: Build & Test

```bash
# Build
cd C:\Users\Box\plane_game
cargo build --release

# Copy assets (CRITICAL - textures won't load without this)
cp -r assets target/release/

# Run
target/release/plane_game.exe
```

**What to look for:**
1. ✅ Ground has grass texture (not plain green)
2. ✅ Ground looks bumpy/detailed (normal map working)
3. ✅ No blue cube in ground (if you fixed horizon)
4. ✅ Rocks scattered around (visible obstacles)
5. ✅ Can fly and see texture details up close
6. ✅ FPS still 60+

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Textures don't load | Check file paths match exactly, ensure `cp -r assets target/release/` |
| Ground looks plain/white | Asset paths wrong or texture file missing |
| Normal maps don't work | Check `normal_map_texture` is spelled correctly |
| Blue cube still visible | Adjust Y position further (try -1000.0) |
| Rocks don't appear | Check `spawn_rocks_in_chunk()` is called in spawn_chunk() |
| Game crashes | Check asset server paths don't have typos |

---

## Code Locations to Modify

| Task | File | Line | What to Change |
|------|------|------|-----------------|
| Fix blue cube | src/main.rs | ~590 | HorizonDisk Y position |
| Texture material | src/main.rs | ~680 | StandardMaterial in spawn_chunk() |
| Add rocks | src/main.rs | ~700+ | Add spawn_rocks_in_chunk() call |

---

## Deliverables

When complete, provide:

1. ✅ **Confirm assets downloaded:**
   - Location: C:\Users\Box\plane_game\assets\textures\
   - Files present: *_BaseColor.jpg, *_Normal.jpg, *_Roughness.jpg

2. ✅ **Show code changes:**
   - Updated spawn_chunk() with texture material
   - New spawn_rocks_in_chunk() function
   - HorizonDisk Y position change

3. ✅ **Test results:**
   - Did ground texture appear?
   - Did rocks spawn?
   - Did blue cube disappear?
   - FPS stable?

4. ✅ **Screenshot or description:**
   - What does ground look like now?
   - Texture quality satisfactory?
   - Rocks look good?

5. ✅ **Issues encountered:**
   - Any problems? How fixed?
   - Any adjustments needed?

6. ✅ **Next job recommendation:**
   - What should Claude/Gemini do next?

---

## Success Criteria

✅ Ground has visible grass texture
✅ Normal map adds bumpy appearance
✅ Rocks scatter throughout world
✅ No blue cube visible
✅ Game compiles and runs
✅ FPS 60+
✅ Trees still rendering correctly
✅ Drones still moving

---

**Time Estimate:** 1-2 hours (most time spent downloading/copying assets)
**Complexity:** Medium (straightforward texture substitution + one new function)
**Risk:** Low (doesn't affect core systems, easy to rollback)

