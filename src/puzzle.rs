#![allow(missing_docs)]

/// Built-in puzzle definitions and solvers.

use crate::backtracking;
use crate::csp::{Constraint, ConstraintProblem, Constraint::Binary, SolverConfig, SolverStats, Variable};
use std::collections::HashMap;

/// N-Queens diagonal check: |v[i] - v[j]| != |i - j| for all i != j.
/// Fn pointer, no captures needed — row indices come from array position.
pub fn nqueens_diag_check(vals: &[i64]) -> bool {
    for i in 0..vals.len() {
        for j in (i + 1)..vals.len() {
            if (vals[i] - vals[j]).abs() == (i as i64 - j as i64).abs() {
                return false;
            }
        }
    }
    true
}

/// Build an N-Queens problem.
/// Uses binary != for rows and a single Nary for all diagonal checks.
pub fn nqueens_problem(n: usize) -> ConstraintProblem {
    let mut vars = Vec::new();
    let mut cs = Vec::new();
    for i in 0..n {
        vars.push(Variable::range(&format!("Q{}", i), 1, n as i64));
    }
    let idxs: Vec<usize> = (0..n).collect();
    // Row alldiff (binary != constraints)
    cs.extend(ConstraintProblem::all_diff(&idxs));
    // Diagonal: encoded via Nary. Each pair (i,j) has |qi-qj| != |i-j|.
    // Since fn pointer can't capture i,j, we use a single Nary constraint
    // that checks ALL pairs at once. This is O(n^2) but fine for n <= 50.
    cs.push(Constraint::Nary {
        vars: idxs,
        check: nqueens_diag_check,
        desc: "all-diag",
    });
    ConstraintProblem::new(vars, cs)
}

/// Solve N-Queens. Returns the column positions as a HashMap.
pub fn solve_nqueens(n: usize) -> Option<HashMap<usize, i64>> {
    let p = nqueens_problem(n);
    backtracking::solve_bt_fc(&p)
}

/// Count number of solutions for N-Queens (small n).
/// Self-contained backtracker that checks diagonal constraints directly
/// without relying on Nary constraint closures.
pub fn count_nqueens(n: usize) -> u64 {
    let mut count = 0;
    let mut board = vec![0i64; n];
    count_nqueens_bt(n, &mut board, 0, &mut count);
    count
}

fn count_nqueens_bt(n: usize, board: &mut [i64], row: usize, count: &mut u64) {
    if row == n {
        *count += 1;
        return;
    }
    for col in 1..=n as i64 {
        if is_safe(board, row, col) {
            board[row] = col;
            count_nqueens_bt(n, board, row + 1, count);
        }
    }
}

fn is_safe(board: &[i64], row: usize, col: i64) -> bool {
    for r in 0..row {
        if board[r] == col {
            return false;
        }
        if (board[r] - col).abs() == (r as i64 - row as i64).abs() {
            return false;
        }
    }
    true
}

/// Build a 4x4 Sudoku problem.
pub fn sudoku4x4_problem() -> ConstraintProblem {
    let mut vars = Vec::new();
    for r in 0..4 {
        for c in 0..4 {
            vars.push(Variable::range(&format!("{}{}", r, c), 1, 4));
        }
    }
    let mut cs = Vec::new();
    let idx = |r: usize, c: usize| r * 4 + c;

    // Row alldiff
    for r in 0..4 {
        let cells: Vec<usize> = (0..4).map(|c| idx(r, c)).collect();
        cs.extend(ConstraintProblem::all_diff(&cells));
    }
    // Col alldiff
    for c in 0..4 {
        let cells: Vec<usize> = (0..4).map(|r| idx(r, c)).collect();
        cs.extend(ConstraintProblem::all_diff(&cells));
    }
    // Box alldiff (2x2 boxes)
    for br in 0..2 {
        for bc in 0..2 {
            let cells: Vec<usize> = (0..2)
                .flat_map(|dr| (0..2).map(move |dc| idx(br * 2 + dr, bc * 2 + dc)))
                .collect();
            cs.extend(ConstraintProblem::all_diff(&cells));
        }
    }
    ConstraintProblem::new(vars, cs)
}

/// Solve 4x4 Sudoku, return ordered values.
pub fn solve_sudoku4x4() -> Option<Vec<i64>> {
    let p = sudoku4x4_problem();
    let result = backtracking::solve_bt_fc(&p)?;
    let mut vals: Vec<(usize, i64)> = result.into_iter().collect();
    vals.sort_by_key(|&(i, _)| i);
    Some(vals.into_iter().map(|(_, v)| v).collect())
}

fn neq_fn(x: i64, y: i64) -> bool { x != y }
fn diag_fn(x: i64, y: i64) -> bool { true } // placeholder

/// Build a graph k-coloring problem.
pub fn graph_coloring_problem(adjacency: &[(usize, usize)], num_colors: usize) -> ConstraintProblem {
    let max_node = adjacency.iter().flat_map(|&(a, b)| vec![a, b]).max().unwrap_or(0) + 1;
    let mut vars = Vec::new();
    for i in 0..max_node {
        vars.push(Variable::range(&format!("V{}", i), 0, num_colors as i64 - 1));
    }
    let mut cs = Vec::new();
    for &(a, b) in adjacency {
        cs.push(Constraint::Binary {
            a, b,
            check: neq_fn,
            desc: "!=",
        });
    }
    ConstraintProblem::new(vars, cs)
}

/// Solve graph coloring.
pub fn solve_graph_coloring(adjacency: &[(usize, usize)], num_colors: usize) -> Option<HashMap<usize, i64>> {
    let p = graph_coloring_problem(adjacency, num_colors);
    backtracking::solve_bt_fc(&p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nqueens_4() {
        let result = solve_nqueens(4);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.len(), 4);
        let cols: Vec<i64> = (0..4).map(|i| r[&i]).collect();
        let mut sorted = cols.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4]);
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert!((cols[i] - cols[j]).abs() != (i as i64 - j as i64).abs());
            }
        }
    }

    #[test]
    fn test_count_nqueens() {
        assert_eq!(count_nqueens(4), 2);
        assert_eq!(count_nqueens(5), 10);
    }

    #[test]
    fn test_sudoku4x4() {
        let result = solve_sudoku4x4();
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.len(), 16);
        for &v in &r {
            assert!(v >= 1 && v <= 4);
        }
    }

    #[test]
    fn test_graph_3color_k4() {
        let adj: Vec<(usize, usize)> = (0..4)
            .flat_map(|i| ((i + 1)..4).map(move |j| (i, j)))
            .collect();
        let result = solve_graph_coloring(&adj, 4);
        assert!(result.is_some());
        let result3 = solve_graph_coloring(&adj, 3);
        assert!(result3.is_none());
    }

    #[test]
    fn test_nqueens_stats() {
        let p = nqueens_problem(8);
        let config = SolverConfig::default();
        let (_result, stats) = backtracking::solve_with_stats(&p, &config);
        // 8-Queens with AC-3 + MRV + FC. The all-N Nary diag constraint
        // provides no incremental pruning, so more nodes are visited.
        assert!(stats.nodes_visited < 50000);
        assert!(stats.elapsed.as_secs_f64() < 5.0);
    }
}
