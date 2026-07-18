use aletheia::runtime::Executor;
use aletheia::workloads::{DatasetScan, VectorAdd};

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("  Aletheia: Processing-Near-Memory Demonstration");
    println!("═══════════════════════════════════════════════════════\n");

    // Initialize executor and memory engine
    let mut executor = Executor::new();
    let engine = executor.engine_mut();

    // Allocate test buffers
    const BUFFER_SIZE: usize = 1_000_000;
    let buf_a = engine.allocate_buffer(BUFFER_SIZE, 100);
    let buf_b = engine.allocate_buffer(BUFFER_SIZE, 200);
    let buf_scan = engine.allocate_buffer(BUFFER_SIZE, 0); // Will be filled with varied values

    // Fill scan buffer with random-ish values for interesting results
    if let Some(buf) = engine.get_buffer_mut(buf_scan) {
        for (i, val) in buf.iter_mut().enumerate() {
            *val = ((i * 7919) % 1000) as u32; // Pseudo-random distribution
        }
    }

    println!("Test Setup:");
    println!(
        "  Buffer Size: {} elements ({} MB each)",
        BUFFER_SIZE,
        BUFFER_SIZE * 4 / 1_000_000
    );
    println!("  Operations: vector operations and dataset scan\n");

    // Demonstration 1: Vector Add
    println!("\n--- Vector Add Operation ---");
    let engine = executor.engine();
    let vec_add_comparison = VectorAdd::compare_modes(engine, buf_a, buf_b);
    vec_add_comparison.print_summary();

    // Demonstration 2: Dataset Scan
    println!("\n--- Dataset Scan (threshold > 500) ---");
    let scan = DatasetScan::new(500);
    let scan_comparison = scan.compare_modes(engine, buf_scan);
    scan_comparison.print_summary();

    // Run full benchmark suite
    println!("\n\n--- Full Benchmark Suite ---");
    let results = executor.run_suite();
    executor.print_benchmark_summary(&results);

    println!("\n═══════════════════════════════════════════════════════");
    println!("  Key Insight:");
    println!("  The Memory Engine mode demonstrates the potential");
    println!("  of processing near memory, reducing unnecessary");
    println!("  data movement between DRAM and CPU.");
    println!("═══════════════════════════════════════════════════════\n");
}
