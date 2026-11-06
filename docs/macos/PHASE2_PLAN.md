# Phase 2: Metal proving performance on macOS

## WP2b scope — revised source brief, 2026-11-05

The latest re-export has SHA-256
`f92bf25a4d8d6c94f83acca60383ff7b81fc04e0c5bd1ada20ec603df72d4454`.
It places WP2b immediately after the closed WP3 checkpoint, followed by WP5
and WP4. WP2b has one bounded iteration: widen the overlap so CPU preflight,
witness generation and accumulation can run during the preceding segment's
GPU work, while preserving WP2's explicit buffer ownership and cancellation
rules. Its paired gate is at least 4 s saved on both 10M fixtures against the
sealed WP3 worker at `target/macos/phase2/wp3-baseline/` (manifest SHA-256
`8f1f57c81a2955c91c7aa773c8309822d13557f8527bf1b2c93b4b2953888836`).

The revised protocol distinguishes paired package gates, which record
background load without closing apps, from absolute midpoint and final-line
readings. The latter require a closed browser, default 16 GiB reserve and
nominal initial thermal state. The sealed WP3 midpoint retake under those
conditions passed all 12 proofs and 36 tamper checks: arithmetic 59.556 s,
hashing 60.369 s complete five-proof medians. The campaign started nominal,
with Brave closed and the default 16 GiB reserve; thermal state was fair by
the hashing fixture. The hashing absolute line remains 0.369 s above 60 s,
so the owner's recorded choice to continue into WP2b is supported by the
controlled reading. See `evidence/phase2-wp3-controlled-midpoint.{json,csv}`.

WP2b closed after its single bounded iteration. Engine `a9120a42e` put the
next segment's DATA transforms and Merkle stages on a second Metal queue while
the previous eval-check command buffer was pending. It kept the same dispatch
order and buffer sets, and a complete 10M diagnostic proof verified. The
profile showed that the overlapped transform command took about 1.7 s GPU,
while the prior eval-check took about 1.6 s; DATA commitment still occupied
about 2.0 s wall per steady segment. The second queue competed for GPU time
instead of hiding the dependency. The 21-test Metal primitive suite passed.

The fixed paired campaign passed all 24 external proof verifications and 72
tamper rejections. With recorded Codex, Claude and Colima load, Brave absent,
8 GiB reserve, 18 Rayon/Go threads and fair initial thermal state, the
five-proof complete medians were arithmetic **60.602 s WP3 → 61.630 s WP2b**
and hashing **63.472 s WP3 → 63.342 s WP2b**. Savings were −1.028 s and
+0.129 s, both below the required 4 s. Median transform dispatch wall time
per segment was effectively unchanged: 1.992 → 1.994 s arithmetic and
1.895 → 1.901 s hashing. Peak owned guard footprint was 21.962 GiB.
Engine `bc5dfd6eb` reverts the experiment; its tree is identical to the
sealed WP3 engine. WP3 remains the best correct worker and the WP5 baseline.
No WP2b candidate is deployed, so candidate lifecycle and cancellation were
not used as acceptance gates. See `evidence/phase2-wp2b-paired.{json,csv}`
and `evidence/phase2-wp2b-validation.json`.

## WP3 transform checkpoint — 2026-11-05

The current re-export has SHA-256
`07467098c3493a0bbc0f543b0960676ba3a3e21ebd1fb7a65d10bb8dec40dc97`.
It clarifies that each package pairs against the immediately preceding closed
checkpoint. WP3 therefore uses the sealed WP2 worker at
`target/macos/phase2/wp2-baseline/` (manifest SHA-256
`274a86fb0e10438983285aed6232a6ef922fcb7a73d3cec4bb64f7be9420ca19`).
It also attributes about 3–4 s of the WP2 campaign's measured 6 s to
pipelining itself, with the earlier WP0.5 batching contribution included in
that WP1 comparison. Engine
`7143255acda4e51383237a479b51f914a8be904a` adds fused commit NTT,
zk-shift, LDE, resident twiddles and folded bit reversal; engine
`8520c737725698ddb156ccda2a41e0521213b878` makes radix-four tiles and
column-parallel dispatch the default. `RISC0_METAL_NTT_RADIX2=1` selects the
old transform route for bisection. The separate local Merkle differential
uses existing engine crates and adds no dependency or version to the miner
or engine production graph.

