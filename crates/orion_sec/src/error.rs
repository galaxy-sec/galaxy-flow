use orion_conf::error::SerdeReason;
use orion_error::{ErrorCode, StructError, UvsConfFrom, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, Error)]
pub enum SecErrReason {
    #[error("{0}")]
    Uvs(UvsReason),
}

pub type SecError = StructError<SecErrReason>;
pub type SecResult<T> = Result<T, SecError>;
