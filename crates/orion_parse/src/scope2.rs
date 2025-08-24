#[derive(Default)]
pub struct ScopeEval2 {
    beg: char,
    end: char,
    esc_beg: char,
    esc_end: char,
}

enum IterMode {
    Sleep,
    Work,
    Fight,
}
impl ScopeEval2 {
    #[inline(always)]
    pub fn new(beg: char, end: char, esc_beg: char, esc_end: char) -> Self {
        Self {
            beg,
            end,
            esc_beg,
            esc_end,
        }
    }
    #[inline(always)]
    pub fn len(&self, data: &str) -> usize {
        let mut take_len = 0;
        let mut mode = IterMode::Sleep;
        let mut work_level = 0;
        for c in data.chars() {
            match mode {
                IterMode::Sleep => {
                    if c == self.beg {
                        mode = IterMode::Work;
                        work_level += 1;
                        take_len += 1;
                    }
                    break;
                }
                IterMode::Work => {
                    if c == self.end {
                        take_len += 1;
                        work_level -= 1;
                        if work_level == 0 {
                            break;
                        }
                        continue;
                    }
                    if c == self.beg {
                        take_len += 1;
                        work_level += 1;
                        continue;
                    }
                    if c == self.esc_beg {
                        mode = IterMode::Fight;
                        take_len += 1;
                        continue;
                    }
                    take_len += 1;
                }
                IterMode::Fight => {
                    if c == self.esc_end {
                        mode = IterMode::Work;
                        take_len += 1;
                        continue;
                    }
                    take_len += 1;
                }
            }
        }
        take_len
    }
}

#[cfg(test)]
mod tests {
    use crate::scope2::ScopeEval2;
    #[test]
    fn test_scope_debug() {
        let scope_rule = ScopeEval2::new('{', '}', '"', '"');
        let data = r#"{ "a" : "} hello {" }"#;
        let size = scope_rule.len(data);
        assert_eq!(size, 21);

        let data = r#"{ "a" : 123 } {"b" : 234 }"#;
        let size = scope_rule.len(data);
        assert_eq!(size, 13);
        let data = r#" { "a" : 123 } {"b" : 234 }"#;
        let size = scope_rule.len(data);
        assert_eq!(size, 0);

        let data = r#"{ "a" : 123 , "b": { "x" : { "y" :1 }} }"#;
        let size = scope_rule.len(data);
        assert_eq!(size, 40);
    }
}
