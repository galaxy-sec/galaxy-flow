#[derive(Default)]
pub struct ScopeEval {
    count: i32,
    beg: char,
    end: char,
    in_scope: bool,
    end_last: bool,
}
impl ScopeEval {
    #[inline(always)]
    pub fn new(beg: char, end: char) -> Self {
        ScopeEval {
            count: 0,
            beg,
            end,
            in_scope: false,
            end_last: false,
        }
    }
    #[inline(always)]
    pub fn in_scope(&mut self, i: char) -> bool {
        if self.end_last {
            self.end_last = false;
            self.in_scope = false;
        }
        if self.in_scope {
            if i == self.end {
                self.count -= 1;
                if self.count == 0 {
                    self.end_last = true;
                }
            } else if i == self.beg {
                self.count += 1;
            }
        } else if i == self.beg {
            self.count += 1;
            self.in_scope = true;
        }
        self.in_scope
    }
    #[inline(always)]
    pub fn len(data: &str, beg: char, end: char) -> usize {
        let mut op = ScopeEval::new(beg, end);
        let mut len_size: usize = 0;
        for x in data.chars() {
            if op.in_scope(x) {
                len_size += 1;
            } else {
                break;
            }
        }
        len_size
    }
}

#[cfg(test)]
mod tests {
    use super::ScopeEval;

    #[test]
    fn test_scope() {
        let data = r#"(hello)"#;
        let size = ScopeEval::len(data, '(', ')');
        assert_eq!(size, 7);

        let data = r#"what(hello)"#;
        let size = ScopeEval::len(data, '(', ')');
        assert_eq!(size, 0);

        let data = r#"(what(hello))"#;
        let size = ScopeEval::len(data, '(', ')');
        assert_eq!(size, 13);
    }

    #[test]
    fn test_scope2() {
        let data = r#"(ip(10.0.0.1), ip(10.0.0.10)) => crate(city1) ;
ip(10.0.10.1)  => crate(city2) ;
_  => chars(bj) ;
"#;
        let size = ScopeEval::len(data, '(', ')');
        assert_eq!(size, 29);
    }
}
