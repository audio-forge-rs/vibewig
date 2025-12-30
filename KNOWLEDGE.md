# Vibewig Knowledge Base

**Product Name:** Voxel
**Tagline:** "Speak music into existence"

Living document for research, design decisions, and accumulated knowledge.

---

## Project Vision

Interactive music generation where typing prompts in Claude Code produces music in Bitwig Studio. The human listens and iteratively refines through conversation.

**Flow:**
```
1. Human describes what they want (natural language)
2. Claude Code translates → batch of structured instructions
3. Conductor stages new state to ALL plugins (prepare phase)
4. Conductor sends synchronized "commit" signal
5. All plugins switch to new program on next beat 1 of Bitwig transport
6. Conductor returns version label (e.g., "v12-bright-arpeggio")
7. Claude reports label to user
```

Patterns loop and persist until changed. Transport-synced. Stable, not one-shots.

**Key insight:** This is NOT low-latency fire-and-forget. It's a **two-phase commit** model:
- **Prepare:** Stage next state to all plugins
- **Commit:** Synchronized trigger, all switch on beat 1

---

## Architecture

### Component Responsibilities

| Component | Role | Does What It's Best At |
|-----------|------|------------------------|
| **Human** | Creative director | Listens, conveys intent in natural language |
| **Claude Code** | Translator + Builder | Translates intent → structured commands, builds/maintains system |
| **Conductor** | Orchestrator | Manages plugins, stages state, coordinates synchronized commits |
| **Plugin** | Music engine | Receives state, syncs to transport, outputs MIDI |

### System Overview

```
┌─────────────┐      HTTP       ┌─────────────────────────────────┐
│  Claude     │ ◄─────────────► │         CONDUCTOR               │
│  Code       │    REST API     │         (daemon)                │
│             │                 │                                 │
│  invokes:   │                 │  - Manages plugin connections   │
│  vibewig    │                 │  - Stores version history       │
│  CLI        │                 │  - Orchestrates PREPARE/COMMIT  │
└─────────────┘                 └─────────────────────────────────┘
                                          │
                                          │ OSC over UDP
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
                    ▼                     ▼                     ▼
             ┌──────────┐          ┌──────────┐          ┌──────────┐
             │ Plugin 1 │          │ Plugin 2 │          │ Plugin N │
             │          │          │          │          │          │
             │ GUI:     │          │ GUI:     │          │ GUI:     │
             │ v5-intro │          │ v5-intro │          │ v5-intro │
             │ (staged: │          │ (staged: │          │ (staged: │
             │  v6-drop)│          │  v6-drop)│          │  v6-drop)│
             └──────────┘          └──────────┘          └──────────┘
                    │                     │                     │
                    ▼                     ▼                     ▼
               MIDI out              MIDI out              MIDI out
                    │                     │                     │
                    ▼                     ▼                     ▼
              Bitwig Track         Bitwig Track         Bitwig Track
```

### OSC Protocol

**Conductor → Plugin:**
| Message | Args | Description |
|---------|------|-------------|
| `/vibewig/prepare` | state, label | Stage next program |
| `/vibewig/commit` | | Switch to staged on beat 1 |
| `/vibewig/cancel` | | Delete staged program, keep current |
| `/vibewig/mute` | bool | Mute/unmute current output |

**Plugin → Conductor:**
| Message | Args | Description |
|---------|------|-------------|
| `/vibewig/register` | plugin_id, port | Plugin announces itself |
| `/vibewig/ack` | plugin_id | Acknowledge PREPARE received |
| `/vibewig/status` | plugin_id, current, staged, muted | Periodic heartbeat |

