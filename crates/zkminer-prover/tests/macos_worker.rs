//! Explicit local acceptance: real WorkerPool, real proofs, unchanged external verifier.
#![cfg(target_os = "macos")]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use zkminer_prover::{dispatcher::WorkerPool, worker::WorkerHandle};
use zkminer_prover_protocol::{WorkerCommand, WorkerResponse, PROTOCOL_VERSION};

const KEY: &str = "risc0:generic";

fn required(name: &str) -> PathBuf {
    PathBuf::from(
        std::env::var_os(name)
            .unwrap_or_else(|| panic!("set {name}; acceptance cannot skip missing prerequisites")),
    )
}
fn mode() -> String {
    std::env::var("ZKMINER_RISC0_MODE").unwrap_or_else(|_| "cpu".into())
}
fn observed_mode() -> String {
    std::env::var("ZKMINER_MACOS_EXPECT_MODE").unwrap_or_else(|_| "cpu".into())
}
fn worker_kind() -> String {
    std::env::var("ZKMINER_MACOS_WORKER_KIND").unwrap_or_else(|_| "cpu".into())
}
fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn bounded_output(command: &mut Command) -> String {
    let (status, stdout) = zkminer_prover_protocol::proc::output_with_timeout_capturing_stdout(
        command,
        Duration::from_secs(10),
        1024 * 1024,
    );
    assert!(
        matches!(
            status,
            zkminer_prover_protocol::proc::Outcome::Ran { success: true, .. }
        ),
        "observation command failed: {}",
        status.describe()
    );
    stdout
}
fn helpers() -> Vec<String> {
    bounded_output(Command::new("docker").args([
        "--context",
        &std::env::var("ZKMINER_RISC0_DOCKER_CONTEXT").unwrap(),
        "ps",
        "-a",
        "--filter",
        "name=zkminer-macos-",
        "--format",
        "{{.Names}}",
    ]))
    .lines()
    .map(str::to_owned)
    .collect()
}
fn helper_running(name: &str) -> bool {
    assert!(
        name.starts_with(&format!("zkminer-macos-{}-", std::process::id())),
        "not this test's helper"
    );
    let names = bounded_output(Command::new("docker").args([
        "--context",
        &std::env::var("ZKMINER_RISC0_DOCKER_CONTEXT").unwrap(),
        "ps",
        "--filter",
        "status=running",
        "--format",
        "{{.Names}}",
    ]));
    names.lines().any(|value| value == name)
}
fn assert_group_gone(pid: u32) {
    let processes = bounded_output(Command::new("ps").args(["-axo", "pid=,pgid=,stat="]));
    for line in processes.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 3, "invalid process observation");
        assert!(
            !(fields[1].parse::<u32>().unwrap() == pid && !fields[2].starts_with('Z')),
            "owned worker group still has live member: {line}"
        );
    }
}
fn sidecar_paths() -> BTreeSet<PathBuf> {
    fs::read_dir(required("ZKMINER_RISC0_EVIDENCE_DIR"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|s| s == "json"))
        .collect()
}
fn new_sidecar(before: &BTreeSet<PathBuf>) -> Option<(PathBuf, Value)> {
    let paths = sidecar_paths();
    let new: Vec<_> = paths.difference(before).collect();
    assert!(
        new.len() <= 1,
        "more than one request active in the single worker slot"
    );
    new.first().map(|path| {
        (
            (*path).clone(),
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap(),
        )
    })
}
fn require_sidecar(before: &BTreeSet<PathBuf>) -> (PathBuf, Value) {
    new_sidecar(before).expect("worker returned without persistent request evidence")
}
fn positive(value: &Value, field: &str) -> u64 {
    value[field]
        .as_u64()
        .filter(|n| *n > 0)
        .unwrap_or_else(|| panic!("missing positive {field}: {value}"))
}
fn validate_stage(record: &Value) {
    assert_eq!(record["schema_version"], 1);
    assert_eq!(record["operation"], "prove");
    assert_eq!(record["requested_mode"], mode());
    assert_eq!(record["observed_mode"], observed_mode());
    assert_eq!(record["input_sha256"], hash(&20u32.to_le_bytes()));
    if record["phase"] == "complete" {
        assert_eq!(record["journal_hex"], "6d1a000000000000");
    }
    assert_eq!(record["po2"], 18);
    if mode() == "auto" && observed_mode() == "cpu" {
        assert!(
            !record["fallback_reason"].as_str().unwrap_or("").is_empty(),
            "auto CPU fallback needs a reason"
        );
    }
    let count = positive(&record["stats"], "segments");
    let stage = &record["observed_stages"]["metal_stage"];
    if observed_mode() == "cpu" {
        assert!(stage.is_null(), "CPU mode unexpectedly executed Metal");
        return;
    }
    assert_eq!(stage["name"], "segment_poseidon2_merkle_fold");
    let segments = stage["segments"]
        .as_array()
        .expect("missing per-segment GPU evidence");
    assert_eq!(segments.len() as u64, count);
    let mut dispatches = 0;
    for (index, segment) in segments.iter().enumerate() {
        assert_eq!(segment["segment_index"], index as u64);
        let done = positive(segment, "completed_dispatches");
        assert!(done >= positive(segment, "fold_calls"));
        dispatches += done;
        assert_eq!(
            positive(segment, "input_digests"),
            2 * positive(segment, "output_digests")
        );
        let budget = positive(segment, "staging_limit_bytes");
        assert!((4096..=16 * 1024 * 1024).contains(&budget));
        assert_eq!(
            segment["staging_limit_bytes"],
            record["staging_limit_bytes"]
        );
        assert!(positive(segment, "peak_staging_bytes") <= budget);
    }
    assert_eq!(stage["completed_dispatches"], dispatches);
}
fn ledger(evidence: &Path, value: &Value) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(evidence.join("requests.jsonl"))
        .unwrap();
    writeln!(file, "{value}").unwrap();
}
fn verify(
    verifier: &Path,
    program: &Path,
    input: &Path,
    dir: &Path,
    proof: zkminer_prover::engine::ProofOutput,
    record: &Value,
) {
    fs::create_dir(dir).unwrap();
    assert_eq!(proof.journal, 6765u64.to_le_bytes());
    assert_eq!(proof.seal.len(), 256);
    assert!(proof.cycles > 0);
    assert_eq!(record["phase"], "complete");
    assert_eq!(record["proof_kind"], "groth16");
    assert_eq!(record["seal_sha256"], hash(&proof.seal));
    validate_stage(record);
    fs::write(
        dir.join("worker.json"),
        serde_json::to_vec_pretty(record).unwrap(),
    )
    .unwrap();
    fs::write(dir.join("journal.bin"), &proof.journal).unwrap();
    fs::write(dir.join("seal.bin"), &proof.seal).unwrap();
    let output = Command::new(verifier)
        .arg("verify-seal")
        .arg(program)
        .arg(input)
        .arg("6d1a000000000000")
        .arg(dir)
        .arg("18")
        .output()
        .unwrap();
    fs::write(dir.join("verifier.stdout"), &output.stdout).unwrap();
    fs::write(dir.join("verifier.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "independent verification failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let verification: Value =
        serde_json::from_slice(&fs::read(dir.join("verification.json")).unwrap()).unwrap();
    assert_eq!(verification["image_id"], record["image_id"]);
    assert_eq!(verification["verification"], "passed");
    assert_eq!(verification["journal_hex"], "6d1a000000000000");
    assert_eq!(
        verification["negative_tests"],
        json!(["wrong_image", "modified_journal", "corrupt_seal"])
    );
    eprintln!(
        "independently verified {} ({} cycles, {:.3}s, mode={})",
        dir.display(),
        proof.cycles,
        proof.duration.as_secs_f64(),
        observed_mode()
    );
}

#[test]
#[ignore = "requires the real native worker and dedicated local helper VM"]
fn native_worker_mode_preflight() {
    let binary = required("ZKMINER_MACOS_WORKER");
    for (requested, expected) in [
        ("cpu", "cpu"),
        (
            "auto",
            if worker_kind() == "metal" {
                "metal"
            } else {
                "cpu"
            },
        ),
    ] {
        let overrides = HashMap::from([("ZKMINER_RISC0_MODE".into(), requested.into())]);
        let mut handle = WorkerHandle::spawn("risc0", &binary, &overrides).unwrap();
        handle
            .send(&WorkerCommand::Warmup { request_id: 17 })
            .unwrap();
        match handle.recv().unwrap() {
            WorkerResponse::WarmupDone {
                request_id,
                summary,
                groth16_helper,
            } => {
                assert_eq!(request_id, 17);
                assert!(summary.to_lowercase().contains(expected), "{summary}");
                assert!(groth16_helper);
            }
            other => panic!("unexpected warmup response: {other:?}"),
        }
    }
    for (key, value, expected) in [
        (
            "ZKMINER_RISC0_MODE",
            "invalid-mode",
            "unknown ZKMINER_RISC0_MODE",
        ),
        ("RISC0_DEV_MODE", "1", "unset RISC0_DEV_MODE"),
        ("RISC0_SKIP_VERIFY", "1", "unset RISC0_SKIP_VERIFY"),
        (
            "ZKMINER_RISC0_METAL_STAGING_BYTES",
            "bad",
            "invalid ZKMINER_RISC0_METAL_STAGING_BYTES",
        ),
        (
            "ZKMINER_RISC0_METAL_STAGING_BYTES",
            "4095",
            "Metal staging budget",
        ),
        (
            "ZKMINER_RISC0_METAL_STAGING_BYTES",
            "16777217",
            "Metal staging budget",
        ),
    ] {
        let overrides = HashMap::from([(key.into(), value.into())]);
        let error = match WorkerHandle::spawn("risc0", &binary, &overrides) {
            Ok(_) => panic!("invalid startup admitted"),
            Err(error) => format!("{error:#}"),
        };
        assert!(error.contains(expected), "{error}");
    }
    if worker_kind() == "cpu" {
        let overrides = HashMap::from([("ZKMINER_RISC0_MODE".into(), "metal".into())]);
        assert!(
            WorkerHandle::spawn("risc0", &binary, &overrides).is_err(),
            "CPU-only worker admitted forced Metal"
        );
    }
}

#[test]
#[ignore = "explicit local Groth16 acceptance; requires built worker, guest, unchanged verifier, and dedicated helper VM"]
fn native_worker_proofs_failure_restart_and_shutdown() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(std::io::stderr)
        .try_init();
    let binary = required("ZKMINER_MACOS_WORKER");
    let program = required("ZKMINER_MACOS_PROGRAM");
    let verifier = required("ZKMINER_MACOS_VERIFIER");
    let evidence = required("ZKMINER_MACOS_EVIDENCE_DIR");
    fs::create_dir(&evidence).expect("use a fresh evidence directory");
    assert!(helpers().is_empty(), "use an idle dedicated VM");
    let input = evidence.join("input.bin");
    fs::write(&input, 20u32.to_le_bytes()).unwrap();
    let elf = fs::read(&program).unwrap();
    let repeats: usize = std::env::var("ZKMINER_MACOS_REPEAT")
        .unwrap_or_else(|_| "3".into())
        .parse()
        .unwrap();
    assert!((1..=100).contains(&repeats));
    let lifecycle = std::env::var("ZKMINER_MACOS_LIFECYCLE").unwrap_or_else(|_| "1".into()) == "1";
    assert!(
        !lifecycle || repeats >= 3,
        "lifecycle acceptance needs at least three complete repeat proofs"
    );
    let overrides = HashMap::from([(
        "ZKMINER_RISC0_DOCKER_CONTEXT".into(),
        "hemi-deliberately-missing-context".into(),
    )]);
    assert!(
        WorkerHandle::spawn("risc0", &binary, &overrides).is_err(),
        "missing helper context admitted"
    );
    let mut handle = WorkerHandle::spawn("risc0", &binary, &HashMap::new()).unwrap();
    handle.send(&WorkerCommand::Capabilities).unwrap();
    match handle.recv().unwrap() {
        WorkerResponse::CapabilitiesReport {
            backend,
            gpu_available,
            gpu_name,
            supported_benchmarks,
            ..
        } => {
            assert_eq!(backend, "risc0");
            assert_eq!(gpu_available, observed_mode() == "metal");
            assert_eq!(gpu_name.is_some(), observed_mode() == "metal");
            assert_eq!(supported_benchmarks, ["fibonacci", "sha256-chain"]);
        }
        other => panic!("unexpected capabilities: {other:?}"),
    }
    handle
        .send(&WorkerCommand::Hello {
            protocol_version: PROTOCOL_VERSION + 1,
        })
        .unwrap();
    assert!(matches!(
        handle.recv().unwrap(),
        WorkerResponse::Error { .. }
    ));
    drop(handle);
    let mut pool = WorkerPool::new(
        HashMap::from([("risc0".into(), binary)]),
        vec![],
        Some(Duration::from_secs(600)),
    );
    assert_eq!(pool.discover_and_spawn(), [KEY]);
    assert!(pool
        .warm_up("risc0", Duration::from_secs(15))
        .unwrap()
        .contains("helper"));
    assert!(
        pool.execute_cycles(
            "risc0",
            &elf,
            &20u32.to_le_bytes(),
            Some(Duration::from_secs(30))
        )
        .unwrap()
            > 0
    );
    assert!(pool
        .execute_cycles(
            "risc0",
            b"invalid guest",
            &[],
            Some(Duration::from_secs(10))
        )
        .is_err());
    for index in 0..repeats {
        let before = sidecar_paths();
        ledger(
            &evidence,
            &json!({"case":format!("proof-{index}"),"status":"started"}),
        );
        let result = pool.prove(
            "risc0",
            &elf,
            &20u32.to_le_bytes(),
            Some(18),
            Some(Duration::from_secs(600)),
            None,
        );
        if let Err(error) = &result {
            ledger(
                &evidence,
                &json!({"case":format!("proof-{index}"),"status":"failed","error":format!("{error:#}")}),
            );
        }
        let proof = result.unwrap();
        let (_, record) = require_sidecar(&before);
        verify(
            &verifier,
            &program,
            &input,
            &evidence.join(format!("proof-{index}")),
            proof,
            &record,
        );
        ledger(
            &evidence,
            &json!({"case":format!("proof-{index}"),"status":"complete_independently_verified","worker":record}),
        );
        assert!(helpers().is_empty(), "helper survived a successful proof");
    }
    if lifecycle {
        // Trigger real parent cancellation only after this request's helper is running.
        let before = sidecar_paths();
        ledger(
            &evidence,
            &json!({"case":"helper-cancel","status":"started"}),
        );
        let mut observed = None;
        let failure = std::thread::scope(|scope| {
            let task = scope.spawn(|| {
                pool.prove(
                    "risc0",
                    &elf,
                    &20u32.to_le_bytes(),
                    Some(18),
                    Some(Duration::from_secs(600)),
                    None,
                )
            });
            let deadline = Instant::now() + Duration::from_secs(240);
            while !task.is_finished() && Instant::now() < deadline {
                if let Some((_, record)) = new_sidecar(&before) {
                    if record["phase"] == "wrapping" {
                        let name = record["helper_container"]
                            .as_str()
                            .expect("missing helper lease identity");
                        if helper_running(name) {
                            validate_stage(&record);
                            pool.cancel(KEY, record["request_id"].as_u64().unwrap())
                                .unwrap();
                            observed = Some(record);
                            break;
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            if observed.is_none() && !task.is_finished() {
                pool.cancel(KEY, 0).unwrap();
            }
            task.join().unwrap()
        });
        ledger(
            &evidence,
            &json!({"case":"helper-cancel","status":if failure.is_err() && observed.is_some() {"expected_failure"} else {"failed"},"error":failure.as_ref().err().map(|e|format!("{e:#}")),"observed_running_helper":observed}),
        );
        assert!(failure.is_err(), "cancelled request unexpectedly completed");
        let observed =
            observed.expect("cancellation did not observe this request's running helper");
        assert!(helpers().is_empty(), "owned helper survived cancellation");
        assert_group_gone(observed["worker_pid"].as_u64().unwrap() as u32);

        // Keep the production watchdog test separate from explicit cancellation.
        let before = sidecar_paths();
        let seen = Arc::new(Mutex::new(None));
        let done = Arc::new(AtomicBool::new(false));
        let (capture, stop) = (seen.clone(), done.clone());
        let monitor = std::thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                if let Some((_, record)) = new_sidecar(&before) {
                    if record["phase"] == "wrapping"
                        && helper_running(record["helper_container"].as_str().unwrap())
                    {
                        *capture.lock().unwrap() = Some(record);
                    }
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
        let seconds: u64 = std::env::var("ZKMINER_MACOS_FAILURE_TIMEOUT_SECS")
            .unwrap_or_else(|_| "70".into())
            .parse()
            .unwrap();
        assert!((1..=300).contains(&seconds));
        ledger(
            &evidence,
            &json!({"case":"helper-timeout","status":"started","timeout_seconds":seconds}),
        );
        let timeout_started = Instant::now();
        let failure = pool.prove(
            "risc0",
            &elf,
            &20u32.to_le_bytes(),
            Some(18),
            Some(Duration::from_secs(seconds)),
            None,
        );
        let timeout_elapsed = timeout_started.elapsed().as_secs_f64();
        done.store(true, Ordering::Release);
        monitor.join().unwrap();
        let observed = seen.lock().unwrap().clone();
        ledger(
            &evidence,
            &json!({"case":"helper-timeout","status":if failure.is_err() && observed.is_some() && timeout_elapsed >= seconds as f64 {"expected_failure"} else {"failed"},"timeout_seconds":seconds,"elapsed_seconds":timeout_elapsed,"error":failure.as_ref().err().map(|e|format!("{e:#}")),"observed_running_helper":observed}),
        );
        assert!(failure.is_err(), "deadline should stop a real proof");
        assert!(
            timeout_elapsed >= seconds as f64,
            "request failed before the configured watchdog deadline"
        );
        let observed = observed.expect("watchdog did not reach this request's running helper");
        validate_stage(&observed);
        assert!(helpers().is_empty(), "owned helper survived watchdog");
        assert_group_gone(observed["worker_pid"].as_u64().unwrap() as u32);

        let before = sidecar_paths();
        ledger(
            &evidence,
            &json!({"case":"proof-after-restart","status":"started"}),
        );
        let proof = pool
            .prove(
                "risc0",
                &elf,
                &20u32.to_le_bytes(),
                Some(18),
                Some(Duration::from_secs(600)),
                None,
            )
            .unwrap();
        let (_, record) = require_sidecar(&before);
        verify(
            &verifier,
            &program,
            &input,
            &evidence.join("proof-after-restart"),
            proof,
            &record,
        );
        ledger(
            &evidence,
            &json!({"case":"proof-after-restart","status":"complete_independently_verified","worker":record}),
        );
    }
    let pid = pool.worker_pid(KEY).unwrap();
    pool.shutdown_all();
    drop(pool);
    assert!(helpers().is_empty(), "helper survived shutdown");
    if pid != 0 {
        assert_group_gone(pid);
    }
    fs::write(evidence.join("complete.json"), serde_json::to_vec_pretty(&json!({
        "status":"complete_independently_verified", "requested_mode":mode(),"observed_mode":observed_mode(),
        "verified_proofs":repeats + usize::from(lifecycle), "helper_cancel":lifecycle,"helper_timeout":lifecycle,
        "restart":lifecycle,"shutdown":true,
    })).unwrap()).unwrap();
}
