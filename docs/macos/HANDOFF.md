# macOS proving handoff

Updated 2026-10-31. CPU Gates 1/2 passed. The first complete hybrid Groth16
proof is accepted by the unchanged pinned verifier. Full Gate 3 remains open:
hybrid worker integration, extended stability and controlled performance tests
are not complete. No push, wallet access, job claim, signing or transaction.

## Repositories and commits

Implementation branches remain `codex/hemi-macos-metal` (local only).

* Miner `/Users/dbur/dev/zk-miner`: previous primitive milestone
  `0070e0b45c105ce9488bd9d2d2fef1b14af61e1f`; this milestone's harness and
  evidence changes are in the containing/latest local commit (`git log -1`).
  CPU checkpoint `3503563849661f0e99b5ad9c7da7229bd561ea4f` remains available.
* Engine `/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0`: hybrid stage
  `56a5347e4` plus Intel type-selection fix
  **`0d5b41e8e44876c1e8b5a8a671aabc154e33106c`**. It is a descendant of the
  unchanged miner pin `f8dcbdf8124a3f1528146bb01d539c55545cbb3d`.
  CPU checkpoint `e274e19ee6e77bcd5704ca25c9880bd143f0783c`; primitive checkpoint
  `130f98fb91f6c5ccb1d243a498bf303c760c6388` remain available.
* Supplied Hemi `/Users/dbur/dev/risc0` stays clean/unchanged at
  `6b673a0c26c0c195c1e4c9e59408de25fc434492`.
* Upstream `/Users/dbur/dev/risc0-parent` stays read-only/clean at
  `164c64bb3a844d393bb7addc5781c705c647f2d5`.

No dependency pin/version, field constant, circuit, key, cryptographic verifier
or proof format was replaced. No broad dependency update or upstream import.

## Implemented and validated

The explicit `risc0-zkvm/metal-hybrid` build and
`LocalProver::new_metal(name, staging_bytes)` select only segment Poseidon2
Merkle folds. CPU-owned buffers are copied in bounded chunks to the retained
Metal kernel, waited on, and copied back into the proof. The harness uses
512 KiB staging (API range 4 KiB..16 MiB). Per-chunk autorelease pools prevent
completed native command resources from accumulating. This cap is staging
storage, not a claim about total driver/CPU/VM memory.

Witness generation, leaf hashing, NTTs, constraints, FRI arithmetic, recursion
and identity stay CPU. Final Groth16 uses the existing local Linux/amd64 helper
under QEMU emulation. The worker still rejects `ZKMINER_RISC0_MODE=metal`;
hybrid acceptance is currently through the isolated harness only.

Source-bound primitive/bridge run `767200b6ed0c46058d177bee61fe6354` passed
235 cases / 451 actual GPU completions. Thirteen bridge cases cover two budgets,
chunk boundaries/tails, sliced-buffer sentinels and full-tree reuse. Retained
kernel hash: `2fc1eb3c8a00e4fb40f5055dbaef08074e4c16cbc4e06b89de2bfe1e57d1bcd0`.

First hybrid complete proof `d4e86941bd7b464f9954aa24ea1b2375`: Fibonacci20,
168.439 seconds, 194 actual Metal dispatches, 90 fold calls and 532,986 output
digests. Complete Groth16 verified with original pinned Git verifier; wrong
image, changed journal and corrupt seal all rejected. Second hybrid Fibonacci `1ad9ae716a73463584b840fcf268bac4` passed in 166.856s.
Five-segment hybrid `0551842728b9449ab3d85de41a3668f8` passed in 237.822s with
1,269 Metal completions across all five segments. CPU regression `08b5cca54b024b9ca9d3400f41b23b1b`
passed in 165.581s with no Metal feature/work. All four runs
passed final Groth16 and the three unchanged-verifier negative checks. Exact
records are in VALIDATION.md and evidence/hybrid-validation.json.

Prior CPU evidence remains: arithmetic/SHA/five-segment complete proofs,
WorkerPool three consecutive requests, timeout cleanup, restart proof and
shutdown. Earlier primitive and CPU regression evidence is retained unchanged.
Eight new control tests validate evidence/provenance rejection; their synthetic
fixtures are not counted as proof generation. See VALIDATION.md for failures.

## Environment and reproduction

Mac17,6, Apple M5 Max, 18 CPU / 40 GPU cores, 128 GiB RAM, macOS/SDK26.4.
No manual installation is needed for the working Metal runtime compiler.
Xcode.app and `xcrun metal` remain absent. No privileged installation or global
configuration change was made. Ask before any future privileged installation.

From `/Users/dbur/dev/zk-miner`, source `scripts/macos/env.sh` with
`HEMI_MINER_DIR=/Users/dbur/dev/zk-miner`. Host Rust1.93.0 and guest r0.1.88.0
are task-local. See SETUP.md for the dedicated wrapping VM restart command.

```sh
ENGINE_DIR=/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
ENGINE_REV=0d5b41e8e44876c1e8b5a8a671aabc154e33106c
sh scripts/macos/build-hybrid.sh "$ENGINE_DIR" "$ENGINE_REV"
sh scripts/macos/build-verifier.sh
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --mode metal --engine "$ENGINE_DIR" --repeat 2
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --mode metal --engine "$ENGINE_DIR" --repeat 1 --workload segment-boundary --po2 16
```

The source generator requires exact reviewed HEAD and pinned ancestry. Hybrid
and verifier build seals bind binary, actual sources, locks and metadata;
changes require rebuilding. The verifier builds from the original Git pin,
without proving features or local overrides, using the committed standalone
Verifier.Cargo.lock. CPU/Metal/hybrid/verifier target dirs remain separate.
The runner shares one-proof locking, bounded timeout/RSS, local Docker socket
checks and exact-name helper cleanup across CPU and hybrid modes.

Dedicated profile `hemi-macos-cpu` / context `colima-hemi-macos-cpu` is used
only for local acceptance (6 CPUs,32 GiB VM RAM,80 GiB disk,24 GiB helper cap,
6 helper CPUs, network disabled). It is now stopped after checking there were
no leftover containers. Original default Colima state is untouched. Pinned helper digest:
`sha256:a4f80ce2e0b8e2bb7637a93c37136a6776ac00ec843a3fdf1c67b1d5ffea64ee`.

## Next work and limits

Next concrete step: wire this validated stage into the existing native worker's
explicit Metal mode, keep CPU/auto reporting honest, and repeat cancellation,
cleanup and restart tests with GPU work. Do not enable marketplace jobs by
default. Then run at least 20 consecutive worker requests and controlled five-
run-per-mode measurements before performance or unattended-use claims.

No speedup claim is made. Full GPU fault injection, extended memory-pressure
recovery, non-Mac runtime validation and formal performance tests remain open.
The pinned tiny-tail missing `lift_rv32im_v2_13.zkr` limitation remains; supported
multi-segment fixture uses po2 16. Do not regenerate keys or import upstream
parameters. Network mining is unauthorized and unrun.
