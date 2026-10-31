# Reproduce the local CPU path

This experimental path is local only. It does not require a wallet or a
marketplace job. Use one proof at a time. Do not treat these instructions as
approval to run `mine`, `init`, claim, stake or submit commands.

## Prerequisites and source

See BASELINE.md for full source SHAs and hardware. The miner is based on
`1bbdd68edf66f4bb9e72d2aad73e41ae36751d0e`; the modified engine must be based
on `f8dcbdf8124a3f1528146bb01d539c55545cbb3d`, not supplied Hemi HEAD or upstream.
Current paths are `/Users/dbur/dev/zk-miner` and
`/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0`. Preserve both working trees.

Required: ARM64 macOS, CLT C/C++ compiler/SDK, Python 3.11+ (`tomllib`), Git,
Docker/Colima, host Rust 1.93.0, and the pinned RISC Zero guest compiler.
No Metal compiler is needed for this CPU build. The runtime probe is:

```sh
clang -fobjc-arc -framework Foundation -framework Metal scripts/macos/hardware.m -o target/macos/hardware
target/macos/hardware
```

The session installed rustup into `target/macos/toolchain/{cargo,rustup}`,
without modifying shell profiles. Official rustup-init and its SHA-256 record
are in `target/macos/bootstrap`. Guest archive:
`https://github.com/risc0/rust/releases/download/r0.1.88.0/rust-toolchain-aarch64-apple-darwin.tar.gz`,
SHA-256 `f3d6a3dbe09536f7166cff53fd5ded632a0b5409b7cfaa5202423f6c3323c04c`.
It is extracted under `target/macos/toolchain/guest-r0.1.88.0` and linked with
`rustup toolchain link risc0` to that directory. Verify the archive hash before
extracting on another machine. Rust 1.97.1 is installed but not used by this
baseline. No global install or `sudo` is part of the build scripts.

Start in the miner root:

```sh
cd /Users/dbur/dev/zk-miner
export HEMI_MINER_DIR=/Users/dbur/dev/zk-miner
source scripts/macos/env.sh
rustup toolchain list
python3 scripts/macos/local-engine.py /Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0 target/macos/local-engine.toml
```

The generator rejects the wrong HEAD/origin. Do not export x86 RUSTFLAGS,
RISC0_DEV_MODE, skip-build variables, Bonsai credentials, or proof-selection
variables. Acceptance rejects them instead of silently masking them.

## Local wrapping VM

The initial default Colima disk was full. The session created this *separate*
profile; the original context/profile/images were not changed or deleted:

```sh
colima start hemi-macos-cpu --cpu 6 --memory 32 --disk 80 --activate=false --ssh-config=false --template=false --mount /Users/dbur/dev/zk-miner/target/macos:w --vz-rosetta
docker --context colima-hemi-macos-cpu pull --platform linux/amd64 risczero/risc0-groth16-prover:v2025-04-03.1
```

Rosetta is absent; Colima fell back to QEMU x86_64 binfmt. `--vz-rosetta` did
not install Rosetta. Confirm the actual runtime on another host. The required
image digest is `sha256:a4f80ce2e0b8e2bb7637a93c37136a6776ac00ec843a3fdf1c67b1d5ffea64ee`.
Preflight refuses an absent/mismatched image or non-local Docker endpoint.
The shared path must exist and be mounted into the VM. Budget at least the
image/key sizes plus Rust/native build caches; this session's initial free
disk was 532 GiB. No image pull occurs during proving.

## Build and verify sources

```sh
cargo +1.93.0 build --release --features prove --manifest-path tools/macos-proof/Cargo.toml --config target/macos/local-engine.toml --target-dir target/macos/cpu
python3 scripts/macos/build-guest.py fibonacci --config target/macos/local-engine.toml
python3 scripts/macos/build-guest.py sha256-chain --config target/macos/local-engine.toml
python3 scripts/macos/build-guest.py segment-boundary --config target/macos/local-engine.toml
sh scripts/macos/build-verifier.sh
python3 scripts/macos/build-worker.py --config target/macos/local-engine.toml --engine /Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
cargo +1.93.0 build --locked --release -p zkminer-cli --bin zkminer --target-dir target/macos/miner
```