WP3's fixed five-pair campaign against sealed WP2 passed all 24 proofs and
72 tamper checks: arithmetic 72.684 → 61.589 s and hashing
76.409 → 65.384 s, saving 11.095 and 11.025 s. Brave, Codex, Claude and
Colima were present, as recorded by the revised spec's background-load
protocol; both arms used po2 20, 18 Rayon/Go threads and 8 GiB reserve.
The largest paired guard peak was 22.544 GiB, below the existing 24 GiB
initial budget and 32 GiB ceiling. One pipeline-disabled complete proof
measured 0.455 s GPU and 0.636 s dispatch wall steady transform time per
segment; the requested-buffer traffic model gives 327 GB/s, above the
180 GB/s floor. The 21-test Metal primitive suite on both default and
radix-two fallback, exact Merkle differential,
real-witness CPU differentials at po2 14/18/20, CPU-only build, three
offline Colima feature paths, and 20-request lifecycle passed. Idle footprint
fell 14.58%; the 46M stretch verified in 217.957 s with a 17.905 GiB guard
peak. See `evidence/phase2-wp3-{paired,validation}.json` and
`evidence/phase2-wp3-paired.csv`.

The WP3 stage and paired-saving gates pass. Its recorded-load paired medians
were 1.589 s arithmetic and 5.384 s hashing above 60 s; these are not
absolute midpoint readings under the clarified protocol. The controlled
retake above puts arithmetic below 60 s and hashing 0.369 s above it.
The next baseline arm is sealed at `target/macos/phase2/wp3-baseline/` with
manifest SHA-256 `8f1f57c81a2955c91c7aa773c8309822d13557f8527bf1b2c93b4b2953888836`;
it pins engine `8520c7377` and miner checkpoint `afa97e3`.
The 46M result is below the 220 s minimum line but above the 180 s pass
line. A separate browser-closed repeat was stopped after arithmetic once
the revised recorded-load rule was re-read; its partial ledger is retained.
A first lifecycle attempt passed 20 steady proofs and idle drift, then used
a 60 s helper-timeout deadline that exceeded the tiny proof's 45.4 s
completion. The 35 s rerun passed every control and is the closure run.

Earlier WP3 launch-grid diagnostics are recorded in
`docs/macos/evidence/phase2-wp3-start.json`. Full butterfly coverage measured
2.420 s steady transform wall time per po2-20 segment; a bounded grid per
column measured 2.220 s. Both complete arithmetic proofs passed the pinned
external verifier and its three tamper tests. The 20-test Metal primitive
suite passed for each variant. These were unfused pre-port diagnostics and
are separate from the final paired gate measurement above.

## Revised source brief — 2026-11-05

The current re-export of `Phase 2 spec Metal proving performance on macOS.md`
has SHA-256 `966e6bbfd2cbef4912c5cd18903e776b9ffc67c50f612ac2f71805d53247359d`.
It recalibrates WP0.5's 2–3 s batching closure while retaining its measured
results, assigns the remaining transform host gap to WP3's wall-time gate, and
keeps WP2 segment pipelining next. WP2 is closed below on the existing Phase 2
branches.

## WP2 checkpoint — 2026-11-05

Engine `6ebdadd840257b4c8b59171f6e12f063b496b1a7` commits the evalfast
phase command buffer without waiting; engine
`a80fcb3c0d912f0ca82befe0fbcbff18e9f35eaa` overlaps the next segment's
CPU witness and accumulation with the previous GPU-owned buffer set, logging
buffer-set handoffs and completion. `RISC0_SEGMENT_PIPELINE=0` restores the
synchronous route for bisection. No kernel, proof format, or 32 GiB ceiling
change was needed.

Po2-14/18/20 real-witness CPU differentials and the unchanged external
verifier passed, including all three tamper rejections. Targeted cancellation
occurred at `cpu_begin set=12 gpu_owned_set=Some(11)` and left no worker group
or helper. The 20-request lifecycle passed with 21 verified proofs, expected
cancel/timeout failures, recovery, shutdown, normal pressure, and a 15.50%
decrease in idle footprint. A first lifecycle attempt retained in evidence
failed only its harness assertion: the 20-second watchdog fired during
recursion before the helper had started. The 35-second rerun reached the helper
and passed. The CPU-only worker and its five tests passed; Colima compiled the
recursion Metal, RV32IM prove, and zkVM Metal-recursion Linux paths.

