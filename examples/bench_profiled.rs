//! Advanced performance benchmark with detailed profiling
//!
//! This benchmark provides detailed timing breakdowns and metrics
//! to guide optimization efforts.
//!
//! Run with:
//!   cargo run --release --example bench_profiled

use constraint_theory_core::PythagoreanManifold;
use std::time::Instant;

fn main() {
    let n = 100_000;
    let warmup = 10_000;
    let iterations = 5;

    println!("========================================");
    println!("Constraint Theory - Detailed Profiling");
    println!("========================================\n");

    // Create manifold with 200 density (~1000 states)
    let manifold = PythagoreanManifold::new(200);
    println!("Manifold states: {}", manifold.state_count());
    println!(
        "Manifold memory: {} KB\n",
        manifold.state_count() * 8 / 1024
    );

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

    println!("Test vectors: {}", n);
    println!("Vector memory: {} KB\n", vectors.len() * 8 / 1024);

    // ========================================
    // Warmup
    // ========================================
    println!("Warming up...");
    for _ in 0..3 {
        let _ = manifold.snap_batch_simd(&warmup_vectors);
        for vec in &warmup_vectors {
            let _ = manifold.snap(*vec);
        }
    }
    println!("Warmup complete.\n");

    // ========================================
    // Detailed profiling breakdown
    // ========================================
    println!("========================================");
    println!("Detailed Profiling Breakdown");
    println!("========================================\n");

    let mut profile_data = Vec::new();

    for iter in 0..iterations {
        println!("--- Iteration {} ---", iter);

        let total_start = Instant::now();

        // Phase 1: Normalization (scalar, before SIMD)
        let norm_start = Instant::now();
        let mut normalized = Vec::with_capacity(n);
        for vec in &vectors {
            let norm = (vec[0] * vec[0] + vec[1] * vec[1]).sqrt();
            normalized.push([vec[0] / norm, vec[1] / norm]);
        }
        let norm_time = norm_start.elapsed();

        // Phase 2: SIMD batch snapping
        let simd_start = Instant::now();
        let mut results = vec![([0.0f32, 0.0], 0.0f32); n];
        manifold.snap_batch_simd_into(&vectors, &mut results);
        let simd_time = simd_start.elapsed();

        // Phase 3: Result validation
        let valid_start = Instant::now();
        let total_noise: f32 = results.iter().map(|(_, n)| *n).sum();
        let valid_time = valid_start.elapsed();

        let total_time = total_start.elapsed();

        profile_data.push((norm_time, simd_time, valid_time, total_time));

        println!(
            "  Total:           {:.2} ms",
            total_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Normalization:   {:.2} ms ({:.1}%)",
            norm_time.as_secs_f64() * 1000.0,
            (norm_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
        );
        println!(
            "  SIMD snapping:   {:.2} ms ({:.1}%)",
            simd_time.as_secs_f64() * 1000.0,
            (simd_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
        );
        println!(
            "  Validation:      {:.2} ms ({:.1}%)",
            valid_time.as_secs_f64() * 1000.0,
            (valid_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
        );
        println!("  Total noise:     {:.6}", total_noise);
        println!();
    }

    // ========================================
    // Summary statistics
    // ========================================
    println!("========================================");
    println!("Summary Statistics");
    println!("========================================\n");

    let avg_norm: f64 = profile_data
        .iter()
        .map(|(n, _, _, _)| n.as_secs_f64())
        .sum::<f64>()
        / iterations as f64;
    let avg_simd: f64 = profile_data
        .iter()
        .map(|(_, s, _, _)| s.as_secs_f64())
        .sum::<f64>()
        / iterations as f64;
    let avg_valid: f64 = profile_data
        .iter()
        .map(|(_, _, v, _)| v.as_secs_f64())
        .sum::<f64>()
        / iterations as f64;
    let avg_total: f64 = profile_data
        .iter()
        .map(|(_, _, _, t)| t.as_secs_f64())
        .sum::<f64>()
        / iterations as f64;

    println!("Average times:");
    println!("  Total:           {:.2} ms", avg_total * 1000.0);
    println!(
        "  Normalization:   {:.2} ms ({:.1}%)",
        avg_norm * 1000.0,
        (avg_norm / avg_total) * 100.0
    );
    println!(
        "  SIMD snapping:   {:.2} ms ({:.1}%)",
        avg_simd * 1000.0,
        (avg_simd / avg_total) * 100.0
    );
    println!(
        "  Validation:      {:.2} ms ({:.1}%)",
        avg_valid * 1000.0,
        (avg_valid / avg_total) * 100.0
    );
    println!();

    // ========================================
    // Performance metrics
    // ========================================
    println!("========================================");
    println!("Performance Metrics");
    println!("========================================\n");

    let per_tile_ns = avg_simd * 1e9 / n as f64;
    let per_tile_us = per_tile_ns / 1000.0;
    let throughput = n as f64 / avg_simd;

    println!("SIMD Performance:");
    println!(
        "  Per-tile:        {:.2} ns ({:.3} us)",
        per_tile_ns, per_tile_us
    );
    println!("  Throughput:      {:.0} tiles/sec", throughput);
    println!();

    // Calculate FLOPs
    let flops_per_vector = manifold.state_count() as f64 * 2.0; // 2 ops per dot product
    let total_flops = flops_per_vector * n as f64;
    let gflops = total_flops / (avg_simd * 1e9);

    println!("Compute Metrics:");
    println!("  FLOPs/vector:    {:.0}", flops_per_vector);
    println!("  Total FLOPs:     {:.2} GF", total_flops / 1e9);
    println!("  Performance:     {:.2} GFLOPS", gflops);
    println!();

    // Calculate memory bandwidth
    let bytes_per_vector = manifold.state_count() * 8; // Read all states
    let total_bytes = bytes_per_vector * n;
    let bandwidth_gb = total_bytes as f64 / (avg_simd * 1e9);

    println!("Memory Metrics:");
    println!(
        "  Bytes/vector:    {:.0} KB",
        bytes_per_vector as f64 / 1024.0
    );
    println!("  Total memory:    {:.2} GB", total_bytes as f64 / 1e9);
    println!("  Bandwidth:       {:.2} GB/sec", bandwidth_gb);
    println!();

    // Arithmetic intensity
    let arithmetic_intensity = total_flops / total_bytes as f64;

    println!(
        "Arithmetic Intensity: {:.2} FLOPs/byte",
        arithmetic_intensity
    );

    if arithmetic_intensity < 1.0 {
        println!("  Status: MEMORY-BOUND (optimize memory access)");
    } else {
        println!("  Status: COMPUTE-BOUND (optimize computations)");
    }
    println!();

    // ========================================
    // SIMD efficiency analysis
    // ========================================
    println!("========================================");
    println!("SIMD Efficiency Analysis");
    println!("========================================\n");

    // Benchmark scalar version for comparison
    println!("Benchmarking scalar version...");
    let scalar_start = Instant::now();
    let mut _total_noise = 0.0f32;
    for vec in &vectors {
        let (_, noise) = manifold.snap(*vec);
        _total_noise += noise;
    }
    let scalar_time = scalar_start.elapsed();

    let scalar_per_tile_us = scalar_time.as_secs_f64() * 1e6 / n as f64;
    let speedup = scalar_time.as_secs_f64() / avg_simd;
    let theoretical_speedup = 8.0; // AVX2 processes 8 floats
    let efficiency = speedup / theoretical_speedup;

    println!("Scalar Performance:");
    println!("  Per-tile:        {:.2} us", scalar_per_tile_us);
    println!(
        "  Total time:      {:.2} ms",
        scalar_time.as_secs_f64() * 1000.0
    );
    println!();

    println!("SIMD vs Scalar:");
    println!("  Speedup:         {:.2}x", speedup);
    println!("  Theoretical:     {:.2}x (AVX2)", theoretical_speedup);
    println!("  Efficiency:      {:.1}%", efficiency * 100.0);
    println!();

    if efficiency < 0.5 {
        println!("  Status: POOR SIMD utilization (<50%)");
        println!("  Recommendation: Check for scalar code in SIMD function");
    } else if efficiency < 0.8 {
        println!("  Status: MODERATE SIMD utilization (50-80%)");
        println!("  Recommendation: Reduce scalar overhead");
    } else {
        println!("  Status: GOOD SIMD utilization (>80%)");
    }
    println!();

    // ========================================
    // Optimization recommendations
    // ========================================
    println!("========================================");
    println!("Optimization Recommendations");
    println!("========================================\n");

    // Analyze bottleneck
    let norm_pct = (avg_norm / avg_simd) * 100.0;

    if norm_pct > 10.0 {
        println!("1. HIGH PRIORITY: SIMD normalization");
        println!("   - Normalization consumes {:.1}% of SIMD time", norm_pct);
        println!("   - Move normalization into SIMD kernel");
        println!("   - Expected speedup: 1.5-2x");
        println!();
    }

    if per_tile_us > 1.0 {
        println!("2. HIGH PRIORITY: Replace linear search with KD-tree");
        println!(
            "   - Current: O(N) search through {} states",
            manifold.state_count()
        );
        println!(
            "   - With KD-tree: O(log N) ≈ {} comparisons",
            (manifold.state_count() as f64).log2() as usize
        );
        println!("   - Expected speedup: 5-10x");
        println!();
    }

    if arithmetic_intensity < 1.0 {
        println!("3. MEDIUM PRIORITY: Optimize memory access");
        println!(
            "   - Memory-bound kernel (AI = {:.2})",
            arithmetic_intensity
        );
        println!("   - Implement cache-aligned data structures");
        println!("   - Add prefetching for next cache lines");
        println!("   - Expected speedup: 2-3x");
        println!();
    }

    if efficiency < 0.8 {
        println!("4. MEDIUM PRIORITY: Improve SIMD utilization");
        println!("   - Current efficiency: {:.1}%", efficiency * 100.0);
        println!("   - Eliminate horizontal operations");
        println!("   - Use AVX-512 if available");
        println!("   - Expected speedup: 1.5-2x");
        println!();
    }

    println!("5. LOW PRIORITY: Multi-threading");
    println!("   - Add Rayon for parallel processing");
    println!("   - Expected speedup: 4-8x (number of cores)");
    println!("   - Only after single-threaded is optimized");
    println!();

    // ========================================
    // Target projection
    // ========================================
    println!("========================================");
    println!("Target Projection");
    println!("========================================\n");

    let current_per_tile_us = per_tile_us;
    let target_per_tile_us = 0.10;
    let gap = current_per_tile_us / target_per_tile_us;

    println!("Current:  {:.3} us/tile", current_per_tile_us);
    println!("Target:   {:.3} us/tile", target_per_tile_us);
    println!("Gap:      {:.1}x slower", gap);
    println!();

    println!("Potential speedups:");
    println!("  + KD-tree:              5-10x");
    println!("  + SIMD optimization:    1.5-2x");
    println!("  + Memory optimization:  2-3x");
    println!("  + Multi-threading:      4-8x");
    println!();

    let conservative = 5.0 * 1.5 * 2.0; // KD-tree + SIMD + memory
    let aggressive = 10.0 * 2.0 * 3.0 * 8.0; // All optimizations

    println!("Expected results:");
    println!(
        "  Conservative (Phases 1-3):  {:.3} us/tile ({:.1}x)",
        current_per_tile_us / conservative,
        conservative
    );
    println!(
        "  Aggressive (all phases):    {:.3} us/tile ({:.1}x)",
        current_per_tile_us / aggressive,
        aggressive
    );
    println!();

    if conservative >= gap {
        println!("  Status: ON TRACK to meet target");
    } else {
        println!("  Status: NEEDS ALL OPTIMIZATIONS to meet target");
    }

    println!();
    println!("========================================");
    println!("Profiling Complete");
    println!("========================================");
}
