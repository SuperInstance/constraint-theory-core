//! SIMD optimization example for constraint-theory-core
//!
//! Demonstrates SIMD capabilities and optimizations:
//! - Runtime SIMD detection
//! - SIMD vs scalar performance comparison
//! - Optimal batch sizes for SIMD
//! - Platform-specific considerations

use constraint_theory_core::PythagoreanManifold;
use std::time::Instant;

#[cfg(target_arch = "x86_64")]
use constraint_theory_core::simd::is_avx2_available;

fn main() {
    println!("=== Constraint Theory Core - SIMD Optimization Example ===\n");

    // Example 1: SIMD capability detection
    println!("--- Example 1: SIMD Capability Detection ---");
    
    #[cfg(target_arch = "x86_64")]
    {
        let has_avx2 = is_avx2_available();
        println!("Platform: x86_64");
        println!("AVX2 available: {}", has_avx2);
        
        if has_avx2 {
            println!("SIMD will use AVX2 (8x f32 parallelism)");
        } else {
            println!("SIMD will fall back to scalar implementation");
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        println!("Platform: ARM64");
        println!("NEON available: true (always on for ARM64)");
        println!("SIMD will use NEON (4x f32 parallelism)");
    }
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        println!("Platform: Unknown/Other");
        println!("SIMD: Scalar fallback");
    }
    println!();

    // Create manifold
    let manifold = PythagoreanManifold::new(200);
    println!("Manifold: {} states\n", manifold.state_count());

    // Example 2: Optimal batch sizes
    println!("--- Example 2: Optimal Batch Sizes ---");
    println!("SIMD processes multiple vectors in parallel.");
    println!("Optimal batch sizes depend on SIMD width:\n");

    #[cfg(target_arch = "x86_64")]
    {
        if is_avx2_available() {
            println!("AVX2 processes 8 vectors at a time");
            println!("Optimal batch sizes: multiples of 8");
            println!("  - Minimum efficient: 8");
            println!("  - Good: 64, 128, 256");
            println!("  - Large: 1000+");
        }
    }
    println!();

    // Example 3: Batch size performance comparison
    println!("--- Example 3: Batch Size Performance ---");
    let batch_sizes = [8, 16, 64, 256, 1024, 4096, 16384];

    println!("{:>10} {:>12} {:>12} {:>8}", 
             "Batch Size", "Scalar (us)", "SIMD (us)", "Speedup");
    println!("{:-<10} {:->12} {:->12} {:->8}", "", "", "", "");

    for &size in &batch_sizes {
        let vectors: Vec<[f32; 2]> = (0..size)
            .map(|i| {
                let angle = (i as f32) * 0.01;
                [angle.cos(), angle.sin()]
            })
            .collect();

        // Warmup
        let mut warmup = vec![([0.0, 0.0], 0.0f32); size];
        manifold.snap_batch_simd_into(&vectors, &mut warmup);

        // Scalar
        let mut results_scalar = vec![([0.0, 0.0], 0.0f32); size];
        let start = Instant::now();
        manifold.snap_batch(&vectors, &mut results_scalar);
        let scalar_time = start.elapsed();

        // SIMD
        let mut results_simd = vec![([0.0, 0.0], 0.0f32); size];
        let start = Instant::now();
        manifold.snap_batch_simd_into(&vectors, &mut results_simd);
        let simd_time = start.elapsed();

        let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
        let scalar_us = scalar_time.as_nanos() as f64 / 1000.0;
        let simd_us = simd_time.as_nanos() as f64 / 1000.0;

        println!("{:>10} {:>12.2} {:>12.2} {:>8.2}x", 
                 size, scalar_us, simd_us, speedup);
    }
    println!();

    // Example 4: Memory-aligned processing
    println!("--- Example 4: Memory-Efficient Processing ---");
    println!("For large datasets, pre-allocate buffers and reuse:\n");

    let large_batch_size = 100_000;
    let mut vectors = vec![[0.0f32, 0.0]; large_batch_size];
    let mut results = vec![([0.0f32, 0.0], 0.0f32); large_batch_size];

    // Initialize vectors
    for (i, v) in vectors.iter_mut().enumerate() {
        let angle = (i as f32) * 0.0001;
        *v = [angle.cos(), angle.sin()];
    }

    // Process multiple times (simulating streaming)
    let iterations = 10;
    let start = Instant::now();
    
    for _ in 0..iterations {
        manifold.snap_batch_simd_into(&vectors, &mut results);
    }

    let total_time = start.elapsed();
    let total_vectors = large_batch_size * iterations;
    let throughput = total_vectors as f64 / total_time.as_secs_f64();

    println!("Batch size: {}", large_batch_size);
    println!("Iterations: {}", iterations);
    println!("Total vectors: {}", total_vectors);
    println!("Total time: {:?}", total_time);
    println!("Throughput: {:.0} vectors/sec", throughput);
    println!("Per-vector latency: {:.2} ns\n", 
             total_time.as_nanos() as f64 / total_vectors as f64);

    // Example 5: SIMD for different manifold sizes
    println!("--- Example 5: SIMD vs Manifold Density ---");
    println!("SIMD speedup varies with manifold size:\n");

    let batch_size = 10_000;
    let vectors: Vec<[f32; 2]> = (0..batch_size)
        .map(|i| {
            let angle = (i as f32) * 0.001;
            [angle.cos(), angle.sin()]
        })
        .collect();

    println!("{:>10} {:>12} {:>12} {:>8}", 
             "Density", "Scalar (us)", "SIMD (us)", "Speedup");
    println!("{:-<10} {:->12} {:->12} {:->8}", "", "", "", "");

    for density in [50, 100, 200, 500] {
        let m = PythagoreanManifold::new(density);

        let mut results_scalar = vec![([0.0, 0.0], 0.0f32); batch_size];
        let start = Instant::now();
        m.snap_batch(&vectors, &mut results_scalar);
        let scalar_time = start.elapsed();

        let mut results_simd = vec![([0.0, 0.0], 0.0f32); batch_size];
        let start = Instant::now();
        m.snap_batch_simd_into(&vectors, &mut results_simd);
        let simd_time = start.elapsed();

        let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
        let scalar_us = scalar_time.as_nanos() as f64 / 1000.0;
        let simd_us = simd_time.as_nanos() as f64 / 1000.0;

        println!("{:>10} {:>12.2} {:>12.2} {:>8.2}x", 
                 density, scalar_us, simd_us, speedup);
    }
    println!();

    // Example 6: Building for maximum SIMD performance
    println!("--- Example 6: Build Optimization ---");
    println!("For maximum SIMD performance, compile with:\n");
    println!("  RUSTFLAGS=\"-C target-cpu=native\" cargo build --release");
    println!();
    println!("This enables:");
    println!("  - AVX2/AVX-512 on supported x86_64 CPUs");
    println!("  - NEON on ARM64");
    println!("  - CPU-specific optimizations");
    println!();

    // Example 7: SIMD feature flag
    println!("--- Example 7: Feature Flags ---");
    println!("The simd feature flag controls SIMD behavior:\n");
    println!("  # Cargo.toml");
    println!("  [dependencies]");
    println!("  constraint-theory-core = {{ version = \"1.0\", features = [\"simd\"] }");
    println!();
    println!("Runtime detection ensures safe fallback on unsupported CPUs.");
    println!();

    println!("=== SIMD Optimization Example Complete ===");
}