### Synchronization Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         CONDUCTOR                                │
│                                                                  │
│  PREPARE FLOW:                                                  │
│  1. Receive batch from Claude Code (HTTP POST)                  │
│  2. Generate version label (e.g., "v6-drop")                    │
│  3. For each plugin:                                            │
│     └── Send OSC: /vibewig/prepare {state, label}               │
│     └── Wait for OSC: /vibewig/ack                              │
│  4. All plugins ready?                                          │
│     └── Send OSC: /vibewig/commit to all                        │
│  5. Return HTTP 200 with version label                          │
│                                                                  │
│  CANCEL FLOW:                                                   │
│  1. Receive cancel from Claude Code (HTTP DELETE)               │
│  2. Send OSC: /vibewig/cancel to all plugins                    │
│  3. Staged programs discarded, current keeps playing            │
│                                                                  │
│  MUTE FLOW:                                                     │
│  1. Receive mute from Claude Code (HTTP POST /mute)             │
│  2. Send OSC: /vibewig/mute true to target plugins              │
│  3. Plugins silence output (notes off), program stays loaded    │
└─────────────────────────────────────────────────────────────────┘
```

### Plugin State Machine

```
                              MUTE
                    ┌─────────────────────────────────────┐
                    │                                     │
                    ▼                                     │
            ┌──────────────┐                              │
            │    MUTED     │◄───── MUTE ──────┐           │
            │  (silence)   │                  │           │
            └──────┬───────┘                  │           │
                   │                          │           │
                UNMUTE                        │           │
                   │                          │           │
                   ▼                          │           │
            ┌─────────────┐                   │           │
            │   PLAYING   │◄──────────────────┼───────────┤
            │  (current)  │                   │           │
            └──────┬──────┘                   │           │
                   │                          │           │
            PREPARE(next_state)               │           │
                   │                          │           │
                   ▼                          │           │
            ┌─────────────┐     CANCEL        │           │
            │   STAGED    │──────────────────►│           │
            │  (current + │                   │           │
            │   pending)  │                              │
            └──────┬──────┘                              │
                   │                                     │
                COMMIT                                   │
                   │                                     │
                   ▼                                     │
            ┌─────────────┐                              │
            │  COMMITTED  │                              │
            │  (waiting   │──── beat 1 ─────────────────►│
            │  for beat 1)│                    back to PLAYING
            └─────────────┘
