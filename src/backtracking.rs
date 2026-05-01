#![allow(missing_docs)]

/// Backtracking search with heuristics (MRV, LCV, FC, AC-3/MAC).

use crate::ac3;
use crate::csp::{Constraint, Constraint::Binary, ConstraintProblem, SolverConfig, SolverStats, Variable};
use std::collections::HashMap;
use std::time::Instant;

type Domain = Vec<i64>;

/// Solve with basic chronological backtracking.
pub fn solve_bt(problem: &ConstraintProblem) -> Option<HashMap<usize, i64>> {
    let mut stats = SolverStats::new();
    let start = Instant::now();
    let mut domains: Vec<Domain> = problem.variables.iter().map(|v| v.domain.clone()).collect();
    let order: Vec<usize> = (0..problem.var_count()).collect();
    let result = backtrack(problem, &mut domains, &order, 0, &mut stats);
    stats.elapsed = start.elapsed();
    result
}

/// Solve with MRV heuristic.
pub fn solve_bt_mrv(problem: &ConstraintProblem) -> Option<HashMap<usize, i64>> {
    let mut stats = SolverStats::new();
    let start = Instant::now();
    let mut domains: Vec<Domain> = problem.variables.iter().map(|v| v.domain.clone()).collect();
    let result = backtrack_mrv(problem, &mut domains, &mut stats);
    stats.elapsed = start.elapsed();
    result
}

/// Solve with forward checking (maintains arc consistency at each node).
pub fn solve_bt_fc(problem: &ConstraintProblem) -> Option<HashMap<usize, i64>> {
    let mut stats = SolverStats::new();
    let start = Instant::now();
    let mut domains: Vec<Domain> = problem.variables.iter().map(|v| v.domain.clone()).collect();
    // AC-3 preprocessing
    if !ac3::enforce_ac3(problem, &mut domains) {
        return None;
    }
    let result = backtrack_mrv_fc(problem, &mut domains, &mut stats);
    stats.elapsed = start.elapsed();
    result
}

/// Solve with Maintaining Arc Consistency (MAC).
pub fn solve_bt_mac(problem: &ConstraintProblem) -> Option<HashMap<usize, i64>> {
    let mut stats = SolverStats::new();
    let start = Instant::now();
    let mut domains: Vec<Domain> = problem.variables.iter().map(|v| v.domain.clone()).collect();
    if !ac3::enforce_ac3(problem, &mut domains) {
        return None;
    }
    let result = backtrack_mac(problem, &mut domains, &mut stats);
    stats.elapsed = start.elapsed();
    result
}

/// Collect solver stats for a run.
pub fn solve_with_stats(problem: &ConstraintProblem, config: &SolverConfig) -> (Option<HashMap<usize, i64>>, SolverStats) {
    let start = Instant::now();
    let mut stats = SolverStats::new();
    let mut domains: Vec<Domain> = problem.variables.iter().map(|v| v.domain.clone()).collect();

    if config.use_ac3 {
        stats.propagations += 1;
        if !ac3::enforce_ac3(problem, &mut domains) {
            stats.elapsed = start.elapsed();
            return (None, stats);
        }
    }

    let result = if config.use_mrv && config.use_forward_checking {
        backtrack_mrv_fc(problem, &mut domains, &mut stats)
    } else if config.use_mrv {
        // Keep domains for MRV; no FC
        // We'll use the order-based approach with reordered variable picking
        backtrack_mrv_no_fc(problem, &mut domains, &mut stats)
    } else {
        let order: Vec<usize> = (0..problem.var_count()).collect();
        backtrack(problem, &mut domains, &order, 0, &mut stats)
    };

    stats.elapsed = start.elapsed();
    (result, stats)
}

// ---- Internal backtrack drivers ----

/// Simple chronological backtracking.
fn backtrack(
    problem: &ConstraintProblem,
    domains: &mut [Domain],
    order: &[usize],
    idx: usize,
    stats: &mut SolverStats,
) -> Option<HashMap<usize, i64>> {
    if idx == order.len() {
        // Full assignment — verify all constraints
        let result: HashMap<usize, i64> = (0..problem.var_count())
            .map(|i| (i, domains[i][0]))
            .collect();
        if problem.is_satisfied(&result) {
            return Some(result);
        }
        return None;
    }

    let var = order[idx];
    for &val in &domains[var].clone() {
        stats.nodes_visited += 1;
        let pair = (var, val);
        if !problem.is_consistent(&[pair]) {
            stats.backtracks += 1;
            continue;
        }
        let saved = domains[var].clone();
        domains[var] = vec![val];
        if let Some(rest) = backtrack(problem, domains, order, idx + 1, stats) {
            return Some(rest);
        }
        domains[var] = saved;
    }
    None
}

