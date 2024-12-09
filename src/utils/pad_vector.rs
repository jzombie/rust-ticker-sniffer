/// Pad a vector with zeros to the desired length
pub fn pad_vector(vector: &[u32], length: usize) -> Vec<u32> {
    let mut padded = vector.to_vec();
    if padded.len() < length {
        padded.resize(length, 0);
    }
    padded
}
