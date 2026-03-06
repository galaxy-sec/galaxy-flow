#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="galaxy-sec"
REPO_NAME="galaxy-flow"

DEFAULT_CHANNEL="stable"
DEFAULT_INSTALL_DIR="${HOME}/bin"

CHANNEL="${CHANNEL:-$DEFAULT_CHANNEL}"
INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
INSTALL_TMP_DIR=""

usage() {
  cat <<USAGE
Galaxy Flow installer

Usage:
  curl -fsSL https://github.com/${REPO_OWNER}/${REPO_NAME}/raw/main/install.sh | bash

Options:
  --channel <stable|alpha|beta>   Release channel (default: stable)
  --install-dir <dir>             Install directory (default: ~/bin)
  -h, --help                      Show this help

Environment:
  CHANNEL                          Same as --channel
  INSTALL_DIR                      Same as --install-dir

Examples:
  curl -fsSL https://github.com/${REPO_OWNER}/${REPO_NAME}/raw/main/install.sh | bash
  curl -fsSL https://github.com/${REPO_OWNER}/${REPO_NAME}/raw/main/install.sh | bash -s -- --channel alpha
  curl -fsSL https://github.com/${REPO_OWNER}/${REPO_NAME}/raw/main/install.sh | INSTALL_DIR=/usr/local/bin bash
USAGE
}

log() {
  printf '[install] %s\n' "$*"
}

warn() {
  printf '[install][warn] %s\n' "$*" >&2
}

fail() {
  printf '[install][error] %s\n' "$*" >&2
  exit 1
}

cleanup() {
  if [[ -n "${INSTALL_TMP_DIR:-}" ]] && [[ -d "${INSTALL_TMP_DIR:-}" ]]; then
    rm -rf "$INSTALL_TMP_DIR"
  fi
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

json_has_tool() {
  command -v jq >/dev/null 2>&1 || command -v python3 >/dev/null 2>&1
}

json_get() {
  local file="$1"
  local expr="$2"

  if command -v jq >/dev/null 2>&1; then
    jq -r "$expr" "$file"
    return 0
  fi

  if command -v python3 >/dev/null 2>&1; then
    python3 - "$file" "$expr" <<'PY'
import json
import sys

path = sys.argv[1]
expr = sys.argv[2]

with open(path, 'r', encoding='utf-8') as f:
    data = json.load(f)

# Supported expressions:
# .version
# .channel
# .assets["target"].url
# .assets["target"].sha256
# .assets_keys
if expr == '.version':
    print(data.get('version', ''))
elif expr == '.channel':
    print(data.get('channel', ''))
elif expr == '.assets_keys':
    assets = data.get('assets', {})
    print(' '.join(sorted(assets.keys())))
elif expr.startswith('.assets["') and expr.endswith('"].url'):
    target = expr[len('.assets["'):-len('"].url')]
    print((data.get('assets', {}).get(target, {}) or {}).get('url', ''))
elif expr.startswith('.assets["') and expr.endswith('"].sha256'):
    target = expr[len('.assets["'):-len('"].sha256')]
    print((data.get('assets', {}).get(target, {}) or {}).get('sha256', ''))
else:
    raise SystemExit(f'unsupported expr: {expr}')
PY
    return 0
  fi

  fail "need jq or python3 to parse manifest json"
}

sha256_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
    return 0
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
    return 0
  fi
  fail "need sha256sum or shasum for checksum verification"
}

manifest_url_for_channel() {
  local channel="$1"
  case "$channel" in
    stable)
      printf 'https://github.com/%s/%s/raw/main/updates/stable/manifest.json\n' "$REPO_OWNER" "$REPO_NAME"
      ;;
    alpha)
      printf 'https://github.com/%s/%s/raw/alpha/updates/alpha/manifest.json\n' "$REPO_OWNER" "$REPO_NAME"
      ;;
    beta)
      printf 'https://github.com/%s/%s/raw/beta/updates/beta/manifest.json\n' "$REPO_OWNER" "$REPO_NAME"
      ;;
    *)
      fail "unsupported channel: $channel (expected: stable|alpha|beta)"
      ;;
  esac
}

candidate_targets() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os/$arch" in
    Darwin/arm64|Darwin/aarch64)
      printf 'aarch64-apple-darwin\n'
      ;;
    Darwin/x86_64)
      printf 'x86_64-apple-darwin\n'
      ;;
    Linux/x86_64|Linux/amd64)
      # Prefer glibc build first, then musl fallback.
      printf 'x86_64-unknown-linux-gnu\n'
      printf 'x86_64-unknown-linux-musl\n'
      ;;
    *)
      fail "unsupported platform: $os/$arch"
      ;;
  esac
}

