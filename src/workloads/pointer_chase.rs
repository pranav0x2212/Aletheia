use crate::engine::{ExecutionResult, MemoryEngine, Operation};

pub struct PointerChase {
    pub iterations: usize,
}

impl PointerChase {
    pub fn new(iterations: usize) -> Self {
        PointerChase { iterations }
    }

    /// Initialize buffer with pointer chain
    /// Each element points to the next, forming a linked list pattern
    pub fn init_pointer_chain(engine: &mut MemoryEngine, size: usize, seed: u32) -> usize {
        let buf = engine.allocate_buffer(size, 0);

        if let Some(buf_data) = engine.get_buffer_mut(buf) {
            // Create a shuffled pointer chain: each element i points to some next index
            // Using a pseudorandom permutation for cache-unfriendly access pattern
            for i in 0..size {
                let next = ((i as u32).wrapping_mul(seed).wrapping_add(12345)) % (size as u32);
                buf_data[i] = next;
            }
        }

        buf
    }

    pub fn execute_cpu(&self, engine: &MemoryEngine, buffer: usize) -> ExecutionResult {
        engine.execute_cpu(
            Operation::MemPointerChase,
            &[buffer],
            &[self.iterations as u32],
        )
    }

    pub fn execute_memory_engine(&self, engine: &MemoryEngine, buffer: usize) -> ExecutionResult {
        engine.execute_memory_engine(
            Operation::MemPointerChase,
            &[buffer],
            &[self.iterations as u32],
        )
    }

    pub fn compare_modes(&self, engine: &MemoryEngine, buffer: usize) -> PointerChaseComparison {
        let cpu_result = self.execute_cpu(engine, buffer);
        let mem_result = self.execute_memory_engine(engine, buffer);

        PointerChaseComparison {
            cpu_result: cpu_result.clone(),
            mem_result: mem_result.clone(),
            speedup: if mem_result.stats.cycles > 0 {
                (cpu_result.stats.cycles as f64) / (mem_result.stats.cycles as f64)
            } else {
                1.0
            },
            latency_reduction: if cpu_result.stats.cycles > 0 {
                ((cpu_result.stats.cycles as f64 - mem_result.stats.cycles as f64)
                    / cpu_result.stats.cycles as f64)
                    * 100.0
            } else {
                0.0
            },
        }
    }
}

pub struct PointerChaseComparison {
    pub cpu_result: ExecutionResult,
    pub mem_result: ExecutionResult,
    pub speedup: f64,
    pub latency_reduction: f64,
}

impl PointerChaseComparison {
    pub fn print_summary(&self) {
        println!("Pointer Chase Comparison");
        println!("========================");
        println!("CPU Mode:");
        println!("  Data Moved: {} bytes", self.cpu_result.stats.data_moved);
        println!(
            "  Memory Access: {} bytes",
            self.cpu_result.stats.memory_access
        );
        println!("  Cycles: {}", self.cpu_result.stats.cycles);
        println!();
        println!("Memory Engine Mode:");
        println!("  Data Moved: {} bytes", self.mem_result.stats.data_moved);
        println!(
            "  Memory Access: {} bytes",
            self.mem_result.stats.memory_access
        );
        println!("  Cycles: {}", self.mem_result.stats.cycles);
        println!();
        println!("Speedup: {:.2}x", self.speedup);
        println!("Latency Reduction: {:.2}%", self.latency_reduction);
        println!("Note: Pointer chasing measures true memory latency (no prefetching)");
    }
}
