use derive_getters::Getters;
use std::fmt::Display;
use std::marker::PhantomData;

mod cmp_pri;
mod crator;
mod display;

#[derive(Debug, PartialEq, Clone)]
pub enum LogicOP {
    And,
    Or,
    Not,
}
impl Display for LogicOP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicOP::And => write!(f, "&&"),
            LogicOP::Or => write!(f, "||"),
            LogicOP::Not => write!(f, "!"),
        }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CmpOP {
    // width match =*
    We,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}
impl Display for CmpOP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmpOP::We => write!(f, "=*"),
            CmpOP::Eq => write!(f, "=="),
            CmpOP::Ne => write!(f, "!="),
            CmpOP::Gt => write!(f, ">"),
            CmpOP::Ge => write!(f, ">="),
            CmpOP::Lt => write!(f, "<"),
            CmpOP::Le => write!(f, "<="),
        }
    }
}

pub trait Condition<V> {
    fn is_true(&self, data: &V) -> bool;
}

pub trait ValueGet<T> {
    fn value_get(&self, var: &str) -> Option<&T>;
}
pub trait ValueGet0<T> {
    fn value_get(&self) -> Option<&T>;
}
pub trait ValueMust<T> {
    fn value_must(&self) -> &T;

    fn value_must_mut(&mut self) -> &mut T;
}

pub trait WildMatchAble {
    fn wild_match(&self, other: &Self) -> bool;
}

pub trait CompareAble {
    fn is_support(&self, op: CmpOP) -> bool;
    fn compare(&self, other: &Self, op: &CmpOP) -> bool;
}

impl<T> CompareAble for T
where
    T: PartialOrd + WildMatchAble,
{
    fn is_support(&self, _op: CmpOP) -> bool {
        true
    }
    fn compare(&self, other: &Self, op: &CmpOP) -> bool {
        match op {
            CmpOP::We => self.wild_match(other),
            CmpOP::Eq => *self == *other,
            CmpOP::Ne => *self != *other,
            CmpOP::Gt => *self < *other,
            CmpOP::Ge => *self <= *other,
            CmpOP::Lt => *self > *other,
            CmpOP::Le => *self >= *other,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Getters)]
pub struct LogicExpress<T: Sized, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    _keep1: PhantomData<S>,
    pub op: LogicOP,
    pub left: Option<Box<ExpressEnum<T, S>>>,
    pub right: Box<ExpressEnum<T, S>>,
}

#[derive(Debug, PartialEq, Clone)]

pub enum ExpressEnum<T: Sized, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    Logic(LogicExpress<T, S>),
    Compare(CompareExpress<T, S>),
}
pub trait LogicCrator<T> {
    fn from_or(left: T, right: T) -> Self;
    fn from_and(left: T, right: T) -> Self;
    fn from_not(right: T) -> Self;
}

pub trait CmpCrator<T> {
    fn from_eq<S: Into<String>>(left: S, right: T) -> Self;
    fn from_gt<S: Into<String>>(left: S, right: T) -> Self;
    fn from_lt<S: Into<String>>(left: S, right: T) -> Self;
}

impl<T: Sized + Display, S> Display for ExpressEnum<T, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressEnum::Logic(v) => v.fmt(f),
            ExpressEnum::Compare(v) => v.fmt(f),
        }
    }
}

impl<T, S> LogicExpress<T, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    pub fn new(op: LogicOP, left: Option<ExpressEnum<T, S>>, right: ExpressEnum<T, S>) -> Self {
        Self {
            _keep1: PhantomData,
            op,
            left: left.map(Box::new),
            right: Box::new(right),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Getters)]
pub struct CompareExpress<T: Sized, S: CmpSymbolDef> {
    _keep: PhantomData<S>,
    var_name: String,
    compare_op: CmpOP,
    right_const: T,
}

impl<T, CS> CompareExpress<T, CS>
where
    CS: CmpSymbolDef,
{
    pub fn new<S: Into<String>>(op: CmpOP, name: S, target: T) -> Self {
        Self {
            _keep: PhantomData,
            var_name: name.into(),
            compare_op: op,
            right_const: target,
        }
    }
}

impl<T, V, S> Condition<V> for CompareExpress<T, S>
where
    V: ValueGet<T>,
    T: CompareAble + Sized,
    S: CmpSymbolDef,
{
    fn is_true(&self, data: &V) -> bool {
        if let Some(var_obj) = data.value_get(self.var_name()) {
            return self.right_const.compare(var_obj, self.compare_op());
        }
        false
    }
}

impl<T, V, S> Condition<V> for ExpressEnum<T, S>
where
    V: ValueGet<T>,
    LogicExpress<T, S>: Condition<V>,
    CompareExpress<T, S>: Condition<V>,
    T: Sized,
    S: LogicSymbolDef + CmpSymbolDef,
{
    fn is_true(&self, data: &V) -> bool {
        match self {
            ExpressEnum::Logic(logic) => logic.is_true(data),
            ExpressEnum::Compare(compare) => compare.is_true(data),
        }
    }
}

