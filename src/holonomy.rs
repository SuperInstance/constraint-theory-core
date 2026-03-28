//! Holonomy Verification for Constraint Consistency
//!
//! This module implements holonomy computation and verification for ensuring
//! global consistency of constraint satisfaction around cycles.
//!
//! # Core Concept
//!
//! **Holonomy** measures the inconsistency that accumulates when transporting
//! a vector around a closed loop in a curved space. In constraint theory:
//!
//! - **Zero holonomy** = Globally consistent constraints
//! - **Non-zero holonomy** = Inconsistent constraints, need resolution
//!
//! # Mathematical Foundation
//!
//! For a cycle γ, the holonomy is:
//! ```text
//! Hol(γ) = ∮ ∇ - [∇, ∇] dγ
//! ```
//!
//! The holonomy-information relationship:
//! ```text
//! I = -log|Hol(γ)|
//! ```
//!
//! # Example
//!
//! ```
//! use constraint_theory_core::holonomy::{compute_holonomy, verify_holonomy, HolonomyResult};
//!
//! // A cycle of rotations
//! let cycle = vec![
//!     [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], // Identity
//!     [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], // Identity
//! ];
//!
//! let result = compute_holonomy(&cycle);
//! assert!(result.is_identity());
//! ```

use std::f64;

/// A 3x3 rotation matrix representing a transformation.
pub type RotationMatrix = [[f64; 3]; 3];

/// Result of holonomy computation.
#[derive(Clone, Debug)]
pub struct HolonomyResult {
    /// The holonomy matrix (product of all transformations around the cycle)
    pub matrix: RotationMatrix,
    /// Holonomy norm (deviation from identity)
    pub norm: f64,
    /// Information content: I = -log|Hol(γ)|
    pub information: f64,
    /// Whether holonomy is zero (identity matrix within tolerance)
    pub is_identity: bool,
    /// Tolerance used for identity check
    pub tolerance: f64,
}

impl HolonomyResult {
    /// Check if the holonomy is effectively zero (identity matrix).
    pub fn is_identity(&self) -> bool {
        self.is_identity
    }
    
    /// Check if the holonomy is within a custom tolerance.
    pub fn is_within_tolerance(&self, tolerance: f64) -> bool {
        self.norm < tolerance
    }
    
    /// Get the angular deviation from identity (in radians).
    pub fn angular_deviation(&self) -> f64 {
        // Extract angle from rotation matrix
        // For rotation matrix R, trace(R) = 1 + 2*cos(θ)
        let trace = self.matrix[0][0] + self.matrix[1][1] + self.matrix[2][2];
        let cos_angle = ((trace - 1.0) / 2.0).clamp(-1.0, 1.0);
        cos_angle.acos()
    }
}

/// Compute the identity matrix.
pub fn identity_matrix() -> RotationMatrix {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

/// Multiply two 3x3 matrices.
fn matrix_multiply(a: &RotationMatrix, b: &RotationMatrix) -> RotationMatrix {
    let mut result = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

/// Compute the Frobenius norm of a matrix's deviation from identity.
fn deviation_from_identity(matrix: &RotationMatrix) -> f64 {
    let mut sum = 0.0;
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            let diff = matrix[i][j] - expected;
            sum += diff * diff;
        }
    }
    sum.sqrt()
}

