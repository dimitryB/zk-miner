# macOS proving measurements

## WP1 paired gate and hashing re-baseline — 2026-11-04

The sealed WP0 worker and the opt-in evalfast worker each completed one
excluded warmup and five alternating, counterbalanced measured proofs per
fixture. Brave was closed before the campaign and absent in the before/after
snapshots for both fixtures. Both arms used po2 20, 18 Rayon and 18 Go threads,
the default 16 GiB host reserve, the same helper and unchanged external
verifier. There were no admission waits or failed proofs. All 24 proofs
verified and all 72 tamper checks rejected; pressure stayed normal.

| Complete proof plus verifier, median of five | WP0 archive | Evalfast | Saving |
| --- | ---: | ---: | ---: |
| Arithmetic 10M | 118.554 s | 78.688 s | 39.867 s |
| Hashing 10M | 119.566 s | 80.779 s | 38.787 s |

Arithmetic ranges were 117.103–118.952 s (WP0) and 78.403–78.765 s
(evalfast); within-pair ratios were 1.5005, 1.4867, 1.5121, 1.5079, and
1.5117. Hashing ranges were 119.149–120.065 s (WP0) and 80.576–81.717 s
(evalfast); within-pair ratios were 1.4750, 1.4743, 1.4632, 1.4841, and
1.4867. Median evalfast split-kernel GPU time per segment was 1.692 s for
arithmetic and 1.633 s for hashing. Both complete-latency savings exceed the
25 s WP1 gate, and both segment medians are below 2.0 s.

The **119.566 s WP0 hashing median** is the clean re-baseline under the same
conditions as evalfast. The earlier 130.6 s hashing result with Brave open and
a 12 GiB reserve is historical and is not used for this comparison. Raw rows,
host snapshots, source and binary hashes, and all five pair ratios are in
[paired evidence](evidence/phase2-wp1-paired.json) and
[paired CSV](evidence/phase2-wp1-paired.csv). The sealed WP0 proof sidecar lacks
per-proof GPU kernel timestamps; its separate WP0 profile remains available.

Engine `5acb4b8721b54ba2621d89f5f8d677cd5eca29d2` makes evalfast the
default. `RISC0_RV32IM_TAIL_SPLIT=1` selects the tail-split/256 fallback. A
rebuilt sealed worker (`6896db28…`) passed a complete po2-14 CPU differential
and external verification on each route. Its 20-request lifecycle passed all
functional controls and verified 21 proofs. Idle footprint fell 10.60%
from request 1 to request 20, with a 3.68% settled range and no upward leak.
This passes the revised one-sided memory gate, so WP1 is closed.
See [default-path evidence](evidence/phase2-wp1-default.json).

## Reopened WP1 evalfast phase split — 2026-11-04

The in-tree evalfast output through `remat_lu.py` is regenerated and translated
to MSL. The three-kernel path is selected by `RISC0_RV32IM_EVALFAST=1`; the
tail-split/256 evaluator remains the default fallback. The isolated po2-20
CPU differential matched the fallback output hash, with 1.694 s repeat GPU
time at 256 threads (1.741 s at 512 threads). Complete po2-14, po2-18 and
po2-20 proofs passed real-witness CPU differentials and external verification.
The same phase split without evalfast's `EF_WIDE` arithmetic took 4.766 s
per repeat dispatch, so the measured gain cannot be attributed to removing
LogUp work alone.

| Verified 10M diagnostic, 12 GiB host reserve | Tail-split/256 | Evalfast phase split | Saving |
| --- | ---: | ---: | ---: |
| Arithmetic constraint wall | 47.813 s | 16.899 s | 30.914 s |
| Arithmetic complete proof and verifier | 118.593 s | 86.494 s | 32.099 s |
| Hashing constraint wall | 51.059 s | 16.948 s | 34.111 s |
| Hashing complete proof and verifier | 124.648 s | 88.577 s | 36.072 s |

These are single diagnostic proofs per fixture, with ten po2-20 segments each.
The evalfast split kernel's median GPU time per segment was 1.624 s for
arithmetic and 1.623 s for hashing; each segment stayed below 1.667 s.
The 20-request lifecycle passed with 21 verified proofs including recovery,
63 tamper rejections, cancellation, a calibrated helper watchdog, restart and
shutdown. Nine workload-limit controls also passed. The first-to-last idle
footprint fell 12.39%; after settling, the range was 0.25%. Pressure stayed
normal. A separate po2-14 proof passed with
the default tail-split/256 fallback and real-witness CPU differential. The
first lifecycle attempt completed 20 steady proofs but its 10-second watchdog
captured an incomplete sidecar; the calibrated 20-second rerun passed. Raw run
paths and source hashes are in [evalfast evidence](evidence/phase2-wp1-evalfast.json).

## Reopened WP1 LogUp timing diagnosis — 2026-11-04

An isolated timing-only shader removed the LogUp continuation at the
`rv32im_v2_3` `x1102` boundary. It was never used for proving and its output
was not compared with the CPU polynomial. The unchanged tail-split/256 shader
was the reference. Five alternating pairs at po2 20 used Metal command-buffer
GPU timestamps; the order was reversed for the final two pairs.

| Repeat dispatch GPU time | Tail-split/256 reference | Main only |
| --- | ---: | ---: |
| Median of five | 4.969 s | 4.470 s |

The median ratio was **1.112×** (0.499 s saved); pair ratios were
1.070–1.115×. This is a percent-scale change, not the multiples-scale signal
proposed for the live-state diagnosis. Input and output guards held, inputs
were unchanged, and each shader was deterministic on repeat dispatch. The two
shaders intentionally produced different outputs. See
[diagnostic evidence](evidence/phase2-wp1-logup-probe.json) and
[samples](evidence/phase2-wp1-logup-probe.csv).

The separate fold-only cleanup changed the worker binary after the WP1 timing
campaign. Its validation is in [cleanup evidence](evidence/phase2-fold-cleanup.json);
the performance measurements below refer to the preceding WP1 binary.

## Phase 2 WP1 bounded evaluator trial — 2026-11-04

Engine `afd4342f90160c60829818dca953bcfe952a5639` splits all 20 generated
RV32IM chain functions at low-live-value points and uses 256-thread groups.
The worker SHA-256 is
`9b0f07321888abf9292c466fe4d3dee93d499de3dca8b03cec491020d66bc2c1`;
the original external verifier is unchanged. Synthetic CPU comparisons passed
at po2 4/8/14/18/20 with output reuse, and real-witness differentials plus
complete verification passed at po2 14/18/20. All three tamper checks passed
for each complete proof.

| Po2-20 synthetic split shader | 32 threads | 64 threads | 128 threads | 256 threads |
| --- | ---: | ---: | ---: | ---: |
| Repeated dispatch | 5.087 s | 5.079 s | 5.033 s | **4.866 s** |

The in-tree fast Fp and FpExt reduction forms were ported as optional shader
variants and passed the synthetic differential, but their repeated po2-20
dispatches took 5.205 and 5.373 s at 256 threads. The split-only form is
retained. Metal reported 32-thread execution width, 1024 maximum threads per
group and zero static threadgroup bytes; register/spill counts were unavailable
through the runtime API.

| Single 10M diagnostic, 12 GiB host reserve | WP0 instrumented worker | WP1 split worker | Change |
| --- | ---: | ---: | ---: |
| Arithmetic constraint stage wall | 57.295 s | 47.813 s | −9.482 s |
| Arithmetic constraint GPU execution | 55.902 s | 46.495 s | −9.407 s |
| Arithmetic complete proof and external verification | 128.053 s | 118.593 s | −9.460 s |
| Hashing constraint stage wall | 57.207 s | 51.059 s | −6.148 s |
| Hashing constraint GPU execution | 55.812 s | 49.812 s | −6.000 s |
| Hashing complete proof and external verification | 129.157 s | 124.648 s | −4.509 s |

