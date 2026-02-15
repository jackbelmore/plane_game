# Asset Loading Fix Plan

**Objective:** Replace green cube placeholders with actual .glb tree models
**Status:** Ready to implement
**Estimated Time:** 1-2 hours
**Blocker Level:** CRITICAL (main visual issue)

---

## Current State

### What Works
- ✅ Chunks spawning and loading correctly
- ✅ Green cube Cuboid meshes render perfectly (proof rendering pipeline works)
- ✅ Ambient lighting system displays all objects clearly
- ✅ Physics/camera/fog all working

### What's Broken
- ❌ Trees render as invisible green cubes (spawning but not visible)
- ❌ Buildings/villages not rendering
- ❌ No actual 3D models visible (only test geometry)

### Root Cause
**Bevy 0.15's SceneRoot component silently fails for simple .glb files.**
- SceneRoot doesn't properly load scene children from .glb
- Files spawn but children entities don't become visible
- Previous attempts: `asset_server.load("fantasy_town/wall.glb#Scene0")` returns entity, but no mesh appears

---

## Solution: Direct Mesh Loading

Instead of using SceneRoot, load Mesh directly from glTF and apply StandardMaterial:

```rust
// OLD (Broken in Bevy 0.15):
commands.spawn((
    SceneRoot(asset_server.load("fantasy_town/tree.glb#Scene0")),
    Transform::from_translation(pos),
    ...
));

// NEW (Direct Mesh Loading):
let mesh_handle = meshes.add(Mesh::from(Box::default())); // Placeholder
let material_handle = materials.add(StandardMaterial {
    base_color_texture: Some(asset_server.load("fantasy_town/tree.glb")),
    ..default()
});

commands.spawn((
    Mesh3d(mesh_handle),
    MeshMaterial3d(material_handle),
    Transform::from_translation(pos),
    ...
));
```

---

## Implementation Plan (4 Steps)

### Step 1: Understand Current Tree Spawning Code
**File:** `src/main.rs` lines 803-870 (`spawn_trees_in_chunk` function)

Current code structure:
```rust
fn spawn_trees_in_chunk(chunk_coord: ChunkCoordinate, commands: &mut Commands, asset_server: &AssetServer) {
    let tree_models = vec![
        "fantasy_town/wall.glb",      // Current: using wall.glb as test
        // ... other models
    ];

    for i in 0..tree_count {
        let tree_model = tree_models[rng.gen_range(0..tree_models.len())];

        commands.spawn((
            SceneRoot(asset_server.load(tree_model)),  // ❌ BROKEN LINE
            Transform::from_xyz(local_x, local_y, local_z),
            ...
        ));
    }
}
```

### Step 2: Identify Asset Files
**Location:** `assets/fantasy_town/` (167 .glb files available)

Available tree models:
- `tree.glb` - Basic tree
- `tree-crooked.glb` - Twisted tree
- `tree-high.glb` - Tall tree
- `tree-high-crooked.glb` - Tall twisted tree
- `tree-high-round.glb` - Round canopy tree

**Strategy:** Start with `tree.glb` (basic model), verify it renders, then test others

### Step 3: Load Mesh + Material Directly

**Option A: Simple Cuboid Replacement (Proven Working)**
- Keep using Cuboid geometry but change color
- Confirms if asset system is the issue
- Takes 5 minutes

**Option B: Load glTF Mesh from File**
- Use `asset_server.load_folder("fantasy_town/")` to preload all meshes
- Spawn matching Mesh3d with StandardMaterial
- More complex but gives real trees
- Takes 1-2 hours

### Step 4: Test and Verify

```bash
# Build
cd /c/Users/Box/plane_game
cargo build --release

# Copy assets (CRITICAL - game loads from target/release/)
cp -r assets target/release/

# Run
target/release/plane_game.exe

# Expected result:
# - Fly around
# - See actual tree models (not green cubes)
# - Check console for any load errors
# - Verify FPS stays 60+
```

---

## Technical Approach (Recommended)

### Phase 1: Verify Asset File Format
Before loading, confirm glTF structure:
```bash
# Check if tree.glb contains mesh data
file assets/fantasy_town/tree.glb
# Should show: "tree.glb: glTF binary format with embedded data"
```

### Phase 2: Load Mesh via Asset Server
```rust
fn spawn_trees_in_chunk(
    chunk_coord: ChunkCoordinate,
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let tree_models = vec![
        "fantasy_town/tree.glb",
        "fantasy_town/tree-crooked.glb",
    ];

    for position in tree_positions {
        let tree_model = tree_models[...];  // Random selection

        // Load the mesh from glTF file
        // Note: Bevy 0.15 requires special handling for .glb files
        // This might require using GltfLoader directly

        commands.spawn((
            Mesh3d(loaded_mesh_handle),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.8, 0.5),
                ..default()
            })),
            Transform::from_translation(position),
            Tree,
            LODLevel(0),
        ));
    }
}
```

---

## Known Issues & Workarounds

| Issue | Workaround |
|-------|-----------|
| glTF loader async delays | Use asset_server.load() which returns handle immediately |
| Mesh scale mismatch | Apply scale factor in Transform::scale() |
| Material color vs texture | Use base_color for temporary solid colors |
| Asset not found at runtime | Ensure `cp -r assets target/release/` after build |

---

## Fallback Plan (If glTF Direct Loading Fails)

If direct mesh loading doesn't work in Bevy 0.15:

**Option 1: Keep Green Cubes, Increase Scale**
- Scale Cuboid to 5-10x to make them more visible
- Quick visual improvement while investigating further
- Time: 5 minutes

**Option 2: Use Placeholder Meshes**
- Create UV-mapped plane with tree texture baked in
- Simple 2D billboards facing camera
- Time: 30 minutes

**Option 3: Wait for Bevy 0.16**
- If SceneRoot is fixed in next version
- Time: N/A (external dependency)

---

## Success Criteria

✅ Game launches without crash
✅ Flying around shows real tree models (not green cubes)
✅ Multiple tree variants visible (different shapes/sizes)
✅ FPS maintains 60+
✅ No console errors about missing assets
✅ Trees respond to lighting (see shading/shadows)
✅ Can see through to horizon (trees have proper transparency)

---

## Code Locations

| Task | File | Lines |
|------|------|-------|
| Spawn trees | src/main.rs | 803-870 |
| Asset loading | src/main.rs | 1-50 (imports) |
| Tree component | src/main.rs | 118-120 |
| Lighting setup | src/main.rs | 500-600 |

---

## Related Documentation

- **CLAUDE.md:** Complete architecture + known issues
- **FIXES_APPLIED.md:** Emergency patches applied in previous session
- **PHASE3_IMPLEMENTATION_PROMPTS.md:** Combat system (can start once trees render)

---

**Ready to Implement:** Yes
**Block New Work:** No (Phase 3 can be planned in parallel)
**Revert Point:** Git commit `bc8c479` (before this plan)

