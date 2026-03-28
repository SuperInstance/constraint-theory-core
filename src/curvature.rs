//! Ricci Flow and Curvature Computation

/// Ricci flow evolution state
#[derive(Clone)]
pub struct RicciFlow {
    alpha: f32,
    target_curvature: f32,
}

impl RicciFlow {
    /// Create a new Ricci flow evolution
    ///
    /// # Arguments
    ///
    /// * `alpha` - Learning rate for curvature evolution (0.0 to 1.0)
    /// * `target_curvature` - Target curvature to converge toward
    ///
    /// # Example
    ///
    /// ```rust
    /// use constraint_theory_core::curvature::RicciFlow;
    ///
    /// let rf = RicciFlow::new(0.1, 0.0);
    /// ```
    pub fn new(alpha: f32, target_curvature: f32) -> Self {
        Self {
            alpha,
            target_curvature,
        }
    }

    /// Creates a default RicciFlow with alpha=0.1 and target_curvature=0.0
    pub fn with_defaults() -> Self {
        Self::new(0.1, 0.0)
    }

    /// Evolve curvatures toward target using Ricci flow
    ///
    /// # Arguments
    ///
    /// * `curvatures` - Mutable slice of curvature values to evolve
    /// * `steps` - Number of evolution steps to perform
    ///
    /// # Example
    ///
    /// ```rust
    /// use constraint_theory_core::curvature::RicciFlow;
    ///
    /// let mut rf = RicciFlow::new(0.1, 0.0);
    /// let mut curvatures = vec![1.0, 0.5, -0.5];
    /// rf.evolve(&mut curvatures, 10);
    /// ```
    pub fn evolve(&mut self, curvatures: &mut [f32], steps: usize) {
        for _ in 0..steps {
            for c in curvatures.iter_mut() {
                *c += self.alpha * (self.target_curvature - *c);
            }
        }
    }
}

/// Single step of Ricci flow evolution
///
/// # Arguments
///
/// * `curvature` - Current curvature value
/// * `alpha` - Learning rate (0.0 to 1.0)
/// * `target` - Target curvature
///
/// # Returns
///
/// New curvature value after one evolution step
///
/// # Example
///
/// ```rust
/// use constraint_theory_core::curvature::ricci_flow_step;
///
/// let c_new = ricci_flow_step(1.0, 0.1, 0.0);
/// assert_eq!(c_new, 0.9);
/// ```
pub fn ricci_flow_step(curvature: f32, alpha: f32, target: f32) -> f32 {
    curvature + alpha * (target - curvature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ricci_flow() {
        let mut rf = RicciFlow::new(0.1, 0.0);
        let mut curvatures = [1.0, 0.5, -0.5];
        rf.evolve(&mut curvatures, 10);

        for &c in &curvatures {
            assert!(c.abs() < 1.0);
        }
    }

    #[test]
    fn test_ricci_flow_step() {
        let c = 1.0;
        let c_new = ricci_flow_step(c, 0.1, 0.0);
        assert_eq!(c_new, 0.9);
    }
}