These are one diagnostic proof per worker and fixture, not paired warm medians.
WP1 constraint wall time is **4.78 s per arithmetic segment** and **5.11 s per
hashing segment**, above the ≤2.0 s gate. The five-pair ≥25 s whole-proof gate
was therefore not claimed. Both WP1 proofs verified independently with six
negative checks; group peaks were 15.417/15.429 GiB and sampled pressure was
normal. The generated main/LogUp phase split and scratch-buffer lazy reduction
remain future work. See [WP1 evidence](evidence/phase2-wp1-iteration.json).

The checkpoint lifecycle passed 20 steady proofs plus one recovery proof and
63 tamper rejections. Idle worker footprint changed by −0.42% from the first
to twentieth steady request; all sampled pressure readings were normal.
Cancellation, worker timeout, restart, shutdown, and nine limit controls
passed. Helper timeout was not exercised in this run.

## Phase 2 starting baseline — 2026-11-04

The Phase 1 source commits were rebuilt and sealed as a new Phase 2 baseline:
miner `9bc7466eaf6a4b48d8655d39f69cc89fa40f0298`, engine
`dcbe9f4b9140a48d4792ffcd3a3eba19897be175`, worker SHA-256
`1c70d2ef9eb3dddd94ded3588eedf7c93f3cca176a5b4a9d660ec570637938d8`.
The unchanged external verifier SHA-256 is
`2bcdcb7f50e2303545898962d23bdc6cba9512038383a93c975f95b89cf8225c`.
The archive is `target/macos/phase2/baseline/`. These measurements replace the
earlier 117.2 s and 484.7 s observations for Phase 2 comparisons.

| Fixture | Measured proofs | Complete median [minimum–maximum] | Segment/execution median | Recursion median | Identity/wrap median | Guarded group peak | Host reserve |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Arithmetic 10M | 5, after one warmup | 119.568 [117.595–119.607] s | 89.394 s | 11.438 s | 17.726 s | 19.435 GiB | 16 GiB |
| Hashing 10M | 5, after one warmup | 130.641 [128.620–131.655] s | 98.718 s | 11.967 s | 18.362 s | 19.502 GiB | 12 GiB |
| Arithmetic 46M | 1 | 501.037 s | 411.597 s | 63.380 s | 24.621 s | 15.402 GiB | 12 GiB |

All 13 dispatched proofs passed the unchanged verifier and 39 negative checks.
The first hashing campaign at the default 16 GiB reserve was refused before
dispatch when conservative available memory was about 40 GiB; the failed
attempt is retained in the WP0 evidence. The new hashing and stretch campaigns
used the documented 12 GiB local override. Their settings differ from the
arithmetic campaign and must be matched within each later paired comparison.
Host snapshots during hashing and stretch showed Brave, Codex, Claude, and
Colima running and reported fair thermal state. The snapshots are instantaneous,
not a per-proof thermal trace. GPU kernel timing was added after this sealed
baseline and is not attributed to its pre-instrumentation binary. See
[baseline JSON](evidence/phase2-wp0-baseline.json) and
[proof CSV](evidence/phase2-wp0-baseline.csv).

The WP0 instrumented worker, byte-identical before and after engine commit
`09c2f4d2836cbcd88cd29e65b6a6bdd8db262416`, produced two additional
single-proof diagnostic profiles. Every Metal command buffer records device
GPU time, host dispatch wall time, pipeline creation time and pipeline limits;
the worker sidecar aggregates per kernel. Both proofs passed the unchanged
verifier and six negative checks. The synchronous path has no segment CPU/GPU
overlap.

| Fixture and stage | Stage wall | GPU execution | Dispatch wall |
| --- | ---: | ---: | ---: |
| Arithmetic constraints | 57.295 s | 55.902 s | 57.294 s |
| Arithmetic transforms | 17.693 s | 15.450 s | 17.383 s |
| Arithmetic Merkle | 3.873 s | 3.607 s | 4.060 s |
| Arithmetic recursion | 11.716 s | 7.572 s | 9.504 s |
| Hashing constraints | 57.207 s | 55.812 s | 57.206 s |
| Hashing transforms | 17.622 s | 15.510 s | 17.418 s |
| Hashing Merkle | 3.855 s | 3.598 s | 4.060 s |
| Hashing recursion | 11.562 s | 7.446 s | 9.406 s |

Constraint GPU time is within 3% of its wall timer. Transform GPU time is
about 12–13% below its wall timer, just outside WP0's 10% GPU-only
reconciliation target; the measured host dispatch cost accounts for most of
that difference across 1,873 transform commands per 10M proof. Merkle command
times include some Poseidon calls outside the narrower Merkle timer, so their
dispatch-wall sum slightly exceeds that timer. Recursion includes CPU witness
and accumulation. See [GPU profile JSON](evidence/phase2-wp0-gpu-profile.json)
and [stage CSV](evidence/phase2-wp0-gpu-profile.csv); these are diagnostic
observations, not warm medians.

The sealed instrumented worker also passed one po2-14 real-witness CPU
differential proof, nine workload-limit rejections, and a 20-request lifecycle.
The lifecycle verified 21 proofs including a post-restart proof and passed 63
negative checks, helper cancellation, expected helper-timeout failure, worker
watchdog, recovery and shutdown. First/last settled worker footprints were
4.215/4.404 GiB, **4.49% drift**, below the 5% gate; readings varied between
4.207 and 4.404 GiB rather than rising steadily. Native and CPU-only worker
unit builds each passed five tests. Only the Apple Silicon target is installed
in the pinned local Rust toolchain, so a Linux feature compile was not run.
See [WP0 validation](evidence/phase2-wp0-validation.json) and
[lifecycle CSV](evidence/phase2-wp0-lifecycle.csv).

## Refreshed resident worker — warm gates completed 2026-11-03

The predeclared five-pair arithmetic comparison passed: **3.645× median
speedup**, with **5/5 faster pairs**. Metric: successful WorkerPool call
plus the unchanged independent verifier. Each mode used one persistent worker,
one excluded warm-up and five measured proofs; request order alternated. All
twelve proofs and 36 negative checks passed. The fresh run had no admission
waits or proof failures. Both cases used identical ELF, input, 10,003,116 guest
cycles, ten po2-20 segments, and 18 Rayon / 18 Go threads.

| Complete seconds, n=5 | Median | Minimum–maximum |
|---|---:|---:|
| Committed step-5 baseline | 427.192 | 420.829–432.260 |
| Refreshed resident Metal | 117.192 | 117.177–118.000 |
| Resident Metal hashing | 120.800 | 120.173–124.663 |

Both usefulness medians are below 600 seconds. The separate 46M arithmetic
observation verified in 484.740 seconds with 46 segments and a 15.401 GiB
guarded group peak. Largest observed review peak across warmed proof groups is
19.783 GiB. Default admission/reserve and Colima state were unchanged. The
fresh comparison began after Brave closed; earlier partial timings are retained
separately and are not pooled. This is a fixed-fixture, whole-stack comparison
including parent changes, not isolated Metal attribution or statistical significance.

See [paired JSON](evidence/rv32im-five-pair-validation.json),
[all paired proof rows](evidence/rv32im-five-pair-proofs.csv),
[warm hashing](evidence/rv32im-warm-hash-validation.json),
[stretch](evidence/rv32im-current-tip-validation.json), and
[current Metal profiles](evidence/rv32im-metal-current-profile.json).
The [review publication record](evidence/rv32im-review-publication.json) binds
these measurements to this miner commit and engine checkpoint `dcbe9f4b9`.

## Resident RV32IM worker validated 2026-11-02

The candidate, uncommitted at measurement time, passed 28 complete proofs and 84 negative
verifier checks. Twenty consecutive tiny requests completed on one worker,
then cancellation, worker watchdog, restart/recovery proof and shutdown passed.
Three complete proofs also compared all constraints with CPU on real witnesses.
Explicit CPU selection, boundary/tiny-tail and workload-limit controls pass;
CPU-only and resident worker feature builds each pass five unit tests.

