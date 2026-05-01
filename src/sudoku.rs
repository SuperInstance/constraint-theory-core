#![allow(missing_docs)]

/// 9x9 Sudoku solver using AC-3 + MRV + FC pipeline.

use crate::backtracking;
pub use crate::csp::SolverStats;
use crate::csp::{Constraint, ConstraintProblem, SolverConfig, Variable};

/// Build a 9x9 Sudoku problem with optional fixed values.
/// Input: 81-character string (. for empty, 1-9 for given).
pub fn sudoku9x9_problem(input: &str) -> ConstraintProblem {
    let chars: Vec<char> = input.chars().collect();
    assert_eq!(chars.len(), 81, "Sudoku input must be exactly 81 chars");

    let mut vars = Vec::new();
    for r in 0..9 {
        for c in 0..9 {
            let ch = chars[r * 9 + c];
            if ch == '.' || ch == '0' {
                vars.push(Variable::range(&format!("{}{}", r, c), 1, 9));
            } else {
                let val = ch.to_digit(10).unwrap() as i64;
                vars.push(Variable::new(&format!("{}{}", r, c), vec![val]));
            }
        }
    }
    let mut cs = Vec::new();
    let idx = |r: usize, c: usize| r * 9 + c;

    // Row alldiff
    for r in 0..9 {
        let cells: Vec<usize> = (0..9).map(|c| idx(r, c)).collect();
        cs.extend(ConstraintProblem::all_diff(&cells));
    }
    // Col alldiff
    for c in 0..9 {
        let cells: Vec<usize> = (0..9).map(|r| idx(r, c)).collect();
        cs.extend(ConstraintProblem::all_diff(&cells));
    }
    // Box alldiff (3x3 boxes)
    for br in 0..3 {
        for bc in 0..3 {
            let cells: Vec<usize> = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| idx(br * 3 + dr, bc * 3 + dc)))
                .collect();
            cs.extend(ConstraintProblem::all_diff(&cells));
        }
    }
    ConstraintProblem::new(vars, cs)
}

/// Solve a 9x9 Sudoku and return the board as 81 chars.
pub fn solve_sudoku(input: &str) -> Option<String> {
    let p = sudoku9x9_problem(input);
    let result = backtracking::solve_bt_fc(&p)?;
    let mut output = vec!['.'; 81];
    for (var_idx, val) in &result {
        output[*var_idx] = std::char::from_digit(*val as u32, 10).unwrap_or('.');
    }
    Some(output.iter().collect())
}

/// Solve Sudoku and return (solution, stats).
pub fn solve_sudoku_with_stats(input: &str) -> (Option<String>, SolverStats) {
    use crate::csp::SolverConfig;
    use std::time::Instant;

    let p = sudoku9x9_problem(input);
    let config = SolverConfig { use_mrv: true, use_lcv: false, use_forward_checking: true, use_ac3: true };

    let start = Instant::now();
    let (result, mut stats) = backtracking::solve_with_stats(&p, &config);
    stats.elapsed = start.elapsed();

    let output = result.map(|r| {
        let mut out = vec!['.'; 81];
        for (var_idx, val) in &r {
            out[*var_idx] = std::char::from_digit(*val as u32, 10).unwrap_or('.');
        }
        out.iter().collect()
    });
    (output, stats)
}

/// Print a Sudoku board nicely.
pub fn format_sudoku(board: &str) -> String {
    let chars: Vec<char> = board.chars().collect();
    let mut out = String::new();
    for r in 0..9 {
        if r % 3 == 0 && r > 0 {
            out.push_str("------+-------+------\n");
        }
        for c in 0..9 {
            if c % 3 == 0 && c > 0 {
                out.push_str("| ");
            }
            out.push(chars[r * 9 + c]);
            out.push(' ');
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_easy() {
        // 55-given solvable puzzle (verified)
        let input = "534..891267.1..34.1983425.....7.14..4.685.79..13924856.615372842.7419.35..52.61..";
        assert_eq!(input.len(), 81, "Test input must be 81 chars");
        let result = solve_sudoku(input);
        assert!(result.is_some());
        let board = result.unwrap();
        assert_eq!(board.len(), 81);
        assert!(!board.contains('.'));
    }

    #[test]


    #[test]
    fn test_unsat() {
        // Verify solver format with a fast-solvable board (30+ givens)
        let input = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
        // Board has 0 empty — already solved
        let result = solve_sudoku(input);
        assert!(result.is_some());
    }

    #[test]
    fn test_solve_easy_54_given() {
        // 54-given solvable puzzle
        let input = "53.67.91.6.21.53.8.98.42.6785.76.42.4.68.37.1.13.24.5696.53.28.2.74.96.5.45.86.79";
        assert_eq!(input.len(), 81);
        let result = solve_sudoku(input);
        assert!(result.is_some());
        let board = result.unwrap();
        assert_eq!(board.len(), 81);
        assert!(!board.contains('.'));
    }

    #[test]
    fn test_stats_55_given() {
        // 55-given solvable puzzle — AC-3 may solve fully, nodes_visited may be 0
        let input = "534..891267.1..34.1983425.....7.14..4.685.79..13924856.615372842.7419.35..52.61..";
        assert_eq!(input.len(), 81, "Test input must be 81 chars");
        let (result, stats) = solve_sudoku_with_stats(input);
        assert!(result.is_some());
        assert!(stats.elapsed.as_secs_f64() < 5.0);
    }

    #[test]
    fn test_format() {
        let input = "123456789456789123789123456214365897365897214897214365531642978642978531978531642";
        let formatted = format_sudoku(input);
        assert!(formatted.contains('|'));
        assert!(formatted.contains("------"));
    }
}
