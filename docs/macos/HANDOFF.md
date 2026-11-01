# macOS proving handoff

Updated 2026-11-01. CPU Gates 1/2 and the bounded native hybrid worker milestone
passed. Expanded workloads and twenty consecutive supervised-worker requests
also passed. Segment Poseidon2 Merkle folds run on Metal; complete Groth16
outputs pass the unchanged pinned verifier. The controlled Fibonacci comparison is now complete; broader workload
performance and fault/memory coverage remain open. No production-readiness claim.
No push, privileged installation, wallet access, job claim or transaction.

## Current completed controlled benchmark

Campaign `f57fcc0ff71d4d6391605cb761c0a1a6` completed on 2026-11-01 in
2234.736s (37.246 minutes): 12 genuine complete Groth16 proofs, 36 negative
verifier rejections, 24 native tests passed, zero failed/ignored or retries.
Two warmups are excluded; all five measured CPU/Metal pairs are retained.
The immutable plan/accepted miner HEAD is `a3342fb80ec2fcfba909f921e381c6636fefb731`;
the containing results commit changes only documentation/evidence. Engine,
production worker/protocol code, pins and sealed CPU/Metal/verifier binaries
remain unchanged. Earlier failed attempt `f8b460…` is retained separately.

Same sealed Metal-capable binary, explicit CPU/Metal modes, Fibonacci(20), po2
cap 18, six threads, 512 KiB staging and the same local emulated Groth16 helper.
Measured worker-body median/range seconds: CPU 165.582 [164.774–166.101], Metal
165.823 [165.439–166.970]. Execution+segment: CPU 2.731 [2.723–2.843], Metal 2.791
[2.723–2.922]. No observed speedup for this fixture. Identity plus wrapping is
about 93.8% of the body; this does not establish performance on larger workloads.
Every Metal proof recorded 194 actual GPU commands (1,164 total); CPU proving
recorded none. Fresh worker startup is timed separately; no persistent warm-
worker or cold-boot claim. No application was closed or guard reduced.

Evidence: `docs/macos/evidence/benchmark-comparison.json/.csv`, companion
`-acceptance.json/.csv`, and `benchmark-comparison-controls.json` (13 reporting
guards passed, zero proofs). Raw runs are bound in the campaign archive and
BENCHMARKS.md reports sample counts, ranges, paired ratios and limitations.
A separate review recomputed results and checked 365 archived artifact hashes,
142 planned source hashes, worker/verifier seals and all 36 negative records.

Memory nuance: the earlier ad hoc `vm_stat` readiness numbers below omitted
speculative pages, so they were conservative proxies, not exact production
policy readings. `benchmark-memory-counter-check.json` verifies the SDK counter
mapping. Production guard/resource records were correct and unchanged; their
observed available range was 25.122–42.469 GiB during this campaign. All 240
thermal snapshots were nominal/AC, with the documented unknown-state caveat;
background fseventsd and other activity remained. No energy/GPU-allocation peak
or broad statistical significance claim is made.

No workers/containers remained after acceptance. The dedicated VM is stopped;
the original default VM remains running. No privileged installation, push,
wallet access, job claim or transaction occurred. Next: benchmark the already
validated hash and multi-segment fixtures under the same fixed protocol before
choosing another acceleration stage. Broader resource/fault and deployment gates
remain open; see VALIDATION.md for exact commands and unrun checks.

## Earlier benchmark attempt and readiness checks

Readiness recheck on 2026-11-01: policy-estimated available memory was
29.324 GiB with the dedicated VM stopped and 27.596 GiB after restarting it
with the same 6 CPU/32 GiB configuration. This clears the unchanged 22 GiB
startup floor but is below the previous campaign's 32.955 GiB pre-proof level.
That warmup ended at 21.212 GiB (11.743 GiB less; this includes all host and
background activity, not an isolated helper allocation measurement).
A conservative retry target is 36–38 GiB available **with the VM running**,
roughly 8–10 GiB more than this recheck. This is guidance, not a new admission
rule or allocation guarantee. No new proof was launched. `vm_stat`, the existing
Darwin policy formula, `colima list --json`, and the same documented VM start/stop
commands were used. The dedicated VM was stopped again; default remains running.
No code, pins, CPU binaries or resource guards changed; no push or installation.

Instrumentation/runner/thermal helper are locally committed at `461965e` on
`codex/hemi-macos-metal`; the following collector/evidence/documentation commit
records this attempt. Production workers, engine and CPU path remain unchanged.
The first campaign **did not complete**: host memory admission refused Metal
warmup before proving. Do not infer a CPU/Metal speedup from that attempt.