Against the sealed WP1 worker, the fixed five-pair campaign passed both 10M
gates: arithmetic 80.419 → 74.391 s (6.029 s saved), hashing
81.463 → 75.413 s (6.050 s saved). All 24 proofs and 72 tamper rejections
passed. The campaign used an 8 GiB reserve on both arms with Brave, Codex,
Claude, and Colima present; thermal snapshots changed from nominal to fair.
One excluded arithmetic warmup waited 60.95 s for admission, while all
measured proofs had zero wait. The maximum paired WP2 guard peak was
22.014 GiB, below the existing 24 GiB initial resident budget and 32 GiB
ceiling. One 46M stretch proof verified in 284.415 s with 46 handoffs and
normal pressure. WP2 is closed and WP3 is next. See
`evidence/phase2-wp2-{paired,validation}.json` and
`evidence/phase2-wp2-paired.csv`.

## Revised source brief — 2026-11-04

The re-exported `Phase 2 spec Metal proving performance on macOS.md` has
SHA-256 `299ebd441aa2e1c0c70f7587a88b8cc32154d0835a0eaea0d807adef11028597`.
It adds WP0.5 command batching before WP2, makes the 20-request idle footprint
criterion one-sided (at most 5% growth), and amends the WP3 mid-point decision.
The WP1 final lifecycle's 10.60% decrease passes the revised memory rule;
WP1 is closed at 78.688 s arithmetic and 80.779 s hashing paired medians.
WP0.5 is closed below; WP2 is next.

Engine checkpoint `6b3ec9539fb3a4e5c267c2a4f8f81d6d90ac577d`
adds command-buffer batching for NTT, shift/LDE, Merkle, and GPU-only prover
stages, plus per-buffer counters. The sealed WP1 baseline binary and provenance live in
`target/macos/phase2/wp1-baseline/` (manifest SHA-256
`4fb202a4b4578b49763c90dbe89ee7a4edf45b7a8886d02377f968ee7b6882a2`).
The 20-test Metal primitive suite, recursion offset differential, and
CPU-only feature build passed before candidate timing.

Three bounded batching iterations have verified complete po2-14, po2-18 and
po2-20 proofs with exact real-witness CPU differentials and the unchanged
external verifier. The final GPU profile cuts 10M command buffers from
1,873 to 143 for transforms, 1,163 to 61 for Merkle, and 5,539 to 540 for
recursion. Its host dispatch overhead remains 1.809/1.756 s for transforms
and 0.703/0.715 s for recursion (arithmetic/hash), above the 0.3 s gate.
The two earlier profiles and their raw runs are retained as iteration evidence.
The po2-14/18/20 real-witness differentials, external verification and tamper
checks, CPU-only build, Linux feature-path check, and 20-request lifecycle
passed. The initial lifecycle's 10-second helper watchdog captured a partial
sidecar and failed; the recorded 20-second rerun passed all controls with
21 verified proofs and a 14.51% drop in idle footprint. The controlled
five-pair campaign saved 2.049 s on arithmetic (77.758 → 75.709 s) and
3.023 s on hashing (79.801 → 76.778 s), with all 24 proofs verified and
72 tamper rejections. Arithmetic misses the required 3 s saving; transform
and recursion host overhead also miss the 0.3 s target. WP0.5 is closed as
gate-failed after three bounded iterations, retaining the best correct
variant on the Phase 2 branch. The 2026-11-05 source brief subsequently
recalibrated the WP0.5 gate to the measured 2–3 s range and records this
variant as the closed default; the original 3 s and 0.3 s misses remain in
the evidence. WP2 is in progress. See
`evidence/phase2-wp05-{gpu-profile,validation,paired}.json`.

## WP1 paired checkpoint — 2026-11-04

The five-pair campaign against the sealed WP0 archive passed both timing
gates with Brave closed and the default 16 GiB reserve: arithmetic complete
median 118.554 → 78.688 s (39.867 s saved), hashing complete median
119.566 → 80.779 s (38.787 s saved). Evalfast median split-kernel GPU time
per segment was 1.692 s and 1.633 s. All 24 paired-campaign proofs verified,
all 72 tamper checks rejected, and admission wait was zero. The hashing WP0
median is now re-baselined under the same conditions; the prior 130.6 s
Brave-open, 12 GiB result is historical. See
`evidence/phase2-wp1-paired.{json,csv}`.

