use anyhow::Result;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::{
    engine::{MemoryEngine, Operation},
    network::send_command,
    protocol::{Command, MemOp},
};

/// Where an operation should execute.
pub enum ExecutionPolicy {
    Cpu,
    RemoteNode { address: String },
}
pub struct PolicyResult {
    pub elapsed: Duration,
    pub cycles: u64,
    pub instructions: u64,
    pub cache_references: u64,
    pub cache_misses: u64,
    pub memory_access: u64,
    pub data_moved: u64,
    pub result_count: usize,
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
                    elapsed,
                    cycles: cpu_result.stats.cycles,
                    instructions: 0,
                    cache_references: 0,
                    cache_misses: 0,
                    memory_access: cpu_result.stats.memory_access,
                    data_moved: cpu_result.stats.data_moved,
                    result_count: cpu_result.data.len(),
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
                    elapsed,
                    cycles,
                    instructions: response.data.instructions,
                    cache_references: response.data.cache_references,
                    cache_misses: response.data.cache_misses,
                    memory_access: response.data.memory_access,
                    data_moved: response.data.data_moved,
                    result_count: response.data.result_count,
                })
            }
        }
    }

    pub async fn run_vec_add(
        &self,
        buf_size: usize,
        buffer_a: &str,
        buffer_b: &str,
    ) -> Result<PolicyResult> {
        match self {
            ExecutionPolicy::Cpu => {
                let mut engine = MemoryEngine::new();
                let buf_a_idx = engine.allocate_buffer(buf_size, 100);
                let buf_b_idx = engine.allocate_buffer(buf_size, 200);

                let start = Instant::now();
                let cpu_result =
                    engine.execute_cpu(Operation::MemVecAdd, &[buf_a_idx, buf_b_idx], &[]);
                let elapsed = start.elapsed();

                Ok(PolicyResult {
                    elapsed,
                    cycles: cpu_result.stats.cycles,
                    instructions: 0,
                    cache_references: 0,
                    cache_misses: 0,
                    memory_access: cpu_result.stats.memory_access,
                    data_moved: cpu_result.stats.data_moved,
                    result_count: cpu_result.data.len(),
                })
            }
            ExecutionPolicy::RemoteNode { address } => {
                let cmd = Command {
                    id: Uuid::new_v4().to_string(),
                    op: MemOp::MemVecAdd {
                        buffer_a: buffer_a.to_string(),
                        buffer_b: buffer_b.to_string(),
                    },
                };

                let start = Instant::now();
                let response = send_command(address, cmd).await?;
                let elapsed = start.elapsed();

                // Derive cycles from measured elapsed time (4 GHz CPU frequency)
                let estimated_cpu_freq_hz = 4_000_000_000.0;
                let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;

                Ok(PolicyResult {
                    elapsed,
                    cycles,
                    instructions: response.data.instructions,
                    cache_references: response.data.cache_references,
                    cache_misses: response.data.cache_misses,
                    memory_access: response.data.memory_access,
                    data_moved: response.data.data_moved,
                    result_count: response.data.result_count,
                })
            }
        }
    }

    pub async fn run_stride_scan(&self, buf_size: usize, stride: usize) -> Result<PolicyResult> {
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
                let cpu_result =
                    engine.execute_cpu(Operation::MemStrideScan, &[buf], &[stride as u32]);
                let elapsed = start.elapsed();

                Ok(PolicyResult {
                    elapsed,
                    cycles: cpu_result.stats.cycles,
                    instructions: 0,
                    cache_references: 0,
                    cache_misses: 0,
                    memory_access: cpu_result.stats.memory_access,
                    data_moved: cpu_result.stats.data_moved,
                    result_count: cpu_result.data.len(),
                })
            }
            ExecutionPolicy::RemoteNode { address } => {
                let cmd = Command {
                    id: Uuid::new_v4().to_string(),
                    op: MemOp::MemStrideScan {
                        buffer: "dataset".to_string(),
                        stride,
                    },
                };

                let start = Instant::now();
                let response = send_command(address, cmd).await?;
                let elapsed = start.elapsed();

                // Derive cycles from measured elapsed time (4 GHz CPU frequency)
                let estimated_cpu_freq_hz = 4_000_000_000.0;
                let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;

                Ok(PolicyResult {
                    elapsed,
                    cycles,
                    instructions: response.data.instructions,
                    cache_references: response.data.cache_references,
                    cache_misses: response.data.cache_misses,
                    memory_access: response.data.memory_access,
                    data_moved: response.data.data_moved,
                    result_count: response.data.result_count,
                })
            }
        }
    }

    pub async fn run_pointer_chase(
        &self,
        buf_size: usize,
        buffer: &str,
        iterations: usize,
    ) -> Result<PolicyResult> {
        match self {
            ExecutionPolicy::Cpu => {
                let mut engine = MemoryEngine::new();
                let buf = engine.allocate_buffer(buf_size, 0);

                // Initialize pointer chain (each element points to next in chain)
                if let Some(buf_data) = engine.get_buffer_mut(buf) {
                    for (i, value) in buf_data.iter_mut().enumerate().take(buf_size) {
                        let next =
                            ((i as u32).wrapping_mul(12345).wrapping_add(67890)) % (buf_size as u32);
                        *value = next;
                    }
                }

                let start = Instant::now();
                let cpu_result =
                    engine.execute_cpu(Operation::MemPointerChase, &[buf], &[iterations as u32]);
                let elapsed = start.elapsed();

                Ok(PolicyResult {
                    elapsed,
                    cycles: cpu_result.stats.cycles,
                    instructions: 0,
                    cache_references: 0,
                    cache_misses: 0,
                    memory_access: cpu_result.stats.memory_access,
                    data_moved: cpu_result.stats.data_moved,
                    result_count: cpu_result.data.len(),
                })
            }
            ExecutionPolicy::RemoteNode { address } => {
                let cmd = Command {
                    id: Uuid::new_v4().to_string(),
                    op: MemOp::MemPointerChase {
                        buffer: buffer.to_string(),
                        iterations,
                    },
                };

                let start = Instant::now();
                let response = send_command(address, cmd).await?;
                let elapsed = start.elapsed();

                // Derive cycles from measured elapsed time (4 GHz CPU frequency)
                let estimated_cpu_freq_hz = 4_000_000_000.0;
                let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;

                Ok(PolicyResult {
                    elapsed,
                    cycles,
                    instructions: response.data.instructions,
                    cache_references: response.data.cache_references,
                    cache_misses: response.data.cache_misses,
                    memory_access: response.data.memory_access,
                    data_moved: response.data.data_moved,
                    result_count: response.data.result_count,
                })
            }
        }
    }
}