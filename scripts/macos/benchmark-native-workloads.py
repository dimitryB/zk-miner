#!/usr/bin/env python3
"""Serial step-4 acceptance: boundary proofs, fixed 10M fixtures, and lifecycle."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import statistics
import subprocess
import sys
import threading
import time
import uuid
from proof_provenance import ROOT, sha, verify_build
from worker_provenance import verify_build as verify_worker


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, required=True)
    parser.add_argument("--stretch", action="store_true", help="also prove 46M guest cycles once")
    args = parser.parse_args()
    worker, verifier = verify_worker("native", args.engine), verify_build("verifier")
    directory = ROOT / "target/macos/native-wrap/workload-campaigns" / uuid.uuid4().hex
    directory.mkdir(parents=True)
    common = [sys.executable, str(ROOT / "scripts/macos/test-native-worker.py"), "--engine", str(args.engine)]
    cases = [
        ("limits", ["--limit-controls", "--repeat", "1", "--proof-only"]),
        ("tiny-tail", ["--workload", "segment-boundary", "--iterations", "0", "--po2", "14", "--repeat", "1", "--proof-only"]),
        ("segment-boundary", ["--workload", "segment-boundary", "--repeat", "1", "--proof-only"]),
        ("arithmetic-10m", ["--workload", "fibonacci", "--iterations", "1000000", "--min-guest-cycles", "10000000", "--repeat", "5", "--warmup", "--proof-only", "--timeout-seconds", "1800"]),
        ("hash-10m", ["--workload", "sha256-chain", "--iterations", "2262", "--min-guest-cycles", "10000000", "--repeat", "5", "--warmup", "--proof-only", "--timeout-seconds", "1800"]),
    ]
    if args.stretch:
        cases.append(("arithmetic-46m", ["--workload", "fibonacci", "--iterations", "4600000", "--min-guest-cycles", "46000000", "--repeat", "1", "--proof-only", "--timeout-seconds", "3600"]))
    cases.append(("twenty-request-lifecycle", ["--repeat", "20"]))
    for name, options in cases:
        if name in ["arithmetic-10m", "hash-10m", "arithmetic-46m", "twenty-request-lifecycle"]:
            options.extend(["--po2", "20"])
    report = {"status":"running", "started":datetime.now(timezone.utc).isoformat(),
              "worker_build":worker,"verifier_build":verifier,"script_sha256":sha(Path(__file__)),
              "cases":[{"name":name,"command":common+options,"status":"pending"} for name,options in cases]}
    (directory / "plan.json").write_text(json.dumps(report, indent=2)+"\n")
    print(directory, flush=True)
    stop = threading.Event()
    def thermal():
        binary = ROOT / "target/macos/benchmark-tools/thermal"
        with (directory / "thermal.jsonl").open("w") as out:
            while not stop.is_set():
                row = {"time":datetime.now(timezone.utc).isoformat()}
                try:
                    sample = subprocess.run([str(binary)], capture_output=True, text=True, check=True, timeout=5)
                    row["observation"] = json.loads(sample.stdout)
                except Exception as error:
                    row["error"] = str(error)
                out.write(json.dumps(row)+"\n");out.flush()
                stop.wait(15)
    thread = threading.Thread(target=thermal, daemon=True)
    thread.start()
    try:
        for row in report["cases"]:
            row.update(status="running", started=datetime.now(timezone.utc).isoformat())
            (directory / "result.json").write_text(json.dumps(report,indent=2)+"\n")
            started = time.monotonic()
            with (directory / (row["name"]+".log")).open("w") as out:
                subprocess.run(row["command"], cwd=ROOT, stdout=out, stderr=subprocess.STDOUT, check=True)
            run = Path((directory / (row["name"]+".log")).read_text().splitlines()[0])
            result = json.loads((run / "result.json").read_text())
            if result["status"] != "passed": raise RuntimeError(f"case did not pass: {row['name']}")
            row.update(status="passed", seconds=time.monotonic()-started, run=str(run), result_sha256=sha(run / "result.json"))
            if row["name"] in ["arithmetic-10m", "hash-10m"]:
                ledger = [json.loads(line) for line in (run / "acceptance/requests.jsonl").read_text().splitlines()]
                proofs = [r for r in ledger if r.get("status")=="complete_independently_verified" and r.get("warmup") is False]
                if len(proofs) != 5: raise RuntimeError("missing measured warm proofs")
                seconds = [(p["timings"]["prove_call_ms"]+p["timings"]["external_verifier_ms"])/1000 for p in proofs]
                row.update(measured_complete_seconds=seconds, median_complete_seconds=statistics.median(seconds), useful_target_passed=statistics.median(seconds)<=600)
            (directory / "result.json").write_text(json.dumps(report,indent=2)+"\n")
            print(row["name"], "passed", flush=True)
        if verify_worker("native", args.engine) != worker or verify_build("verifier") != verifier:
            raise RuntimeError("build provenance changed during campaign")
        report.update(status="completed", useful_target_passed=all(r["useful_target_passed"] for r in report["cases"] if "useful_target_passed" in r))
    except BaseException as error:
        report.update(status="failed", error=str(error))
        for row in report["cases"]:
            if row["status"] == "running": row["status"] = "failed"
        raise
    finally:
        stop.set();thread.join(timeout=6)
        report["finished"] = datetime.now(timezone.utc).isoformat()
        (directory / "result.json").write_text(json.dumps(report,indent=2)+"\n")


if __name__ == "__main__": main()
