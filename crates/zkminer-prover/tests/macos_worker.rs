//! Explicit local acceptance: real worker, real proofs, unchanged external verifier.
#![cfg(target_os = "macos")]
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use zkminer_prover::{dispatcher::WorkerPool, worker::WorkerHandle};
use zkminer_prover_protocol::{WorkerCommand, WorkerResponse, PROTOCOL_VERSION};

fn required(name: &str) -> PathBuf {
    PathBuf::from(
        std::env::var_os(name)
            .unwrap_or_else(|| panic!("set {name}; acceptance cannot skip missing prerequisites")),
    )
}

fn helpers() -> Vec<String> {
    let output = Command::new("docker")
        .args([
            "--context",
            &std::env::var("ZKMINER_RISC0_DOCKER_CONTEXT").unwrap(),
            "ps",
            "-a",
            "--filter",
            "name=zkminer-macos-",
            "--format",
            "{{.Names}}",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn verify(
    verifier: &Path,
    program: &Path,
    input: &Path,
    dir: &Path,
    proof: zkminer_prover::engine::ProofOutput,
) {
    std::fs::create_dir(dir).unwrap();
    assert_eq!(proof.journal, 6765u64.to_le_bytes());
    assert_eq!(proof.seal.len(), 256);
    assert!(proof.cycles > 0);
    std::fs::write(dir.join("journal.bin"), proof.journal).unwrap();
    std::fs::write(dir.join("seal.bin"), proof.seal).unwrap();
    let output = Command::new(verifier)
        .arg("verify-seal")
        .arg(program)
        .arg(input)
        .arg("6d1a000000000000")
        .arg(dir)
        .arg("18")
        .output()
        .unwrap();
    std::fs::write(dir.join("verifier.stdout"), &output.stdout).unwrap();
    std::fs::write(dir.join("verifier.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "independent verification failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!(
        "independently verified {} ({} cycles, {:.3}s)",
        dir.display(),
        proof.cycles,
        proof.duration.as_secs_f64()
    );
}

#[test]
#[ignore = "requires the real native worker and dedicated local helper VM"]
fn native_worker_mode_preflight() {
    let binary = required("ZKMINER_MACOS_WORKER");
    let overrides = HashMap::from([("ZKMINER_RISC0_MODE".into(), "auto".into())]);
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
            assert!(summary.contains("CPU"));
            assert!(groth16_helper);
        }
        other => panic!("unexpected warmup response: {other:?}"),
    }
    drop(handle);
    for (key, value, expected) in [
        (
            "ZKMINER_RISC0_MODE",
            "invalid-mode",
            "unknown ZKMINER_RISC0_MODE",
        ),
        ("RISC0_DEV_MODE", "1", "unset RISC0_DEV_MODE"),
    ] {
        let overrides = HashMap::from([(key.into(), value.into())]);
        let error = match WorkerHandle::spawn("risc0", &binary, &overrides) {
            Ok(_) => panic!("invalid mode admitted"),
            Err(error) => format!("{error:#}"),
        };
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
#[ignore = "explicit local Groth16 acceptance; requires the built worker, guest, verifier, and dedicated helper VM"]
fn native_worker_proofs_failure_restart_and_shutdown() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(std::io::stderr)
        .try_init();
    let binary = required("ZKMINER_MACOS_WORKER");
    let program = required("ZKMINER_MACOS_PROGRAM");
    let verifier = required("ZKMINER_MACOS_VERIFIER");
    let evidence = required("ZKMINER_MACOS_EVIDENCE_DIR");
    std::fs::create_dir(&evidence).expect("use a fresh evidence directory");
    assert!(helpers().is_empty(), "use an idle dedicated VM");
    let input = evidence.join("input.bin");
    std::fs::write(&input, 20u32.to_le_bytes()).unwrap();
    let elf = std::fs::read(&program).unwrap();
    let repeats: usize = std::env::var("ZKMINER_MACOS_REPEAT")
        .unwrap_or_else(|_| "3".into())
        .parse()
        .unwrap();
    assert!((3..=100).contains(&repeats));

    for (key, value, expected) in [
        ("ZKMINER_RISC0_MODE", "metal", "Metal"),
        (
            "ZKMINER_RISC0_DOCKER_CONTEXT",
            "hemi-deliberately-missing-context",
            "preflight",
        ),
    ] {
        let env = HashMap::from([(key.to_owned(), value.to_owned())]);
        let error = match WorkerHandle::spawn("risc0", &binary, &env) {
            Ok(_) => panic!("unsupported path admitted"),
            Err(error) => format!("{error:#}"),
        };
        assert!(error.contains(expected), "{error}");
    }

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
            assert!(!gpu_available);
            assert!(gpu_name.is_none());
            assert_eq!(supported_benchmarks, ["fibonacci", "sha256-chain"]);
        }
        other => panic!("unexpected capability response: {other:?}"),
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

    // Exercise the real miner discovery, spawn, admission, timeout and restart paths.
    let mut pool = WorkerPool::new(
        HashMap::from([("risc0".into(), binary)]),
        vec![],
        Some(Duration::from_secs(600)),
    );
    assert_eq!(pool.discover_and_spawn(), ["risc0:generic"]);
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
        verify(
            &verifier,
            &program,
            &input,
            &evidence.join(format!("proof-{index}")),
            proof,
        );
    }

    // Kill during wrapping, not merely before the helper was ever started.
    let observed = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    let (seen, stop) = (observed.clone(), done.clone());
    let monitor = std::thread::spawn(move || {
        while !stop.load(Ordering::Acquire) {
            if !helpers().is_empty() {
                seen.store(true, Ordering::Release);
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });
    let failure = pool.prove(
        "risc0",
        &elf,
        &20u32.to_le_bytes(),
        Some(18),
        Some(Duration::from_secs(70)),
        None,
    );
    done.store(true, Ordering::Release);
    monitor.join().unwrap();
    assert!(failure.is_err(), "deadline should stop a real proof");
    assert!(
        observed.load(Ordering::Acquire),
        "test did not reach the wrapping helper"
    );
    assert!(helpers().is_empty(), "owned helper survived the watchdog");
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
    verify(
        &verifier,
        &program,
        &input,
        &evidence.join("proof-after-restart"),
        proof,
    );
    pool.shutdown_all();
    drop(pool);
    assert!(helpers().is_empty(), "helper survived shutdown");
    std::fs::write(evidence.join("complete.txt"), format!("{} real verified proofs; invalid guest, version mismatch, missing helper, forced Metal rejection, wrap watchdog, restart, shutdown passed\n", repeats + 1)).unwrap();
}
