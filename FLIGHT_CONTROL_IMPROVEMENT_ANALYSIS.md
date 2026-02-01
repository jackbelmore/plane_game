# Flight Control System: Deep Analysis & Improvement Map

This doc identifies **which parts of our flight control need improving** and **which open-source repo has the best solutions**. Use it with `FLIGHT_CONTROL_OPEN_SOURCE_REFERENCE.md` for implementation.

---

## Current Architecture (Summary)

| Component | What we do | Where in code |
|-----------|------------|----------------|
| **Input** | W/S pitch, A/D roll, Q/E yaw, Shift/Ctrl throttle. Raw → lerp to `PlayerInput` (0.1). Throttle: ±0.01 per frame, no target. | `read_player_input` |
| **Physics** | Arcade only. `target_omega = right*pitch*PITCH_RATE + up*yaw*YAW_RATE + forward*roll*ROLL_RATE`; `ang_vel.0 = lerp(ang_vel.0, target_omega, SMOOTHING_FACTOR)`. Thrust from throttle with pitch decomposition + boost >80%. Quadratic drag. | `arcade_flight_physics` |
| **FBW** | Present but **not in schedule** (`fly_by_wire_control` is `#[allow(dead_code)]`). PID pitch/roll, auto-level when no input. | `fly_by_wire_control`, `FlightControlComputer` |
| **Visual bank** | **None.** `ModelContainer` is never updated; parent (physics) rotation drives everything. No separate “wing bank” visual. | `ModelContainer` only used at spawn |
| **Throttle** | No “target throttle” vs “actual throttle”; `input.throttle` used directly. Step ±0.01 per frame. | `read_player_input`, `arcade_flight_physics` |
| **Bank-induced yaw** | **None.** Banking does not add yaw; only explicit Q/E does. | — |
| **Auto-level** | **None.** Releasing stick just lerps input to 0 → target_omega → 0, so rotation stops. We do **not** pull roll/pitch back to level. | — |
| **Grounded behavior** | Ground collision + respawn only. No “min flight speed”, “no bank on ground”, or “pitch up only above min speed”. | `check_ground_collision` |
| **Camera** | Behind plane, offset (0, 5, 15); smooth slerp to look at plane. | `update_flight_camera` |

---

## Parts That Need Improving (Prioritized)

### 1. **Throttle feel** — Medium impact, easy win

**Issue:** Throttle is stepped ±0.01 per frame and applied directly. No smooth acceleration/deceleration; speed changes feel abrupt.

**Best repo:** **Godot recipe** + **ArcadeJetFlightExample (Unity)**  
**Idea:**  
- Maintain **target throttle** (from input) and **actual throttle** (used for thrust).  
- Each frame: `actual_throttle = lerp(actual_throttle, target_throttle, acceleration * dt)`.  
- Use actual throttle for thrust; optionally keep raw throttle for HUD/particles.  
**Tunable:** “Throttle response” rate (how fast actual catches up to target).

---

### 2. **No auto-level when stick released** — High impact on “plane feel”

**Issue:** When you release A/D or W/S, we only lerp input to 0, so we stop commanding rotation. We do **not** pull the plane back to level flight. Result: you can stay banked or nose-up/nose-down indefinitely.

**Best repo:** **Godot recipe** (“level_speed”) and **FlyDangerous** (plane-fixed blend)  
**Ideas:**  
- **Simple:** When `input.roll` and `input.pitch` are near zero, add a small “leveling” torque that pulls current roll (and optionally pitch) toward 0. E.g. roll_torque -= current_roll * LEVEL_GAIN.  
- **Advanced (FlyDangerous):** Blend between “plane-fixed” (roll = 0, free yaw/pitch) and “free rotation” based on angle from horizontal; near level = more pull to wings level.  
**Tunable:** Leveling strength (or blend angles).

---

### 3. **Bank-induced yaw (turn from bank)** — High impact on realism/fun

**Issue:** In real flight (and many arcade games), banking right tends to turn the nose right. We only turn with explicit Q/E yaw. Banking feels “flat” — you roll but don’t get a natural turn.

**Best repo:** **ArcadeJetFlightExample (Unity)**  
**Idea:** Add yaw torque proportional to “wing height”: e.g. `bank_factor = transform.right().y` (how high the right wing is). When banked right, bank_factor > 0 → add yaw torque to the right. Magnitude can scale with speed.  
**Tunable:** Bank-to-yaw gain; optional speed scaling.

---

### 4. **Visual bank vs physics roll** — Medium impact (optional)

**Issue:** We have a `ModelContainer` child but never change its rotation. The whole body (physics + visual) rolls together. Some games exaggerate or smooth **visual** bank (mesh) separately from physics for better readability.