The guest script copies source unchanged into an isolated workspace and seeds
its initial lock from the miner. `--reset-lock` reseeds only that isolated lock;
it was needed once when an earlier fixture lock selected compiler-incompatible
versions. Packing uses the engine's V1COMPAT kernel. No stub builds are allowed.
`build-worker.py` temporarily changes source identities in the root lock, checks
all package versions and intended changes, saves metadata/local lock, then
restores the root Git lock. Run it without concurrent root Cargo mutations.
An unexpected lock diff fails and retains both snapshots for review.

The unchanged verifier build has no local Cargo override or `prove` feature.
`check-source.py` validates all resolved engine packages (17 in the worker,
10 in the verifier in this session). Metadata and feature records are under
`target/macos`; use the helper to reject accidental mixed-source builds.
The native miner itself keeps the root Git lock and talks to the isolated
worker. No replacement of the root engine pin is required.

For a native C++/Metal source change, use a fresh backend-specific target
directory or clean the affected sys/circuit package in that directory and
rebuild. Do not reuse a CPU result as evidence that new Metal kernels ran.

## Local acceptance

```sh
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 2 --workload fibonacci
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 1 --workload sha256-chain
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 1 --workload segment-boundary --po2 16
```

Every attempt gets a new directory and a ledger entry. A multi-segment attempt
must contain at least two segments and complete independent Groth16 verification.
The ledger retains failures. The harness's RSS sampling excludes VM memory.
Do not run the smoke harness concurrently with the worker test.

For real discovery/IPC/admission/lifecycle testing, run from the sourced shell:

```sh
mkdir -p target/macos/worker-work
export ZKMINER_RISC0_DOCKER_CONTEXT=colima-hemi-macos-cpu
export ZKMINER_RISC0_WORK_ROOT=/Users/dbur/dev/zk-miner/target/macos/worker-work
export ZKMINER_RISC0_MODE=cpu
export ZKMINER_MACOS_WORKER=/Users/dbur/dev/zk-miner/target/macos/worker/release/zkminer-prove-risc0
export ZKMINER_MACOS_PROGRAM=/Users/dbur/dev/zk-miner/target/macos/guest-build/riscv32im-risc0-zkvm-elf/release/fibonacci.bin
export ZKMINER_MACOS_VERIFIER=/Users/dbur/dev/zk-miner/target/macos/verifier/release/hemi-macos-proof
export ZKMINER_MACOS_EVIDENCE_DIR=/Users/dbur/dev/zk-miner/target/macos/worker-acceptance-new
cargo +1.93.0 test --locked -p zkminer-prover --test macos_worker -- --ignored --nocapture
```

Use a fresh evidence-directory name each time. This test is ignored by default
because it requires substantial local prerequisites; `--ignored` explicitly
executes it. A skipped test is not a successful proof. Default is three
consecutive successful requests, an intentional wrapping timeout, and a
successful proof after restart. `ZKMINER_MACOS_REPEAT=20` requests the extended
suite; it has not yet been run. It may take approximately an hour with QEMU.

Do not manually run the worker as a second server. The miner's WorkerPool
launches it and supplies the helper lease. CPU and auto use one generic slot;
forced metal fails. Calibration is unsupported. The two tiny benchmark inputs
are not the GPU benchmark suite and must not produce comparative rankings.

## Stop / remove

After tests, ensure no `zkminer-macos-*` / `hemi-macos-*` containers remain in
the dedicated context. Stop only the dedicated profile with
`colima stop hemi-macos-cpu`. Keep evidence and source until reviewed. Removing
`target/macos` would remove the task-local toolchains, artifacts and receipts;
removing the dedicated Colima profile would remove its helper image/cache.
Those destructive cleanup operations were not performed. No global shell,
wallet, upstream checkout or default VM cleanup is needed.
