# Technical Blueprint: Cinematic Audio System
**Date:** 2026-02-17
**Project:** F-16 Flight Simulator (ViperEye)

This document details the high-fidelity audio implementation designed to bypass Bevy's spatial audio limitations and provide a physics-based cinematic experience.

---

## 1. Asset Preparation
High-fidelity assets from the Sonniss GDC bundle were converted to `.ogg` (libvorbis, 96kHz) to ensure stability and quality.

**Paths:**
- `assets/sounds/cinematic/air_rip.ogg` (Wing stress loop)
- `assets/sounds/cinematic/lock_on.ogg` (Target acquisition)
- `assets/sounds/cinematic/afterburner_thump.ogg` (Boost engage)

---

## 2. Core Components
The system uses manual tracking rather than the built-in spatial engine.

```rust
struct ManualAttenuation {
    max_distance: f32,       // Stop processing beyond this (e.g. 10,000m)
    reference_distance: f32, // Distance where volume is 50% (Standard: 400m)
    base_volume: f32,        // Multiplier for the sound
    age: f32,                // Tracking for ramp logic
    doppler_ramp_time: f32,  // Transition time (Standard: 0.5s)
}

struct WindStressSound;      // Marker for high-G loop
```

---

## 3. The "Manual Attenuation" System
This system calculates realistic distance and speed effects every frame.

### Inverse Distance Law (Volume)
Instead of a linear fade, we use the physics-based roll-off:
`Volume = BaseVolume * (ReferenceDistance / (ReferenceDistance + CurrentDistance))`

### Doppler Effect (Pitch)
Calculates the relative speed between the camera and the projectile.
1. `RelativeVelocity = SourceVelocity - PlayerVelocity`
2. `SpeedTowardsPlayer = RelativeVelocity.dot(DirectionToPlayer)`
3. `TargetPitch = (1.0 + (SpeedTowardsPlayer / 343.0)) * 0.5 (Cinematic Scale)`
4. `ClampedPitch = TargetPitch.clamp(0.5, 2.0)`

### The "Doppler Ramp" (Initial Punch)
To prevent the launch "crack" from sounding muddy, we ramp the Doppler effect:
`CurrentPitch = Lerp(1.0, ClampedPitch, Age / 0.5s)`

---

## 4. Cinematic Layering Logic

### Air Rip (G-Force Stress)
Triggered by **Simulated G-Force** ($Speed 	imes TurnRate$) rather than just rotation.
- **Formula:** `StressFactor = LinearVelocity.length() * AngularVelocity.length()`
- **Threshold:** Starts at `60.0`, Maxes at `150.0`.
- **Curve:** `Volume = NormalizedStress.powi(2) * 2.5` (Quadratic ramp for smooth swells).
- **Pitch:** `0.8 + (Speed / 400.0) * 0.6` (Screams at high speed).

### Afterburner Thump
- **Trigger:** Throttle state jumps from `< 0.9` to `> 0.9`.
- **Volume:** `0.2` (Subtle sub-bass thump).

### Missile Pacing & Variety
- **Fire Rate:** `5.06 shots/sec` (provides spacing for individual sounds).
- **Hero Sound (Long Whoosh):**
    - **Probability:** 1 in 30 (3.3% chance).
    - **Volume:** `1.8x`.
- **Light Sound (Standard):**
    - **Volume:** `1.0x`.

---

## 5. Critical Architecture Notes

1. **Transform Dependency:** Audio child entities **MUST** have `Transform::IDENTITY` and `Visibility::default()` to appear in the `GlobalTransform` queries used for distance math.
2. **Persistence Fix:** The `handle_restart` system must query `Entity, With<Projectile>` and call `despawn_recursive()` to kill the missiles and their attached audio children simultaneously.
3. **Stereo Support:** By using `spatial: false` in `PlaybackSettings` and handling volume via `sink.set_volume()` in code, we allow high-quality Stereo files to function as 3D sources without engine errors.
