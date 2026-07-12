pub mod engine;
pub mod workloads;
pub mod runtime;
pub mod protocol;
pub mod network;
pub mod results;
pub mod profiler;

pub use engine::{MemoryEngine, Operation};
pub use runtime::Executor;
pub use protocol::{Command, Response, MemOp, ResponseStatus, ResponseData};
pub use network::{send_command, listen_and_serve};
pub use results::ExperimentResult;

#[derive(Copy, Clone, Debug)]
pub struct OperationStats {
    pub cycles: u64,
    pub memory_access: u64,
    pub data_moved: u64,
}
