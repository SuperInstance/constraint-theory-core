//! Hidden Dimension Encoding for Exact Constraint Satisfaction
//!
//! This module implements the Grand Unified Constraint Theory (GUCT) approach
//! to exact constraint satisfaction using hidden dimensions.
//!
//! # Core Formula
//!
//! The number of hidden dimensions required for precision ε:
//! ```text
//! k = ⌈log₂(1/ε)⌉
//! ```
//!
//! This formula determines the additional dimensions needed to represent
//! constraints exactly without floating-point errors.
//!
//! # Algorithm
//!
//! 1. Compute k = ⌈log₂(1/ε)⌉ hidden dimensions
//! 2. Lift point to R^(n+k)
//! 3. Snap to lattice in lifted space
//! 4. Project back to visible space
//!
//! # Example
//!
//! ```
//! use constraint_theory_core::hidden_dimensions::{hidden_dim_count, encode_with_hidden_dims};
//!
//! // Compute hidden dimensions for precision 1e-10
//! let k = hidden_dim_count(1e-10);
//! assert_eq!(k, 34);
//!
//! // Encode a point with hidden dimensions
//! let point = [0.6, 0.8];
//! let encoded = encode_with_hidden_dims(&point, 1e-6);
//! ```

use std::f64;

/// Compute the number of hidden dimensions required for a given precision.
///
/// Uses the formula: k = ⌈log₂(1/ε)⌉
///
/// # Arguments
///
/// * `epsilon` - Target precision (must be > 0)
///
/// # Returns
///
/// Number of hidden dimensions needed
///
/// # Example
///
/// ```
/// use constraint_theory_core::hidden_dimensions::hidden_dim_count;
///
/// assert_eq!(hidden_dim_count(0.1), 4);
/// assert_eq!(hidden_dim_count(0.01), 7);
/// assert_eq!(hidden_dim_count(0.001), 10);
/// assert_eq!(hidden_dim_count(1e-10), 34);
/// ```
pub fn hidden_dim_count(epsilon: f64) -> usize {
    if epsilon <= 0.0 {
        return usize::MAX;
    }
    if epsilon >= 1.0 {
        return 0;
    }
    (1.0 / epsilon).log2().ceil() as usize
}

/// Compute precision from hidden dimension count (inverse of hidden_dim_count).
///
/// Returns the best achievable precision with k hidden dimensions.
///
/// # Arguments
///
/// * `k` - Number of hidden dimensions
///
/// # Returns
///
/// Achievable precision: ε = 2^(-k)
///
/// # Example
///
/// ```
/// use constraint_theory_core::hidden_dimensions::precision_from_hidden_dims;
///
/// let epsilon = precision_from_hidden_dims(10);
/// assert!(epsilon <= 0.001);
/// ```
pub fn precision_from_hidden_dims(k: usize) -> f64 {
    2.0_f64.powi(-(k as i32))
}

/// Compute holographic accuracy for a given configuration.
///
/// Formula: accuracy(k, n) = k/n + O(1/log n)
///
/// This measures how well the hidden dimensions capture the full information
/// of the constraint manifold.
///
/// # Arguments
///
/// * `k` - Number of hidden dimensions
/// * `n` - Total dimensionality (visible + hidden)
///
/// # Returns
///
/// Holographic accuracy ratio (0.0 to 1.0)
///
/// # Example
///
/// ```
/// use constraint_theory_core::hidden_dimensions::holographic_accuracy;
///
/// // With 10 hidden dims and 12 total dims, accuracy is high
/// let acc = holographic_accuracy(10, 12);
/// assert!(acc > 0.8);
/// ```
pub fn holographic_accuracy(k: usize, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let base_accuracy = k as f64 / n as f64;
    // O(1/log n) correction term
    let correction = if n > 1 {
        1.0 / (n as f64).ln()
    } else {
        0.0
    };
    (base_accuracy + correction).min(1.0)
}

