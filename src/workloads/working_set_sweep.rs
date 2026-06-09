use crate::engine::{MemoryEngine, Operation, ExecutionResult};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct WorkingSetSweep {
    pub working_set_sizes: Vec<usize>, // in bytes
    pub iterations: usize,             // iterations for pointer chasing
    pub warmup_iterations: usize,      // warmup before measurement
}

impl WorkingSetSweep {
    pub fn new() -> Self {
        // Sizes in bytes: 1KB to 64MB
        let sizes_kb = vec![1, 4, 16, 32, 64, 128, 256, 512, 1024, 4096, 16384, 65536];
        let working_set_sizes = sizes_kb.iter().map(|kb| kb * 1024).collect();

        WorkingSetSweep {
            working_set_sizes,
            iterations: 100_000,      // 100K iterations to reduce noise
            warmup_iterations: 10_000, // 10K warmup iterations
        }
    }

    /// Generate a randomized pointer chain for the buffer
    pub fn generate_pointer_chain(buffer_size: usize) -> Vec<u32> {
        let n = buffer_size as u32;
        let mut indices: Vec<u32> = (0..n).collect();
        let mut rng = thread_rng();
        indices.shuffle(&mut rng);
        indices
    }

    /// Generate random indices for independent random access
    pub fn generate_random_indices(buffer_size: usize) -> Vec<u32> {
        let n = buffer_size as u32;
        let mut indices: Vec<u32> = (0..n).collect();
        let mut rng = thread_rng();
        indices.shuffle(&mut rng);
        indices
    }

    /// Get scaled iteration count based on working set size to reduce measurement noise
    /// Small sets (L1/L2): baseline, Medium (L3): 3x, Large (DRAM): 10x
    pub fn get_scaled_iterations(working_set_bytes: usize) -> usize {
        const L2_BOUNDARY: usize = 256 * 1024;      // 256KB
        const L3_BOUNDARY: usize = 4 * 1024 * 1024; // 4MB
        const L1_BASELINE: usize = 100_000;
        const L3_MULTIPLIER: usize = 3;
        const DRAM_MULTIPLIER: usize = 10;

        if working_set_bytes <= L2_BOUNDARY {
            L1_BASELINE
        } else if working_set_bytes <= L3_BOUNDARY {
            L1_BASELINE * L3_MULTIPLIER
        } else {
            L1_BASELINE * DRAM_MULTIPLIER
        }
    }

    /// Execute pointer chasing on CPU
    pub fn execute_cpu_pointer_chase(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
    ) -> ExecutionResult {
        engine.execute_cpu(
            Operation::MemPointerChase,
            &[buffer_idx],
            &[self.iterations as u32],
        )
    }

    /// Execute pointer chasing on memory engine
    pub fn execute_memory_engine_pointer_chase(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
    ) -> ExecutionResult {
        engine.execute_memory_engine(
            Operation::MemPointerChase,
            &[buffer_idx],
            &[self.iterations as u32],
        )
    }

    /// Execute independent random access on CPU
    pub fn execute_cpu_random_access(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
    ) -> ExecutionResult {
        engine.execute_cpu(
            Operation::MemRandomAccess,
            &[buffer_idx],
            &[self.iterations as u32],
        )
    }

    /// Execute independent random access on memory engine
    pub fn execute_memory_engine_random_access(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
    ) -> ExecutionResult {
        engine.execute_memory_engine(
            Operation::MemRandomAccess,
            &[buffer_idx],
            &[self.iterations as u32],
        )
    }

    /// Calculate per-access latency in nanoseconds
    pub fn calculate_latency_ns(runtime_ns: u128, iterations: usize) -> f64 {
        (runtime_ns as f64) / (iterations as f64)
    }

    pub fn compare_modes(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
        cpu_runtime_ms: u128,
        mem_runtime_ms: u128,
    ) -> WorkingSetResult {
        let cpu_result = self.execute_cpu_pointer_chase(engine, buffer_idx);
        let mem_result = self.execute_memory_engine_pointer_chase(engine, buffer_idx);

        let cpu_latency_ns = Self::calculate_latency_ns(cpu_runtime_ms, self.iterations);
        let mem_latency_ns = Self::calculate_latency_ns(mem_runtime_ms, self.iterations);

        WorkingSetResult {
            cpu_result: cpu_result.clone(),
            mem_result: mem_result.clone(),
            cpu_latency_ns,
            mem_latency_ns,
            iterations: self.iterations,
        }
    }
}

pub struct WorkingSetResult {
    pub cpu_result: ExecutionResult,
    pub mem_result: ExecutionResult,
    pub cpu_latency_ns: f64,
    pub mem_latency_ns: f64,
    pub iterations: usize,
}

impl WorkingSetResult {
    pub fn print_summary(&self, size_bytes: usize) {
        let size_kb = size_bytes / 1024;
        let size_mb = size_bytes / (1024 * 1024);

        let size_str = if size_mb > 0 {
            format!("{}MB", size_mb)
        } else {
            format!("{}KB", size_kb)
        };

        // Identify cache level
        let cache_level = if size_bytes <= 32 * 1024 {
            "→ L1"
        } else if size_bytes <= 256 * 1024 {
            "→ L2"
        } else if size_bytes <= 8 * 1024 * 1024 {
            "→ L3"
        } else {
            "→ DRAM"
        };

        println!("Working Set: {:>6}  {} ", size_str, cache_level);
        println!("  CPU Latency: {:.2} ns/access", self.cpu_latency_ns);
        println!("  Memory Engine Latency: {:.2} ns/access", self.mem_latency_ns);
        println!(
            "  Latency Ratio: {:.2}x",
            self.mem_latency_ns / self.cpu_latency_ns
        );
    }
}

