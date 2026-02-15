# Realistic Terrain & Texture Implementation Plan

**Objective:** Replace plain green ground with realistic terrain (grass, rocks, mountains, bumpy surfaces with physics)
**Priority:** Visual improvement + collision realism
**Estimated Effort:** 4-8 hours depending on approach

---

## Current Problem

**Ground Appearance:**
- Flat green planes (boring, unrealistic)
- No texture detail
- No variation across map
- No rocks/obstacles
- No elevation changes

**Also Notice:**
- Blue cube rendering in ground (unknown source - possible horizon disk clipping or debug geometry)
- Trees appear small (may need scale adjustment - currently showing actual models)

---

## Solution Approaches (3 Options)

### Option A: Texture + Normal Maps on Current Ground Planes (EASIEST)
- Keep chunk system as-is
- Replace flat StandardMaterial with PBR material
- Add normal maps for bumpy appearance
- Add rocks/boulders as separate entities
- **Time:** 2-3 hours
- **Physics:** Still flat AABB, no terrain collision
- **Visual Quality:** 7/10 (looks better but still fundamentally flat)

### Option B: Heightmap-Based Terrain (MEDIUM COMPLEXITY)
- Generate heightmaps per chunk (Perlin noise or loaded from file)
- Create varied terrain mesh per chunk (mountains, valleys)
- Apply realistic textures
- Complex collision meshes for terrain
- **Time:** 4-6 hours
- **Physics:** Real terrain collision, plane crashes into mountains
- **Visual Quality:** 9/10 (realistic varied terrain)

### Option C: Pre-Built Terrain Assets + Stitching (COMPLEX)
- Download terrain models from asset stores
- Stitch together seamlessly across chunks
- Blend textures at boundaries
- Full physics collision
- **Time:** 6-8 hours
- **Physics:** Excellent collision realism
- **Visual Quality:** 10/10 (professional quality)

---

## Recommended Approach: HYBRID (Option A + Partial B)

**Phase 1 (2 hours):** Textures + Materials
- Add grass texture to ground planes
- Add normal/roughness maps for bumpy appearance
- Scatter rocks as props

**Phase 2 (3-4 hours):** Terrain Variation
- Add Perlin noise heightmap per chunk
- Create varied terrain mesh per chunk
- Add collision mesh matching terrain

**Phase 3 (2-3 hours):** Final Polish
- Blend textures across chunk boundaries
- Add more detail (grass variations, scattered stones)
- Optimize physics performance

---

## Asset Sources (FREE & HIGH QUALITY)

### Terrain Textures
| Source | Type | Quality | License |
|--------|------|---------|---------|
| **Poly Haven** (polyhaven.com) | PBR Textures | Excellent | CC0 |
| **AmbientCG** (ambientcg.com) | PBR Materials | Excellent | CC0 |
| **OpenGameArt** | Game Textures | Good | Various |
| **Texturify** | Procedural Textures | Good | CC0 |

### Recommended Downloads
1. **Grass Textures** (search "grass" on Poly Haven)
   - Look for: Tileable, PBR format (BaseColor, Normal, Roughness)
   - Size: 1024×1024 minimum
   - Files: `*_BaseColor.jpg`, `*_Normal.jpg`, `*_Roughness.jpg`

2. **Rock Textures** (search "rock" on Poly Haven)
   - For terrain variation and obstacles
   - Same format as grass

3. **Dirt/Earth Textures** (search "dirt" or "soil")
   - For exposed terrain areas
   - Adds realism to mountainous regions

### 3D Models (Terrain Features)
- **Rocks/Boulders:** Poly Haven, Sketchfab, OpenGameArt
- **Mountains:** Procedurally generated (easier than manually created)
- **Trees:** Already have Kenney forest assets

---

## Implementation Architecture

### Current System
```rust
// Every chunk spawns flat ground plane
fn spawn_chunk(chunk_coord, commands) {
    commands.spawn((
        Mesh3d(ground_mesh),           // Flat plane
        MeshMaterial3d(green_color),   // Plain green
        Transform::from_xyz(...),
        Collider::cuboid(...),         // Flat AABB
    ));
}
```

### Option A Implementation (Texture + Materials)
```rust
fn spawn_chunk(chunk_coord, commands, materials) {
    // Use PBR material instead of solid color
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/grass_BaseColor.jpg")),
        normal_map_texture: Some(asset_server.load("textures/grass_Normal.jpg")),
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(material),  // ← PBR material instead of color
        Transform::from_xyz(...),
        Collider::cuboid(...),
    ));
}
```