**Best repo:** **Godot recipe**  
**Idea:** Keep physics as-is (or add bank-induced yaw). Add a system that drives **only** the ModelContainer’s local roll (e.g. Z) from `input.roll` or current roll: `mesh_roll = lerp(mesh_roll, -turn_input, level_speed * dt)` so wings bank with turn and autolevel when straight. Physics body can stay more level (if you add auto-level) while the model looks banked.  
**Tunable:** Bank amount and level_speed.  
**Note:** Doing both “physics auto-level” and “visual bank only” gives an Ace Combat–style feel: plane tends to level, but the model clearly banks in turns.

---

### 5. **Grounded vs airborne behavior** — Medium impact (consistency)

**Issue:** We don’t distinguish “on ground” from “in flight” for control rules. No min flight speed, no “can’t pitch up until above min speed”, no “no bank on ground”.

**Best repo:** **Godot recipe**  
**Idea:**  
- Track “grounded” (e.g. from collision or altitude < threshold + low vertical speed).  
- When grounded: allow throttle to go to 0; zero or reduce bank; optionally level pitch.  
- When airborne: enforce min flight speed (throttle can’t go below a value that keeps speed above min); pitch up only if speed >= min_flight_speed.  
**Tunable:** Min flight speed, ground threshold, bank damping on ground.

---

### 6. **Input mapping and smoothing** — Lower priority

**Issue:** Pitch/roll/yaw use lerp(0.1) to target ±1. No invert option, no gamepad curve, no dead zone (could add in input layer).

**Best repo:** **FlyDangerous** (invert Y, clamp, remap); **Godot** (input actions).  
**Idea:** Optional invert pitch; optional dead zone and exponent curve for stick; keep lerp but make factor tunable (or use different curve).  
**Tunable:** Invert Y, dead zone, smoothing factor.

---

### 7. **FBW / “sim-lite” mode** — Design choice

**Issue:** We have a full FBW (PID pitch/roll, auto-level) implementation but it’s **not in the schedule**. Only arcade physics runs.

**Options:**  
- **A)** Remove or gate FBW behind a “sim mode” so we don’t maintain dead code.  
- **B)** Add a mode switch (e.g. key or menu): “Arcade” vs “Sim-lite” (FBW on, same input, different response). Then tune FBW for that mode.  
**Best repo for FBW tuning:** JSBSim / FlightGear for concepts; our existing PID is already “sim-lite”.

---

## Repo → Improvement Map

| Repo | Best for |
|------|----------|
| **Godot recipe (KidsCanCode)** | Throttle target/actual + lerp; auto-level (level_speed); grounded vs airborne (min speed, no bank on ground); visual bank only (mesh.rotation.z). |
| **ArcadeJetFlightExample (Unity)** | Smooth target vs actual throttle; **bank-induced yaw** (right.y → yaw torque); high drag + smooth throttle. |
| **FlyDangerous (Unity)** | Auto-level / plane-fixed blend (blend by angle from horizontal); invert Y; remap stick to pitch/yaw/roll; optional “drift” mode. |
| **JSBSim / FlightGear** | Only if we add a “sim” mode or want formulas for lift/drag/moments; not needed for arcade feel. |

---

## Suggested order of work (if you want all of the above)

1. **Throttle feel** (target + lerp) — quick, noticeable.  
2. **Auto-level** (pull roll/pitch toward 0 when stick released) — big improvement in “plane” feel.  
3. **Bank-induced yaw** — makes banking feel useful and natural.  
4. **Grounded vs airborne** — min speed, no bank on ground, etc.  
5. **Visual bank (ModelContainer)** — optional polish.  
6. **Input polish** — dead zone, invert, tunable smoothing.  
7. **FBW** — decide: remove, or add as sim-lite mode and tune.

---

## Questions for you (to narrow scope)

1. **Primary goal:** More “arcade fun” (tighter, more responsive, Ace Combat–like) or more “plane-like” (bank turns you, auto-level, min speed)?  
2. **Throttle:** Does speed feel too on/off, or is it acceptable? (If too on/off → do target/actual throttle first.)  
3. **Releasing stick:** Do you want the plane to **tend to level its wings** when you let go of A/D and W/S, or stay exactly as-is?  
4. **Banking:** Do you want **rolling right to also turn the nose right** (without pressing Q/E)?  
5. **Grounded:** Do you care about “can’t take off below X speed” / “on ground = no bank” / “throttle to 0 on ground”?  
6. **FBW:** Prefer to **remove** the unused FBW code, or **keep and add a Sim-lite mode** (toggle) later?  
7. **Visual only:** Do you want the **3D model** to bank more than the physics (e.g. wings tilt more in turns) for readability?

Answering these will pin down which improvements to implement first and which repo to lean on for each.
