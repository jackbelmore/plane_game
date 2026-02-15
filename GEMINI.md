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
| **Space** | Fire Missiles |
| **R** | Toggle Rocket Mode / Restart (if crashed) |
| **ESC** | Respawn |
| **F10** | Quit |

## Debugging & Output
The console output has been optimized for flight system debugging:
*   **Flight Telemetry:** Look for `[FLIGHT]` logs every second.
    *   Format: `[FLIGHT] ALT: 500 m | SPD: 100 m/s | P: 0.0° R: 0.0° Y: 0.0° | THR: 0%`
*   **Asset Loading:** "Model load state" logs indicate progress.
*   **Chunk System:** "CHUNK SPAWN" logs show world generation.

## Development Status (2026-02-15)
*   **Lighting:** ✅ Implemented (Directional + Ambient).
*   **Fog:** ✅ **Fixed:** Start: 3km, End: 12km. Hides the chunk edge (8km) for seamless horizon.
*   **Villages:** ✅ Spawning (15% chance).
    *   **Walls:** Visible.
    *   **Roofs:** ✅ **Fixed:** Raised to Y=32.0 to sit on top of 30m walls.
*   **Drones:** ✅ Spawning (Infinite Patrols).
    *   **Visibility:** ✅ **Fixed:** Restored red cube visual fallback.
*   **Flight Physics:** Arcade-style physics implemented in `main.rs`.

## Known Issues
*   **Terrain:** Ground is flat (needs heightmap).
*   **Vulkan Errors:** Validation errors on startup (harmless).

## Contribution
*   Avoid adding unused variables (use `_` prefix).
*   Keep flight logs concise.
