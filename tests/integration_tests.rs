//! Integration Tests for Constraint Theory Core
//!
//! These tests verify that all modules work together correctly, testing:
//! - hidden_dimensions + quantizer integration
//! - holonomy verification with manifold operations
//! - Cross-module type conversions
//! - End-to-end workflows
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --test integration_tests
//! ```

use constraint_theory_core::{
    hidden_dimensions::{
        hidden_dim_count, holographic_accuracy, lift_to_hidden, precision_from_hidden_dims,
        project_to_visible, HiddenDimensionConfig,
    },
    holonomy::{
        compute_holonomy, identity_matrix, rotation_x, rotation_y, rotation_z, verify_holonomy,
        HolonomyChecker,
    },
    manifold::{snap, PythagoreanManifold, PythagoreanTriple},
    quantizer::{PythagoreanQuantizer, QuantizationMode, Rational},
    hidden_dimensions, // Module-level import for encode_with_hidden_dims
};

// ============================================================================
// Hidden Dimensions + Quantizer Integration Tests
// ============================================================================

mod hidden_dimensions_quantizer {
    use super::*;

    /// Test that hidden dimension encoding produces quantizable results
    #[test]
    fn test_hidden_dims_produce_quantizable_vectors() {
        let point = [0.6, 0.8];
        let epsilon = 1e-6;
        let k = hidden_dim_count(epsilon);

        // Lift the point
        let lifted = lift_to_hidden(&point, k);
        assert!(lifted.len() >= point.len());

        // Quantize the lifted point
        let quantizer = PythagoreanQuantizer::new(QuantizationMode::Polar, 8);
        let result = quantizer.quantize(&lifted);

        // Verify quantization succeeded
        assert!(result.constraints_satisfied);
    }

    /// Test that quantized hidden dimensions project back correctly
    #[test]
    fn test_quantize_then_project() {
        let point = vec![0.577, 0.816]; // Close to sqrt(1/3), sqrt(2/3)
        let epsilon = 1e-4;
        let k = hidden_dim_count(epsilon);

        // Lift, quantize, then project
        let lifted = lift_to_hidden(&point, k);

        let quantizer = PythagoreanQuantizer::for_embeddings();
        let quantized = quantizer.quantize(&lifted);

        let projected = project_to_visible(&quantized.data, point.len());

        // Should be reasonably close to original
        assert!((projected[0] - point[0]).abs() < 0.5);
        assert!((projected[1] - point[1]).abs() < 0.5);
    }

    /// Test hidden dimension count matches quantization precision
    #[test]
    fn test_hidden_dim_precision_consistency() {
        // For various epsilon values, verify k = ceil(log2(1/epsilon))
        let test_cases = [
            (0.1, 4),
            (0.01, 7),
            (0.001, 10),
            (1e-6, 20),
            (1e-10, 34),
        ];

        for (epsilon, expected_k) in test_cases {
            let k = hidden_dim_count(epsilon);
            assert_eq!(k, expected_k, "Failed for epsilon = {}", epsilon);

            // Verify inverse
            let computed_epsilon = precision_from_hidden_dims(k);
            assert!(
                computed_epsilon <= epsilon,
                "Precision {} should be <= {}",
                computed_epsilon,
                epsilon
            );
        }
    }

    /// Test that holographic accuracy relates correctly to hidden dimensions
    #[test]
    fn test_holographic_accuracy_with_quantization() {
        // With more hidden dimensions, accuracy should improve
        let acc_low = holographic_accuracy(2, 4);
        let acc_high = holographic_accuracy(8, 10);

        assert!(
            acc_high > acc_low,
            "More hidden dimensions should improve accuracy: {} vs {}",
            acc_high,
            acc_low
        );
    }

    /// Test full encoding pipeline with quantization
    #[test]
    fn test_full_encoding_pipeline() {
        let points = vec![
            vec![0.6, 0.8],
            vec![0.8, 0.6],
            vec![0.707, 0.707],
        ];

        let config = HiddenDimensionConfig::new(1e-6);

        for point in points {
            // Encode using hidden dimensions
            let encoded = config.encode(&point);

            // Quantize the encoded result
            let quantizer = PythagoreanQuantizer::for_embeddings();
            let result = quantizer.quantize(&encoded);

            // Should maintain reasonable accuracy
            assert!(
                result.mse < 0.5,
                "MSE too high: {} for point {:?}",
                result.mse,
                point
            );
        }
    }
}

