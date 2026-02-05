# F-16 Flight Simulator - Game Design Document

**Date Created:** 2026-02-04
**Last Updated:** 2026-02-04
**Project Timeline:** Few weeks of active development

---

## Core Vision

**High-Level Concept:**
Earth-based F-16 flight simulator with realistic earth-to-space capability. Player flies over detailed terrain (villages, forests, cities) with combat against drone enemies. Special rocket booster mode enables space travel at realistic distances.

**NOT:** Space combat sim
**IS:** Earth flight sim with optional space travel

---

## Gameplay Mechanics

### Primary Gameplay Loop
- **Exploration/Sightseeing** - Fly over detailed earth terrain
- **Combat** - Destroy drone enemies (Ukraine war-style drones)
  - Mix of swarm behavior (groups, weaving patterns)
  - Kamikaze attacks (fly toward player to explode on contact)
- **Space Travel** - Use rocket booster to reach 25km altitude and enter space

### Flight Modes

#### Normal Flight Mode (Current)
- **Speed:** 200-300 m/s typical
- **Controls:** Arcade-style (current implementation)
- **Purpose:** Earth exploration and combat

#### Rocket Booster Mode (NEW - Secret Key)
- **Activation:** Unlimited toggle via secret key (not revealed in UI)
- **Speed:** Similar to space rockets (need exact speed calculation)
- **Thrust:** HUGE increase (enough to reach 25km in 1-2 minutes)
- **Purpose:** Enable space travel
- **Visual:** Enhanced afterburner effects, speed counter changes

### Combat System
- **Enemies:** Drone enemies (inspired by Ukraine war drones)
- **AI Behaviors:**
  - **Swarm drones:** Fly in groups, weave around, harder to hit
  - **Kamikaze drones:** Actively pursue player for collision/explosion
  - **Mix:** Both types spawn for varied encounters

---

## World Design

### Scale & Distance

#### Altitude Zones
| Zone | Altitude Range | Visual | Purpose |
|------|---------------|--------|---------|
| **Ground** | 0 - 5 km | Earth terrain, villages, forests | Main gameplay area |
| **Low Atmosphere** | 5 - 15 km | Sky with clouds | Normal flight ceiling |
| **High Atmosphere** | 15 - 25 km | **Sky→space transition** | Gradual visual change |
| **Space** | 25+ km | Black space, stars | Destination, special achievement |

**Key Decision:** Space begins at **25 km altitude** (1/4 of real Kármán line)
- Arcade-style distance for gameplay
- Reachable in ~1-2 minutes with rocket booster
- Far enough to feel like achievement
- Close enough to not be boring

#### Map Size
- **Goal:** "As large as we can automate without game crashing"
- **Current:** 10 km × 10 km (too small)
- **Target:** 100+ km × 100+ km minimum
- **Constraint:** Performance must remain stable
- **Solution:** Use chunk-based LOD system (only spawn nearby assets)

### Terrain & Environment

#### Ground Assets (Priority Order)
1. **Vegetation (Highest Priority)**
   - Trees (forests, scattered groups)
   - Grass/ground cover
   - Distribution: Evenly scattered across map for consistent performance

2. **Medieval Villages**
   - Use Kenney fantasy town assets (already in project)
   - Multiple villages scattered across map
   - Each village: 16-32 buildings in circular Angerdorf layout

3. **Future Additions (Not Current Scope)**
   - Cities with modern buildings
   - Roads/highways
   - People walking (NPCs)
   - Detailed city streets
   - Rivers/water bodies

#### Asset Distribution Strategy
**Method:** Even scatter (chosen for performance over large maps)
- Grid-based spawning system
- Chunk loading: Only spawn assets within render distance
- Chunk unloading: Despawn far assets to maintain performance
- Consistent density across map (no sudden performance drops)

**Why Not Random/Clustered:**
- Random: Unpredictable performance spikes in dense areas
- Clustered: Hard to balance density vs performance at scale
- Even scatter: Predictable, stable performance, easier to tune

---

## Visual Style

### Art Direction
**Style:** **Mix/Stylized** (Kenney asset aesthetic)
- Clean, readable shapes
- Stylized textures (not photorealistic)
- Arcade feel with semi-realistic lighting
- Kenney assets for consistency

**References:**
- Current Kenney fantasy town assets ✓
- Kenney space kit (for space assets)
- Kenney particle pack (explosions, effects)

**NOT:**
- Photorealistic (like MSFS)
- Fully realistic lighting/materials

---

## Camera & Controls

### Camera System
**Current:** Third-person chase camera (15 units behind, 5 units above)
- **Keep:** This camera for now
- **Future Addition:** Cockpit view (first-person)
  - User wants to "steal code from open source plane game"
  - Need: Cockpit assets + camera switching system

### Controls
- **Current:** Arcade flight controls (keep as-is)
- **New:** Secret key for rocket booster mode toggle

---

## UI & Feedback

### Speed Counter (High Priority)
**Reference Image:** User provided screenshot (temp file expired)
**Requirements:**
- Display current speed in multiple units
- Visual design similar to reference image
- Update in real-time
- Different visual style for rocket mode vs normal mode

### HUD Elements (Current + Planned)
- [x] Altitude display
- [x] Speed display
- [x] Throttle percentage
- [ ] **NEW:** Rocket mode indicator
- [ ] **NEW:** Speed counter (styled UI)
- [ ] **NEW:** Altitude zones indicator (Ground/Sky/Space)

