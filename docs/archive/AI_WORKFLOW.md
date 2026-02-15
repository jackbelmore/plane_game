# AI-Assisted Development Workflow

**Best practices for working efficiently with Claude, Gemini, and Copilot on this project.**

---

## The Problem We're Solving

When working with multiple AI assistants on a complex project:
- AI models get confused by contradictory information in old files
- Context windows fill with outdated diagnostics and failed experiments
- Each AI session starts from scratch instead of building on previous work
- File cleanup is risky - AI might delete code it doesn't understand

**Solution: The "Golden Files" strategy**

---

## The Golden Files (Source of Truth)

| File | Purpose | When to Update |
|------|---------|-----------------|
| **CLAUDE.md** | Current state, known issues, architecture decisions | After each major feature or blocker |
| **GAME_DESIGN.md** | What the game should become (never changes) | Only if design goals shift |
| **README.md** | Quick project overview + build instructions | When build process changes |
| **FIXES_APPLIED.md** | Emergency patches and workarounds (session-specific) | End of each intensive session |
| **.claude.local.md** | Temporary notes during active development | Throughout session (not checked in) |
| **CHECKPOINT_PROMPT.md** | Template for new sessions (reusable) | When workflow changes |

**Rule:** Keep only these 6 files. Delete everything else that's older than 2 weeks.

---

## Session Startup Pattern

```
NEW SESSION:
1. Read: CLAUDE.md (5 min) - What's the current state?
2. Read: GAME_DESIGN.md (5 min) - What are we building?
3. Read: main.rs (10 min) - How does it work?
4. Upload all 3 files to AI
5. Paste CHECKPOINT_PROMPT.md template
6. AI responds with understanding check
7. Begin work on specific task
```

**Total startup time: ~20 minutes**
**Result: AI has complete, consistent context**

---

## During Development

### Code Changes
- **Request:** "Give me ONLY the updated `spawn_trees_in_chunk()` function, don't rewrite the whole file"
- **Why:** Saves token budget, prevents AI from accidentally deleting things
- **Pattern:** Small, focused requests instead of big refactors

### When Stuck
1. Check FIXES_APPLIED.md - Is this a known issue?
2. Check Common Errors table in CLAUDE.md
3. Ask: "This error happened before - how was it fixed?"

### Adding Features
1. Update CLAUDE.md BEFORE starting code
2. Add feature to "Next Steps" section
3. Let AI know what's documented vs new

### Debugging
- Use CHECKPOINT_PROMPT to re-establish context
- Ask: "Does this match the architecture in CLAUDE.md?"
- Share actual error messages, not summaries

---

## End of Session Pattern

```
AT END OF SESSION:
1. Git commit: "Feature: [name] - description + learning notes"
2. Update CLAUDE.md: Add 1-2 lines about what was learned/fixed
3. Delete .claude.local.md notes (or keep minimal)
4. Push to GitHub
5. Done!

NEXT SESSION:
1. New AI reads updated CLAUDE.md
2. Knows exactly what changed
3. No confusion about old vs new state
```

---

## Git Safety Pattern

```bash
# Before major refactor
git commit -m "Pre-refactor checkpoint: [feature] state"

# If AI breaks something
git revert <commit-hash>

# Keep last 5 commits for quick recovery
```

**Why:** If AI deletes something important, you can recover in 30 seconds.

---

## The "Single File" Rule (While Under 2000 Lines)

**Current:** main.rs is 2540+ lines
**Status:** Getting large, but manageable
**Action:** Don't split yet, but watch for:
- Physics logic + Flight controller → Could split at line 1000
- Asset spawning + Chunk system → Could be separate module

**When to split:** Create `systems/` directory when main.rs hits 3000+ lines

---

## Avoiding AI Hallucination

**Things that cause hallucination:**
- ❌ 10 old diagnostic files contradicting each other
- ❌ Outdated architecture descriptions
- ❌ "What should we do?" (AI invents features)
- ❌ Uploading the entire project without focus

**Things that prevent hallucination:**
- ✅ Clean file structure (only 6 core files)
- ✅ Single source of truth (CLAUDE.md)
- ✅ Specific requests ("Fix the horizon glitch")
- ✅ Clear design document (GAME_DESIGN.md)
- ✅ Known issues listed (FIXES_APPLIED.md)

---

## Multi-AI Coordination

**Current setup:**
- **Gemini:** Implemented Phase 2 features (rocket mode, sky transition)
- **Claude (This chat):** Fixing Phase 1 rendering bugs + workflow setup
- **Copilot:** Available for quick code reviews

**Best practice:**
1. One AI = one feature (don't bounce around)
2. End of feature: Update CLAUDE.md with learnings
3. Next AI starts fresh by reading CLAUDE.md
4. No surprises, no contradictions

---

## The "Dark Bar" and "Horizon" Fixes (From Gemini)

**Status:** Mentioned in previous sessions, not yet applied

**Next steps:**
1. CHECKPOINT_PROMPT to establish context with fresh AI
2. Ask AI to diagnose horizon glitch
3. Check camera far clip vs fog distance
4. Apply fix to setup_scene() function
5. Test with `cargo run --release`

---

## Expected Workflow Timeline

```
DAY 1: Fix asset loading (green cubes → actual trees)
  - 2-3 hours of focused work
  - Commit: "Feature: Load .glb trees via Mesh3d + StandardMaterial"
  - Update CLAUDE.md

DAY 2: Test Phase 1 & 2
  - Verify 60+ FPS, chunk loading, rocket mode
  - 1-2 hours testing + documentation
  - Commit: "Test: Phase 1&2 verification complete"

DAY 3+: Phase 3 preparation
  - Waiting on drone 3D models from user
  - Review PHASE3_IMPLEMENTATION_PROMPTS.md
  - Design architecture for combat system
  - Start implementation when assets available
```

---

## Quick Reference

**When you see:**
- "Horizon has weird black bar" → Check camera far clip (CLAUDE.md line 76)
- "Trees invisible" → Check ambient light (CLAUDE.md line 71)
- "Ground has holes" → LOAD_RADIUS too small (CLAUDE.md line 55)
- "Physics NaN crash" → NaN check missing (CLAUDE.md line 83)

**How to share code with AI:**
- "Here's main.rs (2540 lines) + CLAUDE.md. Fix the horizon glitch."
- AI will read CLAUDE.md first, then look at main.rs
- No need to paste 50 lines of context

---

## Measuring Success

✅ **Good session:**
- Started with CHECKPOINT_PROMPT
- Made 1-2 focused code changes
- Committed to Git
- Updated CLAUDE.md
- Next AI knows where we are

❌ **Bad session:**
- AI proposed 3 contradictory solutions
- Didn't reference GAME_DESIGN.md
- Made sweeping changes without understanding context
- Deleted something important by mistake

---

**Last Updated:** 2026-02-05
**Created by:** Claude Haiku + Gemini advice consolidation
**Status:** Ready for production use