Engine `5acb4b8721b54ba2621d89f5f8d677cd5eca29d2` now selects evalfast
by default; `RISC0_RV32IM_TAIL_SPLIT=1` selects the tail-split/256 fallback.
Both routes passed a rebuilt-worker po2-14 real-witness CPU differential and
external verification. The final build's 20-request lifecycle also passed
functionally, with 21 verified proofs and 63 tamper rejections. Its idle
footprint fell 10.60% from request 1 to request 20, which passes the revised
one-sided memory condition. The later settled range was 3.68% and pressure
stayed normal. WP1 is closed; see
`evidence/phase2-wp1-default.json`.

## WP1 reopened — 2026-11-04

The owner reset WP1's iteration budget. The explicit scope is to port the
in-tree evalfast pass pipeline's output to MSL, beginning with its main/LogUp
phase split and then the lazy-reduction representation across the phase
boundary. The committed tail-split/256 worker remains the fallback until a new
shader passes the synthetic CPU oracle and real-witness differentials. The
prior bounded-trial gate miss remains a historical measurement, not a closure
of the reopened package.

The requested one-hour diagnosis finished within the hour: a diagnostic shader
returns the main-chain value at the same `rv32im_v2_3` `x1102` boundary used by
`accfirst_cuda.py`, omitting the LogUp continuation and the calls to v2_2..0.
Five alternating, counterbalanced po2-20 timing pairs at 256 threads measured
median repeat-command GPU time of 4.969 s for the unchanged fallback and
4.470 s for main-only, a 1.112× ratio (0.499 s saved). Each pair was
1.070–1.115×. The diagnostic does not check a CPU oracle and cannot produce a
proof. This is a percent-scale result, so it does not confirm the proposed
multiples-scale live-state diagnosis. See
`evidence/phase2-wp1-logup-probe.{json,csv}`.

Engine `d194533fa5138a8d9a352c977ae622d59cca4d41` now regenerates the
evalfast pass output through `remat_lu.py` during the Metal build and translates
its main/LogUp phase split to three MSL kernels. Set
`RISC0_RV32IM_EVALFAST=1` to select it; the existing tail-split/256 evaluator
remains the default fallback. The isolated po2-20 CPU differential passed with
1.694 s repeat GPU time at 256 threads and 1.741 s at 512 threads. Complete
po2-14, po2-18 and po2-20 proofs passed real-witness CPU differentials and the
external verifier. In 10M diagnostics, the three-kernel constraint stage took 16.899 s
for arithmetic and 16.948 s for hashing, saving 30.914 s and 34.111 s against
the prior tail-split/256 diagnostics. The same phase split without `EF_WIDE`
took 4.766 s per repeat po2-20 dispatch. The 20-request lifecycle passed with
21 verified proofs including recovery, 63 tamper rejections, cancellation,
helper timeout, restart, and shutdown. Nine workload-limit controls passed.
Pressure stayed normal. Idle footprint fell 12.39% from the first proof, then
varied 0.25% after settling. A separate po2-14 proof passed with the default
tail-split/256 fallback and CPU differential.
See `evidence/phase2-wp1-evalfast.json` for source hashes and verified run paths.
This first MSL target stops after `remat_lu.py`; CUDA's `mksm2.py` shared-memory
placement and `mksub.py` chain tails remain follow-on port work.

Original source brief: `Phase 2 spec Metal proving performance on macOS.md`, dated
2026-11-04, SHA-256
`22c73353915de7154577858ab4556d6d52fc6af3f463b3e6b2009f3ec5c707b7`.
This page records implementation status and measured checkpoints. The source
brief remains the detailed technical and acceptance specification.

## Starting point

- Miner: `9bc7466eaf6a4b48d8655d39f69cc89fa40f0298` on
  `codex/hemi-macos-metal-phase2`.
- Engine: `dcbe9f4b9140a48d4792ffcd3a3eba19897be175` on
  `codex/hemi-macos-metal-phase2`.
- Both starting commits match the published Phase 1 fork branches. The brief's
  earlier request to commit an uncommitted tree is already satisfied. New
  Phase 2 branches follow the owner's request for this task.
- The pinned external verifier SHA-256 is
  `2bcdcb7f50e2303545898962d23bdc6cba9512038383a93c975f95b89cf8225c`.

## Gates and status

