#!/usr/bin/env bash
set -euo pipefail

# End-to-end smoke test for:
#   gx self update --force --yes
#   gx self rollback --id <id>
#
# Defaults:
#   - run copied binaries from target/debug in a temp install dir
#   - isolate state/backups by using a temp HOME
#   - channel: alpha
#
# Usage:
#   tests/self_update_update_rollback.sh [channel] [gx_path]
# Example:
#   tests/self_update_update_rollback.sh alpha ./target/debug/gx

CHANNEL="${1:-alpha}"
GX_PATH="${2:-./target/debug/gx}"

if [[ ! -x "${GX_PATH}" ]]; then
  echo "gx not executable: ${GX_PATH}" >&2
  exit 1
fi

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/gxl-self-update-test.XXXXXX")"
INSTALL_DIR="${WORK_DIR}/install"
HOME_DIR="${WORK_DIR}/home"
mkdir -p "${INSTALL_DIR}" "${HOME_DIR}"

cleanup() {
  rm -rf "${WORK_DIR}"
}
trap cleanup EXIT

cp "${GX_PATH}" "${INSTALL_DIR}/gx"
chmod +x "${INSTALL_DIR}/gx"

echo "[1/5] status(before)"
HOME="${HOME_DIR}" "${INSTALL_DIR}/gx" self status

echo "[2/5] update --channel ${CHANNEL} --force --yes"
UPDATE_OUT="$(
  HOME="${HOME_DIR}" "${INSTALL_DIR}/gx" self update --channel "${CHANNEL}" --force --yes
)"
echo "${UPDATE_OUT}"

if ! grep -q "updated=true" <<<"${UPDATE_OUT}"; then
  echo "update did not report updated=true" >&2
  exit 1
fi

BACKUP_ID="$(awk -F= '/^backup_id=/{print $2}' <<<"${UPDATE_OUT}" | tail -n1)"
if [[ -z "${BACKUP_ID}" ]]; then
  echo "update output missing backup_id" >&2
  exit 1
fi

echo "[3/5] rollback --id ${BACKUP_ID}"
if ROLLBACK_OUT="$(
  HOME="${HOME_DIR}" "${INSTALL_DIR}/gx" self rollback --id "${BACKUP_ID}"
)"; then
  :
else
  # Compatibility fallback for older/newer CLI variants.
  ROLLBACK_OUT="$(
    HOME="${HOME_DIR}" "${INSTALL_DIR}/gx" self rollback --backup-id "${BACKUP_ID}"
  )"
fi
echo "${ROLLBACK_OUT}"

if ! grep -q "^rollback=true$" <<<"${ROLLBACK_OUT}"; then
  echo "rollback did not report rollback=true" >&2
  exit 1
fi

echo "[4/5] status(after)"
HOME="${HOME_DIR}" "${INSTALL_DIR}/gx" self status

echo "[5/5] PASS"
echo "work_dir=${WORK_DIR}"
echo "note=work_dir removed by trap on exit"
