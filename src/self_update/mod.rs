mod client;
mod installer;
mod model;
mod service;
mod storage;

pub use model::{
    CheckResult, ReleaseChannel, SelfUpdateManifest, SelfUpdateState, StatusResult, UpdateResult,
};
pub use service::{CheckRequest, SelfUpdateService, UpdateRequest};
