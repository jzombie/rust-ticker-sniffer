pub fn index_difference_similarity(v1: &[u32], v2: &[u32]) -> f64 {
    assert_eq!(v1.len(), v2.len(), "Vectors must have the same length.");

    // If both vectors are equal, take the optimized path
    if v1 == v2 {
        return 1.0;
    }

    let total_elements = v1.len();
    let differing_elements = v1.iter().zip(v2.iter()).filter(|(a, b)| a != b).count();

    1.0 - differing_elements as f64 / total_elements as f64
}
