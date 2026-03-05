use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    #[default]
    Stable,
    Alpha,
    Beta,
}

impl ReleaseChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            ReleaseChannel::Stable => "stable",
            ReleaseChannel::Alpha => "alpha",
            ReleaseChannel::Beta => "beta",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "stable" => Some(Self::Stable),
            "alpha" => Some(Self::Alpha),
            "beta" => Some(Self::Beta),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SelfUpdateState {
    pub last_checked_at: Option<String>,
    pub last_channel: Option<ReleaseChannel>,
    pub last_remote_version: Option<String>,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
    pub current_version: Option<String>,
    pub installed_at: Option<String>,
    pub last_backup_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestAsset {
    pub url: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateManifest {
    pub version: String,
    pub channel: ReleaseChannel,
    pub published_at: String,
    pub git_ref: Option<String>,
    pub git_commit: Option<String>,
    pub assets: BTreeMap<String, ManifestAsset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusResult {
    pub current_version: String,
    pub install_dir: PathBuf,
    pub state: SelfUpdateState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckResult {
    pub channel: ReleaseChannel,
    pub current_version: String,
    pub remote_version: String,
    pub has_update: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateResult {
    pub channel: ReleaseChannel,
    pub from_version: String,
    pub to_version: String,
    pub backup_id: Option<String>,
    pub updated: bool,
}

#[cfg(test)]
mod tests {
    use super::ReleaseChannel;

    #[test]
    fn parse_release_channel_aliases() {
        assert_eq!(
            ReleaseChannel::parse("stable"),
            Some(ReleaseChannel::Stable)
        );
        assert_eq!(ReleaseChannel::parse("alpha"), Some(ReleaseChannel::Alpha));
        assert_eq!(ReleaseChannel::parse("beta"), Some(ReleaseChannel::Beta));
    }

    #[test]
    fn parse_release_channel_case_insensitive() {
        assert_eq!(
            ReleaseChannel::parse(" StAbLe "),
            Some(ReleaseChannel::Stable)
        );
        assert_eq!(
            ReleaseChannel::parse(" ALPHA "),
            Some(ReleaseChannel::Alpha)
        );
        assert_eq!(ReleaseChannel::parse(" BETA "), Some(ReleaseChannel::Beta));
    }

    #[test]
    fn parse_release_channel_reject_invalid() {
        assert_eq!(ReleaseChannel::parse("gamma"), None);
        assert_eq!(ReleaseChannel::parse("pre"), None);
        assert_eq!(ReleaseChannel::parse(""), None);
    }
}
