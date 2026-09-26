#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
exec bash plugins/postgres/scripts/test-postgres.sh
