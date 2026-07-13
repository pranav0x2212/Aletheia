use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub experiment: String,
    pub mode: String,            // "cpu" or "memory_engine"
    pub working_set_bytes: u64,
    pub runtime_ms: u128,
    pub cycles: u64,
    pub instructions: u64,
    pub cache_references: u64,
    pub cache_misses: u64,
    pub memory_access_bytes: u64,
    pub data_moved_bytes: u64,
    pub operations: u64,
    pub operational_intensity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stride: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ns_per_access: Option<f64>,
}

impl ExperimentResult {
    pub fn new(
        experiment: &str,
        mode: &str,
        working_set_bytes: u64,
        runtime_ms: u128,
        cycles: u64,
        instructions: u64,
        cache_references: u64,
        cache_misses: u64,
        memory_access_bytes: u64,
        data_moved_bytes: u64,
        operations: u64,
    ) -> Self {
        let operational_intensity = if data_moved_bytes > 0 {
            operations as f64 / data_moved_bytes as f64
        } else {
            0.0
        };

        ExperimentResult {
            experiment: experiment.to_string(),
            mode: mode.to_string(),
            working_set_bytes,
            runtime_ms,
            cycles,
            instructions,
            cache_references,
            cache_misses,
            memory_access_bytes,
            data_moved_bytes,
            operations,
            operational_intensity,
            stride: None,
            latency_ns_per_access: None,
        }
    }

    pub fn with_stride(
        experiment: &str,
        mode: &str,
        working_set_bytes: u64,
        runtime_ms: u128,
        cycles: u64,
        instructions: u64,
        cache_references: u64,
        cache_misses: u64,
        memory_access_bytes: u64,
        data_moved_bytes: u64,
        operations: u64,
        stride: usize,
    ) -> Self {
        let operational_intensity = if data_moved_bytes > 0 {
            operations as f64 / data_moved_bytes as f64
        } else {
            0.0
        };

        ExperimentResult {
            experiment: experiment.to_string(),
            mode: mode.to_string(),
            working_set_bytes,
            runtime_ms,
            cycles,
            instructions,
            cache_references,
            cache_misses,
            memory_access_bytes,
            data_moved_bytes,
            operations,
            operational_intensity,
            stride: Some(stride),
            latency_ns_per_access: None,
        }
    }

    /// Helper method to set latency_ns_per_access (for working-set sweep results)
    pub fn with_latency(mut self, latency_ns: f64) -> Self {
        self.latency_ns_per_access = Some(latency_ns);
        self
    }

    /// Export result as a single line of JSONL
    pub fn to_json_line(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Append this result to a JSONL file
    pub fn append_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json_line = self.to_json_line()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", json_line)?;
        Ok(())
    }

    /// Append multiple results to a JSONL file
    pub fn append_batch_to_file<P: AsRef<Path>>(results: &[ExperimentResult], path: P) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        
        for result in results {
            let json_line = result.to_json_line()?;
            writeln!(file, "{}", json_line)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experiment_result_serialization() {
        let operations = 256 * 1024 * 1024 / 4;
        let working_set_bytes = 256 * 1024 * 1024;
        let result = ExperimentResult::new("scan", "cpu", working_set_bytes, 92, 121241, 0, 0, 0, 268000000, 133000000, operations);
        let json = result.to_json_line().unwrap();
        assert!(json.contains("\"experiment\":\"scan\""));
        assert!(json.contains("\"mode\":\"cpu\""));
        assert!(json.contains("\"operations\":"));
        assert!(json.contains("\"operational_intensity\":"));
    }
}