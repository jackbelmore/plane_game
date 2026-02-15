# Next Claude Session - Ready to Continue

## Quick Status
- ✅ Game runs without crashing
- ✅ Flight physics stable for 5+ minutes
- ✅ Drones spawn with advanced swarm AI
- ✅ Missile system working
- ⚠️ Drone 3D model not rendering (using red cube fallback)
- ⚠️ JPG texture not showing (using procedural fallback)
- 🎮 Playable and testable right now

---

## What You're Walking Into

You're joining mid-development of a **Phase 3 drone combat system** in an F-16 flight simulator built with Bevy 0.15 + Avian3D physics.

### The Good News
- Physics is solid and crashes are fixed
- Drone AI is working and threatening
- Combat mechanics (missile + collision) functional
- Can test gameplay loops without crashes

### The Challenges
1. **Bevy 0.15 has known bugs** affecting asset loading
   - Async textures don't update materials after loading [Issue #15081](https://github.com/bevyengine/bevy/issues/15081)
   - SceneRoot mesh loading has limitations
2. **Two visual blockers** (not game-breaking, just cosmetic):
   - Drones render as red cubes instead of 3D models
   - Ground shows procedural green instead of detailed JPG texture

---

## Files You Need to Know

### Core Game Logic
- `src/main.rs` (3000+ lines) - ALL game systems, physics, chunk management, UI
- `src/drone.rs` - Drone component, advanced swarm AI with lead pursuit & flocking
- `assets/models/drone.glb` - 12MB drone model (not rendering currently)
- `assets/textures/grass/` - JPG textures (async loading broken)

### Session Documentation
- `CLAUDE.md` - Full project context (read this first!)
- `SESSION_HANDOFF.md` - Previous session state
- `CLAUDE_CONTEXT_NEXT_SESSION.md` - Detailed technical reference

---

## Build & Run (2 minutes)

```bash
cd /c/Users/Box/plane_game

# Build (takes ~2 minutes on first run)
cargo build --release 2>&1 | tail -20

# Copy assets (MUST do this every time)
cp -r assets target/release/

# Run
target/release/plane_game.exe
```

**Expected output:**
- Blue sky, green ground with texture pattern
- 6 red cube drones in distance
- Your black F-16 model in foreground
- Should not crash during 5+ minutes of flight

---

## Current Issues & Quick Fixes

### Issue 1: Drones Render as Red Cubes
**File:** `src/drone.rs` lines 20-58
**Current code:** Uses `Cuboid` mesh (guaranteed visible)
**Why:** Drone 3D model path might be wrong or SceneRoot not loading properly

**Quick fix attempts:**
1. Verify path: Change `"models/drone.glb#Scene0"` to try `"models/drone.glb#Mesh0/Primitive0"`
2. Add debug logging: Print `asset_server.get_load_state(&drone_scene_handle)`
3. Try loading as Handle<Scene> instead of spawning with SceneRoot

**If all else fails:** Red cubes work for gameplay testing - can live with it

### Issue 2: Ground Texture Not Detailed
**File:** `src/main.rs` lines ~587-630
**Current code:** Procedural green texture (working, but repetitive)
**Why:** JPG loads asynchronously, bind group never updates (Bevy bug)

**Quick fix attempts:**
1. Try using TGA format instead of JPG (uncompressed = faster load)
2. Implement `bevy_asset_loader` crate for proper async handling
3. Use smaller JPG files (try 512x512 instead of 1024x1024)

**If all else fails:** Procedural texture is acceptable - not a blocker

---

## Key Code Sections

### Physics Safety (DON'T CHANGE)
**Location:** `src/main.rs` line 1127
```rust
fn detect_nan_early() {
    // Runs in FixedFirst schedule (BEFORE physics)
    // Catches and FIXES NaN/invalid values
    // CRITICAL: Must run before FixedUpdate physics schedule
}
```

