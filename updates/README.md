# Self-Update Manifests

This directory stores release manifests for `gprj self` update checks.

- `stable/manifest.json`: latest stable release
- `alpha/manifest.json`: latest alpha release
- `beta/manifest.json`: latest beta release

Notes:
- Replace `sha256` placeholders with real checksums from release artifacts.
- Keep `assets` keys aligned with runtime target triples.
- `manifest_base_url` defaults to this repo raw path:
  `https://raw.githubusercontent.com/galaxy-sec/galaxy-flow/main/updates`
