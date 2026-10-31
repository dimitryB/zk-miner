//! Conservative CPU bootstrap. No Metal path is claimed by this worker.
use anyhow::{bail, ensure, Context, Result};
use std::{path::PathBuf, process::Command, time::Duration};

const IMAGE: &str = "risczero/risc0-groth16-prover:v2025-04-03.1";
const DIGEST: &str = "sha256:a4f80ce2e0b8e2bb7637a93c37136a6776ac00ec843a3fdf1c67b1d5ffea64ee";

fn output(command: &mut Command) -> Result<String> {
    let (status, stdout) = zkminer_prover_protocol::proc::output_with_timeout_capturing_stdout(
        command,
        Duration::from_secs(10),
        64 * 1024,
    );
    ensure!(
        matches!(
            status,
            zkminer_prover_protocol::proc::Outcome::Ran { success: true, .. }
        ),
        "helper preflight failed: {}",
        status.describe()
    );
    Ok(stdout.trim().to_owned())
}

pub fn preflight() -> Result<()> {
    for key in [
        "RISC0_DEV_MODE",
        "RISC0_PROVER",
        "RISC0_EXECUTOR",
        "BONSAI_API_URL",
        "BONSAI_API_KEY",
        "RISC0_SKIP_BUILD",
        "RISC0_SKIP_BUILD_KERNELS",
        "RISC0_TEST_INJECT_INVALID",
    ] {
        ensure!(
            std::env::var_os(key).is_none(),
            "unset {key} for native worker acceptance"
        );
    }
    match std::env::var("ZKMINER_RISC0_MODE")
        .as_deref()
        .unwrap_or("auto")
    {
        "cpu" => {
            tracing::info!("requested=cpu observed=cpu; Groth16 helper=linux/amd64 (emulated)")
        }
        "auto" => tracing::info!(
            "requested=auto observed=cpu; fallback reason: Metal proving is not validated"
        ),
        "metal" => bail!("forced Metal is unavailable: no validated Metal proving stages"),
        other => bail!("unknown ZKMINER_RISC0_MODE {other:?}; use cpu, auto, or metal"),
    }
    let context = std::env::var("ZKMINER_RISC0_DOCKER_CONTEXT")
        .context("set a dedicated local Docker context")?;
    let endpoint = output(Command::new("docker").args([
        "context",
        "inspect",
        &context,
        "--format",
        "{{.Endpoints.docker.Host}}",
    ]))?;
    ensure!(
        endpoint.starts_with("unix://"),
        "only a local Docker socket is supported"
    );
    let digest = output(Command::new("docker").args([
        "--context",
        &context,
        "image",
        "inspect",
        IMAGE,
        "--format",
        "{{json .RepoDigests}}",
    ]))?;
    ensure!(
        digest.contains(DIGEST),
        "the pinned Groth16 helper image is missing or has a different digest"
    );
    let name = std::env::var("RISC0_GROTH16_CONTAINER_NAME")
        .context("start the worker through WorkerHandle for helper ownership")?;
    ensure!(
        name.starts_with("zkminer-macos-"),
        "invalid supervisor helper identity"
    );
    let dir = PathBuf::from(
        std::env::var_os("RISC0_WORK_DIR")
            .context("supervisor must supply a shared work directory")?,
    );
    ensure!(dir.is_dir(), "shared helper work directory does not exist");
    std::env::set_var("DOCKER_CONTEXT", &context);
    std::env::set_var("RISC0_PROVER", "local");
    std::env::set_var("RISC0_EXECUTOR", "local");
    std::env::set_var("RAYON_NUM_THREADS", "6");

    // macOS has no Linux PDEATHSIG. This thread exits the worker and cleans only
    // its leased helper if the parent disappears. The parent's lease handles a
    // SIGKILL of this worker (which cannot execute a destructor).
    let parent = unsafe { libc::getppid() };
    std::thread::Builder::new()
        .name("macos-parent-watch".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            if unsafe { libc::getppid() } != parent {
                let _ =
                    output(Command::new("docker").args(["--context", &context, "rm", "-f", &name]));
                std::process::exit(1);
            }
        })?;
    Ok(())
}

pub fn bound_env(builder: &mut risc0_zkvm::ExecutorEnvBuilder<'_>, po2: Option<u8>) -> Result<()> {
    ensure!(
        po2.is_none_or(|value| (14..=18).contains(&value)),
        "po2 value out of valid range 14..=18 for macOS CPU bootstrap"
    );
    builder
        .segment_limit_po2(po2.unwrap_or(18) as u32)
        .session_limit(Some(4 * 1024 * 1024));
    Ok(())
}

pub fn benchmarks() -> &'static [(&'static str, &'static [u8], &'static [u8], f64, bool)] {
    &[
        (
            "fibonacci",
            super::FIBONACCI_ELF,
            &[20, 0, 0, 0],
            1.0,
            false,
        ),
        (
            "sha256-chain",
            super::SHA256_CHAIN_ELF,
            &[10, 0, 0, 0],
            1.0,
            true,
        ),
    ]
}