is_all_zero_hash() {
  local s="$1"
  [[ -n "$s" ]] && [[ "$s" =~ ^0+$ ]]
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --channel)
        [[ $# -ge 2 ]] || fail "--channel requires a value"
        CHANNEL="$2"
        shift 2
        ;;
      --install-dir)
        [[ $# -ge 2 ]] || fail "--install-dir requires a value"
        INSTALL_DIR="$2"
        shift 2
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        fail "unknown argument: $1"
        ;;
    esac
  done
}

main() {
  trap cleanup EXIT

  parse_args "$@"
  CHANNEL="$(printf '%s' "$CHANNEL" | tr '[:upper:]' '[:lower:]')"

  need_cmd curl
  need_cmd tar
  json_has_tool || fail "need jq or python3 to parse manifest json"

  local manifest_url
  manifest_url="$(manifest_url_for_channel "$CHANNEL")"

  local tmp_dir
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/galaxy-flow-install.XXXXXX")"
  INSTALL_TMP_DIR="$tmp_dir"

  local manifest_file
  manifest_file="$tmp_dir/manifest.json"

  log "channel: $CHANNEL"
  log "download manifest: $manifest_url"
  curl -fL --silent --show-error "$manifest_url" -o "$manifest_file"

  local manifest_version manifest_channel
  manifest_version="$(json_get "$manifest_file" '.version')"
  manifest_channel="$(json_get "$manifest_file" '.channel')"

  [[ -n "$manifest_version" ]] || fail "invalid manifest: missing version"
  [[ "$manifest_channel" == "$CHANNEL" ]] || fail "manifest channel mismatch: expected=$CHANNEL got=$manifest_channel"

  local selected_target=""
  local asset_url=""
  local asset_sha=""
  local t
  while IFS= read -r t; do
    [[ -n "$t" ]] || continue
    local u s
    u="$(json_get "$manifest_file" ".assets[\"$t\"].url")"
    s="$(json_get "$manifest_file" ".assets[\"$t\"].sha256")"
    if [[ -n "$u" ]]; then
      selected_target="$t"
      asset_url="$u"
      asset_sha="$s"
      break
    fi
  done < <(candidate_targets)

  if [[ -z "$selected_target" ]]; then
    local keys
    keys="$(json_get "$manifest_file" '.assets_keys')"
    fail "no asset for current platform. available targets: ${keys:-<none>}"
  fi

  log "version: $manifest_version"
  log "target: $selected_target"
  log "download asset: $asset_url"

  local archive_file
  archive_file="$tmp_dir/galaxy-flow.tar.gz"
  curl -fL --progress-bar "$asset_url" -o "$archive_file"

  if [[ -n "$asset_sha" ]] && ! is_all_zero_hash "$asset_sha"; then
    local got_sha
    local got_sha_lc
    local asset_sha_lc
    got_sha="$(sha256_file "$archive_file")"
    got_sha_lc="$(printf '%s' "$got_sha" | tr '[:upper:]' '[:lower:]')"
    asset_sha_lc="$(printf '%s' "$asset_sha" | tr '[:upper:]' '[:lower:]')"
    if [[ "$got_sha_lc" != "$asset_sha_lc" ]]; then
      fail "sha256 mismatch: expect=$asset_sha got=$got_sha"
    fi
    log "sha256 verified"
  else
    warn "manifest sha256 is empty or placeholder, skip checksum verification"
  fi

  local unpack_dir
  unpack_dir="$tmp_dir/unpack"
  mkdir -p "$unpack_dir"
  tar -xzf "$archive_file" -C "$unpack_dir"

  local gprj_src gflow_src
  gprj_src="$(find "$unpack_dir" -type f -name gprj | head -n 1 || true)"
  gflow_src="$(find "$unpack_dir" -type f -name gflow | head -n 1 || true)"

  [[ -n "$gprj_src" ]] || fail "gprj binary not found in archive"
  [[ -n "$gflow_src" ]] || fail "gflow binary not found in archive"

  mkdir -p "$INSTALL_DIR"
  [[ -w "$INSTALL_DIR" ]] || fail "install dir is not writable: $INSTALL_DIR"

  cp "$gprj_src" "$INSTALL_DIR/gprj"
  cp "$gflow_src" "$INSTALL_DIR/gflow"
  chmod +x "$INSTALL_DIR/gprj" "$INSTALL_DIR/gflow"

  log "installed to: $INSTALL_DIR"

  if ! command -v gprj >/dev/null 2>&1 || ! command -v gflow >/dev/null 2>&1; then
    warn "'$INSTALL_DIR' is not in PATH. add it to your shell profile."
  fi

  "$INSTALL_DIR/gprj" --version || true
  "$INSTALL_DIR/gflow" --version || true

  log "done"
}

main "$@"
