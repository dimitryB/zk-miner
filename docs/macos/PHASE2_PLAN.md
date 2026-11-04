# Phase 2: Metal proving performance on macOS

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
| WP1 RV32IM evaluator | At most 2.0 s per po2-20 segment, at least 25 s saved per 10M proof, exact CPU differential | Pending |
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