/// Compute holonomy around a cycle of transformations.
///
/// The holonomy is the product of all transformations around the cycle.
/// If the constraints are globally consistent, the result should be identity.
///
/// # Arguments
///
/// * `cycle` - Sequence of rotation matrices forming a closed loop
///
/// # Returns
///
/// HolonomyResult containing the holonomy matrix and consistency check
///
/// # Example
///
/// ```
/// use constraint_theory_core::holonomy::{compute_holonomy, identity_matrix};
///
/// let cycle = vec![identity_matrix(), identity_matrix()];
/// let result = compute_holonomy(&cycle);
/// assert!(result.is_identity());
/// ```
pub fn compute_holonomy(cycle: &[RotationMatrix]) -> HolonomyResult {
    let tolerance = 1e-6;
    
    if cycle.is_empty() {
        return HolonomyResult {
            matrix: identity_matrix(),
            norm: 0.0,
            information: f64::INFINITY,
            is_identity: true,
            tolerance,
        };
    }
    
    // Compute product of all rotations around the cycle
    let mut product = identity_matrix();
    for rotation in cycle {
        product = matrix_multiply(&product, rotation);
    }
    
    // Compute norm of deviation from identity
    let norm = deviation_from_identity(&product);
    
    // Compute information: I = -log|Hol(γ)|
    let information = if norm > 0.0 {
        -norm.log2()
    } else {
        f64::INFINITY
    };
    
    // Check if identity within tolerance
    let is_identity = norm < tolerance;
    
    HolonomyResult {
        matrix: product,
        norm,
        information,
        is_identity,
        tolerance,
    }
}

/// Verify that holonomy is zero around all given cycles.
///
/// This is a key consistency check for constraint systems.
/// Global convergence ⇔ zero holonomy around all cycles.
///
/// # Arguments
///
/// * `cycles` - Collection of cycles to check
/// * `tolerance` - Maximum allowed deviation from identity
///
/// # Returns
///
/// `true` if all cycles have zero holonomy
///
/// # Example
///
/// ```
/// use constraint_theory_core::holonomy::{verify_holonomy, identity_matrix};
///
/// let cycles = vec![
///     vec![identity_matrix()],
///     vec![identity_matrix(), identity_matrix()],
/// ];
///
/// assert!(verify_holonomy(&cycles, 1e-6));
/// ```
pub fn verify_holonomy(cycles: &[Vec<RotationMatrix>], tolerance: f64) -> bool {
    cycles.iter().all(|cycle| {
        let result = compute_holonomy(cycle);
        result.norm < tolerance
    })
}

/// Compute holonomy for a cycle specified by edges.
///
/// Each edge is represented as a transformation from one node to the next.
///
/// # Arguments
///
/// * `edges` - List of edge transformations
/// * `closed` - Whether the cycle is explicitly closed
///
/// # Returns
///
/// HolonomyResult for the cycle
pub fn compute_edge_holonomy(edges: &[RotationMatrix], closed: bool) -> HolonomyResult {
    if edges.is_empty() {
        return compute_holonomy(&[]);
    }
    
    let mut cycle = edges.to_vec();
    
    // If not explicitly closed, add the inverse of the first transformation
    // to close the loop
    if !closed && edges.len() > 1 {
        // For a rotation matrix, inverse = transpose
        let first = &edges[0];
        let inverse = transpose(first);
        cycle.push(inverse);
    }
    
    compute_holonomy(&cycle)
}

/// Transpose a 3x3 matrix.
fn transpose(matrix: &RotationMatrix) -> RotationMatrix {
    let mut result = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            result[i][j] = matrix[j][i];
        }
    }
    result
}

/// Holonomy checker for incremental verification.
///
/// Allows building up cycles incrementally and checking holonomy at each step.
#[derive(Clone, Debug)]
pub struct HolonomyChecker {
    /// Current accumulated transformation
    accumulated: RotationMatrix,
    /// Number of transformations applied
    step_count: usize,
    /// Tolerance for identity check
    tolerance: f64,
}

impl HolonomyChecker {
    /// Create a new holonomy checker.
    ///
    /// # Arguments
    ///
    /// * `tolerance` - Tolerance for identity check (default: 1e-6)
    pub fn new(tolerance: f64) -> Self {
        Self {
            accumulated: identity_matrix(),
            step_count: 0,
            tolerance,
        }
    }
    
    /// Create a checker with default tolerance.
    pub fn default_tolerance() -> Self {
        Self::new(1e-6)
    }
    