---

## Technical Considerations

### Performance Targets
- **PC Specs:** Unknown (user's current PC)
- **Target FPS:** 60 FPS minimum
- **Asset Density:** "No clue" - need testing and benchmarking

### Efficient Asset Spawning
**Question:** "Is there a way the Rust engine can spawn lots of assets efficiently?"
**Answer:** Yes - Bevy supports:
1. **Chunk-based LOD system** - Only spawn visible/nearby assets
2. **Instancing** - Reuse same mesh for many objects (trees, rocks)
3. **Spatial partitioning** - Cull invisible objects quickly
4. **Asset streaming** - Load/unload assets dynamically

**Implementation Needed:**
- Chunk system for ground assets
- Distance-based LOD (far trees = simple meshes)
- Frustum culling (don't render behind camera)
- Instancing for repeated assets (trees, buildings)

---

## Asset Requirements

### Confirmed Assets (In Project)
- ✓ Kenney fantasy town kit (~180 buildings)
- ✓ Kenney particle pack (flames, explosions)
- ✓ F-16 models (multiple variants)
- ✓ Cloud textures (10 variants)

### Needed Assets
- **Trees** - User will provide/find
- **Ground textures** - Grass, dirt, mud
- **Drone models** - For enemy combat
- **Cockpit interior** - For future first-person view

### Asset Packs to Consider
**From user's downloads:**
- ✓ `kenney_space-kit.zip` - For space structures (if adding space stations)
- ✗ `kenney_city-kit-*.zip` - Future use (modern cities not current priority)
- ✗ `kenney_blocky-characters.zip` - Future use (NPCs not current priority)
- ✗ `kenney_train-kit.zip` - Not needed
- ✗ `Sonniss.com GDC 2019 Audio` - Consider for sound effects expansion

---

## Development Priorities (Next Few Weeks)

### Phase 1: Core World Systems (HIGHEST PRIORITY)
1. **Expand world size** - 100km × 100km minimum
2. **Fix main environment rendering issues**
3. **Implement chunk-based asset spawning system**
4. **Add ground vegetation (trees, grass)**

### Phase 2: Rocket Booster & Space
1. **Rocket booster mode** - Secret key toggle
2. **Sky→space transition** - Gradual visual change 0-25km
3. **Speed counter UI** - Styled display
4. **Space environment** - Black sky, stars at 25km+

### Phase 3: Combat & Enemies
1. **Drone enemy models** - Find/create assets
2. **Swarm AI** - Group movement, weaving
3. **Kamikaze AI** - Pursuit and collision
4. **Combat balancing** - Difficulty tuning

### Phase 4: Polish & Expansion (Future)
- Cockpit view camera
- More building variety
- Cities and roads
- NPCs and traffic
- Water/rivers
- Weather effects

---

## Open Questions & Research Needed

### 1. Rocket Booster Specifications
- **Max speed:** How fast? (Real rockets: 1000-7000 m/s)
- **Acceleration:** How quick to reach max speed?
- **Visual effects:** What changes in rocket mode?
- **Physics:** Keep arcade or add realistic rocket physics?

### 2. Sky→Space Transition
- **Gradual color change:** How to interpolate sky color 0-25km?
- **Fog adjustments:** Reduce fog as altitude increases?
- **Stars appear:** At what altitude do stars become visible?
- **Sun/lighting:** Does sun behavior change in space?

### 3. Performance Benchmarking
- **Current FPS:** What is it now with current assets?
- **Asset density tests:** How many trees before lag?
- **Chunk size:** Optimal chunk dimensions for LOD system?
- **Draw distance:** Max distance before culling?

### 4. Cockpit View Implementation
**User wants:** "Prompt to give Gemini to look for open source plane game code"

**Suggested Prompt for Gemini:**
```
Find open source flight simulator games (on GitHub) that use:
- Rust + Bevy game engine (preferred), OR
- Similar 3D game engines (Godot, Unity)
- Cockpit/first-person camera view implementation
- F-16 or fighter jet cockpit assets (free/open license)

Search for:
1. Repos with working cockpit view code
2. Free cockpit 3D models (GLTF/GLB format)
3. Camera switching systems (third-person ↔ cockpit view)
4. HUD overlay rendering in cockpit view

Filter by:
- Open source license (MIT, Apache, GPL)
- Active repos (updated in last 2 years)
- Well-documented code

Provide: Repo links, relevant file paths, license info
```

---

## Success Metrics

The game is successful when:
1. ✅ Player can fly over detailed earth terrain for extended periods
2. ✅ Rocket booster enables reaching 25km altitude in 1-2 minutes
3. ✅ Sky gradually transitions to space visuals
4. ✅ Ground has forests and villages that make it visually interesting
5. ✅ Drone combat provides engaging gameplay
6. ✅ Game runs at 60 FPS on user's PC
7. ✅ World feels large (100+ km²)
8. ✅ Chunk system prevents performance issues

---

## Notes

- **Project Duration:** Few weeks of active development
- **Context Management:** This file referenced in future sessions for preferences
- **Scope Creep:** Avoid adding cities/NPCs/roads until core systems working
- **Testing:** Frequent performance testing as assets added
