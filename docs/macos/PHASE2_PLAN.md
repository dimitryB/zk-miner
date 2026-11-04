# Phase 2: Metal proving performance on macOS

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

Source brief: `Phase 2 spec Metal proving performance on macOS.md`, dated
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
| WP1 RV32IM evaluator | At most 2.0 s per po2-20 segment, at least 25 s saved per 10M proof, exact CPU differential | Opt-in evalfast phase split meets synthetic and single 10M diagnostic timing gates; exact synthetic and real-witness differentials, fallback proof, and 20-request lifecycle pass |
| WP2 segment pipeline | At least 6 s saved per 10M proof, 20-request stable idle footprint, cancellation clean | Pending |
| WP3 transforms | At most 0.8 s per po2-20 segment, exact primitive tests and unchanged Merkle outputs | Pending |
| WP5 po2 21 assessment | Faster than po2 20 within measured memory ceiling | Pending; owner approval before changing default |
| WP4 recursion evaluator | At most 7 s lift/join at 10M, exact differential | Pending |
| WP6 pipeline cache | Warm process first dispatch at most 5 s, stale archive rebuilds | Pending |
| WP7 admission | No refusal under specified desktop load with at least 40 GiB truly free; pressure refusal still works | Pending |
| WP8 Poseidon254 | Exact differential and verified identity receipts | Optional; requires owner approval before starting |

The mid-point decision after WP3 requires both 10M warm medians to be at most
60 s. The final pass lines are at most 45 s for both 10M fixtures and 180 s for
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

Checkpoint order is WP0, WP1, WP2, WP3, then the 60-second mid-point decision,
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
