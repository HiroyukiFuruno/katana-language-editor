#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version_output="$(bash "${script_dir}/verify-version.sh" "${1:-}")"
version="$(awk -F= '$1 == "version" { print $2 }' <<< "${version_output}")"

if [[ -z "${version}" ]]; then
  echo "Failed to normalize release version." >&2
  exit 1
fi

head_commit="$(git rev-parse HEAD)"
tag_commit="$(git ls-remote --tags origin "refs/tags/${version}^{}" | awk 'NR == 1 { print $1 }')"
if [[ -z "${tag_commit}" ]]; then
  tag_commit="$(git ls-remote --tags origin "refs/tags/${version}" | awk 'NR == 1 { print $1 }')"
fi

if [[ -z "${tag_commit}" ]]; then
  bash "${script_dir}/assert-crates-not-published.sh" "${version}"
  exit 0
fi

if [[ "${tag_commit}" != "${head_commit}" ]]; then
  echo "Tag ${version} does not point to the current release commit." >&2
  echo "tag_commit=${tag_commit}" >&2
  echo "head_commit=${head_commit}" >&2
  exit 1
fi

echo "Tag ${version} already points to current HEAD; allowing idempotent release resume."
