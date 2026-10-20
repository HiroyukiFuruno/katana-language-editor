#!/bin/sh
set -eu
if [ "${KLE_RELEASE_CLEANUP_INTERNAL:-}" = "1" ]; then
    exit 0
fi
exec python3 "$(dirname "$0")/release_cleanup.py" post-merge "$@"
