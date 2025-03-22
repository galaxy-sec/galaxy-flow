use crate::{evaluator::ExpError, ExecReason};

impl std::convert::From<ExpError> for ExecReason {
    fn from(error: ExpError) -> Self {
        match error {
            ExpError::NoVal(msg) => ExecReason::Exp(msg), // _ => {
        }
    }
}
