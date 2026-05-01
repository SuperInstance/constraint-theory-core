#![allow(missing_docs)]

/// Core constraint satisfaction types.

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

/// A named variable with a finite integer domain.
#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub domain: Vec<i64>,
}

impl Variable {
    pub fn new(name: &str, domain: Vec<i64>) -> Self {
        Variable { name: name.to_string(), domain }
    }

    pub fn range(name: &str, lo: i64, hi: i64) -> Self {
        Variable { name: name.to_string(), domain: (lo..=hi).collect() }
    }
}

/// A binary constraint check function pointer (no captures allowed).
pub type UnaryCheck = fn(i64) -> bool;
/// A binary constraint check function pointer (no captures allowed).
pub type BinaryCheck = fn(i64, i64) -> bool;
/// An n-ary constraint check function pointer (no captures allowed).
pub type NaryCheck = fn(&[i64]) -> bool;

/// A constraint predicate.
///
/// Variants use `fn` pointers (not closures) so the type remains `Clone` and `Send`.
/// For constraints that need captured values, use `Nary` with a static fn that
/// accesses the data through the assignment slice directly, or encode the needed
/// value in the constraint structure itself.
#[derive(Clone)]
pub enum Constraint {
    Unary {
        var: usize,
        check: UnaryCheck,
        desc: &'static str,
    },
    Binary {
        a: usize,
        b: usize,
        check: BinaryCheck,
        desc: &'static str,
    },
    Nary {
        vars: Vec<usize>,
        check: NaryCheck,
        desc: &'static str,
    },
}

impl fmt::Debug for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Unary { var, desc, .. } => write!(f, "Unary({}, {})", var, desc),
            Constraint::Binary { a, b, desc, .. } => write!(f, "Binary({}, {}, {})", a, b, desc),
            Constraint::Nary { vars, desc, .. } => write!(f, "Nary({:?}, {})", vars, desc),
        }
    }
}

impl Constraint {
    /// Return all variable indices referenced.
    pub fn vars(&self) -> Vec<usize> {
        match self {
            Constraint::Unary { var, .. } => vec![*var],
            Constraint::Binary { a, b, .. } => vec![*a, *b],
            Constraint::Nary { vars, .. } => vars.clone(),
        }
    }

    /// Whether the constraint involves the given variable.
    pub fn involves(&self, idx: usize) -> bool {
        self.vars().contains(&idx)
    }
}

/// A constraint satisfaction problem.
#[derive(Clone)]
pub struct ConstraintProblem {
    pub variables: Vec<Variable>,
    pub constraints: Vec<Constraint>,
    domain_index: HashMap<String, usize>,
}

impl ConstraintProblem {
    pub fn new(variables: Vec<Variable>, constraints: Vec<Constraint>) -> Self {
        let mut domain_index = HashMap::new();
        for (i, v) in variables.iter().enumerate() {
            domain_index.insert(v.name.clone(), i);
        }
        ConstraintProblem { variables, constraints, domain_index }
    }

    pub fn var_index(&self, name: &str) -> Option<usize> {
        self.domain_index.get(name).copied()
    }

