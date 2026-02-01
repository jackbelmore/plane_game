# Physics Constants Tuning Guide

This guide documents the physics constants used in the F-16 Flight Simulator's arcade physics engine. These constants define the aircraft's maneuverability, speed, and overall "feel".

## MAX_THRUST_NEWTONS

**Current Value**: 100000.0

**Purpose**: Defines the base engine thrust force applied in the forward direction.

**Physics Formula**: `F = ma`. Acceleration = `Thrust / Mass`.
At 9000kg, 100,000 N provides ~11.1 m/s² acceleration (slightly more than 1G).

**Effect**: Determines how quickly the plane reaches top speed and its ability to climb vertically.

**Tuning Range**:
- Low Feel (sluggish): 50000.0
- Medium Feel (current): 100000.0 ← Recommended
- High Feel (overpowered): 200000.0

**How to Adjust**: Increase for faster acceleration and better vertical performance. Decrease for a heavier, underpowered feel.

---

## PITCH_RATE

**Current Value**: 1.8

**Purpose**: Sets the maximum angular velocity for pitch (nose up/down).

**Physics Formula**: Target rotation rate in radians per second.
`1.8 rad/s ≈ 103°/s`.

**Effect**: Controls how "tight" the plane's loops are.

**Tuning Range**:
- Low Feel (heavy): 1.0
- Medium Feel (current): 1.8 ← Recommended
- High Feel (twitchy): 3.6

**How to Adjust**: Increase for tighter turns and faster nose response.

---

## ROLL_RATE

**Current Value**: 2.5

**Purpose**: Sets the maximum angular velocity for roll (banking).

**Physics Formula**: Target rotation rate in radians per second.
`2.5 rad/s ≈ 143°/s`.

**Effect**: Controls how quickly the aircraft can transition from one bank angle to another.

**Tuning Range**:
- Low Feel (sluggish): 1.5
- Medium Feel (current): 2.5 ← Recommended
- High Feel (arcade): 4.5

**How to Adjust**: Increase for snap-rolls and high-speed dogfighting agility.

---

## YAW_RATE

**Current Value**: 1.2

**Purpose**: Sets the maximum angular velocity for yaw (nose left/right).

**Physics Formula**: Target rotation rate in radians per second.
`1.2 rad/s ≈ 69°/s`.

**Effect**: Controls rudder authority. Typically lower than pitch/roll for aircraft stability.

**Tuning Range**:
- Low Feel (weak rudder): 0.5
- Medium Feel (current): 1.2 ← Recommended
- High Feel (drifty): 2.0

**How to Adjust**: Increase for more authority during low-speed maneuvers or crosswind correction.

---

## BOOST_MULTIPLIER

**Current Value**: 3.5

**Purpose**: Multiplies engine thrust when the throttle exceeds the boost threshold (currently 80%).

**Physics Formula**: `Effective Thrust = MAX_THRUST * throttle * BOOST_MULTIPLIER`.

**Effect**: Provides an "Afterburner" effect for rapid acceleration and escaping danger.

**Tuning Range**:
- Low Feel (mild): 2.0
- Medium Feel (current): 3.5 ← Recommended
- High Feel (hypersonic): 6.0

**How to Adjust**: Increase for a more dramatic speed difference when holding Shift.

---

## DRAG_COEFFICIENT

**Current Value**: 0.1

**Purpose**: Simulates air resistance to prevent infinite speed and provide stability.

**Physics Formula**: `Drag Force = -Velocity * Speed * Coefficient`.

**Effect**: Acts as a speed limiter and makes the controls feel "glued" to the air.

**Tuning Range**:
- Low Feel (floaty): 0.05
- Medium Feel (current): 0.1 ← Recommended
- High Feel (sticky): 0.3

**How to Adjust**: Increase to lower the top speed and make the plane stop faster when throttle is cut.

---

## SMOOTHING_FACTOR

**Current Value**: 0.15

**Purpose**: Controls the Lerp (Linear Interpolation) speed between current rotation and target input.

**Physics Formula**: `Current AngVel = Lerp(Current, Target, SMOOTHING_FACTOR)`.

**Effect**: Higher values make the plane react instantly; lower values add a sense of mass and momentum.

**Tuning Range**:
- Low Feel (massive): 0.05
- Medium Feel (current): 0.15 ← Recommended
- High Feel (raw): 0.4

**How to Adjust**: Decrease to make the plane feel heavier and more realistic. Increase for arcade-perfect responsiveness.

---

## Quick Reference Table

| Constant | Current | Low Feel | Medium | High Feel |
|----------|---------|----------|--------|-----------|
| MAX_THRUST | 100000 | 50000 | 100000 | 200000 |
| PITCH_RATE | 1.8 | 1.0 | 1.8 | 3.6 |
| ROLL_RATE | 2.5 | 1.5 | 2.5 | 4.5 |
| YAW_RATE | 1.2 | 0.5 | 1.2 | 2.0 |
| BOOST_MULT | 3.5 | 2.0 | 3.5 | 6.0 |
| DRAG | 0.1 | 0.05 | 0.1 | 0.3 |
| SMOOTHING | 0.15 | 0.05 | 0.15 | 0.4 |

---

## Performance Profiles

### For Arcade Dogfight Feel (Recommended)
- Snappy response, high top speeds, easy loops.
- Use current "Recommended" values.

### For Heavy Interceptor Feel
- Slower rolls, higher top speed, lots of momentum.
- `ROLL_RATE: 1.5`, `SMOOTHING_FACTOR: 0.08`, `MAX_THRUST: 150000`.

### For Realistic/Sim Feel
- Very slow response, requires careful input management.
- `ROLL_RATE: 1.2`, `PITCH_RATE: 0.8`, `SMOOTHING_FACTOR: 0.03`.
