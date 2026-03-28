//! Sheaf Cohomology Computation
//!
//! This module provides fast computation of sheaf cohomology groups
//! for cellular complexes, focusing on H0 (connected components) and
//! H1 (loops/cycles) dimensions.

/// Result of sheaf cohomology computation
///
/// Contains the dimensions of cohomology groups H0 and H1, along with
/// the complex topology information.
#[derive(Clone, Copy, Debug)]
pub struct CohomologyResult {
    /// Dimension of H0 cohomology group (number of connected components)
    pub h0_dim: usize,
    /// Dimension of H1 cohomology group (number of independent cycles)
    pub h1_dim: usize,
    /// Total number of vertices in the complex
    pub n_vertices: usize,
    /// Total number of edges in the complex
    pub n_edges: usize,
}

/// Fast cohomology computation for cellular complexes
///
/// Uses Euler characteristic and component count to compute
/// cohomology group dimensions in O(1) time.
pub struct FastCohomology;

impl FastCohomology {
    /// Compute cohomology groups for a cellular complex
    ///
    /// # Arguments
    ///
    /// * `n_vertices` - Number of vertices in the complex
    /// * `n_edges` - Number of edges in the complex
    /// * `n_components` - Number of connected components
    ///
    /// # Returns
    ///
    /// Cohomology result with H0 and H1 dimensions
    ///
    /// # Example
    ///
    /// ```rust
    /// use constraint_theory_core::cohomology::FastCohomology;
    ///
    /// let result = FastCohomology::compute(10, 15, 1);
    /// assert_eq!(result.h0_dim, 1);
    /// assert_eq!(result.h1_dim, 6);
    /// ```
    pub fn compute(n_vertices: usize, n_edges: usize, n_components: usize) -> CohomologyResult {
        let h0_dim = n_components;

        let h1_dim = if n_edges >= n_vertices {
            n_edges - n_vertices + n_components
        } else {
            0
        };

        CohomologyResult {
            h0_dim,
            h1_dim,
            n_vertices,
            n_edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohomology() {
        let result = FastCohomology::compute(10, 15, 1);
        assert_eq!(result.h0_dim, 1);
        assert_eq!(result.h1_dim, 6);
    }
}
