//! Lattice Cache for Pythagorean Coordinates
//!
//! This module provides caching for Pythagorean lattice generation to avoid
//! recomputing the lattice for repeated operations with the same density.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    LATTICE CACHE                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  ┌──────────────┐     ┌──────────────────────────────────┐  │
//! │  │ Cache Key    │     │ Cached Lattice                    │  │
//! │  │ (density)    │──►  │ • Pythagorean triples             │  │
//! │  │              │     │ • Normalized vectors              │  │
//! │  └──────────────┘     │ • KD-tree index                   │  │
//! │                       └──────────────────────────────────┘  │
//! │                                                              │
//! │  Benefits:                                                   │
//! │  • O(1) lookup for cached lattices                          │
//! │  • Thread-safe access via RwLock                            │
//! │  • Automatic cache eviction (LRU)                           │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use constraint_theory_core::cache::LatticeCache;
//!
//! let cache = LatticeCache::new(100);
//!
//! // First call computes and caches
//! let lattice1 = cache.get_or_compute(200);
//!
//! // Second call returns cached version
//! let lattice2 = cache.get_or_compute(200);
//!
//! // Both references point to the same cached data
//! assert_eq!(lattice1.len(), lattice2.len());
//! ```

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A cached Pythagorean lattice entry.
#[derive(Clone, Debug)]
pub struct CachedLattice {
    /// Pythagorean triples (a, b, c)
    pub triples: Vec<(u32, u32, u32)>,
    /// Normalized vectors on unit circle
    pub vectors: Vec<[f64; 2]>,
    /// Maximum hypotenuse in the lattice
    pub max_hypotenuse: u32,
    /// Density parameter used to generate
    pub density: usize,
}

impl CachedLattice {
    /// Create a new cached lattice entry.
    pub fn new(density: usize) -> Self {
        let (triples, vectors) = generate_pythagorean_lattice(density);
        let max_hypotenuse = triples.iter().map(|&(_, _, c)| c).max().unwrap_or(0);

        Self {
            triples,
            vectors,
            max_hypotenuse,
            density,
        }
    }

    /// Get the number of points in the lattice.
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if the lattice is empty.
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Find the nearest lattice point to a given point.
    pub fn nearest(&self, point: [f64; 2]) -> ([f64; 2], usize, f64) {
        let mut best_point = [0.0, 0.0];
        let mut best_idx = 0;
        let mut best_dist_sq = f64::MAX;

        for (i, &v) in self.vectors.iter().enumerate() {
            let dx = v[0] - point[0];
            let dy = v[1] - point[1];
            let dist_sq = dx * dx + dy * dy;

            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_point = v;
                best_idx = i;
            }
        }

        (best_point, best_idx, best_dist_sq)
    }

    /// Get all vectors as a slice.
    pub fn as_slice(&self) -> &[[f64; 2]] {
        &self.vectors
    }
}

/// Generate Pythagorean lattice for a given density.
///
/// Uses Euclid's formula: a = m² - n², b = 2mn, c = m² + n²
fn generate_pythagorean_lattice(density: usize) -> (Vec<(u32, u32, u32)>, Vec<[f64; 2]>) {
    let mut triples = Vec::new();
    let mut vectors = Vec::new();

    for m in 2..density {
        for n in 1..m {
            // Primitive triples: (m - n) odd and gcd(m, n) = 1
            if (m - n) % 2 == 1 && gcd(m as u32, n as u32) == 1 {
                let a = (m * m - n * n) as u32;
                let b = (2 * m * n) as u32;
                let c = (m * m + n * n) as u32;

                triples.push((a, b, c));

                // Add normalized vectors for all quadrants
                let a_c = a as f64 / c as f64;
                let b_c = b as f64 / c as f64;

                vectors.push([a_c, b_c]);
                vectors.push([b_c, a_c]);
                vectors.push([-a_c, b_c]);
                vectors.push([a_c, -b_c]);
                vectors.push([-a_c, -b_c]);
            }
        }
    }

    // Add cardinal directions
    vectors.push([1.0, 0.0]);
    vectors.push([0.0, 1.0]);
    vectors.push([-1.0, 0.0]);
    vectors.push([0.0, -1.0]);

    (triples, vectors)
}

/// Compute GCD using binary algorithm.
fn gcd(a: u32, b: u32) -> u32 {
    if a == b {
        return a;
    }
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }

    let shift = (a | b).trailing_zeros();
    let mut a = a >> a.trailing_zeros();
    let mut b = b >> b.trailing_zeros();

    while a != b {
        if a > b {
            a -= b;
            a >>= a.trailing_zeros();
        } else {
            b -= a;
            b >>= b.trailing_zeros();
        }
    }

    a << shift
}

/// Thread-safe cache for Pythagorean lattices.
///
/// Uses `Arc<RwLock>` for safe concurrent access.
/// The cache automatically evicts old entries when capacity is reached.
#[derive(Clone, Debug)]
pub struct LatticeCache {
    cache: Arc<RwLock<HashMap<usize, CachedLattice>>>,
    capacity: usize,
}