```

**States:**
- **PLAYING** - Current program running, outputting MIDI
- **MUTED** - Program loaded but silent (all notes off)
- **STAGED** - Current playing + next program waiting
- **COMMITTED** - Waiting for beat 1 to switch

### Version Labeling

Each commit gets a label for traceability:
- **Claude generates contextual names** based on what's being created
  - Examples: `gentle-intro`, `rising-tension`, `drop-bass`, `bright-arpeggio`
- Includes version number for ordering: `v12-bright-arpeggio`
- Claude reports label so human can reference: "go back to gentle-intro"

**Plugin UI displays:**
- Current version: what's playing now
- Staged version: what will play after COMMIT (if any)

**Synchronized mental model:**
- Human sees version label in Claude Code output
- Human sees same label in Bitwig plugin UI
- Both match → human knows exactly what's playing
- Enables commands like "go back to rising-tension" - human can verify in plugin UI

**History lives in Conductor, not Plugin:**
- Plugin only knows: current program, next program (if staged)
- Plugin is stateless regarding history
- Conductor stores all versions with labels
- Rollback = Conductor sends old version as new PREPARE
- "Go back to rising-tension" → Conductor looks up, sends as next state

**Version recall in Claude Code session:**
- Human asks: "what versions do we have?"
- Claude calls tool to get version list from Conductor
- Claude adds context from conversation memory:
  - `v1-gentle-intro` - "the soft opening with piano arpeggios"
  - `v2-rising-tension` - "added the bass and faster rhythm"
  - `v3-drop` - "the heavy section with low synth"
- Human gets both the labels AND the creative context
- Conductor stores state data, Claude remembers intent/meaning

---

## Design Principles

### Priority Order

1. **Performance** - Low latency, no audio glitches, responsive
2. **Reliability** - Robust, predictable, doesn't crash
3. **Debuggability** - Can see what's happening, trace issues
4. **Testability** - Unit tests where practical

### Debuggability

- Conductor logs all messages (OSC in/out, HTTP requests)
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- Plugin can log to stderr (nih-plug captures)
- Version labels provide audit trail
- `/vibewig/status` heartbeat shows plugin state
- CLI can query Conductor for current state of all plugins

**Debug commands (via CLI):**
```bash
vibewig status          # Show all plugins, their state, versions
vibewig history         # Show version history
vibewig log --follow    # Tail Conductor logs
```

### Testability

- Core logic separated from I/O (OSC, HTTP)
- State machines are pure functions where possible
- Mock transport for plugin tests
- Integration tests: Conductor + mock plugins

**What to test:**
- State machine transitions (PLAYING → STAGED → COMMITTED)
- Version history storage/retrieval
- OSC message parsing/serialization
- PREPARE/COMMIT/CANCEL flows

**What NOT to test (performance sensitive):**
- Audio thread code in plugin
- Real-time OSC latency

---

## Design Decisions

### Component Naming: Conductor

**Decided Dec 2025**

"Conductor" - orchestrates the ensemble, gives the downbeat. Musical metaphor fits the domain.

### Connection Direction: Plugin → Conductor

**Decided Dec 2025**

- Plugin dials out to Conductor on load, registers itself
- Conductor is always-on server (daemon)
- Plugin stays simple - just a client that reports in
- Natural discovery - Conductor knows which plugins are alive

### Conductor Lifecycle: Daemon + CLI Wrapper

**Decided Dec 2025**

- `vibewig-conductor` - long-running daemon, maintains connections and state history
- `vibewig` (CLI) - thin wrapper that sends commands to daemon
- Claude Code invokes CLI, CLI talks to daemon
- Clean separation of concerns

### Synchronization: Conductor sends COMMIT, Plugin handles timing

**Decided Dec 2025**

- Conductor sends COMMIT message
- Plugin watches Bitwig transport internally
- **If transport playing:** switch on beat 1 of next bar
- **If transport stopped:** switch immediately on COMMIT
- Conductor must send COMMIT to all plugins as fast as possible (tight loop, no delays)
- Keeps timing logic in plugin (where transport info is native)
- Simpler protocol - Conductor doesn't need to know transport position

**Why immediate when stopped:**
- User is editing/preparing, wants to hear changes now
- No musical timing to sync to anyway
- Natural workflow: stop, tweak, listen, repeat

### Protocol: OSC over UDP (Plugin ↔ Conductor)

**Decided Dec 2025**

- OSC (Open Sound Control) over UDP
- Music ecosystem standard - could integrate with TouchOSC, Max/MSP, etc.
- Low latency
- Trade-off: UDP has no guaranteed delivery

**Reliability consideration:**
- For PREPARE: plugin sends ACK back
- For COMMIT: fire-and-forget is probably fine (plugin will catch next beat 1 anyway)
- Registration: might need retry logic

### Protocol: HTTP REST (CLI ↔ Conductor)

**Decided Dec 2025**

- Standard HTTP REST API
- Easy to debug with curl
- Well-understood patterns
- Claude Code calls CLI, CLI makes HTTP request to daemon

### Plugin GUI: Yes

**Decided Dec 2025**

- Shows current version label (what's playing)
- Shows staged version label (what's pending after COMMIT)
- Visual sync with Claude Code output
- nih-plug supports egui, iced, VIZIA for GUI

### Why CLAP Plugin (not Bitwig Controller)?

**Decided Dec 2025**

| Factor | CLAP Plugin (Rust/nih-plug) | Bitwig Controller (Java) |
|--------|----------------------------|--------------------------|
| Transport sync | Native via host | Native via API |
| MIDI output | Plugin outputs to track | Can send MIDI |
| Threading | Full control | Event-driven, GC pauses |
| Portability | Any CLAP host | Bitwig only |
| Debugging | Standard Rust tooling | Remote debug via env var |
| Real-time safety | Compiler-enforced | Manual, GC unpredictable |

**Decision:** CLAP plugin. Simpler threading for WebSocket, Rust memory safety, portable.

**Revisit if:** Need to control other tracks, clips, arrangement, or Bitwig-specific features.

### Why WebSocket (not OSC)?

- JSON messages are self-describing, easy to extend
- Bidirectional - plugin can send transport state back
- tokio-tungstenite + tungstenite are mature
- OSC is great for knobs/faders, less ideal for structured commands

**Revisit if:** Latency becomes critical, need UDP, or integrating with OSC ecosystem.

### Why Rust for Client?

- Same language as plugin, shared understanding
- Type-safe message serialization
- Claude Code works well with Rust (compiler as reviewer)
- Could share types via workspace crate later

---

## Technical Research

### Real-Time Audio Constraints

Audio plugins run on a real-time thread where:
- **No allocations** - malloc can block
- **No locks** - mutexes can block
- **No I/O** - network/disk can block
- **No logging** - in release builds

Safe patterns:
- Allocate in `initialize()` only
- Use lock-free channels (crossbeam bounded)
- Pre-allocate buffers
- `try_recv()` / `try_send()` - never block

nih-plug provides `assert_process_allocs` feature to catch violations in debug.

### nih-plug Architecture

```rust
// Plugin lifecycle
impl Plugin for MyPlugin {
    fn initialize(...) -> bool {
        // Safe to allocate, spawn threads, open sockets
    }

