//! Cross-Repository Integration Tests
//!
//! These tests verify compatibility between constraint-theory-core and other repos:
//! - constraint-theory-python
//! - constraint-theory-web (WASM)
//! - constraint-flow
//! - constraint-ranch

use constraint_theory_core::{
    hidden_dim_count, holographic_accuracy, lift_to_hidden, project_visible,
    PythagoreanManifold, PythagoreanQuantizer, QuantizationMode, snap,
    compute_holonomy, verify_holonomy,
};

/// Test that hidden dimension formula matches expected values
/// This must match Python: compute_hidden_dim_count()
/// This must match TypeScript: hiddenDimCount()
#[test]
fn test_hidden_dim_formula_cross_repo() {
    // Test cases: (epsilon, expected_k)
    let test_cases = vec![
        (0.1, 4),
        (0.01, 7),
        (0.001, 10),
        (0.0001, 14),
        (1e-6, 20),
        (1e-10, 34),
    ];
    
    for (epsilon, expected_k) in test_cases {
        let k = hidden_dim_count(epsilon);
        assert_eq!(
            k, expected_k,
            "hidden_dim_count({}) = {}, expected {}",
            epsilon, k, expected_k
        );
    }
}

/// Test that holographic accuracy formula matches expected values
/// This must match Python: holographic_accuracy()
/// This must match TypeScript: holographicAccuracy()
#[test]
fn test_holographic_accuracy_cross_repo() {
    // Test cases: (k, n, expected range)
    let test_cases = vec![
        (1, 10, 0.1, 0.2),      // k/n = 0.1
        (5, 10, 0.5, 0.6),      // k/n = 0.5
        (10, 10, 0.9, 1.1),     // k/n = 1.0
        (34, 100, 0.33, 0.38),  // k/n ≈ 0.34
    ];
    
    for (k, n, min_acc, max_acc) in test_cases {
        let acc = holographic_accuracy(k, n);
        assert!(
            acc >= min_acc && acc <= max_acc,
            "holographic_accuracy({}, {}) = {}, expected [{}, {}]",
            k, n, acc, min_acc, max_acc
        );
    }
}

/// Test lift/project round-trip
/// This must match Python: lift_to_hidden() / project_visible()
/// This must match TypeScript: liftToHidden() / projectVisible()
#[test]
fn test_lift_project_round_trip() {
    let test_points = vec![
        vec![0.6, 0.8],
        vec![0.577, 0.816],
        vec![0.1, 0.995],
        vec![1.0, 0.0],
        vec![0.0, 1.0],
    ];
    
    for point in test_points {
        for k in [1, 5, 10, 34].iter() {
            let lifted = lift_to_hidden(&point, *k);
            assert_eq!(lifted.len(), point.len() + k);
            
            let projected = project_visible(&lifted, point.len());
            assert_eq!(projected.len(), point.len());
            
            // Verify projection recovers original point (approximately)
            for (p, o) in projected.iter().zip(point.iter()) {
                assert!(
                    (p - o).abs() < 1e-10,
                    "Projection mismatch: {} vs {}",
                    p, o
                );
            }
        }
    }
}

/// Test snap produces deterministic results
/// This must match Python: manifold.snap()
/// This must match WASM: manifold.snap()
#[test]
fn test_snap_determinism_cross_repo() {
    let manifold = PythagoreanManifold::new(200);
    
    // Test 3-4-5 triangle
    let (snapped, noise) = snap(&manifold, [0.6, 0.8]);
    assert!((snapped[0] - 0.6).abs() < 0.01);
    assert!((snapped[1] - 0.8).abs() < 0.01);
    assert!(noise < 0.001, "Noise should be near zero for exact triple");
    
    // Test determinism: same input = same output
    for _ in 0..100 {
        let (s1, n1) = snap(&manifold, [0.577, 0.816]);
        let (s2, n2) = snap(&manifold, [0.577, 0.816]);
        assert_eq!(s1, s2);
        assert!((n1 - n2).abs() < 1e-10);
    }
}

