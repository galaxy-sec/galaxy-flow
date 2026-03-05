use std::fs;
use std::path::{Path, PathBuf};

use orion_accessor::addr::{Address, HttpResource};
use orion_accessor::types::ResourceDownloader;
use orion_accessor::update::DownloadOptions;
use orion_error::{ErrorOwe, ErrorWith, ToStructError};
use orion_variate::vars::EnvDict;

use crate::err::{RunReason, RunResult};
use crate::util::accessor::build_accessor;

use super::model::{ReleaseChannel, SelfUpdateManifest};

#[derive(Clone, Debug, Default)]
pub struct SelfUpdateClient {}

impl SelfUpdateClient {
    pub fn manifest_url(base: &str) -> String {
        format!("{}/manifest.json", base.trim_end_matches('/'))
    }

    pub async fn fetch_manifest(
        &self,
        base: &str,
        channel: ReleaseChannel,
        temp_dir: &Path,
    ) -> RunResult<SelfUpdateManifest> {
        let url = Self::manifest_url(base);
        ensure_whitelist(&url)?;
        let path = temp_dir.join(format!("manifest-{}.json", channel.as_str()));
        self.download_to_path(&url, &path).await?;

        let content = fs::read_to_string(&path)
            .owe_res()
            .want("read self update manifest file")
            .with(("url", url.as_str()))
            .with(("path", path.as_path()))?;
        let manifest = serde_json::from_str::<SelfUpdateManifest>(&content)
            .owe_data()
            .want("parse self update manifest")
            .with(("url", url.as_str()))
            .with(("path", path.as_path()))?;
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
            fs::create_dir_all(parent)
                .owe_res()
                .want("create download destination parent")
                .with(("path", parent))?;
        }
        let addr = HttpResource::from(url);
        build_accessor(&EnvDict::default())
            .download_to_local(&Address::from(addr), dst, &DownloadOptions::default())
            .await
            .owe_res()
            .want("download file")
            .with(("url", url))
            .with(("dst", dst))?;
        Ok(dst.to_path_buf())
    }
}

fn ensure_whitelist(url: &str) -> RunResult<()> {
    let parsed = url::Url::parse(url)
        .owe_data()
        .want("parse download url")
        .with(("url", url))?;
    if parsed.scheme() != "https" {
        return Err(RunReason::Args("only https download is allowed".into())
            .to_err()
            .want("validate download url scheme")
            .with(("url", url))
            .with_detail(format!("url={url}")));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| RunReason::Args("url has no host".into()).to_err())
        .want("validate download url host")
        .with(("url", url))?;
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
        .want("validate download host allowlist")
        .with(("url", url))
        .with(("host", host))
        .with_detail(format!("host={host}")))
}

#[cfg(test)]
mod tests {
    use super::{SelfUpdateClient, ensure_whitelist};

    #[test]
    fn manifest_url_uses_base_path() {
        let stable = SelfUpdateClient::manifest_url("https://example.com/updates");
        let alpha = SelfUpdateClient::manifest_url("https://example.com/alpha/updates/");
        let beta = SelfUpdateClient::manifest_url("https://example.com/beta/updates/");
        assert_eq!(stable, "https://example.com/updates/manifest.json");
        assert_eq!(alpha, "https://example.com/alpha/updates/manifest.json");
        assert_eq!(beta, "https://example.com/beta/updates/manifest.json");
    }

    #[test]
    fn whitelist_requires_https_and_known_hosts() {
        assert!(ensure_whitelist("https://raw.githubusercontent.com/a/b/c").is_ok());
        assert!(ensure_whitelist("https://release-assets.githubusercontent.com/x").is_ok());
        assert!(ensure_whitelist("http://raw.githubusercontent.com/a/b").is_err());
        assert!(ensure_whitelist("https://example.com/x").is_err());
    }
}
