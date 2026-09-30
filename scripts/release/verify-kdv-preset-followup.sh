#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

version_stdout="$(bash "${SCRIPT_DIR}/verify-version.sh" "${1:-}")"
version=""
while IFS= read -r line; do
  case "${line}" in
    version=*)
      version="${line#version=}"
      ;;
  esac
done <<< "${version_stdout}"

if [[ -z "${version}" ]]; then
  echo "Failed to normalize version from verify-version output." >&2
  exit 1
fi

KDV_REPO="${KDV_REPO:-../katana-document-viewer}"
KDV_REPO_NAME="${KDV_REPO_NAME:-HiroyukiFuruno/katana-document-viewer}"

if [[ -n "${KDV_FOLLOWUP_OUTPUT:-}" ]]; then
  followup_output="${KDV_FOLLOWUP_OUTPUT}"
else
  followup_output="target/release/kdv-preset-followup-${version}.md"
fi

bash "${SCRIPT_DIR}/prepare-kdv-preset-followup.sh" "${version}" "${KDV_REPO}" "${followup_output}"

if [[ ! -f "${followup_output}" ]]; then
  echo "Follow-up artifact not found: ${followup_output}" >&2
  exit 1
fi

if grep -q 'status: follow-up not required' "${followup_output}"; then
  echo "PASS: KDV preset follow-up status is not required for ${version}"
  exit 0
fi

if ! grep -q 'status: follow-up required' "${followup_output}"; then
  echo "Follow-up artifact does not declare a required or not-required status: ${followup_output}" >&2
  exit 1
fi

follow_up_title="KDV preset follow-up for katana-language-editor ${version}"

if ! issues_json="$(gh issue list --repo "${KDV_REPO_NAME}" --state open --search "${follow_up_title} in:title" --json number,title,url --limit 100)"; then
  echo "Unable to list open KDV follow-up issues in ${KDV_REPO_NAME}" >&2
  exit 1
fi

if ! python_match_output="$(python3 - "${follow_up_title}" "${issues_json}" <<'PY'
import json
import sys

expected_title = sys.argv[1]
raw = sys.argv[2]
issues = json.loads(raw)

if not isinstance(issues, list):
    print("non-list response", file=sys.stderr)
    sys.exit(1)

matches = [issue for issue in issues if issue.get("title") == expected_title]

if not matches:
    print("missing")
    raise SystemExit(0)

if len(matches) >= 2:
    print("duplicate")
    for issue in matches:
        print(issue.get("url", ""))
    raise SystemExit(0)

print("ok")
print(matches[0].get("number", ""))
print(matches[0].get("url", ""))
PY
 )"; then
  echo "Failed to parse follow-up issue list response as JSON." >&2
  exit 1
fi
if [[ -z "${python_match_output}" ]]; then
  echo "Failed to evaluate follow-up issue list for ${follow_up_title}" >&2
  exit 1
fi

mapfile -t match_lines <<< "${python_match_output}"
match_status="${match_lines[0]:-}"

if [[ "${match_status}" == "missing" ]]; then
  echo "Missing follow-up issue for ${follow_up_title}" >&2
  echo "Create command example: gh issue create --repo ${KDV_REPO_NAME} --title \"${follow_up_title}\" --body-file \"${followup_output}\"" >&2
  exit 1
fi

if [[ "${match_status}" == "duplicate" ]]; then
  echo "Duplicate follow-up issues found for ${follow_up_title}" >&2
  if (( ${#match_lines[@]} > 1 )); then
    for ((i = 1; i < ${#match_lines[@]}; i++)); do
      if [[ -n "${match_lines[i]}" ]]; then
        echo "${match_lines[i]}" >&2
      fi
    done
  fi
  exit 1
fi

if [[ "${match_status}" != "ok" ]]; then
  echo "Unexpected follow-up issue matching state: ${match_status}" >&2
  echo "${python_match_output}" >&2
  exit 1
fi

follow_up_number="${match_lines[1]:-}"
follow_up_url="${match_lines[2]:-}"

if [[ -z "${follow_up_number}" ]]; then
  echo "Unable to identify follow-up issue number for ${follow_up_title}" >&2
  exit 1
fi

if ! issue_json="$(gh issue view "${follow_up_number}" --repo "${KDV_REPO_NAME}" --json number,title,url,body)"; then
  echo "Unable to read follow-up issue ${follow_up_url:-${follow_up_number}} in ${KDV_REPO_NAME}" >&2
  exit 1
fi

if ! python3 - "${follow_up_url}" "${issue_json}" <<'PY'
import sys
import json

issue_url = sys.argv[1]
issue = json.loads(sys.argv[2])
body = issue.get("body") or ""

required_markers = (
    "strings::en()",
    "locale::en_ltr()",
    "settings::default_editor()",
)

if not body.strip():
    print(f"Follow-up issue body is empty: {issue_url}", file=sys.stderr)
    for marker in required_markers:
        print(f"Missing marker: {marker}", file=sys.stderr)
    raise SystemExit(1)

missing = [marker for marker in required_markers if marker not in body]
if missing:
    print(f"Missing marker(s) in follow-up issue body: {issue_url}", file=sys.stderr)
    for marker in missing:
        print(f"Missing marker: {marker}", file=sys.stderr)
    raise SystemExit(1)
PY
then
  exit 1
fi

echo "PASS: KDV preset follow-up status and issue audit passed for ${version}"
