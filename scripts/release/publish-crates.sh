#!/usr/bin/env bash
set -euo pipefail

version="$(bash "$(dirname "$0")/verify-version.sh" "${1:-}" | awk -F= '$1 == "version_bare" { print $2 }')"

require_token() {
  local name="$1"
  if [[ -n "${!name:-}" ]]; then
    return
  fi
  echo "${name} is required." >&2
  exit 1
}

publish_if_needed() {
  local package="$1"
  local token="$2"
  if cargo info "${package}@${version}" --registry crates-io >/dev/null 2>&1; then
    echo "${package} ${version} already published; skipping."
    return
  fi
  cargo publish -p "${package}" --locked --token "${token}"
}

require_kuc_release_set() {
  local workspace_manifest="${PWD}/Cargo.toml"
  local kuc_release_set

  kuc_release_set="$(python3 - "${workspace_manifest}" <<'PY'
import sys
import tomllib

manifest_path = sys.argv[1]
with open(manifest_path, "rb") as manifest_file:
    document = tomllib.load(manifest_file)

dependencies = document.get("workspace", {}).get("dependencies")
if not isinstance(dependencies, dict):
    raise SystemExit("Cargo.toml is missing [workspace.dependencies].")

dependency_name = "katana-ui-core"
declaration = dependencies.get(dependency_name)
if not isinstance(declaration, dict):
    raise SystemExit("Cargo.toml must declare katana-ui-core as a table dependency.")
if "path" in declaration or "git" in declaration:
    raise SystemExit("katana-ui-core must resolve from crates.io before KLE publication.")
version = declaration.get("version")
features = set(declaration.get("features", []))
if version != "=0.3.11":
    raise SystemExit("KLE v0.1.0 requires katana-ui-core v0.3.11 exactly.")
if not {"egui", "text-raster"}.issubset(features):
    raise SystemExit("katana-ui-core must enable egui and text-raster features.")
print(f"{dependency_name}\t{version}")
PY
)"

  [[ -n "${kuc_release_set}" ]] || {
    echo "KUC release set is empty; refusing to publish KLE." >&2
    exit 1
  }

  while IFS=$'\t' read -r package version; do
    [[ -n "${package}" && -n "${version}" ]] || {
      echo "KUC release set contains an incomplete package/version entry." >&2
      exit 1
    }
    if ! cargo info "${package}@${version}" --registry crates-io >/dev/null 2>&1; then
      echo "${package} ${version} is not visible on crates.io; refusing to publish KLE." >&2
      exit 1
    fi
  done <<< "${kuc_release_set}"
}

wait_for_crate() {
  local package="$1"
  for _ in {1..30}; do
    if cargo info "${package}@${version}" --registry crates-io >/dev/null 2>&1; then
      return
    fi
    sleep 10
  done
  echo "${package} ${version} did not become visible on crates.io in time." >&2
  exit 1
}

require_token CARGO_REGISTRY_TOKEN
require_kuc_release_set
publish_if_needed katana-language-editor "${CARGO_REGISTRY_TOKEN}"
wait_for_crate katana-language-editor
publish_if_needed katana-language-editor-egui "${CARGO_REGISTRY_TOKEN}"
wait_for_crate katana-language-editor-egui
