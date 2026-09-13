#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

version_stdout="$(bash "${SCRIPT_DIR}/verify-version.sh" "${1:-}")"
version=""
version_bare=""
while IFS= read -r line; do
  case "${line}" in
    version=*)
      version="${line#version=}"
      ;;
    version_bare=*)
      version_bare="${line#version_bare=}"
      ;;
  esac
done <<< "${version_stdout}"

if [[ -z "${version}" || -z "${version_bare}" ]]; then
  echo "Failed to normalize version from verify-version output." >&2
  exit 1
fi

kdv_repo="${2:-${KDV_REPO:-../katana-document-viewer}}"
output_path="${3:-${KDV_FOLLOWUP_OUTPUT:-target/release/kdv-preset-followup-${version}.md}}"

manifest_file="${kdv_repo}/Cargo.toml"
if [[ ! -d "${kdv_repo}" ]]; then
  echo "KDV repo does not exist: ${kdv_repo}" >&2
  exit 1
fi
if [[ ! -f "${manifest_file}" ]]; then
  echo "KDV manifest does not exist: ${manifest_file}" >&2
  exit 1
fi
if [[ ! -r "${manifest_file}" ]]; then
  echo "KDV manifest is not readable: ${manifest_file}" >&2
  exit 1
fi

kdv_reference_markers=(
  "katana-language-editor"
  "katana_language_editor"
)
required_markers=(
  "strings::en()"
  "locale::en_ltr()"
  "settings::default_editor()"
)

grep_args=(
  --fixed-strings
  --binary-files=without-match
  --exclude-dir='.git'
  --exclude-dir='target'
)

has_kdv_reference=false
reference_locations=()

record_reference_location() {
  local location="$1"
  reference_locations+=("${location}")
}

has_reference_in_file() {
  local marker="$1"
  local target="$2"
  grep -n --fixed-strings -- "${marker}" "${target}" >/dev/null 2>&1
}

has_marker_in_scope() {
  local marker="$1"
  if has_reference_in_file "${marker}" "${manifest_file}"; then
    return 0
  fi
  if [[ -d "${kdv_repo}/crates" ]] && grep -R -n "${grep_args[@]}" -- "${marker}" "${kdv_repo}/crates" >/dev/null 2>&1; then
    return 0
  fi
  if [[ -d "${kdv_repo}/tools" ]] && grep -R -n "${grep_args[@]}" -- "${marker}" "${kdv_repo}/tools" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

for marker in "${kdv_reference_markers[@]}"; do
  if has_reference_in_file "${marker}" "${manifest_file}"; then
    has_kdv_reference=true
    record_reference_location "${manifest_file}"
  fi
done

for crate_path in \
  "${kdv_repo}/crates/katana-language-editor" \
  "${kdv_repo}/tools/katana-language-editor" \
  "${kdv_repo}/crates/katana_language_editor" \
  "${kdv_repo}/tools/katana_language_editor"
do
  if [[ -d "${crate_path}" ]]; then
    has_kdv_reference=true
    record_reference_location "${crate_path}"
  fi
done

for marker in "${kdv_reference_markers[@]}"; do
  if [[ -d "${kdv_repo}/crates" ]] && grep -R -n "${grep_args[@]}" -- "${marker}" "${kdv_repo}/crates" >/dev/null 2>&1; then
    has_kdv_reference=true
    record_reference_location "${kdv_repo}/crates"
  fi
  if [[ -d "${kdv_repo}/tools" ]] && grep -R -n "${grep_args[@]}" -- "${marker}" "${kdv_repo}/tools" >/dev/null 2>&1; then
    has_kdv_reference=true
    record_reference_location "${kdv_repo}/tools"
  fi
done

missing_markers=()
for marker in "${required_markers[@]}"; do
  if ! has_marker_in_scope "${marker}"; then
    missing_markers+=("${marker}")
  fi
done

if [[ "${#missing_markers[@]}" -eq 0 ]]; then
  followup_status="follow-up not required"
else
  followup_status="follow-up required"
fi

reference_locations_text="$(printf "%s\n" "${reference_locations[@]}" | sed '/^$/d' | sort -u | tr '\n' ' ' | sed 's/[[:space:]]*$//')"
if [[ -z "${reference_locations_text}" ]]; then
  reference_locations_text="none"
fi

mkdir -p "$(dirname "${output_path}")"

if [[ "${followup_status}" == "follow-up not required" ]]; then
  cat > "${output_path}" <<EOF
# KDV preset follow-up for ${version}

All required KDV preset markers are present in this KDV version.

- version: ${version}
- version_bare: ${version_bare}
- KDV repository: ${kdv_repo}
- status: follow-up not required
- required markers: ${required_markers[*]}
- KDV reference markers found: ${has_kdv_reference}
- KDV reference locations: ${reference_locations_text}

## completion audit
EOF
  for marker in "${required_markers[@]}"; do
    printf -- '- `%s`: found\n' "${marker}" >> "${output_path}"
  done

  cat >> "${output_path}" <<'EOF'
必要な follow-up が見つからなかった場合、このファイルは新規実装要件の起点メモとして残します。
EOF
else
  title="KDV preset follow-up for katana-language-editor ${version}"
  cat > "${output_path}" <<EOF
# Issue draft

title: ${title}

## 背景
Katana Language Editor のリリース ${version}（${version_bare}）で KDV 側の preset 連携で不足 API を確認したため、必要な follow-up の起点として出力します。
KDV 参照有無（文字列ベース）: ${has_kdv_reference}
KDV 参照箇所: ${reference_locations_text}
status: follow-up required

## 必要 API
- \`strings::en()\`
- \`locale::en_ltr()\`
- \`settings::default_editor()\`

## missing required markers
EOF
  if [[ "${#missing_markers[@]}" -eq 0 ]]; then
    echo "- none" >> "${output_path}"
  else
    for marker in "${required_markers[@]}"; do
      if [[ " ${missing_markers[*]} " == *" ${marker} "* ]]; then
        printf -- '- `%s`\n' "${marker}" >> "${output_path}"
      fi
    done
  fi

  cat >> "${output_path}" <<'EOF'

## completion audit
EOF

  for marker in "${required_markers[@]}"; do
    if [[ " ${missing_markers[*]} " == *" ${marker} "* ]]; then
      printf -- '- `%s`: missing\n' "${marker}" >> "${output_path}"
    else
      printf -- '- `%s`: found\n' "${marker}" >> "${output_path}"
    fi
  done

  cat >> "${output_path}" <<EOF

## Optional adapter helpers（必要に応じて）
- KDV theme / typography / spacing を 'katana_language_editor::EditorConfig' に渡すための adapter helper
- KDV settings state から 'katana_language_editor::EditorSettings' を生成する helper
- KDV Storybook から KLE preset を使った live harness / contract test を起動する helper

## 検証コマンド候補
\`\`\`bash
cd "${kdv_repo}"
cargo check --workspace --locked
rg -n "katana-language-editor|katana_language_editor" Cargo.toml crates tools
rg -n "strings::en\\(\\)|locale::en_ltr\\(\\)|settings::default_editor\\(\\)" src crates tools
cargo test --workspace --locked
\`\`\`

## Issue作成コマンド例
\`\`\`bash
gh issue create --repo HiroyukiFuruno/katana-document-viewer --title "${title}" --body-file "${output_path}"
\`\`\`
EOF
fi

echo "kdv preset follow-up artifact generated: ${output_path}"
