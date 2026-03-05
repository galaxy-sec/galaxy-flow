use std::fs;
use std::path::{Path, PathBuf};

use orion_accessor::addr::{Address, HttpResource};
use orion_accessor::types::ResourceDownloader;
use orion_accessor::update::DownloadOptions;
use orion_error::ToStructError;
use orion_variate::vars::EnvDict;

use crate::err::{RunReason, RunResult};
use crate::util::accessor::build_accessor;

use super::model::{ReleaseChannel, SelfUpdateManifest};

#[derive(Clone, Debug, Default)]
pub struct SelfUpdateClient {}

impl SelfUpdateClient {
    pub fn manifest_url(base: &str, channel: ReleaseChannel) -> String {
        format!(
            "{}/{}/manifest.json",
            base.trim_end_matches('/'),
            channel.as_str()
        )
    }

    pub async fn fetch_manifest(
        &self,
        base: &str,
        channel: ReleaseChannel,
        temp_dir: &Path,
    ) -> RunResult<SelfUpdateManifest> {
        let url = Self::manifest_url(base, channel);
        ensure_whitelist(&url)?;
        let path = temp_dir.join(format!("manifest-{}.json", channel.as_str()));
        self.download_to_path(&url, &path).await?;

        let content = fs::read_to_string(path).map_err(io_err)?;
        let manifest = serde_json::from_str::<SelfUpdateManifest>(&content).map_err(parse_err)?;
        if manifest.channel != channel {
            return Err(RunReason::Exec("manifest channel mismatch".into())
                .to_err()
                .with_detail(format!(
                    "requested={}, got={}",
                    channel.as_str(),
                    manifest.channel.as_str()
                )));
        }
        Ok(manifest)
    }

    pub async fn download_to_path(&self, url: &str, dst: &Path) -> RunResult<PathBuf> {
        ensure_whitelist(url)?;
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
        let addr = HttpResource::from(url);
        build_accessor(&EnvDict::default())
            .download_to_local(
                &Address::from(addr),
                &dst.to_path_buf(),
                &DownloadOptions::default(),
            )
            .await
            .map_err(res_err)?;
        Ok(dst.to_path_buf())
    }
}

fn ensure_whitelist(url: &str) -> RunResult<()> {
    let parsed = url::Url::parse(url).map_err(parse_err)?;
    if parsed.scheme() != "https" {
        return Err(RunReason::Args("only https download is allowed".into())
            .to_err()
            .with_detail(format!("url={url}")));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| RunReason::Args("url has no host".into()).to_err())?;
    let allowed = [
        "github.com",
        "raw.githubusercontent.com",
        "objects.githubusercontent.com",
        "github-releases.githubusercontent.com",
        "release-assets.githubusercontent.com",
    ];
    if allowed
        .iter()
        .any(|v| host == *v || host.ends_with(&format!(".{v}")))
    {
        return Ok(());
    }
    Err(RunReason::Args("download host is not allowed".into())
        .to_err()
        .with_detail(format!("host={host}")))
}

fn io_err(err: std::io::Error) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}

fn parse_err(err: impl std::fmt::Display) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}

fn res_err(err: impl std::fmt::Display) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}

#[cfg(test)]
mod tests {
    use super::{SelfUpdateClient, ensure_whitelist};
    use crate::self_update::ReleaseChannel;

    #[test]
    fn manifest_url_uses_channel_path() {
        let stable =
            SelfUpdateClient::manifest_url("https://example.com/updates", ReleaseChannel::Stable);
        let pre =
            SelfUpdateClient::manifest_url("https://example.com/updates/", ReleaseChannel::Pre);
        assert_eq!(stable, "https://example.com/updates/stable/manifest.json");
        assert_eq!(pre, "https://example.com/updates/pre/manifest.json");
    }

    #[test]
    fn whitelist_requires_https_and_known_hosts() {
        assert!(ensure_whitelist("https://raw.githubusercontent.com/a/b/c").is_ok());
        assert!(ensure_whitelist("https://release-assets.githubusercontent.com/x").is_ok());
        assert!(ensure_whitelist("http://raw.githubusercontent.com/a/b").is_err());
        assert!(ensure_whitelist("https://example.com/x").is_err());
    }
}
