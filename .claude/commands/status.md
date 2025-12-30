Report the current project state:

1. Read **docs/progress.md** and **docs/features.json**

2. Run:
```bash
git status
cargo check 2>&1 | head -20
```

3. Summarize:
   - Features complete vs in-progress vs pending
   - Any compilation errors?
   - Any uncommitted changes?
   - What was the last thing worked on?
   - Any blockers?
