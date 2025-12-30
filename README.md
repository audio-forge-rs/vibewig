# vibewig

Interactive music generation where Claude Code prompts produce music in Bitwig Studio.

## Architecture

```
Human prompt → Claude Code → vibewig client → Plugin (Track 1)
                                    ↓
                             Plugin (Track 2)
                                    ↓
                             Bitwig → Audio
```

## Components

- **vibewig-plugin**: CLAP instrument plugin (Rust/nih-plug) with embedded WebSocket server
- **vibewig-client**: CLI tool that Claude Code uses to send musical commands

## Quick Start

### Build

```bash
cargo build --release
```

### Install Plugin

Copy the plugin to your CLAP folder:

```bash
# macOS
cp target/release/libvibewig_plugin.dylib ~/Library/Audio/Plug-Ins/CLAP/vibewig.clap

# Linux
cp target/release/libvibewig_plugin.so ~/.clap/vibewig.clap
```

### Usage

1. Load **Vibewig** as an instrument on two tracks in Bitwig
2. Set track 1 plugin port to 9001, track 2 to 9002
3. Route plugin MIDI output to synths
4. Start the client:

```bash
./target/release/vibewig
```

5. Send patterns:

```bash
> play 60 64 67 72          # C major arpeggio on both tracks
> 1:play 48 dur:1           # Low C on track 1, 1 beat duration
> 2:clear                    # Stop track 2
```

## Client Commands

| Command | Description |
|---------|-------------|
| `play <notes> [dur:X] [vel:X]` | Set pattern with MIDI notes |
| `clear` / `stop` | Clear the pattern |
| `1:` / `2:` prefix | Target specific track |
| `help` | Show help |
| `quit` | Exit |

## How It Works

1. The CLAP plugin runs a WebSocket server on a configurable port
2. The client connects to one or more plugin instances
3. Commands set looping note patterns that sync to Bitwig's transport
4. Patterns persist and loop until changed or cleared

## Development

```bash
cargo clippy --all-targets    # Lint
cargo fmt                      # Format
cargo test                     # Test
```

## License

MIT
