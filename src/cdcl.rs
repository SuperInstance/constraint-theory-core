#![allow(missing_docs)]

/// Conflict-Driven Clause Learning (CDCL).
///
/// Simplified CDCL with 1-UIP learning scheme and non-chronological backtracking.
/// Operates on SAT problems (boolean variables) and can be extended to CSPs.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// A literal is a variable index with sign: positive = true, negative = false.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Lit(pub i64); // signed var: positive = true, negative = false

impl Lit {
    pub fn var(self) -> i64 {
        self.0.abs()
    }
    pub fn sign(self) -> bool {
        self.0 > 0
    }
    pub fn not(self) -> Lit {
        Lit(-self.0)
    }
}

/// A clause is a disjunction of literals.
#[derive(Clone, Debug)]
pub struct Clause {
    pub lits: Vec<Lit>,
    pub learnt: bool,
}

impl Clause {
    pub fn new(lits: Vec<Lit>) -> Self {
        Clause { lits, learnt: false }
    }

    pub fn learnt(lits: Vec<Lit>) -> Self {
        Clause { lits, learnt: true }
    }

    pub fn is_satisfied(&self, assignment: &HashMap<i64, bool>) -> bool {
        self.lits.iter().any(|l| {
            assignment.get(&l.var()) == Some(&l.sign())
        })
    }

    pub fn is_unit(&self, assignment: &HashMap<i64, bool>) -> Option<Lit> {
        let mut unassigned = None;
        for &l in &self.lits {
            match assignment.get(&l.var()) {
                Some(&v) if v == l.sign() => return None, // satisfied
                Some(_) => continue, // falsified
                None => {
                    if unassigned.is_some() {
                        return None; // >1 unassigned
                    }
                    unassigned = Some(l);
                }
            }
        }
        unassigned
    }

    pub fn is_conflict(&self, assignment: &HashMap<i64, bool>) -> bool {
        self.lits.iter().all(|l| {
            assignment.get(&l.var()) == Some(&(!l.sign()))
        })
    }
}

/// A SAT problem: list of clauses.
#[derive(Clone)]
pub struct SATProblem {
    pub clauses: Vec<Clause>,
    num_vars: i64,
}

impl SATProblem {
    pub fn new(clauses: Vec<Clause>) -> Self {
        let num_vars = clauses.iter()
            .flat_map(|c| c.lits.iter().map(|l| l.var()))
            .max()
            .unwrap_or(0);
        SATProblem { clauses, num_vars }
    }

    pub fn num_vars(&self) -> i64 { self.num_vars }
}

/// Decision level tracking for a variable.
#[derive(Clone, Copy, Debug)]
struct Assignment {
    value: bool,
    level: usize,
    antecedent: Option<usize>, // clause index that implied this
}

/// CDCL Solver state.
#[derive(Debug)]
pub struct CDCL {
    pub assignment: HashMap<i64, bool>,
    pub trail: Vec<(i64, bool)>, // (var, value) in order of assignment
    pub trail_lim: Vec<usize>, // decision level boundaries in trail
    pub clauses: Vec<Clause>,
    antecedents: HashMap<i64, usize>, // var -> clause index that implied it
    learnts: Vec<usize>,
}

impl CDCL {
    pub fn new(problem: &SATProblem) -> Self {
        CDCL {
            assignment: HashMap::new(),
            trail: Vec::new(),
            trail_lim: Vec::new(),
            clauses: problem.clauses.clone(),
            antecedents: HashMap::new(),
            learnts: Vec::new(),
        }
    }

    pub fn current_level(&self) -> usize {
        self.trail_lim.len()
    }

    /// Enqueue a literal with reason clause (or None for decision).
    pub fn enqueue(&mut self, lit: Lit, antecedent: Option<usize>) -> bool {
        let var = lit.var();
        if let Some(&existing) = self.assignment.get(&var) {
            if existing == lit.sign() {
                return true; // already satisfied
            } else {
                return false; // conflict
            }
        }
        self.assignment.insert(var, lit.sign());
        self.trail.push((var, lit.sign()));
        if let Some(ci) = antecedent {
            self.antecedents.insert(var, ci);
        }
        true
    }

