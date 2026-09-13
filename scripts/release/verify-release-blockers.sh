#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version_output="$(bash "${SCRIPT_DIR}/verify-version.sh" "${1:-}")"

version=""
version_bare=""
while IFS= read -r line; do
  case "${line}" in
    version=*) version="${line#version=}" ;;
    version_bare=*) version_bare="${line#version_bare=}" ;;
  esac
done <<< "${version_output}"

if [[ -z "${version}" || -z "${version_bare}" ]]; then
  echo "Failed to normalize version from verify-version output." >&2
  exit 1
fi

RELEASE_REPO="${RELEASE_REPO:-HiroyukiFuruno/katana-language-editor}"
expected_release_commit="${EXPECTED_RELEASE_COMMIT:-$(git rev-parse HEAD)}"
blocker_labels=()
blocker_details=()

add_blocker() {
  blocker_labels+=("$1")
  blocker_details+=("$2")
}

resolve_tag_commit() {
  python3 - "$1" "$2" <<'PY'
import sys

version = sys.argv[1]
refs = dict(line.split(maxsplit=1) for line in sys.argv[2].splitlines() if line.strip())
print(refs.get(f"refs/tags/{version}^{{}}") or refs.get(f"refs/tags/{version}") or "")
PY
}

remote_tag_refs="$(git ls-remote --tags origin "refs/tags/${version}" "refs/tags/${version}^{}")"
remote_tag_commit="$(resolve_tag_commit "${version}" "${remote_tag_refs}")"
if [[ -z "${remote_tag_commit}" ]]; then
  add_blocker "remote-tag" "Remote tag does not exist: ${version}"
elif [[ "${remote_tag_commit}" != "${expected_release_commit}" ]]; then
  add_blocker "remote-tag" "Remote tag commit mismatch for ${version}
remote_tag_commit=${remote_tag_commit}
expected_release_commit=${expected_release_commit}"
fi

if ! release_json="$(gh release view "${version}" --repo "${RELEASE_REPO}" --json tagName,isDraft,isPrerelease,url 2>&1)"; then
  add_blocker "github-release" "GitHub release not found for ${version} on ${RELEASE_REPO}
${release_json}"
elif ! python3 - "${version}" "${release_json}" <<'PY'
import json
import sys

version, raw = sys.argv[1:]
release = json.loads(raw)
if release.get("tagName") != version or release.get("isDraft") or release.get("isPrerelease"):
    raise SystemExit(1)
PY
then
  add_blocker "github-release" "GitHub release metadata is not a published ${version} release."
fi

for crate in katana-language-editor katana-language-editor-egui; do
  case "${crate}" in
    katana-language-editor) crate_label="core" ;;
    katana-language-editor-egui) crate_label="egui" ;;
  esac
  if ! cargo info "${crate}@${version_bare}" --registry crates-io >/dev/null 2>&1; then
    add_blocker "crates.io-${crate_label}" "crate not found on crates.io: ${crate}@${version_bare}"
  fi
done

if ! kuc_dependency_output="$(python3 - <<'PY'
import json
import pathlib
import subprocess
import sys
import tomllib

manifest = tomllib.loads(pathlib.Path("Cargo.toml").read_text(encoding="utf-8"))
required = "katana-ui-core"
required_features = {"egui", "text-raster"}
dependencies = manifest.get("workspace", {}).get("dependencies", {})
errors = []
dependency = dependencies.get(required)
if not isinstance(dependency, dict) or "path" in dependency or "git" in dependency:
    errors.append(f"{required} must use a published registry dependency")
else:
    version = dependency.get("version")
    features = set(dependency.get("features", []))
    if version != "=0.3.10":
        errors.append(f"{required} must pin the published v0.3.10 API exactly")
    if not required_features.issubset(features):
        errors.append(f"{required} must enable {sorted(required_features)}")
metadata = json.loads(subprocess.run(
    ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
    check=True, capture_output=True, text=True,
).stdout)
for package in metadata["packages"]:
    for dependency in package["dependencies"]:
        if dependency["name"] == required and dependency["source"] is None:
            errors.append(f"{package['name']} resolves {required} from a local source")
if errors:
    print("\n".join(errors))
    sys.exit(1)
PY
)"; then
  add_blocker "kuc-registry-dependency" "KLE release may not retain local path or git KUC dependencies.
${kuc_dependency_output}"
fi

if ! kuc_consumer_artifact_output="$(cargo run --locked -p kle-storybook -- --contract-check 2>&1)"; then
  add_blocker "kuc-consumer-artifact" "KUC registry consumer artifact must emit pinned, color-glyph Unicode evidence for the complete editor sequence.
${kuc_consumer_artifact_output}"
fi

if ! future_check_output="$(cargo check --workspace --all-targets --locked 2>&1)"; then
  add_blocker "dependency-check" "Cargo locked workspace check failed before dependency compatibility audit.
${future_check_output}"
fi

if future_incompatibility_output="$(cargo report future-incompatibilities 2>&1)" \
  && grep -Fq "currently triggers the following future incompatibility lints" <<< "${future_incompatibility_output}"; then
  add_blocker "future-incompatibility" "Cargo dependency graph contains future-incompatible packages.
${future_incompatibility_output}"
fi

if (( ${#blocker_labels[@]} == 0 )); then
  echo "PASS: no release blockers detected for ${version}"
  exit 0
fi

echo "Release blockers for ${version}:"
for ((i = 0; i < ${#blocker_labels[@]}; i++)); do
  echo "- ${blocker_labels[i]}"
  while IFS= read -r detail_line; do
    [[ -z "${detail_line}" ]] || echo "  ${detail_line}"
  done <<< "${blocker_details[i]}"
done
exit 1
