use std::collections::HashSet;
use std::hash::Hash;

/// Deduplicates a vector while maintaining the original order.
///
/// # Arguments
/// * `vec` - A vector containing elements to deduplicate.
///
/// # Returns
/// A new vector with duplicates removed, preserving the original order.
pub fn dedup_vector<T: Eq + Hash + Clone>(vec: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    vec.iter()
        .filter_map(|item| {
            if seen.insert(item) {
                Some(item.clone())
            } else {
                None
            }
        })
        .collect()
}
