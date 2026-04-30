//! Hidden Dimensions Encoding Example
//!
//! Demonstrates the Grand Unified Constraint Theory (GUCT) approach to
//! exact constraint satisfaction using hidden dimensions.
//!
//! # Key Concepts
//!
//! - Hidden dimensions encode precision logarithmically: k = ⌈log₂(1/ε)⌉
//! - Lifting to higher dimensions enables exact constraint satisfaction
//! - Projecting back preserves the constraint properties

use constraint_theory_core::{
    hidden_dimensions::{
        hidden_dim_count, holographic_accuracy, lift_to_hidden, precision_from_hidden_dims,
        project_to_visible, HiddenDimensionConfig,
    },
    PythagoreanManifold,
};

fn main() {
    println!("=== Hidden Dimensions Encoding Example ===\n");

    // Example 1: Understanding the hidden dimension formula
    println!("--- Example 1: Hidden Dimension Formula ---");
    println!("The formula: k = ⌈log₂(1/ε)⌉\n");

    let precisions = [0.1, 0.01, 0.001, 1e-6, 1e-10, 1e-15];

    for epsilon in precisions {
        let k = hidden_dim_count(epsilon);
        let computed_epsilon = precision_from_hidden_dims(k);

        println!(
            "ε = {:8.0e}: k = {:2} hidden dims, computed ε = {:.2e}",
            epsilon, k, computed_epsilon
        );
    }
    println!();

    // Example 2: Lifting points to hidden dimensions
    println!("--- Example 2: Lifting Points ---");
    let point = vec![0.6, 0.8]; // 3-4-5 triangle
    let epsilon = 1e-6;
    let k = hidden_dim_count(epsilon);

    println!("Original point: {:?}", point);
    println!("Precision ε = {}", epsilon);
    println!("Hidden dimensions k = {}", k);

    let lifted = lift_to_hidden(&point, k);
    println!("Lifted to {} dimensions: {:?}", lifted.len(), &lifted[..5.min(lifted.len())]);
    println!("... (showing first 5 components)\n");

    // Example 3: Projecting back to visible dimensions
    println!("--- Example 3: Projecting Back ---");
    let projected = project_to_visible(&lifted, point.len());
    println!("Projected back: {:?}", projected);
    println!("Original:       {:?}", point);
    println!(
        "Difference: [{:.6}, {:.6}]\n",
        (projected[0] - point[0]).abs(),
        (projected[1] - point[1]).abs()
    );

    // Example 4: Using HiddenDimensionConfig
    println!("--- Example 4: HiddenDimensionConfig ---");
    let config = HiddenDimensionConfig::new(1e-10);
    println!("Config epsilon: {}", config.epsilon);
    println!("Config hidden_dims: {}", config.hidden_dims);

    let encoded = config.encode(&[0.707, 0.707]);
    println!("Encoded [0.707, 0.707]: {:?}", encoded);
    println!();

    // Example 5: Holographic accuracy
    println!("--- Example 5: Holographic Accuracy ---");
    println!("Formula: accuracy(k,n) = k/n + O(1/log n)\n");

    let configs = [
        (4, 6),   // Few hidden dims
        (8, 10),  // Medium
        (16, 18), // Many hidden dims
    ];

    for (k, n) in configs {
        let accuracy = holographic_accuracy(k, n);
        println!("k={}, n={}: accuracy = {:.4} ({:.1}%)", k, n, accuracy, accuracy * 100.0);
    }
    println!();

    // Example 6: Precision requirements for applications
    println!("--- Example 6: Precision Requirements ---");
    let applications = [
        ("Animation", 1e-2),
        ("Game physics", 1e-4),
        ("Robotics", 1e-6),
        ("ML inference", 1e-8),
        ("Consensus systems", 1e-10),
        ("Scientific computing", 1e-15),
    ];

    for (name, epsilon) in applications {
        let k = hidden_dim_count(epsilon);
        let accuracy = holographic_accuracy(k, k + 2);
        println!("{:20}: ε = {:.0e}, k = {:2}, accuracy = {:.2}%", 
                 name, epsilon, k, accuracy * 100.0);
    }
    println!();

    // Example 7: Integration with manifold snapping
    println!("--- Example 7: Integration with Manifold ---");
    let manifold = PythagoreanManifold::new(200);
    let config = HiddenDimensionConfig::new(1e-6);

    let test_points = vec![
        vec![0.577, 0.816], // Close to sqrt(1/3), sqrt(2/3)
        vec![0.707, 0.707], // sqrt(2)/2, sqrt(2)/2
        vec![0.6, 0.8],     // Exact 3-4-5
    ];

    for point in test_points {
        // Encode with hidden dimensions
        let encoded = config.encode(&point);

        // Snap via manifold
        let (snapped, noise) = manifold.snap([encoded[0] as f32, encoded[1] as f32]);

        println!("Original: {:?} -> Snapped: {:?} (noise: {:.4})", 
                 point, snapped, noise);
    }
    println!();

    // Example 8: Trade-offs
    println!("--- Example 8: Trade-offs ---");
    println!("Higher precision = more hidden dimensions = more computation");
    println!("Lower precision = fewer dimensions = faster but less accurate\n");

    for bits in [8, 16, 32, 64] {
        let epsilon = 2.0_f64.powi(-(bits as i32));
        let k = hidden_dim_count(epsilon);
        println!("{:2} bits: ε = {:.2e}, k = {:2} hidden dims", bits, epsilon, k);
    }

    println!("\n=== Hidden Dimensions Example Complete ===");
}