    fn reset(&mut self) {
        // Called on audio thread - must be RT-safe
        // Called when transport stops, sample rate changes, etc.
    }

    fn process(...) -> ProcessStatus {
        // Audio thread - must be RT-safe
        // Called ~44100/buffer_size times per second
    }
}
```

### Lock-Free Communication Pattern

```
[WebSocket Thread]              [Audio Thread]
       │                              │
       │ parse JSON                   │
       │ validate                     │
       ▼                              │
   tx.try_send(msg)                   │
       │                              │
       └──── bounded channel ────────►│
             (capacity 64)            │
             (lock-free)              ▼
                                 rx.try_recv()
                                      │
                                 update state
                                      │
                                 emit MIDI
```

crossbeam-channel bounded channels:
- Lock-free in common case
- Won't block sender if full (try_send returns Err)
- Won't block receiver if empty (try_recv returns Err)

### CLAP Plugin Format

- Open standard by Bitwig + u-he
- MIT licensed, no fees
- C ABI, bindings for any language
- Features: polyphonic modulation, note expressions, remote controls
- Well supported in Bitwig, Reaper, Studio One, others

### Bitwig Controller Alternative

If we ever need deeper integration:
- Java-based extension API
- Full access to tracks, clips, scenes, devices, parameters
- Event-driven model
- Debug: set `BITWIG_DEBUG_PORT` env var, attach IntelliJ
- Docs: Bitwig Dashboard → Help → Developer Resources

---

## Similar Projects / Inspiration

### OSC/PILOT
- Bidirectional OSC control surface
- Can drive Bitwig, Ableton
- Receives OSC to update UI
- Sends MIDI

### ossia score
- Intermedia sequencer
- Supports: OSC, MIDI, WebSocket, MQTT, DMX
- Open source, cross-platform

### Vezér
- Timeline-based MIDI/OSC sequencer
- Remote controllable
- macOS

### AudioGridder
- Networked plugin hosting
- Offload DSP to other machines
- Streams plugin UIs

---

## Open Questions

### Architecture - DECIDED
- [x] **Naming:** Conductor
- [x] **Connection direction:** Plugin → Conductor
- [x] **Plugin ↔ Conductor protocol:** OSC over UDP
- [x] **CLI ↔ Conductor protocol:** HTTP REST
- [x] **Conductor lifecycle:** Daemon + CLI wrapper
- [x] **Plugin GUI:** Yes, shows version labels
- [x] **History location:** Conductor (plugin is stateless)

### Synchronization
- [x] How does Conductor know transport position? → It doesn't. Plugin handles timing.
- [x] Immediate vs beat 1? → **If playing: beat 1. If stopped: immediate.**
- [x] Conductor timing? → **Send COMMIT to all plugins as fast as possible** (tight loop)
- [ ] What if a plugin doesn't ACK prepare? (Timeout? Abort? Proceed without?)

### Musical Features (later)
- [ ] Polyphony - when to add chord support?
- [ ] Pattern length - fixed bars or variable?
- [ ] Quantization - snap to grid or free timing?
- [ ] Scale/key awareness - constrain notes to scale?
- [ ] How many versions to keep? Persist to disk?

### User Experience
- [x] How does user "go back to v10"? → Conductor stores history, replays as new PREPARE
- [x] Visual feedback in plugin UI? → Yes, current + staged version labels
- [ ] Error reporting - how does user know if something failed?
- [ ] What happens if Conductor restarts? (Reconnection, state recovery)

---

## Claude Opus 4.5 & Long-Running Sessions

### Research conducted Dec 2025

### Opus 4.5 Strengths (What I'm Good At)

| Strength | Application to Vibewig |
|----------|----------------------|
| **Multi-step autonomous tasks** | Building full system across sessions |
| **80.9% SWE-bench** | Real-world software engineering |
| **Fewer dead-ends in agentic workflows** | Less backtracking on architecture |
| **Best vision model (80.7% MMMU)** | Can analyze diagrams, mockups |
| **Robust to prompt injection** | Reliable tool execution |
| **Extended context with compaction** | Long sessions without degradation |

### What I'm Fast At
- Code generation and refactoring
- Architecture design and planning
- Multi-file changes with consistency
- Backend systems (stronger than UI-heavy frontend)

### Prompt Translation Strategy

For this project, prompts should be translated through these lenses:

**1. Intent Extraction**
- What does the human want to hear/experience?
- What musical outcome are they after?
- Is this a creative direction or a technical fix?

**2. Technical Mapping**
- Which plugins are affected?
- What OSC messages need to be sent?
- What state changes in Conductor?

**3. Verification Planning**
- How will we know it worked?
- What should the plugin UI show?
- What should Claude report back?

**Example Translation:**
```
Human: "make it brighter and faster"

