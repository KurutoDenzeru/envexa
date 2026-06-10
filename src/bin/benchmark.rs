use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("--- ENVEXA BENCHMARK ---");
    
    // Benchmark Brew specifically
    println!("Benchmarking Brew Scanner...");
    let mut brew_times = vec![];
    for _ in 0..5 {
        let start = Instant::now();
        let _ = envexa::toolchains::scan_one("brew").await;
        let duration = start.elapsed();
        brew_times.push(duration);
    }
    let brew_avg = brew_times.iter().sum::<std::time::Duration>() / brew_times.len() as u32;
    println!("Brew Scanner Average (5 runs): {:.2?}", brew_avg);
    println!("Brew Scanner Nanoseconds: {} ns", brew_avg.as_nanos());
    println!("Brew Scanner Microseconds: {} µs", brew_avg.as_micros());
    println!("Brew Scanner Milliseconds: {} ms", brew_avg.as_millis());
    
    // Benchmark End-to-End Scan
    println!("\nBenchmarking End-to-End Scan (Equivalent to TUI/CLI wait time)...");
    let start = Instant::now();
    let _ = envexa::toolchains::scan_all_with(30, None).await;
    let scan_all_duration = start.elapsed();
    println!("End-to-End Scan Duration: {:.2?}", scan_all_duration);
    println!("End-to-End Scan Milliseconds: {} ms", scan_all_duration.as_millis());
    
    println!("------------------------");
}
