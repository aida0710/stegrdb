#!/usr/bin/env bash
set -euo pipefail
cd /opt/stegrdb-lab
node=$1
case "$node" in a|b|c) ;; *) exit 1 ;; esac
systemctl stop stegrdb.service 2>/dev/null || true
if ! id stegrdb >/dev/null 2>&1; then useradd --system --shell /usr/sbin/nologin stegrdb; fi
chmod 755 stegrdb
chmod 600 postgres.env
printf '%s\n' "$node" > node
cat > stegrdb.toml <<EOF
node_id = "vm-$node"
channel = "vm-lab"
interface = "relay0"
promiscuous = true
[relay]
plugin = "postgres"
[firewall]
policy = "blacklist"
rules = []
EOF

if [[ $node == a ]]; then
  pg_conftool 16 main set listen_addresses '*'
  # QEMUのhost forwardingはゲストから10.0.2.2に見える。専用DB・専用ロールだけ許可する。
  rule='host stegrdb stegrdb 10.0.2.2/32 scram-sha-256'
  if ! grep -qF "$rule" /etc/postgresql/16/main/pg_hba.conf; then
    printf '%s\n' "$rule" >> /etc/postgresql/16/main/pg_hba.conf
  fi
  systemctl restart postgresql
  set -a
  source postgres.env
  set +a
  python3 - <<'PY'
import os
import re
import subprocess

password = os.environ["STEGRDB_LAB_PASSWORD"]
assert re.fullmatch(r"[a-f0-9]{48}", password)
prefix = ["runuser", "-u", "postgres", "--", "psql", "-v", "ON_ERROR_STOP=1"]
sql = "DO $$ BEGIN CREATE ROLE stegrdb LOGIN; EXCEPTION WHEN duplicate_object THEN NULL; END $$;\n"
sql += f"ALTER ROLE stegrdb PASSWORD '{password}';\n"
subprocess.run(prefix, input=sql, text=True, check=True, stdout=subprocess.DEVNULL)
exists = subprocess.check_output(prefix + ["-Atc", "SELECT 1 FROM pg_database WHERE datname='stegrdb'"], text=True)
if not exists.strip():
    subprocess.run(["runuser", "-u", "postgres", "--", "createdb", "-O", "stegrdb", "stegrdb"], check=True)
PY
  # オブジェクトの所有者も接続ロールに揃える。
  PGPASSWORD="$STEGRDB_LAB_PASSWORD" psql -h 127.0.0.1 -U stegrdb -d stegrdb -v ON_ERROR_STOP=1 -f schema.sql
fi
install -m 644 stegrdb.service stegrdb-network.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now stegrdb-network.service stegrdb.service
printf 'VM %s: 中継を配備しました\n' "$node"
