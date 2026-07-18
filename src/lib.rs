pub mod engine;
pub mod network;
pub mod profiler;
pub mod protocol;
pub mod results;
pub mod runtime;
pub mod workloads;

pub use engine::{MemoryEngine, Operation};
pub use network::{listen_and_serve, send_command};
pub use protocol::{Command, MemOp, Response, ResponseData, ResponseStatus};
pub use results::ExperimentResult;
pub use runtime::Executor;

#[derive(Copy, Clone, Debug)]
pub struct OperationStats {
    pub cycles: u64,
    pub memory_access: u64,
    pub data_moved: u64,
}
