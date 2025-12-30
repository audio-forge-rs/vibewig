# Vibewig

Interactive music generation system where Claude Code prompts produce music in Bitwig Studio.

## Architecture

```
Human prompt → Claude Code → vibewig-client → [Plugin Track 1]
                                   ↓
                             [Plugin Track 2]
                                   ↓
                             Bitwig → Audio Output
```

## Components

### vibewig-plugin (CLAP, Rust/nih-plug)
- CLAP plugin format using nih-plug framework
- Runs as instrument on Bitwig tracks (2 instances for MVP)
- Embeds WebSocket server, listens for commands from client
- Syncs to host transport (tempo, time signature, play/stop state)
- Outputs MIDI notes as looping patterns that persist until changed
- MVP: Monophonic melody output per instance

### vibewig-client (Rust)
- WebSocket client that Claude Code interacts with
- Connects to multiple plugin instances simultaneously
- Sends musical commands (set notes, change patterns, etc.)
- Receives transport state from plugins

## Tech Stack
- **Language**: Rust (entire stack)
- **Plugin Framework**: nih-plug (CLAP format)
- **Communication**: WebSocket (tokio-tungstenite)
- **Audio Host**: Bitwig Studio

## Development Commands
```bash
# Build everything
cargo build --release

# Build plugin only
cargo build -p vibewig-plugin --release

# Build client only
cargo build -p vibewig-client --release

# Run clippy
cargo clippy --all-targets

# Format code
cargo fmt

# Run tests
cargo test
```

## Plugin Installation
After building, copy the `.clap` file from `target/release/` to:
- macOS: `~/Library/Audio/Plug-Ins/CLAP/`
- Linux: `~/.clap/`
- Windows: `C:\Program Files\Common Files\CLAP\`

## Message Protocol (Client ↔ Plugin)

JSON over WebSocket:

```json
// Client → Plugin: Set loop pattern
{
  "type": "set_pattern",
  "notes": [60, 62, 64, 65],  // MIDI note numbers
  "durations": [0.25, 0.25, 0.25, 0.25],  // in beats
  "velocities": [100, 80, 90, 85]
}

// Client → Plugin: Clear pattern
{
  "type": "clear"
}

// Plugin → Client: Transport state
{
  "type": "transport",
  "playing": true,
  "tempo": 120.0,
  "time_sig_num": 4,
  "time_sig_denom": 4,
  "position_beats": 4.5
}
```

## Conventions

- Use `cargo clippy` before committing - all warnings should be addressed
- Format with `cargo fmt`
- Plugin WebSocket server default port: 9001 (instance 1), 9002 (instance 2)
- All musical time values in beats (not seconds)
- MIDI note numbers use standard mapping (60 = middle C)

## Project Structure
```
vibewig/
├── Cargo.toml              # Workspace root
├── CLAUDE.md               # This file
├── vibewig-plugin/         # CLAP plugin crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── vibewig-client/         # CLI client crate
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Claude Code Interaction Style

When the user prompts about music:
1. Connect to plugins via the client
2. Translate musical intent to pattern commands
3. Send to appropriate track(s)
4. Report what was sent

Example prompts and responses:
- "play a C major arpeggio" → set_pattern with [60, 64, 67, 72]
- "make track 2 play bass notes" → send low notes to plugin on port 9002
- "stop the melody" → send clear to track 1
- "double the tempo feel" → halve the durations

---

## Best Practices (from research)

### Real-Time Audio Safety (Critical)

**DO:**
- Allocate memory only in `initialize()` - this is the ONE safe place
- Use `crossbeam-channel` bounded channels for WebSocket → audio thread communication
- Keep audio thread (`process()`) allocation-free
- Enable `assert_process_allocs` feature in debug builds to catch violations
- Use `eprintln!` for logging (nih-plug captures it), but NOT in release audio thread code

**DON'T:**
- Allocate in `process()` or `reset()` - these run on audio thread
- Use mutexes on audio thread - violates real-time safety
- Block on network I/O from audio thread
- Use `println!`/`dbg!` - nih-plug provides its own logging

### Threading Architecture

```
[WebSocket Server Thread]     [Audio Thread]
        │                           │
        ├── tungstenite accept      │
        ├── parse JSON              │
        ├── tx.try_send(msg) ──────►├── rx.try_recv()
        │   (lock-free)             ├── update pattern state
        │                           ├── output MIDI notes
        │                           │
```

- WebSocket server runs in separate thread spawned in `initialize()`
- Use bounded channel (e.g., capacity 64) - won't block audio if full
- Audio thread only does `try_recv()` - never blocks

### nih-plug Specifics

- Use `nih_plug::prelude::*` for all framework types
- Export with `nih_export_clap!()` and optionally `nih_export_vst3!()`
- Bundle with `cargo xtask bundle <package>` for proper plugin format
- CLAP ID format: reverse domain notation (e.g., `com.vibewig.plugin`)

### Alternative: Bitwig Controller (Java)

If deeper Bitwig integration needed (controlling other tracks, clips, devices):
- Controller scripts use event-driven model
- Java GC can cause timing issues
- Better API access to arrangement, remote controls, transport
- Debug with `BITWIG_DEBUG_PORT` env var + IntelliJ remote debugger
- Documentation: Bitwig Dashboard → Help → Developer Resources

Current approach (CLAP plugin) chosen for:
- Portability across hosts
- Rust memory safety + real-time guarantees
- Simpler threading model for WebSocket integration
- No GC pauses

---

## Resources

- [nih-plug docs](https://nih-plug.robbertvanderhelm.nl/nih_plug/)
- [nih-plug examples](https://github.com/robbert-vdh/nih-plug/tree/master/plugins)
- [CLAP spec](https://github.com/free-audio/clap)
- [crossbeam-channel](https://docs.rs/crossbeam-channel/)
- [Bitwig Controller Tutorial](https://github.com/outterback/bitwig-controller-tutorial)
- [Rust Audio Resources](https://rust.audio/)
