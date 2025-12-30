# Voxel

*Speak music into existence*

Interactive music generation where natural language prompts produce music in Bitwig Studio.

**Website:** [audio-forge-rs.github.io/vibewig](https://audio-forge-rs.github.io/vibewig/)

---

## Architecture

```
Human prompt → Claude Code → Conductor → Plugin (Track 1)
                                 ↓
                           Plugin (Track 2)
                                 ↓
                           Bitwig → Audio
```

## Components

- **vibewig-conductor**: Daemon that orchestrates plugins, stores version history
- **vibewig-cli**: Thin CLI wrapper for sending commands
- **vibewig-plugin**: CLAP instrument plugin (Rust/nih-plug) with OSC client and GUI

## How It Works

1. You describe what you want to hear in natural language
2. Claude Code translates your intent to structured commands
3. Conductor stages the new state to all plugins (PREPARE)
4. Conductor sends a synchronized commit signal (COMMIT)
5. All plugins switch to the new program on beat 1
6. You listen, refine, iterate

Patterns persist and loop until changed. Version labels let you say "go back to the bright arpeggio" and the system knows what you mean.

## Development

```bash
cargo build --release     # Build
cargo clippy --all-targets # Lint
cargo fmt                  # Format
cargo test                 # Test
```

## Documentation

- [Essay: When to Hold On and When to Let Go](https://audio-forge-rs.github.io/vibewig/essay.html)
- [KNOWLEDGE.md](./KNOWLEDGE.md) - Architecture decisions, research, patterns
- [docs/progress.md](./docs/progress.md) - Current development state

## License

MIT

---

## Author

**Brian Edwards**
brian.mabry.edwards@gmail.com
Waco, Texas, USA

Built with Claude Opus 4.5 via Claude Code CLI 2.0.76
December 2025
