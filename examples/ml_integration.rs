//! Machine Learning Integration Example
//!
//! This example demonstrates how to integrate ConstraintTheory with
//! machine learning workflows, specifically for embedding normalization
//! and feature discretization.
//!
//! Run with: cargo run --release --example ml_integration

use constraint_theory_core::manifold::PythagoreanManifold;

fn main() {
    println!("================================================");
    println!("ConstraintTheory - ML Integration Example");
    println!("================================================\n");

    // Example 1: Embedding Normalization
    println!("Example 1: Embedding Normalization");
    println!("-----------------------------------\n");

    let manifold = PythagoreanManifold::new(200);

    // Simulated ML embeddings (typically from neural networks)
    let embeddings = vec![
        [0.342, 0.940],  // Close to (3/5, 4/5) = (0.6, 0.8)
        [0.600, 0.800],  // Exact Pythagorean triple
        [0.707, 0.707],  // Close to 1/√2 ≈ 0.7071
        [0.960, 0.280],  // Close to (24/25, 7/25) = (0.96, 0.28)
    ];

    println!("Original embeddings -> Normalized embeddings:");
    for emb in &embeddings {
        let (normalized, noise) = manifold.snap(*emb);
        println!(
            "  ({:.3}, {:.3}) -> ({:.3}, {:.3}) | noise = {:.4}",
            emb[0], emb[1], normalized[0], normalized[1], noise
        );
    }

    // Example 2: Feature Discretization
    println!("\n\nExample 2: Feature Discretization");
    println!("-----------------------------------\n");

    // Discretize continuous angles into Pythagorean directions
    let angles: Vec<f32> = (0..8).map(|i| i as f32 * std::f32::consts::PI / 4.0).collect();

    println!("Discretizing angles to 8 principal directions:");
    for angle in angles {
        let vec = [angle.cos(), angle.sin()];
        let (discretized, noise) = manifold.snap(vec);
        println!(
            "  Angle {:.2} rad -> ({:.3}, {:.3}) | noise = {:.4}",
            angle, discretized[0], discretized[1], noise
        );
    }

    // Example 3: Batch Processing for ML Inference
    println!("\n\nExample 3: Batch Processing for ML Inference");
    println!("---------------------------------------------\n");

    // Simulate a batch of 1000 embeddings
    let batch_size = 1000;
    let mut batch = Vec::with_capacity(batch_size);
    for i in 0..batch_size {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / batch_size as f32;
        batch.push([angle.cos(), angle.sin()]);
    }

    // Normalize entire batch
    let start = std::time::Instant::now();
    let results: Vec<_> = batch.iter().map(|v| manifold.snap(*v)).collect();
    let elapsed = start.elapsed();

    let avg_noise: f32 = results.iter().map(|(_, n)| n).sum::<f32>() / batch_size as f32;

    println!("Batch processing results:");
    println!("  Batch size: {}", batch_size);
    println!("  Total time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("  Per-vector: {:.2} μs", elapsed.as_micros() as f64 / batch_size as f64);
    println!("  Average noise: {:.4}", avg_noise);
    println!("  Throughput: {:.0} vectors/sec", batch_size as f64 / elapsed.as_secs_f64());

    // Example 4: Feature Bucketing
    println!("\n\nExample 4: Feature Bucketing");
    println!("----------------------------\n");

    // Create buckets for quantizing features
    let bucket_count = 12;
    println!("Creating {} discrete feature buckets:", bucket_count);

    for i in 0..bucket_count {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / bucket_count as f32;
        let vec = [angle.cos(), angle.sin()];
        let (bucket_center, _) = manifold.snap(vec);

        println!(
            "  Bucket {:2}: ({:.3}, {:.3})",
            i, bucket_center[0], bucket_center[1]
        );
    }

    // Example 5: Dimensionality Reduction Validation
    println!("\n\nExample 5: Dimensionality Reduction Validation");
    println!("-----------------------------------------------\n");

    // Validate that snapped vectors maintain geometric relationships
    let test_vectors = vec![
        ([1.0, 0.0], [0.0, 1.0]),
        ([0.707, 0.707], [-0.707, 0.707]),
        ([0.6, 0.8], [-0.6, -0.8]),
    ];

    println!("Validating orthogonal relationships after snapping:");
    for (v1, v2) in test_vectors {
        let (s1, _) = manifold.snap(v1);
        let (s2, _) = manifold.snap(v2);

        let dot_product = s1[0] * s2[0] + s1[1] * s2[1];
        let is_orthogonal = dot_product.abs() < 0.1;

        println!(
            "  ({:.2}, {:.2}) ⊥ ({:.2}, {:.2}) -> dot = {:.3}, orthogonal = {}",
            v1[0], v1[1], v2[0], v2[1], dot_product, is_orthogonal
        );
    }

    println!("\n================================================");
    println!("ML Integration Example Complete");
    println!("================================================");
}
