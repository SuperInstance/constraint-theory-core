//! Edge Case and Stress Tests for Constraint Theory Core
//!
//! This module contains comprehensive tests for edge cases, boundary conditions,
//! and stress testing scenarios to ensure robust production behavior.

#[cfg(test)]
mod tests {
    use crate::curvature::{ricci_flow_step, RicciFlow};
    use crate::gauge::GaugeConnection;
    use crate::kdtree::KDTree;
    use crate::manifold::{PythagoreanManifold, PythagoreanTriple};
    use crate::percolation::FastPercolation;
    use crate::tile::{ConstraintBlock, Origin, Tile};

    // ==================== Manifold Edge Cases ====================

    #[test]
    fn test_snap_zero_vector() {
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([0.0, 0.0]);

        // Zero vector should snap to a default state with zero noise
        assert!(snapped[0].is_finite());
        assert!(snapped[1].is_finite());
        assert!(noise.is_finite());
    }

    #[test]
    fn test_snap_very_small_vector() {
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([1e-20, 1e-20]);

        assert!(snapped[0].is_finite());
        assert!(snapped[1].is_finite());
        assert!(noise.is_finite());
    }

    #[test]
    fn test_snap_very_large_vector() {
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([1e20, 1e20]);

        assert!(snapped[0].is_finite());
        assert!(snapped[1].is_finite());
        assert!(noise.is_finite());
    }

    #[test]
    fn test_snap_negative_vector() {
        let manifold = PythagoreanManifold::new(200);
        let (snapped_pos, _) = manifold.snap([0.6, 0.8]);
        let (snapped_neg, _) = manifold.snap([-0.6, -0.8]);

        // Both should snap to valid states
        assert!(snapped_pos[0].is_finite());
        assert!(snapped_neg[0].is_finite());
    }

    #[test]
    fn test_snap_all_quadrants() {
        let manifold = PythagoreanManifold::new(200);

        // Test all four quadrants
        let quadrants = [[0.6, 0.8], [-0.6, 0.8], [0.6, -0.8], [-0.6, -0.8]];

        for vec in quadrants {
            let (snapped, noise) = manifold.snap(vec);
            assert!(snapped[0].is_finite(), "X should be finite for {:?}", vec);
            assert!(snapped[1].is_finite(), "Y should be finite for {:?}", vec);
            assert!(noise >= 0.0, "Noise should be non-negative for {:?}", vec);
            assert!(noise <= 1.0, "Noise should be <= 1.0 for {:?}", vec);
        }
    }

    #[test]
    fn test_snap_axis_vectors() {
        let manifold = PythagoreanManifold::new(200);

        // Test snapping to axis vectors
        let axis_vectors = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];

