# macOS proving handoff

Updated 2026-10-31. CPU Gates 1/2 and the bounded native hybrid worker milestone
passed. Segment Poseidon2 Merkle folds run on Metal and complete Groth16 outputs
pass the unchanged pinned verifier. Full Gate 3 remains open: extended stability,
additional worker workloads and controlled performance measurements are not done.
No push, privileged installation, wallet access, job claim or transaction.

## Repositories and refs

Implementation branch: `codex/hemi-macos-metal` in both implementation worktrees.

* Miner `/Users/dbur/dev/zk-miner`, remote `https://github.com/hemilabs/zk-miner.git`:
  this native-worker change is based on `3b2720110c1721a53d19c09a6ff2a16235aeb9be`;
  its source-bound builds were made with the reviewed uncommitted changes.
  The containing/latest local commit records this milestone (`git log -1`).
* Engine `/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0`, remote
  `https://github.com/hemilabs/risc0.git`: unchanged during this milestone at
  `0d5b41e8e44876c1e8b5a8a671aabc154e33106c`.
* Supplied Hemi `/Users/dbur/dev/risc0`: clean release-3.0 at
  `6b673a0c26c0c195c1e4c9e59408de25fc434492`.
* Read-only upstream `/Users/dbur/dev/risc0-parent`, remote
  `https://github.com/risc0/risc0.git`: clean main at
  `164c64bb3a844d393bb7addc5781c705c647f2d5`.

Miner pin and independent verifier remain
`f8dcbdf8124a3f1528146bb01d539c55545cbb3d`; the local engine descends from it.
No circuit, field constant, key, proof format, dependency version or pinned
revision changed. Root Cargo.lock only adds four existing worker dependency
edges: metal, risc0-zkp, serde_json and sha2. No broad updates/upstream import.
CPU checkpoints remain miner `3503563849661f0e99b5ad9c7da7229bd561ea4f` and
engine `e274e19ee6e77bcd5704ca25c9880bd143f0783c`.

## Implemented native worker

One native worker implementation now supports frozen startup `cpu|metal|auto`
selection. Forced Metal requires an isolated, bounded CPU/Metal differential
probe and actual per-segment GPU work; failures do not retry proving on CPU.
CPU explicitly selects LocalProver's CPU path. Auto reports a startup fallback
reason when the Metal feature/device is unavailable. A generic slot serializes
requests; no fake CUDA device or second unified-memory budget is introduced.

Only segment Poseidon2 Merkle folds run on Metal. Witness, leaf hashing, NTTs,
constraints, FRI arithmetic, recursion and identity remain CPU. Staging defaults
to 512 KiB, configurable from 4 KiB through 16 MiB. Final Groth16 still uses the
pinned local Linux/amd64 helper under QEMU emulation. This is not fully native
wrapping. IPC and non-Mac proving branches retain their existing semantics.

Changed implementation: `crates/zkminer-prove-risc0/{Cargo.toml,src/main.rs,
src/macos.rs,src/macos_metal.rs}`, four root lock dependency edges,
`scripts/macos/{build-worker.py,worker_provenance.py,test-worker.py}` and
`crates/zkminer-prover/tests/macos_worker.rs`. Build seals bind worker sources,
local engine revision/diff, actual dependency graph/lock and executable. The
acceptance runner additionally binds the real WorkerPool test and unchanged
verifier. Optional atomic per-request JSON preserves stage evidence before
wrapping, including interrupted requests, without changing positional IPC.

## Executed evidence

Native Metal lifecycle run `f8ff5abb2cef4a19bb02e24fbcb318d0` passed:
2 tests, 0 failed, 0 ignored, 777.73 seconds. Four complete Groth16 proofs were
independently verified, with wrong-image, changed-journal and corrupt-seal
rejection each time. Three consecutive proofs used one worker PID; cancellation
and a separate 70-second watchdog killed owned workers only after real wrapping
helpers appeared and Metal work was recorded. Helpers were cleaned, a restart
proof passed, and shutdown left no owned worker/helper. Proof seconds:
163.997, 159.590, 159.140 and 164.465; each has 194 completed GPU dispatches.
Sampled native worker peak was 2,013,675,520 bytes, excluding the VM/helper;
this is a per-PID run peak, not a driver-memory or per-proof growth measurement.

