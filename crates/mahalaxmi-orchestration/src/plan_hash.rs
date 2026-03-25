use sha2::{Digest, Sha256};

/// Compute a deterministic SHA-256 hash of a task plan.
///
/// Sorts `task_descriptions` before hashing so insertion order does not affect
/// the result. Two plans with identical task descriptions in different order
/// produce the same hash. The returned value is a lowercase 64-character hex
/// string.
pub fn plan_hash(task_descriptions: &[&str]) -> String {
    let mut sorted: Vec<&str> = task_descriptions.to_vec();
    sorted.sort_unstable();
    let joined = sorted.join("\n");
    let digest = Sha256::digest(joined.as_bytes());
    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::plan_hash;

    #[test]
    fn same_descriptions_different_order_produce_same_hash() {
        let h1 = plan_hash(&["B", "A"]);
        let h2 = plan_hash(&["A", "B"]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_descriptions_produce_different_hashes() {
        let h1 = plan_hash(&["A"]);
        let h2 = plan_hash(&["B"]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn empty_slice_returns_stable_nonempty_64_char_hex() {
        let h = plan_hash(&[]);
        // SHA-256 of empty string is the well-known constant
        assert_eq!(
            h,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn single_element_slice_works_correctly() {
        let h1 = plan_hash(&["hello"]);
        let h2 = plan_hash(&["hello"]);
        assert_eq!(h1, h2);
        // Must differ from a different single element
        let h3 = plan_hash(&["world"]);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 64);
    }
}
