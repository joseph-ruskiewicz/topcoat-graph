#!/usr/bin/env bash
# Formats the registry's component sources and reinstalls them into the docs site.
# Run it after editing anything under crates/topcoat-graph/registry/components.
#
# Only the topcoat-graph components are reinstalled (`--registry topcoat-graph`). The
# built-in Topcoat components the site's pages use, such as button and sheet, are left
# as they are.
#
# Edit the registry sources, never the copies in site/src/components: this overwrites
# those. site/tests/registry_sync.rs fails when the two drift apart.
set -euo pipefail
cd "$(dirname "$0")/.."

rustfmt --edition 2024 crates/topcoat-graph/registry/components/*.rs
(cd crates/topcoat-graph && topcoat fmt)
(cd site && topcoat ui add --all --registry topcoat-graph --overwrite && topcoat fmt)
cargo fmt