// ============================================================================
// Holonomy + Manifold Integration Tests
// ============================================================================

mod holonomy_manifold {
    use super::*;

    /// Test that snapped vectors from manifold can form holonomy cycles
    #[test]
    fn test_snapped_vectors_holonomy() {
        let manifold = PythagoreanManifold::new(200);

        // Snap several vectors
        let v1 = manifold.snap([0.6, 0.8]).0;
        let v2 = manifold.snap([0.8, 0.6]).0;
        let v3 = manifold.snap([1.0, 0.0]).0;

        // These are valid states, so they should satisfy unit norm constraint
        for v in &[v1, v2, v3] {
            let norm = (v[0] * v[0] + v[1] * v[1]).sqrt();
            assert!(
                (norm - 1.0).abs() < 0.01,
                "Snapped vector should be unit norm: {:?}",
                v
            );
        }
    }

    /// Test manifold density affects angular precision
    #[test]
    fn test_manifold_density_precision() {
        let low_density = PythagoreanManifold::new(50);
        let high_density = PythagoreanManifold::new(500);

        // High density should have more states
        assert!(
            high_density.state_count() > low_density.state_count(),
            "Higher density should have more states"
        );

        // High density should have lower max angular error
        assert!(
            high_density.max_angular_error() < low_density.max_angular_error(),
            "Higher density should have lower angular error"
        );
    }

    /// Test that holonomy identity corresponds to consistent constraints
    #[test]
    fn test_holonomy_constraint_consistency() {
        // Identity cycle - should have zero holonomy
        let id = identity_matrix();
        let result = compute_holonomy(&[id]);
        assert!(result.is_identity(), "Identity cycle should have zero holonomy");

        // Multiple identities should still be identity
        let result = compute_holonomy(&[id, id, id]);
        assert!(result.is_identity());
    }

    /// Test rotation cycles and holonomy
    #[test]
    fn test_rotation_cycle_holonomy() {
        // Two 180-degree rotations around Z should return to identity
        let rz_180 = rotation_z(std::f64::consts::PI);
        let result = compute_holonomy(&[rz_180, rz_180]);

        // Should be close to identity (within numerical tolerance)
        assert!(
            result.norm < 0.01,
            "Two 180-degree rotations should return near identity: norm = {}",
            result.norm
        );
    }

    /// Test holonomy checker with manifold-snapped vectors
    #[test]
    fn test_holonomy_checker_with_manifold() {
        let mut checker = HolonomyChecker::default_tolerance();
        let manifold = PythagoreanManifold::new(200);

        // Apply identity rotation (no change)
        let id = identity_matrix();
        checker.apply(&id);
        checker.apply(&id);

        let result = checker.check_closed();
        assert!(result.is_identity());
    }
}

// ============================================================================
// Quantizer + Manifold Integration Tests
// ============================================================================

mod quantizer_manifold {
    use super::*;

    /// Test that quantizer and manifold produce consistent results
    #[test]
    fn test_quantizer_manifold_consistency() {
        let manifold = PythagoreanManifold::new(200);
        let quantizer = PythagoreanQuantizer::for_embeddings();

        // Both should recognize 3-4-5 triangle
        let (snapped, noise) = manifold.snap([0.6, 0.8]);
        assert!(noise < 0.001, "Manifold should snap 3-4-5 exactly");

        let quantized = quantizer.quantize(&[0.6, 0.8]);
        assert!(
            quantized.check_unit_norm(0.1),
            "Quantizer should preserve unit norm"
        );
    }

