pub struct LineBuilder {
    chars: Vec<char>,
}

impl LineBuilder {
    pub fn new(width: usize) -> Self {
        Self {
            chars: vec![' '; width],
        }
    }

    pub fn set_str(&mut self, x: usize, text: &str) {
        for (idx, ch) in text.chars().enumerate() {
            let pos = x + idx;
            if pos < self.chars.len() {
                self.chars[pos] = ch;
            }
        }
    }

    pub fn set_repeat(&mut self, x: usize, ch: char, count: usize) {
        for i in 0..count {
            let pos = x + i;
            if pos < self.chars.len() {
                self.chars[pos] = ch;
            }
        }
    }

    pub fn set_char(&mut self, x: usize, ch: char) {
        if x < self.chars.len() {
            self.chars[x] = ch;
        }
    }

    pub fn finish(self) -> String {
        self.chars.into_iter().collect()
    }
}

pub fn bar_repeat(ch: char, count: usize) -> String {
    std::iter::repeat(ch).take(count).collect()
}

pub fn padded_number(value: &str, width: usize) -> String {
    format!("{value:>width$}", width = width)
}

pub fn pad_right(value: &str, width: usize) -> String {
    format!("{value:<width$}", width = width)
}