| Single 10M observation | Guest cycles | Segments | Complete request including external verification |
|---|---:|---:|---:|
| Arithmetic | 10,003,116 | 10 | 126.680 s |
| Hashing | 10,003,440 | 10 | 130.646 s |

The largest completed-proof guarded worker/helper footprint was 16.305 GiB.
For the twenty tiny requests, idle worker footprint ranged 3.859–3.948 GiB;
first/last were 3.873/3.860 GiB. Their median request latency was 19.212 seconds,
with range 19.133–26.185 seconds including the first request. This is stability
evidence on tiny jobs, not a 10M warm median or controlled speedup comparison.
At this original checkpoint the 46M Metal stretch and whole-job 2× gate were
unmeasured; the subsequent results are recorded above.

See [integration report](RV32IM_METAL_WORKER.md),
[validation JSON](evidence/rv32im-worker-validation.json) and
[proof CSV](evidence/rv32im-worker-proofs.csv). Source and binary seals cover
the measured engine changes; the independent verifier remains byte-identical
to its original pinned build. Three preliminary proofs are excluded after the
writable-FFI-pointer correction. No VM state changed and no test process remains.

## Bounded RV32IM evaluator feasibility completed 2026-11-02

Two complete po2-20 synthetic evaluator domains independently matched the CPU
oracle, including output reset and reuse. Four GPU commands each produced
16,777,216 matching field elements. Seed 19 took 5.847/5.853 seconds on Metal
(first/reuse) versus 19.258 seconds on CPU; seed 23 took 5.845/5.848 versus
19.469 seconds. Peak charged footprint was 13.070 GiB. These are isolated,
fixed-order observations with synthetic field matrices, not whole-proof timings
or controlled warm medians. Cold library/pipeline compilation of the final
shader took 9.790/40.159 seconds; later processes used the driver cache.

Seven final-candidate cases at po2 4/8/14/18/20 passed fourteen GPU commands.
Five earlier failures remain recorded: missing operators, two maximum-threadgroup
failures (one confirmed out-of-memory), and two original normalization mismatches.
Retain thirty-two-thread groups and the independently checked four-entry
normalization table. See [prototype](RV32IM_METAL_PROTOTYPE.md) and
[bound evidence](evidence/rv32im-evaluator-prototype.json). All work is uncommitted;
engine and verified step-5 worker inputs remain unchanged.

## RV32IM port assessment completed

Step-5 checkpoints are miner `4cbfb3e` and engine `0515e0bde`. Two additional
10M verbose diagnostics, on the exact same worker binary, independently verified
in 416.158 seconds (arithmetic) and 425.240 seconds (hashing), with six negative
checks. These are attribution runs, not a new speedup comparison or warm median.

The exclusive segment breakdown is approximately 51% constraint evaluation,
31% Merkle commitments, 11% transforms, 2–3% witness/accumulation and 4.5% other
protocol work. The original usefulness goal already passes. A full RV32IM port
is a separate scalability investment; first de-risk the larger evaluator and
shared-buffer integration. At assessment time no RV32IM Metal speedup had been
measured and no worker integration had started. The subsequent isolated
prototype is recorded above. See the [assessment](RV32IM_METAL_ASSESSMENT.md),
[bound JSON](evidence/rv32im-metal-assessment.json) and
[exclusive profile CSV](evidence/rv32im-metal-profile.csv).

## Poseidon2 lift/join recursion Metal completed 2026-11-01

The opt-in native worker now proves Poseidon2 lift/join using shared Metal
buffers for transforms, commitment hashing and constraint evaluation. Witness
generation and accumulation run the pinned CPU routines directly on those
buffers. RV32IM segment proving, identity-p254 and native Groth16 remain CPU.
Every GPU command completes before CPU access, and temporary Objective-C command
objects drain after each dispatch. Control buffers/groups retain the engine's
bounded caches. The evaluator inserts the current CUDA polynomial body verbatim
into the Metal address-space wrapper using the retained field headers.

A five-pair warm-worker comparison passed the frozen performance criterion:
**86.921 → 77.772 seconds** median complete latency, **10.5% less time** (1.118×
throughput), with all five pairs faster. The criterion, recorded before any
case started, required at least 1.05× median speedup and four faster pairs.
Both modes used the same sealed worker, helper, 18 Rayon / 18 Go threads,
fixture and segment cap. One first-process warmup per mode was excluded; request
order alternated each round. Both workers remained resident, with one active
proof at a time. All twelve proofs, including warmups, verified independently
and passed 36 negative checks. No measured row was removed or retried.

Fibonacci(150,000) executes 1,503,116 guest / 1,572,864 padded cycles in two
segments at the production po2-20 cap. The comparison is specific to this fixed
fixture; it does not establish a speedup for every job or statistical significance.

| Seconds, median [minimum–maximum], n=5 per mode | CPU recursion | Metal recursion |
|---|---:|---:|
| Execution + segment proving | 58.380 [58.204–58.434] | 58.363 [57.932–58.691] |
| Recursion | 10.350 [10.334–10.415] | 1.824 [1.817–1.840] |
| Identity + native wrapping | 17.332 [17.280–17.385] | 17.322 [17.286–17.424] |
| Worker body | 86.091 [85.958–86.221] | 77.545 [77.135–77.834] |
| Complete call + independent verification | 86.921 [86.903–86.952] | 77.772 [77.746–78.806] |

The two 10M fixtures also passed as separate first-process observations:

| Fixture | Guest / padded cycles | Segments | Worker body | Complete call + independent verification |
|---|---:|---:|---:|---:|
| Fibonacci(1,000,000) | 10,003,116 / 10,485,760 | 10 | 417.286 s | 418.323 s |
| SHA-256 chain, 2,262 hashes | 10,003,440 / 10,485,760 | 10 | 420.833 s | 421.236 s |

Each recorded 19 successful Metal constraint checks (ten lifts and nine joins).
These are single observations, not new five-sample usefulness medians or
cold-boot measurements. The step-4 CPU dataset below remains the established
10M acceptance set. The 46M CPU stretch was not repeated with Metal.

The final campaign passed nine input controls, tiny-tail and segment-boundary
proofs, the twelve comparison proofs, two 10M proofs, twenty consecutive
requests on one worker, helper cancellation, the ten-second watchdog, a verified
restart proof and shutdown: **37 complete proofs and 111 negative checks**.
The twenty-request worker's settled footprint ranged from 3.849 to 4.008 GiB
(first 3.902, last 3.851). The largest sampled owned-group footprint was
17.279 GiB during Metal startup; the native CPU comparison peaked at 15.112 GiB.
Conservative host availability stayed above 22.0 GiB and sampled pressure was
normal. Worker/helper maxima need not occur together; footprints and peaks are
sampled observations, not hard allocation bounds. Thermal nominal can mean
unavailable; no energy or frequency claim is made.

After timing, the initial Metal admission budget was raised from 16 to **20 GiB**
to exceed that startup peak; native CPU keeps 16 GiB. The existing measured safety
factor, host reserve and 32-GiB group watchdog remain. A separate final check
held 93 GiB of logical reservations, received a typed refusal before dispatch,
released reservations to zero and verified a recovery proof with three negative
checks. The represented reservation was not physically allocated. This final
supervisor change leaves the sealed proving worker unchanged; the timing
campaign's original acceptance binaries and source manifests are retained.

Validation also passed 18 Metal primitive tests, the evaluator CPU differential
at po2 4/7/10 with offsets/sentinels, four worker startup tests, 62 benchmark
unit tests, 12 memory tests and CLI compilation. An isolated, source-bound test
reused actual shared buffers across CPU writes and GPU hashing, covering empty
inputs, zero-output rejection, nonzero offsets and grid boundaries: 36 GPU
commands matched CPU results. Linux ARM64 CPU checks passed through both
recursion and zkVM Metal feature paths on Rust 1.93.1; no Linux GPU runtime was
tested. The dedicated VM was stopped before native timing.

