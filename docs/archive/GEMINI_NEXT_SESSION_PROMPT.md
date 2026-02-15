# Gemini Session - Next Priority Tasks

**Date:** 2026-02-06 (Claude → Gemini Handoff)
**Status:** Game stable + infinite drone spawning working, but critical regression found

---

## 🚨 CRITICAL ISSUE: Drone Visibility Regression

### What Happened
Your previous work removed the visual fallback for drones when switching from direct mesh loading to SceneRoot. This means **drones are now invisible if the SceneRoot doesn't load properly**.

### Before Your Changes
```rust
// Direct mesh loading with visible fallback
let drone_handle: Handle<Mesh> = asset_server.load("models/drone.glb#Mesh0/Primitive0");
let material = materials.add(StandardMaterial {
    base_color: Color::srgb(0.2, 0.2, 0.2),  // Dark gray - VISIBLE
    ...
});
commands.spawn((
    Mesh3d(drone_handle),
    MeshMaterial3d(material),  // ALWAYS VISIBLE
    ...
));
```

### After Your Changes
```rust
// SceneRoot with NO visual fallback
let drone_scene_handle = asset_server.load("models/drone.glb#Scene0");
commands.spawn((
    Drone { ... },
    // NO MESH3D, NO VISUAL MATERIAL → Drones may be INVISIBLE
))
.with_children(|parent| {
    parent.spawn((
        SceneRoot(drone_scene_handle),  // IF this fails to load = invisible drone
    ));
    // FALLBACK REMOVED HERE
});
```

**Result:** Drones spawn and run AI, but may not be visible in-game.

---

## 🎯 Priority 1: Fix Drone Visibility (MUST DO FIRST)

### Option A: Restore Visual Fallback (RECOMMENDED)
Add a visible fallback cube so drones are always visible during testing:

```rust
.with_children(|parent| {
    // Try to load SceneRoot
    parent.spawn((
        SceneRoot(drone_scene_handle),
        Transform::from_scale(Vec3::splat(15.0)),
    ));

    // ADD FALLBACK: Red cube visible if model fails to load
    parent.spawn((
        Mesh3d(meshes.add(Cuboid {
            half_size: Vec3::new(9.0, 4.5, 12.0),
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),  // RED = visible
            emissive: LinearRgba::rgb(0.5, 0.0, 0.0),
            ..default()
        })),
        Transform::from_scale(Vec3::splat(1.0)),
    ));
});
```

**Why:** Even if SceneRoot fails, you'll see red cubes and know the AI is working. Game is fully playable + testable.

### Option B: Verify SceneRoot Path
If you want to use ONLY the 3D model, verify the path is correct:

1. Check the drone.glb file structure:
   ```bash
   # On Windows:
   cd C:\Users\Box\plane_game
   # Try to list scene names:
   # - `#Scene0` = entire scene
   # - `#Mesh0` = first mesh
   # - `#Mesh0/Primitive0` = first mesh's first primitive
   ```

2. Test both paths:
   - Current: `models/drone.glb#Scene0`
   - Previous: `models/drone.glb#Mesh0/Primitive0`
   - Try: `models/drone.glb` (no selector)

3. Add debug logging:
   ```rust
   let load_state = asset_server.get_load_state(&drone_scene_handle);
   println!("Drone model load state: {:?}", load_state);
   ```

**If using SceneRoot:** Still add a fallback cube so you have visual feedback.

---

## 🎮 Priority 2: Test Drone Combat Loop

After fixing visibility:

1. **Launch game** and confirm:
   - [ ] Drones visible as red cubes OR 3D models
   - [ ] Drones approaching from distance
   - [ ] Can fire missiles (Space key)
   - [ ] Missiles hit drones and cause explosions
   - [ ] Drones disappear after being hit
   - [ ] FPS stable 60+ during combat

