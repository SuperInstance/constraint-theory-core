//! Basic usage example for constraint-theory-core
//!
//! Demonstrates the fundamental operations:
//! - Creating a Pythagorean manifold
//! - Snapping vectors to discrete Pythagorean coordinates
//! - Understanding noise/resonance metrics

use constraint_theory_core::{PythagoreanManifold, snap};

fn main() {
    println!("=== Constraint Theory Core - Basic Example ===\n");

    // Create a Pythagorean manifold with density 200
    // This generates ~1000 valid Pythagorean states
    let density = 200;
    let manifold = PythagoreanManifold::new(density);
    
    println!("Created manifold with density {}", density);
    println!("Total valid states: {}\n", manifold.state_count());

    // Example 1: Snap a known Pythagorean triple (3-4-5)
    println!("--- Example 1: Exact Pythagorean Triple ---");
    let vector = [0.6_f32, 0.8_f32]; // 3/5, 4/5
    let (snapped, noise) = manifold.snap(vector);
    
    println!("Input vector: {:?}", vector);
    println!("Snapped to:   {:?}", snapped);
    println!("Noise:        {:.6}", noise);
    println!("Expected noise near 0.0 for exact triples\n");

    // Example 2: Snap a non-Pythagorean vector
    println!("--- Example 2: Non-Pythagorean Vector ---");
    let vector = [0.707_f32, 0.707_f32]; // ~45 degrees, not Pythagorean
    let (snapped, noise) = manifold.snap(vector);
    
    println!("Input vector: {:?}", vector);
    println!("Snapped to:   {:?}", snapped);
    println!("Noise:        {:.6}", noise);
    println!("Higher noise indicates distance from nearest Pythagorean state\n");

    // Example 3: Using the convenience function
    println!("--- Example 3: Using snap() function ---");
    let vector = [0.8_f32, 0.6_f32]; // 4/5, 3/5 (reversed)
    let (snapped, noise) = snap(&manifold, vector);
    
    println!("Input vector: {:?}", vector);
    println!("Snapped to:   {:?}", snapped);
    println!("Noise:        {:.6}\n", noise);

    // Example 4: Snap vectors at various angles
    println!("--- Example 4: Various Angles ---");
    let test_vectors = [
        [1.0, 0.0],   // 0 degrees
        [0.0, 1.0],   // 90 degrees
        [-1.0, 0.0],  // 180 degrees
        [0.0, -1.0],  // 270 degrees
        [0.5, 0.866], // ~60 degrees
        [0.866, 0.5], // ~30 degrees
    ];

    for vector in test_vectors {
        let (snapped, noise) = manifold.snap(vector);
        println!("  {:?} -> {:?} (noise: {:.4})", vector, snapped, noise);
    }
    println!();

    // Example 5: Understanding noise
    println!("--- Example 5: Understanding Noise ---");
    println!("Noise represents how far the input is from a valid Pythagorean state.");
    println!("- noise = 0.0: Exact Pythagorean triple (perfect resonance)");
    println!("- noise < 0.01: Very close to a valid state");
    println!("- noise > 0.1: Significant deviation from nearest state\n");

    // Demonstrate noise calculation
    let exact_triple = [0.6, 0.8];
    let (_, noise_exact) = manifold.snap(exact_triple);
    
    let approximate = [0.61, 0.79];
    let (_, noise_approx) = manifold.snap(approximate);
    
    println!("Exact triple {:?}: noise = {:.6}", exact_triple, noise_exact);
    println!("Approximate {:?}: noise = {:.6}", approximate, noise_approx);

    // Example 6: Using different manifold densities
    println!("\n--- Example 6: Density Comparison ---");
    let test_vec = [0.707, 0.707];
    
    for density in [50, 100, 200, 500] {
        let m = PythagoreanManifold::new(density);
        let (_, noise) = m.snap(test_vec);
        println!("Density {:3}: {:4} states, noise = {:.6}", 
                 density, m.state_count(), noise);
    }

    println!("\n=== Basic Example Complete ===");
}
