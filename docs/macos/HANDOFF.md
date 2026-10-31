# macOS proving handoff

Updated 2026-10-31. CPU Gates 1/2 passed; first runtime Metal primitive
differentials passed. Complete hybrid Gate 3 and network acceptance remain open.
No push, wallet access, job claim, signing or transaction occurred.

## Repositories and commits

Both implementation branches are now `codex/hemi-macos-metal` (new local
branches requested by the user). CPU checkpoints are preserved:

* Miner `/Users/dbur/dev/zk-miner`: CPU checkpoint
  `3503563849661f0e99b5ad9c7da7229bd561ea4f`, based on
  `1bbdd68edf66f4bb9e72d2aad73e41ae36751d0e`. The Metal harness/evidence commit
  is the containing/latest commit on this branch (`git log -1`).
* Engine `/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0`: CPU checkpoint
  `e274e19ee6e77bcd5704ca25c9880bd143f0783c`, latest Metal commit
  `130f98fb91f6c5ccb1d243a498bf303c760c6388`, based on the unchanged miner pin
  `f8dcbdf8124a3f1528146bb01d539c55545cbb3d`.
* Supplied Hemi `/Users/dbur/dev/risc0` remains untouched at
  `6b673a0c26c0c195c1e4c9e59408de25fc434492`.
* Upstream `/Users/dbur/dev/risc0-parent` remains read-only/untouched at
  `164c64bb3a844d393bb7addc5781c705c647f2d5`.

No dependency pin/version, circuit, key, verifier or proof format was replaced.

## Validated state

CPU: genuine arithmetic, SHA-256 and five-segment Groth16 proofs independently
verify. WorkerPool delivered three consecutive successful requests, cleaned a
helper after a deliberate wrapping timeout, restarted for a fourth verified
proof, and shut down. One conservative warm-worker recycle occurred. See
VALIDATION.md for exact code/test snapshots and all failed attempts.

CPU regression after enabling the optional Metal build: run
`target/macos/runs/f738364b836b4e3a902203f6151f6276` passed complete Groth16 and
unchanged-verifier positive/three negative checks (166.672 seconds). The CPU
source guard confirms Metal was not enabled. All earlier artifacts remain.

Metal: the existing MTLDevice runtime compiler works without an offline compiler
or privileged installation. Explicit `risc0-zkp/metal-runtime` embeds retained
kernel sources; ordinary CPU and offline `metal` defaults remain distinct.
The worker still rejects forced Metal—no proof stage is accelerated yet.

Final source-bound run
`target/macos/metal-runs/54482afeee90464dbca9fbc8183d5b06` passed 222 bounded
cases with 404 actual GPU completions on the committed engine. Coverage includes
field arithmetic, offsets/sentinels/empty work, ownership/reuse, NTT and SHA/
Poseidon rows/folds. Narrow fixes addressed buffer slice/tracking defects and
Poseidon's empty-column condition. All failures are retained; see METAL.md.

## Environment and commands

Mac17,6 M5 Max, 18 CPU / 40 GPU cores, 128 GiB RAM, macOS/SDK26.4.
Source `scripts/macos/env.sh` from the miner root: host Rust1.93.0 and guest
r0.1.88.0 are task-local. Xcode.app and `xcrun metal` remain absent. No privileged
installation/global developer-directory change was made. Ask before any future
privileged installation, as the user explicitly requires.

Review the engine HEAD/diff before passing an explicit full revision:

```sh
cd /Users/dbur/dev/zk-miner
export HEMI_MINER_DIR=/Users/dbur/dev/zk-miner
source scripts/macos/env.sh
ENGINE_DIR=/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
ENGINE_REV=$(git -C "$ENGINE_DIR" rev-parse HEAD)
sh scripts/macos/build-metal.sh "$ENGINE_DIR" "$ENGINE_REV"
python3 scripts/macos/test-metal.py --engine "$ENGINE_DIR" --mode all
```

The source generator requires exact reviewed HEAD and pinned-baseline ancestry;
CPU guards reject Metal graphs. The Metal runner validates its build-time binary/
source/lock/metadata binding before execution. Separate CPU/Metal target dirs
prevent feature/cache confusion. Rebuild after engine, harness or lock changes.

The dedicated Colima profile `hemi-macos-cpu` / Docker context
`colima-hemi-macos-cpu` is stopped. It uses 6 CPUs,32 GiB RAM,80 GiB disk and
QEMU x86 wrapping (Rosetta absent). SETUP.md gives its restart command. Original
Colima profile/state is untouched. Pinned helper digest:
`sha256:a4f80ce2e0b8e2bb7637a93c37136a6776ac00ec843a3fdf1c67b1d5ffea64ee`.

## Evidence and next work

`METAL.md`, `evidence/metal-validation.json`, `metal-final-cases.json`, and
`hemi-metal.patch` record this milestone. CPU baseline evidence remains in
BASELINE/VALIDATION/BENCHMARKS and the earlier JSON/patch files. Raw proofs,
metadata, binaries and logs remain under `target/macos`; nothing was removed.

Next concrete implementation: select one pinned-compatible generic Metal stage
inside segment proving, retain CPU witness/recursion/identity and the local
wrapping helper, observe actual GPU execution and require a full proof accepted
by the unchanged verifier. Do not enable the worker's Metal mode before that.
Extend FRI/circuit differential coverage, GPU error/lifetime tests and bounded
unified-memory accounting. The CPU's missing `lift_rv32im_v2_13.zkr` tiny-tail
limitation remains; do not regenerate keys or import upstream parameters.

Twenty-request worker stability and controlled five-run-per-mode performance
comparisons are not run. Do not recommend unattended use or claim speedup.
Offline compiler provisioning may follow with compatible Xcode and any required
explicit privileged-install approval. Network mining remains opt-in and unrun.