**Why important:** This prevents AABB assertion crashes. Moving this to Update schedule will break it.

### Drone AI (Advanced but Stable)
**Location:** `src/drone.rs` line 61
- Lead pursuit (predicts player movement 1.2 seconds ahead)
- Swarm separation/alignment/cohesion
- Obstacle avoidance (avoids meteors)
- Tactical weaving (looks cool)
- Dynamic speed multiplier:
  - 5.0x when >5km away (warp pursuit)
  - 3.0x when >2km away (sprint)
  - 2.2x when <1km away (combat)

### Ground Material (Texture System)
**Location:** `src/main.rs` line 587
- Creates material with procedural texture at startup
- System `check_grass_texture_loaded()` monitors JPG loading
- Applies texture when ready (if Bevy fixes bind group issue)

---

## Next Priority Tasks

**Pick ONE to work on:**

### Option A: Fix Drone 3D Model Rendering (HIGH IMPACT)
- Would make drones look like actual drones instead of boxes
- Likely just needs correct scene path or SceneRoot debugging
- Estimated: 30-60 minutes

### Option B: Fix JPG Texture Rendering (MEDIUM IMPACT)
- Would make ground look more detailed
- Requires implementing asset loading workaround
- Estimated: 1-2 hours

### Option C: Add Speed Counter UI (NICE TO HAVE)
- Display velocity on screen
- Completely independent of blocker issues
- Estimated: 1-2 hours

### Option D: Improve Drone Difficulty (GAMEPLAY)
- Increase drone speed or spawn distance tweaks
- Easy 5-minute fix
- Makes combat more challenging

---

## Testing Checklist

Before you claim success:
- [ ] Game launches
- [ ] Can fly for 5+ minutes without crash
- [ ] Drones spawn and approach (visible as red cubes)
- [ ] Can fire missiles (Space key)
- [ ] Missiles hit drones (visual explosion)
- [ ] Drones explode and disappear
- [ ] ESC respawns you
- [ ] F10 exits game
- [ ] FPS stays 60+ during combat

---

## Debugging Hints

### Check Drone Spawning
```bash
# Run game and watch console for:
# - "DEBUG: Spawning drone at Vec3(...)"
# - "DEBUG: Drone pursuing player at dist: XXXm"
```

### Check Texture Loading
```bash
# Watch for:
# - "🌿 STARTUP: Creating procedural grass texture..."
# - "🌿 TEXTURE LOADED!" (if JPG works)
```

### Check Physics Safety
```bash
# Should NOT see:
# - "🚨 EARLY:" messages (NaN being caught)
# - "assertion failed: b.min.cmple(b.max)" (physics crash)
```

---

## Session History

| Date | Session | Status |
|------|---------|--------|
| 2026-02-05 | Claude (this one) | Physics fixed, drones added, blockers documented |
| 2026-02-04 | Gemini Pro/Flash | Drone AI implemented, advanced swarm behavior |
| 2026-02-03 | Gemini | Initial combat system |
| Earlier | Gemini | Flight physics, terrain, rocket mode |

---

## Critical Don'ts

🚫 **DON'T:**
- Move `detect_nan_early()` from FixedFirst to Update schedule
- Remove scale validation checks
- Change drone spawn distance to <500m (causes instant collisions)
- Commit large changes without testing 5+ minutes flight
- Try to fix both blockers at once (do one at a time)

✅ **DO:**
- Run `cp -r assets target/release/` after every build
- Test for at least 5 minutes before declaring success
- Read CLAUDE.md before making architecture changes
- Use red cubes and procedural texture as acceptable fallbacks

---

## Your Mission (If You Choose To Accept)

1. **Build and test** - Verify everything works
2. **Pick one blocker** to fix (drone model OR texture)
3. **Document what you find** - Update SESSION_HANDOFF.md with learnings
4. **Leave clear notes** - Future sessions will thank you

The game is playable and stable. You're just polishing the visuals. Good luck! 🚀