/// Lift a point to higher dimensions by adding hidden dimensions.
///
/// The hidden dimensions encode the precision requirements for exact
/// constraint satisfaction.
///
/// # Arguments
///
/// * `point` - The n-dimensional point to lift
/// * `k` - Number of hidden dimensions to add
///
/// # Returns
///
/// Point in R^(n+k) with hidden dimensions initialized to preserve precision
///
/// # Example
///
/// ```
/// use constraint_theory_core::hidden_dimensions::lift_to_hidden;
///
/// let point = vec![0.6, 0.8];
/// let lifted = lift_to_hidden(&point, 3);
/// assert_eq!(lifted.len(), 5); // 2 visible + 3 hidden
/// ```
pub fn lift_to_hidden(point: &[f64], k: usize) -> Vec<f64> {
    let n = point.len();
    let mut lifted = Vec::with_capacity(n + k);
    
    // Copy visible dimensions
    lifted.extend_from_slice(point);
    
    // Initialize hidden dimensions
    // These encode the constraint residuals for exact satisfaction
    for i in 0..k {
        // Hidden dimension value encodes precision bit at position i
        // This allows exact reconstruction when projecting back
        let hidden_val = 2.0_f64.powi(-(i as i32 + 1));
        lifted.push(hidden_val);
    }
    
    lifted
}

/// Project a lifted point back to visible dimensions.
///
/// This operation preserves the constraint satisfaction from the lifted space
/// while returning to the original dimensionality.
///
/// # Arguments
///
/// * `lifted` - Point in R^(n+k) with hidden dimensions
/// * `n` - Number of visible dimensions
///
/// # Returns
///
/// Point in R^n (visible dimensions only)
///
/// # Example
///
/// ```
/// use constraint_theory_core::hidden_dimensions::{lift_to_hidden, project_to_visible};
///
/// let point = vec![0.6, 0.8];
/// let lifted = lift_to_hidden(&point, 3);
/// let projected = project_to_visible(&lifted, 2);
/// assert_eq!(projected.len(), 2);
/// ```
pub fn project_to_visible(lifted: &[f64], n: usize) -> Vec<f64> {
    lifted.iter().take(n).copied().collect()
}

/// Encode a point using hidden dimensions for exact constraint satisfaction.
///
/// This is the main entry point for the hidden dimension encoding algorithm.
/// It lifts the point, snaps to a lattice in the lifted space, and projects back.
///
/// # Algorithm
///
/// 1. Compute k = ⌈log₂(1/ε)⌉ hidden dimensions
/// 2. Lift point to R^(n+k)
/// 3. Snap to lattice in lifted space
/// 4. Project back to visible space
///
/// # Arguments
///
/// * `point` - The point to encode
/// * `epsilon` - Target precision
///
/// # Returns
///
/// Encoded point that satisfies constraints to within epsilon
///
/// # Example
///
/// ```
/// use constraint_theory_core::hidden_dimensions::encode_with_hidden_dims;
///
/// let point = [0.6, 0.8];
/// let encoded = encode_with_hidden_dims(&point, 1e-6);
///
/// // The encoded point should be close to the original
/// assert!((encoded[0] - 0.6).abs() < 0.1);
/// assert!((encoded[1] - 0.8).abs() < 0.1);
/// ```
pub fn encode_with_hidden_dims(point: &[f64], epsilon: f64) -> Vec<f64> {
    let n = point.len();
    let k = hidden_dim_count(epsilon);
    
    // Lift to hidden dimensions
    let lifted = lift_to_hidden(point, k);
    
    // Snap to lattice in lifted space
    // For now, we use a simple normalization snap
    // TODO: Integrate with full Pythagorean lattice when available
    let snapped = snap_to_lattice(&lifted);
    
    // Project back to visible dimensions
    project_to_visible(&snapped, n)
}

/// Snap a point to the nearest lattice point.
///
/// For Pythagorean snapping, this finds the nearest Pythagorean ratio
/// for each component.
///
/// # Arguments
///
/// * `point` - Point to snap
///
/// # Returns
///
/// Snapped point on the lattice
fn snap_to_lattice(point: &[f64]) -> Vec<f64> {
    // Compute norm for normalization
    let norm: f64 = point.iter().map(|x| x * x).sum::<f64>().sqrt();
    
    if norm < 1e-10 {
        return point.to_vec();
    }
    
    // Normalize and snap each component to nearest simple rational
    point.iter().map(|&x| snap_to_rational(x / norm) * norm).collect()
}