/// MRV-based backtracking (no forward checking).
fn backtrack_mrv(
    problem: &ConstraintProblem,
    domains: &mut [Domain],
    stats: &mut SolverStats,
) -> Option<HashMap<usize, i64>> {
    // Find unassigned var with smallest domain
    let unassigned: Vec<usize> = (0..domains.len())
        .filter(|&i| domains[i].len() > 1)
        .collect();

    if unassigned.is_empty() {
        // All assigned (domains of size 1 = assigned)
        let result: HashMap<usize, i64> = (0..domains.len())
            .filter_map(|i| {
                if domains[i].len() == 1 {
                    Some((i, domains[i][0]))
                } else {
                    None
                }
            })
            .collect();
        return if problem.is_satisfied(&result) { Some(result) } else { None };
    }

    let var = *unassigned.iter().min_by_key(|&&i| domains[i].len())?;

    for &val in &domains[var].clone() {
        stats.nodes_visited += 1;
        if !problem.is_consistent(&[(var, val)]) {
            stats.backtracks += 1;
            continue;
        }

        // Assign by reducing domain to single value
        let saved = domains[var].clone();
        domains[var] = vec![val];

        if let Some(mut result) = backtrack_mrv(problem, domains, stats) {
            result.insert(var, val);
            return Some(result);
        }

        domains[var] = saved;
    }
    None
}

/// MRV with forward checking (FC).
fn backtrack_mrv_fc(
    problem: &ConstraintProblem,
    domains: &mut [Domain],
    stats: &mut SolverStats,
) -> Option<HashMap<usize, i64>> {
    let unassigned: Vec<usize> = (0..domains.len())
        .filter(|&i| domains[i].len() > 1)
        .collect();

    if unassigned.is_empty() {
        let result: HashMap<usize, i64> = (0..domains.len())
            .filter_map(|i| {
                if domains[i].len() == 1 {
                    Some((i, domains[i][0]))
                } else {
                    None
                }
            })
            .collect();
        return if problem.is_satisfied(&result) { Some(result) } else { None };
    }

    let var = *unassigned.iter().min_by_key(|&&i| domains[i].len())?;
    let values = if unassigned.len() > 1 {
        // LCV: order values by least-constraining
        let mut scored: Vec<(i64, usize)> = domains[var]
            .iter()
            .map(|&v| {
                let conflicts = count_conflicts(problem, domains, var, v);
                (v, conflicts)
            })
            .collect();
        scored.sort_by_key(|&(_, c)| c);
        scored.into_iter().map(|(v, _)| v).collect()
    } else {
        domains[var].clone()
    };

    for &val in &values {
        stats.nodes_visited += 1;

        // Save state
        let saved_domains: Vec<Domain> = domains.iter().map(|d| d.clone()).collect();

        // Assign var = val
        domains[var] = vec![val];

        // Forward check: prune neighbors
        let mut consistent = true;
        for neighbor in &unassigned {
            if *neighbor == var {
                continue;
            }
            let saved_neighbor = domains[*neighbor].clone();
            let revised = forward_check(problem, domains, var, *neighbor);
            stats.propagations += 1;
            if revised && domains[*neighbor].is_empty() {
                consistent = false;
                domains[*neighbor] = saved_neighbor;
                break;
            }
        }

        if consistent {
            if let Some(mut result) = backtrack_mrv_fc(problem, domains, stats) {
                result.insert(var, val);
                return Some(result);
            }
        } else {
            stats.backtracks += 1;
        }

        // Restore
        for (d, s) in domains.iter_mut().zip(saved_domains.iter()) { *d = s.clone(); }
    }
    None
}

/// MRV backtracking without forward checking (keeps domains).
fn backtrack_mrv_no_fc(
    problem: &ConstraintProblem,
    domains: &mut [Domain],
    stats: &mut SolverStats,
) -> Option<HashMap<usize, i64>> {
    let unassigned: Vec<usize> = (0..domains.len())
        .filter(|&i| domains[i].len() > 1)
        .collect();

    if unassigned.is_empty() {
        let result: HashMap<usize, i64> = (0..domains.len())
            .filter_map(|i| {
                if domains[i].len() == 1 {
                    Some((i, domains[i][0]))
                } else {
                    None
                }
            })
            .collect();
        return if problem.is_satisfied(&result) { Some(result) } else { None };
    }

    let var = *unassigned.iter().min_by_key(|&&i| domains[i].len())?;

    for &val in &domains[var].clone() {
        stats.nodes_visited += 1;
        if !problem.is_consistent(&[(var, val)]) {
            stats.backtracks += 1;
            continue;
        }

        let saved = domains[var].clone();
        domains[var] = vec![val];

        if let Some(mut result) = backtrack_mrv_no_fc(problem, domains, stats) {
            result.insert(var, val);
            return Some(result);
        }

        domains[var] = saved;
    }
    None
}

