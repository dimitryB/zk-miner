# Reproduce the local CPU and experimental hybrid paths

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
No offline Metal compiler is needed for CPU builds or the runtime-compiled
hybrid path. The existing MTLDevice compiler has already compiled and dispatched
kernels on this Mac, so no manual installation is needed for the current work.
Do not install Xcode components, Rosetta or other privileged software without
owner approval. The runtime capability probe is:

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
ENGINE_DIR=/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
ENGINE_REV=0d5b41e8e44876c1e8b5a8a671aabc154e33106c
python3 scripts/macos/local-engine.py "$ENGINE_DIR" target/macos/local-engine.toml --revision "$ENGINE_REV"
```

Review the engine commit/diff before supplying ENGINE_REV. The generator requires
that exact HEAD, the Hemi origin, and ancestry from the pinned baseline; it rejects
an unreviewed descendant unless --revision is explicit. Do not export x86 RUSTFLAGS,
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
cargo +1.93.0 build --locked --release --features prove --manifest-path tools/macos-proof/Cargo.toml --config target/macos/local-engine.toml --target-dir target/macos/cpu
python3 scripts/macos/build-guest.py fibonacci --config target/macos/local-engine.toml
python3 scripts/macos/build-guest.py sha256-chain --config target/macos/local-engine.toml
python3 scripts/macos/build-guest.py segment-boundary --config target/macos/local-engine.toml
sh scripts/macos/build-verifier.sh
python3 scripts/macos/build-worker.py --mode cpu --engine "$ENGINE_DIR" --config target/macos/local-engine.toml --revision "$ENGINE_REV"
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
`build-verifier.sh` always copies the reviewed standalone
`tools/macos-proof/Verifier.Cargo.lock` and builds with `--locked`; external
package versions and the Hemi Git pin match the root lock. The script seals the
copied source/lock, metadata and executable. Both CPU and Metal smoke tests
require this seal: rerun `build-verifier.sh` if the verifier predates provenance
sealing or its sealed inputs changed. No dependency update is needed.
`check-source.py` validates all resolved engine packages (17 in the worker,
10 in the verifier in this session). Metadata and feature records are under
`target/macos`; use the helper to reject accidental mixed-source builds.
The native miner itself keeps the root Git lock and talks to the isolated
worker. No replacement of the root engine pin is required.

For a native C++/Metal source change, use a fresh backend-specific target
directory or clean the affected sys/circuit package in that directory and
rebuild. Do not reuse a CPU result as evidence that new Metal kernels ran.

## Build the experimental hybrid harness

The new local API `LocalProver::new_metal(name, staging_bytes)` requires the
`risc0-zkvm/metal-hybrid` feature. The isolated `tools/macos-hybrid` manifest
selects it explicitly and shares the proof harness's source. CPU builds retain
their defaults; native-worker Metal support is selected separately below. Review
the selected commit before supplying its full SHA:

```sh
ENGINE_DIR=/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
ENGINE_REV=0d5b41e8e44876c1e8b5a8a671aabc154e33106c
sh scripts/macos/build-hybrid.sh "$ENGINE_DIR" "$ENGINE_REV"
sh scripts/macos/build-verifier.sh
```

The hybrid executable is `target/macos/hybrid/release/hemi-macos-hybrid-proof`;
it does not replace `target/macos/cpu/release/hemi-macos-proof`. The build uses a
separate target and lock, requires the zkvm/circuit hybrid feature chain plus
the runtime Metal HAL, and seals the binary to engine HEAD/diff, source/lock,
overrides and dependency metadata. Stage new engine files before building so
the source diff records them; rebuild after changing sealed inputs.

Only segment Poseidon2 Merkle pair folds use Metal. The harness sets a **512 KiB
per-chunk staging cap** and reports actual completed work for every segment.
Witness generation, NTTs, leaf hashing, other segment operations, recursion and
identity remain CPU. Final Groth16 wrapping still uses the pinned local
Linux/amd64 helper under emulation. The staging cap is not the whole process or
VM memory budget. See [METAL.md](METAL.md) for API limits and synchronization.

## Build and configure the native Metal worker

The native-worker integration has passed bounded forced-Metal lifecycle
acceptance: three consecutive complete proofs, observed wrapping cancellation
and watchdog cleanup, a verified proof after restart, and clean shutdown.
Run `f8ff5abb2cef4a19bb02e24fbcb318d0` contains four unchanged-verifier-accepted
Groth16 proofs with 194 completed Metal commands each. The CPU-build auto
fallback and Metal-build forced-CPU/auto cases also passed. Across four
source-bound runs, seven complete proofs (five Metal, two CPU) passed independent
verification and 21 negative checks. Full Gate 3 remains open for extended
stability and controlled measurements. See [VALIDATION.md](VALIDATION.md) for
actual outcomes, separately from the standalone hybrid harness.

```sh
ENGINE_DIR=/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
ENGINE_REV=0d5b41e8e44876c1e8b5a8a671aabc154e33106c
python3 scripts/macos/build-worker.py --mode metal --engine "$ENGINE_DIR" --config target/macos/local-engine.toml --revision "$ENGINE_REV"
```

The `macos-metal` worker feature is a marker plus optional host dependencies.
The build script explicitly adds the local SDK's `risc0-zkvm/metal-hybrid`
feature while preserving the original Git pin in the root manifests and lock.
The Metal-capable executable is
`target/macos/worker-metal/release/zkminer-prove-risc0`; CPU remains
`target/macos/worker/release/zkminer-prove-risc0`. The basename, discovery and
wire protocol are unchanged. Build CPU and Metal serially; source/binary seals
must match before acceptance.

Set `ZKMINER_RISC0_MODE=cpu`, `metal`, or `auto` before the supervisor starts the
worker. Selection is frozen at startup. CPU skips the GPU probe. Forced Metal
requires a successful isolated three-second probe that compiles retained kernels
and compares actual GPU folds with CPU. Auto selects Metal on success or records
a reason for CPU fallback on startup failure. Once selected, request failures do
not trigger a backend change. A CPU-only build in auto mode records the missing
Metal feature as its fallback reason.

`ZKMINER_RISC0_METAL_STAGING_BYTES` defaults to `524288` bytes and must be within
`4096..=16777216`. This bounds fold staging, not whole-process or VM memory.
Optional `ZKMINER_RISC0_EVIDENCE_DIR` enables request JSON sidecars with
requested/observed mode, actual segment work, timings, proof phases and hashes;
the acceptance runner creates fresh sidecar directories automatically. Proof and
benchmark requests share the bounded staged proving path and final verification.
No new fields are added to the worker protocol. See [METAL.md](METAL.md) for the
stage boundary and probe details.

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

To request the separate forced-Metal harness, after the hybrid build:

```sh
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --mode metal --engine "$ENGINE_DIR" --repeat 2 --workload fibonacci
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --mode metal --engine "$ENGINE_DIR" --repeat 1 --workload segment-boundary --po2 16
```

`--mode cpu` is the default; `--mode metal` requires `--engine` and never silently
falls back to an entirely CPU proof. Both modes share the one-proof lock. Metal
attempts archive the checked hybrid and unchanged-verifier build seals. A passing
result requires the full Groth16 proof, expected identity/public output,
positive bounded Metal fold work for every segment, and independent verification
including wrong-image, changed-journal and corrupted-seal rejection. Failed
attempts and intermediate receipts remain recorded. These commands are
reproduction instructions; consult [VALIDATION.md](VALIDATION.md) for actual
executed outcomes.

For source-bound discovery/IPC/admission/lifecycle testing, use the new runner
from the sourced shell after building the chosen worker and unchanged verifier:

```sh
python3 scripts/macos/test-worker.py --engine "$ENGINE_DIR" --worker-kind cpu --mode auto --context colima-hemi-macos-cpu --repeat 1 --proof-only
python3 scripts/macos/test-worker.py --engine "$ENGINE_DIR" --worker-kind metal --mode cpu --context colima-hemi-macos-cpu --repeat 1 --proof-only
python3 scripts/macos/test-worker.py --engine "$ENGINE_DIR" --worker-kind metal --mode metal --context colima-hemi-macos-cpu --repeat 3 --staging-bytes 524288
python3 scripts/macos/test-worker.py --engine "$ENGINE_DIR" --worker-kind metal --mode auto --context colima-hemi-macos-cpu --repeat 1 --proof-only
```

The worker kind selects the sealed CPU or Metal-capable binary; mode selects
its startup policy. The CPU/auto case requires an explicit recorded fallback
reason. The Metal/auto case expects actual Metal selection. Every command
requires complete Groth16 proofs accepted by the unchanged verifier, including
wrong-image, changed-journal and corrupted-seal rejection. Metal proofs also
require positive bounded GPU work for every segment. The forced-Metal lifecycle
command and the three listed CPU/auto cases have passed for this worker
revision. The CPU-build auto case recorded an explicit missing-feature fallback;
forced CPU in the Metal-capable build produced no GPU proof work, and auto in
that build recorded 194 GPU proof commands. Exact run IDs and times appear in
[METAL.md](METAL.md); these are correctness checks rather than a benchmark.

Each run acquires the same one-proof lock as smoke tests, creates a fresh
`target/macos/worker-runs/<id>` directory, verifies build seals before and after,
and preserves reports and failed attempts. The runner explicitly executes the
ignored real-worker test; skipped tests are not accepted. Full lifecycle runs
require at least three consecutive proofs, observed running-helper cancellation
and timeout failures, and one independently verified proof after restart.
`--proof-only` excludes lifecycle cases. `--repeat 20` requests extended stability
coverage, which has not been established; it can take roughly an hour with QEMU.
The default acceptance-execution bound is 3600 seconds, excluding test
compilation; `--timeout` changes it, and
`--failure-timeout` bounds the intentional timeout case (default 70 seconds).
Owned native-worker RSS is sampled against 32 GiB; helper VM memory is excluded.
The accepted forced-Metal lifecycle attempt reached a maximum sampled worker
RSS of 2,013,675,520 bytes. This is the per-process maximum over the whole attempt,
not a per-proof peak, and does not establish a GPU allocation peak or memory-growth
trend. The stage cap is logical fold staging, excluding
fixed kernel constants and driver overhead. Metal's recommended working-set
size is a performance recommendation, not an entitlement to allocate that much.

Keep the 20-request stability target and controlled benchmarks as separate
checks. Performance comparisons require a cold-start record, warm-up and at
least five complete measured runs per mode with identical engine, guest/input,
po2, proof kind and thread/resource settings. Report median/range, failures and
thermal/background-load limitations. Native worker hash/precompile,
multi-segment and bounded memory-pressure workloads remain separate coverage;
this runner currently exercises the Fibonacci fixture.

Do not manually start a second worker server. The existing WorkerPool launches
one generic macOS slot and supplies the helper lease for either selected mode.
Calibration remains unsupported. The two tiny benchmark fixtures now use the
same verified staged path as proofs, but do not establish comparative GPU
rankings, speedup or profitability.

## Stop / remove

After tests, ensure no `zkminer-macos-*` / `hemi-macos-*` containers remain in
the dedicated context. Stop only the dedicated profile with
`colima stop hemi-macos-cpu`. Keep evidence and source until reviewed. Removing
`target/macos` would remove the task-local toolchains, artifacts and receipts;
removing the dedicated Colima profile would remove its helper image/cache.
Those destructive cleanup operations were not performed. No global shell,
wallet, upstream checkout or default VM cleanup is needed.
