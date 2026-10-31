# Native CPU proving implementation

The miner keeps backend `risc0` and protocol v5 unchanged. It discovers the
new generic executable `zkminer-prove-risc0` using existing discovery. A
`macos` feature compiles the shared worker for aarch64-apple-darwin without
CUDA/ROCm. Existing non-Mac code paths remain behind their original features.
One generic worker serializes requests; CPU and Metal are not separate devices.
No marketplace commands are invoked by setup or acceptance scripts.

## Cryptographic boundary

All engine packages derive from Hemi commit
`f8dcbdf8124a3f1528146bb01d539c55545cbb3d`. Six engine files change: explicit
Metal feature propagation/build gating and opt-in macOS helper ownership.
Circuit code, kernels, security parameters, keys, verifier parameters, public
inputs and raw 256-byte Groth16 seal format are unchanged. Upstream 5.0.0
HEAD is a reference only; its restored Metal circuit APIs are not a drop-in
replacement for this SDK 3.0.6 tree.

The harness directly constructs LocalProver. Worker preflight rejects remote,
dev, skip-build and injection overrides, then selects local executor/prover;
metadata confirms no Bonsai/GPU feature closure. Both compile with
`disable-dev-mode`. The independent verifier is a separate build with the
original Git source, no local patches and no proving features. It accepts the
complete receipt or reconstructs the expected receipt claim from raw worker
seal/journal, then rejects wrong image, changed journal and corrupted seal.
This tests the worker's real output format, not a deployed contract adapter.

Guests are unchanged miner fixtures built in isolated workspaces. The v3
ProgramBinary contains the authentic user ELF plus the pinned V1COMPAT kernel;
a raw user ELF is not accepted as that container. Source, ELF, kernel, packed
program, image and input hashes are retained. Patches are explicit for both
crates.io and Git sources and for each independent guest workspace. Metadata
is checked, not inferred from manifest entries. Root lock changes add only
existing `libc` and `risc0-binfmt` dependencies to the worker: no package/version
updates. `build-worker.py` records a temporary local-source lock and restores
the Git lock after validating its diff. Do not run concurrent Cargo commands
that mutate that root lock during this script.

## New interfaces and limits

* Build feature `macos`, binary `zkminer-prove-risc0`, authentic packed guests
  supplied through `ZKMINER_RISC0_GUEST_DIR`; absent/stub guests fail the build.
* `ZKMINER_RISC0_MODE=cpu|auto|metal`. Default auto reports CPU fallback because
  Metal is not validated. Forced metal fails before handshake. CPU means native
  CPU execution/segments/recursion/identity with emulated final wrapping.
* `ZKMINER_RISC0_DOCKER_CONTEXT`: explicit local Unix-socket context with the
  pinned image already present. `ZKMINER_RISC0_WORK_ROOT`: existing shared local
  directory. The parent creates a unique owned subdirectory and helper name.
* Internal supervisor settings `RISC0_GROTH16_CONTAINER_NAME` and
  `RISC0_WORK_DIR` are per worker lease; sequential requests reuse the lease,
  and each helper is `--rm`. Normal worker use requires the parent supervisor.
* Prove/Execute: at most 4M guest cycles, po2 14..18 (default 18), six Rayon
  threads, no Mac proof retry ladder. Inputs outside those bounds fail.
* Two small genuine benchmark programs are advertised: Fibonacci(20) and
  SHA-256-chain(10). Calibration is explicitly unsupported. Benchmark values
  are not comparable to existing GPU suite scores because input sizes differ.
* Warmup reports helper preflight, not a fabricated proof or GPU warmup.
  IPC stdout remains binary-only; mode and stage diagnostics use stderr.

## Resource and lifecycle policy

The supplied default Colima VM was full. A separate local VM uses 6 CPUs,
32 GiB RAM and 80 GiB disk; Rosetta is not installed, so its x86 helper uses
QEMU binfmt. Image and proving artifacts are pinned (see evidence). Each
worker helper has `--pull never --network none --memory 24g --cpus 6` and an
internal 1800-second `timeout` backstop. The parent owns the exact helper name
and removes only that helper on shutdown/timeout/drop. No global prune/kill
is used. A worker parent-watch thread also cleans its lease if the parent dies.
If both processes are forcibly killed or Docker is unreachable, the helper's
internal deadline is the remaining bound; daemon failure recovery is not tested.

macOS process identity uses proc_pidinfo start time, parent PID and group
leader identity; missing identity fails closed. Each worker owns its process
group. Native proc_listpids enumerates surviving group members, excluding
zombies, while an unreaped leader pins its identity. Process-group unit tests
exercise real child processes. Linux proc/cgroup paths are preserved.

Host capacity comes from hw.memsize. Admission uses a conservative *estimate*
`(free_pages + min(file_backed_pages, inactive_pages)/2) * page_size`, then
reserves 16 GiB. This is not Darwin's guarantee of allocatable memory or a
memory-pressure metric. Free pages alone proved unusable after filesystem
cache growth. Anonymous/compressed pages receive no separate credit; overlapping
cache counters are not added. A one-second watchdog checks worker RSS <=32 GiB
and the available estimate >=8 GiB; failed observations terminate only the
identity-checked owned process group. Sampling is not a kernel memory limit.
Helper memory is capped separately inside the 32 GiB VM, whose host footprint
already reduces host availability. There is no invented GPU capacity and no
second 128 GiB budget. Native worker cgroup-style anonymous/high-water metrics
remain unavailable; the existing scheduler may conservatively recycle a warm
worker instead of crediting unknown retained memory. Harness RSS samples exclude
the VM/helper and are explicitly labeled; do not add overlapping figures.

Synchronous proving cannot consume IPC Cancel mid-proof. The parent deadline
kills the owned worker group and cleans its helper, releases the slot and
permits a restart. The real acceptance suite deliberately times out while a
helper exists, then requires another independently verified complete proof.
Malformed/oversized framing and version checks use the existing protocol.

## Known envelope and next Metal step

The pinned recursion archive lacks `lift_rv32im_v2_13.zkr`; a tiny final segment
can fail after valid segment proving. A boundary run at po2 14 exposed this.
No artifact/key regeneration or upstream parameter substitution is permitted.
A different supported segmentation must be verified explicitly; this worker
cannot claim support for all marketplace jobs. Keccak-specific coverage and
other guest precompiles are not established by SHA-256-chain coverage.

No complete proof uses Metal yet. The explicit experimental runtime compiler
route now passes retained HAL arithmetic/NTT/hash differential tests; see
METAL.md. The offline compiler is still absent (`xcrun --find metal` exits 72;
no full Xcode). Provision it only with any required privileged-change approval.
The next proving step is selecting one compatible segment stage. Newer upstream circuit/recursion
code must be adapted selectively to the pinned engine, not imported wholesale.
Feature gating and standalone HAL validation do not establish a hybrid prover.
The Metal budget/synchronization and offset/boundary/error tests remain future
work; no speedup, unattended stability or profitability claim is made.