Campaign `f8b460c76d594ae88e6430ab29e1aa27` planned one complete warmup per mode,
then five alternating complete trials per mode. It uses the same sealed
Metal-capable worker in explicit CPU/Metal modes, identical Fibonacci(20), po2 18,
512 KiB staging, six Rayon/helper CPUs, 32 GiB dedicated VM and 24 GiB helper cap.
Fresh WorkerPools receive readiness and execute validation before proving;
startup/setup, exact prove-call latency, external verification and cleanup have
separate monotonic timers. This is neither a cold-boot nor a persistent warm-worker
benchmark. Each successful proof requires unchanged-verifier acceptance and
three tamper rejections; no automatic proof retries or sample removal.

CPU warmup `257de3edef294d10b2f03edda0048d95` completed a real Groth16 proof:
worker total 164.432s, supervisor prove call 164.731s, external verification
39.937ms; all three negatives rejected, zero proving GPU commands. Metal warmup
`dc005b12db1d44a5a98dad07ecb7ed01` failed production admission: 23,597,678,592 bytes
available at the pre-call sample (21.977 GiB), below the unchanged 22 GiB
requirement (6 GiB proof estimate + 16 GiB host reserve). Recycling its small
preflight worker did not free enough memory. No Metal proof/helper started.
Ten measured trials remain unattempted; no comparative statistics are emitted.
Across both runs, 3 actual Rust tests passed, 1 failed, 0 ignored.

Source-bound archive: `docs/macos/evidence/benchmark-attempt-1.json/.csv` and
`benchmark-attempt-1-acceptance.json/.csv`; raw plan, ledger, telemetry and logs
are in `target/macos/benchmarks/f8b460c76d594ae88e6430ab29e1aa27`, with receipts
under `target/macos/worker-runs/<run-id>`. The 13 collector guards passed and
count as zero proofs (`benchmark-summary-controls.json`). Source hashes bind
all inputs; the failed trial and ten unattempted rows are retained.

Host telemetry showed AC power and Foundation nominal/low-power=false; those
API values can also mean unknown. `pmset` had no recorded warnings, not a measured
absence of throttling. Background fseventsd CPU activity was retained (up to
100.4% in decaying `ps` samples). No temperature, wattage or GPU driver-allocation
peak was measured. Available-memory readings stayed around 21.8–21.9 GiB after
the refusal. The user was asked to free roughly 4–8 GiB from unused applications;
no application was closed and no admission guard or resource limit was reduced.

Recovery procedure used for the successful campaign: after headroom recovered,
restart the dedicated VM and rerun the entire campaign, retaining attempt 1.
Do not resume at the failed Metal trial or count the old warmup as a measured
sample. The dedicated VM was stopped after confirming no containers or trial workers
remained; the original default VM is still running. Both worker binaries and
the verifier seal were rechecked, as were 44 raw acceptance artifacts and nine
benchmark artifacts. No installation is required. See SETUP.md for the thermal helper build
and benchmark/collector commands; BENCHMARKS.md and VALIDATION.md record the
attempt and exact timing limits.

## Previously completed workload/stability milestone

The stability changes start at miner
`b871d7ed0f0de83d99cc3e5674bfba95c3f04d7f` and were committed in
`244a3a1eb69e2ed611fa1b52e00abfaae31b32f7`. Worker binaries, production worker/protocol code,
engine, dependency pins and root lock remain unchanged. Changed code is
`crates/zkminer-prover/tests/macos_worker.rs`,
`scripts/macos/test-worker.py`, new `scripts/macos/summarize-worker.py` and
its runnable `test-worker-summary-controls.py` guards.

Four source-bound runs passed: SHA/admission recovery
`71387ac25ac74d4395278a3459680152`; five segments
`3c0b5d8179934bafb482174615a05d22`; minimum 4 KiB staging
`d93743621b584487accb84c7df1833c6`; twenty requests
`28bdd9ca009a4f6eacda4e11df402aed`. Total: **23 complete Groth16 proofs,
69 negative-verifier rejections, 8 actual tests, zero ignored or failed tests**.
All outputs use the unchanged pinned verifier. The logical admission refusal
is separately recorded and never counted as a completed proof.

Twenty consecutive independently verified requests passed in 3341.356s (55.69 minutes),
with zero failed requests. Normal memory-driven recycling used 16 proving processes; the
longest same-PID streak was 2. Proof seconds ranged 157.222–166.442, median 164.845.
Sampled request peaks ranged 1,970,487,296–1,996,292,096 bytes. All twenty outputs
passed 60 tamper rejections and recorded 3,880 completed GPU commands.