    /// Apply a transformation step.
    pub fn apply(&mut self, rotation: &RotationMatrix) {
        self.accumulated = matrix_multiply(&self.accumulated, rotation);
        self.step_count += 1;
    }
    
    /// Check current holonomy without closing the cycle.
    pub fn check_partial(&self) -> HolonomyResult {
        let norm = deviation_from_identity(&self.accumulated);
        let information = if norm > 0.0 { -norm.log2() } else { f64::INFINITY };
        
        HolonomyResult {
            matrix: self.accumulated,
            norm,
            information,
            is_identity: norm < self.tolerance,
            tolerance: self.tolerance,
        }
    }
    
    /// Close the cycle and check holonomy.
    ///
    /// This applies the inverse transformation to return to the start.
    pub fn check_closed(&self) -> HolonomyResult {
        // Apply inverse of accumulated to close the cycle
        let inverse = transpose(&self.accumulated);
        let cycle = vec![self.accumulated, inverse];
        compute_holonomy(&cycle)
    }
    
    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.accumulated = identity_matrix();
        self.step_count = 0;
    }
    
    /// Get the number of steps applied.
    pub fn step_count(&self) -> usize {
        self.step_count
    }
}

/// Generate a rotation matrix around the X axis.
pub fn rotation_x(angle: f64) -> RotationMatrix {
    let c = angle.cos();
    let s = angle.sin();
    [
        [1.0, 0.0, 0.0],
        [0.0, c, -s],
        [0.0, s, c],
    ]
}

/// Generate a rotation matrix around the Y axis.
pub fn rotation_y(angle: f64) -> RotationMatrix {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, 0.0, s],
        [0.0, 1.0, 0.0],
        [-s, 0.0, c],
    ]
}

/// Generate a rotation matrix around the Z axis.
pub fn rotation_z(angle: f64) -> RotationMatrix {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, -s, 0.0],
        [s, c, 0.0],
        [0.0, 0.0, 1.0],
    ]
}

/// Generate a rotation matrix from Euler angles (ZYX convention).
pub fn rotation_from_euler(roll: f64, pitch: f64, yaw: f64) -> RotationMatrix {
    let rx = rotation_x(roll);
    let ry = rotation_y(pitch);
    let rz = rotation_z(yaw);
    
    // ZYX convention: R = Rz * Ry * Rx
    let ry_rx = matrix_multiply(&ry, &rx);
    matrix_multiply(&rz, &ry_rx)
}

