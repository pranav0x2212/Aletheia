use crate::engine::{MemoryEngine, Operation, ExecutionResult};

pub struct WorkingSetSweep {
    pub working_set_sizes: Vec<usize>, // in bytes
}

impl WorkingSetSweep {
    pub fn new() -> Self {
        // Sizes in bytes: 1KB to 64MB
        let sizes_kb = vec![1, 4, 16, 32, 64, 128, 256, 512, 1024, 4096, 16384, 65536];
        let working_set_sizes = sizes_kb.iter().map(|kb| kb * 1024).collect();

        WorkingSetSweep {
            working_set_sizes,
        }
    }

    pub fn execute_cpu(&self, engine: &MemoryEngine, buffer_idx: usize) -> ExecutionResult {
        engine.execute_cpu(
            Operation::MemScan,
            &[buffer_idx],
            &[500], // threshold for filtering
        )
    }

    pub fn execute_memory_engine(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
    ) -> ExecutionResult {
        engine.execute_memory_engine(
            Operation::MemScan,
            &[buffer_idx],
            &[500],
        )
    }

    pub fn compare_modes(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
        cpu_runtime_ns: u128,
        mem_runtime_ns: u128,
    ) -> WorkingSetResult {
        let cpu_result = self.execute_cpu(engine, buffer_idx);
        let mem_result = self.execute_memory_engine(engine, buffer_idx);

        // Calculate latency per access (runtime / number of elements)
        let buffer = engine.get_buffer(buffer_idx).unwrap_or(&[]);
        let num_elements = buffer.len() as f64;
        let cpu_latency_per_access = if num_elements > 0.0 {
            (cpu_runtime_ns as f64) / num_elements
        } else {
            0.0
        };
        let mem_latency_per_access = if num_elements > 0.0 {
            (mem_runtime_ns as f64) / num_elements
        } else {
            0.0
        };

        WorkingSetResult {
            cpu_result: cpu_result.clone(),
            mem_result: mem_result.clone(),
            cpu_latency_ns: cpu_latency_per_access,
            mem_latency_ns: mem_latency_per_access,
        }
    }
}

pub struct WorkingSetResult {
    pub cpu_result: ExecutionResult,
    pub mem_result: ExecutionResult,
    pub cpu_latency_ns: f64,
    pub mem_latency_ns: f64,
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

        println!("Working Set Size: {}", size_str);
        println!("  CPU Latency: {:.2} ns/access", self.cpu_latency_ns);
        println!("  Memory Engine Latency: {:.2} ns/access", self.mem_latency_ns);
        println!(
            "  Latency Ratio: {:.2}x",
            self.mem_latency_ns / self.cpu_latency_ns
        );
    }
}
