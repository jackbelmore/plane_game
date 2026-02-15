# Machine Gun Implementation Plan

**Objective:** Add a secondary primary weapon (Vulcan Cannon) firing from wingtips.
**Status:** Planned
**Dependencies:** Audio (needs processing), Input (LMB)

---

## 1. Audio Asset Generation
**Goal:** Create a rapid-fire cannon sound from existing assets.
- **Source:** `assets/sounds/missile_light.ogg`
- **Processing:** Pitch shift +3.0x (Speed), Boost Volume +2dB.
- **Output:** `assets/sounds/machine_gun.ogg`

## 2. Code Architecture

### A. Asset Loader (`src/assets.rs`)
Add the new sound handle:
```rust
#[asset(path = "sounds/machine_gun.ogg")]
pub machine_gun: Handle<AudioSource>,
```

### B. Components (`src/main.rs`)
1. **MachineGunState** (Player Component)
   ```rust
   #[derive(Component)]
   struct MachineGunState {
       last_fired: f32,
       fire_side: bool, // Toggles Left/Right
   }
   ```

2. **Bullet** (Projectile Component)
   ```rust
   #[derive(Component)]
   struct Bullet {
       lifetime: f32,
   }
   ```

### C. Constants
```rust
const MG_FIRE_RATE: f32 = 0.08; // ~12 shots/sec
const MG_SPEED: f32 = 1200.0;   // Very fast
const MG_DAMAGE: f32 = 8.0;     // Low damage per hit
const MG_OFFSET_LEFT: Vec3 = Vec3::new(-7.0, -1.0, -2.0);
const MG_OFFSET_RIGHT: Vec3 = Vec3::new(1.0, -1.0, -2.0);
```

## 3. Systems

### `handle_machine_gun_input`
- **Input:** `MouseButton::Left`
- **Logic:**
  - Check cooldown.
  - Determine offset based on `fire_side`.
  - Toggle `fire_side`.
  - Spawn Bullet (Mesh: Capsule, Material: Emissive Yellow).
  - Play Sound (Non-spatial, randomized pitch).

### `update_bullets`
- Move bullets (LinearVelocity handled by physics, or manual if Raycast preferred).
- *Decision:* Use Physics `RigidBody::Kinematic` or `Dynamic`?
  - **Decision:** `Dynamic` with `GravityScale(0.0)` matches Missiles and is easiest.

### `bullet_drone_collision`
- separate system from missiles?
  - **Yes**, to keep logic clean and allow different hit effects.
- **Visuals:** Small yellow "spark" sphere on hit.

## 4. Execution Steps
1. **Generate Sound** (ffmpeg).
2. **Update Code** (Assets -> Components -> Input -> Collision).
3. **Build & Test**.