/// Test quantizer modes
/// This must match Python: PythagoreanQuantizer
#[test]
fn test_quantizer_modes_cross_repo() {
    // Ternary mode (BitNet)
    let ternary = PythagoreanQuantizer::for_llm();
    let data = vec![0.5, -0.3, 0.8, -0.9, 0.1];
    let result = ternary.quantize(&data);
    
    // Verify ternary values: {-1, 0, 1}
    for v in &result.data {
        assert!(
            *v == -1.0 || *v == 0.0 || *v == 1.0,
            "Ternary value must be -1, 0, or 1, got {}",
            v
        );
    }
    
    // Polar mode (PolarQuant) - preserves unit norm
    let polar = PythagoreanQuantizer::for_embeddings();
    let unit_vec = vec![0.6, 0.8, 0.0, 0.0];
    let result = polar.quantize(&unit_vec);
    
    // Verify unit norm preserved
    let norm: f64 = result.data.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!((norm - 1.0).abs() < 0.1, "Unit norm should be preserved");
    
    // Turbo mode (TurboQuant) - near-optimal distortion
    let turbo = PythagoreanQuantizer::for_vector_db();
    let vec_data: Vec<f64> = (0..128).map(|i| (i as f64).sin()).collect();
    let result = turbo.quantize(&vec_data);
    assert!(result.distortion < 0.5, "Distortion should be low");
}

/// Test holonomy computation
/// This must match Python: compute_holonomy() / verify_holonomy()
#[test]
fn test_holonomy_cross_repo() {
    use constraint_theory_core::holonomy::rotation_z;
    
    // Create a cycle that should have zero holonomy (identity)
    let identity_cycle = vec![
        rotation_z(0.0),
        rotation_z(0.0),
        rotation_z(0.0),
    ];
    
    let result = compute_holonomy(&identity_cycle);
    assert!(result.is_identity(1e-10), "Identity cycle should have zero holonomy");
    
    // Create a cycle with known holonomy
    let quarter_turns = vec![
        rotation_z(std::f64::consts::FRAC_PI_2),
        rotation_z(std::f64::consts::FRAC_PI_2),
        rotation_z(std::f64::consts::FRAC_PI_2),
        rotation_z(std::f64::consts::FRAC_PI_2),
    ];
    
    let result = compute_holonomy(&quarter_turns);
    assert!(result.is_identity(1e-10), "4 × π/2 should be identity");
    
    // Verify holonomy checking
    let cycles = vec![identity_cycle, quarter_turns];
    assert!(verify_holonomy(&cycles, 1e-10).is_ok());
}

/// Test batch processing matches single processing
/// This must match Python: snap_batch()
#[test]
fn test_batch_consistency() {
    let manifold = PythagoreanManifold::new(200);
    
    let points: Vec<[f64; 2]> = (0..100)
        .map(|i| {
            let angle = (i as f64) * 0.0628; // ~2π/100
            [angle.cos(), angle.sin()]
        })
        .collect();
    
    // Process individually
    let individual: Vec<_> = points.iter()
        .map(|p| snap(&manifold, *p))
        .collect();
    
    // Process as batch
    let batch = manifold.snap_batch(&points);
    
    // Verify same results
    for (ind, bat) in individual.iter().zip(batch.iter()) {
        assert!(
            (ind.0[0] - bat.0[0]).abs() < 1e-10,
            "Batch snap should match individual"
        );
        assert!(
            (ind.0[1] - bat.0[1]).abs() < 1e-10,
            "Batch snap should match individual"
        );
    }
}

/// Test edge cases that must be handled consistently
#[test]
fn test_edge_cases_cross_repo() {
    let manifold = PythagoreanManifold::new(200);
    
    // Test zero vector handling
    let result = manifold.validate_input([0.0, 0.0]);
    assert!(result.is_err(), "Zero vector should be invalid");
    
    // Test NaN handling
    let result = manifold.validate_input([f64::NAN, 0.5]);
    assert!(result.is_err(), "NaN should be invalid");
    
    // Test Infinity handling
    let result = manifold.validate_input([f64::INFINITY, 0.5]);
    assert!(result.is_err(), "Infinity should be invalid");
    
    // Test very small values
    let (snapped, _) = snap(&manifold, [1e-10, 1.0]);
    assert!(snapped[0].is_finite());
    assert!(snapped[1].is_finite());
}

/// Performance benchmark - verify O(log N) complexity
#[test]
fn test_performance_characteristics() {
    use std::time::Instant;
    
    // Test that snap time is independent of manifold density
    let small_manifold = PythagoreanManifold::new(100);
    let large_manifold = PythagoreanManifold::new(10000);
    
    let iterations = 10000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = snap(&small_manifold, [0.577, 0.816]);
    }
    let small_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = snap(&large_manifold, [0.577, 0.816]);
    }
    let large_time = start.elapsed();
    
    // Large manifold should not be significantly slower (log N scaling)
    let ratio = large_time.as_nanos() as f64 / small_time.as_nanos() as f64;
    assert!(
        ratio < 5.0,
        "Large manifold ({}) should not be much slower than small ({}): ratio = {}",
        large_time.as_millis(),
        small_time.as_millis(),
        ratio
    );
}