| Package | Required gate | Status |
| --- | --- | --- |
| WP0 baseline and GPU timing | Sealed archive; five warm proofs per 10M fixture, one 46M proof; per-kernel GPU and pipeline statistics | Baseline, profiles, CPU-only build, limits, and 20-request lifecycle measured; transform GPU-only wall reconciliation misses 10%; Linux cross-target compile unavailable locally |
| WP1 RV32IM evaluator | At most 2.0 s per po2-20 segment, at least 25 s saved per 10M proof, exact CPU differential | Closed under the revised one-sided memory gate; five paired proofs per fixture pass timing and correctness; evalfast is default with tail-split override. |
| WP0.5 command batching | Revised 2026-11-05 gate: about 2–3 s saved; command counts and correctness/lifecycle gates | Closed with batching default after three iterations. Arithmetic saves 2.049 s, hashing saves 3.023 s; the former 3 s and 0.3 s targets missed, and the remaining transform gap moves to WP3. |
| WP2 segment pipeline | At least 6 s saved per 10M proof, 20-request stable idle footprint, cancellation clean | Closed, passed: 6.029/6.050 s paired savings; 15.50% idle decrease; overlap cancellation and CPU differential passed. |
| WP3 transforms | At most 0.8 s transform wall time per po2-20 segment, at least 30% of the approximately 600 GB/s bandwidth ceiling, exact primitive tests and unchanged Merkle outputs | Closed, stage and paired-saving gates passed: 0.636 s isolated transform wall, 327 GB/s modeled requested-buffer bandwidth, 11.095/11.025 s paired savings. Midpoint 60 s line missed. |
| WP2b wider overlap | At least 4 s saved at 10M against WP3, lifecycle and cancellation green | Closed after one bounded iteration; gate missed (−1.028/+0.129 s). Queue experiment reverted; WP3 worker retained. |
| WP5 po2 21 assessment | Faster than po2 20 within measured memory ceiling | Pending; owner approval before changing default |
| WP4 recursion evaluator | At most 7 s lift/join at 10M, exact differential | Pending |
| WP6 pipeline cache | Warm process first dispatch at most 5 s, stale archive rebuilds | Pending |
| WP7 admission | No refusal under specified desktop load with at least 40 GiB truly free; pressure refusal still works | Pending |
| WP8 Poseidon254 | Exact differential and verified identity receipts | Optional; requires owner approval before starting |

The mid-point decision after WP3 targets both 10M warm medians at most 60 s.
A near miss within a few seconds is the owner's call; the revised order is
WP2b wider overlap, then WP5 with the profile. A large miss above roughly
70 s stops for a re-plan. The final
pass lines are at most 45 s for both 10M fixtures and 180 s for
46M; minimum acceptance is 55 s and 220 s respectively. Every complete proof
must pass the unchanged verifier and the three tamper rejections. Campaigns use
one excluded warmup per arm, five measured proofs, and retain failures.

## Frozen conditions

The pinned circuit, tap set, control IDs, proving and verification keys, 256-byte
seal, original verifier, dependency versions, and on-wire protocol remain fixed.
Metal dispatch must fail rather than silently fall back to CPU. CPU access to a
shared buffer requires completion of every command that can write it. The
worker/helper ceiling remains 32 GiB until a measured peak and owner approval
justify a change. Apple Silicon Metal features stay opt-in; CPU-only and other
platform feature paths must keep compiling. No mining, network jobs, or
transactions are part of Phase 2.

Each evaluator change requires build-time body checks, the synthetic Metal
prototype suite, and real-witness CPU differentials at po2 14, 18, and 20
before timing. NTT, hashing, or FRI changes require the primitive suite and a
full verified proof. Each checkpoint also runs the 20-request lifecycle,
limit controls, and CPU-only and Linux feature checks. The source brief gives
the exact per-package performance gates and bounded iteration rules.

Checkpoint order is WP0, WP1, WP0.5, WP2, WP3, the 60-second mid-point decision,
WP2b, WP5, WP4, WP6/WP7, and optional WP8. Changing the default po2, raising the
32 GiB ceiling, or starting WP8 requires owner approval. Engine commits precede
miner commits that cite the engine SHA. Only personal fork Phase 2 branches are
publication destinations; the three open upstream PR branches are untouched.

## WP0 record

