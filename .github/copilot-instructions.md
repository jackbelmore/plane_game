# Copilot Instructions for Plane Game

## Build, Run, and Test

**Build:**
```bash
cargo build --release
```

**Run:**
```bash
cargo run --release
```

**Development mode (faster iteration, slower runtime):**
```bash
cargo run
```

The project has custom optimization settings in `Cargo.toml` - dev builds use `opt-level = 1` with dependencies at `opt-level = 3` for fast iteration without terrible performance.

## Architecture Overview

This is a **Bevy ECS-based 3D arcade dogfighting game** using Avian3D for physics. All code is currently in `src/main.rs`.

### Entity Hierarchy

The player is a **3-level entity hierarchy**:

1. **PlayerPlane (parent)** - Physics and logic container
   - Has all physics components (RigidBody, Collider, LinearVelocity, etc.)
   - Has game components (PlayerInput, FlightState, FlightCamera)
   - Transform represents the actual position/rotation in the world

2. **ModelContainer (child)** - Visual rotation layer
   - Allows rotating the visual model (e.g., banking animation) without affecting physics
   - Currently at `Transform::IDENTITY`

3. **Visual meshes (grandchildren)**
   - Temporary blue cube placeholder (`Mesh3d` + `MeshMaterial3d`)
   - GLTF model (`SceneRoot`) loaded asynchronously from `assets/models/fighter_jet.gltf`

**Why this structure?** Physics operates on the parent's transform, but we can rotate/tilt the visual model independently for effects like banking or animations.

### System Flow

Systems run in this order each frame:
1. `read_player_input` - Reads keyboard, updates `PlayerInput` component
2. `apply_flight_physics` - Converts input to forces/torques, applies arcade flight model
3. `apply_angular_damping` - Multiplies angular velocity by damping constant (plane stops spinning when keys released)
4. `update_flight_camera` - Slerps camera rotation for smooth lag effect
5. `debug_player_state` - Prints debug info every 60 frames

### Flight Physics Model

**Arcade-style physics** with these key behaviors:
- **Thrust**: Constant forward force along `-transform.local_z()` (Bevy's forward is -Z)
- **Lift**: Speed-based upward force (only kicks in above `min_speed`)
- **Bank-to-turn**: Roll angle (`roll`) creates yaw torque via `TURN_FACTOR`
- **Pitch limiting**: Torque is reduced as pitch approaches `MAX_TILT_ANGLE` (~80°)
- **Angular damping**: `ANGULAR_DAMPING = 0.75` creates strong slowdown when input stops

All physics constants are in the `CONSTANTS` section and tuned for arcade feel, not realism.

## Key Conventions

**Coordinate system:**
- Bevy uses **Y-up, -Z forward, +X right**
- `transform.local_z()` points backward, so forward is `-transform.local_z()`
- `transform.local_x()` is right/left axis
- `transform.local_y()` is up/down axis

**Component design pattern:**
- Components are pure data (`struct` with fields)
- Systems are functions that query components and mutate them
- Use marker components (e.g., `PlayerPlane`) to identify specific entities

**Transform parent-child behavior:**
- Child transforms are **local** to parent
- Physics (Avian3D) operates on the **parent's global transform**
- Children inherit parent's transform but can have their own local offset/rotation

**Resource vs Component:**
- Components attach to entities (e.g., `PlayerInput` on player entity)
- Resources are global singletons (e.g., `Time`, `Assets<Mesh>`)

## Current Phase

**Phase 1: COMPLETE** - Greybox prototype with arcade flight physics, camera, and basic collision.

**Next phases** (from README):
- Phase 2: Replace cube with 3D plane model
- Phase 3: Add shooting mechanics
- Phase 4: Implement enemy AI
- Phase 5: Polish (sound, particles, UI, menus)

## Notes

- The `assets/models/fighter_jet.gltf` model loads asynchronously - it may not appear immediately
- Debug output prints every ~1 second (60 frames) showing position, velocity, and input
- Camera uses **slerp** (spherical linear interpolation) for smooth rotation lag
- All startup systems (`setup_scene`, `spawn_player`) run once; update systems run every frame
