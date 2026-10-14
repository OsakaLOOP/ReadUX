"""在独立数据目录验证来源故障、单元重试、进程恢复与损坏缓存恢复。"""

import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

from test_reader_pipeline import ROOT


class Service:
    def __init__(self, binary, environment):
        self.process = subprocess.Popen(
            [str(binary), "stdio"], cwd=ROOT, env=environment,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, encoding="utf-8",
        )

    def request(self, command, **fields):
        self.process.stdin.write(json.dumps({"command": command, **fields}, ensure_ascii=False) + "\n")
        self.process.stdin.flush()
        response = json.loads(self.process.stdout.readline())
        if response.get("error"):
            raise RuntimeError(f"{command}: {response['error']}")
        return response["result"]

    def close(self):
        self.process.stdin.close()
        try:
            self.process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            if os.name == "nt":
                subprocess.run(["taskkill", "/PID", str(self.process.pid), "/T", "/F"], check=True, capture_output=True)
            else:
                self.process.kill()
            self.process.wait(timeout=10)
        self.process.stdout.close()
        self.process.stderr.close()


def settle(service, update):
    deadline = time.monotonic() + 180
    while time.monotonic() < deadline:
        update = service.request("sync_document", session_id=update["session_id"])
        if not update["progress"]["pending"]:
            return update
        time.sleep(0.1)
    raise TimeoutError("恢复验证超过 180 秒")


def control_fields(update):
    return {key: update[key] for key in ("session_id", "text_version", "generation")}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/readux-nlp.exe")
    parser.add_argument("--portable", type=Path, default=ROOT / "packages/ReadUX-insider-portable-win64")
    parser.add_argument("--report", type=Path, default=ROOT / ".agents/analysis/reader-recovery.json")
    args = parser.parse_args()
    portable = args.portable.resolve()
    python = next(path for path in (portable / "python/python.exe", portable / "python/Scripts/python.exe") if path.is_file())
    text = "".join(f"検証番号{time.time_ns()}第{i}。七日は警察署へ向かった。彼女は窓の外を見ている。\n" for i in range(36))
    report = {}
    with tempfile.TemporaryDirectory(prefix="readux-recovery-") as temporary:
        data = Path(temporary)
        shutil.copytree(portable / "dict-sources", data / "dict-sources")
        # 复用已有查询数据库；后端继续核验源包版本，恢复测试集中验证分析生命周期。
        shutil.copytree(ROOT / "data/dicts", data / "dicts")
        environment = {**os.environ, "READUX_DATA_DIR": str(data),
                       "READUX_UNIDIC_CWJ": str(portable / "nlp/cwj.dic"),
                       "READUX_UNIDIC_CSJ": str(portable / "nlp/csj.dic")}
        config = {"ginza": {"python": str(data / "missing-python.exe"), "model": "ja_ginza", "enabled": True,
                            "timeout_seconds": 120, "dictionary": ""}, "analysis_timing_enabled": True}
        config_path = data / "providers.local.json"
        config_path.write_text(json.dumps(config, ensure_ascii=False), encoding="utf-8")
        service = Service(args.binary.resolve(), environment)
        try:
            opened = service.request("open_document", text=text, policy="auto")
            failed = settle(service, opened["update"])
            total = failed["progress"]["total"]
            assert total > 1 and failed["progress"]["failed"] == total, failed["progress"]
            assert all("无法启动" in unit["error"] for unit in failed["changes"])
            report["missing_python"] = failed["progress"]
            config["ginza"]["python"] = str(python)
            config_path.write_text(json.dumps(config, ensure_ascii=False), encoding="utf-8")
            selected = failed["changes"][0]
            retry = service.request("retry_document", **control_fields(failed), unit_id=selected["unit_id"])
            assert retry["generation"] == failed["generation"] + 1 and retry["snapshot"]
            reset = next(unit for unit in retry["changes"] if unit["unit_id"] == selected["unit_id"])
            assert reset["stage"] == "pending" and reset["error"] is None
            assert reset["artifact_revision"] > selected["artifact_revision"]
            partial = settle(service, retry)
            assert partial["progress"]["complete"] == 1 and partial["progress"]["failed"] == total - 1, partial["progress"]
            report["single_retry"] = partial["progress"]
            complete = settle(service, service.request("retry_document", **control_fields(partial), unit_id=None))
            assert complete["progress"]["complete"] == total and complete["progress"]["failed"] == 0
            report["retry_all"] = complete["progress"]
            status = service.request("provider_status")
            provider_pid = status["providers"][0]["pid"]
            assert provider_pid
            os.kill(provider_pid, signal.SIGTERM)
            restart = service.request("open_document", text="進程復旧検証。窓の外へ歩いている。" + str(time.time_ns()), policy="auto")
            restarted = settle(service, restart["update"])
            assert restarted["progress"]["complete"] == restarted["progress"]["total"] and not restarted["progress"]["failed"]
            assert service.request("provider_status")["providers"][0]["pid"] != provider_pid
            report["process_restart"] = restarted["progress"]
            service.request("close_document", session_id=restarted["session_id"])
        finally:
            service.close()
        cache_files = list((data / "analysis-cache").glob("*.msgpack"))
        assert cache_files
        # 故障注入仅修改临时数据目录，用户书库和正式缓存保持独立。
        for path in cache_files:
            path.write_bytes(b"broken-cache")
        service = Service(args.binary.resolve(), environment)
        try:
            opened = service.request("open_document", text=text, policy="auto")
            recovered = settle(service, opened["update"])
            assert recovered["progress"]["complete"] == total and not recovered["progress"]["failed"]
            assert all(not unit["cache_hit"] for unit in recovered["changes"])
            report["corrupt_cache"] = recovered["progress"]
        finally:
            service.close()
        service = Service(args.binary.resolve(), environment)
        try:
            opened = service.request("open_document", text=text, policy="auto")
            warm = settle(service, opened["update"])
            assert warm["progress"]["complete"] == total and not warm["progress"]["failed"]
            assert all(unit["cache_hit"] for unit in warm["changes"])
            report["restart_cache"] = warm["progress"]
        finally:
            service.close()
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"通过：{total} 个单元；缺失解释器、单元重试、全部重试、进程重启、损坏缓存及重启缓存。报告：{args.report}")


if __name__ == "__main__":
    main()
