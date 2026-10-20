#!/usr/bin/env bash
set -euo pipefail
exec bash scripts/governance/validate-commit-message.sh "$@"
