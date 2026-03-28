//! KD-tree spatial index for fast constraint state lookup
//!
//! This module provides a KD-tree implementation that allows O(log N) nearest
//! neighbor queries instead of O(N) linear search through all valid states.
//! This is critical for performance when dealing with thousands of states.
//!
//! # Performance
//!
//! - Build: O(N log N) where N is number of states
//! - Query: O(log N) per lookup vs O(N) for linear search
//! - Memory: O(N) for tree storage
//!
//! # Usage
//!
//! ```rust
//! use constraint_theory_core::kdtree::KDTree;
//!
//! let states = vec![[1.0, 0.0], [0.0, 1.0], [0.6, 0.8]];
//! let tree = KDTree::build(&states);
//!
//! let query = [0.59, 0.81];
//! let (nearest, _idx, distance_sq) = tree.nearest(&query).unwrap();
//! assert_eq!(nearest, [0.6, 0.8]);
//! assert!(distance_sq < 0.01);
//! ```

use std::f32;

const MAX_LEAF_SIZE: usize = 16;

/// KD-tree node
#[derive(Debug)]
enum KDNode {
    /// Internal node with split dimension and value
    Internal {
        dimension: usize,
        split_value: f32,
        left: Box<KDTree>,
        right: Box<KDTree>,
    },
    /// Leaf node containing multiple points
    Leaf {
        points: Vec<[f32; 2]>,
        indices: Vec<usize>,
    },
}

/// KD-tree for fast 2D nearest neighbor queries
#[derive(Debug)]
pub struct KDTree {
    root: KDNode,
    size: usize,
}

impl KDTree {
    /// Build a KD-tree from a set of 2D points
    ///
    /// # Arguments
    ///
    /// * `points` - Slice of 2D points to index
    ///
    /// # Returns
    ///
    /// Constructed KD-tree
    ///
    /// # Complexity
    ///
    /// O(N log N) time, O(N) space
    pub fn build(points: &[[f32; 2]]) -> Self {
        let size = points.len();
        let indices: Vec<usize> = (0..size).collect();
        let points_vec = points.to_vec();

        KDTree {
            root: Self::build_recursive(&points_vec, &indices, 0),
            size,
        }
    }

