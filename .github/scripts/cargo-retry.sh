#!/usr/bin/env bash
# Runs a cargo command, and runs it again (up to 3 times) only when site/build.rs failed.
#
# The build script downloads Tailwind from GitHub releases. On this runner Topcoat does
# not find its shared download cache, so each build script run downloads again, and a
# passing 5xx from GitHub fails the step. Any other failure, such as a lint or a test,
# fails at once.
set -uo pipefail

log=$(mktemp)
for attempt in 1 2 3; do
  "$@" 2>&1 | tee "$log"
  status=${PIPESTATUS[0]}
  if [ "$status" -eq 0 ]; then
    exit 0
  fi
  if ! grep -q 'failed to run custom build command for `site' "$log"; then
    exit "$status"
  fi
  echo "::warning::site/build.rs failed (attempt $attempt); retrying in 15 seconds"
  sleep 15
done
exit "$status"
