//! Benchmark for Constraint Theory Core - SIMD vs Scalar Performance
//!
//! This benchmark compares the performance of:
//! - Scalar implementation (one vector at a time)
//! - SIMD implementation (8 vectors at a time via AVX2)
//!
//! Run with: cargo run --release --example bench

use constraint_theory_core::{snap, PythagoreanManifold};

fn main() {
    let n = 100_000;
    let warmup = 10_000;
    let iterations = 5;

    println!("========================================");
    println!("Constraint Theory Core - SIMD Benchmark");
    println!("========================================\n");

    // Create manifold with 200 density (~1000 states)
    let manifold = PythagoreanManifold::new(200);
    println!("Manifold states: {}", manifold.state_count());

    // Generate test vectors
    let vectors: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let angle = (i as f32) * 0.0001;
            [angle.sin(), angle.cos()]
        })
        .collect();

    let warmup_vectors: Vec<[f32; 2]> = (0..warmup)
        .map(|i| {
            let angle = (i as f32) * 0.0001;
            [angle.sin(), angle.cos()]
        })
        .collect();

    println!("Test vectors: {}\n", n);

    // ========================================
    // Warmup runs (to stabilize CPU frequency)
    // ========================================
    println!("Warming up...");
    for _ in 0..3 {
        let _ = manifold.snap_batch_simd(&warmup_vectors);
        for vec in &warmup_vectors {
            let _ = snap(&manifold, *vec);
        }
    }
    println!("Warmup complete.\n");

    // ========================================
    // SIMD Benchmark
    // ========================================
    println!("--- SIMD Implementation (AVX2) ---");

    let mut simd_times = Vec::with_capacity(iterations);
    let mut simd_total_noise = 0.0f32;

    for iter in 0..iterations {
        let start = std::time::Instant::now();

        // Use pre-allocated buffer for maximum performance
        let mut results = vec![([0.0f32, 0.0], 0.0f32); n];
        manifold.snap_batch_simd_into(&vectors, &mut results);

        let duration = start.elapsed();
        simd_times.push(duration);

        // Verify results on first iteration
        if iter == 0 {
            simd_total_noise = results.iter().map(|(_, n)| *n).sum();
        }
    }

    let simd_avg_ns: f64 =
        simd_times.iter().map(|d| d.as_nanos() as f64).sum::<f64>() / iterations as f64;

    let simd_avg_per_tile_ns = simd_avg_ns / n as f64;
    let simd_throughput = (n as f64 * 1e9) / simd_avg_ns;

    println!("  Iterations: {}", iterations);
    println!("  Average time: {:.2} ms", simd_avg_ns / 1e6);
    println!(
        "  Per-tile: {:.2} ns ({:.3} us)",
        simd_avg_per_tile_ns,
        simd_avg_per_tile_ns / 1000.0
    );
    println!("  Throughput: {:.0} tiles/sec", simd_throughput);
    println!("  Total noise: {:.4}", simd_total_noise);

    // ========================================
    // Scalar Benchmark
    // ========================================
    println!("\n--- Scalar Implementation ---");

    let mut scalar_times = Vec::with_capacity(iterations);
    let mut scalar_total_noise = 0.0f32;

    for iter in 0..iterations {
        let start = std::time::Instant::now();

        let mut total_noise = 0.0f32;
        for vec in &vectors {
            let (_, noise) = snap(&manifold, *vec);
            total_noise += noise;
        }

        let duration = start.elapsed();
        scalar_times.push(duration);

        if iter == 0 {
            scalar_total_noise = total_noise;
        }
    }

    let scalar_avg_ns: f64 = scalar_times
        .iter()
        .map(|d| d.as_nanos() as f64)
        .sum::<f64>()
        / iterations as f64;

    let scalar_avg_per_tile_ns = scalar_avg_ns / n as f64;
    let scalar_throughput = (n as f64 * 1e9) / scalar_avg_ns;

    println!("  Iterations: {}", iterations);
    println!("  Average time: {:.2} ms", scalar_avg_ns / 1e6);
    println!(
        "  Per-tile: {:.2} ns ({:.3} us)",
        scalar_avg_per_tile_ns,
        scalar_avg_per_tile_ns / 1000.0
    );
    println!("  Throughput: {:.0} tiles/sec", scalar_throughput);
    println!("  Total noise: {:.4}", scalar_total_noise);

    // ========================================
    // Comparison
    // ========================================
    println!("\n========================================");
    println!("Performance Comparison");
    println!("========================================");

    let speedup = scalar_avg_ns / simd_avg_ns;

    println!("  SIMD speedup:     {:.1}x", speedup);
    println!(
        "  Time saved:       {:.2} ms per batch",
        (scalar_avg_ns - simd_avg_ns) / 1e6
    );
    println!("  Target:           8-16x (AVX2 theoretical max)");
    println!();

    // Performance targets
    println!("Performance Targets:");
    println!(
        "  Current SIMD:     {:.2} us/tile",
        simd_avg_per_tile_ns / 1000.0
    );
    println!("  Target:           <0.1 us/tile");
    println!(
        "  Progress:         {:.1}%",
        (0.1 / (simd_avg_per_tile_ns / 1000.0)) * 100.0
    );
    println!();

    // Noise verification (should match between implementations)
    let noise_diff = (simd_total_noise - scalar_total_noise).abs();
    if noise_diff < 0.01 {
        println!(
            "Verification: SIMD results match scalar (noise diff: {:.6})",
            noise_diff
        );
    } else {
        println!(
            "WARNING: SIMD results differ from scalar (noise diff: {:.6})",
            noise_diff
        );
    }

    // Individual iteration details
    println!("\n--- Detailed Results ---");
    println!(
        "SIMD times (ms):   {:?}",
        simd_times.iter().map(|d| d.as_millis()).collect::<Vec<_>>()
    );
    println!(
        "Scalar times (ms): {:?}",
        scalar_times
            .iter()
            .map(|d| d.as_millis())
            .collect::<Vec<_>>()
    );

    // Success criteria
    println!("\n========================================");
    println!("Success Criteria");
    println!("========================================");

    if speedup >= 8.0 {
        println!("  [PASS] Speedup >= 8x: {:.1}x", speedup);
    } else if speedup >= 4.0 {
        println!("  [PARTIAL] Speedup >= 4x: {:.1}x (target: 8x)", speedup);
    } else {
        println!("  [FAIL] Speedup < 4x: {:.1}x (target: 8x)", speedup);
    }

    if simd_avg_per_tile_ns < 1000.0 {
        println!("  [PASS] Per-tile < 1us: {:.2} ns", simd_avg_per_tile_ns);
    } else {
        println!(
            "  [PARTIAL] Per-tile: {:.2} ns (target: <1000 ns)",
            simd_avg_per_tile_ns
        );
    }

    if noise_diff < 0.01 {
        println!("  [PASS] Results verified: SIMD matches scalar");
    } else {
        println!("  [FAIL] Results mismatch: noise diff = {:.6}", noise_diff);
    }

    println!("\n========================================");
    println!("Next Steps for Further Optimization");
    println!("========================================");
    println!("1. Add KD-tree for O(log N) state lookup (5-10x speedup)");
    println!("2. Implement cache-aligned memory layout (2-3x speedup)");
    println!("3. Add AVX-512 support for 16x parallelism (2x speedup)");
    println!("4. Consider CUDA for GPU acceleration (100-1000x speedup)");
}
