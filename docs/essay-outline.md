# Building with AI Agents: Lessons from a Real-Time Audio Project

**An essay on vibe coding, session persistence, and human-AI collaboration in December 2025**

---

## I. Introduction: The Promise and Problem of Agentic Coding

- Vibe coding defined: describing intent, letting AI generate implementation
- The gap between demo and production: why most AI coding projects stall
- Thesis: Success requires infrastructure for **persistence**, **precision**, and **partnership**

---

## II. The Project: A Concrete Example

*Brief description without getting lost in specifics*

- Goal: Natural language control of music production software
- Architecture: Human → AI Agent → Orchestrator → Plugins → Audio output
- Why this project is instructive:
  - Real-time constraints (audio can't glitch)
  - Multi-component system (not just one script)
  - Long-term development (weeks/months, not hours)
  - Domain expertise required (music, audio DSP, plugin formats)

---

## III. Choosing Your Tools: Model Capabilities Matter

### What Claude Opus 4.5 Brings (December 2025)

- 80.9% SWE-bench: Real software engineering, not just snippets
- Agentic strength: Fewer dead-ends in multi-step autonomous tasks
- Extended context with compaction: Long sessions without degradation
- Vision capabilities: Can analyze diagrams, mockups, architecture drawings

### Matching Model to Task

- Use strongest model for architecture decisions (they compound)
- Faster models for routine tasks
- The "effort" parameter: trading speed for depth

### Generalizable Insight

> Your AI's capabilities should inform your workflow, not the other way around. Research what your model is good at.

---

## IV. The Persistence Problem: Memory Across Sessions

### Why Sessions End

- Context windows fill up (200K tokens ≈ a few hours of work)
- Compaction summarizes but loses detail
- Crashes, restarts, context switches

### The Handoff Pattern

*From Anthropic's research on long-running agents*

```
End of session:
  1. Update progress file with current state
  2. Commit work with clear message
  3. Note blockers, decisions, context

Start of session:
  1. Read progress file
  2. Read recent git history
  3. Summarize and continue
```

### Artifacts as Memory

- `progress.md`: Human-readable state
- `features.json`: Structured tracking
- `decisions.md`: Immutable log of why
- Git commits: Checkpoints with context

### Generalizable Insight

> External artifacts are your agent's long-term memory. Design them deliberately.

---

## V. Prompt Translation: From Intent to Action

### The Gap Between Human and Machine

- Human: "Make it brighter and faster"
- Machine needs: Note numbers, durations, velocities, which tracks

### A Translation Framework

```
1. UNDERSTAND - What outcome does the human want?
2. TRANSLATE  - Map to system concepts
3. EXECUTE    - Call tools in correct order
4. VERIFY     - Confirm expected state
5. LEARN      - Note patterns for future
```

### Domain-Specific Patterns

- Musical: "[mood] [direction]" → notes, timing, dynamics
- Technical: "[component] should [behavior]" → code changes
- Debugging: "[X] isn't working" → query state first, then diagnose

### Generalizable Insight

> Define explicit translation patterns for your domain. Don't assume the AI will infer them.

---

## VI. Avoiding Regression: Guardrails and Checkpoints

### Why AI Projects Regress

- "Works for me" without actual testing
- Fixing one thing breaks another
- Context loss leads to repeated mistakes
- Overconfidence in generated code

### Technical Guardrails

- Pre-commit hooks: Format, lint, test before every commit
- Structured feature tracking: Know what's done vs. claimed
- Permission boundaries: Prevent destructive operations

### Process Guardrails

- Commit often, small commits
- Session boundaries as checkpoints
- Tests prevent "it works" lies
- Don't tackle too much at once

### Generalizable Insight

> Tests are remarkably effective at preventing regressions with AI assistants. Invest in them early.

---

## VII. Architecture Decisions: Human Judgment, AI Execution

### What Humans Should Decide

- Component boundaries and responsibilities
- Protocol choices (trade-offs require judgment)
- Priority order (performance vs. flexibility vs. simplicity)
- User experience principles

### What AI Excels At

- Implementing decided architecture consistently
- Generating boilerplate and scaffolding
- Maintaining patterns across many files
- Researching options and summarizing trade-offs

### The Decision Log

- Record decisions as immutable entries
- Include rationale and alternatives considered
- AI can read this to understand "why" not just "what"

### Generalizable Insight

> Separate decision-making (human-led) from implementation (AI-assisted). Document the decisions explicitly.

---

## VIII. Project Structure for AI Collaboration

### Files the AI Reads Every Session

```
CLAUDE.md        - Universal conventions (< 300 lines)
                   Facts, not procedures

KNOWLEDGE.md     - Deep context, research, patterns
                   Can be longer, referenced as needed
```

### Files for Session Management

```
docs/progress.md     - Current state, next steps
docs/features.json   - Structured feature tracking
docs/decisions.md    - Why we made choices
```

### Custom Commands

```
.claude/commands/catchup.md   - Restore context
.claude/commands/handoff.md   - Prepare for next session
.claude/commands/status.md    - Report current state
```

### Generalizable Insight

> Design your file structure for AI consumption. Separate facts from procedures, current state from history.

---

## IX. The Two-Phase Commit Pattern

*A specific pattern that generalizes beyond audio*

### The Problem

- Multiple components need coordinated updates
- Changes should be atomic (all or nothing)
- Timing matters (in our case, musical timing)

### The Solution

```
1. PREPARE  - Stage next state to all components
2. ACK      - Each component confirms ready
3. COMMIT   - All switch simultaneously
4. VERIFY   - Confirm new state active
```

### Where This Applies

- Distributed systems updates
- Multi-file refactoring
- Configuration changes across services
- Any coordinated state transition

### Generalizable Insight

> Complex systems benefit from explicit staging and commit phases. Design for atomic, verifiable transitions.

---

## X. Debugging and Observability

### What the AI Needs to Debug

- Logs with configurable verbosity
- Status queries that return structured data
- Clear error messages with context
- Version/state labels for traceability

### What Humans Need

- Visual confirmation (in our case, plugin UI shows version labels)
- Ability to correlate AI output with system state
- History for "go back to when it worked"

### Generalizable Insight

> Build observability for both human and AI consumers. They need different things.

---

## XI. Performance, Reliability, Debuggability, Testability

### A Priority Order

1. **Performance** - System must meet real-time constraints
2. **Reliability** - Predictable, doesn't crash
3. **Debuggability** - Can trace issues when they occur
4. **Testability** - Automated verification where practical

### Why This Order

- Some domains have hard constraints (audio, finance, safety)
- Reliability enables everything else
- Debugging is needed before tests exist
- Tests are valuable but not at cost of #1-3

### Generalizable Insight

> Establish explicit priority order for your quality attributes. Not everything can be #1.

---

## XII. Conclusion: Partnership, Not Replacement

### What We Learned

- AI agents are powerful but need infrastructure
- Persistence requires deliberate artifact design
- Human judgment is irreplaceable for architecture
- Guardrails prevent regression
- Domain expertise amplifies AI capabilities

### The Future

- Vibe coding is maturing into "vibe engineering"
- Multi-session agents will become standard
- Human-AI collaboration patterns will formalize
- The developers who master these patterns will build faster, better software

### Final Thought

> AI doesn't replace the need for good software engineering practices. It rewards them.

---

## Appendix: Tools and Resources

- Claude Code best practices (Anthropic)
- Effective harnesses for long-running agents (Anthropic)
- nih-plug for Rust audio plugins
- CLAP plugin format
- OSC protocol for music applications

---

*Essay based on research and development conducted December 2025.*
