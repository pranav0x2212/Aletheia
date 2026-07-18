use crate::engine::{ExecutionResult, MemoryEngine, Operation};

pub struct VectorAdd;

impl VectorAdd {
    pub fn execute_cpu(engine: &MemoryEngine, buffer_a: usize, buffer_b: usize) -> ExecutionResult {
        engine.execute_cpu(Operation::MemVecAdd, &[buffer_a, buffer_b], &[])
    }

    pub fn execute_memory_engine(
        engine: &MemoryEngine,
        buffer_a: usize,
        buffer_b: usize,
    ) -> ExecutionResult {
        engine.execute_memory_engine(Operation::MemVecAdd, &[buffer_a, buffer_b], &[])
    }

    pub fn compare_modes(
        engine: &MemoryEngine,
        buffer_a: usize,
        buffer_b: usize,
    ) -> VectorAddComparison {
        let cpu_result = Self::execute_cpu(engine, buffer_a, buffer_b);
        let mem_result = Self::execute_memory_engine(engine, buffer_a, buffer_b);

        VectorAddComparison {
            cpu_result: cpu_result.clone(),
            mem_result: mem_result.clone(),
            speedup: if mem_result.stats.cycles > 0 {
                (cpu_result.stats.cycles as f64) / (mem_result.stats.cycles as f64)
            } else {
                1.0
            },
        }
    }
}

pub struct VectorAddComparison {
    pub cpu_result: ExecutionResult,
    pub mem_result: ExecutionResult,
    pub speedup: f64,
}

impl VectorAddComparison {
    pub fn print_summary(&self) {
        println!("Vector Add Comparison");
        println!("====================");
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
        println!(
            "Results Match: {}",
            self.cpu_result.data == self.mem_result.data
        );
    }
}
