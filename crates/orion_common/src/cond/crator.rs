use super::{
    CmpCrator, CmpOP, CmpSymbolDef, CompareExpress, ExpressEnum, LogicCrator, LogicExpress,
    LogicOP, LogicSymbolDef,
};

type CmpEXP<T, S> = CompareExpress<T, S>;
impl<T, S> LogicCrator<CmpEXP<T, S>> for ExpressEnum<T, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    fn from_or(left: CmpEXP<T, S>, right: CmpEXP<T, S>) -> Self {
        ExpressEnum::Logic(LogicExpress::new(
            LogicOP::Or,
            Some(ExpressEnum::Compare(left)),
            ExpressEnum::Compare(right),
        ))
    }
    fn from_not(right: CmpEXP<T, S>) -> Self {
        ExpressEnum::Logic(LogicExpress::new(
            LogicOP::Not,
            None,
            ExpressEnum::Compare(right),
        ))
    }
    fn from_and(left: CmpEXP<T, S>, right: CmpEXP<T, S>) -> Self {
        ExpressEnum::Logic(LogicExpress::new(
            LogicOP::And,
            Some(ExpressEnum::Compare(left)),
            ExpressEnum::Compare(right),
        ))
    }
}

type Exp<T, S> = ExpressEnum<T, S>;
impl<T, S> LogicCrator<Exp<T, S>> for ExpressEnum<T, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    fn from_or(left: Exp<T, S>, right: Exp<T, S>) -> Self {
        ExpressEnum::Logic(LogicExpress::new(LogicOP::Or, Some(left), right))
    }
    fn from_not(right: Exp<T, S>) -> Self {
        ExpressEnum::Logic(LogicExpress::new(LogicOP::Not, None, right))
    }
    fn from_and(left: Exp<T, S>, right: Exp<T, S>) -> Self {
        ExpressEnum::Logic(LogicExpress::new(LogicOP::And, Some(left), right))
    }
}

impl<T, S> CmpCrator<T> for ExpressEnum<T, S>
where
    S: LogicSymbolDef + CmpSymbolDef,
{
    fn from_eq<STR: Into<String>>(left: STR, right: T) -> Self {
        ExpressEnum::Compare(CompareExpress::new(CmpOP::Eq, left.into(), right))
    }

    fn from_gt<STR: Into<String>>(left: STR, right: T) -> Self {
        ExpressEnum::Compare(CompareExpress::new(CmpOP::Gt, left.into(), right))
    }

    fn from_lt<STR: Into<String>>(left: STR, right: T) -> Self {
        ExpressEnum::Compare(CompareExpress::new(CmpOP::Lt, left.into(), right))
    }
}
