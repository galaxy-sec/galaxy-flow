use crate::cond::{
    CmpSymbolDef, CompareExpress, ExpressEnum, LogicExpress, LogicOP, LogicSymbolDef, RustSymbol,
    SQLSymbol,
};
use std::fmt::Display;

impl<T: Sized + Display, S> Display for LogicExpress<T, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(left) = &self.left {
            match left.as_ref() {
                ExpressEnum::Logic(v) => match v.op() {
                    LogicOP::Not => {
                        write!(f, "{} ", v)?;
                    }
                    _ => {
                        write!(f, "({}) ", v)?;
                    }
                },
                ExpressEnum::Compare(v) => {
                    write!(f, "{} ", v)?;
                }
            }
        }
        match self.right.as_ref() {
            ExpressEnum::Logic(v) => match v.op() {
                LogicOP::Not => {
                    write!(f, "{}{}", S::symbol_logic(self.op()), self.right)
                }
                _ => {
                    write!(f, "{} ({})", S::symbol_logic(self.op()), self.right)
                }
            },
            ExpressEnum::Compare(_v) => {
                write!(f, "{} {}", S::symbol_logic(self.op()), self.right)
            }
        }
    }
}

impl<T: Sized + std::fmt::Display, CS> Display for CompareExpress<T, CS>
where
    CS: CmpSymbolDef,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            CS::symbol_var(self.var_name()),
            CS::symbol_cmp(self.compare_op()),
            self.right_const()
        )
    }
}

impl LogicSymbolDef for SQLSymbol {
    fn symbol_and() -> &'static str {
        "and"
    }

    fn symbol_not() -> &'static str {
        "not"
    }

    fn symbol_or() -> &'static str {
        "or"
    }
}

impl CmpSymbolDef for SQLSymbol {
    fn symbol_eq() -> &'static str {
        "="
    }

    fn symbol_we() -> &'static str {
        "="
    }

    fn symbol_ne() -> &'static str {
        "!="
    }

    fn symbol_ge() -> &'static str {
        ">="
    }

    fn symbol_gt() -> &'static str {
        ">"
    }

    fn symbol_le() -> &'static str {
        "<="
    }

    fn symbol_lt() -> &'static str {
        "<"
    }

    fn symbol_var(name: &str) -> String {
        name.to_string()
    }
}
impl LogicSymbolDef for RustSymbol {
    fn symbol_and() -> &'static str {
        "&&"
    }

    fn symbol_not() -> &'static str {
        "!"
    }

    fn symbol_or() -> &'static str {
        "||"
    }
}
impl CmpSymbolDef for RustSymbol {
    fn symbol_eq() -> &'static str {
        "=="
    }

    fn symbol_we() -> &'static str {
        "=*"
    }

    fn symbol_ne() -> &'static str {
        "!="
    }

    fn symbol_ge() -> &'static str {
        ">="
    }

    fn symbol_gt() -> &'static str {
        ">"
    }

    fn symbol_le() -> &'static str {
        "<="
    }

    fn symbol_lt() -> &'static str {
        "<"
    }
    fn symbol_var(name: &str) -> String {
        format!("${}", name)
    }
}
