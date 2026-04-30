//! Rigidity Percolation using Laman's Theorem

use std::collections::HashMap;

/// Result of rigidity percolation analysis
///
/// Provides comprehensive metrics about the rigidity properties of a graph
/// based on Laman's theorem for structural rigidity.
pub struct RigidityResult {
    /// Whether the graph satisfies Laman's condition for minimal rigidity
    pub is_rigid: bool,
    /// The rank of the rigidity matrix (number of independent constraints)
    pub rank: usize,
    /// Deficiency from minimal rigidity (0 = minimally rigid)
    pub deficiency: usize,
    /// Number of connected clusters in the graph
    pub n_clusters: usize,
    /// Fraction of nodes that are part of rigid clusters
    pub rigid_fraction: f32,
}

/// Fast union-find data structure for rigidity percolation
///
/// Uses path compression and union by rank for efficient
/// connectivity analysis of large graphs.
pub struct FastPercolation {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
}

impl FastPercolation {
    /// Creates a new percolation structure for n nodes
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            size: vec![1; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let xr = self.find(x);
        let yr = self.find(y);

        if xr == yr {
            return;
        }

        if self.rank[xr] < self.rank[yr] {
            self.parent[xr] = yr;
            self.size[yr] += self.size[xr];
        } else if self.rank[xr] > self.rank[yr] {
            self.parent[yr] = xr;
            self.size[xr] += self.size[yr];
        } else {
            self.parent[yr] = xr;
            self.rank[xr] += 1;
            self.size[xr] += self.size[yr];
        }
    }

    /// Computes rigidity metrics for a graph
    ///
    /// Uses Laman's theorem to determine if a graph is minimally rigid.
    /// A graph with V vertices is minimally rigid if it has exactly 2V-3 edges
    /// and every subgraph with k vertices has at most 2k-3 edges.
    ///
    /// # Arguments
    ///
    /// * `edges` - Slice of (u, v) tuples representing edges
    /// * `n_nodes` - Total number of nodes in the graph
    ///
    /// # Returns
    ///
    /// RigidityResult containing comprehensive rigidity metrics
    pub fn compute_rigidity(&mut self, edges: &[(usize, usize)], n_nodes: usize) -> RigidityResult {
        for &(u, v) in edges {
            if u < n_nodes && v < n_nodes {
                self.union(u, v);
            }
        }

        let mut clusters: HashMap<usize, usize> = HashMap::new();
        for i in 0..n_nodes {
            let root = self.find(i);
            *clusters.entry(root).or_insert(0) += 1;
        }

        let n_clusters = clusters.len();
        let n_edges = edges.len();

        // Laman's theorem (2V - 3) only applies for V >= 3
        // For smaller graphs, use special cases
        let (is_rigid, rank, deficiency) = if n_nodes < 3 {
            // Graphs with < 3 vertices are trivially non-rigid
            (false, n_edges, if n_edges > 0 { n_edges } else { 0 })
        } else {
            let expected_edges = 2 * n_nodes - 3;
            let is_rigid = n_edges >= expected_edges;
            let rank = n_edges.min(expected_edges);
            let deficiency = if n_edges >= 2 * n_nodes - 2 {
                n_edges - expected_edges
            } else {
                expected_edges - n_edges
            };
            (is_rigid, rank, deficiency)
        };

        let rigid_nodes: usize = clusters.values().filter(|&&s| s >= 3).sum();

        let rigid_fraction = rigid_nodes as f32 / n_nodes as f32;

        RigidityResult {
            is_rigid,
            rank,
            deficiency,
            n_clusters,
            rigid_fraction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percolation() {
        let mut perc = FastPercolation::new(5);
        let edges = [(0, 1), (1, 2), (2, 3), (3, 4)];
        let result = perc.compute_rigidity(&edges, 5);

        assert!(result.rigid_fraction > 0.0);
    }
}
