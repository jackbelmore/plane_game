╔═══════════════════════════════════════════════════════════════════════════════╗
║                         GEMINI FLASH ASSIGNMENT                              ║
║                    Diagnostics & Physics Documentation                       ║
╚═══════════════════════════════════════════════════════════════════════════════╝

GEMINI FLASH - WORK PROMPT
═══════════════════════════════════════════════════════════════════════════════

You are helping improve a Rust/Bevy flight simulator.

PROJECT: F-16 Flight Simulator (arcade physics, 3rd person camera)
FILE: C:\Users\Box\plane_game\src\main.rs
REPO: C:\Users\Box\plane_game\

YOUR MISSION: Create diagnostic system + physics documentation

While another AI (Claude) fixes the actual physics bugs, you will:
1. Create a diagnostic system that prints flight state every frame
2. Document physics constants with effects and tuning guide

TIME: 30 minutes
PARALLEL WITH: Claude Code (who fixes thrust + turning)

═══════════════════════════════════════════════════════════════════════════════

PART 1: DIAGNOSTIC SYSTEM (20 minutes)
────────────────────────────────────────────────────────────────────────────────

TASK: Create new system function `debug_flight_diagnostics()` that runs in the
      main game loop and prints diagnostics every 0.5 seconds.

LOCATION: Add to src/main.rs after the `update_flight_camera()` function

WHAT TO PRINT (every 0.5 seconds):

═════════════════════════════════════════════════════════════════════
FLIGHT DIAGNOSTICS
─────────────────────────────────────────────────────────────────────
ALT: XXXXX m  |  CLIMB: +XX.X m/s  |  SPEED: XXX m/s  |  THR: XX%
ROLL: XX.X°   |  PITCH: XX.X°  |  YAW: XX.X°
INPUTS: [Pitch: X.X][Roll: X.X][Yaw: X.X][Throttle: X.X]
───────────────────────────────────────────────────────────────────
TURNING: Roll+Pitch detected? [YES/NO]  Yaw Rate: XX°/s
═════════════════════════════════════════════════════════════════════

HOW TO IMPLEMENT:

```rust
// Add new component to track time
#[derive(Component)]
struct DiagnosticTimer(f32);

// New system function
fn debug_flight_diagnostics(
    time: Res<Time>,
    mut timer_query: Query<&mut DiagnosticTimer>,
    player_query: Query<
        (
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
            &PlayerInput,
        ),
        With<PlayerPlane>,
    >,
) {
    // Track time, print every 0.5 seconds
    
    if let Ok((transform, velocity, ang_vel, input)) = player_query.get_single() {
        // 1. Calculate altitude: transform.translation.y
        // 2. Calculate climb rate: velocity.y
        // 3. Calculate speed: velocity.length()
        // 4. Calculate angles:
        //    let (roll, pitch, yaw) = transform.rotation.to_euler(EulerRot::XYZ);
        //    Convert to degrees: * 180.0 / PI
        // 5. Check if turning: roll != 0 and pitch != 0
        // 6. Get yaw rate from angular velocity
        
        // Print formatted output
        println!("═════════════════════════════════════════════════════════════════════");
        println!("FLIGHT DIAGNOSTICS");
        println!("─────────────────────────────────────────────────────────────────────");
        println!("ALT: {:>5} m  |  CLIMB: {:>+6.1} m/s  |  SPEED: {:>3} m/s  |  THR: {:>2}%",
            altitude, climb_rate, speed, throttle_percent);
        println!("ROLL: {:>6.1}°  |  PITCH: {:>6.1}°  |  YAW: {:>6.1}°",
            roll_deg, pitch_deg, yaw_deg);
        println!("INPUTS: [Pitch: {:.1}][Roll: {:.1}][Yaw: {:.1}][Throttle: {:.1}]",
            input.pitch, input.roll, input.yaw, input.throttle);
        println!("─────────────────────────────────────────────────────────────────────");
        println!("TURNING: {}  Yaw Rate: {:.1}°/s",
            if turning { "YES" } else { "NO " }, yaw_rate_deg_per_sec);
        println!("═════════════════════════════════════════════════════════════════════\n");
    }
}
```

ADD TO SCHEDULE (line ~290):
```rust
.add_systems(Update, (
    debug_flight_diagnostics,  // ADD THIS LINE
    // ... other systems ...
))
```

KEY CALCULATIONS:
- Altitude: transform.translation.y
- Climb Rate: velocity.0.y (vertical component)
- Speed: velocity.length()
- Roll/Pitch/Yaw: transform.rotation.to_euler(EulerRot::XYZ), then * 180/PI
- Yaw Rate: angular_velocity.0.y (around Y axis) * 180/PI
- Turning: (input.roll != 0) && (input.pitch != 0)
- Throttle %: input.throttle * 100.0 as i32

═════════════════════════════════════════════════════════════════════

PART 2: PHYSICS CONSTANTS GUIDE (10 minutes)
────────────────────────────────────────────────────────────────────────────────

TASK: Create new file `PHYSICS_CONSTANTS_GUIDE.md` that documents each
      physics constant with effects, formulas, and tuning ranges.

