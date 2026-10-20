#!/usr/bin/env bash
set -euo pipefail
exec just JOBS=2 pre-push-check