The preliminary campaign `ac530c478a9047daa7a424c43b76de8d` was deliberately
stopped after review found the missing per-dispatch autorelease pool. Its seven
verified artifacts, started request, pending slots, `replan.json`, binary and
source archive are retained separately. A separate three-proof pilot is also
retained. Neither contributes to the final performance result. Pre-proof setup
rejections and the operator-interrupted Linux check remain alongside corrected
runs. No cryptographic proof failure was retried.

Evidence: [validation JSON](evidence/recursion-metal-validation.json),
[ten measured rows](evidence/recursion-metal-comparison.csv) and
[engine patch](evidence/recursion-metal-engine.patch).
Raw final campaign:
`target/macos/native-wrap/workload-campaigns/ed853b04d39243a4b6da292774bc34f6`.
The exact worker, shader and acceptance executables are preserved under
`target/macos/recursion-metal-final-build`. Steps 1–4 are committed in miner
`09a6910` and engine `98d940d4b`. Step-5 measurements were captured before its
checkpoint commits; the owner subsequently authorized committing step 5 and
assessing RV32IM Metal. The step-5 engine checkpoint is `0515e0bde`. Nothing
has been pushed. Poseidon254 Metal and full RV32IM acceleration remain
separate milestones.

## Workload expansion completed 2026-11-01

The native worker defaults to po2 20 and a configurable 64-Mi guest-cycle
ceiling. Legacy Mac wrapping retains its po2-18 default. Final-segment padding
has a po2-14 floor for supported proving caps, with unchanged claims/journals
in 51 before/after executions and real tiny-tail proofs accepted by the unchanged
verifier. Shared executor tests passed on macOS and Linux ARM64.

The five measured warm arithmetic proofs have completed. Fibonacci(1,000,000)
executes 10,003,116 guest / 10,485,760 total cycles in ten segments. Complete
latency, including independent verification, was a median **483.927 seconds**
[470.785–502.439], n=5, with one excluded first-process warmup. The warmup was
493.118 seconds of worker body; these are not cold-boot measurements.

The hashing set also passed five warm measurements. Its 2,262 hashes execute
10,003,440 guest / 10,485,760 total cycles in ten segments. Complete latency
was a median **476.853 seconds** [473.839–483.927], n=5, with one excluded
first-process warmup (494.056 seconds of worker body). All twelve complete
proofs across both sets, including warmups, passed the unchanged verifier and
36 negative checks; there were no retries or excluded measured rows.

| Seconds, median [minimum–maximum], n=5 each | Arithmetic | Hashing |
|---|---:|---:|
| Execution + segment proving | 396.674 [383.056–417.827] | 390.767 [386.540–392.632] |
| Recursion | 67.897 [65.676–72.628] | 66.900 [65.832–77.850] |
| Identity + wrapping | 17.798 [17.267–20.526] | 17.453 [17.277–19.221] |
| Worker body | 483.075 [469.804–501.893] | 476.604 [473.160–483.748] |
| Complete call + external verification | 483.927 [470.785–502.439] | 476.853 [473.839–483.927] |

The prior po2-18 sizing screen completed one verified 10M proof in 731.067
seconds of worker body: 403.655 segment, 302.261 recursion and 24.690 identity/
wrap. Its remaining planned samples were intentionally cancelled to test larger
segments; the next started request and failed/interrupted campaign result are
retained with `replan.json`. This is a single first-process screen, not a paired
speedup campaign. The final campaign fixes po2 20 and retains every sample.

The timing model was frozen from these ten warm measurements before observing
the stretch result: 22,691.360 padded cycles per STARK second plus 17.518 seconds
fixed identity/wrap. It predicts 2,108.537 seconds of body time for the 46M
arithmetic fixture, excluding fresh-process preparation. The forecast is saved
in the campaign's `frozen-prediction.json`.

The arithmetic stretch passed: 46,003,116 guest / 47,448,064 total cycles in
46 segments, with 2,139.636 seconds of worker body and 2,139.840 seconds including
the supervisor call and independent verification. This one fresh-process sample
is 1.5% slower than the frozen warm-profile body forecast. Its segment phase was
1,790.132 seconds, recursion 324.378 seconds, and identity/wrapping about 25 seconds.
Peak sampled group footprint was 11.6 GiB; pressure remained normal. This is
separate from the ten-minute target and is not a repeated stretch benchmark.

The twenty-request lifecycle gate passed: twenty consecutive successful requests
on one supervised worker, observed helper cancellation, the ten-second worker
watchdog, a separately verified restart proof and shutdown. The completed
campaign contains **36 verified full proofs and 108 negative verifier checks**,
plus nine input-limit controls. No failed proof was retried and no measured row
was removed. Both 10M medians meet the agreed ten-minute target. Across the
campaign, sampled simultaneous group footprint peaked at 12.6 GiB and memory
pressure stayed normal; this is a sampled bound, not an exact allocation peak.

Evidence: [validation JSON](evidence/native-workloads-validation.json),
[ten measured rows](evidence/native-workloads-comparison.csv),
[exported CPU calibration](evidence/native-workloads-calibration.json), and
[consolidated engine patch](evidence/native-workloads-engine.patch).
A separate final supervisor check loaded the exported 12.6-GiB group peak,
producing a 25.2-GiB admission budget with the existing safety factor. Holding
87 GiB of logical reservations produced a typed refusal before dispatch;
releasing them to zero allowed a verified recovery proof. Fibonacci(50,000)
executed 503,116 guest / 589,824 padded cycles in two segments at requested cap
19. That proof and its three negative checks passed on final supervisor sources
with the same sealed worker/verifier. Its 60.126-second worker body is a single
correctness observation. No represented reservation was physically allocated.

The preliminary screen's three verified proofs and nine negative checks are
retained separately from the completed campaign. Hashing at 46M was execution
sized only. No Linux GPU proving or marketplace job acceptance was tested.

Raw final campaign:
`target/macos/native-wrap/workload-campaigns/2edca085145446d5a3e89270d6087a7a`.
The retained screen is `a90a508b35fc4e9d926e9582781284bf` under the same directory.
All changes remain uncommitted/unpushed.

## Native thread tuning and admission completed 2026-11-01

The native defaults are now **18 Rayon threads and Go concurrency 18**. A
counterbalanced five-pair comparison on persistent workers measured a 2.05×
worker-body speedup over 6/6. Each setting had one excluded warmup; all twelve
complete proofs passed the unchanged independent verifier and all 36 tamper
checks. Both settings used the same sealed native worker and runtime bundle,
with one active proof at a time and no Docker helper.

| Seconds: median [minimum–maximum], n=5 per setting | Rayon 6 / Go 6 | Rayon 18 / Go 18 |
|---|---:|---:|
| Execution + segment proving | 2.648 [2.620–2.721] | 1.199 [1.178–1.252] |
| Recursion | 6.384 [6.262–6.502] | 3.455 [3.397–3.495] |
| Identity + native wrapping | 36.964 [35.829–37.359] | 17.672 [17.149–17.810] |
| Complete worker body | 46.069 [44.739–46.609] | 22.421 [21.794–22.504] |
| Supervisor prove call | 46.217 [45.208–47.210] | 23.126 [22.136–23.137] |

Worker-body timing includes final SDK verification; external verification and
three negative checks are recorded separately. Supervisor timing includes
admission and IPC. This is Fibonacci(20): 3,316 guest / 32,768 total cycles, one
actual po2-15 segment under a po2-18 cap, with the same program/input hashes as
the earlier controlled campaign. It does not establish the 10M-cycle target.
Round order alternated, with no failed proofs, retries or removed measured rows.
Recorded thermal state was nominal throughout; the platform probe can report
nominal when information is unavailable. Background host activity remains a
limitation; no frequency, energy or statistical-significance claim is made.

Separate screens preceded the final comparison:

| Screen | Setting | Measured samples | Median seconds |
|---|---|---:|---:|
| Complete worker, Go fixed at 6 | Rayon 6 | 3 | 45.427 |
| Complete worker, Go fixed at 6 | Rayon 12 | 3 | 30.599 |
| Complete worker, Go fixed at 6 | Rayon 18 | 3 | 25.714 |
| Native gnark on a fixed witness | Go 6 | 5 | 7.944 |
| Native gnark on a fixed witness | Go 12 | 5 | 5.268 |
| Native gnark on a fixed witness | Go 18 | 5 | 4.571 |

Each screen excluded one warmup per setting. The Rayon screen verified twelve
complete proofs. The Go screen verified eighteen fresh wraps of one retained
witness and includes child startup plus circuit/key/witness loading, while
excluding witness generation. It measures the actual host Go child, with
explicit `GOMAXPROCS`; it is not a VM-only measurement.

For the measured pairs, peak sampled worker/helper footprints were 3.936/7.181
GiB at 6/6 and 4.040/7.187 GiB at 18/18. Peak simultaneous combined observations
were 11.107/11.104 GiB; the independent one-second watchdog captured a slightly
higher 11.124-GiB peak at 18/18. The host's conservative available-memory estimate
stayed above 33 GiB and sampled pressure stayed normal. Worker and helper maxima
need not occur together. Host estimates are not added to process footprints.

Admission now uses a 16-GiB initial native budget and measured group-peak
calibration, with configurable host reserve (default 16 GiB). Warning pressure
blocks admission; critical/unknown pressure stops the owned group. The existing
conservative host-memory formula is retained. The group watchdog uses physical
footprint rather than worker-only RSS for its 32-GiB ceiling. These sampled
observations are not hard allocation limits or exact peaks.

The cache-warm admission run read 2,255,766,633 bytes of the native key through
a 4-MiB buffer, obtained a typed refusal before dispatch with real accounting
reservations held, released them to zero, and independently verified a recovery
proof. Its represented 97-GiB reservation was not physically allocated. Separate
unit coverage allocated/touched 64 MiB in an owned group and verified the guard
stopped that group under a 16-MiB test limit. No global OS pressure was forced;
live warning/critical pressure transitions were not induced.

Fresh-worker 18/18 correctness samples also passed: SHA-256 chain (47,348 guest /
131,072 total cycles) in 36.979 seconds, and five segments (126,077 guest /
294,912 total cycles) in 67.858 seconds. These single observations include cold
worker preparation and are separate from the paired warm-worker comparison.
The final 18/18 lifecycle run verified three consecutive proofs, cancelled a
running helper, exercised the ten-second worker watchdog, verified a restart
proof and checked shutdown. Across this milestone, 31 complete worker proofs,
18 fixed-witness rewraps and 147 negative checks passed, alongside 20 Rust unit
tests and eight invalid-thread startup checks. No proof failure was retried.
One queued follow-up failed before the test build because `env.sh` had not been
loaded and Cargo was missing; its failed result is retained separately from
the corrected run. The verifier binary and root dependency lock are unchanged.

Evidence: [native-tuning-validation.json](evidence/native-tuning-validation.json)
with [measured pairs CSV](evidence/native-tuning-comparison.csv), and
[consolidated engine patch](evidence/native-tuning-engine.patch). Raw screens:
`target/macos/native-wrap/worker-runs/4e3bfe41cddd47e3a7993e0673ad999b`
and `target/macos/native-wrap/go-tuning/b59364cac27e481ab9697bf5af867025`.
The final paired run is
`target/macos/native-wrap/worker-runs/1f51fe69dbf54339947f84a435200518`.
All changes remain uncommitted. The subsequent workload milestone above closes
the 10M-cycle and native 20-request gates.

## Native macOS helper completed 2026-11-01

The production worker now completes proofs using the shared Rust witness
evaluator and a Darwin ARM64 Go prover. Six complete supervised proofs and three
rewraps passed the unchanged verifier and 27 tamper checks. Witnesses for all
three fixtures match the pinned C++ generator byte for byte; the Rust writer now
emits the same WTNS version-2 header. The converter ran on macOS against the
original proving key, whose decompressed hash matches the rzup component.

| Complete worker body, seconds | Observed result |
|---|---:|
| Fibonacci, first proof | 59.714 |
| Fibonacci, next two proofs on the same worker | 44.712 / 45.763 |
| Fibonacci after cancellation and watchdog restart | 57.464 |
| SHA-256 chain | 64.290 |
| Five-segment fixture, 294,912 total cycles | 129.774 |

These are correctness-run observations at six Rayon threads and Go concurrency
six. Startup/preflight is separate; first-process proofs include witness-graph
preparation. Background compilation overlapped some acceptance work. This is
not the controlled five-pair performance campaign or the 10M-cycle acceptance.
The dedicated helper VM was stopped and acceptance blocked the Docker executable;
the original default VM was left untouched.

The WorkerPool suite observed a running native helper before cancellation,
checked its owned group disappeared, exercised a ten-second worker watchdog,
then independently verified a restart proof and clean shutdown. Separate tests
cancelled actual witness generation, rejected missing/corrupt bundles, and
exercised the real Go child's deadline and parent-death guards while it waited
on a FIFO. The macOS process/group tests passed (2 tests); the refactored Intel
wrapping feature compile-check passed. CUDA/ROCm GPU runtime tests remain unrun.

Sampled worker RSS peaks across the three consecutive Fibonacci proofs were
3.949, 4.051, and 4.051 GiB. Those observations exclude helper RSS; the live
watchdog conservatively includes owned group members under the existing 32-GiB
ceiling. Three requests establish only this bounded cache behavior. Native
20-request acceptance and physical-memory characterization remain ahead.

Evidence: [native-wrap-validation.json](evidence/native-wrap-validation.json)
and [complete engine diff](evidence/native-wrap-engine.patch). Reproduction and
native environment settings are in [SETUP.md](SETUP.md). All implementation
changes are uncommitted. Thread selection and admission results follow in the
newer milestone above.

## ARM64 wrapper feasibility completed 2026-11-01

Four new Groth16 wraps of retained, previously verified identity-p254 inputs
passed the unchanged-pin verifier and all 12 tamper checks. This campaign did
not rerun guest execution, segment proving, recursion, or the production worker.
Each archived reference seal was also reverified before its input was reused.

| Workload and layout, one sample each | Witness seconds | Gnark including loading seconds | Host wrap call seconds |
|---|---:|---:|---:|
| Fibonacci, serial stages | 4.668 | 10.006 | 15.677 |
| SHA-256 chain, serial stages | 4.656 | 9.821 | 14.553 |
| Five-segment fixture, serial stages | 4.704 | 9.796 | 14.580 |
| Fibonacci, original pipelined entrypoint | — | — | 11.312 |

The helper runs natively as Linux ARM64 in the existing six-vCPU, 32-GiB VM,
with six helper CPUs, `GOMAXPROCS=6`, a 24-GiB memory cap, and network disabled.
Runtime `uname` reported `aarch64`; both executables have ELF machine 183
(AArch64). This is native ARM64 wrapping inside a VM, not a native macOS helper.
The serial stage clock is VM wall time; host call timing is monotonic and
includes container launch/exit. Gnark includes constraint/key/witness loading.
The original entrypoint overlaps witness generation and loading through a FIFO.
These are feasibility samples, not a controlled five-pair speedup campaign or a
whole-job performance result. The memory cap is not a measured peak.

The successful build took 325.544 seconds after the Circom lockfile and missing
LFS-source issues were resolved; a final-script cached rebuild took 1.206 seconds.
The exact pinned circuit compiled to 5,676,573 non-linear constraints and
5,635,930 witness elements. The ARM64 converter used the original checksum-pinned
zkey. The only engine edit is `cargo install --locked` for pinned Circom.
Buildx 0.38.1 was downloaded and hash-verified into the ignored local tools
directory; no global installation was needed.

