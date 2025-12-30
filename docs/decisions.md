# Decision Log

Immutable record of architectural and design decisions.

---

## 2025-12-30: Component Naming

**Decision:** Call the orchestration component "Conductor"

**Rationale:** Musical metaphor - conducts the ensemble, gives the downbeat. Fits the domain.

**Alternatives considered:** Bridge, Hub, Engine, Director

---

## 2025-12-30: Connection Direction

**Decision:** Plugin connects to Conductor (not vice versa)

**Rationale:**
- Plugin dials out on load, registers itself
- Conductor is always-on server (daemon)
- Natural discovery - Conductor knows who's alive
- Plugin stays simpler

---

## 2025-12-30: Protocol - Plugin ↔ Conductor

**Decision:** OSC over UDP

**Rationale:**
- Music ecosystem standard
- Low latency
- Could integrate with TouchOSC, Max/MSP, etc.

**Trade-off:** UDP has no guaranteed delivery. Mitigation: ACKs for PREPARE, heartbeats for liveness.

---

## 2025-12-30: Protocol - CLI ↔ Conductor

**Decision:** HTTP REST

**Rationale:**
- Standard, well-understood
- Easy to debug with curl
- Good Rust libraries (axum, actix-web)

---

## 2025-12-30: Timing Model

**Decision:**
- If transport playing: switch on beat 1 of next bar
- If transport stopped: switch immediately

**Rationale:**
- When playing, musical timing matters
- When stopped, user is editing and wants immediate feedback
- Conductor sends COMMIT to all plugins in tight loop for sync

---

## 2025-12-30: Plugin Format

**Decision:** CLAP (not Bitwig Controller)

**Rationale:**
- Portable across hosts
- Rust/nih-plug is mature
- Real-time safety enforced by compiler
- No Java GC pauses
- Plugins are natural UX for Bitwig users

---

## 2025-12-30: History Ownership

**Decision:** Conductor stores version history, plugin is stateless

**Rationale:**
- Plugin only needs current + staged
- Simplifies plugin code
- Conductor can persist history to disk
- Rollback = Conductor sends old version as new PREPARE

---

## 2025-12-30: Design Principles Priority

**Decision:** Performance > Reliability > Debuggability > Testability

**Rationale:**
- Audio cannot glitch
- System must be predictable
- Need to trace issues
- Tests where practical, but not at cost of performance