LOCATION: C:\Users\Box\plane_game\PHYSICS_CONSTANTS_GUIDE.md

TEMPLATE FOR EACH CONSTANT:

## CONSTANT_NAME

**Current Value**: X.X

**Purpose**: What does it do?

**Physics Formula**: How it affects the game

**Effect**: In-game behavior

**Tuning Range**:
- Low Feel (sluggish): X.X
- Medium Feel (current): X.X  ← Current
- High Feel (twitchy): X.X

**How to Adjust**: What happens if you increase/decrease it

**Recommended**: Keep at X for arcade fighter feel

---

CONSTANTS TO DOCUMENT (from src/main.rs lines 835-845):

1. MAX_THRUST_NEWTONS (currently 100000)
   Purpose: Maximum engine thrust force
   Physics: F=ma, force determines acceleration upward
   Effect: Higher = easier to climb/maintain altitude
   Tuning: 50000 (weak) → 100000 (current) → 150000 (strong)
   Formula: Climb rate ≈ (thrust - gravity_force) / mass
          = (thrust - 88290) / 9000

2. PITCH_RATE (currently 1.8)
   Purpose: How fast pitch input rotates nose up/down
   Physics: Angular velocity in rad/s
   Effect: Higher = snappier pitch response
   Tuning: 1.0 (sluggish) → 1.8 (current) → 3.6 (arcade)
   Convert: 1.8 rad/s = 103°/s

3. ROLL_RATE (currently 2.5)
   Purpose: How fast roll input banks wings left/right
   Physics: Angular velocity in rad/s
   Effect: Higher = quicker banking for turns
   Tuning: 1.5 (sluggish) → 2.5 (current) → 4.0 (arcade)
   Convert: 2.5 rad/s = 143°/s

4. YAW_RATE (currently 1.2)
   Purpose: How fast yaw rotates nose left/right
   Physics: Angular velocity in rad/s
   Effect: Higher = sharper rudder response
   Tuning: 0.8 (sluggish) → 1.2 (current) → 2.0 (arcade)
   Convert: 1.2 rad/s = 69°/s

5. BOOST_MULTIPLIER (currently 3.5)
   Purpose: Thrust multiplier when throttle > 50%
   Physics: Scales acceleration force
   Effect: Press Shift for 3.5x speed boost
   Tuning: 2.0 (mild) → 3.5 (current) → 5.0 (extreme)
   Formula: Boost thrust = normal_thrust * multiplier

6. DRAG_COEFFICIENT (currently 0.1)
   Purpose: Air resistance/drag force
   Physics: Drag = -velocity * speed * coefficient
   Effect: Higher = more wind resistance, tighter controls
   Tuning: 0.05 (floaty) → 0.1 (current) → 0.3 (sticky)

7. SMOOTHING_FACTOR (currently 0.15)
   Purpose: Lerp smoothing for rotation response
   Physics: Animation lerp weight (0.0 = no change, 1.0 = instant)
   Effect: Higher = snappier response, more twitchy
   Tuning: 0.05 (sluggish) → 0.15 (current) → 0.4 (twitchy)

---

CREATE FILE WITH:
- Title: Physics Constants Tuning Guide
- Intro: What each constant does
- Each constant documented as above
- Quick reference table (see example below)
- Section: "For Arcade Dogfight Feel" (recommended values)
- Section: "For Realistic Feel" (alternate values)

QUICK REFERENCE TABLE:

| Constant | Current | Low Feel | Medium | High Feel |
|----------|---------|----------|--------|-----------|
| MAX_THRUST | 100000 | 75000 | 100000 | 150000 |
| PITCH_RATE | 1.8 | 1.0 | 1.8 | 3.6 |
| ROLL_RATE | 2.5 | 1.5 | 2.5 | 4.0 |
| YAW_RATE | 1.2 | 0.8 | 1.2 | 2.0 |
| BOOST_MULT | 3.5 | 2.0 | 3.5 | 5.0 |
| DRAG | 0.1 | 0.05 | 0.1 | 0.3 |
| SMOOTHING | 0.15 | 0.05 | 0.15 | 0.4 |

═════════════════════════════════════════════════════════════════════

DELIVERABLES:

✅ debug_flight_diagnostics() system (ready to add to main.rs)
✅ Schedule integration (copy-paste ready)
✅ PHYSICS_CONSTANTS_GUIDE.md (complete documentation)
✅ Print format (copy-paste ready)

═════════════════════════════════════════════════════════════════════

SUCCESS CRITERIA:

✅ Diagnostic output appears every 0.5 seconds in console
✅ Shows altitude, climb rate, speed, angles, inputs
✅ Shows yaw rate (key for verifying turning works)
✅ Output is clean and readable
✅ Physics guide is thorough and well-organized
✅ Tuning values make sense (with formulas backing them)
✅ Can be easily referenced for future adjustments

═══════════════════════════════════════════════════════════════════════════════

NEXT STEP:
  1. Complete both deliverables (diagnostics + guide)
  2. Send to user (Claude in main session)
  3. Claude merges into src/main.rs
  4. Run game with diagnostics
  5. Use output to verify Claude's physics fixes work correctly

═══════════════════════════════════════════════════════════════════════════════
