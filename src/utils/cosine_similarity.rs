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
// use packed_simd::u32x8;
//
// pub fn cosine_similarity_simd(v1: &[u32], v2: &[u32]) -> f64 {
//   assert_eq!(
//       v1.len(),
//       v2.len(),
//       "Vectors must have the same length for cosine similarity"
//   );

//   let chunks = v1.len() / 8;
//   let mut dot = 0u64;
//   let mut mag1 = 0u64;
//   let mut mag2 = 0u64;

//   for (chunk1, chunk2) in v1.chunks_exact(8).zip(v2.chunks_exact(8)) {
//       let a = u32x8::from_slice(chunk1);
//       let b = u32x8::from_slice(chunk2);

//       dot += a.wrapping_mul(b).wrapping_sum() as u64;
//       mag1 += a.wrapping_mul(a).wrapping_sum() as u64;
//       mag2 += b.wrapping_mul(b).wrapping_sum() as u64;
//   }

//   let magnitude_v1 = (mag1 as f64).sqrt();
//   let magnitude_v2 = (mag2 as f64).sqrt();

//   if magnitude_v1 == 0.0 || magnitude_v2 == 0.0 {
//       return 0.0;
//   }

//   dot as f64 / (magnitude_v1 * magnitude_v2)
// }
