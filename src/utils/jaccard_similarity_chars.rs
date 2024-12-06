use std::collections::HashSet;

/// Compute the Jaccard similarity between two strings by treating characters as sets.
pub fn jaccard_similarity_chars(s1: &str, s2: &str) -> f32 {
    let set1: HashSet<_> = s1.chars().collect();
    let set2: HashSet<_> = s2.chars().collect();

    let intersection_size = set1.intersection(&set2).count();
    let union_size = set1.union(&set2).count();

    if union_size == 0 {
        0.0 // Avoid division by zero if both sets are empty
    } else {
        intersection_size as f32 / union_size as f32
    }
}