Measured image:
`sha256:75e360af5ce65deffeee8a8db0765e9fbbdbdb2d85bb5fe93d5eb6c43ca8e052`.
Evidence: [arm64-wrap-feasibility.json](evidence/arm64-wrap-feasibility.json)
and [engine diff](evidence/arm64-wrap-engine.patch). Raw build manifests,
artifact hashes, logs, and receipts are under `target/macos/arm64-wrap-builds`
and `target/macos/wrap-runs`. Next is the native macOS helper in
[IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md), before resource tuning.

## Controlled CPU and Metal Fibonacci comparison completed 2026-11-01

The recovered memory headroom allowed the full fixed-setting campaign to finish:
**12 genuine complete Groth16 proofs accepted by the unchanged verifier**, 36
negative-verifier rejections, 24 actual native tests passed, zero failed/ignored.
One complete CPU warmup and one Metal warmup are excluded from the five measured
pairs. No retries, selective exclusions or unattempted slots occurred in this
campaign. The earlier admission failure remains archived separately below.

Campaign `f57fcc0ff71d4d6391605cb761c0a1a6` ran 11:08:29–11:45:44 EST on
2026-11-01 (2234.736s, 37.246 minutes). Miner acceptance HEAD was
`a3342fb80ec2fcfba909f921e381c6636fefb731`; engine stayed
`0d5b41e8e44876c1e8b5a8a671aabc154e33106c`, descended from original pin
`f8dcbdf8124a3f1528146bb01d539c55545cbb3d`. The independently built verifier
remains at that original pin with its unchanged binary/source seal.

| Seconds: median [minimum–maximum], n=5 per mode | CPU | Metal |
|---|---:|---:|
| Execution + segment proving | 2.731 [2.723–2.843] | 2.791 [2.723–2.922] |
| Recursion/compression | 7.475 [7.425–7.553] | 7.464 [7.386–7.677] |
| Identity + emulated wrapping | 155.313 [154.425–155.916] | 155.547 [154.974–156.830] |
| Complete worker body | 165.582 [164.774–166.101] | 165.823 [165.439–166.970] |
| Supervisor prove call | 165.741 [165.504–166.457] | 166.361 [165.768–167.421] |

This fixture showed **no observed Metal speedup**. Metal's worker-body median
was 0.242s (0.146%) higher; the median of the five within-pair percentage changes
was +0.291%. These are different descriptive summaries, not significance claims.
The execution-plus-segment median was 0.060s higher with Metal. Sample ranges
overlap, and background activity prevents treating small differences as a clean
causal estimate. No larger-workload, throughput, energy or profitability claim.

| Measured pair | CPU worker seconds | Metal worker seconds | Metal change within pair |
|---|---:|---:|---:|
| 1 | 164.901 | 165.439 | +0.327% |
| 2 | 165.582 | 166.064 | +0.291% |
| 3 | 165.709 | 165.823 | +0.069% |
| 4 | 166.101 | 165.450 | -0.392% |
| 5 | 164.774 | 166.970 | +1.333% |

Identity plus emulated wrapping accounted for about 93.8% of the worker body.
Rounded production stderr reports median identity-p254 diagnostics of 33.826s
CPU / 34.148s Metal, and shrink/wrap of 121.498s / 121.333s (about 73% of the
body). Those labels include preparation/helper overhead and have 0.1ms rounding;
they are not newly isolated arithmetic timers. Final SDK verification medians
were 1.784ms / 1.762ms; the external unchanged verifier, including process launch,
positive acceptance and three negative checks, took 26.619ms / 35.602ms.

## Fixed settings and timing boundaries

Both modes used the **same sealed Metal-capable executable** (`2c3bf297…`) with
explicit `cpu` or forced `metal`. The separate CPU-only binary (`97588b915…`)
was preserved and its build seal rechecked; it was not substituted or rebuilt.
Guest packed-program hash `27e25b9504cdcc17419ba22a5025d79dc45167fbbc7d44d4910036d413684793`,
input hash `447e12701a0d03cf90a4ad7f02f1a045b35d284e26fe520440edb116d76bf700`,
image and expected journal match in every trial. Fibonacci(20) executed 3,316
user / 32,768 total cycles in one segment. The configured segment cap was po2 18;
the actual segment reported po2 15. Groth16 seal format, randomness and keys stayed
unchanged; randomized proof bytes were not required to match.

Settings: six Rayon threads, one active proof, 512 KiB logical Metal staging,
6 CPU / 32 GiB dedicated Colima VM, pinned local linux/amd64 QEMU helper capped
at 6 CPUs / 24 GiB with network disabled. Helper digest remains
`sha256:a4f80ce2e0b8e2bb7637a93c37136a6776ac00ec843a3fdf1c67b1d5ffea64ee`.
This is not native ARM Groth16 wrapping. The original default VM stayed running.

Order was CPU/Metal warmups, then five CPU→Metal pairs. It was fixed alternating
order, not randomized/counterbalanced or per-mode tuning. A fresh WorkerPool was
created each trial. Readiness, mode preflight (including Metal probe controls)
and valid/invalid execution checks preceded the timed proof. CPU proofs report
zero GPU proving commands; preflight controls are outside the proof-stage timers.
Each Metal proof records 194 completed segment Poseidon2 Merkle-fold dispatches:
970 measured / 1,164 including warmup. All other declared stages remained CPU.

Startup/discovery medians were 108.205ms CPU / 213.873ms Metal; total setup,
including readiness/execution checks, was 3.129s / 3.228s. This measures fresh
worker startup with warmed host/runtime/helper caches, not a cold OS boot or a
persistent warm proving process. All 12 proof requests used their initial
validation-worker PID; every trial had a distinct PID/start identity. The generic
timing labels still allow for policy-driven replacement in future runs.
`prove_call_ms` includes admission/replacement/IPC; its residual over worker time
must not be called pure startup. Execution and segment proving remain combined.
Worker total excludes external verifier time and final evidence-file writes.

## Resources, interference and evidence

Measured native request peak RSS median/range: CPU
1,965,981,696 [1,965,817,856–1,967,505,408] bytes; Metal
1,975,861,248 [1,975,320,576–1,976,188,928] bytes. These are approximately 1.831–1.840
GiB and are periodic RSS samples, not exact allocation high-water marks. They
exclude helper/VM and unmeasured Metal driver allocations. Logical Metal staging
peaked at 524,288 bytes per segment; that is not a measured GPU memory peak.
Production host-available samples across all requests ranged 25.122–42.469 GiB.
No guard or reserve was relaxed; fresh-worker trials do not establish long-lived
allocation stability.

All 240 telemetry snapshots reported AC power, Foundation nominal thermal state
and low-power=false. Those API values may also mean unknown; `pmset` had no
recorded warnings, which is not a temperature or no-throttling measurement.
fseventsd appeared in all snapshots at 95.8–125.4% decaying `ps` CPU, with browser,
indexing, security and other activity also recorded. No wattage/energy data or
isolated driver-allocation measurements were collected. No background process
was stopped to improve the results, and no noisy sample was discarded.

Memory reporting correction: earlier conversational readiness checks used a
conservative `vm_stat` proxy that omitted its separately printed speculative
pages. The SDK documents that Mach `free_count` already includes those pages.
Thus those quick checks understated the production policy reading; they were
not exact admission observations. A local SDK counter check confirmed the
mapping and is saved in `evidence/benchmark-memory-counter-check.json`.
The production guard, source-bound resource records and original historical
observations were not modified. Use the native resource records above for this
campaign's authoritative memory evidence.

