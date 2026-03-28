//! Constraint Theory Core - High-Performance Geometric Engine
//!
//! This crate provides the core mathematical operations for the SuperInstance
//! Constraint Theory system, implementing the Grand Unified Constraint Theory (GUCT).
//!
//! # Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`manifold`] | Pythagorean snapping with O(log N) KD-tree lookup |
//! | [`hidden_dimensions`] | Exact encoding via k = ⌈log₂(1/ε)⌉ formula |
//! | [`quantizer`] | Constraint-preserving quantization (TurboQuant, BitNet, PolarQuant) |
//! | [`holonomy`] | Consistency verification around cycles |
//! | [`cache`] | Thread-safe lattice caching for performance |
//! | [`kdtree`] | Spatial indexing for fast nearest neighbor queries |
//! | [`simd`] | SIMD-optimized batch processing (AVX2) |
//!
//! # Core Concepts
//!
//! ## Pythagorean Manifold
//!
//! The `PythagoreanManifold` is the primary data structure, representing a discrete
//! set of exact Pythagorean coordinates on the unit circle. It enables deterministic
//! projection of continuous vectors to exact rational ratios.
//!
//! ## Hidden Dimensions Formula
//!
//! The number of hidden dimensions for precision ε:
//! ```text
//! k = ⌈log₂(1/ε)⌉
//! ```
//!
//! This formula determines the additional dimensions needed to represent
//! constraints exactly without floating-point errors.
//!
//! ## Quantization Modes
//!
//! The `PythagoreanQuantizer` supports multiple quantization modes:
//! - **Ternary (BitNet)**: {-1, 0, 1} for LLM weights, 16x memory reduction
//! - **Polar (PolarQuant)**: Exact unit norm preservation for embeddings
//! - **Turbo (TurboQuant)**: Near-optimal distortion for vector databases
//! - **Hybrid**: Auto-select mode based on input characteristics
//!
//! # Performance
//!
//! | Operation | Complexity | Notes |
//! |-----------|------------|-------|
//! | Single snap | O(log N) | KD-tree lookup |
//! | Batch snap | O(n log N) | SIMD optimized |
//! | Holonomy check | O(n²) | Spectral method |
//! | Lattice cache | O(1) | Thread-safe |
//!
//! # Example
//!
//! ```
//! use constraint_theory_core::{PythagoreanManifold, snap};
//!
//! let manifold = PythagoreanManifold::new(200);
//! let vec = [0.6f32, 0.8];
//! let (snapped, noise) = snap(&manifold, vec);
//! assert!(noise < 0.01);
//! ```
//!
//! # Hidden Dimensions Example
//!
//! ```
//! use constraint_theory_core::hidden_dimensions::{hidden_dim_count, lift_to_hidden};
//!
//! // Compute hidden dimensions for precision 1e-10
//! let k = hidden_dim_count(1e-10);
//! assert_eq!(k, 34);
//!
//! // Lift a point to higher dimensions
//! let point = vec![0.6, 0.8];
//! let lifted = lift_to_hidden(&point, k);
//! assert_eq!(lifted.len(), 36); // 2 visible + 34 hidden
//! ```
//!
//! # Quantization Example
//!
//! ```
//! use constraint_theory_core::quantizer::{PythagoreanQuantizer, QuantizationMode};
//!
//! // Create a quantizer for embeddings (unit norm preservation)
//! let quantizer = PythagoreanQuantizer::for_embeddings();
//! let vector = vec![0.6, 0.8, 0.0, 0.0];
//! let result = quantizer.quantize(&vector);
//!
//! // Verify unit norm is preserved
//! assert!(result.check_unit_norm(0.1));
//! ```
//!
//! # SIMD Batch Processing
//!
//! For high-throughput applications, use SIMD batch processing:
//!
//! ```
//! use constraint_theory_core::PythagoreanManifold;
//!
//! let manifold = PythagoreanManifold::new(200);
//! let vectors = vec![[0.6, 0.8], [0.8, 0.6], [0.1, 0.99]];
//! let results = manifold.snap_batch_simd(&vectors);
//!
//! for (snapped, noise) in results {
//!     println!("Snapped: {:?}, Noise: {}", snapped, noise);
//! }
//! ```
//!
//! # Error Handling
//!
//! For consensus-critical applications, validate inputs before snapping:
//!
//! ```
//! use constraint_theory_core::PythagoreanManifold;
//!
//! let manifold = PythagoreanManifold::new(200);
//!
//! // Validate input for consensus-critical code
//! if let Err(reason) = manifold.validate_input([f32::NAN, 0.0]) {
//!     println!("Invalid input: {}", reason);
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `simd`: Enable SIMD optimizations (enabled automatically on supported platforms)

