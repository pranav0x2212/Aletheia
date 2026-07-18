use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub op: MemOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MemOp {
    #[serde(rename = "MEM_COPY")]
    MemCopy { buffer: String },
    #[serde(rename = "MEM_VEC_ADD")]
    MemVecAdd { buffer_a: String, buffer_b: String },
    #[serde(rename = "MEM_VEC_AND")]
    MemVecAnd { buffer_a: String, buffer_b: String },
    #[serde(rename = "MEM_VEC_OR")]
    MemVecOr { buffer_a: String, buffer_b: String },
    #[serde(rename = "MEM_SCAN")]
    MemScan { buffer: String, threshold: u32 },
    #[serde(rename = "MEM_STRIDE_SCAN")]
    MemStrideScan { buffer: String, stride: usize },
    #[serde(rename = "MEM_POINTER_CHASE")]
    MemPointerChase { buffer: String, iterations: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub status: ResponseStatus,
    pub data: ResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ResponseStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub cycles: u64,
    pub instructions: u64,
    pub cache_references: u64,
    pub cache_misses: u64,
    pub memory_access: u64,
    pub data_moved: u64,
    pub result_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ResponseData {
    pub fn ok(
        cycles: u64,
        instructions: u64,
        cache_references: u64,
        cache_misses: u64,
        memory_access: u64,
        data_moved: u64,
        result_count: usize,
    ) -> Self {
        ResponseData {
            cycles,
            instructions,
            cache_references,
            cache_misses,
            memory_access,
            data_moved,
            result_count,
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        ResponseData {
            cycles: 0,
            instructions: 0,
            cache_references: 0,
            cache_misses: 0,
            memory_access: 0,
            data_moved: 0,
            result_count: 0,
            error: Some(msg),
        }
    }
}
