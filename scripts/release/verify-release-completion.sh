#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version_output="$(bash "${SCRIPT_DIR}/verify-version.sh" "${1:-}")"
version="$(awk -F= '$1 == "version" { print $2 }' <<< "${version_output}")"
version_bare="$(awk -F= '$1 == "version_bare" { print $2 }' <<< "${version_output}")"
RELEASE_REPO="${RELEASE_REPO:-HiroyukiFuruno/katana-language-editor}"
expected_release_commit="${EXPECTED_RELEASE_COMMIT:-$(git rev-parse HEAD)}"

remote_tag_refs="$(git ls-remote --tags origin "refs/tags/${version}" "refs/tags/${version}^{}")"
remote_tag_commit="$(python3 - "${version}" "${remote_tag_refs}" <<'PY'
import sys

version = sys.argv[1]
refs = dict(line.split(maxsplit=1) for line in sys.argv[2].splitlines() if line.strip())
print(refs.get(f"refs/tags/{version}^{{}}") or refs.get(f"refs/tags/{version}") or "")
PY
)"
if [[ -z "${remote_tag_commit}" || "${remote_tag_commit}" != "${expected_release_commit}" ]]; then
  echo "Remote tag is missing or does not match the release commit for ${version}" >&2
  exit 1
fi

release_json="$(gh release view "${version}" --repo "${RELEASE_REPO}" --json tagName,isDraft,isPrerelease,url)"
python3 - "${version}" "${release_json}" <<'PY'
import json
import sys

version, raw = sys.argv[1:]
release = json.loads(raw)
if release.get("tagName") != version or release.get("isDraft") or release.get("isPrerelease"):
    raise SystemExit("GitHub Release is not a published exact-tag release")
PY

for crate in katana-language-editor katana-language-editor-egui; do
  cargo info "${crate}@${version_bare}" --registry crates-io >/dev/null
done

echo "PASS: release completion verification passed for ${version}"