### Option B Implementation (Heightmap Terrain)
```rust
fn spawn_chunk(chunk_coord, commands, materials) {
    // 1. Generate or load heightmap for this chunk
    let heightmap = generate_heightmap(chunk_coord);  // or load from file

    // 2. Create varied mesh from heightmap
    let mesh = create_terrain_mesh_from_heightmap(heightmap);

    // 3. Create detailed collision mesh
    let collider = create_terrain_collider(heightmap);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(terrain_material),
        Transform::from_xyz(...),
        collider,  // ← Complex shape matching terrain
    ));
}
```

---

## Quick Wins (Immediate Improvements)

### 1. Fix the Blue Cube Issue
- Unknown what's rendering blue in ground
- Likely: Horizon disk clipping into ground
- Fix: Check horizon disk Y position (-2.0) vs ground level
- Or: Adjust camera near clip plane

### 2. Scale Adjustment for Trees
- Trees rendering but very small
- Likely: tree.glb is small relative to 1000m chunk
- Fix: Add scale factor in spawn_trees_in_chunk() (multiply by 3-5x)

### 3. Add Grass Texture (Quick)
- Download one grass texture from Poly Haven
- Replace green color with BaseColor texture
- Takes 30 minutes
- Immediate visual improvement

---

## Step-by-Step Implementation Plan

### Phase 1: Quick Texture (30 min)
1. Download grass texture set from Poly Haven
2. Add to assets/textures/
3. Update spawn_chunk() to use texture instead of color
4. Build and test

### Phase 2: Add Normal Maps (1 hour)
1. Include normal map texture
2. Update StandardMaterial to reference normal_map_texture
3. Adjust roughness/metallic values
4. Test lighting on textured ground

### Phase 3: Scatter Rocks (1 hour)
1. Download rock models (3D .glb files)
2. Spawn rocks randomly in chunks (similar to trees)
3. Add collision cuboids to rocks
4. Test flying around obstacles

### Phase 4: Terrain Variation (3-4 hours)
1. Implement Perlin noise heightmap generation
2. Create terrain mesh per chunk (not flat plane)
3. Create matching collision mesh
4. Blend textures (grass → dirt → rock by elevation)

### Phase 5: Polish (2 hours)
1. Fix blue cube issue
2. Adjust tree scaling
3. Optimize performance
4. Add final details

---

## Gemini Flash Job: Download Assets + Implement Textures

**What Flash should do:**
1. Download grass + rock textures from Poly Haven
2. Copy to assets/textures/
3. Modify spawn_chunk() to use textures
4. Test ground appearance
5. Adjust colors/roughness as needed

---

## Gemini 3 Pro Job: Implement Heightmap Terrain

**What Pro should do:**
1. Implement Perlin noise heightmap generation
2. Create terrain mesh from heightmap (not flat plane)
3. Create collision mesh matching terrain
4. Test flying over varied terrain

---

## Performance Considerations

| Feature | Performance Impact | Optimization |
|---------|-------------------|---------------|
| Texture loading | Low | Cache materials, use atlasing |
| Normal maps | Low | Already computed by GPU |
| Heightmap generation | Low (per-chunk) | Can be done once at startup |
| Complex collision mesh | Medium | Use simpler shapes for far chunks |
| Rock models | Medium | Limit count per chunk or use LOD |
| Blend multiple textures | Medium | Use texture splatting / blend maps |

---

## Blue Cube Issue - Investigation

The blue cube might be:
1. **Horizon disk** clipping into visible area (Y position wrong)
2. **Sky sphere** bottom edge becoming visible (unlikely)
3. **Debug geometry** left from previous testing (check setup_scene)
4. **Chunk edge** rendering incorrectly

**Quick fix to test:**
- In setup_scene(), check HorizonDisk Y = -2.0
- Adjust to Y = -10.0 or Y = -100.0
- If cube moves/disappears, that's the issue
- Then reposition to be unnoticeable

---

## Asset File Structure (After Implementation)

```
assets/
├── textures/
│   ├── grass_BaseColor.jpg
│   ├── grass_Normal.jpg
│   ├── grass_Roughness.jpg
│   ├── rock_BaseColor.jpg
│   ├── rock_Normal.jpg
│   ├── dirt_BaseColor.jpg
│   └── dirt_Normal.jpg
├── models/
│   ├── drone.glb
│   ├── rock_large.glb
│   ├── rock_medium.glb
│   └── rock_small.glb
└── fantasy_town/
    └── (existing tree/building assets)
```

---

## Success Criteria

✅ Ground has visible grass texture (not plain color)
✅ Bumpy appearance with normal maps
✅ Rocks/obstacles scattered in world
✅ Varied terrain elevation (mountains/valleys)
✅ Plane can crash into terrain realistically
✅ 60+ FPS maintained
✅ No visual glitches or blue cubes
✅ Seamless chunk boundaries

---

**Ready to proceed?**
- Recommend: Start with Phase 1 (Quick texture) while investigating blue cube
- Then Phase 2 (Normal maps)
- Then Phase 3 (Rocks)
- Then Phase 4 (Heightmap terrain) if time permits

