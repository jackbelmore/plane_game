# Parallel Work Prompts - For Haiku, Gemini Flash, or other AI models

## Task 1: Fix Camera Following (Priority: HIGH)
**AI Model**: Copilot Haiku or Claude Haiku
**Time**: 20-30 minutes

### Prompt to Use

```
You are helping debug a Bevy flight simulator camera system.

PROBLEM: Camera doesn't stay properly behind the plane during flight maneuvers.

FILE: C:\Users\Box\plane_game\src\main.rs (Lines 966-1003)

Current code:
- Camera should stay 15 units behind plane
- Camera should look at plane center
- Camera should follow rolls/pitches smoothly

WHAT'S BROKEN:
When the plane rolls (A/D keys) or pitches (W/S keys), the camera doesn't follow correctly.
The camera may lag behind, move sideways, or not rotate with the plane.

YOUR TASK:
1. Analyze the current update_flight_camera() function
2. Identify what's wrong with:
   - Position calculation (should be: plane_pos + plane_local_back*15 + plane_local_up*5)
   - Look target calculation (should be: plane_center)
   - Rotation following (camera should look at plane, respecting plane's up vector)
3. Suggest specific code fixes
4. Include debug output code to verify the fix

CONTEXT:
- Bevy 0.15
- Using Transform, LinearVelocity
- Camera is 3rd person, should follow behind
- Movement speed is 100+ m/s

DELIVERABLE:
- Fixed update_flight_camera() function
- Debug println! statements to verify camera position/rotation
- Explanation of what was wrong and why
```

---

## Task 2: Fix Turning Mechanics (Priority: HIGH)
**AI Model**: Gemini Flash or Claude Haiku
**Time**: 25-35 minutes

### Prompt to Use

```
You are helping fix broken flight turning mechanics in a Bevy flight simulator.

PROBLEM: When the player rolls the plane (left/right) and then pitches up, the plane should turn (yaw), but it doesn't.

FILE: C:\Users\Box\plane_game\src\main.rs (Lines 870-887)

Current implementation uses local-space rotation:
- right * input.pitch + up * input.yaw + forward * input.roll

EXPECTED BEHAVIOR (Bank-to-Turn):
1. Hold A (roll left) → plane tips left, should NOT turn
2. While rolled left, hold S (pitch up) → plane should curve LEFT (yaw left)
3. This is basic aerodynamics: rolled + pitched = turn

ACTUAL BEHAVIOR:
- Plane rolls and pitches fine individually
- But when combined, plane doesn't yaw/turn properly
- Plane just pitches up without turning around the map

YOUR TASK:
1. Review the local-space rotation implementation
2. Identify why roll+pitch doesn't create yaw
3. Check if:
   - Axes are truly local-space (use transform.right/up/forward())
   - Angular velocity is applied correctly
   - Need to add yaw coupling (roll × pitch interaction)
4. Provide fixed code that makes turns work

PHYSICS PRINCIPLE:
In arcade flight, rolling + pitching should naturally create turns because:
- Rolling changes which way "up" is
- Pitching then pulls in that new "up" direction
- This creates yaw/turning motion

DELIVERABLE:
- Fixed arcade_flight_physics() rotation section
- Explanation of local vs world space axes
- Test cases: "roll left + pitch up = turn left"
- Debug output to verify yaw rate when rolling+pitching
```

---

## Task 3: Create Improved Test Suite (Priority: MEDIUM)
**AI Model**: Copilot Haiku
**Time**: 15-20 minutes

### Prompt to Use

```
Create a test/verification system for the flight simulator.

CONTEXT:
- Bevy flight simulator in Rust
- File: C:\Users\Box\plane_game\src\main.rs
- Need to verify: camera following, turning mechanics, climbing

YOUR TASK:
Create a debug/test system that:

1. CAMERA TEST:
   - Print camera position relative to plane every frame
   - Show distance from plane (should be ~15 units)
   - Show angle looking at plane
   - Highlight when camera is NOT behind plane

2. TURNING TEST:
   - Print yaw angle every frame
   - Show yaw rate (degrees/sec)
   - When roll+pitch held, verify yaw rate increases
   - Indicate if turn is working or broken

3. CLIMBING TEST:
   - Print altitude every 0.5 sec
   - Print vertical velocity
   - When pitch up + high throttle, should climb
   - Verify altitude increases

IMPLEMENTATION:
- Add debug_counter (already exists around line 1000+)
- Use Local<f32> for time tracking
- Print to console with clear formatting
- Include actual vs expected values

DELIVERABLE:
- New system function: debug_camera_and_turning()
- Activation: Add to Update systems schedule
- Output format: Clear, easy to read, shows if issues exist
```

---

## Task 4: Analyze Physics Constants (Priority: LOW)
**AI Model**: Gemini Flash
**Time**: 15-20 minutes

