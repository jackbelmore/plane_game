# Technical Knowledge Base - Plane Game

This document consolidates critical technical insights, bug fixes, and architectural workarounds discovered during development.

---

## 🚀 Physics & Safety (NaN Protection)

**Problem:** Avian3D crashes (AABB assertion) when `Transform` or `Velocity` components contain `NaN` or invalid values.
**Solution:** A multi-layered safety system in `FixedFirst` schedule (runs before physics).

### 1. detect_nan_early (System)
- **Scale Validation:** If `transform.scale` has any dimension ≤ 0 or `NaN`, it's reset to a safe default (e.g., `Vec3::ONE` or `Vec3::splat(0.08)` for the F-16).
- **Position/Velocity Reset:** If `NaN` is detected in translation or velocity, the entity is reset to a safe world position (e.g., `(0, 500, 0)`).
- **Implementation Note:** This system must run in `FixedFirst` to catch issues before the physics engine processes the frame.

### 2. Ground Collision Safety
- **Hard Reset:** If the player's altitude exceeds 100,000m or drops below -1,000m, a hard reset is triggered to prevent physics engine overflows.

---

## 🎨 Rendering & Asset Loading (Bevy 0.15)

### 1. SceneRoot Workaround
- **Issue:** `SceneRoot` often fails to spawn visible children for simple `.glb` files (e.g., trees, drones) in Bevy 0.15.
- **Solution:** Use direct `Mesh3d` + `MeshMaterial3d` loading.
- **Pattern:**
  ```rust
  let mesh_handle = asset_server.load("models/tree.glb#Mesh0/Primitive0");
  commands.spawn((
      Mesh3d(mesh_handle),
      MeshMaterial3d(material_handle),
      // ...
  ));
  ```

### 2. Async Texture/Material Bug (Issue #15081)
- **Issue:** Materials with slow-loading textures (JPG/PNG) may never update their bind groups, leaving models black or with missing textures.
- **Workaround:**
  - Use **Procedural Textures** created at startup for immediate availability.
  - Or use a `check_asset_loaded` system that replaces a temporary material with the final one once `AssetServer::get_load_state` returns `Loaded`.

### 3. Horizon & Sky Continuity
- **Jagged Horizon:** Occurs when the camera far clip is shorter than the fog distance.
- **Fix:** Set `Projection::Perspective { far: 100000.0, ..default() }` (100km).
- **Infinite Earth:** Use a large `HorizonDisk` (100km radius) at a slight Y-offset, matching the chunk ground color exactly, synced with Fog start/end distances.

---

## 🔧 World & Chunk System

### 1. Performance Baseline
- **LOAD_RADIUS:** 8km (8 chunks) provides a 3km cushion before ground holes appear.
- **UNLOAD_RADIUS:** 12km (12 chunks).
- **Tree Density:** 5-10 trees per chunk is the stable range for RTX 3080 Ti / Ryzen 5800X.

### 2. Spawning Gotchas
- **Deterministic Spawning:** Use chunk coordinates as a seed for RNG to ensure the same assets spawn in the same chunks every time.
- **LOD System:** Move LOD update systems to `PostUpdate` to ensure they see newly spawned entities immediately.

---

## 🔊 Audio & Formatting
- **Rodio Metadata Crash:** Bevy's audio backend (`rodio`) panics on OGG/WAV files with embedded cover art or certain metadata.
- **Fix:** Strip metadata using FFmpeg: `ffmpeg -i input.ogg -vn -c:a copy output.ogg`.

---
**Last Updated:** February 15, 2026