#[allow(clippy::borrowed_box)]
pub fn cmp_is_true<V, T: Condition<V>>(
    op: &LogicOP,
    left: Option<&Box<T>>,
    right: &Box<T>,
    data: &V,
) -> bool {
    match op {
        LogicOP::And => {
            if let Some(left) = left {
                left.is_true(data) && right.is_true(data)
            } else {
                right.is_true(data)
            }
        }
        LogicOP::Or => {
            if let Some(left) = left {
                left.is_true(data) || right.is_true(data)
            } else {
                right.is_true(data)
            }
        }
        LogicOP::Not => !right.is_true(data),
    }
}

//不能进行泛化实现,会造成递归展开失败!!!!
/*
impl<T, V> Condition<V> for LogicExpress<T>
where
    V: ValueGet<T>,
    T: CompareAble + Sized,
    ExpressEnum<T>: Condition<V>,
{
    fn is_true(&self, data: &V) -> bool {
        match self.op {
            LogicOP::And => {
                if let Some(left) = &self.left {
                    left.is_true(data) && self.right.is_true(data)
                } else {
                    self.right.is_true(data)
                }
            }
            LogicOP::Or => {
                if let Some(left) = &self.left {
                    left.is_true(data) || self.right.is_true(data)
                } else {
                    self.right.is_true(data)
                }
            }
            LogicOP::Not => !self.right.is_true(data),
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use crate::cond::{cmp_is_true, Condition, ExpressEnum, LogicExpress, LogicOP, ValueGet};
    use crate::cond::{CmpOP, RustSymbol};
    use std::collections::HashMap;

    type CompareExpress<T> = crate::cond::CompareExpress<T, RustSymbol>;

    type XMap<T> = HashMap<&'static str, T>;
    impl<T> ValueGet<T> for HashMap<&'static str, T> {
        fn value_get(&self, var: &str) -> Option<&T> {
            return self.get(var);
        }
    }
    impl Condition<XMap<u32>> for LogicExpress<u32, RustSymbol> {
        fn is_true(&self, data: &XMap<u32>) -> bool {
            cmp_is_true(&self.op, self.left.as_ref(), &self.right, data)
        }
    }

    #[test]
    pub fn test_eq_u32() {
        let express = CompareExpress::new(CmpOP::Eq, "a", 1u32);
        let data = HashMap::from([("a", 1)]);
        assert!(express.is_true(&data));
        let data = HashMap::from([("a", 2)]);
        assert!(!express.is_true(&data));
        let data = HashMap::from([("b", 1)]);
        assert!(!express.is_true(&data));

        let express2 = CompareExpress::new(CmpOP::Eq, "a", 1u32);
        let express3 = LogicExpress::<u32, RustSymbol>::new(
            LogicOP::And,
            Some(ExpressEnum::Compare(express)),
            ExpressEnum::Compare(express2),
        );
        let data = HashMap::from([("a", 1)]);
        assert!(express3.is_true(&data))
    }
    #[test]
    pub fn test_eq_f64() {
        let express = CompareExpress::new(CmpOP::Eq, "a", 1.1f64);
        let data = HashMap::from([("a", 1.1f64)]);
        assert!(express.is_true(&data));
        let data = HashMap::from([("a", 1.10000000001f64)]);
        assert!(!express.is_true(&data));
    }

    #[test]
    pub fn test_compare_we() {
        let express = CompareExpress::new(CmpOP::We, "a", "*ll*".to_string());
        let data = HashMap::from([("a", "hello".to_string())]);
        assert!(express.is_true(&data));
        let data = HashMap::from([("a", "ell".to_string())]);
        assert!(express.is_true(&data));
        let data = HashMap::from([("a", "heloo".to_string())]);
        assert!(!express.is_true(&data));
    }
    #[test]
    pub fn test_compare_ge() {
        let express = CompareExpress::new(CmpOP::Le, "a", 2u32);
        let data = HashMap::from([("a", 1)]);
        assert!(express.is_true(&data));
        let data = HashMap::from([("a", 2)]);
        assert!(express.is_true(&data));
        let data = HashMap::from([("a", 3)]);
        assert!(!express.is_true(&data));
        let data = HashMap::from([("b", 1)]);
        assert!(!express.is_true(&data));
    }
}

pub trait LogicSymbolDef {
    fn symbol_and() -> &'static str;
    fn symbol_not() -> &'static str;
    fn symbol_or() -> &'static str;
    fn symbol_logic(op: &LogicOP) -> &'static str {
        match op {
            LogicOP::And => Self::symbol_and(),
            LogicOP::Or => Self::symbol_or(),
            LogicOP::Not => Self::symbol_not(),
        }
    }
}

pub trait CmpSymbolDef {
    fn symbol_eq() -> &'static str;
    fn symbol_we() -> &'static str;
    fn symbol_ne() -> &'static str;
    fn symbol_ge() -> &'static str;
    fn symbol_gt() -> &'static str;
    fn symbol_le() -> &'static str;
    fn symbol_lt() -> &'static str;

    fn symbol_var(name: &str) -> String;
    fn symbol_cmp(op: &CmpOP) -> &'static str {
        match op {
            CmpOP::We => Self::symbol_we(),
            CmpOP::Eq => Self::symbol_eq(),
            CmpOP::Ne => Self::symbol_ne(),
            CmpOP::Gt => Self::symbol_gt(),
            CmpOP::Ge => Self::symbol_ge(),
            CmpOP::Lt => Self::symbol_lt(),
            CmpOP::Le => Self::symbol_le(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SQLSymbol {}

#[derive(Debug, PartialEq, Clone)]
pub struct RustSymbol {}
