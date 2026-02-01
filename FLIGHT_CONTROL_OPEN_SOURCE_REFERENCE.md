# Open Source Flight Control Reference for Similar Video Games

Curated links and code patterns from open source flight / arcade airplane games. Use these to improve or compare flight control in this Bevy + avian3d plane game.

---

## 1. Arcade-style airplane (Godot) – KidsCanCode

**Best for:** Simple, readable arcade flight; throttle + pitch + turn + bank.

- **Link:** [Arcade-style Airplane (Godot 4)](https://kidscancode.org/godot_recipes/4.x/3d/simple_airplane/index.html)
- **Repo:** [godotrecipes/3d_airplane_demo](https://github.com/godotrecipes/3d_airplane_demo)
- **Approach:** `CharacterBody3D` (no full physics). Target speed vs actual speed, lerp throttle. Pitch/turn via `transform.basis.rotated()`. **Bank = mesh roll only** (visual), turn = yaw; “wings autolevel” when not turning.

**Key patterns (concepts to port to Bevy):**

- **Throttle:** `target_speed` from input; `forward_speed = lerp(forward_speed, target_speed, acceleration * delta)`.
- **Pitch:** `transform.basis = transform.basis.rotated(transform.basis.x, pitch_input * pitch_speed * delta)`.
- **Turn (yaw):** `transform.basis = transform.basis.rotated(Vector3.UP, turn_input * turn_speed * delta)`.
- **Bank (visual only):** `mesh.rotation.z = lerp(mesh.rotation.z, -turn_input, level_speed * delta)` so wings bank with turn and autolevel when straight.
- **Ground:** When grounded, no bank; throttle can go to 0; pitch down only when airborne; takeoff only above `min_flight_speed`.

**Tunables:** `min_flight_speed`, `max_flight_speed`, `turn_speed`, `pitch_speed`, `level_speed`, `throttle_delta`, `acceleration`.

---

## 2. FlyDangerous – ShipArcadeFlightComputer (Unity C#)

**Best for:** Blending “plane-fixed” (yaw/pitch, roll to 0) vs “free rotation”; invert-Y; auto-roll from yaw.

- **Link:** [ShipArcadeFlightComputer.cs](https://github.com/jukibom/FlyDangerous/blob/main/Assets/Scripts/Core/Player/ShipArcadeFlightComputer.cs)
- **Approach:** Separate **target** transform (where the player “wants” to go) and **ship** transform. Blend between “plane-fixed” (roll = 0, free yaw/pitch) and “free” rotation based on ship angle from horizontal. Input remaps to pitch/yaw/roll deltas; optional auto-roll from yaw.

**Key patterns:**

- **Blend by angle from horizontal:** `shipAngleFromPlane = abs(deltaAngle(0, shipRotEuler.x))`. Blend factor: `remap(fixedToPlaneAngle, freeMoveAngle, planeTransformDamping, 1)` so near-level = more “plane” behavior, steep = more free rotation (avoids pole weirdness).
- **Plane rotation:** `planeRotation = Quaternion.Euler(shipRotEuler.x, shipRotEuler.y, 0)` (zero roll). Lerp `planeRotation` and `freeRotation` by blend factor.
- **Input → rotation deltas:** `pitchRotate = lateralV.Remap(-1, 1, -maxTargetRotationDegrees, maxTargetRotationDegrees)`, same for yaw; `rollRotate = autoShipRoll ? -yawRotate : 0`.
- **Output:** Overwrite lateralH, lateralV, throttle from target position; add to pitch, yaw, roll from target rotation deltas. “Drift” mode disables lateral/auto-roll.

**Tunables:** `fixedToPlaneAngle`, `freeMoveAngle`, `planeTransformDamping`, `maxTargetRotationDegrees`, `autoShipRoll`.

---

## 3. Arcade Jet Flight Example (Unity)

**Best for:** Rigidbody-based arcade jet; high drag + smooth throttle; bank-induced yaw.

- **Link:** [brihernandez/ArcadeJetFlightExample](https://github.com/brihernandez/ArcadeJetFlightExample)
- **Approach:** Full Rigidbody; high linear drag (e.g. 5) so velocity aligns with nose quickly; **target throttle** vs **actual throttle** smoothed by an “acceleration” value; **bank force:** yaw torque from wing height (transform.right.y) so banking turns the jet.

**Key patterns:**

- **Smooth throttle:** Target throttle = input; true throttle lerps toward target at a rate (acceleration). Prevents instant speed jumps with high drag.
- **Bank-induced yaw:** Use “right” vector Y (how high the right wing is) as bank factor; apply yaw torque in that direction. At 90° bank, full yaw; at level, no extra yaw. Mimics “bank and turn” without full aero.
- **High drag:** e.g. 5, so plane doesn’t slip much and feels tight; forces must be smooth (hence throttle smoothing).

---

## 4. JSBSim (C++ flight dynamics)

**Best for:** Realistic sims, not arcade; reference only for lift/drag/moments.

- **Link:** [JSBSim](https://jsbsim.sourceforge.net/) | [GitHub: JSBSim-Team/jsbsim](https://github.com/JSBSim-Team/jsbsim)
- **Used by:** FlightGear, Outerra, OpenEaagles. XML-configurable aero, propulsion, FCS.
- **Use here:** Only if you later add “sim-like” mode or want formulas for lift/drag; not needed for arcade feel.

---

## 5. Bevy / Rust specific

- **flight-sim-bevy-rust:** [andoco/flight-sim-bevy-rust](https://github.com/andoco/flight-sim-bevy-rust) – Bevy + Rapier flight sim; good to see how another Bevy project structures flight + physics.
- **bevy-flight:** [hail-faro/bevy-flight](https://github.com/hail-faro/bevy-flight) – Another Bevy flight project.
- **bevy_flights (crate):** [docs.rs/bevy-flights](https://docs.rs/bevy-flights/latest/bevy_flights) – For complex flight paths (e.g. danmaku), not direct control replacement.
- **Avian:** You already use avian3d for physics; no extra “flight” crate required for basic control.

---

## 6. Gamedev Stack Exchange / Forums

- **Simplified aerodynamics 3D airplane:** [gamedev.stackexchange.com/questions/186777](https://gamedev.stackexchange.com/questions/186777/simplified-aerodynamics-for-3d-airplane) – Pitch/roll/yaw, quaternions, local vs world.
- **Controlling pitch, yaw, roll:** [gamedev.net/forums/topic/668911](https://www.gamedev.net/forums/topic/668911-controlling-pitch-yaw-and-roll-of-an-airplane/) – Prefer quaternions; use relative torque/force (e.g. `AddRelativeTorque`) so inputs are in plane space.

---

## Quick takeaways for this project

| Goal                         | Where to look                    | Idea |
|-----------------------------|----------------------------------|------|
| Throttle feel               | Godot recipe, ArcadeJetExample  | Target speed + lerp to actual; smooth throttle in, high drag optional. |
| Bank = visual only          | Godot recipe                    | Roll only on mesh (ModelContainer); physics body yaw/pitch only. |
| Auto-level / plane-fixed    | FlyDangerous                    | Blend “zero roll” vs free rotation by angle from horizontal. |
| Bank-induced turn           | ArcadeJetExample                | Yaw torque from `transform.right().y` (wing height). |
| No gimbal lock              | Gamedev.net/SE                  | Use quaternions; apply torques in local (relative) space. |
| Grounded vs airborne       | Godot recipe                    | Different min speed, no bank on ground, pitch/takeoff rules. |

Use this doc as a prompt or reference when asking Claude (or yourself) to “improve flight control using patterns from similar open source games.”