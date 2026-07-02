use crate::OperationStats;
pub use super::operations::{Operation};
use std::time::Instant;

pub struct MemoryEngine {
    buffers: Vec<Vec<u32>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub data: Vec<u32>,
    pub stats: OperationStats,
    pub execution_mode: ExecutionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    CPU,
    MemoryEngine,
}

impl MemoryEngine {
    pub fn new() -> Self {
        MemoryEngine {
            buffers: Vec::new(),
        }
    }

    pub fn allocate_buffer(&mut self, size: usize, value: u32) -> usize {
        let buffer = vec![value; size];
        self.buffers.push(buffer);
        self.buffers.len() - 1
    }

    pub fn get_buffer(&self, idx: usize) -> Option<&[u32]> {
        self.buffers.get(idx).map(|b| b.as_slice())
    }

    pub fn get_buffer_mut(&mut self, idx: usize) -> Option<&mut [u32]> {
        self.buffers.get_mut(idx).map(|b| b.as_mut_slice())
    }

    pub fn execute_cpu(
        &self,
        operation: Operation,
        buffer_indices: &[usize],
        params: &[u32],
    ) -> ExecutionResult {
        let start = Instant::now();
        
        let buffers: Vec<&[u32]> = buffer_indices
            .iter()
            .filter_map(|&idx| self.get_buffer(idx))
            .collect();

        let data = match operation {
            Operation::MemCopy => {
                if !buffers.is_empty() {
                    buffers[0].to_vec()
                } else {
                    Vec::new()
                }
            }
            Operation::MemVecAdd => {
                if buffers.len() >= 2 {
                    buffers[0]
                        .iter()
                        .zip(buffers[1].iter())
                        .map(|(a, b)| a.wrapping_add(*b))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemVecAnd => {
                if buffers.len() >= 2 {
                    buffers[0]
                        .iter()
                        .zip(buffers[1].iter())
                        .map(|(a, b)| a & b)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemVecOr => {
                if buffers.len() >= 2 {
                    buffers[0]
                        .iter()
                        .zip(buffers[1].iter())
                        .map(|(a, b)| a | b)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemScan => {
                if !buffers.is_empty() && !params.is_empty() {
                    let threshold = params[0];
                    buffers[0]
                        .iter()
                        .filter(|&&v| v > threshold)
                        .copied()
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemStrideScan => {
                if !buffers.is_empty() && params.len() >= 1 {
                    let n = buffers[0].len();
                    let stride = params[0] as usize;
                    let mut result = Vec::with_capacity(n);
                    for k in 0..n {
                        let idx = (k * stride) % n;
                        result.push(buffers[0][idx]);
                    }
                    result
                } else {
                    Vec::new()
                }
            }
            Operation::MemPointerChase => {
                if !buffers.is_empty() && params.len() >= 1 {
                    let buffer = buffers[0];
                    let iterations = params[0] as usize;
                    let n = buffer.len();
                    let mut result = Vec::with_capacity(iterations);
                    let mut idx = 0usize;
                    
                    for _ in 0..iterations {
                        let val = buffer[idx];
                        result.push(val);
                        idx = (val as usize) % n;
                    }
                    result
                } else {
                    Vec::new()
                }
            }
            Operation::MemRandomAccess => {
                if !buffers.is_empty() && params.len() >= 1 {
                    let buffer = buffers[0];
                    let iterations = params[0] as usize;
                    let n = buffer.len();
                    let mut result = Vec::with_capacity(iterations);
                    
                    // Linear iteration through pre-shuffled indices stored in buffer
                    // (no dependency like pointer chase, but random spatial access)
                    for i in 0..iterations {
                        let idx = buffer[i % n] as usize % n;
                        result.push(buffer[idx]);
                    }
                    result
                } else {
                    Vec::new()
                }
            }
        };

        let elapsed = start.elapsed();
        // Derive cycles from elapsed time using estimated CPU frequency (~4 GHz)
        let estimated_cpu_freq_hz = 4_000_000_000.0;
        let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;
        let data_moved = data.len() as u64 * 4;
        let memory_access = buffers.iter().map(|b| b.len() as u64).sum::<u64>() * 4;

        ExecutionResult {
            data,
            stats: OperationStats {
                cycles,
                memory_access,
                data_moved,
            },
            execution_mode: ExecutionMode::CPU,
        }
    }

    pub fn execute_memory_engine(
        &self,
        operation: Operation,
        buffer_indices: &[usize],
        params: &[u32],
    ) -> ExecutionResult {
        let start = Instant::now();
        
        let buffers: Vec<&[u32]> = buffer_indices
            .iter()
            .filter_map(|&idx| self.get_buffer(idx))
            .collect();

        // Memory engine execution: same operations but conceptually "at memory"
        let data = match operation {
            Operation::MemCopy => {
                if !buffers.is_empty() {
                    buffers[0].to_vec()
                } else {
                    Vec::new()
                }
            }
            Operation::MemVecAdd => {
                if buffers.len() >= 2 {
                    buffers[0]
                        .iter()
                        .zip(buffers[1].iter())
                        .map(|(a, b)| a.wrapping_add(*b))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemVecAnd => {
                if buffers.len() >= 2 {
                    buffers[0]
                        .iter()
                        .zip(buffers[1].iter())
                        .map(|(a, b)| a & b)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemVecOr => {
                if buffers.len() >= 2 {
                    buffers[0]
                        .iter()
                        .zip(buffers[1].iter())
                        .map(|(a, b)| a | b)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemScan => {
                if !buffers.is_empty() && !params.is_empty() {
                    let threshold = params[0];
                    buffers[0]
                        .iter()
                        .filter(|&&v| v > threshold)
                        .copied()
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Operation::MemStrideScan => {
                if !buffers.is_empty() && params.len() >= 1 {
                    let n = buffers[0].len();
                    let stride = params[0] as usize;
                    let mut result = Vec::with_capacity(n);
                    for k in 0..n {
                        let idx = (k * stride) % n;
                        result.push(buffers[0][idx]);
                    }
                    result
                } else {
                    Vec::new()
                }
            }
            Operation::MemPointerChase => {
                if !buffers.is_empty() && params.len() >= 1 {
                    let buffer = buffers[0];
                    let iterations = params[0] as usize;
                    let n = buffer.len();
                    let mut result = Vec::with_capacity(iterations);
                    let mut idx = 0usize;
                    
                    for _ in 0..iterations {
                        let val = buffer[idx];
                        result.push(val);
                        idx = (val as usize) % n;
                    }
                    result
                } else {
                    Vec::new()
                }
            }
            Operation::MemRandomAccess => {
                if !buffers.is_empty() && params.len() >= 1 {
                    let buffer = buffers[0];
                    let iterations = params[0] as usize;
                    let n = buffer.len();
                    let mut result = Vec::with_capacity(iterations);
                    
                    // Linear iteration through pre-shuffled indices stored in buffer
                    // (no dependency like pointer chase, but random spatial access)
                    for i in 0..iterations {
                        let idx = buffer[i % n] as usize % n;
                        result.push(buffer[idx]);
                    }
                    result
                } else {
                    Vec::new()
                }
            }
        };

        let elapsed = start.elapsed();
        // Derive cycles from elapsed time using estimated CPU frequency (~4 GHz)
        let estimated_cpu_freq_hz = 4_000_000_000.0;
        let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;
        
        // Memory engine reduces data movement: only results flow back to CPU
        let data_moved = data.len() as u64 * 4;
        // Memory access is the same since we still need to scan/process
        let memory_access = buffers.iter().map(|b| b.len() as u64).sum::<u64>() * 4;

        ExecutionResult {
            data,
            stats: OperationStats {
                cycles,
                memory_access,
                data_moved,
            },
            execution_mode: ExecutionMode::MemoryEngine,
        }
    }
}

impl Default for MemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}
