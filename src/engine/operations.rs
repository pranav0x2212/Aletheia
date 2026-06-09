#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    MemCopy,
    MemVecAdd,
    MemVecAnd,
    MemVecOr,
    MemScan,
    MemStrideScan,
    MemPointerChase,
    MemRandomAccess,
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match self {
            Operation::MemCopy => "MEM_COPY",
            Operation::MemVecAdd => "MEM_VEC_ADD",
            Operation::MemVecAnd => "MEM_VEC_AND",
            Operation::MemVecOr => "MEM_VEC_OR",
            Operation::MemScan => "MEM_SCAN",
            Operation::MemStrideScan => "MEM_STRIDE_SCAN",
            Operation::MemPointerChase => "MEM_POINTER_CHASE",
            Operation::MemRandomAccess => "MEM_RANDOM_ACCESS",
        }
    }
}

pub trait MemoryOp {
    fn execute_cpu(&self, buffers: &[&[u32]]) -> Vec<u32>;
    fn execute_memory_engine(&self, buffers: &[&[u32]]) -> Vec<u32>;
    fn operation_type(&self) -> Operation;
}
