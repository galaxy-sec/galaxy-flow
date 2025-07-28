use std::fmt::Debug;

use super::traits::Evaluation;
use crate::{calculate::traits::DecideResult, context::ExecContext, execution::VarSpace};

#[derive(Clone, Debug)]
pub enum BinLogic {
    AND,
    OR,
}

#[derive(Clone, Debug)]
pub struct BinLogicExpress<T, E>
where
    T: Evaluation + Debug,
    E: Evaluation + Debug,
{
    relation: BinLogic,
    first: T,
    second: E,
}
#[derive(Clone, Debug)]
pub struct NotLogicExpress<T>
where
    T: Evaluation + Debug,
{
    first: T,
}

#[derive(Clone, Debug)]
pub enum LogicExpress<T, E>
where
    T: Evaluation + Debug,
    E: Evaluation + Debug,
{
    Bin(BinLogicExpress<T, E>),
    Not(NotLogicExpress<T>),
}

impl<T, E> LogicExpress<T, E>
where
    T: Evaluation + Debug,
    E: Evaluation + Debug,
{
    pub fn and_exp(first: T, second: E) -> Self {
        Self::Bin(BinLogicExpress {
            relation: BinLogic::AND,
            first,
            second,
        })
    }
    pub fn or_exp(first: T, second: E) -> Self {
        Self::Bin(BinLogicExpress {
            relation: BinLogic::OR,
            first,
            second,
        })
    }
    pub fn not_exp(first: T) -> Self {
        Self::Not(NotLogicExpress { first })
    }
}

impl<T, E> Evaluation for LogicExpress<T, E>
where
    T: Evaluation + Debug,
    E: Evaluation + Debug,
{
    fn decide(&self, ctx: ExecContext, def: &VarSpace) -> DecideResult {
        match self {
            LogicExpress::Bin(bin) => bin.decide(ctx, def),
            LogicExpress::Not(not) => not.decide(ctx, def),
        }
    }
}

impl<T, E> Evaluation for BinLogicExpress<T, E>
where
    T: Evaluation + Debug,
    E: Evaluation + Debug,
{
    fn decide(&self, ctx: ExecContext, def: &VarSpace) -> DecideResult {
        match self.relation {
            BinLogic::AND => {
                let first = self.first.decide(ctx.clone(), def)?;
                if first && self.second.decide(ctx.clone(), def)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            BinLogic::OR => {
                let first = self.first.decide(ctx.clone(), def)?;
                if first || self.second.decide(ctx.clone(), def)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

impl<T> Evaluation for NotLogicExpress<T>
where
    T: Evaluation + Debug,
{
    fn decide(&self, ctx: ExecContext, def: &VarSpace) -> DecideResult {
        let first = self.first.decide(ctx.clone(), def)?;
        Ok(!first)
    }
}
