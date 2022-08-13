macro_rules! assert_sorted_by_key {
    ($s:expr, $f:expr) => {{
        let mut i = 0;
        while i + 1 < $s.len() {
            assert!(*$f(&$s[i]) < *$f(&$s[i + 1]));
            i += 1;
        }
        $s
    }};
}

const fn elem_0<T>(x: &(u8, T)) -> &u8 {
    &x.0
}

pub(crate) const fn assert_sorted(s: &[u8]) -> &[u8] {
    assert_sorted_by_key!(s, core::convert::identity)
}

pub(crate) const fn assert_sorted_elem_0<T>(s: &[(u8, T)]) -> &[(u8, T)] {
    assert_sorted_by_key!(s, elem_0)
}

#[allow(dead_code)]
pub(crate) const fn assert_is_ascii_uppercase(table: &[&str], upper_table: &[&str]) {
    assert!(table.len() == upper_table.len());

    let mut index = 0;
    while index < table.len() {
        let (s, upper_s) = (table[index].as_bytes(), upper_table[index].as_bytes());
        assert!(s.len() == upper_s.len());

        let mut i = 0;
        while i < s.len() {
            assert!(s[i].is_ascii());
            assert!(upper_s[i].is_ascii());
            assert!(upper_s[i] == s[i].to_ascii_uppercase());
            i += 1;
        }

        index += 1;
    }
}
