#!/usr/bin/env bash
set -euo pipefail

version_output="$(bash "$(dirname "$0")/verify-version.sh" "${1:-}")"
version="$(awk -F= '$1 == "version" { print $2 }' <<<"${version_output}")"
katana_repo="${2:-${KATANA_REPO:-../katana}}"
kuc_repo="${3:-${KUC_REPO:-../katana-ui-core}}"
kle_repo_url="${4:-${KLE_REPO_URL:-https://github.com/HiroyukiFuruno/katana-language-editor}}"

if [[ "${kle_repo_url}" != *"://"* && "${kle_repo_url}" != git@* ]]; then
  kle_repo_url="https://github.com/${kle_repo_url}"
fi

katana_root="$(cd "${katana_repo}" && pwd)"
kuc_root="$(cd "${kuc_repo}" && pwd)"
katana_ui_manifest="${katana_root}/crates/katana-ui/Cargo.toml"

if [[ ! -f "${katana_ui_manifest}" ]]; then
  echo "KatanA katana-ui manifest not found: ${katana_ui_manifest}" >&2
  exit 1
fi

if [[ ! -f "${kuc_root}/crates/katana-ui-core/Cargo.toml" ]]; then
  echo "KUC manifest not found under: ${kuc_root}" >&2
  exit 1
fi

tmp_parent="$(mktemp -d "${TMPDIR:-/tmp}/kle-katana-tag-build.XXXXXX")"
cleanup() {
  if [[ -n "${tmp_parent:-}" && -d "${tmp_parent}" ]]; then
    rm -rf "${tmp_parent}"
  fi
}
trap cleanup EXIT

copy_root="${tmp_parent}/katana"
rsync -a --exclude target --exclude .git "${katana_root}/" "${copy_root}/"

python3 - "${copy_root}/crates/katana-ui/Cargo.toml" "${kle_repo_url}" "${version}" "${kuc_root}" <<'PY'
import re
import sys
from pathlib import Path

manifest = Path(sys.argv[1])
kle_repo_url = sys.argv[2]
version = sys.argv[3]
kuc_root = Path(sys.argv[4])

text = manifest.read_text(encoding="utf-8")
replacements = {
    "katana-language-editor": f'{{ git = "{kle_repo_url}", tag = "{version}" }}',
    "katana-language-editor-egui": f'{{ git = "{kle_repo_url}", tag = "{version}" }}',
    "katana-ui-core": f'{{ path = "{kuc_root / "crates" / "katana-ui-core"}" }}',
}

for package, spec in replacements.items():
    pattern = re.compile(rf"^{re.escape(package)}\s*=\s*\{{[^\n]*\}}$", re.MULTILINE)
    next_text, count = pattern.subn(f"{package} = {spec}", text)
    if count != 1:
        raise SystemExit(f"expected exactly one {package} dependency in {manifest}, got {count}")
    text = next_text

manifest.write_text(text, encoding="utf-8")
PY

(
  cd "${copy_root}"
  cargo check -p katana-ui --test ui_integration_parallel
)

echo "KatanA katana-ui builds with katana-language-editor ${version} git tag"
