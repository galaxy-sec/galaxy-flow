mod client;
mod installer;
mod model;
mod service;
mod storage;

pub use model::{
    AutoMode, CheckResult, ReleaseChannel, SelfUpdateManifest, SelfUpdatePolicy, SelfUpdateState,
    StatusResult, UpdateResult,
};
pub use service::{AutoSetRequest, CheckRequest, SelfUpdateService, UpdateRequest};
