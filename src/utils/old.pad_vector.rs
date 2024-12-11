/// Pad a vector with zeros to the desired length
pub fn pad_vector(vector: &[u32], length: usize) -> Vec<u32> {
    let mut padded = vector.to_vec();
    if padded.len() < length {
        padded.resize(length, 0);
    }
    padded
}

pub fn pad_vectors_to_match(v1: &[u32], v2: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let max_length = v1.len().max(v2.len());
    (pad_vector(v1, max_length), pad_vector(v2, max_length))
}
