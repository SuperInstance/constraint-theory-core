//! Module Coverage Tests for Constraint Theory Core
//!
//! Tests targeting modules with low test coverage:
//! - csp (0 inline tests → comprehensive coverage)
//! - cohomology (1 test → expanded coverage)
//! - curvature (2 tests → expanded coverage)
//! - gauge (1 test → expanded coverage)
//! - percolation (1 test → expanded coverage)
//! - ac3 (3 tests → expanded coverage)

// Top-level imports used by cross_module tests below
#[allow(unused_imports)]
use constraint_theory_core::{
    cohomology::FastCohomology, curvature::RicciFlow, percolation::FastPercolation,
};

// ============================================================================
// CSP Module Tests
// ============================================================================

mod csp_tests {
    use constraint_theory_core::csp::{
        eq, eq_fn, lt, lt_fn, neq, neq_fn, Constraint, ConstraintProblem, SolverConfig,
        SolverStats, Variable,
    };

    #[test]
    fn test_variable_new() {
        let v = Variable::new("x", vec![1, 2, 3]);
        assert_eq!(v.name, "x");
        assert_eq!(v.domain, vec![1, 2, 3]);
    }

    #[test]
    fn test_variable_range() {
        let v = Variable::range("y", 1, 5);
        assert_eq!(v.name, "y");
        assert_eq!(v.domain, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_constraint_vars_unary() {
        let c = Constraint::Unary {
            var: 2,
            check: |_| true,
            desc: "test",
        };
        assert_eq!(c.vars(), vec![2]);
        assert!(c.involves(2));
        assert!(!c.involves(0));
    }

    #[test]
    fn test_constraint_vars_binary() {
        let c = Constraint::Binary {
            a: 0,
            b: 1,
            check: neq_fn,
            desc: "!=",
        };
        assert_eq!(c.vars(), vec![0, 1]);
        assert!(c.involves(0));
        assert!(c.involves(1));
        assert!(!c.involves(2));
    }

    #[test]
    fn test_constraint_vars_nary() {
        let c = Constraint::Nary {
            vars: vec![0, 2, 4],
            check: |_| true,
            desc: "test",
        };
        assert_eq!(c.vars(), vec![0, 2, 4]);
        assert!(c.involves(0));
        assert!(!c.involves(1));
        assert!(c.involves(4));
    }

    #[test]
    fn test_constraint_debug_format() {
        let c1 = Constraint::Unary {
            var: 0,
            check: |_| true,
            desc: "pos",
        };
        assert!(format!("{:?}", c1).contains("Unary"));

        let c2 = Constraint::Binary {
            a: 0,
            b: 1,
            check: neq_fn,
            desc: "!=",
        };
        assert!(format!("{:?}", c2).contains("Binary"));

        let c3 = Constraint::Nary {
            vars: vec![0, 1],
            check: |_| true,
            desc: "test",
        };
        assert!(format!("{:?}", c3).contains("Nary"));
    }

    #[test]
    fn test_problem_new_and_var_index() {
        let vars = vec![Variable::range("x", 1, 3), Variable::range("y", 1, 3)];
        let cs = vec![neq(0, 1)];
        let problem = ConstraintProblem::new(vars, cs);

        assert_eq!(problem.var_index("x"), Some(0));
        assert_eq!(problem.var_index("y"), Some(1));
        assert_eq!(problem.var_index("z"), None);
    }

    #[test]
    fn test_problem_var_count() {
        let vars = vec![
            Variable::range("a", 0, 1),
            Variable::range("b", 0, 1),
            Variable::range("c", 0, 1),
        ];
        let problem = ConstraintProblem::new(vars, vec![]);
        assert_eq!(problem.var_count(), 3);
    }

    #[test]
    fn test_problem_domain_size_and_values() {
        let vars = vec![Variable::range("x", 1, 5)];
        let problem = ConstraintProblem::new(vars, vec![]);
        assert_eq!(problem.domain_size(0), 5);
        assert_eq!(problem.domain_values(0), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_is_consistent_binary_neq() {
        let vars = vec![Variable::range("x", 1, 3), Variable::range("y", 1, 3)];
        let cs = vec![neq(0, 1)];
        let problem = ConstraintProblem::new(vars, cs);

        assert!(problem.is_consistent(&[(0, 1), (1, 2)]));
        assert!(problem.is_consistent(&[(0, 1), (1, 3)]));
        assert!(!problem.is_consistent(&[(0, 2), (1, 2)]));
    }

    #[test]
    fn test_is_consistent_partial_assignment() {
        let vars = vec![Variable::range("x", 1, 3), Variable::range("y", 1, 3)];
        let cs = vec![neq(0, 1)];
        let problem = ConstraintProblem::new(vars, cs);

        // Only one var assigned — constraint not fully touched, should be consistent
        assert!(problem.is_consistent(&[(0, 1)]));
    }

    #[test]
    fn test_is_satisfied_full_assignment() {
        let vars = vec![Variable::range("x", 1, 3), Variable::range("y", 1, 3)];
        let cs = vec![neq(0, 1)];
        let problem = ConstraintProblem::new(vars, cs);

        use std::collections::HashMap;
        let sat: HashMap<usize, i64> = [(0, 1), (1, 2)].into_iter().collect();
        assert!(problem.is_satisfied(&sat));

        let unsat: HashMap<usize, i64> = [(0, 2), (1, 2)].into_iter().collect();
        assert!(!problem.is_satisfied(&unsat));
    }

    #[test]
    fn test_is_satisfied_unary_constraint() {
        fn positive(x: i64) -> bool {
            x > 0
        }
        let vars = vec![Variable::range("x", -3, 3)];
        let cs = vec![Constraint::Unary {
            var: 0,
            check: positive,
            desc: "positive",
        }];
        let problem = ConstraintProblem::new(vars, cs);

        use std::collections::HashMap;
        let sat: HashMap<usize, i64> = [(0, 2)].into_iter().collect();
        assert!(problem.is_satisfied(&sat));

        let unsat: HashMap<usize, i64> = [(0, -1)].into_iter().collect();
        assert!(!problem.is_satisfied(&unsat));
    }

    #[test]
    fn test_is_satisfied_nary_constraint() {
        let vars = vec![Variable::range("x", 1, 5), Variable::range("y", 1, 5)];
        // Sum must be even
        fn sum_even(vals: &[i64]) -> bool {
            vals.iter().sum::<i64>() % 2 == 0
        }
        let cs = vec![Constraint::Nary {
            vars: vec![0, 1],
            check: sum_even,
            desc: "even_sum",
        }];
        let problem = ConstraintProblem::new(vars, cs);

        use std::collections::HashMap;
        let sat: HashMap<usize, i64> = [(0, 1), (1, 3)].into_iter().collect();
        assert!(problem.is_satisfied(&sat));

        let unsat: HashMap<usize, i64> = [(0, 1), (1, 2)].into_iter().collect();
        assert!(!problem.is_satisfied(&unsat));
    }

    #[test]
    fn test_constraints_involving() {
        let vars = vec![
            Variable::range("x", 1, 3),
            Variable::range("y", 1, 3),
            Variable::range("z", 1, 3),
        ];
        let cs = vec![neq(0, 1), neq(1, 2)];
        let problem = ConstraintProblem::new(vars, cs);

        let c_x = problem.constraints_involving(0);
        assert_eq!(c_x.len(), 1);

        let c_y = problem.constraints_involving(1);
        assert_eq!(c_y.len(), 2);

        let c_z = problem.constraints_involving(2);
        assert_eq!(c_z.len(), 1);
    }

    #[test]
    fn test_all_diff() {
        let constraints = ConstraintProblem::all_diff(&[0, 1, 2]);
        assert_eq!(constraints.len(), 3); // C(3,2) = 3 pairwise constraints

        // Verify all are binary != constraints
        for c in &constraints {
            if let Constraint::Binary { check, desc, .. } = c {
                assert_eq!(*desc, "alldiff");
                assert!(!check(1, 1));
                assert!(check(1, 2));
            } else {
                panic!("Expected binary constraint");
            }
        }
    }

    #[test]
    fn test_helper_convenience_fns() {
        // neq
        let _c = neq(0, 1);
        assert!(!neq_fn(5, 5));
        assert!(neq_fn(3, 4));

        // eq
        let _c = eq(0, 1);
        assert!(eq_fn(5, 5));
        assert!(!eq_fn(3, 4));

        // lt
        let _c = lt(0, 1);
        assert!(lt_fn(3, 4));
        assert!(!lt_fn(4, 3));
        assert!(!lt_fn(3, 3));
    }

    #[test]
    fn test_solver_config_default() {
        let config = SolverConfig::default();
        assert!(config.use_mrv);
        assert!(!config.use_lcv);
        assert!(config.use_forward_checking);
        assert!(config.use_ac3);
    }

    #[test]
    fn test_solver_stats_default() {
        let stats = SolverStats::new();
        assert_eq!(stats.nodes_visited, 0);
        assert_eq!(stats.backtracks, 0);
        assert_eq!(stats.propagations, 0);
        assert!(stats.summary().contains("nodes=0"));
    }
}

// ============================================================================
// Cohomology Module Tests
// ============================================================================

mod cohomology_tests {
    use constraint_theory_core::cohomology::{CohomologyResult, FastCohomology};

    #[test]
    fn test_single_vertex() {
        let result = FastCohomology::compute(1, 0, 1);
        assert_eq!(result.h0_dim, 1);
        assert_eq!(result.h1_dim, 0);
        assert_eq!(result.n_vertices, 1);
        assert_eq!(result.n_edges, 0);
    }

    #[test]
    fn test_tree_no_cycles() {
        // A tree with 5 vertices and 4 edges has no cycles
        let result = FastCohomology::compute(5, 4, 1);
        assert_eq!(result.h0_dim, 1); // connected
        assert_eq!(result.h1_dim, 0); // no independent cycles
    }

    #[test]
    fn test_single_cycle() {
        // Triangle: 3 vertices, 3 edges → 1 cycle
        let result = FastCohomology::compute(3, 3, 1);
        assert_eq!(result.h0_dim, 1);
        assert_eq!(result.h1_dim, 1);
    }

    #[test]
    fn test_disconnected_components() {
        // Two disconnected vertices
        let result = FastCohomology::compute(2, 0, 2);
        assert_eq!(result.h0_dim, 2);
        assert_eq!(result.h1_dim, 0);
    }

    #[test]
    fn test_complex_topology() {
        // Complete graph K4: 4 vertices, 6 edges
        let result = FastCohomology::compute(4, 6, 1);
        assert_eq!(result.h0_dim, 1);
        assert_eq!(result.h1_dim, 3); // 6 - 4 + 1 = 3
    }

    #[test]
    fn test_fewer_edges_than_vertices() {
        // Sparse graph: 10 vertices, 5 edges, 5 components
        let result = FastCohomology::compute(10, 5, 5);
        assert_eq!(result.h0_dim, 5);
        assert_eq!(result.h1_dim, 0); // edges < vertices, so 0
    }

    #[test]
    fn test_cohomology_result_clone_copy() {
        let r1 = CohomologyResult {
            h0_dim: 2,
            h1_dim: 3,
            n_vertices: 10,
            n_edges: 12,
        };
        let r2 = r1; // Copy
        assert_eq!(r1.h0_dim, r2.h0_dim);
        assert_eq!(r1.h1_dim, r2.h1_dim);
    }
}

// ============================================================================
// Curvature Module Tests
// ============================================================================

mod curvature_tests {
    use constraint_theory_core::curvature::{ricci_flow_step, RicciFlow};

    #[test]
    fn test_ricci_flow_convergence_to_target() {
        let mut rf = RicciFlow::new(0.5, 1.0);
        let mut curvatures = vec![0.0, 0.0, 0.0];
        rf.evolve(&mut curvatures, 20);

        for &c in &curvatures {
            assert!(
                (c - 1.0).abs() < 0.01,
                "Should converge to target=1.0, got {}",
                c
            );
        }
    }

    #[test]
    fn test_ricci_flow_negative_curvature() {
        let mut rf = RicciFlow::new(0.1, 0.0);
        let mut curvatures = vec![-2.0];
        rf.evolve(&mut curvatures, 50);
        assert!(
            curvatures[0].abs() < 0.1,
            "Negative curvature should converge to 0"
        );
    }

    #[test]
    fn test_ricci_flow_step_boundary() {
        // Alpha = 0 → no change
        let c = ricci_flow_step(5.0, 0.0, 0.0);
        assert_eq!(c, 5.0);

        // Alpha = 1 → jump to target
        let c = ricci_flow_step(5.0, 1.0, 3.0);
        assert_eq!(c, 3.0);
    }

    #[test]
    fn test_ricci_flow_with_defaults() {
        let rf = RicciFlow::with_defaults();
        // Just verify it can be created and used
        let mut rf = rf;
        let mut c = vec![2.0];
        rf.evolve(&mut c, 10);
        assert!(c[0] < 2.0, "Should evolve toward target=0");
    }

    #[test]
    fn test_ricci_flow_preserves_target() {
        // If curvature is already at target, it should stay
        let mut rf = RicciFlow::new(0.5, 1.0);
        let mut curvatures = vec![1.0, 1.0];
        rf.evolve(&mut curvatures, 100);
        for &c in &curvatures {
            assert!((c - 1.0).abs() < 1e-6, "Should stay at target");
        }
    }

    #[test]
    fn test_ricci_flow_zero_steps() {
        let mut rf = RicciFlow::new(0.5, 0.0);
        let mut curvatures = vec![5.0];
        rf.evolve(&mut curvatures, 0);
        assert_eq!(curvatures[0], 5.0, "Zero steps = no change");
    }

    #[test]
    fn test_ricci_flow_empty_slice() {
        let mut rf = RicciFlow::new(0.1, 0.0);
        let mut curvatures: Vec<f32> = vec![];
        rf.evolve(&mut curvatures, 10);
        // Should not panic
    }
}

// ============================================================================
// Gauge Module Tests
// ============================================================================

mod gauge_tests {
    use constraint_theory_core::gauge::GaugeConnection;
    use constraint_theory_core::tile::Tile;

    #[test]
    fn test_parallel_transport_identity() {
        let tiles = vec![Tile::new(0), Tile::new(1), Tile::new(2)];
        let conn = GaugeConnection::new(tiles);

        let result = conn.parallel_transport([1.0, 0.0, 0.0], &[0, 1, 2]);
        // Default holonomy is identity
        assert!((result[0] - 1.0).abs() < 0.01);
        assert!(result[1].abs() < 0.01);
        assert!(result[2].abs() < 0.01);
    }

    #[test]
    fn test_parallel_transport_single_tile() {
        let tiles = vec![Tile::new(0)];
        let conn = GaugeConnection::new(tiles);

        // Path with single tile — no edges to traverse
        let result = conn.parallel_transport([0.0, 1.0, 0.0], &[0]);
        assert!((result[0] - 0.0).abs() < 0.01);
        assert!((result[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parallel_transport_empty_path() {
        let tiles = vec![Tile::new(0), Tile::new(1)];
        let conn = GaugeConnection::new(tiles);

        let result = conn.parallel_transport([1.0, 2.0, 3.0], &[]);
        // No path → no transformation
        assert_eq!(result, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_parallel_transport_out_of_bounds() {
        let tiles = vec![Tile::new(0)];
        let conn = GaugeConnection::new(tiles);

        // Path references non-existent tile — should handle gracefully
        let result = conn.parallel_transport([1.0, 0.0, 0.0], &[0, 5]);
        // Should not panic; result may be identity since tile 5 doesn't exist
        assert!(result[0].is_finite());
    }

    #[test]
    fn test_gauge_with_many_tiles() {
        let tiles: Vec<Tile> = (0..10).map(Tile::new).collect();
        let conn = GaugeConnection::new(tiles);

        let result = conn.parallel_transport([0.5, 0.5, 0.5], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        // All default identity holonomies → vector unchanged
        assert!((result[0] - 0.5).abs() < 0.01);
        assert!((result[1] - 0.5).abs() < 0.01);
        assert!((result[2] - 0.5).abs() < 0.01);
    }
}

// ============================================================================
// Percolation Module Tests
// ============================================================================

mod percolation_tests {
    use constraint_theory_core::percolation::FastPercolation;

    #[test]
    fn test_single_edge() {
        let mut perc = FastPercolation::new(2);
        let result = perc.compute_rigidity(&[(0, 1)], 2);
        assert!(!result.is_rigid); // 2 nodes < 3, trivially non-rigid
        assert_eq!(result.n_clusters, 1);
    }

    #[test]
    fn test_minimally_rigid_graph() {
        // 3 vertices, 3 edges (triangle): 2*3-3 = 3 edges needed
        let mut perc = FastPercolation::new(3);
        let edges = [(0, 1), (1, 2), (0, 2)];
        let result = perc.compute_rigidity(&edges, 3);
        assert!(result.is_rigid, "Triangle should be minimally rigid");
        assert_eq!(result.rank, 3);
        assert_eq!(result.deficiency, 0);
    }

    #[test]
    fn test_over_constrained_graph() {
        // 4 vertices with 7 edges (2*4-3 = 5 needed, 7 > 5)
        let mut perc = FastPercolation::new(4);
        let edges = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3), (0, 1)];
        let result = perc.compute_rigidity(&edges, 4);
        assert!(result.is_rigid);
    }

    #[test]
    fn test_floppy_graph() {
        // 5 vertices with only 3 edges (2*5-3 = 7 needed)
        let mut perc = FastPercolation::new(5);
        let edges = [(0, 1), (1, 2), (3, 4)];
        let result = perc.compute_rigidity(&edges, 5);
        assert!(
            !result.is_rigid,
            "Under-constrained graph should not be rigid"
        );
    }

    #[test]
    fn test_disconnected_components_rigidity() {
        let mut perc = FastPercolation::new(6);
        // Two separate triangles
        let edges = [(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)];
        let result = perc.compute_rigidity(&edges, 6);
        assert_eq!(result.n_clusters, 2);
    }

    #[test]
    fn test_no_edges() {
        let mut perc = FastPercolation::new(4);
        let result = perc.compute_rigidity(&[], 4);
        assert!(!result.is_rigid);
        assert_eq!(result.n_clusters, 4);
        assert_eq!(result.rigid_fraction, 0.0);
    }

    #[test]
    fn test_single_node() {
        let mut perc = FastPercolation::new(1);
        let result = perc.compute_rigidity(&[], 1);
        assert!(!result.is_rigid); // < 3 vertices
        assert_eq!(result.n_clusters, 1);
    }

    #[test]
    fn test_rigid_fraction() {
        // 6 nodes: triangle (0,1,2) + 3 isolated nodes
        let mut perc = FastPercolation::new(6);
        let edges = [(0, 1), (1, 2), (0, 2)];
        let result = perc.compute_rigidity(&edges, 6);
        // The triangle nodes are in a cluster of size 3 (≥ 3), others are size 1
        // rigid_fraction = 3/6 = 0.5
        assert!((result.rigid_fraction - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_larger_rigid_graph() {
        // 10 nodes, fully connected subset → 2*10-3 = 17 edges for rigidity
        let mut perc = FastPercolation::new(10);
        let mut edges = Vec::new();
        for i in 0..10 {
            for j in (i + 1)..10 {
                edges.push((i, j));
            }
        }
        let result = perc.compute_rigidity(&edges, 10);
        assert!(result.is_rigid, "Complete graph K10 should be rigid");
        assert_eq!(result.n_clusters, 1);
    }
}

// ============================================================================
// AC-3 Module Tests (expanded)
// ============================================================================

mod ac3_tests {
    use constraint_theory_core::ac3::enforce_ac3;
    use constraint_theory_core::csp::{
        eq_fn, lt_fn, neq_fn, Constraint, ConstraintProblem, Variable,
    };

    #[test]
    fn test_ac3_eq_constraint() {
        // x == y, both in {1,2,3}
        let vars = vec![Variable::range("x", 1, 3), Variable::range("y", 1, 3)];
        let cs = vec![Constraint::Binary {
            a: 0,
            b: 1,
            check: eq_fn,
            desc: "==",
        }];
        let problem = ConstraintProblem::new(vars.clone(), cs);
        let mut domains: Vec<Vec<i64>> = vars.iter().map(|v| v.domain.clone()).collect();
        assert!(enforce_ac3(&problem, &mut domains));
        // All values have support under ==
        assert_eq!(domains[0].len(), 3);
        assert_eq!(domains[1].len(), 3);
    }

    #[test]
    fn test_ac3_lt_prunes_lower() {
        // x < y, x in {1,2,3}, y in {1,2,3}
        // y=1 has no support (no x < 1), so y domain should prune 1
        let vars = vec![Variable::range("x", 1, 3), Variable::range("y", 1, 3)];
        let cs = vec![Constraint::Binary {
            a: 0,
            b: 1,
            check: lt_fn,
            desc: "<",
        }];
        let problem = ConstraintProblem::new(vars.clone(), cs);
        let mut domains: Vec<Vec<i64>> = vars.iter().map(|v| v.domain.clone()).collect();
        assert!(enforce_ac3(&problem, &mut domains));
        // x=3 should be pruned (no y > 3), y=1 should be pruned (no x < 1)
        assert!(!domains[0].contains(&3), "x=3 should be pruned (no y > 3)");
        assert!(!domains[1].contains(&1), "y=1 should be pruned (no x < 1)");
    }

    #[test]
    fn test_ac3_three_var_chain() {
        // x != y, y != z, x != z (all-diff on 3 vars with domain {1,2})
        let vars = vec![
            Variable::range("x", 1, 2),
            Variable::range("y", 1, 2),
            Variable::range("z", 1, 2),
        ];
        let cs = vec![
            Constraint::Binary {
                a: 0,
                b: 1,
                check: neq_fn,
                desc: "!=",
            },
            Constraint::Binary {
                a: 1,
                b: 2,
                check: neq_fn,
                desc: "!=",
            },
        ];
        let problem = ConstraintProblem::new(vars.clone(), cs);
        let mut domains: Vec<Vec<i64>> = vars.iter().map(|v| v.domain.clone()).collect();
        // No x!=z constraint, so this should be satisfiable with 2 vars
        assert!(enforce_ac3(&problem, &mut domains));
    }

    #[test]
    fn test_ac3_no_constraints() {
        let vars = vec![Variable::range("x", 1, 5)];
        let problem = ConstraintProblem::new(vars.clone(), vec![]);
        let mut domains: Vec<Vec<i64>> = vars.iter().map(|v| v.domain.clone()).collect();
        assert!(enforce_ac3(&problem, &mut domains));
        assert_eq!(domains[0].len(), 5);
    }

    #[test]
    fn test_ac3_tight_domains() {
        // x != y, x in {1}, y in {1,2}
        // x=1 → y can only be 2
        let vars = vec![Variable::new("x", vec![1]), Variable::new("y", vec![1, 2])];
        let cs = vec![Constraint::Binary {
            a: 0,
            b: 1,
            check: neq_fn,
            desc: "!=",
        }];
        let problem = ConstraintProblem::new(vars.clone(), cs);
        let mut domains: Vec<Vec<i64>> = vars.iter().map(|v| v.domain.clone()).collect();
        assert!(enforce_ac3(&problem, &mut domains));
        assert_eq!(domains[1], vec![2]); // y=1 pruned
    }
}

// ============================================================================
// Cross-Module Integration Tests
// ============================================================================

mod cross_module {
    use constraint_theory_core::{
        cohomology::FastCohomology, curvature::RicciFlow, percolation::FastPercolation,
    };

    #[test]
    fn test_cohomology_after_percolation() {
        // Build a graph, check rigidity, then compute cohomology
        let mut perc = FastPercolation::new(6);
        let edges = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)];
        let _rigidity = perc.compute_rigidity(&edges, 6);

        // Connected graph with cycle
        let cohomology = FastCohomology::compute(6, 6, 1);
        assert_eq!(cohomology.h0_dim, 1); // connected
        assert_eq!(cohomology.h1_dim, 1); // one cycle (6 - 6 + 1)
    }

    #[test]
    fn test_curvature_on_rigid_graph() {
        // Evolve curvatures on vertices of a rigid graph
        let mut rf = RicciFlow::new(0.1, 0.0);
        let mut curvatures = vec![2.0, -1.0, 0.5, 3.0, -2.0, 1.0];
        rf.evolve(&mut curvatures, 100);

        for &c in &curvatures {
            assert!(c.abs() < 0.01, "Should converge to flat curvature");
        }
    }
}
