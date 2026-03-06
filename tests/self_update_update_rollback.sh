#!/usr/bin/env bash
set -euo pipefail

# End-to-end smoke test for:
#   gprj self update --force --yes
#   gprj self rollback --id <id>
#
# Defaults:
#   - run copied binaries from target/debug in a temp install dir
#   - isolate state/backups by using a temp HOME
#   - channel: alpha
#
# Usage:
#   tests/self_update_update_rollback.sh [channel] [gprj_path]
# Example:
#   tests/self_update_update_rollback.sh alpha ./target/debug/gprj

CHANNEL="${1:-alpha}"
GPRJ_PATH="${2:-./target/debug/gprj}"

if [[ ! -x "${GPRJ_PATH}" ]]; then
  echo "gprj not executable: ${GPRJ_PATH}" >&2
  exit 1
fi

GPRJ_DIR="$(cd "$(dirname "${GPRJ_PATH}")" && pwd)"
GFLOW_PATH="${GPRJ_DIR}/gflow"
if [[ ! -x "${GFLOW_PATH}" ]]; then
  echo "gflow not executable (must be sibling of gprj): ${GFLOW_PATH}" >&2
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

cp "${GPRJ_PATH}" "${INSTALL_DIR}/gprj"
cp "${GFLOW_PATH}" "${INSTALL_DIR}/gflow"
chmod +x "${INSTALL_DIR}/gprj" "${INSTALL_DIR}/gflow"

echo "[1/5] status(before)"
HOME="${HOME_DIR}" "${INSTALL_DIR}/gprj" self status

echo "[2/5] update --channel ${CHANNEL} --force --yes"
UPDATE_OUT="$(
  HOME="${HOME_DIR}" "${INSTALL_DIR}/gprj" self update --channel "${CHANNEL}" --force --yes
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
  HOME="${HOME_DIR}" "${INSTALL_DIR}/gprj" self rollback --id "${BACKUP_ID}"
)"; then
  :
else
  # Compatibility fallback for older/newer CLI variants.
  ROLLBACK_OUT="$(
    HOME="${HOME_DIR}" "${INSTALL_DIR}/gprj" self rollback --backup-id "${BACKUP_ID}"
  )"
fi
echo "${ROLLBACK_OUT}"

if ! grep -q "^rollback=true$" <<<"${ROLLBACK_OUT}"; then
  echo "rollback did not report rollback=true" >&2
  exit 1
fi

echo "[4/5] status(after)"
HOME="${HOME_DIR}" "${INSTALL_DIR}/gprj" self status

echo "[5/5] PASS"
echo "work_dir=${WORK_DIR}"
echo "note=work_dir removed by trap on exit"