/// Compute the holonomy of a triangular cycle.
///
/// A common test case for holonomy verification.
///
/// # Arguments
///
/// * `a`, `b`, `c` - Three rotation matrices forming a triangle
///
/// # Returns
///
/// HolonomyResult for the closed triangle
pub fn triangular_holonomy(a: &RotationMatrix, b: &RotationMatrix, c: &RotationMatrix) -> HolonomyResult {
    // Compute a -> b -> c -> a (implicitly closed)
    let ab = matrix_multiply(a, &transpose(b));
    let bc = matrix_multiply(b, &transpose(c));
    let ca = matrix_multiply(c, &transpose(a));
    
    let cycle = vec![ab, bc, ca];
    compute_holonomy(&cycle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_holonomy() {
        let cycle = vec![identity_matrix()];
        let result = compute_holonomy(&cycle);
        assert!(result.is_identity());
        assert_eq!(result.norm, 0.0);
    }

    #[test]
    fn test_double_identity() {
        let cycle = vec![identity_matrix(), identity_matrix()];
        let result = compute_holonomy(&cycle);
        assert!(result.is_identity());
    }

    #[test]
    fn test_rotation_cycle() {
        // 90-degree rotations around different axes
        let rx = rotation_x(std::f64::consts::FRAC_PI_2);
        let ry = rotation_y(std::f64::consts::FRAC_PI_2);
        
        // A cycle that returns to identity
        let cycle = vec![rx.clone(), ry.clone(), ry.clone(), rx.clone()];
        let result = compute_holonomy(&cycle);
        
        // Should be close to identity (allowing numerical error)
        assert!(result.norm < 0.1, "Holonomy norm should be small, got {}", result.norm);
    }

    #[test]
    fn test_full_rotation() {
        // Two 180-degree rotations around the same axis should return to identity
        let rz = rotation_z(std::f64::consts::PI);
        let cycle = vec![rz.clone(), rz];
        let result = compute_holonomy(&cycle);
        
        // Should be identity (within numerical tolerance)
        assert!(result.norm < 0.01, "Full rotation should return to identity");
    }

    #[test]
    fn test_verify_holonomy() {
        let cycles = vec![
            vec![identity_matrix()],
            vec![identity_matrix(), identity_matrix()],
        ];
        assert!(verify_holonomy(&cycles, 1e-6));
    }

    #[test]
    fn test_verify_holonomy_failure() {
        // A cycle that doesn't return to identity
        let rz = rotation_z(std::f64::consts::FRAC_PI_4);
        let cycles = vec![vec![rz]];
        assert!(!verify_holonomy(&cycles, 1e-6));
    }

    #[test]
    fn test_holonomy_checker() {
        let mut checker = HolonomyChecker::default_tolerance();
        
        checker.apply(&identity_matrix());
        checker.apply(&identity_matrix());
        
        let result = checker.check_closed();
        assert!(result.is_identity());
        assert_eq!(checker.step_count(), 2);
    }

    #[test]
    fn test_holonomy_checker_reset() {
        let mut checker = HolonomyChecker::default_tolerance();
        
        checker.apply(&rotation_x(0.1));
        assert_eq!(checker.step_count(), 1);
        
        checker.reset();
        assert_eq!(checker.step_count(), 0);
        
        let result = checker.check_partial();
        assert!(result.is_identity());
    }

    #[test]
    fn test_rotation_from_euler() {
        // Zero rotation should be identity
        let r = rotation_from_euler(0.0, 0.0, 0.0);
        let id = identity_matrix();
        
        for i in 0..3 {
            for j in 0..3 {
                assert!((r[i][j] - id[i][j]).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_triangular_holonomy() {
        let a = identity_matrix();
        let b = identity_matrix();
        let c = identity_matrix();
        
        let result = triangular_holonomy(&a, &b, &c);
        assert!(result.is_identity());
    }

    #[test]
    fn test_angular_deviation() {
        let result = compute_holonomy(&[identity_matrix()]);
        assert_eq!(result.angular_deviation(), 0.0);
        
        // 90-degree rotation
        let rz = rotation_z(std::f64::consts::FRAC_PI_2);
        let result = compute_holonomy(&[rz]);
        let deviation = result.angular_deviation();
        assert!(deviation > 1.4 && deviation < 1.6, "Expected ~π/2, got {}", deviation);
    }

    #[test]
    fn test_information_content() {
        let result = compute_holonomy(&[identity_matrix()]);
        assert!(result.information.is_infinite());
        
        // Non-identity should have finite information
        let rz = rotation_z(0.1);
        let result = compute_holonomy(&[rz]);
        assert!(result.information.is_finite());
    }

    #[test]
    fn test_matrix_multiply() {
        let a = rotation_x(std::f64::consts::FRAC_PI_2);
        let b = rotation_x(-std::f64::consts::FRAC_PI_2);
        
        let ab = matrix_multiply(&a, &b);
        let id = identity_matrix();
        
        for i in 0..3 {
            for j in 0..3 {
                assert!((ab[i][j] - id[i][j]).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_transpose() {
        let r = rotation_y(0.5);
        let rt = transpose(&r);
        let product = matrix_multiply(&r, &rt);
        
        // R * R^T should be identity
        let id = identity_matrix();
        for i in 0..3 {
            for j in 0..3 {
                assert!((product[i][j] - id[i][j]).abs() < 1e-10);
            }
        }
    }
}
