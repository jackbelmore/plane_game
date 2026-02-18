# Plane Game

## Project Overview
A 3D flight simulator built with **Rust** and **Bevy Engine**. Features procedural terrain chunks, drone combat, and arcade-style flight physics.

## Key Directories
- `src/`: Source code (`main.rs`, `drone.rs`, `ui.rs`).
- `assets/`: 3D models (`.glb`), textures, and sounds.
- `docs/`: Design documents and technical notes.

## Building & Running
*   **Run (Debug):** `cargo run`
*   **Run (Release):** `cargo run --release` (Recommended for performance)
*   **Check:** `cargo check`

## Controls
| Key | Action |
|---|---|
| **W / S** | Pitch Down / Up |
| **A / D** | Roll Left / Right |
| **Q / E** | Yaw Left / Right |
| **Shift** | Throttle Up (Boost) |
| **Ctrl** | Throttle Down |
| **Space / RMB** | Fire Missiles |
| **LMB** | Fire Machine Gun |
| **P** | Pause / Resume Game |
| **R** | Toggle Rocket Mode |
| **F5** | Restart (Respawn) |
| **F10** | Quit |

## Debugging & Output
The console output has been optimized for flight system debugging:
*   **Flight Telemetry:** Look for `[FLIGHT]` logs every second.
    *   Format: `[FLIGHT] ALT: 500 m | SPD: 100 m/s | P: 0.0° R: 0.0° Y: 0.0° | THR: 0%`
*   **Asset Loading:** "Model load state" logs indicate progress.
*   **Chunk System:** "CHUNK SPAWN" logs show world generation.

## Development Status (2026-02-18)
*   **Terrain:** ✅ **Tactical Biome System Implemented:**
    *   Three distinct biomes: Lowlands (Plains), The Spine (1.4km Mountains), and Tactical Canyons.
    *   **Directional Shading:** Terrain now calculates surface normals, providing realistic light and shadow on slopes.
*   **Lighting & Visuals:** ✅ **Physical Rendering (PBR) Overhaul:**
    *   Implemented physical light units (100,000 lux sunlight).
    *   **Environment Mapping (IBL):** Added specular cubemaps for realistic metallic reflections on the aircraft.
    *   **Physical Exposure:** Camera uses EV100=15.0 for accurate outdoor daylight rendering.
    *   **Volumetric Trails:** Jet exhaust upgraded to "Ribbon" interpolation with thermal evolution (fire-to-smoke transition).
*   **Combat & AI:** ✅ **Advanced Swarm Intelligence:**
    *   Drones use lead-pursuit, obstacle avoidance (meteors), and tactical weaving.
    *   Drone weapon systems fire missiles (800-2000m) and machine guns (<1000m).
    *   **Combat Director:** Resource-based balancing to prevent overwhelming the player.
*   **Environment:** ✅ **Dynamic Obstacles:**
    *   Meteors are now Dynamic physical objects with zero-gravity float.
    *   "Titan Class" meteors (up to 120m scale) added as massive floating islands.
*   **Audio:** ✅ **Cinematic Audio System:**
    *   Physics-based "Air Rip" (Wind Stress) sound linked to G-Force and AoA.
    *   Manual Audio Attenuation with Doppler shift for missiles and drones.
*   **Game Loop:** ✅ **Pause & Safety Systems:**
    *   'P' toggles pause/resume; physics and audio freeze correctly.
    *   **NaN Safety:** Aggressive global protection against physics engine crashes.

## Known Issues
*   **Vulkan Errors:** Validation errors on startup (harmless).

## Contribution
*   Avoid adding unused variables (use `_` prefix).
*   Keep flight logs concise.
