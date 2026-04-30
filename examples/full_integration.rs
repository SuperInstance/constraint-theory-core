//! Full Integration Example
//!
//! Demonstrates how all constraint-theory-core modules work together:
//!
//! - PythagoreanManifold for discrete snapping
//! - HiddenDimensions for exact encoding
//! - PythagoreanQuantizer for constraint-preserving quantization
//! - Holonomy verification for consistency checks
//! - LatticeCache for performance optimization

use constraint_theory_core::{
    cache::LatticeCache,
    hidden_dimensions::{hidden_dim_count, lift_to_hidden, project_to_visible, HiddenDimensionConfig},
    holonomy::{compute_holonomy, identity_matrix, verify_holonomy, HolonomyChecker},
    manifold::PythagoreanManifold,
    quantizer::{PythagoreanQuantizer, QuantizationMode, Rational},
};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     Constraint Theory Core - Full Integration Example       ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // =========================================================================
    // Part 1: Core Snapping with PythagoreanManifold
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Part 1: Pythagorean Manifold Snapping                        │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    let manifold = PythagoreanManifold::new(200);
    println!("Created manifold with {} valid states", manifold.state_count());
    println!("Max angular error: {:.4}°\n", manifold.max_angular_error().to_degrees());

    let test_vectors = [
        ([0.6, 0.8], "3-4-5 triangle"),
        ([0.707, 0.707], "45° angle"),
        ([0.5, 0.866], "60° angle"),
    ];

    for (vec, name) in test_vectors {
        let (snapped, noise) = manifold.snap(vec);
        println!("{:15}: {:?} -> {:?} (noise: {:.4})", name, vec, snapped, noise);
    }
    println!();

    // =========================================================================
    // Part 2: Hidden Dimensions Encoding
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Part 2: Hidden Dimensions Encoding                          │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    let config = HiddenDimensionConfig::new(1e-10);
    println!("Configuration:");
    println!("  Target precision: {}", config.epsilon);
    println!("  Hidden dimensions: {}", config.hidden_dims);

    let point = vec![0.577, 0.816];
    let encoded = config.encode(&point);

    println!("\nEncoding point {:?}", point);
    println!("  Encoded to {} dimensions", encoded.len());
    println!("  First 4 components: {:?}", &encoded[..4]);

    let k = hidden_dim_count(1e-6);
    let lifted = lift_to_hidden(&point, k);
    let projected = project_to_visible(&lifted, 2);

    println!("\nLift -> Project cycle:");
    println!("  Original: {:?}", point);
    println!("  Projected: {:?}", projected);
    println!();

    // =========================================================================
    // Part 3: Quantization with Constraint Preservation
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Part 3: Quantization with Constraint Preservation           │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    // Polar mode for unit norm preservation
    let quantizer = PythagoreanQuantizer::for_embeddings();
    let embedding = vec![0.6, 0.8, 0.0, 0.0];
    let result = quantizer.quantize(&embedding);

    println!("Polar Quantization (unit norm preservation):");
    println!("  Input: {:?}", embedding);
    println!("  Output: {:?}", result.data);
    println!("  MSE: {:.6}", result.mse);
    println!("  Unit norm preserved: {}\n", result.unit_norm_preserved);

    // Ternary mode for LLM weights
    let llm_quantizer = PythagoreanQuantizer::for_llm();
    let weights = vec![-0.8, -0.1, 0.1, 0.9, 0.5, -0.3, 0.05, -0.95];
    let llm_result = llm_quantizer.quantize(&weights);

    println!("Ternary Quantization (LLM weights):");
    println!("  Input: {:?}", weights);
    println!("  Output: {:?}", llm_result.data);
    println!("  Memory reduction: 16x (FP32 -> Ternary)\n");

    // =========================================================================
    // Part 4: Holonomy Verification
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Part 4: Holonomy Verification                               │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    let id = identity_matrix();
    let result = compute_holonomy(&[id, id, id]);

    println!("Holonomy of identity cycle:");
    println!("  Norm: {:.6}", result.norm);
    println!("  Is identity: {}", result.is_identity());
    println!("  Information content: {:.2}\n", result.information);

    let mut checker = HolonomyChecker::new(1e-6);
    checker.apply(&id);
    checker.apply(&id);
    let check_result = checker.check_closed();

    println!("Incremental holonomy check:");
    println!("  Steps applied: {}", checker.step_count());
    println!("  Closed cycle norm: {:.6}\n", check_result.norm);

    // =========================================================================
    // Part 5: Lattice Cache for Performance
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Part 5: Lattice Cache for Performance                       │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    let cache = LatticeCache::new(10);

    // Precompute common densities
    cache.precompute(&[50, 100, 200]);
    println!("Precomputed {} lattice densities", cache.len());

    let lattice = cache.get_or_compute(200);
    println!("\nCached lattice (density=200):");
    println!("  Points: {}", lattice.len());
    println!("  Max hypotenuse: {}", lattice.max_hypotenuse);

    let (nearest, idx, dist_sq) = lattice.nearest([0.6, 0.8]);
    println!("  Nearest to [0.6, 0.8]: {:?} (index: {}, dist²: {:.6})\n", 
             nearest, idx, dist_sq);

    // =========================================================================
    // Part 6: Full Pipeline Integration
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Part 6: Full Pipeline Integration                           │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    // Input data
    let input_vectors: Vec<Vec<f64>> = vec![
        vec![0.6, 0.8],
        vec![0.707, 0.707],
        vec![0.5, 0.866],
    ];

    println!("Processing {} vectors through full pipeline:\n", input_vectors.len());

    for (i, vector) in input_vectors.iter().enumerate() {
        println!("Vector {}: {:?}", i, vector);

        // Step 1: Lift to hidden dimensions
        let k = hidden_dim_count(1e-6);
        let lifted = lift_to_hidden(vector, k);

        // Step 2: Quantize with constraint preservation
        let quantizer = PythagoreanQuantizer::for_embeddings();
        let quantized = quantizer.quantize(&lifted);

        // Step 3: Project back to visible space
        let projected = project_to_visible(&quantized.data, vector.len());

        // Step 4: Verify via manifold snapping
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([projected[0] as f32, projected[1] as f32]);

        println!("  Lifted to {} dimensions", lifted.len());
        println!("  Quantized (MSE: {:.6})", quantized.mse);
        println!("  Projected: {:?}", projected);
        println!("  Snapped: {:?} (noise: {:.4})", snapped, noise);
        println!("  Unit norm preserved: {}", quantized.unit_norm_preserved);
        println!();
    }

    // =========================================================================
    // Part 7: Cross-Module Consistency Verification
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Part 7: Cross-Module Consistency Verification               │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    // Verify that different modules agree on Pythagorean coordinates
    let test_point = [0.6_f64, 0.8_f64];

    // Via quantizer
    let quantizer = PythagoreanQuantizer::new(QuantizationMode::Polar, 8);
    let (q_snapped, num, den) = quantizer.snap_to_lattice(test_point[0], 100);
    let rational = Rational::new(num, den);

    println!("Point [0.6, 0.8]:");
    println!("  Quantizer snapped: {}/{} = {:.4}", num, den, q_snapped);
    println!("  Is Pythagorean: {}", rational.is_pythagorean());

    // Via manifold
    let manifold = PythagoreanManifold::new(200);
    let (m_snapped, m_noise) = manifold.snap([0.6, 0.8]);
    println!("  Manifold snapped: {:?} (noise: {:.4})", m_snapped, m_noise);

    // Both should agree
    let agreement = (q_snapped - m_snapped[0] as f64).abs() < 0.01;
    println!("  Modules agree: {}\n", agreement);

    // =========================================================================
    // Summary
    // =========================================================================
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│ Summary                                                     │");
    println!("╰─────────────────────────────────────────────────────────────╯\n");

    println!("✓ PythagoreanManifold: O(log N) snapping via KD-tree");
    println!("✓ HiddenDimensions: Exact encoding via k = ⌈log₂(1/ε)⌉");
    println!("✓ PythagoreanQuantizer: Constraint-preserving quantization");
    println!("✓ Holonomy: Consistency verification around cycles");
    println!("✓ LatticeCache: Thread-safe caching for performance");
    println!();

    // Performance characteristics
    println!("Performance targets:");
    println!("  • Single snap: < 1 μs");
    println!("  • Batch (SIMD): < 100 ns/vector");
    println!("  • Holonomy check: O(n²) per cycle");
    println!("  • Cache lookup: O(1)");

    println!("\n=== Full Integration Example Complete ===");
}