Machine-readable results: `evidence/benchmark-comparison.json/.csv` and companion
`benchmark-comparison-acceptance.json/.csv`. The collector revalidated 12 source-
bound runs, 336 acceptance artifact hashes and 29 benchmark archive hashes;
six one-ULP timing serialization differences are recorded under the existing
narrow rule, with all cryptographic identities and counters exact. The 13
reporting guards passed again against this campaign's records and count as zero
proofs (`benchmark-comparison-controls.json`). Raw artifacts remain under
`target/macos/benchmarks/f57fcc0ff71d4d6391605cb761c0a1a6` and its twelve
`target/macos/worker-runs/<id>` directories. No remaining workers/containers were
found; the dedicated VM was stopped after collection.

Next concrete measurement: apply the same warmup/five-pair methodology to the
already validated hashing and multi-segment fixtures before selecting another
Metal optimization. Physical-memory pressure recovery, GPU hardware faults,
benchmark/calibration IPC, broader non-Mac runtime and contract/network acceptance
remain separate, unrun gates. No new production code or dependency change was
needed for this completed comparison.

## Earlier attempt: host-memory admission refusal

The first fixed-setting campaign did not reach measured trials. One CPU warmup
produced an independently accepted complete Groth16 proof; Metal warmup was
refused before proving by the existing host-memory guard. No CPU/Metal speedup,
median comparison or performance conclusion is established. Attempt 1 and all
unattempted slots are retained, rather than averaging only successful proofs.

Campaign: `f8b460c76d594ae88e6430ab29e1aa27`, source commit `461965e`, engine
`0d5b41e8e44876c1e8b5a8a671aabc154e33106c`, unchanged original verifier pin
`f8dcbdf8124a3f1528146bb01d539c55545cbb3d`. Same sealed Metal-capable worker
(`2c3bf297…`) in explicit CPU/Metal modes; separate CPU-only binary is preserved.
Fibonacci(20), po2 18, final Groth16, six Rayon threads, one active proof, 512 KiB
logical Metal staging, local linux/amd64 QEMU helper, six helper CPUs/24 GiB cap,
32 GiB dedicated VM. No per-mode tuning or cryptographic changes.

The immutable plan is one complete warmup per mode followed by five alternating
CPU/Metal pairs. Every trial creates a fresh supervised pool. Existing readiness
and valid/invalid execution checks precede proving; a validation worker may be
retired before a proof. Cache warmup does not imply persistent worker reuse.
No cache flushing or cold OS boot is claimed. Startup discovery is timed apart
from proving; the exact prove-call timer includes admission/replacement and IPC,
so its residual over worker time is not a pure startup measurement.

| Planned phase | Result | Worker total | Exact prove call | External verifier |
|---|---|---:|---:|---:|
| CPU warmup | Complete Groth16; positive + 3 negative checks passed | 164.432s | 164.731s | 39.937ms |
| Metal warmup | Host-memory admission refused before proving | — | 819.455ms refusal | — |
| Five measured pairs | All ten slots unattempted | — | — | — |

CPU warmup stages: execution plus segment 2.695s, recursion 7.366s,
identity plus emulated wrapping 154.341s, final SDK verification 1.764ms.
Rounded production stderr separately reports identity p254 33.146s and helper
shrink/wrap 121.187s (0.1ms diagnostic precision, not new isolated instrumentation).
Execution-only and isolated GPU-kernel durations remain unmeasured. CPU proving
recorded zero GPU commands. Sampled native peak RSS was 1,965,539,328 bytes;
this excludes VM/helper and unmeasured Metal driver allocations.

At failed Metal admission, the pre-call observation was 23,597,678,592 available
bytes (21.977 GiB); the unchanged policy requires 22 GiB (6 GiB expected proof +
16 GiB reserve). Its small preflight worker was recycled, but admission still
refused. No Metal proof or wrapping helper started. Later observed availability
remained around 21.8–21.9 GiB. Roughly 4–8 GiB extra headroom was requested from
the owner; applications, guard and limits were left intact. A new full campaign
is required when headroom is available; this attempt must stay in the evidence.

Telemetry was sampled before/after and about every ten seconds: AC power,
Foundation nominal thermal category and low-power=false, with no recorded
`pmset` thermal/performance warning. Foundation may map unknown state to those
values; warning absence is not a temperature or no-throttling measurement.
Background fseventsd reached 100.4% in decaying `ps` CPU samples; unrelated apps
and the original Colima VM remained running. No watts/energy or profitability
measurement was made. Raw telemetry retains interference and unknowns.

`evidence/benchmark-attempt-1.json/.csv` stores the immutable plan, all twelve
scheduled slots, timings, telemetry, provenance and failure; `statistics` is null.
The companion `-acceptance.json/.csv` binds real proof artifacts and request
ledgers. `benchmark-summary-controls.json` records 13 passed reporting guards,
which are explicitly zero cryptographic proof tests. SETUP.md documents the
runnable campaign and collector. Earlier correctness measurements follow and
are not substitutes for the missing controlled comparison.

# Correctness-run measurements

These are observed CPU and hybrid correctness timings, **not a controlled
benchmark or CPU/Metal performance comparison**. There was development/build activity and other host
load. No formal cold/warm separation, five-run-per-mode trial, thermal/power
sampling or sustained thermal test was performed. The expanded section below
records the completed twenty-request supervised-worker correctness run.
Do not infer a speedup, economic utility or unattended reliability.

Hardware: Apple M5 Max, 18 CPU/40 GPU cores, 128 GiB unified RAM, macOS 26.4.
The first table records the initial CPU milestone: no proving GPU kernels ran
in those samples. Later sections record actual Metal work. Host threads: six
Rayon; one active proof.
Groth16 helper: pinned linux/amd64 image under QEMU in a local Colima VM,
6 CPUs / 32 GiB VM, each helper capped at 24 GiB. Exact image/key hashes and
source/guest/input provenance are in evidence/validation.json.

| Workload/path | Completed samples | Total seconds | Native sampled peak RSS |
|---|---:|---|---|
| Fibonacci(20), standalone harness, po2 18 | 2 | 166.471, 167.191; median 166.831 | 1.827–1.828 GiB |
| SHA-256-chain(10), harness, po2 18 | 1 | 174.725 | 2.395 GiB |
| Five-segment boundary, harness, po2 16 | 1 | 242.000 | 2.558 GiB |
| Fibonacci(20), real worker IPC, po2 18 | 4 | 159.875–167.932; median 164.630 | Not measured as a high-water metric |

Worker samples include three consecutive successful requests and one after an
intentional timeout/restart. Admission conservatively recycled a worker once
before request three. These are not four identical warmed-process trials.
The five-segment fixture executes 126,077 user cycles and 294,912 total cycles;
Fibonacci uses 3,316 user / 32,768 total cycles. Exact images/inputs are retained.

Example stage times (seconds, harness monotonic timers):

| Stage | First Fibonacci | Five-segment boundary |
|---|---:|---:|
| Execution plus segment proving | 2.689 | 23.622 |
| Recursion/compression | 7.395 | 63.217 |
| Identity p254 plus final wrap | 155.995 | 155.025 |
| In-process final verification | 0.001789 | 0.001805 |
| Harness internal total | 166.105 | 241.926 |

The outer smoke total includes process startup and external verification;
its stage timers combine execution/witness with segment proving. Do not claim
an independently measured execution-only stage. Worker stderr splits identity
and wrapping: successful Fibonacci wraps took about 120.6–122.3 seconds and
identity about 28.6–35.8 seconds. The wrapping bootstrap dominates this tiny job.
Independent verifier positives and tamper checks all pass for completed proofs.

RSS comes from `ps` samples of the native harness (approximately every five
seconds). It can miss peaks and excludes the helper/VM. A manual Docker sample
observed approximately 7.2 GiB in the wrapping container; this was not a traced
peak. The VM's configured 32 GiB is a ceiling, not observed use. At the initial CPU milestone, native worker
resource protection sampled RSS every second without an exported high-water
series; later native hybrid runs retain sampled per-PID peaks. There is no GPU peak
measurement or separate GPU memory allowance.

