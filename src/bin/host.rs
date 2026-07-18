use aletheia::{
    engine::{MemoryEngine, Operation},
    network::send_command,
    protocol::{Command, MemOp},
    results::ExperimentResult,
    workloads::WorkingSetSweep,
};
use clap::{Parser, Subcommand};
use std::process::{Child, Command as ProcessCommand};
use std::time::Instant;
use uuid::Uuid;

use aletheia::runtime::execution_policy::ExecutionPolicy;

#[derive(Parser)]
#[command(name = "aletheia-host")]
#[command(about = "Aletheia distributed memory compute client")]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:9000")]
    node: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a memory scan on the remote node
    Scan {
        /// Threshold value for filtering
        #[arg(long, default_value = "500")]
        threshold: u32,

        /// Buffer name
        #[arg(long, default_value = "dataset")]
        buffer: String,

        /// Export results to JSONL file
        #[arg(short, long)]
        export: Option<String>,
    },
    /// Run vector addition on two buffers
    VecAdd {
        /// First buffer name
        #[arg(long, default_value = "dataset")]
        buffer_a: String,

        /// Second buffer name
        #[arg(long, default_value = "dataset")]
        buffer_b: String,

        /// Export results to JSONL file
        #[arg(short, long)]
        export: Option<String>,
    },
    /// Run comparative benchmark
    Benchmark {
        /// Export results to JSONL file
        #[arg(short, long)]
        export: Option<String>,
    },
    /// Run stride scan to measure memory access patterns
    StrideScan {
        /// Stride value for memory access pattern
        #[arg(long, default_value = "1")]
        stride: usize,

        /// Export results to JSONL file
        #[arg(short, long)]
        export: Option<String>,
    },
    /// Run pointer chasing to measure memory latency
    PointerChase {
        /// Number of iterations for pointer chasing
        #[arg(long, default_value = "1000")]
        iterations: usize,

        /// Buffer name
        #[arg(long, default_value = "dataset")]
        buffer: String,

        /// Export results to JSONL file
        #[arg(short, long)]
        export: Option<String>,
    },
    /// Run dataset scaling experiments
    Experiment {
        #[command(subcommand)]
        exp_type: ExperimentType,
    },
}

