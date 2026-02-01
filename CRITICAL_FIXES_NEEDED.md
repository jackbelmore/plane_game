# Critical Physics Fixes Needed

## Issue 1: Plane Falls Even at Full Throttle ⚠️

**Location**: `src/main.rs:916-926` - Vertical thrust calculation

**Root Cause Analysis**:
```rust
let (_, pitch_angle, _) = transform.rotation.to_euler(EulerRot::XYZ);
```

Problems:
1. **Euler angle order mismatch**: Using XYZ order but should probably use YXZ
2. **Diagnostic uses different order**: Gemini's diagnostic uses `EulerRot::YXZ` (line 1032)
3. **Pitch angle interpretation**: After rolled 90°, the pitch angle from XYZ might not represent actual pitch to horizontal
4. **Coordinate space confusion**: Local `up` vector isn't pure world-space vertical after rotations

**Expected Behavior**:
- At full throttle + level flight: Thrust = 100kN horizontal + some vertical component
- At full throttle + 45° pitch up: Should have vertical component to overcome gravity
- Gravity = mass × g = 9000 kg × 9.81 = ~88,290 N downward
- Max vertical thrust needed: >88,290 N to climb

**Verification from Diagnostic**:
The new diagnostic system shows:
- CLIMB: should be positive when pitched up + full throttle
- If CLIMB is negative or zero at full throttle, vertical thrust is broken

---

## Issue 2: Turning Doesn't Work (Roll + Pitch ≠ Turn) ⚠️

**Location**: `src/main.rs:893-905` - Local-space rotation

**Current Code**:
```rust
let right = transform.right().as_vec3();
let up = transform.up().as_vec3();
let forward = transform.forward().as_vec3();

let target_omega = right * input.pitch * PITCH_RATE +
                  up * input.yaw * YAW_RATE +
                  forward * input.roll * ROLL_RATE;
```

**Problem**:
This LOOKS correct but might not be working because:
1. **Axes might be wrong**: After various rotations, `right/up/forward` might not be what we expect
2. **Rotation accumulation**: Multiple frames of rotation might cause gimbal lock or axis flip
3. **Smoothing hiding the problem**: `lerp(target_omega, 0.15)` means it takes multiple frames to respond

**Test Cases**:
1. **Roll only** (hold A): Plane tips left - should see ROLL value change but no YAW
2. **Pitch only** (hold S): Plane pitches up - should see PITCH value change but no YAW
3. **Roll + Pitch** (hold A then S):
   - Expected: YAW rate increases (plane turns left)
   - Actual: YAW rate stays near zero (no turn)

**Diagnostic Output to Watch**:
```
INPUTS: [Pitch: 1.0][Roll: -1.0][Yaw: 0.0]...
YAW Rate: 0.0°/s  <-- Should be positive when rolling left + pitching up!
```

If YAW Rate stays 0 during roll+pitch, then local-space rotation is broken.

---

## Recommended Fixes (In Order)

### Fix 1: Unify Euler Angle Extraction (5 min)
```rust
// Change THIS:
let (_, pitch_angle, _) = transform.rotation.to_euler(EulerRot::XYZ);

// To THIS (match diagnostic):
let (yaw_angle, pitch_angle, roll_angle) = transform.rotation.to_euler(EulerRot::YXZ);
```

### Fix 2: Simplify Thrust Model (10 min)
Replace complex decomposition with simpler approach:
```rust
// Instead of sin/cos decomposition:
// Just apply thrust forward + bonus vertical if pitched up
let forward = transform.forward().as_vec3();
let up = Vec3::Y;  // World-space up, not local!

let forward_thrust = forward * input.throttle * MAX_THRUST_NEWTONS;
let gravity_assist = if input.pitch > 0.1 {  // Only if pitching up
    up * (input.pitch * input.throttle * MAX_THRUST_NEWTONS * 0.5)
} else {
    Vec3::ZERO
};

let thrust_force = (forward_thrust + gravity_assist) * boost_mult;
```

This is more intuitive: forward thrust + extra lift when pitching up

### Fix 3: Debug Rotation Application (10 min)
Add debug output to see what's happening:
```rust
if frame_count % 30 == 0 {
    println!("Rotation Matrix:");
    println!("  right: {:?}", right);
    println!("  up: {:?}", up);
    println!("  forward: {:?}", forward);
    println!("Target Omega: {:?}", target_omega);
    println!("Applied Omega: {:?}", ang_vel.0);
}
```

### Fix 4: Check for Gimbal Lock (if axes debug shows problem)
If rotation axes seem wrong at certain angles, may need to use quaternion-based rotation instead.

---

## Testing with Diagnostic System

Gemini's new `debug_flight_diagnostics()` will print every 0.5 seconds:

```
═════════════════════════════════════════════════════════════════════
FLIGHT DIAGNOSTICS
─────────────────────────────────────────────────────────────────────
ALT: 500.0 m  |  CLIMB: 0.0 m/s  |  SPEED: 100 m/s  |  THR: 0%
ROLL: 0.0°  |  PITCH: 0.0°  |  YAW: 0.0°
INPUTS: [Pitch: 0.0][Roll: 0.0][Yaw: 0.0][Throttle: 0.0]
─────────────────────────────────────────────────────────────────────
TURNING: NO   Yaw Rate: 0.0°/s
═════════════════════════════════════════════════════════════════════
```

**What to Watch**:
1. CLIMB should be positive when throttle at 100 + pitch > 0
2. YAW Rate should be positive when rolling + pitching

---

## Quick Win: Build & Test Now

```bash
cd C:\Users\Box\plane_game
cargo build
cargo run
```

Test:
1. Hold Shift for 3 sec (throttle to 100%)
2. Press S to pitch up
3. Watch console for:
   - CLIMB: should increase (positive value)
   - If CLIMB stays 0 or negative → vertical thrust broken

Then test turning:
1. Hold A (roll left)
2. While rolled, press S (pitch up)
3. Watch console for:
   - YAW Rate: should become positive
   - If YAW Rate stays 0 → turning broken

---

## Files to Check

- `src/main.rs`:
  - Lines 916-926: Vertical thrust calculation
  - Lines 893-905: Rotation application
  - Lines 1004-1056: Diagnostic output (review to understand what's showing)

- `PHYSICS_CONSTANTS_GUIDE.md`: Created by Gemini, explains tuning values

---

## Priority

1. **Fix falling issue first** (blocks all testing)
2. **Then fix turning** (needed for gameplay)
3. **Flame effect later** (visual polish, not critical)
