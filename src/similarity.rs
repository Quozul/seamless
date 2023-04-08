/// Normalized euclidean distance
pub fn similarity(a: &[u8], b: &[u8]) -> f32 {
    if a.len() != b.len() {
        panic!("Arrays are not of the same size, this is currently not supported.");
    }

    let mut sum = 0.0;
    for i in 0..a.len() {
        let diff = (a[i] as f32 - b[i] as f32) / 255.0;
        sum += diff * diff;
    }
    1.0 - (sum / a.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use crate::similarity::similarity;

    #[test]
    fn test_similarity() {
        assert_eq!(similarity(&[255, 255], &[0, 0]), 0.0);
        assert_eq!(similarity(&[255, 255], &[255, 255]), 1.0);

        let test = similarity(&[255, 255], &[128, 127]) * 10.0;
        assert_eq!(test.round(), 5.0);
    }
}
