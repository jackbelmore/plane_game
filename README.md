# F-16 Fighter Jet Flight Simulator

A Bevy-based 3D flight simulator with arcade physics, responsive controls, and visual effects.

## Current Status ✅

**World & Environment**:
- ✅ **Infinite World**: Chunk-based loading system (10km view distance).
- ✅ **Forests**: Procedurally generated trees (3000+ visible).
- ✅ **Villages**: Procedurally placed medieval villages (Kenney assets).
- ✅ **Space Travel**: Seamless transition from earth sky to black space at 25km+.

**Flight Physics**: Working
- ✅ Local-space rotation (proper turning mechanics)
- ✅ Vertical thrust component based on pitch angle
- ✅ **Rocket Mode**: 8x thrust for space travel (Toggle with 'R')
- ✅ Boost system (3.5x multiplier at 80%+ throttle)

**Controls**: Fully functional
- ✅ W/S: Pitch up/down
- ✅ A/D: Roll left/right
- ✅ Q/E: Yaw left/right
- ✅ Shift: Increase throttle (activates boost + flame)
- ✅ Ctrl: Decrease throttle
- ✅ **R: Toggle Rocket Mode (Secret)**
- ✅ Space: Fire Missiles
- ✅ **F5: Restart game**
- ✅ ESC: Quit

**Visuals**: Implemented
- ✅ Orange afterburner flame at rear exhaust
- ✅ Flame scales with throttle (grows brighter at boost)
- ✅ Realistic cloud layer (billboarded sprites)
- ✅ F-16 jet model with 0.08x scale

**Ground Collision**: Working
- ✅ Avian3D physics-based collision
- ✅ Plane bounces/stops on ground, doesn't clip through

## Prerequisites

- **Rust**: Install from https://rustup.rs/
- **GPU**: DirectX 12 / Vulkan compatible

## Build & Run

```bash
# Build
cargo build

# Run
cargo run

# Release build (optimized, faster - RECOMMENDED)
cargo run --release
```

## Project Structure

```
plane_game/
├── src/
│   └── main.rs              (All game logic)
├── assets/
│   ├── models/              (F-16 GLTF models)
│   ├── fantasy_town/        (Village assets)
│   └── ...
├── Cargo.toml               (Dependencies)
└── README.md                (This file)
```

## Immediate Testing

Test these flight maneuvers:
1. **Rocket Climb** - Press **R** (Rocket Mode), pull up (S), watch altitude climb to 25km+.
2. **Space View** - At 25km, see the sky turn black and fog recede.
3. **Explore** - Fly in any direction to see new chunks, forests, and villages loading.
4. **Combat** - Use Space to shoot missiles at meteors or turrets.

## Physics Model

### Thrust Vector
- Normal: 100,000 N max
- Rocket Mode: 800,000 N max (8x)
- Boost: 3.5x multiplier at 80%+ throttle

### Tuning Constants (in src/main.rs)
```
MAX_THRUST_NEWTONS: 100000.0
ROCKET_MULTIPLIER: 8.0
PITCH_RATE: 1.8 rad/s
ROLL_RATE: 2.5 rad/s
```

## Next Steps

### Immediate
- [x] Chunk system
- [x] Rocket mode
- [x] Sky transition
- [ ] Drone enemies (Phase 3)

## Performance

- Target: 60 FPS at 1080p
- Release build recommended for physics stability

## Dependencies

- **Bevy 0.15**: Game engine
- **Avian3D**: Physics engine
- **GLTF**: 3D model format

---

**Last Updated**: 2026-02-04
**Ready for**: Testing (Phase 1 & 2 Complete)
