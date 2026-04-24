# Changelog

All notable changes to the laplace project are documented here.

---

## [0.8.0-beta-1] - 2026-03-15

### 🧠 Progress Intelligence — Livelock & Starvation Detection

#### Major Features

- **Livelock & Starvation Detection**: Axiom can now mathematically prove and detect thread livelock and starvation conditions via Wait-For Graph (WFG) cycle analysis and progress counter tracking. Threads exceeding `MAX_STARVATION_LIMIT` steps without resource acquisition are immediately flagged as starving and trigger a violation report.

#### Critical Bug Fixes

- **Hash Deduplication False-Negative** (`ki_state.rs:recompute_hash`): Fixed a critical bug where states differing only in starvation counter values were treated as identical by the DPOR explored-set deduplication mechanism. States now include `starvation_counters` in their hash signature, ensuring the state-space explorer properly distinguishes between states with different wait histories and prevents premature termination of exhaustive exploration.

- **Counter Reset Logic** (`ki_state.rs:update_starvation_counters`): Corrected starvation counter semantics — threads now only reset their counter to 0 when they genuinely acquire a resource and reach `Running` state. Threads that execute but remain `Blocked` (e.g., failed resource acquisition attempts, no-op operations on resources they don't own) now correctly increment their counter instead of falsely resetting, preventing starvation detection evasion through spurious state transitions.

- **Oracle Resource Wiring** (`oracle/mod.rs`, `verify.rs`): Added missing `num_resources` field to `OracleConfig` and properly wired it from harness metadata through to `KiDporScheduler::new()`. Previously, the step budget (`max_depth=1000`) was incorrectly passed as the resource count parameter, causing model state initialization inconsistencies and incorrect resource slot allocation.

#### Tuning & Optimization

- **MAX_STARVATION_LIMIT**: Set to `10` (from initial placeholder `50`) to ensure starvation detection fires within the exhaustive DPOR 1000-step exploration budget. With this tuning, detection of starvation scenarios occurs within 15-20 states on the critical path, enabling reliable bug discovery in bounded exploration time.

#### Verification Status

- ✅ All 23 registered harnesses pass exhaustive DPOR verification, including newly-enabled detection of:
  - `deadpool_abba_partial_deadlock` — WFG cycle detection (T0↔T1 circular wait)
  - `deadpool_three_way_deadlock` — 3-thread circular wait (T0→T1→T2→T0)
  - `deadpool_starvation_livelock` — Greedy monopolization causing T1 starvation (T0 cycles r0, T1 starves)
  - `deadpool_four_thread_contention` — Multi-thread ABBA patterns with 4 threads, 2 resources
  - `deadpool_slot_bookkeeping_clean` — Safe non-nested access baseline (no bugs expected, passes)

---

## [Unreleased] — 2026-03-15 — Axiom Formal Verification 100% Coverage & CI Integration

### Summary

Achieved 100% formal verification coverage across all 10 sub-domains of `laplace-core`
via 18 exhaustive Axiom concurrency harnesses, fully integrated into the GitHub Actions
CI/CD pipeline.

---

### Added

- **18 Axiom concurrency verification harnesses** covering 100% of `laplace-core`
  sub-domains: `memory`, `telemetry`, `pool`, `scheduler`, `tracing`, `journal`,
  `time`, `entropy`, `liveness`, `resource_abba`.
- **`#[axiom_harness(expected = "bug")]` attribute** in `laplace-macro` — enables
  intentional deadlock/starvation scenarios to be declared as expected bugs,
  allowing CI to validate that known-bad states are correctly detected without
  failing the pipeline.
- **`axiom-verification` job** in `.github/workflows/ci.yml` — installs Laplace CLI
  globally via `cargo install` and runs `laplace verify --harness all --strict`
  to validate all 18 harnesses on every push.

### Changed

- **`laplace-cli` verdict logic hardened**: harnesses annotated with
  `expected = "bug"` no longer trigger CI failure; the verdict engine correctly
  distinguishes intentional from unintentional bugs.
- **In-memory verification** (`--strict` mode): `.ard` file I/O is suppressed
  entirely, achieving 100% in-memory verification with zero disk overhead.
- **Legacy concurrency tests migrated** to `laplace-harness`: `dpor_abba_test.rs`
  and `liveness_test.rs` replaced by the formal harness suite.

### Removed

- `core_report.md` — master-plan scaffold removed now that the verification suite
  is complete.
- Legacy test files (`dpor_abba_test.rs`, `liveness_test.rs`) — superseded by the
  Axiom harness suite.

---

## [Phase 2.1] — 2026-03-03 — Axiom 엔진 DX 극대화, 1:1 동치성 증명 및 DPOR Replay 파이프라인 개통

### Phase 2.1 Summary

Phase 2.1 hardened the Axiom deterministic simulation engine (`laplace-core`)
across three axes: developer experience, runtime overhead reduction, and
formal correctness.

---

### Sprint 1 — Axiom Abstraction Layer (Builder DX)

**Added**
- `simulation/hooks.rs` — `SimulationObserver`, `VirtualEnvPlugin`, `NullObserver`,
  `StepOutcome`, `SimReport` extracted into a dedicated module.
- `simulation/facade.rs` — `EventDispatcher`, `Simulator<MB, CB>`, `TwinSimulator`
  (new high-level facade with observer broadcasting), `TracingAdapter` trait.
- `simulation/builder.rs` — `TwinSimulatorBuilder<S>` typestate builder with states
  `Unconfigured → MemoryReady → SchedulerReady → FullyConfigured`.
  Backward-compatible `ProductionSimulatorBuilder` and `VerificationSimulatorBuilder` retained.

**Changed**
- `simulation/mod.rs` refactored from a 793-line monolith into four focused sub-modules.
  All existing re-exports preserved for backward compatibility.

**Highlights**
- `TwinSimulatorBuilder` enforces configuration order at compile time via
  `PhantomData` typestate markers — configuration errors surface as type errors,
  not runtime panics.
- `TwinSimulator` broadcasts lifecycle events to all registered `SimulationObserver`s
  without any knowledge of the concrete observer type.

---

### Sprint 2 — Overhead Reduction

**Changed**
- `scheduler/production.rs` — `state_counts()` O(n) → O(1) via `AtomicUsize`
  counters maintained incrementally on every `set_state()` call.
- `scheduler/verification.rs` — same O(1) optimisation via `Cell<usize>` counters.
- `time/mod.rs` — `VirtualClock.next_event_id` changed from `Arc<AtomicU64>` to
  plain `AtomicU64`, eliminating one pointer indirection per event-id allocation.

---

### Sprint 3 — Mathematical Equivalence Proofs

**Added**
- `simulation/equivalence.rs` — `EquivalentTo<Spec>` bijection trait and TLA+
  specification mirror types: `StoreBufferEntry`, `ThreadStateEnum`, `VirtualTime`,
  `LamportTimestamp`, `TwinState`.
- Kani proof harnesses H-SIM1 through H-SIM5 proving lossless round-trips for
  all core type bijections and full memory-content preservation through the
  `TwinState` equivalence.
- `tests/twin_integration_test.rs` — 7 integration tests covering the full
  `TwinSimulatorBuilder` lifecycle, observer pipeline, and `SimReport` consistency.

---

### Sprint 4 — IP Protection, Documentation & Examples

**Added**
- `examples/quickstart_twin.rs` — minimal runnable example demonstrating
  deterministic environment setup with `TwinSimulatorBuilder`.
- `docs/twin_observer_guide.md` — integration guide for external modules
  connecting to the Axiom engine via `SimulationObserver` / `VirtualEnvPlugin`.
- `CHANGELOG.md` — this file.

**Changed**
- Applied hybrid commenting standard across `builder.rs`, `facade.rs`,
  `hooks.rs`, `equivalence.rs`: TLA+ formulae and internal algorithmic details
  demoted from `///` (public rustdoc) to `//` (source-only), preventing
  confidential spec mappings from appearing in generated documentation.

---

### Verification Status

| Check | Result |
|-------|--------|
| `cargo check -p laplace-core --all-features` | ✅ 0 errors, 0 warnings |
| `cargo test -p laplace-core --all-features` | ✅ all tests pass |
| `cargo test --test twin_integration_test` | ✅ 7/7 pass |
| Kani H-SIM1…H-SIM5 | ✅ all proofs verified |
