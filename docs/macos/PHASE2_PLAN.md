# Phase 2: Metal proving performance on macOS

## Revised source brief — 2026-11-05

The current re-export of `Phase 2 spec Metal proving performance on macOS.md`
has SHA-256 `966e6bbfd2cbef4912c5cd18903e776b9ffc67c50f612ac2f71805d53247359d`.
It recalibrates WP0.5's 2–3 s batching closure while retaining its measured
results, assigns the remaining transform host gap to WP3's wall-time gate, and
keeps WP2 segment pipelining next. WP2 implementation and evidence are in
progress on the existing Phase 2 branches.

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
| WP2 segment pipeline | At least 6 s saved per 10M proof, 20-request stable idle footprint, cancellation clean | In progress |
| WP3 transforms | At most 0.8 s per po2-20 segment, exact primitive tests and unchanged Merkle outputs | Pending |
| WP5 po2 21 assessment | Faster than po2 20 within measured memory ceiling | Pending; owner approval before changing default |
| WP4 recursion evaluator | At most 7 s lift/join at 10M, exact differential | Pending |
| WP6 pipeline cache | Warm process first dispatch at most 5 s, stale archive rebuilds | Pending |
| WP7 admission | No refusal under specified desktop load with at least 40 GiB truly free; pressure refusal still works | Pending |
| WP8 Poseidon254 | Exact differential and verified identity receipts | Optional; requires owner approval before starting |

The mid-point decision after WP3 targets both 10M warm medians at most 60 s.
A near miss within a few seconds is the owner's call and continues to WP5 with
the profile; a large miss above roughly 70 s stops for a re-plan. The final
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

Checkpoint order is WP0, WP1, WP0.5, WP2, WP3, then the 60-second mid-point decision,
WP5, WP4, WP6/WP7, and optional WP8. Changing the default po2, raising the
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