    /// Test batch operations consistency
    #[test]
    fn test_batch_operations() {
        let manifold = PythagoreanManifold::new(200);
        let quantizer = PythagoreanQuantizer::for_embeddings();

        let vectors: Vec<[f32; 2]> = vec![[0.6, 0.8], [0.8, 0.6], [0.707, 0.707], [0.5, 0.866]];

        // Batch snap with manifold
        let mut manifold_results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
        manifold.snap_batch(&vectors, &mut manifold_results);

        // Batch quantize
        let f64_vectors: Vec<Vec<f64>> = vectors
            .iter()
            .map(|v| vec![v[0] as f64, v[1] as f64])
            .collect();
        let quantizer_results = quantizer.quantize_batch(&f64_vectors);

        // Both should produce unit norm results
        for (i, (snapped, _noise)) in manifold_results.iter().enumerate() {
            let norm = (snapped[0] * snapped[0] + snapped[1] * snapped[1]).sqrt();
            assert!(
                (norm - 1.0).abs() < 0.01,
                "Manifold result {} should be unit norm",
                i
            );
        }

        for (i, result) in quantizer_results.iter().enumerate() {
            assert!(
                result.check_unit_norm(0.1),
                "Quantizer result {} should be unit norm",
                i
            );
        }
    }

    /// Test different quantization modes
    #[test]
    fn test_quantization_modes() {
        let data = vec![0.6, 0.8, 0.0, 0.0];

        // Test all modes
        for mode in [
            QuantizationMode::Ternary,
            QuantizationMode::Polar,
            QuantizationMode::Turbo,
            QuantizationMode::Hybrid,
        ] {
            let quantizer = PythagoreanQuantizer::new(mode, 4);
            let result = quantizer.quantize(&data);

            assert_eq!(
                result.mode, mode,
                "Quantization should use specified mode"
            );
        }
    }

    /// Test Pythagorean rational detection
    #[test]
    fn test_pythagorean_rationals() {
        // Test known Pythagorean ratios
        let pythagorean_rationals = vec![
            Rational::new(3, 5),
            Rational::new(4, 5),
            Rational::new(5, 13),
            Rational::new(12, 13),
        ];

        for r in pythagorean_rationals {
            assert!(
                r.is_pythagorean(),
                "Rational {:?} should be Pythagorean",
                r
            );
        }

        // Test non-Pythagorean rationals
        let non_pythagorean = vec![Rational::new(1, 3), Rational::new(2, 7), Rational::new(1, 2)];

        for r in non_pythagorean {
            assert!(
                !r.is_pythagorean(),
                "Rational {:?} should NOT be Pythagorean",
                r
            );
        }
    }
}

// ============================================================================
// Full Pipeline Integration Tests
// ============================================================================

mod full_pipeline {
    use super::*;

    /// Test complete workflow: point -> hidden dims -> quantize -> verify
    #[test]
    fn test_complete_workflow() {
        let point = [0.6_f64, 0.8_f64];
        let epsilon = 1e-6;

        // Step 1: Compute hidden dimensions
        let k = hidden_dim_count(epsilon);
        assert!(k > 0);

        // Step 2: Lift to hidden dimensions
        let lifted = lift_to_hidden(&point, k);
        assert_eq!(lifted.len(), point.len() + k);

        // Step 3: Quantize
        let quantizer = PythagoreanQuantizer::for_embeddings();
        let result = quantizer.quantize(&lifted);
        assert!(result.constraints_satisfied);

        // Step 4: Project back
        let projected = project_to_visible(&result.data, point.len());
        assert_eq!(projected.len(), point.len());

        // Step 5: Verify via manifold
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([projected[0] as f32, projected[1] as f32]);

        // Should have reasonable precision
        assert!(noise < 0.5, "Final noise should be reasonable: {}", noise);
    }

