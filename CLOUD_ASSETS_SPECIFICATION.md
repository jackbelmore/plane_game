# Cloud Assets Specification

## Overview
The current cloud implementation uses white rectangular blocks. We need proper cloud textures for a realistic sky.

## Requirements for Cloud Textures

### Cloud Texture Characteristics
- **Format**: PNG with transparency (8-bit or 32-bit with alpha channel)
- **Resolution**: 512x512 or 1024x1024 (larger = better quality at distance)
- **Color**: White or light gray (RGB: 255, 255, 255 or close)
- **Alpha Channel**: Gradient edges (feathered/soft edges, not hard cutoff)
  - Center: Fully opaque (alpha 255)
  - Edges: Gradient fade to transparent (alpha 0-100)
  - This creates soft, realistic cloud silhouettes

### Cloud Texture Types Needed

#### 1. **Fluffy Cloud Texture** (Primary)
- **Purpose**: Main cloud billboard texture
- **Appearance**: Cumulus-style cloud (puffy, rounded shapes)
- **Characteristics**:
  - Multiple bumps/protrusions for organic shape
  - Soft lighting highlights on top
  - Shadow underneath for depth
  - Should look good when tiled/overlapping
- **Quantity**: 3-5 variations

**Example characteristics:**
- Soft white with subtle gray shading
- Bumpy surface texture for realism
- No hard edges - all soft/feathered

#### 2. **Wispy Cloud Texture** (Secondary)
- **Purpose**: Thin clouds, cirrus clouds at high altitude
- **Appearance**: Thin, stretched, wispy clouds
- **Characteristics**:
  - Very soft, almost translucent
  - Brush-stroke like patterns
  - Lower opacity overall (40-60%)
- **Quantity**: 2 variations

#### 3. **Dense Cloud Texture** (Optional)
- **Purpose**: Thick clouds for lower altitudes
- **Appearance**: Heavy, dense clouds
- **Characteristics**:
  - Darker gray-white gradients
  - More opaque
  - Solid feeling
- **Quantity**: 1-2 variations

### Texture Dimensions (Choose One)
```
512x512   - Good balance (recommended for this game)
1024x1024 - Best quality, more file size
2048x2048 - High quality but overkill for a flight game
```

### Where to Find Cloud Assets

#### Free Resources:
1. **Kenney Assets** (similar to existing assets)
   - Has cloud assets in particle pack
   - Recommended: Check `E:\Downloads\kenney_particle-pack\`
   - Look for cloud-related PNGs

2. **OpenGameArt.org**
   - High-quality free cloud textures
   - Search: "cloud texture", "sky", "cumulus"
   - Filter by CC0 or CC-BY licenses

3. **Poly Haven** (polyhaven.com)
   - Excellent free textures
   - Search: "cloud" in textures
   - Free for commercial use
   - Download as PNG with alpha

4. **TextureHaven** (texturehaven.com)
   - Same as Poly Haven
   - Quality cloud textures

5. **Free3D.com**
   - Free cloud textures
   - Various styles

### Custom Cloud Creation (DIY)
If creating custom textures:
- **Photoshop**: Use cloud filter → adjust levels → add soft edges
- **Blender**: Generate procedural clouds → bake to texture
- **Python Pillow**: Generate procedural clouds programmatically

### Recommended Download Approach

**Best Option**: Download from Poly Haven / Kenney
1. Visit polyhaven.com → Textures
2. Search "cloud"
3. Download 5 textures:
   - 2-3 fluffy cloud textures
   - 1-2 wispy cloud textures
4. Save as: `assets/textures/clouds/cloud_01.png`, `cloud_02.png`, etc.

**Free Kenney Option**:
- Check existing kenney_particle-pack for cloud PNGs
- File path: `E:\Downloads\kenney_particle-pack\`
- Look for: `smoke_*.png`, `cloud_*.png`

### Expected File Size
- 512x512 PNG: ~100-200 KB per texture
- 1024x1024 PNG: ~300-500 KB per texture
- Total for 5 textures: 500 KB - 2.5 MB

### Implementation Coordinates

Once assets are downloaded, place them here:
```
C:\Users\Box\plane_game\assets\textures\clouds\
├── cloud_fluffy_01.png
├── cloud_fluffy_02.png
├── cloud_fluffy_03.png
├── cloud_wispy_01.png
└── cloud_wispy_02.png
```

Then update the code in `spawn_clouds()` to use:
```rust
let cloud_textures = vec![
    "textures/clouds/cloud_fluffy_01.png",
    "textures/clouds/cloud_fluffy_02.png",
    "textures/clouds/cloud_fluffy_03.png",
    "textures/clouds/cloud_wispy_01.png",
    "textures/clouds/cloud_wispy_02.png",
];
```

### Visual Quality Comparison

**Current (White Blocks):**
- ❌ Blocky appearance
- ❌ No texture detail
- ❌ Looks like flat planes
- ❌ Unrealistic

**With Proper Cloud Textures:**
- ✅ Soft, organic shapes
- ✅ Atmospheric depth
- ✅ 3D appearance via shading
- ✅ Realistic fluffy clouds
- ✅ Creates motion parallax effect

### Expected Visual Improvement
When flying through clouds with proper textures:
- At 150-200 m/s: Individual clouds visible with detail
- At 300+ m/s: Clouds stream past with visible texture
- Altitude variation: Can see different cloud densities at different heights
- Lighting: Highlights on cloud tops add realism

### Quick Test
Once assets are loaded, fly to 1000-2000m altitude and perform:
- Pitch down to see clouds above (backlighting)
- Pitch up to see clouds below
- Speed up to see parallax effect
- Turn to see cloud shapes from different angles

This creates natural waypoints for navigation during flights.

## Next Steps
1. Download 5 cloud textures (fluffy + wispy types)
2. Place in `assets/textures/clouds/` directory
3. Provide filenames to update code
4. Rebuild and test visual quality
5. Adjust cloud spawn positions/scale if needed

---

**Recommendation**: Use Poly Haven for best free quality. Takes ~5 minutes to download and place in correct directory.