The mode matrix also passed: CPU build/auto (`2b17330c44b84da099de8089a72032aa`,
164.497s) records fallback because it lacks the Metal feature; Metal build/cpu
(`0a20964eec5f4643b0805f52e49a4d44`,164.718s) explicitly stays CPU. Both have
zero GPU proving work. Metal build/auto (`ac9cf5773bf34427925ab5f29ce18ac5`,
164.798s) selects Metal and records 194 completions. Across four runs: seven
complete independently verified proofs (five Metal, two CPU), 21 tamper rejections,
eight actual tests passed, zero ignored. All proofs match program/input hashes,
image, journal, po2 and cycle/segment stats; randomized proof bytes are not compared.
Two mode/deadline unit tests and actual preflight checks passed. Runtime probe equality passed at budgets 4096, 4097,
524288 and 16777216 bytes; invalid lower/upper budgets reject. The full suite
also executes invalid mode, dev/skip-verification, malformed/out-of-range budget,
missing helper, protocol mismatch and invalid guest checks.

Prior source-bound harness evidence remains valid: 235 differential/bridge
cases, 451 actual GPU commands; two complete hybrid Fibonacci proofs and one
five-segment hybrid proof; independent CPU regression and earlier CPU hash/
multi-segment/worker acceptance. See VALIDATION.md and evidence/hybrid-validation.json.
These earlier workload results do not imply the new native worker ran them.

## Environment and reproduction

Discovered Mac17,6 Apple M5 Max, 18 CPU/40 GPU cores, 128 GiB unified memory,
macOS/SDK26.4. No manual installation is needed: the Metal runtime compiler
works without Xcode.app or `xcrun metal`, both still absent. Ask before any
future privileged installation. Host Rust1.93.0 and guest r0.1.88.0 are task-local.

From `/Users/dbur/dev/zk-miner`:

```sh
export HEMI_MINER_DIR=/Users/dbur/dev/zk-miner
source scripts/macos/env.sh
ENGINE_DIR=/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
ENGINE_REV=0d5b41e8e44876c1e8b5a8a671aabc154e33106c
python3 scripts/macos/build-worker.py --mode metal --engine "$ENGINE_DIR" --config target/macos/local-engine.toml --revision "$ENGINE_REV"
python3 scripts/macos/build-worker.py --mode cpu --engine "$ENGINE_DIR" --config target/macos/local-engine.toml --revision "$ENGINE_REV"
python3 scripts/macos/test-worker.py --engine "$ENGINE_DIR" --worker-kind metal --mode metal --context colima-hemi-macos-cpu --repeat 3
```

SETUP.md documents local source generation, verifier build and dedicated VM
restart. CPU and Metal workers have the same basename in separate `worker` and
`worker-metal` target dirs; never substitute the build under an existing seal.
The verifier target is separate, built from original Git pin without local
patches/proving features. Raw runs, seals and logs are under
`target/macos/worker-runs/<id>`; aggregate evidence is `docs/macos/evidence/native-worker-validation.json` and
`native-worker-measurements.csv`, with executed commands in VALIDATION.md.
The separate miner diff is `target/macos/native-worker-metal.patch`; no engine
diff was introduced in this milestone. Previous engine patches remain under
docs/macos/evidence. Source changes require rebuilding sealed artifacts.

Dedicated profile `hemi-macos-cpu` / context `colima-hemi-macos-cpu` is
stopped after acceptance and confirming no containers remained. It has 6 CPUs,
32 GiB VM RAM,80 GiB disk; each helper is
capped at 24 GiB/6 CPUs with network disabled. Original Colima is untouched.
Pinned helper image digest:
`sha256:a4f80ce2e0b8e2bb7637a93c37136a6776ac00ec843a3fdf1c67b1d5ffea64ee`.

## Next work and limits

Next concrete step after this milestone: extend the source-bound native worker
suite to hash/multi-segment and bounded memory-pressure cases, then run at least
20 consecutive successful requests before recommending unattended test use.
Run warmup plus at least five measured complete runs per mode with identical
settings, cold-start separation, failure retention and thermal/background-load
context before performance claims. Current smoke timings are not benchmarks.

GPU hardware-fault recovery, driver allocation peaks/growth trends and full
Linux/CUDA/ROCm runtime regression remain unrun. Optional offline Linux CUDA
metadata failed on uncached `approx v0.5.1`; it is not a passed build. Pinned
`lift_rv32im_v2_13.zkr` remains unavailable, so earlier supported multi-segment
coverage uses po2 16. Do not regenerate keys or substitute artifacts. Exact
contract-adapter/deployed verifier acceptance and network mining remain separate,
unrun and unauthorized. No speedup or production-readiness claim is made.