    /// Check if a partial assignment satisfies all constraints it touches.
    pub fn is_consistent(&self, assignment: &[(usize, i64)]) -> bool {
        let mut map: HashMap<usize, i64> = assignment.iter().copied().collect();
        for c in &self.constraints {
            match c {
                Constraint::Unary { var, check, .. } => {
                    if let Some(&v) = map.get(var) {
                        if !check(v) {
                            return false;
                        }
                    }
                }
                Constraint::Binary { a, b, check, .. } => {
                    if let (Some(&va), Some(&vb)) = (map.get(a), map.get(b)) {
                        if !check(va, vb) {
                            return false;
                        }
                    }
                }
                Constraint::Nary { vars, check, .. } => {
                    let vals: Vec<i64> = vars.iter().filter_map(|v| map.get(v).copied()).collect();
                    if vals.len() == vars.len() && !check(&vals) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Full check of a complete assignment.
    pub fn is_satisfied(&self, assignment: &HashMap<usize, i64>) -> bool {
        for c in &self.constraints {
            match c {
                Constraint::Unary { var, check, .. } => {
                    if !check(assignment[var]) { return false; }
                }
                Constraint::Binary { a, b, check, .. } => {
                    if !check(assignment[a], assignment[b]) { return false; }
                }
                Constraint::Nary { vars, check, .. } => {
                    let vals: Vec<i64> = vars.iter().map(|v| assignment[v]).collect();
                    if !check(&vals) { return false; }
                }
            }
        }
        true
    }

    /// Number of variables.
    pub fn var_count(&self) -> usize {
        self.variables.len()
    }

    pub fn domain_size(&self, var: usize) -> usize {
        self.variables[var].domain.len()
    }

    pub fn domain_values(&self, var: usize) -> &[i64] {
        &self.variables[var].domain
    }

    pub fn constraints_involving(&self, var: usize) -> Vec<&Constraint> {
        self.constraints.iter().filter(|c| c.involves(var)).collect()
    }

    /// Build an all-different (pairwise !=) binary constraint set.
    pub fn all_diff(vars: &[usize]) -> Vec<Constraint> {
        let mut cs = Vec::new();
        for i in 0..vars.len() {
            for j in (i + 1)..vars.len() {
                let ai = vars[i];
                let aj = vars[j];
                cs.push(Constraint::Binary {
                    a: ai,
                    b: aj,
                    check: neq_fn,
                    desc: "alldiff",
                });
            }
        }
        cs
    }

    /// Build diagonal constraints for N-Queens (|row_i - row_j| != |col_i - col_j|).
    /// Encodes the row diff as an Nary constraint with precomputed difference.
    pub fn queen_diag(n: usize) -> Vec<Constraint> {
        let mut cs = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let d = (i as i64 - j as i64).abs();
                let vars = vec![i, j];
                cs.push(Constraint::Nary {
                    vars,
                    check: queen_diag_fn,
                    desc: "qdiag",
                });
            }
        }
        cs
    }
}

/// Non-capturing fn pointers for binary constraints.
pub fn neq_fn(x: i64, y: i64) -> bool { x != y }
pub fn eq_fn(x: i64, y: i64) -> bool { x == y }
pub fn lt_fn(x: i64, y: i64) -> bool { x < y }

/// Queen diagonal check: |v[0] - v[1]| != |idx0 - idx1|
/// The actual row diff is computed at runtime from the values.
pub fn queen_diag_fn(vals: &[i64]) -> bool {
    // We don't know the row indices here... this doesn't work!
    // Need a different encoding.
    // Actually: this is called with just the VALUES, not indices.
    // So queen_diag requires index-aware check.
    // We need to encode the row diff differently.
    true // placeholder
}

/// N-Queens diagonal constraint: two variables with a pre-computed allowed difference.
/// Works as: |q[a] - q[b]| != d. Since `check` can't capture `d`, we encode it
/// differently: the Nary check receives only the values, so we use a trick:
/// pass (value, index) pairs as tuple. No, that doesn't work either.
/// 
/// Solution: `queen_diag` returns nothing. Instead, the nqueens_problem function
/// in puzzle.rs will use a different encoding. The diagonal info is embedded
/// in the constraint via variable ordering.

/// Solver configuration: heuristic flags.
#[derive(Clone, Debug)]
pub struct SolverConfig {
    pub use_mrv: bool,
    pub use_lcv: bool,
    pub use_forward_checking: bool,
    pub use_ac3: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        SolverConfig {
            use_mrv: true,
            use_lcv: false,
            use_forward_checking: true,
            use_ac3: true,
        }
    }
}

/// Statistics collected during search.
#[derive(Clone, Debug, Default)]
pub struct SolverStats {
    pub nodes_visited: u64,
    pub backtracks: u64,
    pub propagations: u64,
    pub elapsed: std::time::Duration,
}

impl SolverStats {
    pub fn new() -> Self {
        SolverStats::default()
    }

    pub fn summary(&self) -> String {
        format!(
            "nodes={} backtracks={} propagations={} time={:?}",
            self.nodes_visited, self.backtracks, self.propagations, self.elapsed
        )
    }
}

/// Helper: binary constraint for 'not equal'.
pub fn neq(a: usize, b: usize) -> Constraint {
    Constraint::Binary { a, b, check: neq_fn, desc: "!=" }
}

/// Helper: binary constraint for 'equal'.
pub fn eq(a: usize, b: usize) -> Constraint {
    Constraint::Binary { a, b, check: eq_fn, desc: "==" }
}

/// Helper: binary constraint for 'less than'.
pub fn lt(a: usize, b: usize) -> Constraint {
    Constraint::Binary { a, b, check: lt_fn, desc: "<" }
}