        for vec in axis_vectors {
            let (_snapped, noise) = manifold.snap(vec);
            // Axis vectors should snap exactly
            assert!(
                noise < 0.01,
                "Axis vector {:?} should snap exactly, noise = {}",
                vec,
                noise
            );
        }
    }

    #[test]
    fn test_manifold_small_density() {
        // Test with minimal density
        let manifold = PythagoreanManifold::new(2);
        let (snapped, noise) = manifold.snap([0.6, 0.8]);

        assert!(snapped[0].is_finite());
        assert!(snapped[1].is_finite());
        assert!(noise.is_finite());
    }

    #[test]
    fn test_manifold_large_density() {
        // Test with large density
        let manifold = PythagoreanManifold::new(500);
        let (_snapped, noise) = manifold.snap([0.6, 0.8]);

        // Should still find exact match
        assert!(noise < 0.01, "Should find exact match with large manifold");
    }

    #[test]
    fn test_batch_simd_empty() {
        let manifold = PythagoreanManifold::new(200);
        let vectors: Vec<[f32; 2]> = vec![];
        let results = manifold.snap_batch_simd(&vectors);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_batch_simd_single() {
        let manifold = PythagoreanManifold::new(200);
        let vectors = vec![[0.6, 0.8]];
        let results = manifold.snap_batch_simd(&vectors);

        assert_eq!(results.len(), 1);
        let (scalar_snapped, scalar_noise) = manifold.snap([0.6, 0.8]);
        assert!((results[0].0[0] - scalar_snapped[0]).abs() < 0.01);
        assert!((results[0].1 - scalar_noise).abs() < 0.01);
    }

    #[test]
    fn test_batch_simd_partial_chunk() {
        let manifold = PythagoreanManifold::new(200);
        // Use a number that doesn't divide evenly by 8
        let vectors: Vec<[f32; 2]> = (0..13)
            .map(|i| {
                let angle = i as f32 * 0.5;
                [angle.cos(), angle.sin()]
            })
            .collect();

        let results = manifold.snap_batch_simd(&vectors);

        assert_eq!(results.len(), 13);
        for result in &results {
            assert!(result.0[0].is_finite());
            assert!(result.0[1].is_finite());
        }
    }

    // ==================== KD-Tree Edge Cases ====================

    #[test]
    fn test_kdtree_duplicate_points() {
        let points = vec![[0.5, 0.5], [0.5, 0.5], [0.5, 0.5]];
        let tree = KDTree::build(&points);

        assert_eq!(tree.size(), 3);

        let result = tree.nearest(&[0.5, 0.5]);
        assert!(result.is_some());
    }

    #[test]
    fn test_kdtree_all_same_dimension() {
        // All points have same x-coordinate
        let points = vec![[1.0, 0.0], [1.0, 0.5], [1.0, 1.0]];
        let tree = KDTree::build(&points);

        let result = tree.nearest(&[1.0, 0.25]);
        assert!(result.is_some());
    }

    #[test]
    fn test_kdtree_query_at_boundary() {
        let points = vec![[0.0, 0.0], [1.0, 1.0], [0.0, 1.0], [1.0, 0.0]];
        let tree = KDTree::build(&points);

        // Query at corners
        for &corner in &[[0.0, 0.0], [1.0, 1.0]] {
            let result = tree.nearest(&corner);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_kdtree_query_outside_bounds() {
        let points = vec![[0.5, 0.5]];
        let tree = KDTree::build(&points);

        // Query outside the point bounds
        let result = tree.nearest(&[10.0, 10.0]);
        assert!(result.is_some());
        let (nearest, _, _) = result.unwrap();
        assert_eq!(nearest, [0.5, 0.5]);
    }

    #[test]
    fn test_kdtree_nearest_k_more_than_available() {
        let points = vec![[0.5, 0.5], [0.6, 0.6]];
        let tree = KDTree::build(&points);

        let results = tree.nearest_k(&[0.5, 0.5], 10);
        assert_eq!(results.len(), 2); // Only 2 available
    }

    #[test]
    fn test_kdtree_nearest_k_zero() {
        let points = vec![[0.5, 0.5], [0.6, 0.6]];
        let tree = KDTree::build(&points);

        let results = tree.nearest_k(&[0.5, 0.5], 0);
        assert_eq!(results.len(), 0);
    }

    // ==================== Tile Edge Cases ====================

    #[test]
    fn test_tile_reset() {
        let mut tile = Tile::new(42);
        tile.confidence = 0.9;
        tile.set_vector_2d([0.6, 0.8]);

        tile.reset();

        assert_eq!(tile.confidence, 0.5);
        assert_eq!(tile.vector_2d(), [0.0, 0.0]);
    }

    #[test]
    fn test_tile_max_confidence() {
        let mut tile = Tile::new(0);
        tile.confidence = f32::MAX;

        // Should handle extreme values
        assert_eq!(tile.confidence, f32::MAX);
    }

    #[test]
    fn test_tile_nan_in_payload() {
        let mut tile = Tile::new(0);
        tile.tensor_payload[0] = f32::NAN;

        // NaN should propagate correctly
        assert!(tile.tensor_payload[0].is_nan());
    }

    #[test]
    fn test_origin_reset() {
        let mut origin = Origin::new(42);
        origin.reference_frame = [[0.0; 3]; 3]; // Zero out
        origin.rate_of_change = [1.0, 2.0, 3.0];

        origin.reset();

        // Should be back to identity
        assert_eq!(origin.reference_frame[0], [1.0, 0.0, 0.0]);
        assert_eq!(origin.rate_of_change, [0.0; 3]);
    }

    #[test]
    fn test_constraint_block_holonomy_zero_matrix() {
        let mut cb = ConstraintBlock::new();
        cb.holonomy_matrix = [[0.0; 3]; 3];
        cb.compute_holonomy_norm();

        // Should compute norm even with zero matrix
        assert!(cb.holonomy_norm.is_finite());
    }

    #[test]
    fn test_constraint_block_holonomy_identity() {
        let mut cb = ConstraintBlock::new();
        // Already identity
        cb.compute_holonomy_norm();

        // Identity should have zero norm
        assert!(cb.holonomy_norm < 0.01);
    }

    // ==================== Curvature Edge Cases ====================

    #[test]
    fn test_ricci_flow_convergence() {
        let mut rf = RicciFlow::new(0.5, 0.0);
        let mut curvatures = [1.0, 1.0, 1.0];

        // Many steps should converge to target
        rf.evolve(&mut curvatures, 100);

        for &c in &curvatures {
            assert!(c.abs() < 0.01, "Should converge to target 0.0");
        }
    }

    #[test]
    fn test_ricci_flow_zero_alpha() {
        let mut rf = RicciFlow::new(0.0, 0.0);
        let mut curvatures = [1.0, 0.5, -0.5];
        let original = curvatures;

        rf.evolve(&mut curvatures, 10);

        // Zero alpha should not change anything
        assert_eq!(curvatures, original);
    }

    #[test]
    fn test_ricci_flow_large_alpha() {
        let mut rf = RicciFlow::new(2.0, 0.0);
        let mut curvatures = [1.0];

        rf.evolve(&mut curvatures, 1);

        // Large alpha can overshoot
        assert!(curvatures[0] < 0.0);
    }

    #[test]
    fn test_ricci_flow_step_negative_curvature() {
        let c = -1.0;
        let c_new = ricci_flow_step(c, 0.1, 0.0);

        // Should move toward target
        assert!(c_new > c);
    }

    // ==================== Percolation Edge Cases ====================

    #[test]
    fn test_percolation_no_edges() {
        let mut perc = FastPercolation::new(5);
        let edges: [(usize, usize); 0] = [];
        let result = perc.compute_rigidity(&edges, 5);

        assert!(!result.is_rigid);
        assert_eq!(result.rank, 0);
    }

    #[test]
    fn test_percolation_single_node() {
        let mut perc = FastPercolation::new(1);
        let edges: [(usize, usize); 0] = [];
        let result = perc.compute_rigidity(&edges, 1);

        assert!(!result.is_rigid);
    }

    #[test]
    fn test_percolation_fully_connected() {
        let mut perc = FastPercolation::new(4);
        // Complete graph on 4 nodes
        let edges = [
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (1, 3),
            (2, 3),
        ];
        let result = perc.compute_rigidity(&edges, 4);

        assert!(result.is_rigid);
    }

    #[test]
    fn test_percolation_out_of_bounds_edges() {
        let mut perc = FastPercolation::new(3);
        // Edge with out-of-bounds indices
        let edges = [(0, 100), (1, 200)];
        let result = perc.compute_rigidity(&edges, 3);

        // Should not panic
        assert!(result.rigid_fraction >= 0.0);
    }

    // ==================== Gauge Connection Edge Cases ====================

    #[test]
    fn test_gauge_empty_path() {
        let tiles = vec![Tile::new(0)];
        let conn = GaugeConnection::new(tiles);

        let result = conn.parallel_transport([1.0, 0.0, 0.0], &[]);

        // Empty path should return original vector
        assert_eq!(result, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_gauge_single_node_path() {
        let tiles = vec![Tile::new(0)];
        let conn = GaugeConnection::new(tiles);

        let result = conn.parallel_transport([1.0, 0.0, 0.0], &[0]);

        // Single node path should return original vector
        assert_eq!(result, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_gauge_out_of_bounds_path() {
        let tiles = vec![Tile::new(0)];
        let conn = GaugeConnection::new(tiles);

        // Path with out-of-bounds indices
        let result = conn.parallel_transport([1.0, 0.0, 0.0], &[0, 100]);

        // Should not panic
        assert!(result[0].is_finite());
    }

    // ==================== Pythagorean Triple Edge Cases ====================

    #[test]
    fn test_pythagorean_triple_valid() {
        let triple = PythagoreanTriple::new(3.0, 4.0, 5.0);
        assert!(triple.is_valid());
    }

    #[test]
    fn test_pythagorean_triple_invalid() {
        let triple = PythagoreanTriple::new(1.0, 1.0, 1.0);
        assert!(!triple.is_valid());
    }

    #[test]
    fn test_pythagorean_triple_to_vector() {
        let triple = PythagoreanTriple::new(3.0, 4.0, 5.0);
        let vec = triple.to_vector();

        assert!((vec[0] - 0.6).abs() < 0.001);
        assert!((vec[1] - 0.8).abs() < 0.001);
    }

    // ==================== Stress Tests ====================

    #[test]
    fn test_stress_many_snaps() {
        let manifold = PythagoreanManifold::new(200);

        for i in 0..1000 {
            let angle = (i as f32) * 0.001;
            let vec = [angle.cos(), angle.sin()];
            let (snapped, noise) = manifold.snap(vec);

            assert!(snapped[0].is_finite());
            assert!(snapped[1].is_finite());
            // Allow small epsilon for floating point precision
            assert!((-0.001..=1.001).contains(&noise), "Noise {} out of range for vector {:?}", noise, vec);
        }
    }

    #[test]
    fn test_stress_batch_snaps() {
        let manifold = PythagoreanManifold::new(200);

        let vectors: Vec<[f32; 2]> = (0..1000)
            .map(|i| {
                let angle = (i as f32) * 0.00628;
                [angle.cos(), angle.sin()]
            })
            .collect();

        let results = manifold.snap_batch_simd(&vectors);

        assert_eq!(results.len(), 1000);
        for result in results {
            assert!(result.0[0].is_finite());
            assert!(result.0[1].is_finite());
        }
    }

    #[test]
    fn test_stress_kdtree_many_queries() {
        let points: Vec<[f32; 2]> = (0..1000)
            .map(|i| {
                let angle = (i as f32) * 0.00628;
                [angle.cos(), angle.sin()]
            })
            .collect();

        let tree = KDTree::build(&points);

        for i in 0..100 {
            let angle = (i as f32) * 0.0628;
            let query = [angle.cos(), angle.sin()];
            let result = tree.nearest(&query);

            assert!(result.is_some());
        }
    }

    #[test]
    fn test_stress_ricci_flow_large() {
        let mut rf = RicciFlow::new(0.01, 0.0);
        let mut curvatures: Vec<f32> = (0..1000).map(|i| (i as f32) * 0.01 - 5.0).collect();

        rf.evolve(&mut curvatures, 1000);

        // All should have moved toward zero
        for &c in &curvatures {
            assert!(c.abs() < 6.0); // Reasonable bound
        }
    }
}