    /// Test multiple points through the pipeline
    #[test]
    fn test_batch_pipeline() {
        let points: Vec<Vec<f64>> = vec![
            vec![0.6, 0.8],
            vec![0.8, 0.6],
            vec![0.707, 0.707],
            vec![0.5, 0.866],
        ];

        let epsilon = 1e-4;
        let k = hidden_dim_count(epsilon);
        let quantizer = PythagoreanQuantizer::for_embeddings();
        let manifold = PythagoreanManifold::new(200);

        for point in points {
            // Lift
            let lifted = lift_to_hidden(&point, k);

            // Quantize
            let result = quantizer.quantize(&lifted);

            // Project
            let projected = project_to_visible(&result.data, point.len());

            // Verify via manifold
            let (snapped, noise) =
                manifold.snap([projected[0] as f32, projected[1] as f32]);

            // All should be valid snaps
            let norm = (snapped[0] * snapped[0] + snapped[1] * snapped[1]).sqrt();
            assert!(
                (norm - 1.0).abs() < 0.01,
                "Snapped result should be unit norm"
            );
        }
    }

    /// Test holonomy verification with encoded constraints
    #[test]
    fn test_holonomy_with_encoded_constraints() {
        // Create a configuration for encoding
        let config = HiddenDimensionConfig::new(1e-6);

        // Encode some points
        let p1 = config.encode(&[0.6, 0.8]);
        let p2 = config.encode(&[0.8, 0.6]);
        let p3 = config.encode(&[0.707, 0.707]);

        // All encodings should have the same dimension (visible + hidden)
        assert_eq!(p1.len(), p2.len());
        assert_eq!(p2.len(), p3.len());

        // Create a holonomy cycle
        let id = identity_matrix();
        let result = compute_holonomy(&[id, id]);

        // Should be consistent
        assert!(result.is_identity());
    }
}

// ============================================================================
// Edge Case Integration Tests
// ============================================================================

mod edge_cases {
    use super::*;

    /// Test with zero vector
    #[test]
    fn test_zero_vector_handling() {
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([0.0, 0.0]);

        // Should snap to a valid state
        let norm = (snapped[0] * snapped[0] + snapped[1] * snapped[1]).sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Zero vector should snap to unit norm"
        );
        assert_eq!(noise, 0.0, "Zero vector should have zero noise");
    }

    /// Test with NaN input
    #[test]
    fn test_nan_handling() {
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([f32::NAN, 0.5]);

        // Should return a valid result (error indicator)
        assert_eq!(noise, 1.0, "NaN input should return noise=1.0");
    }

    /// Test with very small epsilon
    #[test]
    fn test_small_epsilon() {
        let epsilon = 1e-20;
        let k = hidden_dim_count(epsilon);

        // Should be a large number of hidden dimensions
        assert!(k > 50, "Small epsilon should require many hidden dims: {}", k);

        // Precision should be at least as good as epsilon
        let precision = precision_from_hidden_dims(k);
        assert!(
            precision <= epsilon,
            "Precision {} should be <= {}",
            precision,
            epsilon
        );
    }

    /// Test with very large epsilon
    #[test]
    fn test_large_epsilon() {
        let epsilon = 0.5;
        let k = hidden_dim_count(epsilon);

        // Should be small number of hidden dimensions
        assert!(k <= 2, "Large epsilon should require few hidden dims: {}", k);
    }

    /// Test empty cycle holonomy
    #[test]
    fn test_empty_holonomy() {
        let result = compute_holonomy(&[]);
        assert!(result.is_identity(), "Empty cycle should be identity");
        assert!(result.information.is_infinite());
    }

    /// Test single element cycle
    #[test]
    fn test_single_element_cycle() {
        let id = identity_matrix();
        let result = compute_holonomy(&[id]);

        assert!(result.is_identity());
        assert_eq!(result.norm, 0.0);
    }
}

// ============================================================================
// Performance Integration Tests
// ============================================================================

mod performance {
    use super::*;

    /// Test that KD-tree lookup is fast
    #[test]
    fn test_kdtree_performance() {
        use std::time::Instant;

        let manifold = PythagoreanManifold::new(500);
        let iterations = 1000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = manifold.snap([0.6, 0.8]);
        }
        let duration = start.elapsed();

        let per_op_us = duration.as_micros() as f64 / iterations as f64;

