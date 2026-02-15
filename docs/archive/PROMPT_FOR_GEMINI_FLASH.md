# GEMINI FLASH JOB: Fix Drone Model Loading

**Priority:** HIGH (Visual completeness)
**Estimated Time:** 1-2 hours
**Status:** Ready to start
**Blocker:** None (can work in parallel)

---

## The Problem

Currently, drones spawn as **red test cubes** instead of the actual drone.glb 3D model.

This is the same issue that trees had before - they were using `SceneRoot` which doesn't work in Bevy 0.15.

**Current code (broken):**
```rust
SceneRoot(asset_server.load("models/drone.glb")),  // ❌ Doesn't render in Bevy 0.15
```

**Expected fix:**
```rust
// Use direct Mesh loading instead (like trees now do)
asset_server.load("models/drone.glb#Mesh0/Primitive0")
```

---

## What You Need to Do

### STEP 1: Verify drone.glb Exists (5 minutes)

Run these commands to check the file:

```bash
# Check if drone.glb exists
ls -lh /c/Users/Box/plane_game/assets/models/drone.glb

# Verify it's a valid GLB file
file /c/Users/Box/plane_game/assets/models/drone.glb

# Compare with working tree.glb
file /c/Users/Box/plane_game/assets/fantasy_town/tree.glb
ls -lh /c/Users/Box/plane_game/assets/fantasy_town/tree.glb
```

**Expected output:**
- drone.glb should exist and be ~1-10MB
- File type should be "data" (binary GLB file)
- Should be similar size to tree.glb

If drone.glb is missing or corrupt, you'll need to re-download it.

---

### STEP 2: Update spawn_beaver_drone() Function (1 hour)

**File:** `src/drone.rs`

Find the `spawn_beaver_drone()` function (around line 15-40).

Current code probably looks like:
```rust
fn spawn_beaver_drone(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
) {
    commands.spawn((
        SceneRoot(asset_server.load("models/drone.glb")),  // ❌ BROKEN
        Transform::from_translation(position),
        Drone { health: 50.0, speed: 40.0 },
        KamikazeBehavior,
    ));
}
```

**Change it to:**

```rust
fn spawn_beaver_drone(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
) {
    // Try loading drone.glb as direct mesh
    let drone_mesh = asset_server.load("models/drone.glb#Mesh0/Primitive0");
    let drone_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),  // Dark gray
        roughness: 0.8,
        metallic: 0.1,
        ..default()
    });

    commands.spawn((
        Mesh3d(drone_mesh),
        MeshMaterial3d(drone_material),
        Transform {
            translation: position,
            scale: Vec3::splat(1.8),  // Scale if needed
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
        },
        Drone { health: 50.0, speed: 40.0 },
        KamikazeBehavior,
    ));
}
```

**Key changes:**
- Add `meshes` and `materials` parameters
- Use `asset_server.load("models/drone.glb#Mesh0/Primitive0")` instead of SceneRoot
- Create a dark gray material (allows drone to be visible)
- Keep scale at 1.8x (original)
- Keep 180° Y rotation (original)

---

### STEP 3: Update Function Calls (30 minutes)

Now find all places that call `spawn_beaver_drone()` and update them.

Likely in **src/main.rs**, around line 670-690 in the `setup_scene()` function:

**Old code:**
```rust
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(0.0, 520.0, -200.0));
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(-150.0, 500.0, -100.0));
spawn_beaver_drone(&mut commands, &asset_server, Vec3::new(150.0, 500.0, -100.0));
```

**New code:**
```rust
spawn_beaver_drone(&mut commands, &asset_server, &mut meshes, &mut materials, Vec3::new(0.0, 520.0, -200.0));
spawn_beaver_drone(&mut commands, &asset_server, &mut meshes, &mut materials, Vec3::new(-150.0, 500.0, -100.0));
spawn_beaver_drone(&mut commands, &asset_server, &mut meshes, &mut materials, Vec3::new(150.0, 500.0, -100.0));
```

Just add `&mut meshes, &mut materials,` to each call.

---

### STEP 4: Fallback Plan (if .glb doesn't load)