### Prompt to Use

```
Analyze and document the flight physics constants in the simulator.

FILE: C:\Users\Box\plane_game\src\main.rs (Lines 830-840)

CURRENT CONSTANTS:
- MAX_THRUST_NEWTONS: 100000.0
- PITCH_RATE: 1.8 rad/s
- ROLL_RATE: 2.5 rad/s
- YAW_RATE: 1.2 rad/s
- BOOST_MULTIPLIER: 3.5x
- DRAG_COEFFICIENT: 0.1
- SMOOTHING_FACTOR: 0.15

YOUR TASK:
1. Convert rates to degrees/sec for readability
2. Create a tuning guide explaining each constant
3. Suggest ranges for adjustment (low, medium, high feel)
4. Add physics formulas showing how they affect:
   - Climb rate
   - Turn radius
   - Response time
   - Energy bleed during turns

5. Create a quick reference table:
   - Parameter | Current | Low Feel | Medium | High Feel
   - PITCH_RATE | 1.8 | 0.9 | 1.8 | 3.6
   - etc.

DELIVERABLE:
- Physics tuning guide with formulas
- Quick reference table
- Recommendations for "arcade dogfight" feel
- Code comments explaining each constant's effect
```

---

## Task 5: Integration with Space Kit Assets (Priority: MEDIUM)
**AI Model**: Claude or Gemini Flash
**Time**: 30-40 minutes

### Prompt to Use

```
Plan space kit asset integration for the flight simulator.

CONTEXT:
- Game: Bevy flight simulator, F-16 jet
- Assets available: E:\Downloads\kenney_space-kit\ (already extracted)
- Goal: Add space environment (skybox, asteroids, stations)
- Physics: Already working, don't break it

YOUR TASK:
1. Examine what's in the space kit:
   - List available models (skybox, asteroids, stations, planets)
   - Identify file formats and sizes

2. Create implementation plan:
   - How to load skybox (position, scale, rotation)
   - How to spawn asteroids as static obstacles
   - How to place space station as landmark
   - Where to position each element

3. Physics considerations:
   - Which objects should have colliders
   - Which should be static (terrain)
   - Which should be visual-only (background)

4. Code locations:
   - setup_scene() function (Line ~314)
   - Where to add asset loading
   - How to spawn entities

5. Performance:
   - Estimate entity count
   - LOD strategy for distant objects
   - Particle effects if needed

DELIVERABLE:
- Asset list from space kit
- Implementation plan (which assets to use, where)
- Code skeleton for loading/spawning
- Collision/physics assignments
- Performance considerations
```

---

## Work Coordination

### Recommended Parallel Work Order

**Phase 1 (Start these in parallel)**:
1. Task 1: Camera fix (Haiku) - Critical for gameplay
2. Task 2: Turning fix (Gemini) - Critical for gameplay
3. Task 3: Test suite (Haiku) - Helps verify fixes

**Phase 2 (After Phase 1 verified)**:
4. Task 4: Physics tuning (Gemini) - Fine-tuning
5. Task 5: Space kit integration (Claude) - Visual enhancement

---

## Handoff Instructions

### For Haiku (Tasks 1, 3):
```
You are helping fix a Bevy 3D flight simulator.
Focus on camera system and debug output.
Files: C:\Users\Box\plane_game\src\main.rs
Goal: Make camera follow plane properly, add diagnostics.
```

### For Gemini Flash (Tasks 2, 4, 5):
```
You are helping improve a Rust/Bevy flight simulator.
Focus on flight physics, turning mechanics, and asset integration.
Files: C:\Users\Box\plane_game\src\main.rs
Goals: Fix turning, tune physics, plan space kit integration.
```

---

## Success Criteria

### Task 1 (Camera): ✅ Fixed when:
- Camera stays exactly 15 units behind plane
- Camera rotates to look at plane center
- Camera doesn't clip into terrain
- Smooth follow during all maneuvers

### Task 2 (Turning): ✅ Fixed when:
- Roll left + pitch up = turn left
- Roll right + pitch up = turn right
- Yaw rate visible in debug output
- Coordinated turn works consistently

### Task 3 (Tests): ✅ Done when:
- Debug output shows camera distance
- Debug output shows yaw rate
- Easy to see if issues exist
- Helps verify fixes from Tasks 1-2

### Task 4 (Tuning): ✅ Done when:
- All constants documented
- Physics formulas explained
- Quick reference table created
- Easy to adjust feel

### Task 5 (Assets): ✅ Done when:
- Asset list complete
- Implementation plan detailed
- Code skeleton provided
- Performance analyzed

---

## Final Handoff

Once all tasks complete, consolidate into:
- Single updated `src/main.rs`
- Updated README with new features
- Delete this diagnostic file
