#![allow(missing_docs)]

/// AC-3 arc consistency algorithm.

use crate::csp::{Constraint, ConstraintProblem, Constraint::Binary};
use std::collections::{HashSet, VecDeque};

type Domain = Vec<i64>;

/// Enforce arc consistency on binary constraints.
/// Returns false if any domain becomes empty (unsatisfiable).
pub fn enforce_ac3(problem: &ConstraintProblem, domains: &mut [Domain]) -> bool {
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    // Seed queue with all binary constraint arcs.
    for c in &problem.constraints {
        if let Binary { a, b, .. } = c {
            queue.push_back((*a, *b));
            queue.push_back((*b, *a));
        }
    }

    while let Some((xi, xj)) = queue.pop_front() {
        if revise(problem, domains, xi, xj) {
            if domains[xi].is_empty() {
                return false; // inconsistency
            }
            // All neighbors of xi (except xj) need rechecking.
            for c in &problem.constraints {
                if let Binary { a, b, .. } = c {
                    if *a == xi && *b != xj {
                        queue.push_back((*b, *a));
                    } else if *b == xi && *a != xj {
                        queue.push_back((*a, *b));
                    }
                }
            }
        }
    }
    true
}

/// Remove values from domain[xi] that have no support in domain[xj].
/// Returns true if domain[xi] was pruned.
fn revise(problem: &ConstraintProblem, domains: &mut [Domain], xi: usize, xj: usize) -> bool {
    // Find the binary constraint between xi and xj.
    let check = problem.constraints.iter().find_map(|c| {
        if let Binary { a, b, check, .. } = c {
            if (*a == xi && *b == xj) || (*a == xj && *b == xi) {
                return Some(*check);
            }
        }
        None
    });

    let check = match check {
        Some(f) => f,
        None => return false, // no constraint between them
    };

    // Determine whether the arc direction matches the constraint (a,b) ordering
    let forward = problem.constraints.iter().any(|c| {
        if let Binary { a, b, .. } = c {
            *a == xi && *b == xj
        } else {
            false
        }
    });

    let mut changed = false;
    let xj_domain: Vec<i64> = domains[xj].clone();

    domains[xi].retain(|&vx| {
        let supported = if forward {
            xj_domain.iter().any(|&vy| check(vx, vy))
        } else {
            // Arc is opposite constraint direction: swap parameter order
            xj_domain.iter().any(|&vy| check(vy, vx))
        };
        if !supported {
            changed = true;
        }
        supported
    });

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csp::{ConstraintProblem, Variable, Constraint, lt_fn, neq_fn};

    #[test]
    fn test_ac3_basic() {
        // Two vars: x != y
        let vars = vec![
            Variable::range("x", 1, 3),
            Variable::range("y", 1, 3),
        ];
        let cs = vec![Constraint::Binary {
            a: 0, b: 1,
            check: neq_fn,
            desc: "!=",
        }];
        let problem = ConstraintProblem::new(vars.clone(), cs);
        let mut domains: Vec<Domain> = vars.iter().map(|v| v.domain.clone()).collect();
        assert!(enforce_ac3(&problem, &mut domains));
        // x and y still have all values (each value has support)
        assert_eq!(domains[0].len(), 3);
        assert_eq!(domains[1].len(), 3);
    }

    #[test]
    fn test_ac3_unsat() {
        // Two vars with domains {1}, {1}, and x != y
        let vars = vec![
            Variable::new("x", vec![1]),
            Variable::new("y", vec![1]),
        ];
        let cs = vec![Constraint::Binary {
            a: 0, b: 1,
            check: neq_fn,
            desc: "!=",
        }];
        let problem = ConstraintProblem::new(vars.clone(), cs);
        let mut domains: Vec<Domain> = vars.iter().map(|v| v.domain.clone()).collect();
        assert!(!enforce_ac3(&problem, &mut domains));
        assert!(domains[0].is_empty());
    }

    #[test]
    fn test_ac3_prunes() {
        // x in {1,2,3}, y in {2,3,4}, constraint x < y
        // x=3 HAS support (y=4), so no pruning
        let vars = vec![
            Variable::new("x", vec![1, 2, 3]),
            Variable::new("y", vec![2, 3, 4]),
        ];
        let cs = vec![Constraint::Binary {
            a: 0, b: 1,
            check: lt_fn,
            desc: "<",
        }];
        let problem = ConstraintProblem::new(vars.clone(), cs);
        let mut domains: Vec<Domain> = vars.iter().map(|v| v.domain.clone()).collect();
        assert!(enforce_ac3(&problem, &mut domains));
        assert_eq!(domains[0], vec![1, 2, 3]);
        assert_eq!(domains[1], vec![2, 3, 4]);
    }
}