The first Phase 2 worker was rebuilt with the existing pinned dependencies and
local recursion artifact. Its sealed binary, build inputs, Cargo lock, metadata,
verifier, and helper manifest are archived at
`target/macos/phase2/baseline/`. The archive manifest records file hashes and
both starting commits. Five warm arithmetic and hashing proofs and one 46M
stretch are complete; see `BENCHMARKS.md` and
`evidence/phase2-wp0-baseline.{json,csv}`. GPU timing instrumentation is
committed in engine `09c2f4d2836cbcd88cd29e65b6a6bdd8db262416`; two
verified diagnostic profiles are in `evidence/phase2-wp0-gpu-profile.{json,csv}`.
The 10% GPU-only reconciliation target is missed by the transform stage because
host dispatch consumes about 1.9 s across 1,873 commands. The profile separates
that cost from GPU execution. Native and CPU-only builds, the po2-14 CPU
differential, nine limit controls, and the 20-request lifecycle passed; see
`evidence/phase2-wp0-validation.json`. The measured first-to-last idle footprint
drift was 4.49%. Linux cross-target compilation remains unrun because the local
pinned toolchain has only the Apple Silicon target. WP1 is the next package.

The arithmetic campaign used the unchanged 16 GiB host reserve. The first
hashing campaign was refused by the host-memory admission gate before any proof
dispatched, with about 40 GiB conservatively estimated available. Its raw
failure remains under `target/macos/phase2/baseline/hash-10m.log`. A new
hashing and stretch campaign uses the previously documented 12 GiB local trial
override; the default worker policy is unchanged. Campaign settings are
recorded separately in the evidence.

## WP1 bounded iteration

Engine `afd4342f90160c60829818dca953bcfe952a5639` adds a generated
function-tail split to all 20 RV32IM chain functions and selects 256 threads
per group. The pass reissues only literal field constants and immutable trace
loads in each tail; the original CPU/CUDA normalized body check still runs
before the transformation. The isolated Metal prototype matched the CPU oracle
at po2 4, 8, 14, 18, and 20, including a repeated dispatch. Complete proofs
at po2 14, 18, and 20 passed real-witness CPU differentials and the unchanged
external verifier. The worker SHA-256 is
`9b0f07321888abf9292c466fe4d3dee93d499de3dca8b03cec491020d66bc2c1`.

The second iteration retargeted the in-tree fast Fp and extension-field
reduction forms as opt-in RV32IM-only shader variants. Both were exact in the
synthetic matrix but slower at po2 20, so production leaves them disabled. The
third iteration swept 32/64/128/256 threads on the split shader; 256 was
fastest. One diagnostic 10M proof per fixture measured constraint wall time
at 47.813 s arithmetic and 51.059 s hashing, or 4.78 and 5.11 s per segment.
The ≤2.0 s gate is missed, so no five-pair ≥25 s saving claim is made. The
main/LogUp phase split from the evalfast pipeline, lazy Fp values across a
scratch buffer, and the resulting larger occupancy target remain unported.
See `BENCHMARKS.md` and `evidence/phase2-wp1-iteration.json` for the bounded
trial and validation record.

The lifecycle passed 20 steady proofs and one recovery proof, each with three
tamper rejections. First-to-last steady idle worker footprint changed by
−0.42%; pressure stayed normal. Cancellation, worker timeout, restart, and
shutdown controls passed, as did nine workload-limit rejections. The helper
timeout control was not exercised in this run. The checkpoint has no new crates
or version bumps. Linux cross-target compilation remains unavailable locally.

## Fold-only cleanup

A dedicated follow-up engine commit,
`fca0c099582f15729f98af0743f45bc56c217705`, removes the old fold-only
Metal HAL, CPU fold adapter, hybrid feature chain and local prover constructor.
The miner's isolated hybrid probe manifest and launcher are removed. The
shared verifier source and its provenance script remain unchanged to preserve
the pinned verifier binary. The active resident Metal worker was rebuilt and
resealed at the cleanup engine commit; its SHA-256 is
`c5f369bb7f76d6bdb61823149997e850bbb47299981be58cde17ae7aff31b7f5`.
This binary differs from the preceding WP1 measurement binary, so its timing
gate is not reinterpreted. A fresh po2-20 CPU differential and verified proof
passed. The 20-request lifecycle plus recovery proof passed with 63 tamper
rejections, −0.68% idle footprint drift and normal sampled pressure.
Cancellation, worker timeout, restart and shutdown controls passed. See
`evidence/phase2-fold-cleanup.json` for the validation record.
