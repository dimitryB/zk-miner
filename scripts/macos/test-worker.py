#!/usr/bin/env python3
"""Bounded local WorkerPool acceptance with real proofs and preserved failure evidence."""
import argparse
import fcntl
import json
import os
import re
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
import uuid

from proof_provenance import sha, verify_build as verify_verifier
from worker_provenance import verify_build as verify_worker, paths as worker_paths
from smoke import DIGEST, IMAGE, validate_proof_evidence

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "target/macos"


def test_sources():
    files = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", Path(__file__).resolve()]
    for crate in ("zkminer-prover", "zkminer-prover-protocol", "zkminer-config"):
        for path in (ROOT / "crates" / crate).rglob("*"):
            if path.is_file() and "target" not in path.relative_to(ROOT).parts:
                files.append(path)
    return {str(path.relative_to(ROOT)): sha(path) for path in sorted(set(files))}


def build_acceptance(attempt, env):
    before = test_sources()
    command = ["cargo", "+1.93.0", "test", "--locked", "-p", "zkminer-prover", "--test", "macos_worker",
               "--no-run", "--message-format=json", "--target-dir", str(BASE / "worker-acceptance-build")]
    with (attempt / "test-build.jsonl").open("w") as out, (attempt / "test-build.stderr").open("w") as err:
        subprocess.run(command, cwd=ROOT, env=env, stdout=out, stderr=err, timeout=600, check=True)
    if test_sources() != before:
        raise RuntimeError("WorkerPool acceptance source inputs changed during the test build")
    binaries = set()
    for line in (attempt / "test-build.jsonl").read_text().splitlines():
        record = json.loads(line)
        if record.get("reason") == "compiler-artifact" and record.get("target", {}).get("name") == "macos_worker" and record.get("executable"):
            binaries.add(record["executable"])
    if len(binaries) != 1:
        raise RuntimeError("build did not identify exactly one real macOS acceptance test executable")
    binary = Path(binaries.pop())
    manifest = {"source_inputs": before, "binary_path": str(binary), "binary_sha256": sha(binary), "command": command}
    (attempt / "acceptance-build.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return binary, manifest


def read_sidecars(directory, tolerate_errors=False):
    records = []
    for path in sorted(directory.glob("*.json")):
        try:
            records.append(json.loads(path.read_text()))
        except (ValueError, OSError):
            if not tolerate_errors:
                raise
    return records


def owned_process(pid, parent):
    result = subprocess.run(["ps", "-o", "ppid=,pgid=", "-p", str(pid)], capture_output=True, text=True, timeout=5)
    parts = result.stdout.split()
    return result.returncode == 0 and parts == [str(parent), str(pid)]


def validate_results(attempt, args, expected):
    acceptance = attempt / "acceptance"
    summary = json.loads((acceptance / "complete.json").read_text())
    count = args.repeat + (0 if args.proof_only else 1)
    if (summary.get("status") != "complete_independently_verified" or summary.get("verified_proofs") != count
            or summary.get("requested_mode") != args.mode or summary.get("observed_mode") != expected
            or summary.get("shutdown") is not True):
        raise RuntimeError("missing completed real-proof acceptance summary")
    records = [json.loads(path.read_text()) for path in sorted(acceptance.glob("proof-*/worker.json"))]
    if len(records) != count:
        raise RuntimeError("verified proof count differs from saved worker evidence")
    for path in sorted(acceptance.glob("proof-*/worker.json")):
        record = json.loads(path.read_text())
        verification = json.loads(path.with_name("verification.json").read_text())
        if (record.get("phase") != "complete" or record.get("requested_mode") != args.mode
                or record.get("observed_mode") != expected or record.get("seal_sha256") != sha(path.with_name("seal.bin"))
                or record.get("journal_hex") != "6d1a000000000000"
                or verification.get("image_id") != record.get("image_id") or verification.get("verification") != "passed"
                or verification.get("journal_hex") != "6d1a000000000000"
                or set(verification.get("negative_tests", [])) != {"wrong_image", "modified_journal", "corrupt_seal"}):
            raise RuntimeError("worker proof binding or unchanged independent verification evidence is incomplete")
        validate_proof_evidence(dict(record, requested_mode=expected), expected)
        if args.mode == "auto" and expected == "cpu" and not record.get("fallback_reason"):
            raise RuntimeError("auto CPU fallback lacks a recorded reason")
    if not args.proof_only:
        ledger = [json.loads(line) for line in (acceptance / "requests.jsonl").read_text().splitlines()]
        for case in ("helper-cancel", "helper-timeout"):
            results = [entry for entry in ledger if entry.get("case") == case and entry.get("status") == "expected_failure"]
            if len(results) != 1 or not isinstance(results[0].get("observed_running_helper"), dict):
                raise RuntimeError(f"{case} lacks a failure after observing its owned running helper")
            partial = results[0]["observed_running_helper"]
            pid = partial.get("worker_pid")
            log = (attempt / "test.log").read_text()
            if case == "helper-timeout":
                if (results[0].get("elapsed_seconds", 0) < args.failure_timeout
                        or not re.search(r"Proving timeout \([^)]*\) exceeded for risc0:generic, killing worker PID " + str(pid) + r"\b", log)):
                    raise RuntimeError("helper timeout lacks the production watchdog's exact worker-PID evidence")
            elif f"Cancelling proof on risc0:generic by killing worker PID {pid}" not in log:
                raise RuntimeError("helper cancellation lacks the supervisor's exact worker-PID evidence")
            if f"not killing PID {pid} for risc0:generic" in log:
                raise RuntimeError("supervisor declined the worker identity check during the lifecycle case")
            if (partial.get("phase") != "wrapping" or partial.get("proof_kind") != "succinct"
                    or partial.get("observed_mode") != expected or partial.get("requested_mode") != args.mode):
                raise RuntimeError(f"{case} was not observed during the selected request's wrapping phase")
            stage = partial.get("observed_stages", {}).get("metal_stage")
            if expected == "metal":
                if (not isinstance(stage, dict) or stage.get("completed_dispatches", 0) <= 0
                        or len(stage.get("segments", [])) != partial.get("stats", {}).get("segments")):
                    raise RuntimeError(f"{case} lacks completed Metal stage evidence before wrapping")
            elif stage is not None:
                raise RuntimeError("CPU lifecycle case unexpectedly reports Metal work")
        if not all(summary.get(key) is True for key in ("helper_cancel", "helper_timeout", "restart")):
            raise RuntimeError("lifecycle completion evidence is missing")
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", required=True, type=Path)
    parser.add_argument("--worker-kind", choices=["cpu", "metal"], required=True)
    parser.add_argument("--mode", choices=["cpu", "metal", "auto"], required=True)
    parser.add_argument("--context", required=True)
    parser.add_argument("--repeat", type=int, default=3)
    parser.add_argument("--proof-only", action="store_true", help="verify selected mode without the separate cancellation/timeout/restart cases")
    parser.add_argument("--timeout", type=int, default=3600, help="total acceptance runtime bound, excluding test compilation")
    parser.add_argument("--failure-timeout", type=int, default=70)
    parser.add_argument("--staging-bytes", type=int, default=524288)
    args = parser.parse_args()
    if not (1 <= args.repeat <= 100 and 1 <= args.timeout <= 14400 and 1 <= args.failure_timeout <= 300
            and 4096 <= args.staging_bytes <= 16777216):
        parser.error("repeat 1..100, timeout 1..14400, failure timeout 1..300, staging bytes 4096..16777216 required")
    if not args.proof_only and args.repeat < 3:
        parser.error("lifecycle acceptance requires at least three complete repeat proofs")
    if args.mode == "metal" and args.worker_kind != "metal":
        parser.error("forced Metal proving requires a Metal-capable worker build")
    for name in ("RISC0_DEV_MODE", "RISC0_SKIP_BUILD", "RISC0_SKIP_BUILD_KERNELS", "RISC0_SKIP_VERIFY", "DOCS_RS",
                 "RISC0_PROVER", "RISC0_EXECUTOR", "RISC0_SERVER_PATH", "RISC0_API_URL", "BONSAI_API_KEY", "BONSAI_API_URL", "RISC0_TEST_INJECT_INVALID"):
        if name in os.environ:
            parser.error(f"unset {name}; real acceptance rejects proving overrides")
    expected = "metal" if args.worker_kind == "metal" and args.mode != "cpu" else "cpu"
    docker = shutil.which("docker")
    if not docker:
        parser.error("Docker is required for the dedicated local wrapping helper")
    docker_cmd = [docker, "--context", args.context]
    endpoint = subprocess.check_output([docker, "context", "inspect", args.context, "--format", "{{.Endpoints.docker.Host}}"], text=True, timeout=10).strip()
    if not endpoint.startswith("unix://"):
        parser.error("only a dedicated local Unix Docker endpoint is supported")
    info = json.loads(subprocess.check_output(docker_cmd + ["image", "inspect", IMAGE], timeout=10))[0]
    if not any(value.endswith("@" + DIGEST) for value in info.get("RepoDigests", [])):
        parser.error("required pinned helper image digest is absent")
    runs = BASE / "worker-runs"
    runs.mkdir(parents=True, exist_ok=True)
    (BASE / "runs").mkdir(exist_ok=True)
    with (BASE / "runs/active.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        attempt = runs / uuid.uuid4().hex
        attempt.mkdir()
        sidecars = attempt / "requests"
        work = attempt / "shared-work"
        sidecars.mkdir()
        work.mkdir()
        worker = worker_paths(args.worker_kind)["binary"]
        verifier = BASE / "verifier/release/hemi-macos-proof"
        program = BASE / "guest-build/riscv32im-risc0-zkvm-elf/release/fibonacci.bin"
        report = {"id":attempt.name, "status":"failed", "requested_mode":args.mode, "expected_mode":expected,
                  "worker_kind":args.worker_kind, "repeat":args.repeat, "lifecycle":not args.proof_only,
                  "helper_arch":"linux/amd64", "helper_image":DIGEST, "helper_context":args.context,
                  "helper_endpoint":endpoint, "worker_peak_rss_bytes":{},
                  "rss_method":"owned native worker RSS sampled once per second; excludes helper VM"}
        proc = None
        started = time.monotonic()
        print(attempt, flush=True)
        try:
            worker_build = verify_worker(args.worker_kind, args.engine, worker)
            verifier_build = verify_verifier("verifier", verifier)
            for name, value in (("worker-build", worker_build), ("verifier-build", verifier_build)):
                (attempt / f"{name}.json").write_text(json.dumps(value, indent=2) + "\n")
            report.update(worker_binary_sha256=sha(worker), verifier_binary_sha256=sha(verifier), program_sha256=sha(program))
            env = os.environ.copy()
            env.update(ZKMINER_MACOS_WORKER=str(worker), ZKMINER_MACOS_WORKER_KIND=args.worker_kind,
                       ZKMINER_MACOS_PROGRAM=str(program), ZKMINER_MACOS_VERIFIER=str(verifier),
                       ZKMINER_MACOS_EVIDENCE_DIR=str(attempt / "acceptance"), ZKMINER_RISC0_EVIDENCE_DIR=str(sidecars),
                       ZKMINER_RISC0_WORK_ROOT=str(work), ZKMINER_RISC0_DOCKER_CONTEXT=args.context,
                       ZKMINER_RISC0_MODE=args.mode, ZKMINER_MACOS_EXPECT_MODE=expected,
                       ZKMINER_MACOS_REPEAT=str(args.repeat), ZKMINER_MACOS_LIFECYCLE="0" if args.proof_only else "1",
                       ZKMINER_MACOS_FAILURE_TIMEOUT_SECS=str(args.failure_timeout),
                       ZKMINER_RISC0_METAL_STAGING_BYTES=str(args.staging_bytes), RAYON_NUM_THREADS="6")
            binary, test_build = build_acceptance(attempt, env)
            report["command"] = [str(binary), "--ignored", "--nocapture", "--test-threads=1"]
            test_started = time.monotonic()
            with (attempt / "test.log").open("w") as log:
                proc = subprocess.Popen(report["command"], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                while proc.poll() is None:
                    if time.monotonic() - test_started > args.timeout:
                        raise TimeoutError("native worker acceptance deadline exceeded")
                    for record in read_sidecars(sidecars):
                        pid = record.get("worker_pid")
                        if type(pid) is int and owned_process(pid, proc.pid):
                            rss = subprocess.run(["ps", "-o", "rss=", "-p", str(pid)], capture_output=True, text=True, timeout=5)
                            if rss.returncode == 0 and rss.stdout.strip():
                                value = int(rss.stdout.strip()) * 1024
                                key = str(pid)
                                report["worker_peak_rss_bytes"][key] = max(report["worker_peak_rss_bytes"].get(key, 0), value)
                                if value > 32 * 1024**3:
                                    raise RuntimeError("owned native worker exceeded the 32 GiB RSS ceiling")
                    try:
                        proc.wait(timeout=1)
                    except subprocess.TimeoutExpired:
                        pass
            report["test_exit_code"] = proc.returncode
            if proc.returncode:
                raise RuntimeError(f"real native worker acceptance failed ({proc.returncode}); see test.log")
            if verify_worker(args.worker_kind, args.engine, worker) != worker_build or verify_verifier("verifier", verifier) != verifier_build:
                raise RuntimeError("worker/verifier sealed build changed during acceptance")
            if test_sources() != test_build["source_inputs"] or sha(binary) != test_build["binary_sha256"]:
                raise RuntimeError("acceptance test source or executable changed during the run")
            report["results"] = validate_results(attempt, args, expected)
            report["status"] = "complete_independently_verified"
        except (Exception, KeyboardInterrupt) as error:
            report["error"] = str(error)
        finally:
            errors = []
            if proc is not None:
                cleanup_records = read_sidecars(sidecars, tolerate_errors=True)
                if proc.poll() is None:
                    # Signal only direct children whose process group matches their PID.
                    for record in cleanup_records:
                        pid = record.get("worker_pid")
                        try:
                            if type(pid) is int and owned_process(pid, proc.pid):
                                os.killpg(pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                        except Exception as error:
                            errors.append(f"worker cleanup: {error}")
                    try:
                        os.killpg(proc.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    try:
                        proc.wait(timeout=15)
                    except Exception as error:
                        errors.append(f"test process cleanup: {error}")
                prefix = f"zkminer-macos-{proc.pid}-"
                names = {path.name for path in work.iterdir() if path.is_dir() and path.name.startswith(prefix)}
                names |= {record.get("helper_container") for record in cleanup_records
                          if isinstance(record.get("helper_container"), str) and record["helper_container"].startswith(prefix)}
                for name in sorted(names):
                    try:
                        cleanup = subprocess.run(docker_cmd + ["rm", "-f", name], capture_output=True, text=True, timeout=15)
                        if cleanup.returncode and "No such container" not in cleanup.stderr:
                            errors.append(cleanup.stderr.strip())
                    except Exception as error:
                        errors.append(str(error))
            if errors:
                report.update(cleanup_errors=errors, status="failed")
            report["elapsed_seconds"] = time.monotonic() - started
            (attempt / "result.json").write_text(json.dumps(report, indent=2) + "\n")
            with (runs / "results.jsonl").open("a") as ledger:
                ledger.write(json.dumps(report) + "\n")
            print(json.dumps({key:report.get(key) for key in ("id", "status", "elapsed_seconds", "error")}))
        return 0 if report["status"] == "complete_independently_verified" else 1


if __name__ == "__main__":
    raise SystemExit(main())
