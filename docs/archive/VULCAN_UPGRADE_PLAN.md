# ViperEye: Vulcan Cannon (Machine Gun) Upgrade Plan

This document outlines the strategy for modernizing the Vulcan Cannon's combat effectiveness and visual fidelity.

## Phase 1: Continuous Collision Detection (CCD)
**Goal:** Eliminate "tunneling" where high-speed bullets skip over targets between frames.

### 1. Component Update
- Add `previous_translation: Vec3` to the `Bullet` struct in `src/main.rs`.
- Initialize it to the spawn position in `spawn_bullet`.

### 2. Logic Implementation
- **Segment Calculation:** In `bullet_drone_collision`, calculate the line segment from `previous_translation` to current `transform.translation`.
- **Segment-Point Distance:** Use a point-to-line-segment distance formula to check if a drone (point) was passed by the bullet during the last frame.
- **State Management:** In `update_bullets`, update `previous_translation` to the current `translation` *after* collision checks or at the start of the next frame to maintain a valid trail.

## Phase 2: Super Tracer & Visual Polish
**Goal:** Improve tactical feedback and cinematic "feel."

### 1. Shot Counting
- Add `shot_count: u32` to `MachineGunState`.
- Every 5th shot triggers the `Super Tracer` logic.

### 2. Red Super Tracers
- **Visuals:** Red core, 2x brightness, slightly larger mesh than standard rounds.
- **Heat Trail:** Spawns short-lived (0.1s) red `VisualDebris` particles along its path every frame.
- **Impact:** Unique red/orange hit sparks for tracer hits.

### 3. Audio & Feel
- Slight pitch randomization for tracer shots.
- Increase damage of the tracer round specifically to reward sustained fire.

## Phase 3: Impact & Feedback
- Enhance `spawn_hit_spark` with "sparkling" embers that have slight gravity.
- Dynamic damage tuning: Bullets do more damage if the closing velocity is higher.
