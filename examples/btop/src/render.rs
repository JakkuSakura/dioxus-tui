pub fn bar_repeat(ch: char, count: usize) -> String {
    std::iter::repeat(ch).take(count).collect()
}

pub fn padded_number(value: &str, width: usize) -> String {
    format!("{value:>width$}", width = width)
}

pub fn pad_right(value: &str, width: usize) -> String {
    format!("{value:<width$}", width = width)
}
