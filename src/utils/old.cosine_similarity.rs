/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(v1: &[u32], v2: &[u32]) -> f64 {
    assert_eq!(
        v1.len(),
        v2.len(),
        "Vectors must have the same length for cosine similarity"
    );

    // Use iterators to reduce redundant passes through the vectors
    let (dot_product, magnitude_v1_sq, magnitude_v2_sq) =
        v1.iter()
            .zip(v2)
            .fold((0u64, 0u64, 0u64), |(dot, mag1, mag2), (&a, &b)| {
                (
                    dot + (a as u64 * b as u64),
                    mag1 + (a as u64 * a as u64),
                    mag2 + (b as u64 * b as u64),
                )
            });

    let magnitude_v1 = (magnitude_v1_sq as f64).sqrt();
    let magnitude_v2 = (magnitude_v2_sq as f64).sqrt();

    if magnitude_v1 == 0.0 || magnitude_v2 == 0.0 {
        return 0.0;
    }

    dot_product as f64 / (magnitude_v1 * magnitude_v2)
}

// TODO: Will this work in WASM?
// use wide::u32x8;
// https://docs.rs/wide/latest/wide/

// /// Computes the cosine similarity between two vectors using SIMD.
// ///
// /// # Arguments
// ///
// /// * `v1` - A slice of unsigned 32-bit integers.
// /// * `v2` - A slice of unsigned 32-bit integers.
// ///
// /// # Returns
// ///
// /// * A floating-point number representing the cosine similarity.
// ///
// /// # Panics
// ///
// /// Panics if the input slices have different lengths.
// pub fn cosine_similarity_simd(v1: &[u32], v2: &[u32]) -> f64 {
//     assert_eq!(
//         v1.len(),
//         v2.len(),
//         "Vectors must have the same length for cosine similarity"
//     );

//     let mut dot = u32x8::splat(0);
//     let mut mag1 = u32x8::splat(0);
//     let mut mag2 = u32x8::splat(0);

//     let chunks = v1.chunks_exact(8);
//     let remainder = chunks.remainder();

//     for (chunk1, chunk2) in chunks.zip(v2.chunks_exact(8)) {
//         let a = u32x8::from_slice_unaligned(chunk1);
//         let b = u32x8::from_slice_unaligned(chunk2);

//         dot += a * b;
//         mag1 += a * a;
//         mag2 += b * b;
//     }

//     let dot_sum = dot.reduce_add() as u64;
//     let mag1_sum = mag1.reduce_add() as u64;
//     let mag2_sum = mag2.reduce_add() as u64;

//     // Process remaining elements
//     let mut dot_rem = 0u64;
//     let mut mag1_rem = 0u64;
//     let mut mag2_rem = 0u64;

//     for (&a, &b) in remainder.iter().zip(v2.iter().skip(v1.len() - remainder.len())) {
//         dot_rem += a as u64 * b as u64;
//         mag1_rem += (a as u64) * (a as u64);
//         mag2_rem += (b as u64) * (b as u64);
//     }

//     let dot_total = dot_sum + dot_rem;
//     let mag1_total = mag1_sum + mag1_rem;
//     let mag2_total = mag2_sum + mag2_rem;

//     let magnitude_v1 = (mag1_total as f64).sqrt();
//     let magnitude_v2 = (mag2_total as f64).sqrt();

//     if magnitude_v1 == 0.0 || magnitude_v2 == 0.0 {
//         return 0.0;
//     }

//     dot_total as f64 / (magnitude_v1 * magnitude_v2)
// }