#[derive(Subcommand)]
enum ExperimentType {
    /// Scale workloads across dataset sizes
    DatasetScaling {
        /// Node binary path
        #[arg(long, default_value = "./target/release/aletheia-node")]
        node_bin: String,

        /// Use an existing remote node instead of spawning a local one
        #[arg(long)]
        node: Option<String>,
    },
    /// Test memory access stride effects
    StrideTesting {
        /// Node binary path
        #[arg(long, default_value = "./target/release/aletheia-node")]
        node_bin: String,

        /// Use an existing remote node instead of spawning a local one
        #[arg(long)]
        node: Option<String>,
    },
    /// Sweep working set sizes to measure cache hierarchy effects
    WorkingSetSweep {
        /// Measurement mode: "sequential" (old scan-based) or "pointer" (new pointer-chase)
        #[arg(long)]
        mode: String,

        /// Run only a specific working set size (e.g. 64KB, 16MB)
        #[arg(long)]
        size: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("═══════════════════════════════════════════════════════");
    println!("  Aletheia Distributed Memory Engine - Host Client");
    println!("═══════════════════════════════════════════════════════\n");

    match args.command {
        Commands::Scan {
            threshold,
            buffer,
            export,
        } => {
            run_scan(&args.node, &buffer, threshold, export.as_deref()).await?;
        }
        Commands::VecAdd {
            buffer_a,
            buffer_b,
            export,
        } => {
            run_vec_add(&args.node, &buffer_a, &buffer_b, export.as_deref()).await?;
        }
        Commands::Benchmark { export } => {
            run_benchmark(&args.node, export.as_deref()).await?;
        }
        Commands::StrideScan { stride, export } => {
            run_stride_scan(&args.node, stride, export.as_deref()).await?;
        }
        Commands::PointerChase {
            iterations,
            buffer,
            export,
        } => {
            run_pointer_chase(&args.node, &buffer, iterations, export.as_deref()).await?;
        }
        Commands::Experiment { exp_type } => match exp_type {
            ExperimentType::DatasetScaling { node_bin, node } => {
                run_dataset_scaling_experiment(&node_bin, node.as_deref()).await?;
            }
            ExperimentType::StrideTesting { node_bin, node } => {
                run_stride_testing_experiment(&node_bin, node.as_deref()).await?;
            }
            ExperimentType::WorkingSetSweep { mode, size } => {
                run_working_set_sweep_experiment(&mode, size.as_deref()).await?;
            }
        },
    }

    Ok(())
}

async fn run_scan(
    node: &str,
    buffer: &str,
    threshold: u32,
    export_path: Option<&str>,
) -> anyhow::Result<()> {
    println!("Dataset Scan Workload");
    println!("====================\n");

    let buf_size = 256 * 1024 * 1024 / 4; // 256MB

    // CPU mode execution (local)
    print!("CPU Mode (local execution): ");
    let cpu = ExecutionPolicy::Cpu
        .run_scan(buf_size, buffer, threshold)
        .await?;
    println!("{:.3}s", cpu.elapsed.as_secs_f64());
    println!("  Results: {} matches", cpu.result_count);
    println!("  Cycles: {}", cpu.cycles);
    println!("  Memory Access: {}MB", cpu.memory_access / 1_000_000);

    // Memory engine mode (remote)
    println!("\nMemory Engine Mode (remote on RPi): ");
    print!("  Offloading to node... ");

    let remote = ExecutionPolicy::RemoteNode {
        address: node.to_string(),
    };
    let mem = remote.run_scan(buf_size, buffer, threshold).await?;

    println!("{:.3}s", mem.elapsed.as_secs_f64());
    println!("  Results: {} matches", mem.result_count);
    println!("  Cycles: {}", mem.cycles);
    println!("  Memory Access: {}MB", mem.memory_access / 1_000_000);

    println!("\n--- Comparison ---");
    let speedup = cpu.elapsed.as_secs_f64() / mem.elapsed.as_secs_f64();
    println!("Network Latency Speedup: {:.2}x", speedup);
    println!("Data Movement (local): {}MB", cpu.data_moved / 1_000_000);
    println!("Data Movement (remote): {}MB", mem.data_moved / 1_000_000);

    // Export results if requested
    if let Some(path) = export_path {
        let operations = 256 * 1024 * 1024 / 4;
        let working_set_bytes = 256 * 1024 * 1024;
        let cpu_exp = ExperimentResult::new(
            "scan",
            "cpu",
            working_set_bytes,
            cpu.elapsed.as_millis(),
            cpu.cycles,
            0,
            0,
            0,
            cpu.memory_access,
            cpu.data_moved,
            operations,
        );

        let mem_exp = ExperimentResult::new(
            "scan",
            "memory_engine",
            working_set_bytes,
            mem.elapsed.as_millis(),
            mem.cycles,
            mem.instructions,
            mem.cache_references,
            mem.cache_misses,
            mem.memory_access,
            mem.data_moved,
            operations,
        );

        ExperimentResult::append_batch_to_file(&[cpu_exp, mem_exp], path)?;
        println!("\n✓ Results exported to {}", path);
    }

    Ok(())
}

async fn run_vec_add(
    node: &str,
    buffer_a: &str,
    buffer_b: &str,
    export_path: Option<&str>,
) -> anyhow::Result<()> {
    println!("Vector Addition Workload");
    println!("========================\n");

    let buf_size = 256 * 1024 * 1024 / 4;

    // CPU mode
    print!("CPU Mode (local execution): ");
    let cpu = ExecutionPolicy::Cpu
        .run_vec_add(buf_size, buffer_a, buffer_b)
        .await?;
    println!("{:.3}s", cpu.elapsed.as_secs_f64());

    // Memory engine mode
    println!("Memory Engine Mode (remote): ");
    print!("  Offloading to node... ");

    let remote = ExecutionPolicy::RemoteNode {
        address: node.to_string(),
    };
    let mem = remote.run_vec_add(buf_size, buffer_a, buffer_b).await?;
    println!("{:.3}s", mem.elapsed.as_secs_f64());

    println!("\n--- Comparison ---");
    let speedup = cpu.elapsed.as_secs_f64() / mem.elapsed.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);

    // Export results if requested
    if let Some(path) = export_path {
        let operations = 256 * 1024 * 1024 / 4;
        let working_set_bytes = 256 * 1024 * 1024;
        let cpu_exp = ExperimentResult::new(
            "vector_add",
            "cpu",
            working_set_bytes,
            cpu.elapsed.as_millis(),
            cpu.cycles,
            0,
            0,
            0,
            cpu.memory_access,
            cpu.data_moved,
            operations,
        );

        let mem_exp = ExperimentResult::new(
            "vector_add",
            "memory_engine",
            working_set_bytes,
            mem.elapsed.as_millis(),
            mem.cycles,
            mem.instructions,
            mem.cache_references,
            mem.cache_misses,
            mem.memory_access,
            mem.data_moved,
            operations,
        );

        ExperimentResult::append_batch_to_file(&[cpu_exp, mem_exp], path)?;
        println!("\n✓ Results exported to {}", path);
    }

    Ok(())
}

