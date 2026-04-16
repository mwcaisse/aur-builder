#[cfg(test)]
pub fn assert_string_starts_with(expected: &str, actual: &str) {
    if !actual.starts_with(expected) {
        use pretty_assertions::private::CreateComparison;
        ::core::panic!(
            "assertion failed: `(left not start with right)`\n left:  {} \n right: {}\
           \n\
           \n{}\
           \n",
            actual,
            expected,
            (actual, expected).create_comparison()
        )
    }
}