2. **Test infinite spawn:**
   - [ ] Fly 5+ km away
   - [ ] New drones spawn in front of you
   - [ ] Drones warp-pursue if you outrun them
   - [ ] No crashes or NaN errors

---

## 🔧 Priority 3: Optional Fixes (Pick One)

### Option A: Fix Drone 3D Model Properly
**If** the SceneRoot path is wrong, load the mesh correctly:
- Document what `drone.glb` actually contains
- Use correct selector: `#Scene0` vs `#Mesh0` vs `#Mesh0/Primitive0`
- Test with async loading monitor

### Option B: Fix JPG Texture (Bevy Bug Workaround)
The ground texture issue ([Issue #15081](https://github.com/bevyengine/bevy/issues/15081)):
- Implement `bevy_asset_loader` crate for proper async material loading
- OR: Try TGA format instead of JPG (faster, uncompressed)
- OR: Accept procedural texture as permanent solution

### Option C: Add Speed/Altitude UI
Display on-screen:
- Current velocity (m/s)
- Altitude (m)
- Drone count
- Health bar

---

## 📋 Testing Checklist

Before declaring victory:

```
Game Startup:
  [ ] Game launches without panic
  [ ] Drones visible (cubes or models)
  [ ] Chunks load smoothly
  [ ] No "CHUNK PATROL" spam in console

Combat Test (5+ minutes):
  [ ] Can fly around without crashes
  [ ] Drones approach player
  [ ] Fire missiles (Space)
  [ ] Drones explode on hit
  [ ] New drones spawn ahead
  [ ] Can outrun drones with Rocket Mode (R)
  [ ] Drones warp-pursue when far away
  [ ] FPS stays 60+ (no stutters)

Edge Cases:
  [ ] Fly 20km up in space - drones still chase
  [ ] Fire missiles rapid-fire - no overflow
  [ ] ESC respawn works
  [ ] F10 quits cleanly
```

---

## 🔑 Key Code Sections

| Issue | File | Lines | Fix |
|-------|------|-------|-----|
| Drone visibility | `src/drone.rs` | 22-59 | Add fallback cube, restore mesh+material |
| Infinite spawn | `src/main.rs` | 1012-1018 | ✅ Already working (15% per chunk) |
| Warp pursuit | `src/drone.rs` | 167-175 | ✅ Already working (5.0x when >5km) |
| Swarm AI | `src/drone.rs` | 98-144 | ✅ Already working (flocking) |

---

## 🎯 Your Mission

1. **FIX VISIBILITY FIRST** - Add fallback cube to `spawn_beaver_drone()`
2. **Test 5+ minutes** - Confirm drones visible and combat works
3. **Pick one optional fix** - Model path verification OR texture OR UI
4. **Document findings** - Update SESSION_HANDOFF.md with what you did
5. **Commit changes** - Clean git history before handing off

---

## 📝 Build & Test Commands

```bash
# Build
cd /c/Users/Box/plane_game
cargo build --release 2>&1 | tail -20

# Copy assets
cp -r assets target/release/

# Run (watch for "Drone pursuing" messages and visual red cubes)
target/release/plane_game.exe

# Quick test (8 second timeout)
timeout 8 target/release/plane_game.exe 2>&1 | grep -E "(Drone|pursuing|ERROR)"
```

---

## 🚀 Expected Outcome

After your work:
- ✅ Drones always visible (red cubes + optional 3D models)
- ✅ Infinite drone spawning with warp pursuit works
- ✅ Game playable for 30+ minutes without crashes
- ✅ Combat loop complete (spawn → chase → shoot → explode)
- ⚠️ JPG texture still procedural (lower priority)

**The game is 95% complete. You're just finalizing Phase 3 combat polish.**

---

## ❓ Questions for You

1. Do you want to keep only the SceneRoot model (no fallback) or have a visible fallback cube?
2. Should we prioritize fixing the drone model loading OR the JPG texture issue next?
3. Do you want to add visual/audio warnings when drones get close (missile warning system)?

Good luck! 🚀
