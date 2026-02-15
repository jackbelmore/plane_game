# GEMINI.md - F-16 Fighter Jet Flight Simulator

## Project Overview
A high-performance 3D flight simulator built with **Rust** and the **Bevy 0.15** game engine. It features arcade-style flight physics, an infinite procedurally generated world, and a seamless transition from atmospheric flight to space.

### Key Technologies
- **Engine:** Bevy 0.15
- **Physics:** Avian3D (AABB collision detection, rigid body dynamics)
- **Asset Loading:** `bevy_asset_loader` v0.22.0
- **Language:** Rust (2021 Edition)
- **Assets:** GLTF/GLB for 3D models, OGG for audio, PNG for textures.

---

## 🚀 Building and Running

### Commands
- **Run (Recommended):** `cargo run --release`
  - *Note: Release mode is critical for physics stability and smooth framerates.*
- **Check Syntax:** `cargo check`
- **Build Only:** `cargo build --release`

### Backup/Sync
To mirror the project to Google Drive (excluding build artifacts):
```powershell
robocopy "C:\Users\Box\plane_game" "G:\My Drive\plane_game" /MIR /XD target
```

---

## 🏗️ Architecture & State Management

### GameState System
The game uses a state machine to manage initialization and asset loading:
- **`GameState::Loading`:** Initial state where `bevy_asset_loader` pre-loads all critical textures and sounds.
- **`GameState::Playing`:** Active gameplay state. Systems are gated using `.run_if(in_state(GameState::Playing))` to ensure they only execute when assets and the player are ready.

### Asset Management (`GameAssets`)
Centralized asset handling via `src/assets.rs`. This struct uses `AssetCollection` to manage:
- **Textures:** Ground grass (BaseColor and Normal maps).
- **Audio:** Engine loops, weapon sounds, explosion effects, and ambient wind.
- **Stability:** All gameplay systems must access these handles via `Res<GameAssets>` rather than loading them ad-hoc during runtime.

---

## ✈️ Flight Systems & Physics

### Control Scheme
- **Pitch:** `W` (Down) / `S` (Up)
- **Roll:** `A` (Left) / `D` (Right)
- **Yaw:** `Q` (Left) / `E` (Right)
- **Throttle:** `Left Shift` (Increase) / `Left Ctrl` (Decrease)
- **Combat:** `Space` (Fire Missiles/Projectiles)
- **Special:** `R` (Toggle Rocket Mode - 8x thrust for space travel)
- **System:** `F5` (Restart), `ESC` (Quit)

### Physics Model
- **Arcade Flight:** Direct control handling with custom lift, drag, and thrust vectors.
- **Aerodynamics:** Uses `F16AeroData` curves for lift/drag vs. Alpha, including damping factors to prevent control divergence.
- **Safety:** A `safety_check_nan` system monitors transforms and velocities to prevent physics engine crashes from invalid values.

---

## 🌍 World Architecture

### Infinite World (Chunk System)
- **Size:** 1km x 1km chunks (`CHUNK_SIZE: 1000.0`).
- **Loading:** Chunks are dynamically spawned/despawned based on player position.
- **Procedural Content:** Trees and medieval villages are generated per chunk based on seeded RNG.
- **Atmosphere:** Seamless sky-to-space transition between 15km and 25km altitude.

---

## 🎨 Asset Conventions

### Textures (Ground)
- **Source:** `assets/textures/grass/grass_BaseColor.png`.
- **Configuration:** A startup system `configure_grass_texture_sampler` sets the sampler to `AddressMode::Repeat` to ensure seamless tiling across ground chunks.

### Audio (CRITICAL)
- **Format:** **MUST be .ogg**.
- **Constraint:** Files must be **Audio Only**. Use `ffmpeg -vn` to strip any embedded cover art or video streams. Bevy's audio backend (rodio) will panic with `UnrecognizedFormat` if metadata streams are present.

### Models
- **Format:** `.glb` (GLTF Binary).
- **Scaling:** The F-16 model is typically scaled to `0.08x` in-engine to match the world scale.

---

## 📂 Project Structure
- `src/main.rs`: Monolithic entry point containing the game state definition and core systems.
- `src/assets.rs`: Definitions for the `GameAssets` collection.
- `src/ui.rs`: HUD and UI rendering logic (renders via `Camera2d` on top of 3D).
- `src/drone.rs`: Drone AI behavior and swarm logic.
- `assets/`:
  - `models/`: GLB files for the jet and environment.
  - `sounds/`: OGG audio loops and effects.
  - `textures/`: PNG textures for environment detail.
- `.github/`: Contains detailed technical post-mortems and fix documentation.

---

## 🛠️ Development Guidelines
- **System Limits:** Bevy has a 20-system limit per `add_systems` tuple. Split systems into multiple calls if necessary.
- **LOD:** Simple distance-based LOD is implemented for trees to maintain performance.
- **Stability:** Massive damping factors (e.g., `cl_p: -5.0`) are used in aero data to prevent high-speed rotation glitches.
- **Logging:** A custom `debug_log` function is available in `src/main.rs` that writes to `.cursor/debug.log`.