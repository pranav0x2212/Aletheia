use crate::engine::{ExecutionResult, MemoryEngine, Operation};

pub struct DatasetScan {
    pub threshold: u32,
}

impl DatasetScan {
    pub fn new(threshold: u32) -> Self {
        DatasetScan { threshold }
    }

    pub fn execute_cpu(&self, engine: &MemoryEngine, buffer_idx: usize) -> ExecutionResult {
        engine.execute_cpu(Operation::MemScan, &[buffer_idx], &[self.threshold])
    }

    pub fn execute_memory_engine(
        &self,
        engine: &MemoryEngine,
        buffer_idx: usize,
    ) -> ExecutionResult {
        engine.execute_memory_engine(Operation::MemScan, &[buffer_idx], &[self.threshold])
    }

    pub fn compare_modes(&self, engine: &MemoryEngine, buffer_idx: usize) -> ScanComparison {
        let cpu_result = self.execute_cpu(engine, buffer_idx);
        let mem_result = self.execute_memory_engine(engine, buffer_idx);

        ScanComparison {
            cpu_result: cpu_result.clone(),
            mem_result: mem_result.clone(),
            reduction_ratio: if cpu_result.stats.data_moved > 0 {
                (cpu_result.stats.data_moved as f64) / (mem_result.stats.data_moved as f64)
            } else {
                1.0
            },
        }
    }
}

pub struct ScanComparison {
    pub cpu_result: ExecutionResult,
    pub mem_result: ExecutionResult,
    pub reduction_ratio: f64,
}

impl ScanComparison {
    pub fn print_summary(&self) {
        println!("Dataset Scan Comparison");
        println!("=======================");
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
        println!("Data Movement Reduction: {:.2}x", self.reduction_ratio);
        println!(
            "Results Match: {}",
            self.cpu_result.data == self.mem_result.data
        );
    }
}
