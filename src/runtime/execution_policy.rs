use anyhow::Result;
use std::time::Instant;
use uuid::Uuid;

use crate::{
    engine::{MemoryEngine, Operation},
    network::send_command,
    protocol::{Command, MemOp},
};

pub enum ExecutionPolicy {
    Cpu,
    RemoteNode { address: String },
}

pub struct PolicyResult {
    pub elapsed_ms: u128,
    pub cycles: u64,
    pub instructions: u64,
    pub cache_references: u64,
    pub cache_misses: u64,
    pub memory_access: u64,
    pub data_moved: u64,
}

impl ExecutionPolicy {
    pub async fn run_scan(&self, buf_size: usize, threshold: u32) -> Result<PolicyResult> {
        match self {
            ExecutionPolicy::Cpu => {
                let mut engine = MemoryEngine::new();
                let buf = engine.allocate_buffer(buf_size, 0);

                if let Some(buf_data) = engine.get_buffer_mut(buf) {
                    for (i, val) in buf_data.iter_mut().enumerate() {
                        *val = ((i as u32).wrapping_mul(7919)) % 1000;
                    }
                }

                let start = Instant::now();
                let cpu_result = engine.execute_cpu(Operation::MemScan, &[buf], &[threshold]);
                let elapsed = start.elapsed();

                Ok(PolicyResult {
                    elapsed_ms: elapsed.as_millis(),
                    cycles: cpu_result.stats.cycles,
                    instructions: 0,
                    cache_references: 0,
                    cache_misses: 0,
                    memory_access: cpu_result.stats.memory_access,
                    data_moved: cpu_result.stats.data_moved,
                })
            }
            ExecutionPolicy::RemoteNode { address } => {
                let cmd = Command {
                    id: Uuid::new_v4().to_string(),
                    op: MemOp::MemScan {
                        buffer: "dataset".to_string(),
                        threshold,
                    },
                };

                let start = Instant::now();
                let response = send_command(address, cmd).await?;
                let elapsed = start.elapsed();

                // Derive cycles from measured elapsed time (4 GHz CPU frequency)
                let estimated_cpu_freq_hz = 4_000_000_000.0;
                let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;

                Ok(PolicyResult {
                    elapsed_ms: elapsed.as_millis(),
                    cycles,
                    instructions: response.data.instructions,
                    cache_references: response.data.cache_references,
                    cache_misses: response.data.cache_misses,
                    memory_access: response.data.memory_access,
                    data_moved: response.data.data_moved,
                })
            }
        }
    }
}