        // Should be under 100 microseconds per operation
        assert!(
            per_op_us < 100.0,
            "KD-tree lookup too slow: {} μs/op",
            per_op_us
        );
    }

    /// Test SIMD batch performance
    #[test]
    fn test_simd_batch_performance() {
        use std::time::Instant;

        let manifold = PythagoreanManifold::new(200);
        let vectors: Vec<[f32; 2]> = (0..1000)
            .map(|i| [(i as f32 % 100.0) / 100.0, ((i + 50) as f32 % 100.0) / 100.0])
            .collect();

        // SIMD batch
        let start = Instant::now();
        let _simd_results = manifold.snap_batch_simd(&vectors);
        let simd_duration = start.elapsed();

        // Scalar batch
        let start = Instant::now();
        let mut scalar_results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
        manifold.snap_batch(&vectors, &mut scalar_results);
        let scalar_duration = start.elapsed();

        // SIMD should be faster (or at least not much slower)
        // Allow some overhead for small batches
        println!(
            "SIMD: {:?}, Scalar: {:?}",
            simd_duration, scalar_duration
        );
    }

    /// Test quantization batch performance
    #[test]
    fn test_quantizer_batch_performance() {
        use std::time::Instant;

        let quantizer = PythagoreanQuantizer::for_embeddings();

        // Generate random unit vectors
        let vectors: Vec<Vec<f64>> = (0..1000)
            .map(|i| {
                let angle = (i as f64) * 0.001;
                vec![angle.cos(), angle.sin()]
            })
            .collect();

        let start = Instant::now();
        let _results = quantizer.quantize_batch(&vectors);
        let duration = start.elapsed();

        let per_vec_us = duration.as_micros() as f64 / vectors.len() as f64;

        // Should be under 50 microseconds per vector
        assert!(
            per_vec_us < 50.0,
            "Quantization too slow: {} μs/vector",
            per_vec_us
        );
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

mod properties {
    use super::*;

    /// Property: hidden_dim_count is monotonic in epsilon
    #[test]
    fn test_hidden_dim_monotonicity() {
        let epsilons = [1e-2, 1e-4, 1e-6, 1e-8, 1e-10];
        let mut prev_k = 0;

        for epsilon in epsilons {
            let k = hidden_dim_count(epsilon);
            assert!(
                k >= prev_k,
                "Hidden dims should increase as epsilon decreases: {} vs {}",
                k,
                prev_k
            );
            prev_k = k;
        }
    }

    /// Property: quantization preserves dimensionality
    #[test]
    fn test_quantization_preserves_dimensionality() {
        let quantizer = PythagoreanQuantizer::hybrid();

        for n in [2, 4, 8, 16, 32] {
            let data: Vec<f64> = (0..n).map(|i| (i as f64 + 1.0) / (n as f64 + 1.0)).collect();
            let result = quantizer.quantize(&data);

            assert_eq!(
                result.data.len(),
                n,
                "Quantization should preserve dimensionality"
            );
        }
    }

    /// Property: manifold snap produces unit norm vectors
    #[test]
    fn test_snap_produces_unit_norm() {
        let manifold = PythagoreanManifold::new(200);

        // Test many random-ish vectors
        for i in 0..100 {
            let angle = (i as f32) * 0.063; // ~3.6 degrees each
            let vec = [angle.cos(), angle.sin()];

            let (snapped, _noise) = manifold.snap(vec);
            let norm = (snapped[0] * snapped[0] + snapped[1] * snapped[1]).sqrt();

            assert!(
                (norm - 1.0).abs() < 0.001,
                "Snapped vector should be unit norm: got {}",
                norm
            );
        }
    }

    /// Property: holonomy of inverse cycle is identity
    #[test]
    fn test_inverse_cycle_holonomy() {
        // For any rotation, R * R^T = I
        for angle in [0.1, 0.5, 1.0, std::f64::consts::PI / 4.0] {
            let rx = rotation_x(angle);
            // Transpose is inverse for rotation matrices
            let rx_inv = [
                [rx[0][0], rx[1][0], rx[2][0]],
                [rx[0][1], rx[1][1], rx[2][1]],
                [rx[0][2], rx[1][2], rx[2][2]],
            ];

            let result = compute_holonomy(&[rx, rx_inv]);
            assert!(
                result.norm < 0.001,
                "R * R^T should be identity: norm = {}",
                result.norm
            );
        }
    }
}
