# Feature Plans - Plane Game

This document outlines planned and in-progress features for the flight simulator.

---

## 🚁 Phase 3: Drone Combat System (In Progress)

### 1. Spawning & Lifetime
- **Density:** 15% chance per chunk to spawn a drone patrol.
- **Location:** Spawns at chunk center + 500m altitude.
- **Despawning:** Drones automatically despawn when 15km+ from player.

### 2. Drone AI Variants
- **Swarm AI:**
  - Lead pursuit (predicts player movement ~1.2s ahead).
  - Flocking behavior: Separation, alignment, and cohesion within 400m.
  - Tactical weaving: Sine-wave movement patterns.
- **Kamikaze AI:**
  - Aggressive direct pursuit when within 2km.
  - Explodes on contact (20m proximity trigger).
- **Dynamic Speed:**
  - Warp Pursuit (5.0x speed) if > 5km away.
  - Combat Mode (2.2x speed) if < 1km away.

### 3. Combat Mechanics
- **Player Weaponry:** Spacebar fires sprite-based projectiles.
- **Health System:** Drones have 50 HP (approx. 5 hits to kill).
- **Explosions:** Orange sphere visual effect on drone death or collision.

---

## 📊 UI & HUD Enhancements

### 1. Speed Counter
- Show numerical velocity on-screen.
- Distinct visual style for Rocket Mode vs. Normal Mode.

### 2. Indicators
- Fuel gauge (Gas canister icon + dynamic bar).
- Rocket Mode active toggle status.
- Damage/Hit feedback for combat.

---

## 🌍 World & Environment Polish

### 1. Terrain Jaggedness (Priority)
- **Problem:** Visible chunk edges at distance.
- **Plan:** Implement fog-based blending or a simple LOD system for intermediate distance chunks.

### 2. Asset Variety
- Populate villages with more Kenney medieval building variants.
- Add building colliders (Phase 4+).

---

## 🚀 Future Ideas (Phase 4+)
- **Cockpit View:** Switchable first-person camera with interior models.
- **Day/Night Cycle:** Dynamic lighting transition.
- **Ground Grid:** Add a texture/grid to the ground to make relative speed more apparent.

---
**Last Updated:** February 15, 2026
