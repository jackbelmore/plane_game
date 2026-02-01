# Camera & Turning Mechanics - Diagnostic Prompt

## Issues to Fix

### Issue 1: Camera Not Following Plane Correctly
**Problem**: Camera should stay behind plane at all times, but seems to lag or position incorrectly during flight

**Location**: `src/main.rs:966-1003` - `update_flight_camera()` function

**Current Behavior**:
- Camera translates to desired position (behind plane)
- Camera looks at plane and rotates
- But camera may not be maintaining proper offset during maneuvers

**Expected Behavior**:
- Camera always positioned 15 units behind plane
- Camera always looking at plane's center
- Smooth follow during rolls/pitches
- No clipping into plane or terrain

---

### Issue 2: Turning Not Working Properly
**Problem**: When rolling plane (A/D) then pulling up (S), plane doesn't turn around the map correctly

**Location**: `src/main.rs:870-887` - Local-space rotation application

**Current Implementation**:
```rust
let right = transform.right().as_vec3();
let up = transform.up().as_vec3();
let forward = transform.forward().as_vec3();

let target_omega = right * input.pitch * PITCH_RATE +
                  up * input.yaw * YAW_RATE +
                  forward * input.roll * ROLL_RATE;
```

**Expected Behavior**:
- Roll left (A): Plane tips left wing down ✓ (probably working)
- While rolled left, pitch up (S): Plane should curve left (yaw left) ✓ (THIS IS BROKEN)
- Roll right (D): Plane tips right wing down ✓ (probably working)
- While rolled right, pitch up (S): Plane should curve right (yaw right) ✓ (THIS IS BROKEN)

---

## Diagnostic Steps

### For Camera Issue

**Step 1: Check Position**
- Fly the plane forward
- Does camera stay directly behind?
- Does camera move away when rolling?
- Does camera move away when pitching?

**Step 2: Check Rotation**
- Roll the plane 90° left (hold A)
- Does camera follow the roll (tilt with plane)?
- Does camera stay looking at plane?

**Step 3: Check Speed Offset**
- Accelerate with Shift
- Does camera pull back more at high speed?
- Or does it stay fixed distance?

**Step 4: Debug Output**
Add to `update_flight_camera()`:
```rust
if *debug_counter % 60 == 0 {
    println!("Camera Pos: {:?}", camera_transform.translation);
    println!("Plane Pos: {:?}", player_transform.translation);
    println!("Offset: {:?}", camera_transform.translation - player_transform.translation);
    println!("Camera Rot: {:?}", camera_transform.rotation);
}
```

---

### For Turning Issue

**Step 1: Roll Only**
- Hold A (roll left)
- Watch plane tip left
- Plane should NOT yaw/turn (only roll)
- Release A, plane should level

**Step 2: Pitch Only**
- Hold S (pitch up)
- Plane should pitch up nose
- Plane should NOT yaw/turn (only pitch)
- Should climb/loop

**Step 3: Roll + Pitch (THE BROKEN PART)**
- Hold A (roll left) - wait 1 second
- While rolled, hold S (pitch up)
- EXPECTED: Plane curves left (left yaw)
- ACTUAL: Plane probably just pitches up without turning left

**Step 4: Debug Output**
Add to `arcade_flight_physics()`:
```rust
if *debug_counter % 30 == 0 {
    println!("=== ROTATION DEBUG ===");
    println!("Inputs: pitch={:.2} roll={:.2} yaw={:.2}",
        input.pitch, input.roll, input.yaw);

    let (_, pitch_angle, roll_angle) = transform.rotation.to_euler(EulerRot::XYZ);
    println!("Angles: pitch={:.2}° roll={:.2}°",
        pitch_angle.to_degrees(), roll_angle.to_degrees());

    println!("Target Omega: {:?}", target_omega);
    println!("Actual Omega: {:?}", ang_vel.0);

    // Check axes
    println!("Right: {:?}", right);
    println!("Up: {:?}", up);
    println!("Forward: {:?}", forward);
}
```

---

## Likely Root Causes

### Camera Issue (Most Likely)
1. **Speed offset calculation** - May be pulling camera in wrong direction at high speeds
2. **Look target** - May be looking at wrong point (not plane center)
3. **Rotation lag** - Slerp interpolation may be too slow or in wrong direction
4. **Up vector** - Camera up vector may not match plane orientation

**Fix Strategy**:
- Verify camera position = plane position + (plane's local up × 5) + (plane's local back × 15)
- Verify look target = plane center
- Simplify rotation (remove lag temporarily to test)

### Turning Issue (Most Likely)
1. **Axes are in world space, not local space** - The `transform.right()` etc may not be giving local axes
2. **Rotation order matters** - Order of axis applications might be wrong
3. **Gimbal lock** - At 90° pitch, axes might flip
4. **Angular velocity interpretation** - Avian3D might interpret omega differently

**Fix Strategy**:
- Verify `transform.right()`, `transform.up()`, `transform.forward()` are LOCAL axes
- Add debug output to see actual rotation matrices
- Test simple cases (roll only, pitch only) first
- May need to use transform's rotation matrix directly

---

## Test Flight Pattern

1. **Hover at altitude** - 0 input, just maintain altitude
2. **Roll left 90°** - Hold A for 2 seconds
3. **Pitch up** - Hold S while rolled
4. **Observe**:
   - Does plane turn left (map terrain rotates to right)?
   - Or does plane just pitch up while staying in same location?
5. **Repeat right** - Roll right (D) + pitch up (S)

---

## Quick Fix to Try First

If turning is broken, try this modification to `arcade_flight_physics()`:

**Current (broken?)**:
```rust
let target_omega = right * input.pitch * PITCH_RATE +
                  up * input.yaw * YAW_RATE +
                  forward * input.roll * ROLL_RATE;
```

**Try this instead**:
```rust
// Convert input to world-space angular velocity
let target_omega = (right * input.pitch * PITCH_RATE) +
                  (up * (input.yaw * YAW_RATE + input.roll * input.pitch * 2.0)) +
                  (forward * input.roll * ROLL_RATE);
// Note: Added roll*pitch coupling for turn assistance
```

Or try using `AngularVelocity` with world-space directly:
```rust
ang_vel.0 = (input.pitch * PITCH_RATE) * transform.right().as_vec3() +
           (input.yaw * YAW_RATE) * transform.up().as_vec3() +
           (input.roll * ROLL_RATE) * transform.forward().as_vec3();
```

---

## Questions to Answer

1. **Camera**: Does camera stay at fixed distance from plane, or move when plane maneuvers?
2. **Turning**: When rolled and pitching up, does map terrain rotate or stay static?
3. **Both**: Are you seeing any error messages in console?
4. **Both**: Does it work better in release build vs debug?

---

## Next Session

When fixing, provide:
- Screenshot of plane rolled 90° with camera view
- Debug output from one roll+pitch maneuver
- Confirm which issue is worse (camera or turning)
