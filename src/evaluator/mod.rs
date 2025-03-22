// extern crate shells;

#[derive(Debug, PartialEq)]
pub enum ExpError {
    NoVal(String),
}

mod env_exp;
pub use crate::evaluator::env_exp::{EnvExpress, Parser};