#![deny(missing_docs)]
#![warn(unused_extern_crates)]
#![warn(clippy::all)]

pub mod cache;
pub mod cohomology;
pub mod curvature;
pub mod gauge;
pub mod hidden_dimensions;
pub mod holonomy;
pub mod kdtree;
pub mod manifold;
pub mod percolation;
pub mod quantizer;
pub mod simd;
pub mod tile;

#[cfg(test)]
mod edge_case_tests;

// Re-export key types
pub use cache::{CachedLattice, LatticeCache, global_cache, clear_global_cache};
pub use curvature::{ricci_flow_step, RicciFlow};
pub use hidden_dimensions::{
    hidden_dim_count, holographic_accuracy, lift_to_hidden, precision_from_hidden_dims,
    project_to_visible, HiddenDimensionConfig,
};
pub use holonomy::{compute_holonomy, verify_holonomy, HolonomyChecker, HolonomyResult};
pub use manifold::{snap, PythagoreanManifold, PythagoreanTriple};
pub use percolation::{FastPercolation, RigidityResult};
pub use quantizer::{PythagoreanQuantizer, QuantizationMode, QuantizationResult, Rational};
pub use tile::{ConstraintBlock, Origin, Tile};

/// Core error type for constraint theory operations
///
/// This enum represents all possible errors that can occur during
/// constraint theory operations. All fallible operations return `CTResult<T>`.
///
/// # Error Categories
///
/// - **Input Validation**: `NaNInput`, `InfinityInput`, `ZeroVector`, `InvalidDimension`
/// - **State Errors**: `ManifoldEmpty`, `BufferSizeMismatch`
/// - **Numerical**: `NumericalInstability`, `Overflow`, `DivisionByZero`
///
/// # Example
///
/// ```
/// use constraint_theory_core::{CTErr, CTResult};
///
/// fn process_input(x: f32, y: f32) -> CTResult<([f32; 2], f32)> {
///     if !x.is_finite() || !y.is_finite() {
///         return Err(CTErr::InvalidDimension);
///     }
///     Ok(([x, y], 0.0))
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CTErr {
    /// Invalid input dimension - expected 2D vector
    InvalidDimension,
    /// Manifold not initialized - call `new()` first
    ManifoldEmpty,
    /// Numerical instability detected - input may contain NaN or Infinity
    NumericalInstability,
    /// Input vector is zero length - cannot normalize
    ZeroVector,
    /// Input contains NaN values
    NaNInput,
    /// Input contains Infinity values
    InfinityInput,
    /// Batch size mismatch - input and output buffers have different lengths
    BufferSizeMismatch,
    /// Numerical overflow detected - value exceeds f32::MAX
    Overflow,
    /// Division by zero attempted
    DivisionByZero,
    /// Invalid density parameter - must be positive
    InvalidDensity,
    /// Threshold out of valid range
    InvalidThreshold,
}

impl std::fmt::Display for CTErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimension => write!(f, "Invalid input dimension - expected 2D vector. Provide input as [x, y] where both are finite numbers."),
            Self::ManifoldEmpty => write!(f, "Manifold is empty - initialize with new() before performing operations."),
            Self::NumericalInstability => write!(f, "Numerical instability detected - input values may cause precision loss. Consider normalizing input vectors."),
            Self::ZeroVector => write!(f, "Input vector is zero length - cannot normalize. Provide a non-zero vector [x, y]."),
            Self::NaNInput => write!(f, "Input contains NaN values. Ensure all input values are valid numbers."),
            Self::InfinityInput => write!(f, "Input contains Infinity values. Ensure all input values are finite."),
            Self::BufferSizeMismatch => write!(f, "Input and output buffers have different lengths. Ensure buffers are pre-allocated with matching sizes."),
            Self::Overflow => write!(f, "Numerical overflow detected - computed value exceeds f32::MAX. Consider scaling down input values."),
            Self::DivisionByZero => write!(f, "Division by zero attempted - this is an internal error. Please report this issue."),
            Self::InvalidDensity => write!(f, "Invalid density parameter - must be a positive integer. Recommended range: 50-500."),
            Self::InvalidThreshold => write!(f, "Invalid threshold - must be between 0.0 and 1.0 inclusive."),
        }
    }
}

