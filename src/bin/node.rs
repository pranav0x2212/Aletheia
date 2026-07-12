use aletheia::{
    engine::{MemoryEngine, Operation, ExecutionResult},
    protocol::{Command, Response, ResponseStatus, ResponseData, MemOp},
    network::listen_and_serve,
    profiler,
};
use clap::Parser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "aletheia-node")]
#[command(about = "Aletheia memory engine server")]
struct Args {
    #[arg(short, long, default_value = "9000")]
    port: u16,

    #[arg(long, default_value = "256")]
    dataset_size: usize,
}

type BufferStore = Arc<Mutex<HashMap<String, (usize, usize)>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let buffers: BufferStore = Arc::new(Mutex::new(HashMap::new()));
    let mut engine = MemoryEngine::new();

    // Initialize default buffers
    println!("Initializing memory engine...");
    let dataset_size = args.dataset_size * 1024 * 1024 / 4; // Convert MB to u32 elements
    
    let buf_dataset = engine.allocate_buffer(dataset_size, 0);
    
    // Fill with pseudo-random data
    if let Some(buf) = engine.get_buffer_mut(buf_dataset) {
        for (i, val) in buf.iter_mut().enumerate() {
            *val = ((i as u32).wrapping_mul(7919)) % 1000;
        }
    }

    let mut buffers_guard = buffers.lock().unwrap();
    buffers_guard.insert("dataset".to_string(), (buf_dataset, dataset_size));
    drop(buffers_guard);

    println!("Dataset initialized: {}MB", args.dataset_size);

    // Wrap engine in Arc<Mutex<>> for thread-safe access
    let engine = Arc::new(Mutex::new(engine));
    let buffers_clone = buffers.clone();
    let engine_clone = engine.clone();

    // Start listening
    let addr = format!("0.0.0.0:{}", args.port);
    listen_and_serve(&addr, move |cmd: Command| {
        handle_command(cmd, &engine_clone, &buffers_clone)
    })
    .await?;

    Ok(())
}
fn execute_with_profiling(
    engine: &Arc<Mutex<MemoryEngine>>,
    op: Operation,
    buffer_indices: &[usize],
    params: &[u32],
) -> Result<ExecutionResult, String> {
    let engine_lock = engine.lock().unwrap();

    let (result, counters) = profiler::measure(|| {
        engine_lock.execute_memory_engine(op, buffer_indices, params)
    })
    .map_err(|e| e.to_string())?;

    println!("{:#?}", counters);

    Ok(result)
}

fn handle_command(
    cmd: Command,
    engine: &Arc<Mutex<MemoryEngine>>,
    buffers: &BufferStore,
) -> Response {
    let start = Instant::now();

    let op_name = match &cmd.op {
        MemOp::MemCopy { .. } => "MemCopy",
        MemOp::MemVecAdd { .. } => "MemVecAdd",
        MemOp::MemVecAnd { .. } => "MemVecAnd",
        MemOp::MemVecOr { .. } => "MemVecOr",
        MemOp::MemScan { .. } => "MemScan",
        MemOp::MemStrideScan { .. } => "MemStrideScan",
        MemOp::MemPointerChase { .. } => "MemPointerChase",
    };

    println!("[INFO] Executing {}", op_name);
    
    let result = match cmd.op {
        MemOp::MemCopy { buffer } => {
            let buffers_lock = buffers.lock().unwrap();
            if let Some(&(idx, _size)) = buffers_lock.get(&buffer) {
                drop(buffers_lock);
                execute_with_profiling(engine, Operation::MemCopy, &[idx], &[])
            } else {
                Err(format!("Buffer not found: {}", buffer))
            }
        }
        MemOp::MemVecAdd { buffer_a, buffer_b } => {
            let buffers_lock = buffers.lock().unwrap();
            let idx_a = buffers_lock.get(&buffer_a).map(|&(idx, _)| idx);
            let idx_b = buffers_lock.get(&buffer_b).map(|&(idx, _)| idx);
            drop(buffers_lock);

            match (idx_a, idx_b) {
                (Some(a), Some(b)) => {
                    execute_with_profiling(engine, Operation::MemVecAdd, &[a, b], &[])
                }
                _ => Err("Buffers not found".to_string()),
            }
        }
        MemOp::MemVecAnd { buffer_a, buffer_b } => {
            let buffers_lock = buffers.lock().unwrap();
            let idx_a = buffers_lock.get(&buffer_a).map(|&(idx, _)| idx);
            let idx_b = buffers_lock.get(&buffer_b).map(|&(idx, _)| idx);
            drop(buffers_lock);

            match (idx_a, idx_b) {
                (Some(a), Some(b)) => {
                    execute_with_profiling(engine, Operation::MemVecAnd, &[a, b], &[])
                }
                _ => Err("Buffers not found".to_string()),
            }
        }
        MemOp::MemVecOr { buffer_a, buffer_b } => {
            let buffers_lock = buffers.lock().unwrap();
            let idx_a = buffers_lock.get(&buffer_a).map(|&(idx, _)| idx);
            let idx_b = buffers_lock.get(&buffer_b).map(|&(idx, _)| idx);
            drop(buffers_lock);

            match (idx_a, idx_b) {
                (Some(a), Some(b)) => {
                    execute_with_profiling(engine, Operation::MemVecOr, &[a, b], &[])
                }
                _ => Err("Buffers not found".to_string()),
            }
        }
        MemOp::MemScan { buffer, threshold } => {
            let buffers_lock = buffers.lock().unwrap();
            if let Some(&(idx, _)) = buffers_lock.get(&buffer) {
                drop(buffers_lock);
                execute_with_profiling(engine, Operation::MemScan, &[idx], &[threshold])
            } else {
                Err(format!("Buffer not found: {}", buffer))
            }
        }
        MemOp::MemStrideScan { buffer, stride } => {
            let buffers_lock = buffers.lock().unwrap();
            if let Some(&(idx, _)) = buffers_lock.get(&buffer) {
                drop(buffers_lock);
                execute_with_profiling(
                    engine,
                    Operation::MemStrideScan,
                    &[idx],
                    &[stride as u32],
                )
            } else {
                Err(format!("Buffer not found: {}", buffer))
            }
        }
        MemOp::MemPointerChase { buffer, iterations } => {
            let buffers_lock = buffers.lock().unwrap();
            if let Some(&(idx, _)) = buffers_lock.get(&buffer) {
                drop(buffers_lock);
                execute_with_profiling(
                    engine,
                    Operation::MemPointerChase,
                    &[idx],
                    &[iterations as u32],
                )
            } else {
                Err(format!("Buffer not found: {}", buffer))
            }
        }
    };

    match result {
        Ok(exec_result) => {
            let elapsed = start.elapsed();
            println!(
                "[INFO] Completed {} in {:.3} ms",
                op_name,
                elapsed.as_secs_f64() * 1000.0
            );
            // Derive cycles from actual elapsed time using estimated CPU frequency (~4 GHz)
            let estimated_cpu_freq_hz = 4_000_000_000.0;
            let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;
            
            Response {
                id: cmd.id,
                status: ResponseStatus::Ok,
                data: ResponseData::ok(
                    cycles,
                    exec_result.stats.memory_access,
                    exec_result.stats.data_moved,
                    exec_result.data.len(),
                ),
            }
        },
        Err(e) => {
            println!("[ERROR] {} failed: {}", op_name, e);
        
            Response {
                id: cmd.id,
                status: ResponseStatus::Error,
                data: ResponseData::error(e),
            }
        }
    }
}
