Build Voxel autonomously. Follow this protocol:

## 1. Restore Context
Read these files to understand current state:
- docs/progress.md
- docs/features.json
- KNOWLEDGE.md (Design Decisions section)
- git log -5 --oneline

## 2. Identify Next Task
From docs/features.json, find the next feature with status "pending" or "partial".
If current feature is "in_progress", continue it.

## 3. Implement
- Make reasonable decisions based on KNOWLEDGE.md
- Follow the architecture: Conductor (HTTP+OSC), Plugin (OSC+GUI), CLI (HTTP)
- Use OSC over UDP for plugin communication
- Use HTTP REST for CLI to Conductor
- Commit often with clear messages
- Run `cargo clippy` and `cargo fmt` before commits

## 4. Test
- Run `cargo test`
- Run `cargo build --release`
- If build fails, fix before continuing

## 5. Update State
Before stopping:
- Update docs/progress.md with what was done
- Update docs/features.json if feature status changed
- Commit with "Session handoff: [summary]"

## 6. Do Not
- Ask questions—make reasonable decisions
- Skip tests for speed
- Leave uncommitted work
- Break existing functionality

## 7. Report
When done, summarize:
- What was implemented
- What's next
- Any blockers discovered