Translation:
- Intent: Increase energy, higher register, shorter notes
- Technical:
  - Shift notes up an octave (+12 semitones)
  - Halve durations
  - Maybe increase velocity
- Verification:
  - Plugin shows new version label
  - Claude reports "v7-bright-fast: raised octave, doubled tempo feel"
```

### Long-Running Session Best Practices

**From Anthropic's research:**

1. **Progress Files** - Maintain external artifacts as memory
   - `claude-progress.txt` for session handoffs
   - Git commits as incremental checkpoints
   - Structured feature lists

2. **Two-Agent Pattern**
   - Initializer: Sets up environment
   - Coding agent: Makes incremental progress per session

3. **Avoiding Regression**
   - Tests are critical - they prevent "works for me" lies
   - Commit often, small commits
   - Session boundaries are checkpoints
   - Don't tackle too much at once

4. **Context Hygiene**
   - `/clear` between unrelated tasks
   - Use subagents for isolated exploration
   - Keep CLAUDE.md under 300 lines
   - Task-specific docs in separate files

5. **Extended Thinking**
   - "think" → basic analysis
   - "think hard" → deeper exploration
   - "think harder" → comprehensive analysis
   - "ultrathink" → maximum depth

### Self-Refinement Loop for Vibewig

```
Human prompt
    │
    ▼
┌─────────────────────────────────────┐
│ 1. UNDERSTAND                       │
│    - What outcome does human want?  │
│    - Musical or technical?          │
│    - Urgent or exploratory?         │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 2. TRANSLATE                        │
│    - Map to system concepts         │
│    - Identify affected components   │
│    - Generate version label         │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 3. EXECUTE                          │
│    - Call tools in correct order    │
│    - Stage → Commit flow            │
│    - Handle errors gracefully       │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 4. VERIFY                           │
│    - Check tool responses           │
│    - Confirm expected state         │
│    - Report clearly to human        │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 5. LEARN                            │
│    - Did it work as expected?       │
│    - Update KNOWLEDGE.md if needed  │
│    - Note patterns for future       │
└─────────────────────────────────────┘
```

### Project-Specific Prompt Patterns

**For musical changes:**
```
"[mood/energy] [direction]"
→ Translate to: notes, durations, velocities, which tracks
```

**For system changes:**
```
"[component] should [behavior]"
→ Translate to: code changes, tests, documentation
```

**For debugging:**
```
"[something] isn't working"
→ First: query status, check logs
→ Then: diagnose, propose fix
```

**For rollback:**
```
"go back to [label]"
→ Conductor lookup → PREPARE old state → COMMIT
```

---

## Session Management Infrastructure

### Research Dec 2025

Based on Anthropic's engineering posts and community patterns, vibewig uses a multi-artifact system for maintaining consistency across sessions.

### File Structure for Persistence

```
vibewig/
├── .claude/
│   ├── commands/           # Custom slash commands
│   │   ├── catchup.md      # Read changed files, restore context
│   │   ├── status.md       # Query system state
│   │   └── commit.md       # Stage, commit, update progress
│   ├── context/            # Session context files
│   │   └── current.md      # Current session context
│   └── settings.json       # Project permissions
├── docs/
│   ├── progress.md         # What's done, what's next
│   ├── decisions.md        # Why we made choices (immutable log)
│   └── features.json       # Structured feature tracking
├── CLAUDE.md               # Universal project conventions (< 300 lines)
└── KNOWLEDGE.md            # Deep knowledge, research, patterns
```

### Progress Tracking (docs/progress.md)

```markdown
# Vibewig Progress