/// MAC: MRV + AC-3 at each node.
fn backtrack_mac(
    problem: &ConstraintProblem,
    domains: &mut [Domain],
    stats: &mut SolverStats,
) -> Option<HashMap<usize, i64>> {
    let unassigned: Vec<usize> = (0..domains.len())
        .filter(|&i| domains[i].len() > 1)
        .collect();

    if unassigned.is_empty() {
        let result: HashMap<usize, i64> = (0..domains.len())
            .filter_map(|i| {
                if domains[i].len() == 1 {
                    Some((i, domains[i][0]))
                } else {
                    None
                }
            })
            .collect();
        return if problem.is_satisfied(&result) { Some(result) } else { None };
    }

    let var = *unassigned.iter().min_by_key(|&&i| domains[i].len())?;

    for &val in &domains[var].clone() {
        stats.nodes_visited += 1;

        let saved_domains: Vec<Domain> = domains.iter().map(|d| d.clone()).collect();
        domains[var] = vec![val];

        stats.propagations += 1;
        if ac3::enforce_ac3(problem, domains) {
            if let Some(mut result) = backtrack_mac(problem, domains, stats) {
                result.insert(var, val);
                return Some(result);
            }
        } else {
            stats.backtracks += 1;
        }

        for (d, s) in domains.iter_mut().zip(saved_domains.iter()) { *d = s.clone(); }
    }
    None
}

/// Forward check: remove values from domains[b] incompatible with domains[a] (now fixed).
/// Returns true if domain was pruned.
fn forward_check(
    problem: &ConstraintProblem,
    domains: &mut [Domain],
    a: usize,
    b: usize,
) -> bool {
    let check = problem.constraints.iter().find_map(|c| {
        if let Binary { a: ca, b: cb, check, .. } = c {
            if (*ca == a && *cb == b) || (*ca == b && *cb == a) {
                return Some(*check);
            }
        }
        None
    });

    let check = match check {
        Some(f) => f,
        None => return false,
    };

    let a_val = domains[a][0]; // a is assigned to single value
    let old_len = domains[b].len();
    domains[b].retain(|&vb| check(a_val, vb));
    domains[b].len() != old_len
}

/// Count how many values in neighbor domains would be eliminated if var = val.
fn count_conflicts(
    problem: &ConstraintProblem,
    domains: &[Domain],
    var: usize,
    val: i64,
) -> usize {
    let mut conflicts = 0;
    for c in &problem.constraints {
        if let Binary { a, b, check, .. } = c {
            let other = if *a == var { *b } else if *b == var { *a } else { continue };
            let removed = domains[other].iter().filter(|&&ov| !check(val, ov)).count();
            conflicts += removed;
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csp::{ConstraintProblem, Variable, Constraint, SolverConfig, neq_fn};

    fn simple_problem() -> ConstraintProblem {
        // x != y, x in {1,2}, y in {1,2}
        let vars = vec![
            Variable::new("x", vec![1, 2]),
            Variable::new("y", vec![1, 2]),
        ];
        let cs = vec![Constraint::Binary {
            a: 0, b: 1,
            check: neq_fn,
            desc: "!=",
        }];
        ConstraintProblem::new(vars, cs)
    }

    #[test]
    fn test_solve_bt_basic() {
        let p = simple_problem();
        let result = solve_bt(&p);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_ne!(r[&0], r[&1]);
    }

    #[test]
    fn test_solve_mrv() {
        let p = simple_problem();
        let result = solve_bt_mrv(&p);
        assert!(result.is_some());
    }

    #[test]
    fn test_solve_fc() {
        let p = simple_problem();
        let result = solve_bt_fc(&p);
        assert!(result.is_some());
    }

    #[test]
    fn test_solve_mac() {
        let p = simple_problem();
        let result = solve_bt_mac(&p);
        assert!(result.is_some());
    }

    #[test]
    fn test_unsat() {
        // x != y, x == 1, y == 1
        let vars = vec![
            Variable::new("x", vec![1]),
            Variable::new("y", vec![1]),
        ];
        let cs = vec![Constraint::Binary {
            a: 0, b: 1,
            check: neq_fn,
            desc: "!=",
        }];
        let p = ConstraintProblem::new(vars, cs);
        assert!(solve_bt(&p).is_none());
        assert!(solve_bt_mrv(&p).is_none());
        assert!(solve_bt_fc(&p).is_none());
        assert!(solve_bt_mac(&p).is_none());
    }

    #[test]
    fn test_with_stats() {
        let p = simple_problem();
        let config = SolverConfig::default();
        let (result, stats) = solve_with_stats(&p, &config);
        assert!(result.is_some());
        assert!(stats.nodes_visited > 0);
    }
}