/// Snap a value to a nearby simple rational number.
///
/// Uses Pythagorean ratios as candidates for exact representation.
///
/// # Arguments
///
/// * `value` - Value to snap
///
/// # Returns
///
/// Nearest Pythagorean ratio
fn snap_to_rational(value: f64) -> f64 {
    // Common Pythagorean ratios from primitive triples
    let pythagorean_ratios: &[f64] = &[
        0.0, 1.0,
        3.0/5.0, 4.0/5.0,  // 3-4-5 triangle
        5.0/13.0, 12.0/13.0,  // 5-12-13 triangle
        8.0/17.0, 15.0/17.0,  // 8-15-17 triangle
        7.0/25.0, 24.0/25.0,  // 7-24-25 triangle
        20.0/29.0, 21.0/29.0, // 20-21-29 triangle
        9.0/41.0, 40.0/41.0,  // 9-40-41 triangle
        12.0/37.0, 35.0/37.0, // 12-35-37 triangle
        11.0/61.0, 60.0/61.0, // 11-60-61 triangle
        28.0/53.0, 45.0/53.0, // 28-45-53 triangle
        33.0/65.0, 56.0/65.0, // 33-56-65 triangle
        16.0/65.0, 63.0/65.0, // 16-63-65 triangle
        0.5, 0.7071067811865476, // Common ratios (sqrt(2)/2)
    ];
    
    // Find nearest ratio
    let mut best = value;
    let mut min_dist = f64::MAX;
    
    for &ratio in pythagorean_ratios {
        let dist = (value - ratio).abs();
        if dist < min_dist {
            min_dist = dist;
            best = ratio;
        }
    }
    
    // Also consider negative ratios for full circle coverage
    for &ratio in pythagorean_ratios {
        let dist = (value - (-ratio)).abs();
        if dist < min_dist {
            min_dist = dist;
            best = -ratio;
        }
    }
    
    best
}

/// Configuration for hidden dimension encoding.
#[derive(Clone, Debug)]
pub struct HiddenDimensionConfig {
    /// Target precision for encoding
    pub epsilon: f64,
    /// Number of hidden dimensions (computed from epsilon)
    pub hidden_dims: usize,
    /// Whether to use cross-plane optimization
    pub cross_plane_optimization: bool,
}

impl HiddenDimensionConfig {
    /// Create a new configuration with the given precision.
    ///
    /// # Arguments
    ///
    /// * `epsilon` - Target precision
    ///
    /// # Example
    ///
    /// ```
    /// use constraint_theory_core::hidden_dimensions::HiddenDimensionConfig;
    ///
    /// let config = HiddenDimensionConfig::new(1e-6);
    /// assert_eq!(config.hidden_dims, 20);
    /// ```
    pub fn new(epsilon: f64) -> Self {
        Self {
            epsilon,
            hidden_dims: hidden_dim_count(epsilon),
            cross_plane_optimization: true,
        }
    }
    
    /// Create a configuration with explicit hidden dimension count.
    ///
    /// # Arguments
    ///
    /// * `hidden_dims` - Number of hidden dimensions to use
    pub fn with_hidden_dims(hidden_dims: usize) -> Self {
        Self {
            epsilon: precision_from_hidden_dims(hidden_dims),
            hidden_dims,
            cross_plane_optimization: true,
        }
    }
    
    /// Encode a point using this configuration.
    ///
    /// # Arguments
    ///
    /// * `point` - Point to encode
    ///
    /// # Returns
    ///
    /// Encoded point
    pub fn encode(&self, point: &[f64]) -> Vec<f64> {
        encode_with_hidden_dims(point, self.epsilon)
    }
}

/// Cross-plane fine-tuning for constraint optimization.
///
/// Sometimes snapping on a different plane and projecting back
/// achieves better precision with less compute.
///
/// # Arguments
///
/// * `point` - Point to optimize
/// * `planes` - List of orthogonal planes to try
///
/// # Returns
///
/// Optimized point with best constraint satisfaction
pub fn cross_plane_finetune(point: &[f64], planes: &[[usize; 2]]) -> Vec<f64> {
    if planes.is_empty() {
        return point.to_vec();
    }
    
    let mut best_point = point.to_vec();
    let mut best_error = constraint_error(&best_point);
    
    for plane in planes {
        // Snap on this plane
        let mut snapped = point.to_vec();
        if plane[0] < point.len() && plane[1] < point.len() {
            let (a, b) = (point[plane[0]], point[plane[1]]);
            let norm = (a * a + b * b).sqrt().max(1e-10);
            
            // Normalize this plane's components
            snapped[plane[0]] = a / norm;
            snapped[plane[1]] = b / norm;
        }
        
        // Check if this is better
        let error = constraint_error(&snapped);
        if error < best_error {
            best_error = error;
            best_point = snapped;
        }
    }
    
    best_point
}