## Current Session
- Started: [timestamp]
- Focus: [what we're working on]
- Branch: [git branch]

## Completed
- [x] Architecture decisions
- [x] Protocol design (OSC)
- [x] Plugin state machine

## In Progress
- [ ] Conductor daemon implementation
- [ ] Plugin OSC client

## Next Up
- [ ] CLI wrapper
- [ ] Integration tests

## Blockers
- None currently

## Session Handoff Notes
[Claude writes notes here before session ends]
```

### Feature Tracking (docs/features.json)

```json
{
  "version": 1,
  "features": [
    {
      "id": "conductor-daemon",
      "category": "core",
      "description": "Long-running daemon that manages plugins",
      "status": "in_progress",
      "files": ["vibewig-conductor/src/main.rs"],
      "tests": [],
      "notes": "HTTP server + OSC client"
    },
    {
      "id": "plugin-osc",
      "category": "core",
      "description": "Plugin connects to Conductor via OSC",
      "status": "pending",
      "files": ["vibewig-plugin/src/lib.rs"],
      "tests": [],
      "notes": "Replace WebSocket with OSC"
    }
  ]
}
```

### Custom Slash Commands

**/.claude/commands/catchup.md**
```markdown
Read the following files to restore session context:
1. docs/progress.md - current state and next steps
2. docs/features.json - feature status
3. git log -10 --oneline - recent commits
4. git diff --stat HEAD~5 - recent file changes

Summarize what's been done and what's next.
```

**/.claude/commands/handoff.md**
```markdown
Before ending this session:
1. Update docs/progress.md with current state
2. Commit any uncommitted work with clear message
3. Note any blockers or important context
4. Update docs/features.json if feature status changed

This ensures the next session can pick up seamlessly.
```

**/.claude/commands/status.md**
```markdown
Report the current state:
1. Which features are complete vs in-progress?
2. Any failing tests?
3. Any uncommitted changes?
4. What was the last thing we were working on?
```

### GitHub Issues Integration

Use GitHub Issues as source of truth for larger tasks:
- Each epic/milestone = one Issue
- Issue body contains acceptance criteria
- Comments provide history
- Labels for status: `in-progress`, `blocked`, `ready-for-review`

**Pattern:** Issue → local progress.md → code → commit → update Issue

### Checkpointing Strategy

1. **Every 30 mins or major milestone:** Git commit
2. **Before context switch:** Run `/handoff`
3. **Start of session:** Run `/catchup`
4. **After completing feature:** Update features.json

### Avoiding Regression

From Anthropic research:
- "Tests are remarkably effective at preventing regressions"
- "Session boundaries prevent regressions — commit often"
- "Don't try to tackle too much at once"

**Rules for vibewig:**
- Never skip tests for "speed"
- Each commit should be atomic and working
- If something breaks, fix before moving on
- Update progress.md before ending session

---

## Resources

### Documentation
- [nih-plug docs](https://nih-plug.robbertvanderhelm.nl/nih_plug/)
- [nih-plug examples](https://github.com/robbert-vdh/nih-plug/tree/master/plugins)
- [CLAP spec](https://github.com/free-audio/clap)
- [crossbeam-channel](https://docs.rs/crossbeam-channel/)

### Tutorials
- [Writing a CLAP Synth in Rust](https://kwarf.com/2025/03/writing-a-clap-synthesizer-in-rust-part-3/)
- [Bitwig Controller Tutorial](https://github.com/outterback/bitwig-controller-tutorial)
- [Rust Audio Resources](https://rust.audio/)

### Community
- [CLAP Database](https://clapdb.tech/)
- [KVR Audio Forums](https://www.kvraudio.com/forum/)

---

## Autonomous Building

### Overview

Vibewig supports autonomous building sessions where Claude Code works independently without human interaction. This requires proper setup of permissions and context restoration.

### Prerequisites

**.claude/settings.json** - Permissions for autonomous work:
```json
{
  "permissions": {
    "allow": [
      "Bash(cargo:*)",
      "Bash(git:*)",
      "Bash(gh:*)",
      "Bash(./scripts/*)",
      "Bash(cp:*)",
      "Bash(mkdir:*)",
      "Bash(chmod:*)",
      "Bash(ls:*)",
      "Bash(pwd)",
      "Bash(which:*)",
      "Read",
      "Write",
      "Edit"
    ],
    "deny": [
      "Bash(rm -rf /)",
      "Bash(rm -rf ~)",
      "Bash(git push --force)",
      "Bash(git reset --hard)"
    ]
  }
}
```

### Autonomous Session Protocol

Use `/autonomous` slash command to start a focused building session:

**1. Restore Context**
- Read `docs/progress.md` - current state
- Read `docs/features.json` - feature status
- Read `KNOWLEDGE.md` - design decisions
- Run `git log -5 --oneline` - recent history

**2. Identify Next Task**
- Find next `pending` or `partial` feature in `docs/features.json`
- If current feature is `in_progress`, continue it
- Make reasonable decisions based on documented architecture

**3. Implement**
- Follow architecture: Conductor (HTTP+OSC), Plugin (OSC+GUI), CLI (HTTP)
- Use OSC over UDP for plugin communication
- Use HTTP REST for CLI to Conductor
- Commit often with clear messages
- Run `cargo clippy` and `cargo fmt` before commits

**4. Test**
- Run `cargo test`
- Run `cargo build --release`
- Fix any failures before continuing

**5. Update State**
Before stopping:
- Update `docs/progress.md` with what was done
- Update `docs/features.json` if feature status changed
- Commit with "Session handoff: [summary]"

**6. Rules**
- Do NOT ask questions—make reasonable decisions
- Do NOT skip tests for speed
- Do NOT leave uncommitted work
- Do NOT break existing functionality

**7. Report**
When done, summarize:
- What was implemented
- What's next
- Any blockers discovered

### Building the Plugin

```bash
# Build all components
cargo build --release

# Plugin location after build
ls -la target/bundled/
```

The built plugin (`.clap` file) goes in:
- **macOS:** `~/Library/Audio/Plug-Ins/CLAP/`
- **Windows:** `C:\Program Files\Common Files\CLAP\`
- **Linux:** `~/.clap/`

### Installing in Bitwig

1. Build: `cargo build --release`
2. Copy `.clap` to plugin folder (see paths above)
3. In Bitwig: Settings → Locations → Rescan
4. Add to track: Browser → Devices → Vibewig

---

## Changelog

**2025-12-30**
- Initial research: CLAP vs Controller, nih-plug patterns, real-time safety
- Created project scaffold
- Refined architecture: two-phase commit model (prepare → commit on beat 1)
- Added component responsibility matrix
- Added plugin state machine diagram
- Added version labeling concept for traceability
- **Decisions made:**
  - Component name: Conductor
  - Connection: Plugin → Conductor (plugin dials out)
  - Plugin ↔ Conductor: OSC over UDP
  - CLI ↔ Conductor: HTTP REST
  - Conductor lifecycle: Daemon + CLI wrapper
  - Plugin GUI: Yes, shows current + staged version labels
  - History: Conductor stores, plugin is stateless
  - Version recall: Claude enriches version list with creative context
- **Protocol additions:**
  - `/vibewig/cancel` - delete staged program, keep current playing
  - `/vibewig/mute` - mute/unmute current output (notes off, program stays)
  - `/vibewig/status` - plugin heartbeat with current state
  - Updated state machine with MUTED state and CANCEL transition
- **Timing:** If stopped → immediate. If playing → beat 1.
- **Design principles:** Performance > Reliability > Debuggability > Testability
- **Debug CLI:** `vibewig status`, `vibewig history`, `vibewig log`