The admission test holds real production accounting reservations without
allocating their pages: 107 GiB held, typed attempted=false refusal before any
proof/helper, ledger released to zero, then a complete SHA proof. The five-
segment proof records 1,269 GPU completions; the 4 KiB proof records 16,680
completions with exactly 4,096 bytes peak staging. Physical host-memory pressure,
RSS-limit kills and sustained same-process memory behavior remain untested.

Aggregate evidence: `docs/macos/evidence/worker-stability-validation.json` and
`worker-stability-measurements.csv`; raw run artifacts are under
`target/macos/worker-runs/<id>`. The collector rechecks source-bound acceptance,
raw proof/artifact hashes and resource observations. Nonpositive outer RSS
readings are preserved as unavailable. One-ULP JSON timing round-trip differences
are explicitly recorded; cryptographic fields and counters compare exactly.
No raw proof or measurement file was rewritten to make collection pass.
All 12 collector guards passed separately and count as zero proving tests;
results and prior collection refusals are in `worker-stability-controls.json`.

The dedicated VM is stopped after checking no containers remained. No privileged
installation, push, wallet access, job claim or transaction occurred.

## Repositories and refs

Implementation branch: `codex/hemi-macos-metal` in both implementation worktrees.

* Miner `/Users/dbur/dev/zk-miner`, remote `https://github.com/hemilabs/zk-miner.git`:
  the native-worker checkpoint was based on `3b2720110c1721a53d19c09a6ff2a16235aeb9be`;
  its sealed worker builds remain unchanged for this acceptance-only continuation.
  The stability checkpoint is `244a3a1eb69e2ed611fa1b52e00abfaae31b32f7`; benchmark instrumentation is `461965e`. Resolve the latest local evidence commit with `git log -1`.
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
revision changed. The previous native-worker milestone added four existing dependency edges:
metal, risc0-zkp, serde_json and sha2. This continuation leaves Cargo.lock unchanged. No broad updates/upstream import.
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

Previously committed production implementation: `crates/zkminer-prove-risc0/{Cargo.toml,src/main.rs,
src/macos.rs,src/macos_metal.rs}`, four root lock dependency edges,
`scripts/macos/{build-worker.py,worker_provenance.py,test-worker.py}` and
`crates/zkminer-prover/tests/macos_worker.rs`. Build seals bind worker sources,
local engine revision/diff, actual dependency graph/lock and executable. The
acceptance runner additionally binds the real WorkerPool test and unchanged
verifier. Optional atomic per-request JSON preserves stage evidence before
wrapping, including interrupted requests, without changing positional IPC.

## Earlier native-worker evidence

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
The current separate miner diff is `target/macos/benchmark-comparison-miner.patch`
from `a3342fb`; `target/macos/benchmark-comparison-engine.patch` is empty. The
earlier tooling diff remains `target/macos/benchmark-miner.patch` from `244a3a1`.
The earlier stability diff remains `target/macos/worker-stability-miner.patch`. Previous engine patches remain under
docs/macos/evidence. Source changes require rebuilding sealed artifacts.

Dedicated profile `hemi-macos-cpu` / context `colima-hemi-macos-cpu` is
stopped after acceptance and confirming no containers remained. It has 6 CPUs,
32 GiB VM RAM,80 GiB disk; each helper is
capped at 24 GiB/6 CPUs with network disabled. Original Colima is untouched.
Pinned helper image digest:
`sha256:a4f80ce2e0b8e2bb7637a93c37136a6776ac00ec843a3fdf1c67b1d5ffea64ee`.

## Next work and limits

Next concrete step: extend controlled comparisons to the already validated hash
and multi-segment workloads. The Fibonacci campaign is complete; its earlier
admission failure remains archived separately. Benchmark/calibration IPC
acceptance remains separate. The completed twenty-request run retained normal
recycling; longer same-process and physical-memory-pressure recovery coverage
remain open. Earlier correctness-run timings remain separate from the completed
fixed-setting Fibonacci benchmark above.

GPU hardware-fault recovery, driver allocation peaks/growth trends and full
Linux/CUDA/ROCm runtime regression remain unrun. Optional offline Linux CUDA
metadata failed on uncached `approx v0.5.1`; it is not a passed build. Pinned
`lift_rv32im_v2_13.zkr` remains unavailable, so earlier supported multi-segment
coverage uses po2 16. Do not regenerate keys or substitute artifacts. Exact
contract-adapter/deployed verifier acceptance and network mining remain separate,
unrun and unauthorized. No speedup or production-readiness claim is made.
