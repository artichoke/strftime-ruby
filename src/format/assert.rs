#![expect(clippy::indexing_slicing, reason = "const index is always in bounds")]

//! Compile-time assert functions.

/// Helper macro for implementing asserts.
macro_rules! assert_sorted_by_key {
    ($s:expr, $f:expr) => {{
        let mut i = 0;
        while i + 1 < $s.len() {
            assert!(
                *$f(&$s[i]) < *$f(&$s[i + 1]),
                concat!("table is not sorted by ", stringify!($f))
            );
            i += 1;
        }
        $s
    }};
}

/// Returns the first element of a tuple.
const fn elem_0<T>(x: &(u8, T)) -> &u8 {
    &x.0
}

/// Asserts that a slice is sorted and has no duplicates.
pub(crate) const fn assert_sorted(s: &[u8]) -> &[u8] {
    assert_sorted_by_key!(s, core::convert::identity)
}

/// Asserts that a slice is sorted by its first element and has no duplicates.
pub(crate) const fn assert_sorted_elem_0<T>(s: &[(u8, T)]) -> &[(u8, T)] {
    assert_sorted_by_key!(s, elem_0)
}

/// Asserts that converting the first input to uppercase yields the second input.
#[allow(dead_code)]
pub(crate) const fn assert_to_ascii_uppercase(table: &[&str], upper_table: &[&str]) {
    assert!(
        table.len() == upper_table.len(),
        "tables must have the same length"
    );

    let mut index = 0;
    while index < table.len() {
        let (s, upper_s) = (table[index].as_bytes(), upper_table[index].as_bytes());
        assert!(s.len() == upper_s.len(), "keys must have the same length");

        let mut i = 0;
        while i < s.len() {
            assert!(s[i].is_ascii(), "table key must be ascii");
            assert!(upper_s[i].is_ascii(), "upper key must be ascii");
            assert!(
                upper_s[i] == s[i].to_ascii_uppercase(),
                "upper key is not the uppercase of key"
            );
            i += 1;
        }

        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_sorted() {
        assert_sorted(&[1, 2, 3]);
    }

    #[test]
    #[should_panic = "table is not sorted by core::convert::identity"]
    fn test_assert_sorted_invalid() {
        assert_sorted(&[1, 3, 2]);
    }

    #[test]
    fn test_assert_sorted_elem_0() {
        assert_sorted_elem_0(&[(1, 3), (2, 2), (3, 1)]);
    }

    #[test]
    #[should_panic = "table is not sorted by elem_0"]
    fn test_assert_sorted_elem_0_invalid() {
        assert_sorted_elem_0(&[(1, 3), (3, 2), (2, 1)]);
    }

    #[test]
    fn test_assert_to_ascii_uppercase() {
        assert_to_ascii_uppercase(&["aaa"], &["AAA"]);
    }

    #[test]
    #[should_panic = "tables must have the same length"]
    fn assert_to_ascii_uppercase_requires_equal_length_tables() {
        assert_to_ascii_uppercase(&["aaa", "bbb"], &["AAA"]);
    }

    #[test]
    #[should_panic = "upper key is not the uppercase of key"]
    fn test_assert_to_ascii_uppercase_invalid() {
        assert_to_ascii_uppercase(&["aaa"], &["AaA"]);
    }

    #[test]
    #[should_panic = "key must be ascii"]
    fn test_assert_to_ascii_uppercase_invalid_ascii() {
        assert_to_ascii_uppercase(&["€"], &["€"]);
    }

    #[test]
    #[should_panic = "table key must be ascii"]
    fn test_assert_to_ascii_uppercase_invalid_ascii_left() {
        assert_to_ascii_uppercase(&["€"], &["$$$"]);
    }

    #[test]
    #[should_panic = "upper key must be ascii"]
    fn test_assert_to_ascii_uppercase_invalid_ascii_right() {
        assert_to_ascii_uppercase(&["$$$"], &["€"]);
    }
}
