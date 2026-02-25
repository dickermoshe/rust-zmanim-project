#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

TZDB_DIR="${TZDB_DIR:-${REPO_ROOT}/tzdb}"
TZDB_OUTPUT_DIR="${TZDB_OUTPUT_DIR:-${REPO_ROOT}/zoneinfo}"

echo "Preparing tzdb submodule..."
git -C "${REPO_ROOT}" submodule update --init --recursive tzdb

if [[ ! -f "${TZDB_DIR}/Makefile" ]]; then
  echo "tzdb source is missing: ${TZDB_DIR}" >&2
  exit 1
fi

echo "Building zic..."
make -C "${TZDB_DIR}" clean
make -C "${TZDB_DIR}" zic

if [[ ! -x "${TZDB_DIR}/zic" ]]; then
  echo "Expected zic binary missing: ${TZDB_DIR}/zic" >&2
  exit 1
fi

echo "Compiling zoneinfo tree at ${TZDB_OUTPUT_DIR}..."
rm -rf "${TZDB_OUTPUT_DIR}"
mkdir -p "${TZDB_OUTPUT_DIR}"

# Match Ubuntu tzdata defaults: no legacy `backward` aliases, keep historical
# data from backzone (e.g. Europe/Amsterdam).
# Keep `-b fat` for this parser, which does not consume slim POSIX tails.
"${TZDB_DIR}/zic" -b fat -L /dev/null -d "${TZDB_OUTPUT_DIR}" \
  "${TZDB_DIR}/africa" \
  "${TZDB_DIR}/antarctica" \
  "${TZDB_DIR}/asia" \
  "${TZDB_DIR}/australasia" \
  "${TZDB_DIR}/europe" \
  "${TZDB_DIR}/northamerica" \
  "${TZDB_DIR}/southamerica" \
  "${TZDB_DIR}/etcetera" \
  "${TZDB_DIR}/backzone"

echo "Done."
echo "Built test tree: ${TZDB_OUTPUT_DIR}"
