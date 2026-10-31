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
struct Workload {
    name: String,
    input: Vec<u8>,
    expected: Vec<u8>,
    expected_hex: String,
    elf_sha256: String,
    po2: u8,
    min_segments: u64,
}

impl Workload {
    fn from_environment(elf: &[u8]) -> Self {
        const KEYS: [&str; 5] = [
            "ZKMINER_MACOS_WORKLOAD",
            "ZKMINER_MACOS_INPUT",
            "ZKMINER_MACOS_EXPECTED_JOURNAL_HEX",
            "ZKMINER_MACOS_PO2",
            "ZKMINER_MACOS_MIN_SEGMENTS",
        ];
        let supplied = KEYS
            .iter()
            .filter(|key| std::env::var_os(key).is_some())
            .count();
        assert!(
            supplied == 0 || supplied == KEYS.len(),
            "supply all five workload settings together"
        );
        let (name, input, expected, po2, min_segments) = if supplied == 0 {
            (
                "fibonacci".to_owned(),
                20u32.to_le_bytes().to_vec(),
                6765u64.to_le_bytes().to_vec(),
                18,
                1,
            )
        } else {
            let name = std::env::var(KEYS[0]).unwrap();
            assert!(
                ["fibonacci", "sha256-chain", "segment-boundary"].contains(&name.as_str()),
                "unsupported acceptance workload"
            );
            let input_path = required(KEYS[1]);
            assert!(
                (1..=65536).contains(&fs::metadata(&input_path).unwrap().len()),
                "acceptance input must be 1..65536 bytes"
            );
            (
                name,
                fs::read(input_path).unwrap(),
                hex::decode(std::env::var(KEYS[2]).unwrap()).expect("invalid expected journal hex"),
                std::env::var(KEYS[3])
                    .unwrap()
                    .parse()
                    .expect("invalid acceptance po2"),
                std::env::var(KEYS[4])
                    .unwrap()
                    .parse()
                    .expect("invalid minimum segment count"),
            )
        };
        assert!(
            (14..=18).contains(&po2),
            "acceptance po2 must be in 14..=18"
        );
        assert!(
            (1..=256).contains(&min_segments),
            "minimum segment count must be in 1..=256"
        );
        assert!(
            (1..=4096).contains(&expected.len()),
            "expected journal must be 1..4096 bytes"
        );
        Self {
            name,
            input,
            expected_hex: hex::encode(&expected),
            expected,
            elf_sha256: hash(elf),
            po2,
            min_segments,
        }
    }
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
fn validate_stage(record: &Value, workload: &Workload) {
    assert_eq!(record["schema_version"], 1);
    assert_eq!(record["operation"], "prove");
    assert_eq!(record["requested_mode"], mode());
    assert_eq!(record["observed_mode"], observed_mode());
    assert_eq!(record["elf_sha256"], workload.elf_sha256);
    assert_eq!(record["input_sha256"], hash(&workload.input));
    if record["phase"] == "complete" {
        assert_eq!(record["journal_hex"], workload.expected_hex);
        assert_eq!(record["journal_sha256"], hash(&workload.expected));
    }
    assert_eq!(record["po2"], workload.po2);
    if mode() == "auto" && observed_mode() == "cpu" {
        assert!(
            !record["fallback_reason"].as_str().unwrap_or("").is_empty(),
            "auto CPU fallback needs a reason"
        );
    }
    let count = positive(&record["stats"], "segments");
    assert!(
        count >= workload.min_segments,
        "workload did not cover the required segment count"
    );
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
        let requested: u64 = std::env::var("ZKMINER_RISC0_METAL_STAGING_BYTES")
            .unwrap_or_else(|_| "524288".into())
            .parse()
            .unwrap();
        assert_eq!(
            budget, requested,
            "worker did not use the requested staging budget"
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
// Observation only: this never allocates a synthetic workload or changes the worker.
fn rss_observation(pid: u32, expected_start: Option<u64>, started: Instant, phase: &str) -> Value {
    let host_available = zkminer_prover::memory::mem_available_bytes();
    let host_total = zkminer_prover::memory::mem_total_bytes();
    let mut sample = json!({"worker_pid":pid,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"phase":phase,
        "host_available_bytes":host_available,"host_total_bytes":host_total});
    if host_available.is_none() || host_total.is_none() {
        sample["host_memory_unavailable_reason"] =
            json!("production host memory observation unavailable; unknown values are null");
    }
    let identity = zkminer_prover::memory::pid_starttime(pid);
    if pid == 0 || identity.is_none() {
        sample["unavailable_reason"] = json!("no live worker start identity");
        return sample;
    }
    let identity = identity.unwrap();
    sample["worker_start_time_us"] = json!(identity);
    if expected_start.is_some_and(|value| value != identity)
        || !zkminer_prover::memory::pid_is_our_worker(pid, Some(identity))
    {
        sample["unavailable_reason"] =
            json!("worker identity or parent/group ownership changed before RSS observation");
        return sample;
    }
    let (status, stdout) = zkminer_prover_protocol::proc::output_with_timeout_capturing_stdout(
        Command::new("ps").args(["-o", "rss=", "-p", &pid.to_string()]),
        Duration::from_secs(2),
        4096,
    );
    if !matches!(
        status,
        zkminer_prover_protocol::proc::Outcome::Ran { success: true, .. }
    ) {
        sample["unavailable_reason"] =
            json!(format!("ps observation failed: {}", status.describe()));
        return sample;
    }
    if zkminer_prover::memory::pid_starttime(pid) != Some(identity)
        || !zkminer_prover::memory::pid_is_our_worker(pid, Some(identity))
    {
        sample["unavailable_reason"] =
            json!("worker identity or parent/group ownership changed during RSS observation");
        return sample;
    }
    match stdout
        .trim()
        .parse::<u64>()
        .ok()
        .and_then(|value| value.checked_mul(1024))
        .filter(|value| *value > 0)
    {
        Some(bytes) => sample["rss_bytes"] = json!(bytes),
        None => {
            sample["unavailable_reason"] = json!("ps did not return a positive bounded RSS value")
        }
    }
    sample
}

fn measured_prove(
    pool: &WorkerPool,
    elf: &[u8],
    workload: &Workload,
) -> (anyhow::Result<zkminer_prover::engine::ProofOutput>, Value) {
    let started = Instant::now();
    let baseline = rss_observation(
        pool.worker_pid(KEY).unwrap_or(0),
        None,
        started,
        "before_prove_call",
    );
    let done = Arc::new(AtomicBool::new(false));
    let (result, mut observations) = std::thread::scope(|scope| {
        let stop = done.clone();
        let observer = scope.spawn(move || {
            let mut samples = Vec::new();
            while !stop.load(Ordering::Acquire) {
                samples.push(rss_observation(
                    pool.worker_pid(KEY).unwrap_or(0),
                    None,
                    started,
                    "periodic_during_prove_call",
                ));
                // One-second cadence, interruptible so joining does not add a full second.
                for _ in 0..10 {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            samples
        });
        let result = pool.prove(
            "risc0",
            elf,
            &workload.input,
            Some(workload.po2),
            Some(Duration::from_secs(600)),
            None,
        );
        done.store(true, Ordering::Release);
        (result, observer.join().unwrap())
    });
    observations.insert(0, baseline.clone());
    observations.push(rss_observation(
        pool.worker_pid(KEY).unwrap_or(0),
        None,
        started,
        "after_prove_call",
    ));
    (
        result,
        json!({"schema_version":1,
        "method":"ps -o rss=, converted from KiB; PID start identity and parent/process-group ownership checked before and after each observation",
        "scope":"native worker process RSS only; excludes helper VM and is not a Metal driver allocation peak",
        "observation_window":"immediately before WorkerPool::prove through its return; post-verification idle observation is separate",
        "host_memory_method":"production Darwin available-memory estimate (free pages plus conservative reclaim credit); includes all host pressure and is not added to worker RSS or treated as an allocation guarantee",
        "sampling_interval_ms":1000,"request_elapsed_ms":started.elapsed().as_secs_f64()*1000.0,
        "baseline_before_prove_call":baseline,"observations":observations}),
    )
}

fn finish_resources(dir: &Path, record: &Value, resources: &mut Value) {
    let pid = record["worker_pid"].as_u64().unwrap();
    let observations = resources["observations"].as_array().unwrap();
    let identity = observations
        .iter()
        .rev()
        .find(|sample| sample["worker_pid"] == pid && sample["rss_bytes"].as_u64().is_some())
        .and_then(|sample| sample["worker_start_time_us"].as_u64())
        .expect("no identity-checked RSS observation for the worker that produced this proof");
    let samples: Vec<_> = observations
        .iter()
        .filter(|sample| {
            sample["worker_pid"] == pid
                && sample["worker_start_time_us"] == identity
                && sample["rss_bytes"].as_u64().is_some()
        })
        .cloned()
        .collect();
    let periodic = samples
        .iter()
        .filter(|sample| sample["phase"] == "periodic_during_prove_call")
        .count();
    assert!(
        periodic > 0,
        "no periodic RSS observation during the real proving request"
    );
    let peak = samples
        .iter()
        .map(|sample| sample["rss_bytes"].as_u64().unwrap())
        .max()
        .unwrap();
    assert!(
        peak <= 32 * 1024u64.pow(3),
        "sampled native request RSS exceeded the worker bound"
    );
    let identities: BTreeSet<_> = observations
        .iter()
        .filter(|sample| sample["rss_bytes"].as_u64().is_some())
        .filter_map(|sample| {
            Some((
                sample["worker_pid"].as_u64()?,
                sample["worker_start_time_us"].as_u64()?,
            ))
        })
        .collect();
    resources["observed_worker_identities"] = json!(identities.into_iter().map(|(worker_pid, worker_start_time_us)|
        json!({"worker_pid":worker_pid,"worker_start_time_us":worker_start_time_us})).collect::<Vec<_>>());
    resources["worker_pid"] = json!(pid);
    resources["worker_start_time_us"] = json!(identity);
    resources["sample_count"] = json!(samples.len());
    resources["periodic_sample_count"] = json!(periodic);
    resources["peak_rss_bytes"] = json!(peak);
    resources["first_sample"] = samples.first().unwrap().clone();
    resources["last_sample"] = samples.last().unwrap().clone();
    resources["baseline_matches_request_worker"] = json!(
        resources["baseline_before_prove_call"]["worker_pid"] == pid
            && resources["baseline_before_prove_call"]["worker_start_time_us"] == identity
            && resources["baseline_before_prove_call"]["rss_bytes"]
                .as_u64()
                .is_some()
    );
    resources["idle_settle_ms"] = json!(1000);
    let idle_started = Instant::now();
    std::thread::sleep(Duration::from_secs(1));
    let mut idle = rss_observation(
        pid as u32,
        Some(identity),
        idle_started,
        "after_independent_verification_and_settle",
    );
    idle["elapsed_ms_reference"] =
        json!("time since the post-verification settling interval began");
    resources["idle_after_verification_and_settle"] = idle;
    fs::write(
        dir.join("resource.json"),
        serde_json::to_vec_pretty(resources).unwrap(),
    )
    .unwrap();
}

fn admission_pressure(pool: &WorkerPool, elf: &[u8], workload: &Workload, evidence: &Path) {
    const STEP: u64 = 1024 * 1024 * 1024;
    let total =
        zkminer_prover::memory::mem_total_bytes().expect("cannot observe real host total memory");
    let available = zkminer_prover::memory::mem_available_bytes()
        .expect("cannot observe real host available memory");
    let reserve = zkminer_prover::memory::DEFAULT_HOST_RESERVE_BYTES;
    let needed = pool.expected_host_peak_at_po2("risc0", Some(workload.po2));
    assert!(
        available >= STEP + reserve,
        "insufficient live headroom for the logical reservation test prerequisite"
    );
    assert_eq!(
        pool.reserved_host_memory(),
        0,
        "reservation test requires an idle ledger"
    );
    let before = sidecar_paths();
    assert!(helpers().is_empty());
    let pid_before = pool.worker_pid(KEY).unwrap_or(0);
    ledger(
        evidence,
        &json!({"case":"admission-pressure","status":"started", "kind":"in_flight_reservation_admission",
        "physical_memory_pressure":false,"host_total_bytes":total,"host_available_bytes":available}),
    );
    // These are the real production ledger guards. They reserve accounting only;
    // no corresponding host/GPU pages are allocated or touched by this test.
    let mut guards = Vec::new();
    let limit = total / STEP + 1;
    assert!(limit <= 4096, "logical reservation test bound exceeded");
    while pool
        .reserved_host_memory()
        .saturating_add(needed)
        .saturating_add(reserve)
        <= total
    {
        assert!(
            (guards.len() as u64) < limit,
            "logical reservation loop exceeded its host-derived bound"
        );
        guards.push(pool.reserve_host_memory(STEP, KEY)
            .expect("live host headroom changed before the intended committed-reservation condition; prerequisite failure"));
    }
    let held = pool.reserved_host_memory();
    let refusal = pool
        .prove(
            "risc0",
            elf,
            &workload.input,
            Some(workload.po2),
            Some(Duration::from_secs(30)),
            None,
        )
        .expect_err("proof unexpectedly ran while real in-flight reservations exhausted admission");
    let shortage = refusal
        .downcast_ref::<zkminer_prover_protocol::types::HostMemoryShortage>()
        .expect("admission case did not return typed HostMemoryShortage");
    assert!(
        !shortage.attempted,
        "admission case unexpectedly attempted a proof"
    );
    assert_eq!(shortage.slot_key, KEY);
    assert_eq!(shortage.needed, needed);
    assert_eq!(shortage.reserved, held);
    assert_eq!(
        sidecar_paths(),
        before,
        "admission refusal started a worker request"
    );
    let helper_count_after = helpers().len();
    assert_eq!(
        helper_count_after, 0,
        "admission refusal started or leaked a helper"
    );
    let mut report = json!({"kind":"in_flight_reservation_admission","physical_memory_pressure":false,"attempted":false,
        "reserved_before_bytes":0,"reserved_held_bytes":held,"needed_bytes":needed,"host_total_bytes":total,
        "host_available_bytes":available,"host_available_at_refusal_bytes":shortage.available,"host_reserve_bytes":reserve,
        "reservation_count":guards.len(),"step_bytes":STEP,"sidecar_count_before":before.len(),"sidecar_count_after":sidecar_paths().len(),
        "helper_count_after":helper_count_after,"worker_pid_before":pid_before,"worker_pid_after":pool.worker_pid(KEY).unwrap_or(0),
        "error":format!("{refusal:#}")});
    drop(guards);
    assert_eq!(
        pool.reserved_host_memory(),
        0,
        "logical reservations were not released"
    );
    report["reserved_after_bytes"] = json!(pool.reserved_host_memory());
    fs::write(
        evidence.join("pressure.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    ledger(
        evidence,
        &json!({"case":"admission-pressure","status":"expected_admission_refusal","pressure":report}),
    );
    // The normal proof loop immediately following this case must produce and
    // independently verify a genuine full proof after releasing the guards.
}

fn verify(
    verifier: &Path,
    program: &Path,
    input: &Path,
    dir: &Path,
    proof: zkminer_prover::engine::ProofOutput,
    record: &Value,
    workload: &Workload,
) {
    fs::create_dir(dir).unwrap();
    assert_eq!(proof.journal, workload.expected);
    assert_eq!(proof.seal.len(), 256);
    assert!(proof.cycles > 0);
    assert_eq!(record["phase"], "complete");
    assert_eq!(record["proof_kind"], "groth16");
    assert_eq!(record["seal_sha256"], hash(&proof.seal));
    validate_stage(record, workload);
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
        .arg(&workload.expected_hex)
        .arg(dir)
        .arg(workload.po2.to_string())
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
    assert_eq!(verification["journal_hex"], workload.expected_hex);
    assert_eq!(verification["elf_sha256"], workload.elf_sha256);
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
    let elf = fs::read(&program).unwrap();
    let workload = Workload::from_environment(&elf);
    fs::write(&input, &workload.input).unwrap();
    fs::write(evidence.join("workload.json"), serde_json::to_vec_pretty(&json!({
        "name":workload.name,"input_sha256":hash(&workload.input),"elf_sha256":workload.elf_sha256,
        "expected_journal_hex":workload.expected_hex,"po2":workload.po2,"min_segments":workload.min_segments,
    })).unwrap()).unwrap();
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
            &workload.input,
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
    let pressure = match std::env::var("ZKMINER_MACOS_ADMISSION_PRESSURE")
        .as_deref()
        .unwrap_or("0")
    {
        "0" => false,
        "1" => true,
        _ => panic!("admission pressure must be 0 or 1"),
    };
    assert!(
        !pressure || !lifecycle,
        "logical admission pressure requires proof-only acceptance"
    );
    if pressure {
        admission_pressure(&pool, &elf, &workload, &evidence);
    }
    let mut request_identities = BTreeSet::new();
    for index in 0..repeats {
        let before = sidecar_paths();
        ledger(
            &evidence,
            &json!({"case":format!("proof-{index}"),"status":"started"}),
        );
        let (result, mut resources) = measured_prove(&pool, &elf, &workload);
        fs::write(
            evidence.join(format!("proof-{index}-resource-observations.json")),
            serde_json::to_vec_pretty(&resources).unwrap(),
        )
        .unwrap();
        if let Err(error) = &result {
            ledger(
                &evidence,
                &json!({"case":format!("proof-{index}"),"status":"failed","error":format!("{error:#}"),"resources":resources}),
            );
        }
        let proof = result.unwrap();
        let (_, record) = require_sidecar(&before);
        assert!(
            request_identities.insert((
                record["worker_pid"].as_u64().unwrap(),
                record["request_id"].as_u64().unwrap()
            )),
            "duplicate completed request identity"
        );
        verify(
            &verifier,
            &program,
            &input,
            &evidence.join(format!("proof-{index}")),
            proof,
            &record,
            &workload,
        );
        finish_resources(
            &evidence.join(format!("proof-{index}")),
            &record,
            &mut resources,
        );
        ledger(
            &evidence,
            &json!({"case":format!("proof-{index}"),"status":"complete_independently_verified","worker":record,"resources":resources}),
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
                    &workload.input,
                    Some(workload.po2),
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
                            validate_stage(&record, &workload);
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
            &workload.input,
            Some(workload.po2),
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
        validate_stage(&observed, &workload);
        assert!(helpers().is_empty(), "owned helper survived watchdog");
        assert_group_gone(observed["worker_pid"].as_u64().unwrap() as u32);

        let before = sidecar_paths();
        ledger(
            &evidence,
            &json!({"case":"proof-after-restart","status":"started"}),
        );
        let (proof, mut resources) = measured_prove(&pool, &elf, &workload);
        fs::write(
            evidence.join("proof-after-restart-resource-observations.json"),
            serde_json::to_vec_pretty(&resources).unwrap(),
        )
        .unwrap();
        if let Err(error) = &proof {
            ledger(
                &evidence,
                &json!({"case":"proof-after-restart","status":"failed","error":format!("{error:#}"),"resources":resources}),
            );
        }
        let proof = proof.unwrap();
        let (_, record) = require_sidecar(&before);
        assert!(
            request_identities.insert((
                record["worker_pid"].as_u64().unwrap(),
                record["request_id"].as_u64().unwrap()
            )),
            "duplicate completed request identity"
        );
        verify(
            &verifier,
            &program,
            &input,
            &evidence.join("proof-after-restart"),
            proof,
            &record,
            &workload,
        );
        finish_resources(
            &evidence.join("proof-after-restart"),
            &record,
            &mut resources,
        );
        ledger(
            &evidence,
            &json!({"case":"proof-after-restart","status":"complete_independently_verified","worker":record,"resources":resources}),
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
        "restart":lifecycle,"shutdown":true,"workload":workload.name,"po2":workload.po2,
        "min_segments":workload.min_segments,"expected_journal_hex":workload.expected_hex,
        "admission_pressure":pressure,"unique_completed_requests":request_identities.len(),
    })).unwrap()).unwrap();
}
