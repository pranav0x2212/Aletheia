use crate::engine::{MemoryEngine, Operation};

pub struct Executor {
    engine: MemoryEngine,
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub cpu_cycles: u64,
    pub mem_cycles: u64,
    pub speedup: f64,
    pub data_reduction: f64,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            engine: MemoryEngine::new(),
        }
    }

    pub fn engine(&self) -> &MemoryEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut MemoryEngine {
        &mut self.engine
    }

    pub fn benchmark_operation(
        &self,
        name: &str,
        operation: Operation,
        buffer_indices: &[usize],
        params: &[u32],
    ) -> BenchmarkResult {
        let cpu_result = self.engine.execute_cpu(operation, buffer_indices, params);
        let mem_result = self
            .engine
            .execute_memory_engine(operation, buffer_indices, params);

        let speedup = if mem_result.stats.cycles > 0 {
            (cpu_result.stats.cycles as f64) / (mem_result.stats.cycles as f64)
        } else {
            1.0
        };

        let data_reduction = if cpu_result.stats.data_moved > 0 {
            (cpu_result.stats.data_moved as f64) / (mem_result.stats.data_moved as f64)
        } else {
            1.0
        };

        BenchmarkResult {
            name: name.to_string(),
            cpu_cycles: cpu_result.stats.cycles,
            mem_cycles: mem_result.stats.cycles,
            speedup,
            data_reduction,
        }
    }

    pub fn run_suite(&self) -> Vec<BenchmarkResult> {
        let operations = vec![
            "MEM_COPY",
            "MEM_VEC_ADD",
            "MEM_VEC_AND",
            "MEM_VEC_OR",
            "MEM_SCAN",
        ];

        operations
            .iter()
            .enumerate()
            .map(|(_i, name)| {
                let op = match *name {
                    "MEM_COPY" => Operation::MemCopy,
                    "MEM_VEC_ADD" => Operation::MemVecAdd,
                    "MEM_VEC_AND" => Operation::MemVecAnd,
                    "MEM_VEC_OR" => Operation::MemVecOr,
                    "MEM_SCAN" => Operation::MemScan,
                    _ => Operation::MemCopy,
                };

                let buffers = if matches!(op, Operation::MemCopy) {
                    vec![0]
                } else if matches!(op, Operation::MemScan) {
                    vec![0]
                } else {
                    vec![0, 1]
                };

                let params = if matches!(op, Operation::MemScan) {
                    vec![500]
                } else {
                    vec![]
                };

                self.benchmark_operation(name, op, &buffers, &params)
            })
            .collect()
    }

    pub fn print_benchmark_summary(&self, results: &[BenchmarkResult]) {
        println!(
            "\n{:<15} {:<12} {:<12} {:<10} {:<15}",
            "Operation", "CPU Cycles", "Mem Cycles", "Speedup", "Data Reduction"
        );
        println!("{}", "=".repeat(70));
        for result in results {
            println!(
                "{:<15} {:<12} {:<12} {:<10.2}x {:<15.2}x",
                result.name,
                result.cpu_cycles,
                result.mem_cycles,
                result.speedup,
                result.data_reduction
            );
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
