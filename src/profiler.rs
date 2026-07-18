/// Hardware performance counters collected for a single workload execution.
#[derive(Debug, Clone)]
pub struct HardwareCounters {
    pub cycles: u64,
    pub instructions: u64,
    pub cache_references: u64,
    pub cache_misses: u64,
}

use anyhow::Context;
use perf_event::events::Hardware;
use perf_event::{Builder, Group};

/// Runs `f`, measuring CPU cycles, instructions, cache references, and
/// cache misses for exactly the duration of the closure call.
///
/// Counters are opened disabled, enabled immediately before `f` runs,
/// and disabled immediately after — so the measurement window is scoped
/// to this one invocation, not the process lifetime.
pub fn measure<F, R>(f: F) -> anyhow::Result<(R, HardwareCounters)>
where
    F: FnOnce() -> R,
{
    let mut group = Group::new().context("failed to open perf event group")?;

    let cycles = Builder::new(Hardware::CPU_CYCLES)
        .build_with_group(&mut group)
        .context("failed to open CPU_CYCLES counter")?;
    let instructions = Builder::new(Hardware::INSTRUCTIONS)
        .build_with_group(&mut group)
        .context("failed to open INSTRUCTIONS counter")?;
    let cache_references = Builder::new(Hardware::CACHE_REFERENCES)
        .build_with_group(&mut group)
        .context("failed to open CACHE_REFERENCES counter")?;
    let cache_misses = Builder::new(Hardware::CACHE_MISSES)
        .build_with_group(&mut group)
        .context("failed to open CACHE_MISSES counter")?;

    group
        .enable()
        .context("failed to enable perf event group")?;
    let result = f();
    group
        .disable()
        .context("failed to disable perf event group")?;

    let counts = group.read().context("failed to read perf event group")?;

    let counters = HardwareCounters {
        cycles: counts[&cycles],
        instructions: counts[&instructions],
        cache_references: counts[&cache_references],
        cache_misses: counts[&cache_misses],
    };

    Ok((result, counters))
}