impl std::error::Error for CTErr {}

/// Result type for constraint theory operations
///
/// This is a type alias for `Result<T, CTErr>` used throughout the library.
///
/// # Example
///
/// ```
/// use constraint_theory_core::{CTResult, CTErr};
///
/// fn fallible_operation() -> CTResult<f32> {
///     Ok(1.0)
/// }
/// ```
pub type CTResult<T> = Result<T, CTErr>;

/// Version information string
///
/// Matches the crate version from Cargo.toml.
///
/// # Example
///
/// ```
/// use constraint_theory_core::VERSION;
/// println!("Constraint Theory Core v{}", VERSION);
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate version as semver components
pub const VERSION_MAJOR: usize = 1;
pub const VERSION_MINOR: usize = 0;
pub const VERSION_PATCH: usize = 1;

/// Hidden dimensions required for target precision
///
/// Computes k = ⌈log₂(1/ε)⌉
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
/// use constraint_theory_core::hidden_dimensions;
///
/// let k = hidden_dimensions(0.01);
/// assert_eq!(k, 7);
/// ```
pub fn hidden_dimensions(epsilon: f32) -> usize {
    if epsilon <= 0.0 {
        return usize::MAX;
    }
    (1.0 / epsilon).log2().ceil() as usize
}

/// Compute maximum angular error for a manifold
///
/// # Arguments
///
/// * `state_count` - Number of valid states in the manifold
///
/// # Returns
///
/// Maximum angular deviation in radians
///
/// # Example
///
/// ```
/// use constraint_theory_core::max_angular_error_for_states;
///
/// let error = max_angular_error_for_states(1000);
/// assert!(error < 0.01);  // ~0.36 degrees
/// ```
pub fn max_angular_error_for_states(state_count: usize) -> f32 {
    if state_count == 0 {
        return std::f32::consts::PI;
    }
    std::f32::consts::PI / state_count as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_accuracy() {
        let manifold = PythagoreanManifold::new(200);

        // Test 3-4-5 triple
        let vec = [0.6f32, 0.8];
        let (snapped, noise) = snap(&manifold, vec);

        assert!(noise < 0.001, "Noise should be near zero for exact triple");
        assert!((snapped[0] - 0.6).abs() < 0.01);
        assert!((snapped[1] - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(VERSION_MAJOR, 1);
        assert_eq!(VERSION_MINOR, 0);
        assert_eq!(VERSION_PATCH, 1);
    }

    #[test]
    fn test_hidden_dimensions() {
        assert_eq!(hidden_dimensions(0.1), 4);
        assert_eq!(hidden_dimensions(0.01), 7);
        assert_eq!(hidden_dimensions(0.001), 10);
        assert_eq!(hidden_dimensions(0.0001), 14);
    }

    #[test]
    fn test_max_angular_error() {
        let error = max_angular_error_for_states(1000);
        assert!(error > 0.0);
        assert!(error < 0.01); // ~0.36 degrees
    }

    #[test]
    fn test_cterr_display() {
        assert!(!CTErr::InvalidDimension.to_string().is_empty());
        assert!(!CTErr::ManifoldEmpty.to_string().is_empty());
        assert!(!CTErr::NumericalInstability.to_string().is_empty());
        assert!(!CTErr::ZeroVector.to_string().is_empty());
        assert!(!CTErr::NaNInput.to_string().is_empty());
        assert!(!CTErr::InfinityInput.to_string().is_empty());
        assert!(!CTErr::BufferSizeMismatch.to_string().is_empty());
        assert!(!CTErr::Overflow.to_string().is_empty());
        assert!(!CTErr::DivisionByZero.to_string().is_empty());
        assert!(!CTErr::InvalidDensity.to_string().is_empty());
        assert!(!CTErr::InvalidThreshold.to_string().is_empty());
    }

    #[test]
    fn test_cterr_actionable_messages() {
        // Verify error messages contain actionable guidance
        let zero_msg = CTErr::ZeroVector.to_string();
        assert!(zero_msg.contains("Provide"), "Error message should suggest action");
        
        let nan_msg = CTErr::NaNInput.to_string();
        assert!(nan_msg.contains("Ensure"), "Error message should suggest action");
        
        let density_msg = CTErr::InvalidDensity.to_string();
        assert!(density_msg.contains("Recommended"), "Error message should provide guidance");
    }
}