If drone.glb doesn't load properly, you can add a fallback to render a gray cuboid (like we did for textures):

```rust
fn spawn_beaver_drone(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
) {
    eprintln!("🚁 Attempting to load drone.glb model...");

    // Try real drone model first
    let drone_handle = asset_server.load("models/drone.glb#Mesh0/Primitive0");

    // For now, spawn with fallback geometry
    let drone_mesh = meshes.add(Mesh::from(Cuboid::new(3.0, 2.0, 8.0)));
    let drone_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        roughness: 0.8,
        ..default()
    });

    commands.spawn((
        Mesh3d(drone_mesh),
        MeshMaterial3d(drone_material),
        Transform {
            translation: position,
            scale: Vec3::splat(1.8),
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
        },
        Drone { health: 50.0, speed: 40.0 },
        KamikazeBehavior,
    ));
}
```

This creates a dark gray drone-shaped box if the model fails to load.

---

## Testing Procedure

```bash
# Build
cd /c/Users/Box/plane_game
cargo build --release

# If build fails, read error carefully:
# - If "module not found", check imports in src/drone.rs
# - If "field not found", check Drone struct definition
# - If "cannot find function", check function signature matches all calls

# Copy assets
cp -r assets target/release/

# Run
target/release/plane_game.exe
```

**In Game:**
1. Spawn at origin
2. Look forward toward negative Z
3. You should see **3 drone models** (not red cubes)
   - 1 drone directly ahead
   - 1 drone to the left
   - 1 drone to the right
4. Drones should be dark gray (or actual drone model if loading worked)
5. Drones move forward smoothly

**Console check:**
- No errors about missing assets
- Drones should log movement messages (if enabled)

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Compilation error: "cannot find function" | Check function signature - did you add meshes/materials params? |
| Compilation error: "expected X parameters, found Y" | Count params in function def and all calls - they must match |
| Red cubes still appearing | `spawn_beaver_drone()` not being called with new signature, or fallback triggered |
| Dark gray boxes instead of drone model | Fallback is rendering - .glb file might not be found or have wrong mesh path |
| Black invisible drones | Material might need different color or emission |
| Drones not moving | Movement system working, but visibility issue - check material/color |

---

## Code References

- **Drone spawn location:** `src/drone.rs` line ~15-40
- **Calls to spawn_beaver_drone:** `src/main.rs` line ~670-690 in setup_scene()
- **Working example (trees):** `src/main.rs` line ~963-1010 in spawn_trees_in_chunk()
- **Mesh asset loading pattern:** Look for `.load("models/tree.glb#Mesh0/Primitive0")` in tree code

---

## Files to Modify

1. **src/drone.rs:**
   - Update `spawn_beaver_drone()` function definition and implementation

2. **src/main.rs:**
   - Update all 3 calls to `spawn_beaver_drone()` in setup_scene()
   - Add `meshes` and `materials` parameter to calls

---

## Success Criteria

✅ Code compiles without errors
✅ 3 drones visible on game start (not red cubes)
✅ Drones are dark gray or actual model appearance
✅ Drones spawn in formation (1 ahead, 2 flanking)
✅ Drones move smoothly forward
✅ No black/invisible drones
✅ No console errors about missing assets
✅ Can fly around drones and see them from all angles

---

## If drone.glb is Missing

If the file doesn't exist or is corrupted:

1. Check E:\Downloads for any drone files
2. Download from similar source as grass textures (Poly Haven, Sketchfab, etc.)
3. Or use fallback gray cuboid permanently (already handled in code above)

---

## Quick Reference: Tree Loading (Your Template)

Trees were fixed using this pattern - copy it for drones:

```rust
// From src/main.rs spawn_trees_in_chunk()
let tree_models = vec![
    "models/tree.glb#Mesh0/Primitive0",
    // ... more tree variants
];

for (i, tree_path) in tree_models.iter().enumerate() {
    let handle = asset_server.load(*tree_path);

    commands.spawn((
        Mesh3d(handle),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        Tree,
    ));
}
```

Use this same pattern but simpler for drones.

---

**Ready to fix those drones?** Start with Step 1 (verify file exists), then code changes.