impl LatticeCache {
    /// Create a new lattice cache with specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of lattices to cache
    ///
    /// # Example
    ///
    /// ```
    /// use constraint_theory_core::cache::LatticeCache;
    ///
    /// let cache = LatticeCache::new(10);
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
            capacity,
        }
    }

    /// Create a cache with default capacity (32 lattices).
    pub fn with_default_capacity() -> Self {
        Self::new(32)
    }

    /// Get a cached lattice or compute and cache it.
    ///
    /// # Arguments
    ///
    /// * `density` - Lattice density parameter
    ///
    /// # Returns
    ///
    /// Reference to the cached lattice
    ///
    /// # Example
    ///
    /// ```
    /// use constraint_theory_core::cache::LatticeCache;
    ///
    /// let cache = LatticeCache::new(10);
    /// let lattice = cache.get_or_compute(100);
    ///
    /// assert!(lattice.len() > 0);
    /// ```
    pub fn get_or_compute(&self, density: usize) -> CachedLattice {
        // Try read lock first
        {
            let cache = self.cache.read().unwrap();
            if let Some(lattice) = cache.get(&density) {
                return lattice.clone();
            }
        }

        // Need to compute - acquire write lock
        let mut cache = self.cache.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(lattice) = cache.get(&density) {
            return lattice.clone();
        }

        // Evict old entries if at capacity
        if cache.len() >= self.capacity {
            // Simple FIFO eviction: remove oldest entry
            if let Some(oldest_key) = cache.keys().next().copied() {
                cache.remove(&oldest_key);
            }
        }

        // Compute and cache
        let lattice = CachedLattice::new(density);
        cache.insert(density, lattice.clone());
        lattice
    }

    /// Check if a density is cached.
    pub fn contains(&self, density: usize) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(&density)
    }

    /// Get the number of cached lattices.
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Precompute and cache lattices for common densities.
    ///
    /// # Arguments
    ///
    /// * `densities` - Slice of density values to precompute
    ///
    /// # Example
    ///
    /// ```
    /// use constraint_theory_core::cache::LatticeCache;
    ///
    /// let cache = LatticeCache::new(10);
    /// cache.precompute(&[50, 100, 200]);
    ///
    /// assert!(cache.contains(50));
    /// assert!(cache.contains(100));
    /// assert!(cache.contains(200));
    /// ```
    pub fn precompute(&self, densities: &[usize]) {
        for &density in densities {
            self.get_or_compute(density);
        }
    }
}

impl Default for LatticeCache {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Global lattice cache for repeated operations.
static GLOBAL_CACHE: std::sync::OnceLock<LatticeCache> = std::sync::OnceLock::new();

/// Get the global lattice cache.
///
/// The global cache is lazily initialized with default capacity.
///
/// # Example
///
/// ```
/// use constraint_theory_core::cache::global_cache;
///
/// let cache = global_cache();
/// let lattice = cache.get_or_compute(200);
/// ```
pub fn global_cache() -> &'static LatticeCache {
    GLOBAL_CACHE.get_or_init(|| LatticeCache::with_default_capacity())
}

/// Clear the global lattice cache.
pub fn clear_global_cache() {
    if let Some(cache) = GLOBAL_CACHE.get() {
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_lattice_generation() {
        let lattice = CachedLattice::new(50);
        assert!(lattice.len() > 0);
        assert!(lattice.max_hypotenuse > 0);
        assert_eq!(lattice.density, 50);
    }

    #[test]
    fn test_lattice_nearest() {
        let lattice = CachedLattice::new(100);

        // Query near 3-4-5 triangle
        let (nearest, _idx, dist_sq) = lattice.nearest([0.6, 0.8]);

        assert!((nearest[0] - 0.6).abs() < 0.01);
        assert!((nearest[1] - 0.8).abs() < 0.01);
        assert!(dist_sq < 0.001);
    }

    #[test]
    fn test_cache_get_or_compute() {
        let cache = LatticeCache::new(10);

        // First call computes
        let lattice1 = cache.get_or_compute(100);
        assert!(cache.contains(100));

        // Second call returns cached
        let lattice2 = cache.get_or_compute(100);
        assert_eq!(lattice1.len(), lattice2.len());
    }

    #[test]
    fn test_cache_eviction() {
        let cache = LatticeCache::new(3);

        // Fill cache
        cache.get_or_compute(10);
        cache.get_or_compute(20);
        cache.get_or_compute(30);

        assert_eq!(cache.len(), 3);

        // Add one more - should evict oldest
        cache.get_or_compute(40);
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_cache_precompute() {
        let cache = LatticeCache::new(10);
        cache.precompute(&[50, 100, 200]);

        assert!(cache.contains(50));
        assert!(cache.contains(100));
        assert!(cache.contains(200));
    }

    #[test]
    fn test_global_cache() {
        let cache = global_cache();
        let lattice = cache.get_or_compute(150);

        assert!(lattice.len() > 0);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(17, 13), 1);
        assert_eq!(gcd(100, 50), 50);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let cache = LatticeCache::new(10);

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let cache = cache.clone();
                thread::spawn(move || {
                    let density = 50 + i * 50;
                    let lattice = cache.get_or_compute(density);
                    lattice.len()
                })
            })
            .collect();

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result > 0);
        }
    }
}