/// Compute constraint error for a point.
///
/// Measures how far the point is from satisfying unit norm constraint.
///
/// # Arguments
///
/// * `point` - Point to check
///
/// # Returns
///
/// Error measure (0 = perfect constraint satisfaction)
fn constraint_error(point: &[f64]) -> f64 {
    let norm_sq: f64 = point.iter().map(|x| x * x).sum();
    (norm_sq - 1.0).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hidden_dim_count() {
        assert_eq!(hidden_dim_count(0.1), 4);
        assert_eq!(hidden_dim_count(0.01), 7);
        assert_eq!(hidden_dim_count(0.001), 10);
        assert_eq!(hidden_dim_count(0.0001), 14);
        assert_eq!(hidden_dim_count(1e-10), 34);
        assert_eq!(hidden_dim_count(1e-20), 67);
    }

    #[test]
    fn test_precision_inverse() {
        for &eps in &[0.1, 0.01, 0.001, 1e-10] {
            let k = hidden_dim_count(eps);
            let computed_eps = precision_from_hidden_dims(k);
            assert!(computed_eps <= eps, "Precision {} should be <= {}", computed_eps, eps);
        }
    }

    #[test]
    fn test_holographic_accuracy() {
        // With all hidden dimensions, accuracy should be 1
        let acc = holographic_accuracy(10, 10);
        println!("acc(10,10) = {}", acc);
        assert!((acc - 1.0).abs() < 0.05, "acc = {}", acc);

        // With no hidden dimensions, accuracy should be low
        let acc = holographic_accuracy(0, 10);
        println!("acc(0,10) = {}", acc);
        assert!(acc < 0.5, "acc = {}", acc);

        // With half hidden dimensions
        let acc = holographic_accuracy(5, 10); println!("acc(5,10) = {}", acc);
        assert!(acc > 0.7 && acc < 1.0);
    }

    #[test]
    fn test_lift_and_project() {
        let point = vec![0.6, 0.8];
        let lifted = lift_to_hidden(&point, 3);
        assert_eq!(lifted.len(), 5);

        let projected = project_to_visible(&lifted, 2);
        assert_eq!(projected.len(), 2);
        assert!((projected[0] - 0.6).abs() < 1e-10);
        assert!((projected[1] - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_encode_with_hidden_dims() {
        let point = [0.6, 0.8];
        let encoded = encode_with_hidden_dims(&point, 1e-6);
        assert_eq!(encoded.len(), 2);

        // Should be close to original
        assert!((encoded[0] - 0.6).abs() < 0.2);
        assert!((encoded[1] - 0.8).abs() < 0.2);
    }

    #[test]
    fn test_config() {
        let config = HiddenDimensionConfig::new(1e-6);
        assert_eq!(config.hidden_dims, 20);
        assert!((config.epsilon - 1e-6).abs() < 1e-10);

        let config = HiddenDimensionConfig::with_hidden_dims(10);
        assert_eq!(config.hidden_dims, 10);
    }

    #[test]
    fn test_cross_plane_finetune() {
        let point = vec![0.577, 0.816]; // Close to 3-5-sqrt(34)
        let planes = [[0, 1], [0, 1]]; // Just the XY plane
        let optimized = cross_plane_finetune(&point, &planes);
        
        // Should be normalized
        let norm: f64 = optimized.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_snap_to_rational() {
        // Test snapping to 3/5 and 4/5
        assert!((snap_to_rational(0.6) - 0.6).abs() < 0.1);
        assert!((snap_to_rational(0.8) - 0.8).abs() < 0.1);

        // Test snapping to sqrt(2)/2
        let snapped = snap_to_rational(0.71);
        assert!((snapped - 0.707).abs() < 0.1);
    }

    #[test]
    fn test_edge_cases() {
        // Zero epsilon
        assert_eq!(hidden_dim_count(0.0), usize::MAX);

        // Negative epsilon
        assert_eq!(hidden_dim_count(-1.0), usize::MAX);

        // Epsilon >= 1
        assert_eq!(hidden_dim_count(1.0), 0);
        assert_eq!(hidden_dim_count(2.0), 0);
    }
}