    /// Recursively build KD-tree from points
    fn build_recursive(points: &[[f32; 2]], indices: &[usize], depth: usize) -> KDNode {
        if indices.len() <= MAX_LEAF_SIZE {
            // Create leaf node
            let leaf_points = indices.iter().map(|&i| points[i]).collect();
            KDNode::Leaf {
                points: leaf_points,
                indices: indices.to_vec(),
            }
        } else {
            // Split on dimension (alternating x/y for 2D)
            let dimension = depth % 2;

            // Sort indices by this dimension
            let mut sorted_indices = indices.to_vec();
            sorted_indices.sort_by(|&a, &b| {
                points[a][dimension]
                    .partial_cmp(&points[b][dimension])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Median split
            let median_idx = sorted_indices.len() / 2;
            let median_point_idx = sorted_indices[median_idx];
            let split_value = points[median_point_idx][dimension];

            // Partition into left and right
            let left_indices = &sorted_indices[..median_idx];
            let right_indices = &sorted_indices[median_idx + 1..];

            // Recursively build subtrees
            let left = Self::build_recursive(points, left_indices, depth + 1);
            let right = Self::build_recursive(points, right_indices, depth + 1);

            KDNode::Internal {
                dimension,
                split_value,
                left: Box::new(KDTree {
                    root: left,
                    size: left_indices.len(),
                }),
                right: Box::new(KDTree {
                    root: right,
                    size: right_indices.len(),
                }),
            }
        }
    }

    /// Find nearest neighbor to query point
    ///
    /// # Arguments
    ///
    /// * `query` - 2D query point
    ///
    /// # Returns
    ///
    /// `Some((point, index, distance_sq))` - Nearest point, its index, and squared distance
    /// `None` - Tree is empty
    ///
    /// # Complexity
    ///
    /// O(log N) average case, O(N) worst case
    pub fn nearest(&self, query: &[f32; 2]) -> Option<([f32; 2], usize, f32)> {
        if self.size == 0 {
            return None;
        }

        let mut best_point = [0.0, 0.0];
        let mut best_idx = 0;
        let mut best_dist_sq = f32::MAX;

        self.nearest_recursive(
            &self.root,
            query,
            &mut best_point,
            &mut best_idx,
            &mut best_dist_sq,
            0,
        );

        Some((best_point, best_idx, best_dist_sq))
    }

    /// Recursive nearest neighbor search
    fn nearest_recursive(
        &self,
        node: &KDNode,
        query: &[f32; 2],
        best_point: &mut [f32; 2],
        best_idx: &mut usize,
        best_dist_sq: &mut f32,
        _depth: usize,
    ) {
        match node {
            KDNode::Internal {
                dimension,
                split_value,
                left,
                right,
            } => {
                // Determine which side to search first
                let query_val = query[*dimension];
                let (first, second) = if query_val <= *split_value {
                    (left, right)
                } else {
                    (right, left)
                };

                // Search the preferred side
                self.nearest_recursive(
                    &first.root,
                    query,
                    best_point,
                    best_idx,
                    best_dist_sq,
                    _depth + 1,
                );

                // Check if we need to search the other side
                let dist_to_split_plane = query_val - *split_value;
                if dist_to_split_plane * dist_to_split_plane < *best_dist_sq {
                    // The other side could have a closer point
                    self.nearest_recursive(
                        &second.root,
                        query,
                        best_point,
                        best_idx,
                        best_dist_sq,
                        _depth + 1,
                    );
                }
            }
            KDNode::Leaf { points, indices } => {
                // Linear search through leaf points
                for (i, &point) in points.iter().enumerate() {
                    let dx = point[0] - query[0];
                    let dy = point[1] - query[1];
                    let dist_sq = dx * dx + dy * dy;

                    // Deterministic tie-breaking: prefer lower index when distances are equal
                    // This ensures consensus-critical determinism across platforms
                    if dist_sq < *best_dist_sq || (dist_sq == *best_dist_sq && indices[i] < *best_idx) {
                        *best_dist_sq = dist_sq;
                        *best_point = point;
                        *best_idx = indices[i];
                    }
                }
            }
        }
    }

    /// Find k nearest neighbors to query point
    ///
    /// # Arguments
    ///
    /// * `query` - 2D query point
    /// * `k` - Number of neighbors to find
    ///
    /// # Returns
    ///
    /// Vector of (point, index, distance_sq) tuples, sorted by distance
    ///
    /// # Complexity
    ///
    /// O(log N + k) average case
    pub fn nearest_k(&self, query: &[f32; 2], k: usize) -> Vec<([f32; 2], usize, f32)> {
        if self.size == 0 || k == 0 {
            return Vec::new();
        }

        let mut results = NearestK::new(k);
        self.nearest_k_recursive(&self.root, query, &mut results, 0);

        results.into_sorted()
    }

    /// Recursive k-nearest neighbors search
    fn nearest_k_recursive(
        &self,
        node: &KDNode,
        query: &[f32; 2],
        results: &mut NearestK,
        _depth: usize,
    ) {
        match node {
            KDNode::Internal {
                dimension,
                split_value,
                left,
                right,
            } => {
                let query_val = query[*dimension];
                let (first, second) = if query_val <= *split_value {
                    (left, right)
                } else {
                    (right, left)
                };

                // Search preferred side
                self.nearest_k_recursive(&first.root, query, results, _depth + 1);

                // Check if we need to search other side
                if let Some(worst_dist) = results.worst_distance() {
                    let dist_to_split = query_val - *split_value;
                    if dist_to_split * dist_to_split < worst_dist {
                        self.nearest_k_recursive(&second.root, query, results, _depth + 1);
                    }
                }
            }
            KDNode::Leaf { points, indices } => {
                for (i, &point) in points.iter().enumerate() {
                    let dx = point[0] - query[0];
                    let dy = point[1] - query[1];
                    let dist_sq = dx * dx + dy * dy;

                    results.insert(point, indices[i], dist_sq);
                }
            }
        }
    }

    /// Get number of points in the tree
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

/// Helper structure for maintaining k-nearest neighbors during search
struct NearestK {
    neighbors: Vec<([f32; 2], usize, f32)>,
    k: usize,
}

impl NearestK {
    fn new(k: usize) -> Self {
        NearestK {
            neighbors: Vec::with_capacity(k),
            k,
        }
    }

    fn insert(&mut self, point: [f32; 2], index: usize, dist_sq: f32) {
        if self.neighbors.len() < self.k {
            // Not at capacity yet, just add
            self.neighbors.push((point, index, dist_sq));
            // Keep sorted
            self.neighbors
                .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        } else if let Some(&mut (.., ref mut worst_dist)) = self.neighbors.last_mut() {
            // At capacity, check if this is better than worst
            if dist_sq < *worst_dist {
                // Replace worst with this point
                *self.neighbors.last_mut().unwrap() = (point, index, dist_sq);
                // Re-sort
                self.neighbors
                    .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            }
        }
    }

    fn worst_distance(&self) -> Option<f32> {
        self.neighbors.last().map(|&(_, _, dist)| dist)
    }

    fn into_sorted(self) -> Vec<([f32; 2], usize, f32)> {
        self.neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdtree_build() {
        let points = vec![[1.0, 0.0], [0.0, 1.0], [0.6, 0.8], [0.8, 0.6]];
        let tree = KDTree::build(&points);

        assert_eq!(tree.size(), 4);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_kdtree_nearest() {
        let points = vec![[1.0, 0.0], [0.0, 1.0], [0.6, 0.8], [0.8, 0.6]];
        let tree = KDTree::build(&points);

        // Query near [0.6, 0.8]
        let query = [0.59, 0.81];
        let (nearest, idx, dist_sq) = tree.nearest(&query).unwrap();

        assert_eq!(nearest, [0.6, 0.8]);
        assert_eq!(idx, 2);
        assert!(dist_sq < 0.0005); // Very close
    }

    #[test]
    fn test_kdtree_nearest_k() {
        let points = vec![
            [1.0, 0.0],
            [0.0, 1.0],
            [0.6, 0.8],
            [0.8, 0.6],
            [0.5, 0.5],
            [0.9, 0.9],
        ];
        let tree = KDTree::build(&points);

        let query = [0.55, 0.55];
        let results = tree.nearest_k(&query, 3);

        assert_eq!(results.len(), 3);
        // Verify sorted by distance
        for i in 1..results.len() {
            assert!(results[i - 1].2 <= results[i].2);
        }
    }

    #[test]
    fn test_kdtree_empty() {
        let points: Vec<[f32; 2]> = vec![];
        let tree = KDTree::build(&points);

        assert!(tree.is_empty());
        assert_eq!(tree.size(), 0);
        assert!(tree.nearest(&[0.5, 0.5]).is_none());
    }

    #[test]
    fn test_kdtree_single_point() {
        let points = vec![[0.5, 0.5]];
        let tree = KDTree::build(&points);

        let query = [0.51, 0.49];
        let (nearest, idx, dist_sq) = tree.nearest(&query).unwrap();

        assert_eq!(nearest, [0.5, 0.5]);
        assert_eq!(idx, 0);
        assert!(dist_sq < 0.0002);
    }

    #[test]
    fn test_kdtree_large_random() {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let points: Vec<[f32; 2]> = (0..1000)
            .map(|_| [rng.gen::<f32>(), rng.gen::<f32>()])
            .collect();

        let tree = KDTree::build(&points);
        assert_eq!(tree.size(), 1000);

        // Random query
        let query = [rng.gen::<f32>(), rng.gen::<f32>()];
        let result = tree.nearest(&query);

        assert!(result.is_some());
    }
}
