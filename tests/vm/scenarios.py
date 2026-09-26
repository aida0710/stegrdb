"""3台のゲスト間で、DBを経由した実通信と停止中の配送待ちを確認する。"""

from datetime import datetime
import json
import time
import uuid

import qemu
import remote
from settings import NODES, POLL_INTERVAL_SECONDS, PROBE_TIMEOUT_SECONDS, ROOT

PROBE = "sudo ip netns exec client python3 /opt/stegrdb-lab/probe.py"
# MTU上限のICMPを3回送り、一部だけ届く状態も失敗にする。
ICMP_PACKETS = 3
ICMP_BYTES = 1472


def wait_for(check, description):
    deadline = time.monotonic() + PROBE_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        if check():
            return
        time.sleep(POLL_INTERVAL_SECONDS)
    raise TimeoutError(description)


def database_count(sql):
    # SQLはこのファイルの固定文字列だけを渡す。
    completed = remote.run("a", f'sudo -u postgres psql -d stegrdb -Atc "{sql}"')
    return int(completed.stdout.strip())


def prepare_receivers():
    for node in NODES:
        if not qemu.is_running(node):
            raise RuntimeError("先にscripts/vm-lab upを実行してください")
        wait_for(lambda: remote.run(node, "systemctl is-active --quiet stegrdb", check=False).returncode == 0,
                 f"VM {node}の中継が起動していません")
        remote.run(node, "sudo systemctl stop stegrdb-probe 2>/dev/null || true; sudo rm -f /run/stegrdb-probe.ready")
        remote.run(node, "sudo systemd-run --unit=stegrdb-probe --collect "
                        "--property=NetworkNamespacePath=/run/netns/client "
                        "/usr/bin/python3 /opt/stegrdb-lab/probe.py serve")
        wait_for(lambda: remote.run(node, "test -f /run/stegrdb-probe.ready", check=False).returncode == 0,
                 f"VM {node}の受信サーバが起動していません")
    wait_for(lambda: database_count("SELECT count(*) FROM stegrdb_relay.nodes WHERE channel='vm-lab'") == 3,
             "3ノードの登録が揃いません")


def check_connectivity():
    reports = []
    for source, target in (("a", "192.0.2.12"), ("b", "192.0.2.13"), ("c", "192.0.2.11")):
        ping = remote.run(source, f"sudo ip netns exec client env LC_ALL=C ping "
                                 f"-c {ICMP_PACKETS} -W 5 -M do -s {ICMP_BYTES} {target}")
        assert f"{ICMP_PACKETS} packets transmitted, {ICMP_PACKETS} received" in ping.stdout, ping.stdout
        print(ping.stdout, end="", flush=True)
        tcp = json.loads(remote.run(source, f"{PROBE} tcp {target}").stdout)
        udp = json.loads(remote.run(source, f"{PROBE} udp {target} {uuid.uuid4().hex}").stdout)
        reports.append({"source": source, "target": target, "icmp_packets": ICMP_PACKETS,
                        "icmp_payload_bytes": ICMP_BYTES, "tcp": tcp, "udp_packets": len(udp)})
    return reports


def check_offline_delivery():
    token = uuid.uuid4().hex
    # 停止試験中のARP再解決に結果を左右させず、UDPの未処理キューを直接検証する。
    remote.run("a", "sudo ip netns exec client ip neigh replace 192.0.2.13 "
                    "lladdr 02:00:00:00:00:13 nud permanent dev client0")
    remote.run("c", "sudo systemctl stop stegrdb")
    try:
        expected = json.loads(remote.run("a", f"{PROBE} send-udp 192.0.2.13 {token}").stdout)
        wait_for(lambda: database_count("SELECT count(*) FROM stegrdb_relay.pending "
                                        "WHERE channel='vm-lab' AND node_id='vm-c'") >= len(expected),
                 "停止ノード向けのフレームがDBに揃いません")
        before = json.loads(remote.run("c", f"{PROBE} received {token}").stdout)
        assert not before, "中継停止中に別経路で届いています"
    finally:
        remote.run("c", "sudo systemctl start stegrdb")
    wait_for(lambda: json.loads(remote.run("c", f"{PROBE} received {token}").stdout) == expected,
             "再起動後のUDPペイロードが送信内容と一致しません")
    wait_for(lambda: database_count("SELECT count(*) FROM stegrdb_relay.pending "
                                    "WHERE channel='vm-lab' AND node_id='vm-c'") == 0,
             "配送後にACKされていないフレームがあります")
    return {"recovered_udp_packets": len(expected), "payload_sha256_verified": True}


def run_tests():
    timestamp = datetime.now().astimezone()
    destination = ROOT / "artifacts/vm" / timestamp.strftime("%Y-%m-%d/%H%M%S")
    destination.mkdir(parents=True)
    report = {"started_at": timestamp.isoformat(), "status": "failed"}
    try:
        prepare_receivers()
        report["connectivity"] = check_connectivity()
        report["offline_delivery"] = check_offline_delivery()
        report["status"] = "passed"
        print("3台のVM: ICMP・TCP・UDP・停止後の再配送が成功しました", flush=True)
    finally:
        (destination / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        for node in NODES:
            try:
                remote.collect_logs(node, destination)
            except Exception as error:
                print(f"VM {node}のログ取得に失敗: {type(error).__name__}", flush=True)
        print(f"試験記録: {destination}", flush=True)
