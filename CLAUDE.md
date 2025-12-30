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
