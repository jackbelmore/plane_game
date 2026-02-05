# Checkpoint Prompt Template

**Use this prompt at the start of each new major phase or when transitioning between AI assistants.**

---

## Template (Copy & Paste)

```
[PHASE X CHECKPOINT - Use this to establish context with a fresh AI session]

We are starting a new development phase. Please:

1. SUMMARIZE our progress in one paragraph (based on CLAUDE.md)
2. CONFIRM the tech stack: Bevy 0.15, Avian3D physics, Rust
3. IDENTIFY the next 3 implementation steps based on GAME_DESIGN.md goals
4. NOTE any known blockers or workarounds from FIXES_APPLIED.md

Files to reference:
- CLAUDE.md: Current architectural state + known issues
- GAME_DESIGN.md: Design vision and success criteria
- README.md: Quick project overview
- FIXES_APPLIED.md: Recent emergency patches

Then: Provide your assessment of technical debt and risk areas.
```

---

## When to Use

1. **Starting a new chat session** - Paste this + upload main.rs + CLAUDE.md
2. **Switching between AI models** - Gemini → Claude → Copilot transitions
3. **Before major refactors** - Ensures AI understands full context
4. **After major features** - Update CLAUDE.md, then use this to verify understanding
5. **Phase transitions** - Phase 1 → Phase 2 → Phase 3

---

## Expected Response

Good AI will:
- ✅ Summarize: "Phase 1&2 complete: infinite world + rocket mode"
- ✅ Confirm: "Using Bevy 0.15, Avian3D 0.2, Rust latest"
- ✅ List next steps: "1. Fix .glb asset rendering 2. Test horizon glitch 3. Verify Phase 3 readiness"
- ✅ Flag blockers: "Phase 3 blocked on drone 3D models"
- ✅ Assess debt: "Main.rs at 2540+ lines, consider splitting around 1500-2000 line mark"

Bad AI will:
- ❌ Ignore context and propose rebuilding entire game
- ❌ Suggest using different game engine
- ❌ Miss critical blockers or constraints
- ❌ Propose changes contradicting GAME_DESIGN.md

---

## Execution Pattern (Recommended)

```
Session Start:
1. [Upload main.rs + CLAUDE.md]
2. [Paste CHECKPOINT_PROMPT template above]
3. [Wait for AI response]
4. [AI should confirm understanding]
5. [Then proceed with specific implementation request]
6. [At end: Update CLAUDE.md with new learnings]
```

---

**Last Updated:** 2026-02-05
**Status:** Ready to use for Phase 2 → Phase 3 transition
