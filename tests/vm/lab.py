"""ローカルの3台のKVMを再現可能なテスト環境として管理するCLI。"""

import argparse
import fcntl
import os
import shutil
import subprocess
import sys

import images
import qemu
import remote
import scenarios
from settings import NODES, ROOT, SSH_PORTS, STATE


def up():
    subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
    password = images.prepare_credentials()
    image = images.prepare_image()
    # まずDBを持つ1台を起動し、その準備が済んでから3台へ広げる。
    for node in NODES:
        images.seed_guest(node, image)
        qemu.start(node)
        qemu.wait_ready(node)
        remote.deploy(node, password)
    print("VMラボを起動しました。scripts/vm-lab testで通信を検証できます。", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("up", "test", "status", "ssh", "down", "destroy"))
    parser.add_argument("node", nargs="?", choices=NODES)
    options = parser.parse_args()
    os.umask(0o077)
    STATE.mkdir(mode=0o700, parents=True, exist_ok=True)
    # up/down/testの同時実行でディスクやサービスを入れ替えない。
    with (STATE / "lab.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if options.action == "up":
            up()
        elif options.action == "test":
            scenarios.run_tests()
        elif options.action == "status":
            for node in NODES:
                print(f"{node}: {'running' if qemu.is_running(node) else 'stopped'} (SSH 127.0.0.1:{SSH_PORTS[node]})")
        elif options.action == "ssh":
            if options.node is None:
                parser.error("sshにはa/b/cを指定してください")
            # 対話SSHがラボの管理ロックを保持しないよう、exec前に解放する。
            fcntl.flock(lock, fcntl.LOCK_UN)
            os.execvp("ssh", remote.ssh_arguments(options.node))
        elif options.action in ("down", "destroy"):
            for node in reversed(NODES):
                qemu.stop(node)
            if options.action == "destroy":
                # このリポジトリ内の専用ディスクだけを削除する。キャッシュと試験記録は残す。
                for node in NODES:
                    directory = STATE / node
                    if directory.exists():
                        shutil.rmtree(directory)
                for filename in ("id_ed25519", "id_ed25519.pub", "known_hosts", "postgres-password"):
                    (STATE / filename).unlink(missing_ok=True)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        print(error.stdout or "", file=sys.stderr)
        print(error.stderr or "", file=sys.stderr)
        raise SystemExit(f"コマンドが失敗しました（終了コード{error.returncode}）")
