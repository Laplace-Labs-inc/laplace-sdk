//! Harness registry — auto-discovered via `inventory` at link time.

use laplace_axiom::dpor::Operation;
use laplace_core::domain::resource::{ResourceId, ThreadId};

/// Configuration for a single verification harness.
///
/// Submitted to `inventory` by the `#[axiom_harness]` proc-macro; collected
/// here via [`inventory::collect!`].  All fields must be `'static` and the
/// type must be `Copy` so that the linker-section trick works.
#[derive(Clone, Copy)]
pub struct HarnessConfig {
    /// Registry key used for lookup (e.g. `"template_harness"`).
    pub name: &'static str,
    /// Human-readable display name shown in verify output.
    pub display_name: &'static str,
    /// Short description of what the harness verifies.
    pub description: &'static str,
    pub num_threads: usize,
    pub num_resources: usize,
    /// Stateless function pointer: maps `(thread, pc)` to the next operation.
    pub op_provider: fn(ThreadId, usize) -> Option<(Operation, ResourceId)>,
    /// Expected verdict: `"clean"` (no bugs) or `"bug"` (bug must be found).
    /// Analogous to `#[should_panic]` — harnesses with `"bug"` must produce
    /// `BugFound` to be considered passing.
    pub expected: &'static str,
}

inventory::collect!(HarnessConfig);

/// Central registry for all verification harnesses.
///
/// Harnesses are registered automatically at link time via `inventory::submit!`
/// blocks emitted by the `#[axiom_harness]` macro — no manual wiring needed.
pub struct Registry;

impl Registry {
    /// Look up a harness by its registry key.
    pub fn get(name: &str) -> anyhow::Result<HarnessConfig> {
        inventory::iter::<HarnessConfig>()
            .find(|h| h.name == name)
            .copied()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown harness '{name}'.\nAvailable harnesses: {}",
                    Self::list_names().join(", ")
                )
            })
    }

    /// Return every registered harness as `(name, config)` pairs.
    pub fn get_all() -> Vec<(&'static str, HarnessConfig)> {
        inventory::iter::<HarnessConfig>()
            .map(|h| (h.name, *h))
            .collect()
    }

    /// Return the list of registered harness names.
    pub fn list_names() -> Vec<&'static str> {
        inventory::iter::<HarnessConfig>().map(|h| h.name).collect()
    }
}
