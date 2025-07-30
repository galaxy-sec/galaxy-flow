use orion_common::cond::{CmpSymbolDef, CompareExpress, ExpressEnum, LogicSymbolDef};
use std::marker::PhantomData;
use winnow::Result;

use crate::symbol::LogicSymbol;

mod parser;

pub trait CmpParser<T, S>
where
    S: CmpSymbolDef,
{
    fn cmp_exp(data: &mut &str) -> Result<CompareExpress<T, S>>;
}

pub struct WnCondParser<T, H, S> {
    _keep1: PhantomData<T>,
    _keep2: PhantomData<H>,
    _keep3: PhantomData<S>,
}

impl<T, H, S> WnCondParser<T, H, S>
where
    H: CmpParser<T, S>,
    S: LogicSymbolGet + LogicSymbolDef + CmpSymbolDef,
{
    pub fn end_exp(data: &mut &str, stop: &str) -> Result<ExpressEnum<T, S>> {
        Self::lev2_exp(data, Some(stop))
    }
    pub fn exp(data: &mut &str) -> Result<ExpressEnum<T, S>> {
        Self::lev2_exp(data, None)
    }
}

pub trait SymbolFrom<T> {
    fn op_from(value: T) -> Self;
}

pub trait LogicSymbolGet {
    fn logic_and(data: &mut &str) -> Result<LogicSymbol>;
    fn logic_or(data: &mut &str) -> Result<LogicSymbol>;
    fn logic_not(data: &mut &str) -> Result<LogicSymbol>;
}
