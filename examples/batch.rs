//! Batch processing example for constraint-theory-core
//!
//! Demonstrates high-throughput batch operations:
//! - Scalar batch processing
//! - SIMD-optimized batch processing
//! - Performance comparisons
//! - Pre-allocated buffer usage

use constraint_theory_core::PythagoreanManifold;
use std::time::Instant;

fn main() {
    println!("=== Constraint Theory Core - Batch Processing Example ===\n");

    // Create manifold
    let density = 200;
    let manifold = PythagoreanManifold::new(density);
    println!("Manifold: {} states\n", manifold.state_count());

    // Example 1: Scalar batch processing
    println!("--- Example 1: Scalar Batch Processing ---");
    let vectors: Vec<[f32; 2]> = (0..1000)
        .map(|i| {
            let angle = (i as f32) * 0.001 * std::f32::consts::TAU;
            [angle.cos(), angle.sin()]
        })
        .collect();

    let mut results_scalar = vec![([0.0, 0.0], 0.0f32); vectors.len()];
    manifold.snap_batch(&vectors, &mut results_scalar);

    println!("Processed {} vectors (scalar)", vectors.len());
    println!("First result: {:?}", results_scalar[0]);
    println!("Last result:  {:?}\n", results_scalar[vectors.len() - 1]);

    // Example 2: SIMD batch processing
    println!("--- Example 2: SIMD Batch Processing ---");
    let mut results_simd = vec![([0.0, 0.0], 0.0f32); vectors.len()];
    manifold.snap_batch_simd_into(&vectors, &mut results_simd);

    println!("Processed {} vectors (SIMD)", vectors.len());
    println!("First result: {:?}", results_simd[0]);
    println!("Last result:  {:?}\n", results_simd[vectors.len() - 1]);

    // Example 3: Convenience method
    println!("--- Example 3: Convenience Method ---");
    let small_batch: Vec<[f32; 2]> = vec![
        [0.6, 0.8],
        [0.8, 0.6],
        [0.707, 0.707],
        [0.5, 0.866],
    ];
    
    let results = manifold.snap_batch_simd(&small_batch);
    for (i, (snapped, noise)) in results.iter().enumerate() {
        println!("  Vector {}: {:?} -> {:?} (noise: {:.4})", 
                 i, small_batch[i], snapped, noise);
    }
    println!();

    // Example 4: Performance comparison
    println!("--- Example 4: Performance Comparison ---");
    let large_batch: Vec<[f32; 2]> = (0..100_000)
        .map(|i| {
            let angle = (i as f32) * 0.0001;
            [angle.cos(), angle.sin()]
        })
        .collect();

    // Warmup
    let mut warmup = vec![([0.0, 0.0], 0.0f32); 1000];
    manifold.snap_batch_simd_into(&large_batch[..1000], &mut warmup);

    // Scalar benchmark
    let mut results_scalar = vec![([0.0, 0.0], 0.0f32); large_batch.len()];
    let start = Instant::now();
    manifold.snap_batch(&large_batch, &mut results_scalar);
    let scalar_time = start.elapsed();

    // SIMD benchmark
    let mut results_simd = vec![([0.0, 0.0], 0.0f32); large_batch.len()];
    let start = Instant::now();
    manifold.snap_batch_simd_into(&large_batch, &mut results_simd);
    let simd_time = start.elapsed();

    println!("Batch size: {}", large_batch.len());
    println!("Scalar time: {:?}", scalar_time);
    println!("SIMD time:   {:?}", simd_time);
    
    let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
    println!("Speedup:     {:.2}x\n", speedup);

    // Example 5: Streaming large datasets
    println!("--- Example 5: Streaming Large Datasets ---");
    let chunk_size = 10_000;
    let total_vectors = 1_000_000;
    let chunks = total_vectors / chunk_size;

    let mut chunk = vec![([0.0, 0.0], 0.0f32); chunk_size];
    let start = Instant::now();

    for chunk_idx in 0..chunks {
        // Generate chunk (in real use, this would be from a data source)
        let vectors: Vec<[f32; 2]> = (0..chunk_size)
            .map(|i| {
                let idx = chunk_idx * chunk_size + i;
                let angle = (idx as f32) * 0.000001;
                [angle.cos(), angle.sin()]
            })
            .collect();

        // Process chunk
        manifold.snap_batch_simd_into(&vectors, &mut chunk);
    }

    let total_time = start.elapsed();
    let vectors_per_sec = total_vectors as f64 / total_time.as_secs_f64();

    println!("Processed {} vectors in {:?}", total_vectors, total_time);
    println!("Throughput: {:.0} vectors/sec", vectors_per_sec);
    println!("Latency per vector: {:.2} ns\n", 
             total_time.as_nanos() as f64 / total_vectors as f64);

    // Example 6: Verify results consistency
    println!("--- Example 6: Verify Consistency ---");
    let test_vectors: Vec<[f32; 2]> = vec![
        [0.6, 0.8], [0.8, 0.6], [0.707, 0.707],
        [0.5, 0.866], [0.866, 0.5], [0.309, 0.951],
    ];

    let mut scalar_results = vec![([0.0, 0.0], 0.0f32); test_vectors.len()];
    let mut simd_results = vec![([0.0, 0.0], 0.0f32); test_vectors.len()];

    manifold.snap_batch(&test_vectors, &mut scalar_results);
    manifold.snap_batch_simd_into(&test_vectors, &mut simd_results);

    let mut all_match = true;
    for i in 0..test_vectors.len() {
        let s = scalar_results[i];
        let simd = simd_results[i];
        
        let x_match = (s.0[0] - simd.0[0]).abs() < 0.001;
        let y_match = (s.0[1] - simd.0[1]).abs() < 0.001;
        let noise_match = (s.1 - simd.1).abs() < 0.001;
        
        if !(x_match && y_match && noise_match) {
            println!("MISMATCH at index {}:", i);
            println!("  Scalar: {:?}", s);
            println!("  SIMD:   {:?}", simd);
            all_match = false;
        }
    }

    if all_match {
        println!("All {} results match between scalar and SIMD!", test_vectors.len());
    }

    println!("\n=== Batch Processing Example Complete ===");
}
