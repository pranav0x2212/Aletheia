use std::hint::black_box;

use aletheia::profiler::measure;

fn main() -> anyhow::Result<()> {
    let (sum, counters) = measure(|| {
        let mut sum = black_box(0u64);

        for i in 0..100_000_000u64 {
            sum = black_box(sum.wrapping_add(black_box(i)));
        }

        black_box(sum)
    })?;

    println!("Result: {}", sum);
    println!("{:#?}", counters);

    Ok(())
}
