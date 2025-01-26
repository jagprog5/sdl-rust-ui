#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle_empty_vector() {
        let mut v: Vec<usize> = vec![];
        shuffle(&mut v, 42);
        assert!(
            v.is_empty(),
            "Empty vector should remain empty after shuffle"
        );
    }

    #[test]
    fn test_shuffle_single_element_vector() {
        let mut v = vec![42];
        shuffle(&mut v, 42);
        assert_eq!(
            v,
            vec![42],
            "Single element vector should remain unchanged after shuffle"
        );
    }

    #[test]
    fn test_shuffle_normal_case() {
        let mut v = vec![1, 2, 3, 4, 5];
        shuffle(&mut v, 42);
        assert_eq!(
            v,
            vec![2, 5, 4, 1, 3],
            "Deterministic shuffle result did not match"
        );
    }
}

/// deterministic pseudo random shuffle. don't want to add another dep, so doing
/// this very simple method by hand, via a Linear Congruential Generator + Knuth
pub fn shuffle<T>(v: &mut [T], mut seed: u64) {
    for i in (1..v.len()).rev() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (seed % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
}
