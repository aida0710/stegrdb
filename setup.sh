#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
if ! command -v cargo >/dev/null; then
  echo "Rustをインストールしてください。手順はreadme.mdにあります。" >&2
  exit 1
fi
if [ ! -e stegrdb.toml ]; then cp stegrdb.example.toml stegrdb.toml; fi
cargo build --release --locked
echo "stegrdb.tomlとDB接続情報を設定してください。起動手順はreadme.mdにあります。"