async fn run_benchmark(node: &str, export_path: Option<&str>) -> anyhow::Result<()> {
    println!("Full Benchmark Suite");
    println!("====================\n");

    let mut results = Vec::new();

    let operations = vec![(
        "SCAN",
        MemOp::MemScan {
            buffer: "dataset".to_string(),
            threshold: 500,
        },
    )];

    for (name, op) in operations {
        println!("Running {}...", name);
        let cmd = Command {
            id: Uuid::new_v4().to_string(),
            op,
        };

        let start = Instant::now();
        let response = send_command(node, cmd).await?;
        let elapsed = start.elapsed();

        // Derive cycles from measured elapsed time (4 GHz CPU frequency)
        let estimated_cpu_freq_hz = 4_000_000_000.0;
        let cycles = (elapsed.as_secs_f64() * estimated_cpu_freq_hz) as u64;

        println!("  Time: {:.3}s", elapsed.as_secs_f64());
        println!("  Cycles: {}", cycles);
        println!("  Status: {:?}\n", response.status);

        let operations = 256 * 1024 * 1024 / 4;
        let result = ExperimentResult::new(
            &name.to_lowercase(),
            "memory_engine",
            256,
            elapsed.as_millis(),
            cycles,
            response.data.instructions,
            response.data.cache_references,
            response.data.cache_misses,
            response.data.memory_access,
            response.data.data_moved,
            operations,
        );
        results.push(result);
    }

    // Export all benchmark results if requested
    if let Some(path) = export_path {
        ExperimentResult::append_batch_to_file(&results, path)?;
        println!("✓ Benchmark results exported to {}", path);
    }

    Ok(())
}

