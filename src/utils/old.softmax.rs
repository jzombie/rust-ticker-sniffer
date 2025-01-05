pub fn softmax<T>(x: &[T]) -> Vec<f64>
where
    T: Into<f64> + Copy,
{
    let max_value = x
        .iter()
        .map(|&v| v.into())
        .fold(f64::NEG_INFINITY, f64::max);
    let exp_values: Vec<f64> = x.iter().map(|&v| (v.into() - max_value).exp()).collect();
    let sum_exp: f64 = exp_values.iter().sum();
    exp_values.iter().map(|&v| v / sum_exp).collect()
}
