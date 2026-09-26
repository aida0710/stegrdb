"""専用SSH鍵によるゲスト内コマンド実行とファイル配備。"""

import shlex
import subprocess
import tarfile

from settings import DEPLOY_TIMEOUT_SECONDS, REMOTE_TIMEOUT_SECONDS, ROOT, SSH_PORTS, SSH_TIMEOUT_SECONDS, STATE


def ssh_arguments(node):
    return [
        "ssh", "-i", str(STATE / "id_ed25519"),
        "-o", "IdentitiesOnly=yes", "-o", "BatchMode=yes",
        "-o", f"ConnectTimeout={SSH_TIMEOUT_SECONDS}",
        "-o", "StrictHostKeyChecking=accept-new",
        "-o", f"UserKnownHostsFile={STATE / 'known_hosts'}",
        "-p", str(SSH_PORTS[node]), "ubuntu@127.0.0.1",
    ]


def run(node, command, *, input_text=None, check=True, timeout=REMOTE_TIMEOUT_SECONDS):
    return subprocess.run(
        ssh_arguments(node) + [command], input=input_text, text=True,
        capture_output=True, check=check, timeout=timeout,
    )


def deploy(node, password):
    from settings import DATABASE_PORT

    directory = STATE / node
    run(node, "sudo systemctl stop stegrdb.service 2>/dev/null || true")
    host = "127.0.0.1" if node == "a" else "10.0.2.2"
    port = 5432 if node == "a" else DATABASE_PORT
    environment = directory / "postgres.env"
    environment.write_text(
        f'STEGRDB_POSTGRES_URL="host={host} port={port} user=stegrdb '
        f'dbname=stegrdb password={password} sslmode=disable"\n'
        f'STEGRDB_LAB_PASSWORD={password}\n'
    )
    environment.chmod(0o600)
    archive = directory / "deploy.tar"
    with tarfile.open(archive, "w") as bundle:
        for filename in ("configure.sh", "network.sh", "probe.py", "stegrdb.service", "stegrdb-network.service"):
            bundle.add(ROOT / "tests/vm/guest" / filename, arcname=filename)
        bundle.add(ROOT / "target/release/stegrdb", arcname="stegrdb")
        bundle.add(ROOT / "plugins/postgres/schema.sql", arcname="schema.sql")
        bundle.add(environment, arcname="postgres.env")
    archive.chmod(0o600)
    try:
        with archive.open("rb") as stream:
            subprocess.run(
                ssh_arguments(node) + ["sudo mkdir -p /opt/stegrdb-lab && sudo tar -x -C /opt/stegrdb-lab"],
                stdin=stream, check=True, timeout=REMOTE_TIMEOUT_SECONDS,
            )
        completed = run(node, f"sudo bash /opt/stegrdb-lab/configure.sh {shlex.quote(node)}", timeout=DEPLOY_TIMEOUT_SECONDS)
        print(completed.stdout, end="", flush=True)
    finally:
        archive.unlink(missing_ok=True)


def collect_logs(node, destination):
    completed = run(
        node, "sudo journalctl -u stegrdb --no-pager -n 200; sudo systemctl status stegrdb --no-pager",
        check=False,
    )
    (destination / f"node-{node}.log").write_text(completed.stdout + completed.stderr)
