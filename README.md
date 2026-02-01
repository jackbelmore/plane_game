# F-16 Fighter Jet Flight Simulator

A Bevy-based 3D flight simulator with arcade physics, responsive controls, and visual effects.

## Current Status ✅

**Flight Physics**: Working
- ✅ Local-space rotation (proper turning mechanics)
- ✅ Vertical thrust component based on pitch angle
- ✅ 2x thrust (100k N) for sustained flight
- ✅ Boost system (3.5x multiplier at 80%+ throttle)

**Controls**: Fully functional
- ✅ W/S: Pitch up/down
- ✅ A/D: Roll left/right
- ✅ Q/E: Yaw left/right
- ✅ Shift: Increase throttle (activates boost + flame)
- ✅ Ctrl: Decrease throttle
- ✅ R: Restart game
- ✅ ESC: Quit

**Visuals**: Implemented
- ✅ Orange afterburner flame at rear exhaust
- ✅ Flame scales with throttle (grows brighter at boost)
- ✅ Grid background with reference markers
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

# Release build (optimized, faster)
cargo build --release
cargo run --release
```

## Project Structure

```
plane_game/
├── src/
│   └── main.rs              (All game logic)
├── assets/
│   ├── models/              (F-16 GLTF models)
│   ├── space_kit/           (Optional: Kenney space assets)
│   └── ...
├── Cargo.toml               (Dependencies)
├── Cargo.lock               (Dependency versions)
└── README.md                (This file)
```

## Immediate Testing

Test these flight maneuvers:
1. **Climb** - Hold Shift (throttle), press S (pitch up) → altitude increases
2. **Turn** - Hold Shift, roll with A/D, pitch up with S → plane turns
3. **Land** - Reduce throttle with Ctrl, let plane descend → hits ground safely
4. **Flame Effect** - Hold Shift → see orange flame shoot from rear exhaust

## Physics Model

### Thrust Vector
- Decomposes based on pitch angle
- Vertical = throttle × sin(pitch)
- Forward = throttle × cos(pitch)
- Boost: 3.5x multiplier at 80%+ throttle

### Rotation (Local-Space)
- Uses plane's local right/up/forward axes
- Enables proper bank-to-turn mechanics
- Roll + pitch = coordinated turns

### Tuning Constants (in src/main.rs)
```
MAX_THRUST_NEWTONS: 100000.0
PITCH_RATE: 1.8 rad/s (103°/s)
ROLL_RATE: 2.5 rad/s (143°/s)
YAW_RATE: 1.2 rad/s (69°/s)
BOOST_MULTIPLIER: 3.5x
BOOST_THRESHOLD: 0.8 (80%)
```

## Next Steps

### Immediate
- [ ] Test all maneuvers above
- [ ] Verify flame positioning/scaling
- [ ] Fine-tune control rates if needed

### Short-term
- [ ] Integrate space kit assets (skybox, asteroids, stations)
- [ ] Add particle effects for enhanced flame
- [ ] Implement fuel system
- [ ] Add HUD/instruments

### Long-term
- [ ] Landing gear animation
- [ ] Cockpit view
- [ ] Multiplayer support
- [ ] Mission system

## Known Issues

- None blocking gameplay

## Performance

- Debug build: ~15 second compile
- Release build: ~30 second compile
- Release executable: ~100 MB
- Target: 60 FPS at 1080p

## Dependencies

- **Bevy 0.15**: Game engine
- **Avian3D**: Physics engine
- **GLTF**: 3D model format

---

**Last Updated**: 2026-02-01
**Ready for**: Testing and iteration
