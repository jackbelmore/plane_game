╔═══════════════════════════════════════════════════════════════════════════════╗
║           PARALLEL WORK ASSIGNMENT - FLIGHT PHYSICS FIXES                     ║
╚═══════════════════════════════════════════════════════════════════════════════╝

CURRENT STATUS:
  ✅ Camera fix - DONE (Claude, just now)
  ⏳ Flight physics - CRITICAL (need 2 parallel AI models)
  ⏳ Space scenery - SECONDARY (can wait)

═══════════════════════════════════════════════════════════════════════════════

ASSIGNMENT:

┌─────────────────────────────────────────────────────────────────────────────┐
│ CLAUDE CODE SESSION 1: FIX THRUST + TURNING MECHANICS                       │
│ Priority: CRITICAL (Blocks gameplay)                                        │
│ Time: 45 minutes                                                             │
│ Tasks: Phase 1 + Phase 2 from FLIGHT_PHYSICS_FIX_PLAN.md                    │
└─────────────────────────────────────────────────────────────────────────────┘

PROMPT FOR CLAUDE CODE:

```
TASK: Fix critical flight physics issues in F-16 flight simulator

FILE: C:\Users\Box\plane_game\src\main.rs

PROBLEMS TO SOLVE:

1. PLANE FALLING OUT OF SKY (Lines 823-909, arcade_flight_physics function)
   
   Issue: Plane descends rapidly even at 100% throttle
   
   Root Cause: MAX_THRUST_NEWTONS too low (50,000 N < gravity force 88,290 N)
   
   Fix Required:
   - Line ~843: Change `const MAX_THRUST_NEWTONS: f32 = 50000.0;`
   - To: `const MAX_THRUST_NEWTONS: f32 = 100000.0;`
   - Test: At 50%+ throttle, plane should maintain altitude (not fall)

2. TURNING BROKEN - ROLL + PITCH NOT CREATING TURNS (Lines 870-881)
   
   Issue: Roll plane 90°, then pitch up → plane pitches but doesn't turn
   
   Expected: Plane should curve left/right depending on roll direction
   
   Root Cause: Angular velocity may not be using local-space axes correctly
   
   Current code around line 873-881:
   ```rust
   let target_omega = Vec3::new(
       input.pitch * PITCH_RATE,
       input.yaw * YAW_RATE,
       input.roll * ROLL_RATE,
   );
   ```
   
   Check if this should be:
   ```rust
   let forward = transform.forward().as_vec3();
   let right = transform.right().as_vec3();
   let up = transform.up().as_vec3();
   
   let target_omega = 
       right * input.pitch * PITCH_RATE +
       up * input.yaw * YAW_RATE +
       forward * input.roll * ROLL_RATE;
   ```
   
   Fix: Verify axes are truly local-space and apply correctly

VERIFICATION TESTS:
- Test 1: Throttle 50% → plane maintains altitude
- Test 2: Throttle 75% → plane slowly climbs
- Test 3: Roll left (A) 90° → no yaw
- Test 4: Roll left + pitch up (S) → plane curves LEFT
- Test 5: Roll right (D) 90° → no yaw
- Test 6: Roll right + pitch up (S) → plane curves RIGHT

BUILD & TEST:
1. cargo build --release (should compile)
2. cargo run --release
3. Test each case above
4. Report: Does plane stay in air? Do turns work?

DELIVERABLE:
- Fixed arcade_flight_physics() function
- Code changes only (don't restructure)
- Brief explanation of changes
- Test results
```

═══════════════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────────────┐
│ GEMINI FLASH: TEST SUITE + PHYSICS DIAGNOSTICS                              │
│ Priority: HIGH (Helps verify fixes)                                         │
│ Time: 30 minutes                                                             │
│ Tasks: Task 3 + Task 4 from PARALLEL_WORK_PROMPTS.md                        │
└─────────────────────────────────────────────────────────────────────────────┘

PROMPT FOR GEMINI FLASH:

```
TASK: Create diagnostic test suite for flight simulator verification

FILE: C:\Users\Box\plane_game\src\main.rs

GOALS:
1. Create test functions to verify flight physics work correctly
2. Add debug output to diagnose issues
3. Document physics constants and their effects

PART 1: DIAGNOSTIC OUTPUT SYSTEM (20 min)

Create new system function: `debug_flight_diagnostics()`

This should print every 0.5 seconds:

FORMAT:
═════════════════════════════════════════════════════════════
FLIGHT DIAGNOSTICS - Frame XXXX
─────────────────────────────────────────────────────────────
ALTITUDE:     XXXXX m  |  CLIMB RATE: XX.X m/s  |  THROTTLE: XX%
SPEED:        XXX m/s  |  POSITION: (X.X, X.X, X.X)
ATTITUDE:     Roll:XX° Pitch:XX° Yaw:XX°
INPUTS:       Pitch:X.X Roll:X.X Yaw:X.X Throttle:X.X
TURNING TEST: Yaw Rate: XX°/s (should increase when roll+pitch)
═════════════════════════════════════════════════════════════

WHAT TO CALCULATE:
- Altitude: transform.translation.y
- Speed: velocity.length()
- Roll/Pitch/Yaw: from transform.rotation (to_euler)
- Climb rate: velocity.y
- Yaw rate: angular_velocity component around Y axis
- Is turning: (roll != 0 and pitch != 0) = should have yaw rate

PART 2: PHYSICS CONSTANTS DOCUMENTATION (10 min)

Create new file: PHYSICS_CONSTANTS_GUIDE.md

Document each constant:
- MAX_THRUST_NEWTONS
- PITCH_RATE
- ROLL_RATE
- YAW_RATE
- BOOST_MULTIPLIER
- DRAG_COEFFICIENT
- SMOOTHING_FACTOR

For each, provide:
1. Current value
2. What it does
3. Physics formula/effect
4. Recommended range (low/medium/high feel)
5. How to adjust in-game feel

Example:
```
MAX_THRUST_NEWTONS: 100000.0
  Purpose: Maximum forward engine thrust
  Physics: F = m*a, so thrust must exceed gravity (9000 kg * 9.81 = 88,290 N)
  Effect: Higher = faster climbing, easier to maintain altitude
  Range: 50000 (sluggish) → 100000 (current) → 200000 (overpowered)
  Adjustment: For arcade fighter feel, keep at 100000+
```

IMPLEMENTATION:
1. Add system to main schedule (lines ~290)
2. Use Local<f32> for time tracking
3. Print to console (println!)
4. Document in code with comments

DELIVERABLE:
- New debug_flight_diagnostics() system
- PHYSICS_CONSTANTS_GUIDE.md file
- Lines to add to schedule
- Clear output format for debugging
```

═══════════════════════════════════════════════════════════════════════════════

COORDINATION:

Work Start: Both in parallel
Target Completion: 45 minutes
Handoff: When both complete, integrate and test together

Claude Code: Focuses on physics fixes (gameplay critical)
Gemini Flash: Focuses on diagnostics (helps verify Claude's work)

After both complete:
1. Merge fixes
2. Run diagnostics
3. Verify: Plane flies, turns work, altitude maintained
4. If diagnostics show issues, fix immediately
5. Then proceed to space scenery

═══════════════════════════════════════════════════════════════════════════════