    /// Decide: pick an unassigned literal and assign it.
    pub fn decide(&mut self) -> Option<Lit> {
        // Find an unassigned variable
        for i in 1..=self.clauses.iter().flat_map(|c| c.lits.iter().map(|l| l.var())).max().unwrap_or(0) {
            if !self.assignment.contains_key(&i) {
                // Decide positive
                self.trail_lim.push(self.trail.len());
                self.enqueue(Lit(i), None);
                return Some(Lit(i));
            }
        }
        None
    }

    /// Unit propagation: find unit clauses and assign them.
    /// Returns Some(conflict clause index) or None.
    pub fn propagate(&mut self) -> Option<usize> {
        let mut changed = true;
        while changed {
            changed = false;
            let n = self.clauses.len();
            for ci in 0..n {
                let sat;
                let conf;
                let unit;
                // Check clause state outside borrow of self
                {
                    let clause = &self.clauses[ci];
                    sat = clause.is_satisfied(&self.assignment);
                    conf = clause.is_conflict(&self.assignment);
                    unit = clause.is_unit(&self.assignment);
                }
                if sat { continue; }
                if conf { return Some(ci); }
                if let Some(lit) = unit {
                    if !self.enqueue(lit, Some(ci)) {
                        return Some(ci);
                    }
                    changed = true;
                }
            }
        }
        None
    }

    /// Analyze conflict: 1-UIP learning scheme.
    /// Returns (learnt clause, backtrack level).
    pub fn analyze(&mut self, conflict_idx: usize) -> (Clause, usize) {
        let mut learnt: Vec<Lit> = Vec::new();
        let mut seen: HashSet<i64> = HashSet::new();
        let mut counter = 0;
        let mut ci = conflict_idx;

        // Start with the conflict clause
        let mut lit_ptr = self.trail.len().saturating_sub(1);

        loop {
            let clause = &self.clauses[ci];
            // Resolve: for each lit in clause except the at-reference...
            for &l in &clause.lits {
                let v = l.var();
                if seen.insert(v) {
                    if self.antecedents.contains_key(&v) {
                        // Only count literals from current decision level
                        let assigned_at = self.trail.iter()
                            .rposition(|&(tv, _)| tv == v)
                            .unwrap_or(0);
                        // Find decision level for this trail position
                        let dl = self.level_of_trail_index(assigned_at);
                        if dl == self.current_level() {
                            counter += 1;
                        }
                        learnt.push(l);
                    } else {
                        // Decision variable at lower level — keep as terminal
                        learnt.push(l);
                    }
                }
            }

            // Find the latest literal in the trail that's part of the conflict
            while lit_ptr > 0 {
                let (v, _) = self.trail[lit_ptr];
                if seen.contains(&v) && self.antecedents.contains_key(&v) {
                    break;
                }
                lit_ptr = lit_ptr.saturating_sub(1);
            }

            if lit_ptr == 0 || counter == 1 {
                // 1-UIP reached
                break;
            }

            // Resolve with antecedent of the latest literal
            let (v, _) = self.trail[lit_ptr];
            ci = *self.antecedents.get(&v).unwrap_or(&0);
            counter -= 1;
            lit_ptr = lit_ptr.saturating_sub(1);
        }

        // Remove current-level literals from learnt clause except the UIP
        let final_lit = if lit_ptr < self.trail.len() {
            Lit(if self.trail[lit_ptr].1 { self.trail[lit_ptr].0 } else { -self.trail[lit_ptr].0 })
        } else {
            learnt[0]
        };
        learnt.retain(|&l| {
            let dl = self.level_of_var(l.var());
            dl < self.current_level() || l == final_lit
        });
        learnt.push(final_lit.not());

        let bt_level = learnt.iter()
            .filter_map(|&l| {
                let dl = self.level_of_var(l.var());
                if dl < self.current_level() { Some(dl) } else { None }
            })
            .max()
            .unwrap_or(0);

        (Clause::learnt(learnt), bt_level)
    }

