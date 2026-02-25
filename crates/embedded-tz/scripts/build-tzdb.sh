#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

TZDB_DIR="${TZDB_DIR:-${REPO_ROOT}/tzdb}"
TZDB_BUILD_TOPDIR="${TZDB_BUILD_TOPDIR:-${TZDB_DIR}/out}"
TZDB_INSTALL_ROOT="${TZDB_BUILD_TOPDIR}/usr/share/zoneinfo"
TZDB_OUTPUT_DIR="${TZDB_OUTPUT_DIR:-${REPO_ROOT}/zoneinfo}"

echo "Preparing tzdb submodule..."
git -C "${REPO_ROOT}" submodule update --init --recursive tzdb

if [[ ! -f "${TZDB_DIR}/Makefile" ]]; then
  echo "tzdb source is missing: ${TZDB_DIR}" >&2
  exit 1
fi

echo "Building tzdb (this compiles TZif files)..."
make -C "${TZDB_DIR}" clean
# Force "fat" TZif binaries so transitions are explicit in-file.
# Also include backzone data for zone.tab entries used by historical tests.
# This parser currently does not consume the POSIX tail rules found in slim files.
make -C "${TZDB_DIR}" TOPDIR="${TZDB_BUILD_TOPDIR}" DESTDIR= ZFLAGS='-b fat' PACKRATDATA=backzone PACKRATLIST=zone.tab install

if [[ ! -d "${TZDB_INSTALL_ROOT}" ]]; then
  echo "Expected zoneinfo output missing: ${TZDB_INSTALL_ROOT}" >&2
  exit 1
fi

echo "Creating filtered zoneinfo tree at ${TZDB_OUTPUT_DIR}..."
rm -rf "${TZDB_OUTPUT_DIR}"
mkdir -p "${TZDB_OUTPUT_DIR}"

while IFS= read -r -d '' src; do
  # Keep files that look like TZif binaries.
  if [[ "$(dd if="${src}" bs=4 count=1 status=none 2>/dev/null)" == "TZif" ]]; then
    rel="${src#"${TZDB_INSTALL_ROOT}/"}"
    dest="${TZDB_OUTPUT_DIR}/${rel}"
    mkdir -p "$(dirname "${dest}")"
    cp -a "${src}" "${dest}"
  fi
done < <(find "${TZDB_INSTALL_ROOT}" -type f -size +4c -print0)

echo "Done."
echo "Built source tree: ${TZDB_INSTALL_ROOT}"
echo "Filtered test tree: ${TZDB_OUTPUT_DIR}"