async fn run_dataset_scaling_experiment(node_bin: &str, node: Option<&str>) -> anyhow::Result<()> {
    println!("Dataset Scaling Experiment");
    println!("==========================\n");

    let dataset_sizes = vec![64, 128, 256, 512, 1024];
    let export_file = "results/rpi-results/dataset_scaling.jsonl";

    println!("Testing dataset sizes: {:?}MB\n", dataset_sizes);

    for dataset_mb in dataset_sizes {
        println!("╔══════════════════════════════════════╗");
        println!(
            "║  Dataset Size: {}MB",
            format!("{:>5}", dataset_mb).trim_end()
        );
        println!("╚══════════════════════════════════════╝");

        // Start node with this dataset size (unless an existing remote node was supplied)
        let mut node_process = if node.is_none() {
            Some(start_node(node_bin, 9000, dataset_mb)?)
        } else {
            None
        };
        let node_addr = node.unwrap_or("127.0.0.1:9000");
        if node_process.is_some() {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        // Run scan workload
        println!("\n  Running scan...");
        let scan_results = run_scan_experiment(node_addr, dataset_mb).await?;

        // Run vec-add workload
        println!("\n  Running vector-add...");
        let vecadd_results = run_vecadd_experiment(node_addr, dataset_mb).await?;

        // Collect all results
        let mut all_results = vec![];
        all_results.extend(scan_results);
        all_results.extend(vecadd_results);

        // Export results
        ExperimentResult::append_batch_to_file(&all_results, export_file)?;
        println!("\n✓ Results for {}MB exported", dataset_mb);

        // Stop node (only if we started one)
        if let Some(ref mut process) = node_process {
            stop_node(process)?;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    println!("\n✓ All dataset scaling experiments completed!");
    println!("✓ Results saved to {}", export_file);
    Ok(())
}

fn start_node(node_bin: &str, port: u16, dataset_size: usize) -> anyhow::Result<Child> {
    let child = ProcessCommand::new(node_bin)
        .arg("--port")
        .arg(port.to_string())
        .arg("--dataset-size")
        .arg(dataset_size.to_string())
        .spawn()?;

    Ok(child)
}

fn stop_node(process: &mut Child) -> anyhow::Result<()> {
    process.kill()?;
    process.wait()?;
    Ok(())
}

async fn run_scan_experiment(
    node: &str,
    dataset_mb: usize,
) -> anyhow::Result<Vec<ExperimentResult>> {
    let mut results = Vec::new();

    let buf_size = dataset_mb * 1024 * 1024 / 4;
    let operations = dataset_mb as u64 * 1024 * 1024 / 4;
    let working_set_bytes = (dataset_mb as u64) * 1024 * 1024;

    // CPU mode
    let cpu = ExecutionPolicy::Cpu
        .run_scan(buf_size, "dataset", 500)
        .await?;
    results.push(ExperimentResult::new(
        "scan",
        "cpu",
        working_set_bytes,
        cpu.elapsed.as_millis(),
        cpu.cycles,
        cpu.instructions,
        cpu.cache_references,
        cpu.cache_misses,
        cpu.memory_access,
        cpu.data_moved,
        operations,
    ));

    // Memory engine mode
    let remote = ExecutionPolicy::RemoteNode {
        address: node.to_string(),
    };
    match remote.run_scan(buf_size, "dataset", 500).await {
        Ok(mem) => {
            results.push(ExperimentResult::new(
                "scan",
                "memory_engine",
                working_set_bytes,
                mem.elapsed.as_millis(),
                mem.cycles,
                mem.instructions,
                mem.cache_references,
                mem.cache_misses,
                mem.memory_access,
                mem.data_moved,
                operations,
            ));
        }
        Err(_) => {
            println!("    ⚠ Memory engine mode failed (node might be restarting)");
        }
    }

    Ok(results)
}

async fn run_vecadd_experiment(
    node: &str,
    dataset_mb: usize,
) -> anyhow::Result<Vec<ExperimentResult>> {
    let mut results = Vec::new();
    let buf_size = dataset_mb * 1024 * 1024 / 4;
    let operations = dataset_mb as u64 * 1024 * 1024 / 4;
    let working_set_bytes = (dataset_mb as u64) * 1024 * 1024;

    // CPU mode
    let cpu = ExecutionPolicy::Cpu
        .run_vec_add(buf_size, "dataset", "dataset")
        .await?;
    results.push(ExperimentResult::new(
        "vector_add",
        "cpu",
        working_set_bytes,
        cpu.elapsed.as_millis(),
        cpu.cycles,
        0,
        0,
        0,
        cpu.memory_access,
        cpu.data_moved,
        operations,
    ));

    // Memory engine mode
    let remote = ExecutionPolicy::RemoteNode {
        address: node.to_string(),
    };
    match remote.run_vec_add(buf_size, "dataset", "dataset").await {
        Ok(mem) => {
            results.push(ExperimentResult::new(
                "vector_add",
                "memory_engine",
                working_set_bytes,
                mem.elapsed.as_millis(),
                mem.cycles,
                mem.instructions,
                mem.cache_references,
                mem.cache_misses,
                mem.memory_access,
                mem.data_moved,
                operations,
            ));
        }
        Err(_) => {
            println!("    ⚠ Memory engine mode failed (node might be restarting)");
        }
    }

    Ok(results)
}

async fn run_stride_scan(
    node: &str,
    stride: usize,
    export_path: Option<&str>,
) -> anyhow::Result<()> {
    println!("Stride Scan Workload");
    println!("===================\n");

    let dataset_size_mb = 256;
    let mut results = Vec::new();
    let buf_size = dataset_size_mb * 1024 * 1024 / 4;

    // CPU mode
    print!("CPU Mode (stride={}): ", stride);
    let cpu = ExecutionPolicy::Cpu
        .run_stride_scan(buf_size, stride)
        .await?;
    println!("{:.3}s", cpu.elapsed.as_secs_f64());
    println!("  Elements accessed: {}", cpu.result_count);
    println!("  Cycles: {}", cpu.cycles);

    let operations = dataset_size_mb as u64 * 1024 * 1024 / 4;
    let working_set_bytes = (dataset_size_mb as u64) * 1024 * 1024;
    results.push(ExperimentResult::with_stride(
        "stride_scan",
        "cpu",
        working_set_bytes,
        cpu.elapsed.as_millis(),
        cpu.cycles,
        0,
        0,
        0,
        cpu.memory_access,
        cpu.data_moved,
        operations,
        stride,
    ));

    // Memory engine mode
    println!("\nMemory Engine Mode (stride={}): ", stride);
    print!("  Offloading to node... ");

    let remote = ExecutionPolicy::RemoteNode {
        address: node.to_string(),
    };
    let mem = remote.run_stride_scan(buf_size, stride).await?;

    println!("{:.3}s", mem.elapsed.as_secs_f64());
    println!("  Elements accessed: {}", mem.result_count);
    println!("  Cycles: {}", mem.cycles);

    let operations = dataset_size_mb as u64 * 1024 * 1024 / 4;
    let working_set_bytes = (dataset_size_mb as u64) * 1024 * 1024;
    results.push(ExperimentResult::with_stride(
        "stride_scan",
        "memory_engine",
        working_set_bytes,
        mem.elapsed.as_millis(),
        mem.cycles,
        mem.instructions,
        mem.cache_references,
        mem.cache_misses,
        mem.memory_access,
        mem.data_moved,
        operations,
        stride,
    ));

    println!("\n--- Stride Impact ---");
    let speedup = cpu.elapsed.as_secs_f64() / mem.elapsed.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    println!("Stride effect: access pattern every {} elements", stride);

    // Export results if requested
    if let Some(path) = export_path {
        ExperimentResult::append_batch_to_file(&results, path)?;
        println!("\n✓ Results exported to {}", path);
    }

    Ok(())
}

async fn run_pointer_chase(
    node: &str,
    buffer: &str,
    iterations: usize,
    export_path: Option<&str>,
) -> anyhow::Result<()> {
    println!("Pointer Chase Workload");
    println!("======================\n");
    println!("Measuring memory latency with pointer chasing (no prefetching)\n");

    let buf_size = 256 * 1024 * 1024 / 4;

    // CPU mode
    print!("CPU Mode (iterations={}): ", iterations);
    let cpu = ExecutionPolicy::Cpu
        .run_pointer_chase(buf_size, buffer, iterations)
        .await?;
    println!("{:.3}s", cpu.elapsed.as_secs_f64());
    println!("  Accesses: {}", cpu.result_count);
    println!("  Cycles: {}", cpu.cycles);

    // Memory engine mode
    println!("\nMemory Engine Mode (iterations={}): ", iterations);
    print!("  Offloading to node... ");

    let remote = ExecutionPolicy::RemoteNode {
        address: node.to_string(),
    };
    let mem = remote
        .run_pointer_chase(buf_size, buffer, iterations)
        .await?;

    println!("{:.3}s", mem.elapsed.as_secs_f64());
    println!("  Accesses: {}", mem.result_count);
    println!("  Cycles: {}", mem.cycles);

    println!("\n--- Memory Latency Analysis ---");
    let speedup = cpu.elapsed.as_secs_f64() / mem.elapsed.as_secs_f64();
    let latency_reduction = ((cpu.cycles as f64 - mem.cycles as f64) / cpu.cycles as f64) * 100.0;
    println!("Speedup: {:.2}x", speedup);
    println!("Latency Reduction: {:.2}%", latency_reduction);
    println!("Note: Pointer chasing reveals true memory dependency latency");

    // Export results if requested
    if let Some(path) = export_path {
        let operations = iterations as u64;
        let working_set_bytes = 256 * 1024 * 1024;
        let cpu_exp = ExperimentResult::new(
            "pointer_chase",
            "cpu",
            working_set_bytes,
            cpu.elapsed.as_millis(),
            cpu.cycles,
            0,
            0,
            0,
            cpu.memory_access,
            cpu.data_moved,
            operations,
        );

        let mem_exp = ExperimentResult::new(
            "pointer_chase",
            "memory_engine",
            working_set_bytes,
            mem.elapsed.as_millis(),
            mem.cycles,
            mem.instructions,
            mem.cache_references,
            mem.cache_misses,
            mem.memory_access,
            mem.data_moved,
            operations,
        );

        ExperimentResult::append_batch_to_file(&[cpu_exp, mem_exp], path)?;
        println!("\n✓ Results exported to {}", path);
    }

    Ok(())
}

async fn run_stride_testing_experiment(node_bin: &str, node: Option<&str>) -> anyhow::Result<()> {
    println!("Stride Testing Experiment");
    println!("=========================\n");

    let stride_values = vec![1, 4, 16, 64, 256, 4096];
    let export_file = "results/rpi-results/stride_scan.jsonl";

    println!("Testing strides: {:?}\n", stride_values);

    // Start node once (unless an existing remote node was supplied)
    let mut node_process = if node.is_none() {
        Some(start_node(node_bin, 9000, 256)?)
    } else {
        None
    };
    let node_addr = node.unwrap_or("127.0.0.1:9000");
    if node_process.is_some() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    for stride in stride_values {
        println!("Running stride scan with stride={}", stride);

        let buf_size = 256 * 1024 * 1024 / 4;

        // CPU mode
        let cpu = ExecutionPolicy::Cpu
            .run_stride_scan(buf_size, stride)
            .await?;

        let operations = 256 * 1024 * 1024 / 4;
        let cpu_exp = ExperimentResult::with_stride(
            "stride_scan",
            "cpu",
            256,
            cpu.elapsed.as_millis(),
            cpu.cycles,
            0,
            0,
            0,
            cpu.memory_access,
            cpu.data_moved,
            operations,
            stride,
        );

        // Memory engine mode
        let remote = ExecutionPolicy::RemoteNode {
            address: node_addr.to_string(),
        };
        match remote.run_stride_scan(buf_size, stride).await {
            Ok(mem) => {
                let operations = 256 * 1024 * 1024 / 4;
                let working_set_bytes = 256 * 1024 * 1024;
                let mem_exp = ExperimentResult::with_stride(
                    "stride_scan",
                    "memory_engine",
                    working_set_bytes,
                    mem.elapsed.as_millis(),
                    mem.cycles,
                    mem.instructions,
                    mem.cache_references,
                    mem.cache_misses,
                    mem.memory_access,
                    mem.data_moved,
                    operations,
                    stride,
                );

                ExperimentResult::append_batch_to_file(&[cpu_exp, mem_exp], export_file)?;
                println!("  ✓ Results for stride={} exported\n", stride);
            }
            Err(_) => {
                println!("  ⚠ Memory engine mode failed\n");
            }
        }
    }

    // Stop node (only if we started one)
    if let Some(ref mut process) = node_process {
        stop_node(process)?;
    }

    println!("✓ All stride testing experiments completed!");
    println!("✓ Results saved to {}", export_file);
    Ok(())
}

fn parse_working_set_size(size: &str) -> anyhow::Result<usize> {
    let size = size.trim().to_uppercase();

    if let Some(kb) = size.strip_suffix("KB") {
        return Ok(kb.parse::<usize>()? * 1024);
    }

    if let Some(mb) = size.strip_suffix("MB") {
        return Ok(mb.parse::<usize>()? * 1024 * 1024);
    }

    anyhow::bail!(
        "Invalid working set size '{}'. Use values like 64KB or 16MB.",
        size
    );
}

async fn run_working_set_sweep_experiment(mode: &str, size: Option<&str>) -> anyhow::Result<()> {
    let is_pointer = mode == "pointer";
    let is_sequential = mode == "sequential";
    let is_random = mode == "random";

    if !is_pointer && !is_sequential && !is_random {
        anyhow::bail!(
            "Invalid mode: '{}'. Use 'pointer', 'sequential', or 'random'",
            mode
        );
    }

    let title = if is_pointer {
        "Working Set Sweep Experiment (Pointer Chasing)"
    } else if is_random {
        "Working Set Sweep Experiment (Independent Random Access)"
    } else {
        "Working Set Sweep Experiment (Sequential Scan)"
    };

    println!("{}", title);
    println!("{}\n", "=".repeat(title.len()));

    if is_pointer {
        println!("Measuring memory latency by defeating prefetching\n");
        println!("Method: Randomized dependent pointer chasing");
        println!("        (each access creates memory dependency)\n");
    } else if is_random {
        println!("Measuring memory latency with independent random access\n");
        println!("Method: Independent random memory access");
        println!("        (defeats prefetching, allows parallelism - middle ground)\n");
    } else {
        println!("Measuring memory latency with hardware prefetching enabled\n");
        println!("Method: Sequential memory scan");
        println!("        (allows prefetching and spatial locality optimization)\n");
    }

    let workload = WorkingSetSweep::new();
    let export_file = if is_pointer {
        "results/rpi-results/working_set_sweep_pointer.jsonl"
    } else if is_random {
        "results/rpi-results/working_set_sweep_random.jsonl"
    } else {
        "results/rpi-results/working_set_sweep_sequential.jsonl"
    };

    // Cache hierarchy estimates (typical modern x86):
    // L1: 32KB, L2: 256KB, L3: 8MB
    println!("Cache hierarchy reference:");
    println!("  L1 Cache:  32KB");
    println!("  L2 Cache:  256KB");
    println!("  L3 Cache:  8MB");
    println!("  DRAM:      system dependent\n");

    println!("Experiment parameters:");
    println!("  Measured iterations:  {}", workload.iterations);
    println!("  Warmup iterations:    {}", workload.warmup_iterations);
    println!(
        "  Total iterations:     {}\n",
        workload.iterations + workload.warmup_iterations
    );

    println!("Testing working set sizes:");
    println!("─────────────────────────────────────────────────────────\n");

    let mut all_results = Vec::new();

    let working_set_sizes = match size {
        Some(size) => vec![parse_working_set_size(size)?],
        None => workload.working_set_sizes.clone(),
    };

    for working_set_bytes in &working_set_sizes {
        let size_kb = working_set_bytes / 1024;
        let size_mb = working_set_bytes / (1024 * 1024);

        let size_str = if size_mb > 0 {
            format!("{}MB", size_mb)
        } else {
            format!("{}KB", size_kb)
        };

        // Identify cache level
        let cache_level = if working_set_bytes <= &(32 * 1024) {
            "→ L1"
        } else if working_set_bytes <= &(256 * 1024) {
            "→ L2"
        } else if working_set_bytes <= &(8 * 1024 * 1024) {
            "→ L3"
        } else {
            "→ DRAM"
        };

        print!("Working Set: {:>6}  {} ", size_str, cache_level);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let iterations = WorkingSetSweep::get_scaled_iterations(*working_set_bytes);

        let warmup_iterations = iterations / 10;

        if is_pointer {
            // ===== POINTER CHASING MODE =====

            // CPU mode with pointer chasing
            let mut engine = MemoryEngine::new();
            let buf_size = working_set_bytes / 4; // u32 is 4 bytes
            let buf = engine.allocate_buffer(buf_size, 0);

            // Initialize buffer as randomized pointer chain
            let pointer_chain = WorkingSetSweep::generate_pointer_chain(buf_size);
            if let Some(buf_data) = engine.get_buffer_mut(buf) {
                for (i, &ptr) in pointer_chain.iter().enumerate() {
                    buf_data[i] = ptr;
                }
            }

            // Warmup (don't count toward latency)
            let _warmup = engine.execute_cpu(
                Operation::MemPointerChase,
                &[buf],
                &[warmup_iterations as u32],
            );

            // Measured run
            let start = Instant::now();
            let cpu_result =
                engine.execute_cpu(Operation::MemPointerChase, &[buf], &[iterations as u32]);
            let cpu_time = start.elapsed();
            let cpu_latency_ns =
                WorkingSetSweep::calculate_latency_ns(cpu_time.as_nanos(), iterations);

            // Memory engine mode with pointer chasing
            let mut engine = MemoryEngine::new();
            let buf = engine.allocate_buffer(buf_size, 0);

            // Initialize buffer with same pointer chain
            if let Some(buf_data) = engine.get_buffer_mut(buf) {
                for (i, &ptr) in pointer_chain.iter().enumerate() {
                    buf_data[i] = ptr;
                }
            }

            // Warmup (don't count toward latency)
            let _warmup = engine.execute_memory_engine(
                Operation::MemPointerChase,
                &[buf],
                &[warmup_iterations as u32],
            );

            // Measured run
            let start = Instant::now();
            let mem_result = engine.execute_memory_engine(
                Operation::MemPointerChase,
                &[buf],
                &[iterations as u32],
            );
            let mem_time = start.elapsed();
            let mem_latency_ns =
                WorkingSetSweep::calculate_latency_ns(mem_time.as_nanos(), iterations);

            print!(
                "CPU: {:.2} ns/access | ME: {:.2} ns/access\n",
                cpu_latency_ns, mem_latency_ns
            );

            // Store results with exact working set size in bytes (no rounding)
            let cpu_exp = ExperimentResult::new(
                "working_set_sweep",
                "cpu",
                *working_set_bytes as u64,
                cpu_time.as_millis(),
                cpu_result.stats.cycles,
                0,
                0,
                0,
                cpu_result.stats.memory_access,
                cpu_result.stats.data_moved,
                iterations as u64,
            )
            .with_latency(cpu_latency_ns);

            let mem_exp = ExperimentResult::new(
                "working_set_sweep",
                "memory_engine",
                *working_set_bytes as u64,
                mem_time.as_millis(),
                mem_result.stats.cycles,
                0,
                0,
                0,
                mem_result.stats.memory_access,
                mem_result.stats.data_moved,
                iterations as u64,
            )
            .with_latency(mem_latency_ns);

            all_results.push(cpu_exp);
            all_results.push(mem_exp);
        } else if is_sequential {
            // ===== SEQUENTIAL SCAN MODE =======

            // CPU mode with sequential scan
            let mut engine = MemoryEngine::new();
            let buf_size = working_set_bytes / 4; // u32 is 4 bytes
            let buf = engine.allocate_buffer(buf_size, 0);

            // Initialize buffer with simple values (sequential access pattern will be used)
            if let Some(buf_data) = engine.get_buffer_mut(buf) {
                for (i, val) in buf_data.iter_mut().enumerate() {
                    *val = ((i as u32).wrapping_mul(7919)) % 1000;
                }
            }

            // Warmup (don't count toward latency)
            let _warmup = engine.execute_cpu(
                Operation::MemScan,
                &[buf],
                &[500], // threshold for scan
            );

            // Measured run
            let start = Instant::now();
            let cpu_result = engine.execute_cpu(Operation::MemScan, &[buf], &[500]);
            let cpu_time = start.elapsed();
            let cpu_latency_ns =
                WorkingSetSweep::calculate_latency_ns(cpu_time.as_nanos(), iterations);

            // Memory engine mode with sequential scan
            let mut engine = MemoryEngine::new();
            let buf = engine.allocate_buffer(buf_size, 0);

            // Initialize buffer with simple values
            if let Some(buf_data) = engine.get_buffer_mut(buf) {
                for (i, val) in buf_data.iter_mut().enumerate() {
                    *val = ((i as u32).wrapping_mul(7919)) % 1000;
                }
            }

            // Warmup (don't count toward latency)
            let _warmup = engine.execute_memory_engine(Operation::MemScan, &[buf], &[500]);

            // Measured run
            let start = Instant::now();
            let mem_result = engine.execute_memory_engine(Operation::MemScan, &[buf], &[500]);
            let mem_time = start.elapsed();
            let mem_latency_ns =
                WorkingSetSweep::calculate_latency_ns(mem_time.as_nanos(), iterations);

            print!(
                "CPU: {:.2} ns/access | ME: {:.2} ns/access\n",
                cpu_latency_ns, mem_latency_ns
            );

            // Store results with exact working set size in bytes (no rounding)
            let cpu_exp = ExperimentResult::new(
                "working_set_sweep",
                "cpu",
                *working_set_bytes as u64,
                cpu_time.as_millis(),
                cpu_result.stats.cycles,
                0,
                0,
                0,
                cpu_result.stats.memory_access,
                cpu_result.stats.data_moved,
                iterations as u64,
            )
            .with_latency(cpu_latency_ns);

            let mem_exp = ExperimentResult::new(
                "working_set_sweep",
                "memory_engine",
                *working_set_bytes as u64,
                mem_time.as_millis(),
                mem_result.stats.cycles,
                0,
                0,
                0,
                mem_result.stats.memory_access,
                mem_result.stats.data_moved,
                iterations as u64,
            )
            .with_latency(mem_latency_ns);

            all_results.push(cpu_exp);
            all_results.push(mem_exp);
        } else if is_random {
            // ===== INDEPENDENT RANDOM ACCESS MODE =====

            // CPU mode with random access
            let mut engine = MemoryEngine::new();
            let buf_size = working_set_bytes / 4; // u32 is 4 bytes
            let buf = engine.allocate_buffer(buf_size, 0);

            // Initialize buffer as randomized indices (pre-generated, not during benchmark)
            let random_indices = WorkingSetSweep::generate_random_indices(buf_size);
            if let Some(buf_data) = engine.get_buffer_mut(buf) {
                for (i, &idx) in random_indices.iter().enumerate() {
                    buf_data[i] = idx;
                }
            }

            // Warmup (don't count toward latency)
            let _warmup = engine.execute_cpu(
                Operation::MemRandomAccess,
                &[buf],
                &[warmup_iterations as u32],
            );

            // Measured run
            let start = Instant::now();
            let cpu_result =
                engine.execute_cpu(Operation::MemRandomAccess, &[buf], &[iterations as u32]);
            let cpu_time = start.elapsed();
            let cpu_latency_ns =
                WorkingSetSweep::calculate_latency_ns(cpu_time.as_nanos(), iterations);

            // Memory engine mode with random access
            let mut engine = MemoryEngine::new();
            let buf = engine.allocate_buffer(buf_size, 0);

            // Initialize buffer with same random indices
            if let Some(buf_data) = engine.get_buffer_mut(buf) {
                for (i, &idx) in random_indices.iter().enumerate() {
                    buf_data[i] = idx;
                }
            }

            // Warmup (don't count toward latency)
            let _warmup = engine.execute_memory_engine(
                Operation::MemRandomAccess,
                &[buf],
                &[warmup_iterations as u32],
            );

            // Measured run
            let start = Instant::now();
            let mem_result = engine.execute_memory_engine(
                Operation::MemRandomAccess,
                &[buf],
                &[iterations as u32],
            );
            let mem_time = start.elapsed();
            let mem_latency_ns =
                WorkingSetSweep::calculate_latency_ns(mem_time.as_nanos(), iterations);

            print!(
                "CPU: {:.2} ns/access | ME: {:.2} ns/access\n",
                cpu_latency_ns, mem_latency_ns
            );

            // Store results with exact working set size in bytes (no rounding)
            let cpu_exp = ExperimentResult::new(
                "working_set_sweep",
                "cpu",
                *working_set_bytes as u64,
                cpu_time.as_millis(),
                cpu_result.stats.cycles,
                0,
                0,
                0,
                cpu_result.stats.memory_access,
                cpu_result.stats.data_moved,
                iterations as u64,
            )
            .with_latency(cpu_latency_ns);

            let mem_exp = ExperimentResult::new(
                "working_set_sweep",
                "memory_engine",
                *working_set_bytes as u64,
                mem_time.as_millis(),
                mem_result.stats.cycles,
                0,
                0,
                0,
                mem_result.stats.memory_access,
                mem_result.stats.data_moved,
                iterations as u64,
            )
            .with_latency(mem_latency_ns);

            all_results.push(cpu_exp);
            all_results.push(mem_exp);
        }
    }

    // Export all results
    ExperimentResult::append_batch_to_file(&all_results, export_file)?;

    println!("\n─────────────────────────────────────────────────────────");
    println!("\n✓ Working set sweep experiment completed!");
    println!("✓ Results saved to {}", export_file);

    if is_pointer {
        println!("\nExpected results (pointer chasing):");
        println!("  • Low latency within L1 cache (~4 ns/access)");
        println!("  • Slight increase in L2 cache (~10-12 ns/access)");
        println!("  • Jump at L3 cache boundary (~40-50 ns/access)");
        println!("  • Sharp degradation in DRAM (~100-300 ns/access)");
        println!("\nNote: Pointer chasing prevents prefetching, revealing true");
        println!("      memory hierarchy latencies without optimization.");
    } else if is_random {
        println!("\nExpected results (independent random access):");
        println!("  • Higher latency than sequential (~5-20 ns/access typical)");
        println!("  • Lower latency than pointer chasing (parallelism possible)");
        println!("  • Transitions roughly between two extremes");
        println!("\nNote: Random access defeats prefetching but allows out-of-order");
        println!("      execution and memory parallelism - a natural middle ground.");
    } else {
        println!("\nExpected results (sequential scan):");
        println!("  • Much lower latencies due to prefetching (~2-5 ns/access typical)");
        println!("  • More gradual transitions between cache levels");
        println!("  • Spatial locality hidden by hardware prefetching");
        println!("\nNote: Sequential access enables hardware prefetching and");
        println!("      bandwidth optimization, masking true latency.");
    }

    Ok(())
}