Failed attempts are in measurements.csv / validation.json: raw-ELF packing
failure; po2 14 missing lift_13 artifact after a verified composite; po2 18
boundary proof that verified but failed multi-segment coverage; first worker
startup rejected by the original free-only memory guard. The deliberate
70-second wrapping timeout is separately labeled expected, with cleanup and
restart verified. Earlier compilation/platform-test failures are listed in
VALIDATION.md. None is discarded to improve averages.

The early benchmark plan called for extended stability and an identical
supported guest/input/po2 and resource envelope for CPU and a validated hybrid mode. Warm up and collect at least five complete alternating
runs per mode; record cold start, failures, actual stages, thermal/background
load and independently verified outputs. The isolated hybrid harness and subsequent native-worker milestone below pass
complete proof acceptance. The later twenty-request run and controlled Fibonacci
comparison are now complete as recorded above; these early timings remain
correctness observations and are not pooled into that comparison.


## First integrated hybrid correctness runs (2026-10-31)

| Run | Mode / workload | Segments | GPU completions | Wall seconds |
|---|---|---:|---:|---:|
| `d4e86941bd7b464f9954aa24ea1b2375` | metal / fibonacci | 1 | 194 | 168.439 |
| `1ad9ae716a73463584b840fcf268bac4` | metal / fibonacci | 1 | 194 | 166.856 |
| `0551842728b9449ab3d85de41a3668f8` | metal / segment-boundary | 5 | 1269 | 237.822 |
| `08b5cca54b024b9ca9d3400f41b23b1b` | cpu / fibonacci | 1 | 0 | 165.581 |

These are sequential acceptance smoke timings, not a controlled speedup study.
The declared GPU stage is segment Poseidon2 Merkle folds; recursion/identity stay
CPU and the same local emulated Groth16 helper dominates total time. The two
Fibonacci hybrid runs and CPU regression match expected public outputs and
unchanged-verifier acceptance. Peak logical Metal staging was 524,288 bytes
for every segment. Native RSS samples exclude VM/helper memory and are not
combined as if unified memory supplied a second independent GPU budget.

See evidence/hybrid-validation.json and hybrid-measurements.csv for exact stage
timings, sampled RSS, source/binary identities and retained failures. At that harness milestone, five alternating runs per mode, thermal controls
and twenty-request worker stability remained unrun; the later section below
records the completed supervised-worker sequence. No performance improvement or production readiness is claimed.


## Native hybrid worker correctness runs (2026-10-31)

Full lifecycle attempt `f8ff5abb2cef4a19bb02e24fbcb318d0` completed in 778.462s
including preflight, four proofs, two deliberately interrupted requests and
cleanup. Actual Rust acceptance tests took 777.73s (2 passed, 0 ignored).

| Successful request | Worker PID | Proving seconds | GPU completions |
|---|---:|---:|---:|
| proof-0 |8653|163.997|194|
| proof-1 |8653|159.590|194|
| proof-2 |8653|159.140|194|
| proof-after-restart |13430|164.465|194|

All four outputs passed unchanged-verifier positive and three negative checks.
The first three share a worker process; the fourth follows intentional helper
cancellation and a 70-second watchdog (observed 71.723s). Each incomplete request
also had 194 completed GPU commands before interruption; neither is included
as a successful proof or omitted from the evidence ledger. No unexpected proof
failure occurred in this lifecycle run.

Native RSS was sampled once per second for owned worker processes only. Peaks:
PID 8653=2,013,675,520 bytes; PID 12543=1,975,042,048; PID 13430=1,975,107,584.
These are per-PID peaks across the whole attempt, not individual request peaks
or a resident-growth series; they exclude the VM/helper and can miss transient
peaks. Metal logical staging peaked at 524,288 bytes per segment. Driver/GPU
allocation peaks, thermals and power were not measured. Unified memory remains
one host budget, not an independent extra GPU allowance.

Worker `stage_ms` separates execution-plus-segment, recursion,
identity-plus-wrapping, and final verification. `total_ms` covers the staged
worker proof path and diagnostics, excluding startup/IPC and external verifier
latency. It is not directly interchangeable with the prior smoke harness's
outer wall time. These are correctness samples with no cold/warm protocol or
controlled background load; no speedup is claimed. The additional CPU/automatic-mode regression samples completed:

| Run | Build / requested → observed | Proof seconds | GPU commands | Whole-run worker peak RSS bytes |
|---|---|---:|---:|---:|
| `2b17330c44b84da099de8089a72032aa` | cpu / auto → cpu |164.497|0|1,961,787,392|
| `0a20964eec5f4643b0805f52e49a4d44` | metal / cpu → cpu |164.718|0|1,966,292,992|
| `ac9cf5773bf34427925ab5f29ce18ac5` | metal / auto → metal |164.798|194|1,975,173,120|

All three passed unchanged-verifier positive and negative checks. Their source,
program/input/po2, cycle counts and public outputs match the forced-Metal suite.
These are one-sample mode regressions, not five-run benchmarks. Across all four
attempts, seven complete proofs passed and two deliberate interruptions remained
in the ledger; there were no unexpected proof failures. See
`evidence/native-worker-validation.json` and `native-worker-measurements.csv` for
all stage timings, exact identities and per-PID memory measurement limits.


## Expanded native workload and stability correctness samples

| Run | Workload / staging | Segments | GPU completions | Proof seconds | Sampled peak / idle RSS bytes |
|---|---|---:|---:|---:|---|
| `71387ac25ac74d4395278a3459680152` | SHA-256-chain /512 KiB, after logical admission refusal |1|593|171.639|2,449,637,376 /929,841,152|
| `3c0b5d8179934bafb482174615a05d22` | Boundary /512 KiB, po2 16 |5|1,269|237.596|2,756,018,176 /1,551,286,272|
| `d93743621b584487accb84c7df1833c6` | Fibonacci /4 KiB |1|16,680|168.982|1,975,173,120 /1,975,173,120|

All three are complete real proofs with independent positive and negative
verification. A logical reservation refusal is an expected pre-dispatch control
result, not a proof sample or physical-memory exhaustion. Minimum-staging work
is a correctness stress case, not a tuned performance configuration. The twenty-request sequence also passed; timings above establish no speedup.

RSS is sampled about once per second using real PID/start and parent/group
ownership checks. Idle RSS follows independent verification and a one-second
settle. Warm baselines and proof histories must be grouped by PID/start identity.
The production admission policy may recycle a worker; the run does not disable
that policy to force a long-lived process. Whole-host available memory includes
VM effects and must not be added to worker RSS as independent capacity. Native
RSS excludes helper/VM memory and unmeasured Metal driver allocations. Nonpositive
outer RSS readings are preserved as unavailable, not interpreted as zero use.


The twenty-request run `28bdd9ca009a4f6eacda4e11df402aed` completed in 3341.356s
including startup/control checks, observations, independent verification and
shutdown. Proof-path times: median 164.845s, range 157.222–166.442s (n=20). All
twenty complete outputs verified independently with 60 negative rejections.
Each proof recorded 194 GPU completions at 512 KiB staging (3,880 total). No
unexpected request failed. These are correctness-run timings, not controlled
CPU/Metal comparative measurements.

The production policy used 16 worker processes, with a longest same-PID streak
of 2; no guard or recycle threshold was disabled. Sampled per-request peak RSS
ranged 1,970,487,296–1,996,292,096 bytes (about 1.835–1.859 GiB). These process
resets prevent a claim of twenty-proof same-process resident stability. Four
identities each have two idle observations, and all trends remain grouped by
PID/start identity. Idle changes can reflect allocator/cache retention or
reclamation; this small per-process sample cannot establish a leak trend.

See `evidence/worker-stability-validation.json` and
`worker-stability-measurements.csv` for all 23 new complete proofs, the separately
labeled logical admission refusal, raw artifact identities, individual request
peaks and idle observations, host-available estimates and measurement limits.
Physical host-memory exhaustion, the resident-limit kill path, GPU driver
allocation peaks, thermals and power remain unmeasured or untested.