    fn level_of_var(&self, var: i64) -> usize {
        if let Some(pos) = self.trail.iter().position(|&(v, _)| v == var) {
            self.level_of_trail_index(pos)
        } else {
            0
        }
    }

    fn level_of_trail_index(&self, pos: usize) -> usize {
        for (dl, &limit) in self.trail_lim.iter().enumerate().rev() {
            if pos >= limit {
                return dl + 1;
            }
        }
        0
    }

    /// Backtrack to given decision level.
    pub fn backtrack_to(&mut self, level: usize) {
        while self.current_level() > level {
            let lim = self.trail_lim.pop().unwrap_or(0);
            while self.trail.len() > lim {
                let (v, _) = self.trail.pop().unwrap();
                self.assignment.remove(&v);
                self.antecedents.remove(&v);
            }
        }
    }

    /// Main solve loop.
    pub fn solve(&mut self) -> Option<HashMap<i64, bool>> {
        // Unit propagation at level 0
        if let Some(ci) = self.propagate() {
            let (_learnt, bt) = self.analyze(ci);
            self.backtrack_to(bt);
            if bt == 0 {
                return None; // conflict at level 0
            }
        }

        loop {
            if let Some(conflict) = self.propagate() {
                if self.current_level() == 0 {
                    return None; // unsat
                }
                let (learnt, bt) = self.analyze(conflict);
                self.backtrack_to(bt);
                self.clauses.push(learnt.clone());
                self.learnts.push(self.clauses.len() - 1);
                // BCP on the new learnt clause (it's unit at this point)
                if let Some(lit) = learnt.is_unit(&self.assignment) {
                    self.enqueue(lit, Some(self.clauses.len() - 1));
                } else {
                    return None;
                }
            } else {
                // No conflict — decide
                match self.decide() {
                    Some(_) => continue,
                    None => {
                        // All assigned — check if model is complete
                        let max_var = self.clauses.iter()
                            .flat_map(|c| c.lits.iter().map(|l| l.var()))
                            .max().unwrap_or(0);
                        for i in 1..=max_var {
                            if !self.assignment.contains_key(&i) {
                                self.enqueue(Lit(i), None);
                            }
                        }
                        // Verify
                        for clause in &self.clauses {
                            if !clause.is_satisfied(&self.assignment) {
                                return None; // shouldn't happen
                            }
                        }
                        return Some(self.assignment.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sat() {
        // (x OR y) AND (x OR NOT y)
        let clauses = vec![
            Clause::new(vec![Lit(1), Lit(2)]),
            Clause::new(vec![Lit(1), Lit(-2)]),
        ];
        let p = SATProblem::new(clauses);
        let mut solver = CDCL::new(&p);
        let result = solver.solve();
        assert!(result.is_some());
        assert!(result.unwrap().get(&1) == Some(&true));
    }

    #[test]
    fn test_unsat() {
        // (x) AND (NOT x)
        let clauses = vec![
            Clause::new(vec![Lit(1)]),
            Clause::new(vec![Lit(-1)]),
        ];
        let p = SATProblem::new(clauses);
        let mut solver = CDCL::new(&p);
        let result = solver.solve();
        assert!(result.is_none());
    }

    #[test]
    fn test_three_var() {
        // (x OR y) AND (NOT x OR z) AND (NOT y OR NOT z)
        let clauses = vec![
            Clause::new(vec![Lit(1), Lit(2)]),
            Clause::new(vec![Lit(-1), Lit(3)]),
            Clause::new(vec![Lit(-2), Lit(-3)]),
        ];
        let p = SATProblem::new(clauses);
        let mut solver = CDCL::new(&p);
        let result = solver.solve();
        assert!(result.is_some());
        let r = result.unwrap();
        // Verify all clauses satisfied
        assert!(r[&1] || r[&2]);
        assert!(!r[&1] || r[&3]);
        assert!(!r[&2] || !r[&3]);
    }
}
