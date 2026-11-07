# Phase 2 final report: Metal proving on macOS

2026-11-06, updated 2026-11-07 · Dimitry Burukhin

Phase 2 took the resident RV32IM Metal worker on the Apple M5 Max from 117 s to 49.5 s on the fixed 10M-cycle fixture and from 485 s to 160 s on the 46M stretch, with every proof accepted by the unchanged pinned verifier and no cryptographic change. Updated 2026-11-07: the published worker also carries the upstream security hardening merged on the last day, verified by a paired campaign to cost nothing.

## Final readings

All three minimum lines are met; the 46M stretch also meets its desirable line, and both 10M fixtures miss theirs by 4.5–5 s. The published worker is the one that also carries the upstream merge of 2026-11-06 (hemilabs/risc0 `release-3.0` at `54921e770`, hemilabs/zk-miner `main` at `af44acc`). Controlled conditions for its reading on 2026-11-07: 15 minutes idle after the last build, browser closed, default 8 GiB reserve, nominal thermal state at the start, po2 20 with segment pipelining, 18 Rayon / 18 Go threads, one excluded warmup and five measured proofs per fixture; 18 proofs verified by the unchanged external verifier with 54 tamper rejections; peak owned footprint 19.3 GiB; normal memory pressure on every proof.

| Fixture | Published worker, 2026-11-07 (range) | Pre-merge worker, 2026-11-06 (range) | Minimum line | Desirable line | Phase 1 end | Phase 2 start |
| --- | --- | --- | --- | --- | --- | --- |
| Arithmetic 10M (10,003,116 guest cycles) | **49.5 s** (49.5–51.5) | 48.5 s (48.5–50.3) | 55 s, met | 45 s, missed | 117.2 s | 119.6 s |
| Hashing 10M (10,003,440 guest cycles) | **50.3 s** (49.2–52.3) | 49.5 s (49.5–50.5) | 55 s, met | 45 s, missed | 120.8 s | 119.6 s |
| Arithmetic 46M | **159.6 s** (158.4–161.6) | 165.8 s (162.5–167.5) | 220 s, met | 180 s, met | 484.7 s | 501 s |

The two worker columns differ only by the merge and the WP6b/WP7 changes, and a paired campaign on 2026-11-07 — merged worker against the sealed pre-merge worker, both resident, alternating, five pairs per 10M fixture and three at 46M — put that difference at +0.007 s, +0.001 s and +1.0 s: nothing. What separates the columns is the day. Absolute readings on this machine move by a few seconds between days and by more within an hour of a build (a reading taken minutes after the merge rebuild came in at 52.5 s and tripped the stop rule), which is why the protocol now requires a 15-minute idle gap before any absolute reading and answers every "did this cost time" question with a paired campaign. The two phase-2 start readings of the arithmetic fixture differ by a second: 119.6 s is the sealed WP0 campaign, 118.6 s the same worker re-run as the WP1 campaign's baseline arm, which the stage table below uses. Against the 5070 Ti estimate of 10–14 s for the same 10M job, the gap is now roughly 3.5–5×, down from about 10× at the start of the phase; the stated goal of a ~4× gap for development use is met at the upper end of that estimate. A 46M-cycle job proves in well under three minutes on the laptop; it took eight minutes at the start of the week and 35 minutes before phase 1.

## Results by package

Each package was measured as a paired five-proof campaign against the immediately preceding sealed worker (after the WP2 correction), so the savings below are attributable to that package. Running totals are the arithmetic-10M median of the candidate arm in its own campaign, which drift a few seconds with background load; the controlled readings are the authoritative absolute numbers.

| Package | Engine commit(s) | What changed | Measured effect | 10M after |
| --- | --- | --- | --- | --- |
| WP0 | `09c2f4d28` | Sealed baseline, per-kernel GPU timing in the profile | Attribution of GPU vs host time per stage | 119.6 s |
| WP1 | `d194533fa`, `5acb4b872` | Evalfast phase split ported to MSL (three kernels, `EF_WIDE`), made default | Evaluator 5.65 → 1.69 s/segment; −39.9 / −38.8 s | 78.7 s |
| WP0.5 | `6b3ec9539` | One command buffer per stage | −2.0 / −3.0 s; recursion host overhead 1.9 → 0.7 s; transform gap unchanged | 75.7 s |
| WP2 | `6ebdadd84`, `a80fcb3c0` | Next segment's CPU witgen/accum under the prior segment's eval | ~−8 s by serial-vs-pipelined screens (6.0 s in its campaign incl. batching) | 74.4 s |
| WP3 | `7143255ac`, `8520c7377` | Fused iNTT + zk-shift + LDE, resident twiddles, radix-4 tiles, column-parallel | Transforms 1.78 → 0.64 s/segment wall; −11.1 / −11.0 s; recursion 10 → 8 s | 61.6 s |
| WP2b | `a9120a42e` (reverted `bc5dfd6eb`) | Second queue for next segment's commit | −1.0 / +0.1 s; segment stage is GPU-bound | — |
| WP5 | `748e7a202` | po2 21 enabled; assessed with and without pipelining | po2 21 + pipelining needs ~36 GiB vs the 32 GiB ceiling; serial po2 21 slower on hashing; not adopted, kept as profile | — |
| WP4 | `47bbd7f5a`, `bca7044c5` | Recursion gated-sum evaluator in six ordered MSL parts + lazy REDC | Recursion 7.7 → 5.6 s; −2.0 s at 10M, −10.1 s at 46M | 55.6 s |
| WP1b | `7222459e3` | `mksm2.py` hot slab (32 hottest words in threadgroup memory), `mksub.py` chain tails | Evaluator 1.70 → 1.04 s/segment; −6.8 / −7.0 s; −23 s at 46M | 50.5 s |
| WP8-lite | `cd018ee72`, `6521f87c2` | gnark child pre-started under identity, witness streamed over stdin | identity + wrap −0.5 s, complete ~0; kept | 49.5 s |
| WP7 | miner `ed649c6`…`ea8cd11` (engine unchanged) | Admission on kernel pressure plus measured owned footprint; VM estimate kept as a floor (16 GiB for a fresh worker instead of 40); guard pressure-first; default reserve 8 GiB | No latency effect. Admitted and completed with a sibling holding 64 GiB (20 GiB estimated); refuses in 0.5 ms under real warning pressure and recovers | — |
| WP6 | `11025fe0e` (draft PR, unmerged) | `MTLBinaryArchive` pipeline cache keyed by shader hash, device and OS build | −0.16 s with a warm driver cache; gate missed | — |
| WP6b | `fc7af4317`, `7816a0002` | Groth16 bundle verification off the start path; build-time driver-cache warm-up | Spawn → handshake 4.62 → 0.11 s; first-ever proof on a fresh account 25 s to first dispatch | — |
| Upstream merge | `881513e02` (engine), miner `d359e01` | hemilabs `release-3.0` @ `54921e770` and `main` @ `af44acc` folded in: execution refusals, deterministic `sys_rand`, security hardening, Intel AOT | Paired cost +0.007 / +0.001 s at 10M, +1.0 s at 46M | 49.5 s |
| Final | published worker `c3428c37…` | Controlled reading, 2026-11-07 | — | **49.5 s** |

The evaluator alone went from 5.65 s to 1.04 s per po2-20 segment across WP1 and WP1b, which is about 46 of the 70 seconds removed. WP7 and WP6b changed what the worker does before and around a proof, not the proof itself, so they have no "10M after"; their gates were admission behavior and startup time.

## Where the time goes

Arithmetic 10M fixture, medians of five proofs, stage timers from the worker's own instrumentation. Start: the phase2-wp1 baseline arm of the sealed WP0 worker. Final: `docs/macos/evidence/phase2-post-diagnosis-controlled` on the published worker.

| Stage | Phase-2 start (118.6 s) | Published worker, 2026-11-07 (49.5 s) |
| --- | ---: | ---: |
| Segment stage (witgen + STARK) | 89.1 s | 26.2 s |
| Recursion (lift and join) | 11.3 s | 5.5 s |
| identity_p254 + Groth16 wrap | 17.4 s | 17.5 s |
| Residual (IPC, verification) | 0.8 s | 0.4 s |

identity_p254 + wrap, untouched by phase 2, is now 17.5 s of the 49.5 s proof (35%); the gap to the desirable 45 s line is 4.5 s.

Phase 2 took the segment stage from 89 s to 26 s and recursion from 11 s to 5.5 s; it did not touch identity_p254 and the wrap, which were 15% of the proof on the morning of day one and are 35% now. Inside the 26.2 s segment stage the per-segment attribution is roughly evaluator 10 s, transforms 6 s, CPU accumulation 6–7 s and Merkle 4 s, overlapping by about 2 s under pipelining; inside the 17.5 s block it is identity_p254 about 12 s, gnark proving 3.5–4 s and the witness about 1 s, all compute that WP8-lite showed cannot be hidden behind anything else. The 46M proof has the same shape at scale: 116 s segment stage, 26 s recursion, 17 s identity and wrap.

## Memory, cold start and machine floor

Peak owned worker-plus-helper footprint fell during the phase from 22.5 GiB (WP3) to 18.8–19.3 GiB at the final readings, because WP1b's hot-slab evaluator spills far less private memory. The 32 GiB owned-group ceiling and 24 GiB initial resident budget are unchanged; the one configuration that needed more (po2 21 with pipelining, 32.8–35.3 GiB) was not adopted. WP7 changed admission on the last day: the gate refuses unless the kernel's memory-pressure level is normal, the conservative VM estimate only has to cover the reserve plus at most 8 GiB of projected growth (16 GiB for a fresh worker instead of the 40 GiB that refused campaigns all week), the default reserve is 8 GiB, and the in-proof guard follows pressure and the ceiling rather than the estimate. With a sibling process holding 64 GiB the estimate read 20 GiB at normal pressure; the proof was admitted and completed. Under real warning pressure the gate refuses in 0.5 ms before any worker request and recovers when pressure does. Idle footprint after 20 consecutive requests fell at every checkpoint but one, where it rose 1.0%; the criterion is one-sided (no growth beyond 5%).

Cold start: WP6's `MTLBinaryArchive` pipeline cache was built and is correct but saved 0.16 s, because macOS's own driver shader cache already covers every repeat start; it stays in a draft PR, unmerged. What every worker start actually paid was 4.6 s of SHA-256 over the 2.8 GB Groth16 bundle before the handshake. WP6b moved that onto a background thread joined before any helper use — with size, mtime, inode and device checks, and a corrupted bundle still rejected before a helper spawns — taking spawn → handshake to 0.11 s, and `build-worker.py` now runs one po2-14 proof after every build to warm the driver cache. The one true cold measurement, on a freshly created macOS account, was 25 s from first request to first GPU dispatch and 62 s for that first po2-14 proof: paid once per worker build per machine, and absorbed by the build-time warm-up.

The configuration is intended for a 64 GiB-class Mac or larger; every measurement in this phase was on a 128 GiB M5 Max (18 CPU / 40 GPU cores, macOS 26.4). No cross-machine result exists.

## What missed and why

| Package | Gate | Result | Why |
| --- | --- | --- | --- |
| WP0.5 batching | ≥ 3 s saved, host overhead ≤ 0.3 s/stage | 2.0 / 3.0 s; transform overhead stayed 1.8 s | The transform "host overhead" was inter-dispatch idle from per-column kernels, not commit cost; only WP3's column-batched kernels removed it |
| WP2 pipelining | ≥ 6 s | 6.0 s against WP1, ~3–4 s net in that campaign | Paired against WP1 instead of WP0.5 (spec wording); later serial-vs-pipelined screens measured its true value at ~8 s |
| WP2b wider overlap | ≥ 4 s | −1.0 / +0.1 s, reverted | The only exposed CPU phase (accumulation) depends on the same segment's DATA commit; pulling the next commit forward puts GPU work on a GPU already busy ~3.4 s per ~1 s of CPU. The segment stage is GPU-bound |
| WP5 po2 21 | faster on 10M and 46M within the ceiling | serial po2 21: −1.0 / +3.0 / −6.8 s | po2 21 with pipelining exceeded the 32 GiB ceiling; the owner declined a 40 GiB large-memory profile |
| WP8-lite wrap overlap | ≥ 1.5 s | identity + wrap −0.5 s, complete ~0 | The native path already had an in-process witness and a raw-dump key load; under a second was overlappable and the pre-started child cost identity about as much CPU as it saved. The 17 s block is compute: identity_p254 ~12 s, gnark proving 3.5–4 s, witness ~1 s |
| WP6 pipeline archive | second process start ≤ 5 s to first dispatch | 8.27 s cold / 8.11 s warm; archive worth 0.16 s | The campaign cleared only the archive directory, so the driver cache stayed warm and the compile was 0.27 s in both arms; the 8 s was the bundle hash at every start plus harness steps. Re-scoped as WP6b, which removed the hash from the start path |

Every miss was closed with its best correct variant retained (or reverted when it was slower), its profile recorded, and the lesson written into the spec so it is not retried.

## Correctness and validation

Nothing cryptographic changed: pinned circuit and tap set, control IDs (`MIN_LIFT_PO2 = 14`), proving and verification keys, 256-byte Groth16 seal, and the original-pin verifier binary (`2bcdcb7f…`), whose dependency metadata carries no proving backend or local override. Every complete proof in the phase — several hundred across campaigns, lifecycles and diagnostics — was checked by that external verifier with wrong-image, modified-journal and corrupt-seal rejection; no proof was retried or excluded.

Each checkpoint ran: the build-time body check of the generated polynomial against the CPU-generated one (and the pinned CUDA generator's own `--check` for WP1b); the synthetic Metal-vs-CPU evaluator harness at po2 4/8/14/18/20 with buffer reuse, offsets and sentinels; real-witness CPU differentials on complete po2-14, po2-18 and po2-20 proofs (`RISC0_RV32IM_METAL_CHECK_CPU=1`, and `RISC0_RECURSION_METAL_CHECK_CPU=1` for WP4, 19/19 recursion checks); the Metal primitive suite on both the default and fallback kernel routes after NTT or hash changes, with Merkle outputs byte-identical on the same witness; a fallback-route proof; the 20-request lifecycle with cancellation, helper timeout, restart and shutdown, plus a recovery proof; input-limit controls; the CPU-only feature build and unit tests; and the three Linux feature paths compiled in the Colima VM. Every kernel route keeps an environment switch for bisection: `RISC0_RV32IM_TAIL_SPLIT`, `RISC0_SEGMENT_PIPELINE`, `RISC0_METAL_NTT_RADIX2`, `RISC0_RECURSION_METAL_SPLIT`.

The same ladder ran on the WP7, WP6b and upstream-merged workers on the last two days: 21-proof lifecycle and recovery suites, po2-14/18/20 segment and recursion differentials, nine input-limit controls, multi-segment pipeline cancellation, the corrupted-bundle rejection before helper spawn, the CPU-only builds and the three Linux feature paths in Colima. The post-merge regression check used the paired protocol with both exact worker binaries resident; the upstream merge raised the worker IPC protocol from 5 to 10, so a test-feature-only v5 adapter (`ZKMINER_TEST_LEGACY_PROTOCOL`, behind `cfg(feature = "testing")`) drives the sealed old worker from the new supervisor without touching production code or either binary's hash.

## Repository state

Both phase-2 branches are clean, committed and pushed to the personal forks as of 2026-11-07 (non-force pushes, after the paired check lifted the stop); every checkpoint has a sealed worker archive under `target/macos/phase2/` with a hashed manifest and the campaign, validation and Linux-check logs beside it.

| Repository | Branch | Tip | Notes |
| --- | --- | --- | --- |
| `dimitryB/zk-miner` | `codex/hemi-macos-metal-phase2` | `5a7b8cf` Record Phase 2 fork publication after paired lift | Published; carries WP7, WP6b and the merge of hemilabs `main` @ `af44acc`; evidence under `docs/macos/evidence/phase2-*` |
| `dimitryB/zk-miner` | `codex/hemi-macos-metal` | `9bc7466` | Phase-1 publication branch; not yet merged with phase 2 |
| `dimitryB/risc0` | `codex/hemi-macos-metal-phase2` | `881513e02` Merge hemilabs/risc0 release-3.0 at 54921e770 into Phase 2 | Published; phase-2 engine including WP6b, on top of upstream's current tip |
| `dimitryB/risc0` | `codex/hemi-macos-metal` | `dcbe9f4b9` | Phase-1 engine branch |
| `dimitryB/risc0` | `codex/upstream-final-segment-padding-r2` | `d354733d6` | #30 rebased onto `release-3.0`'s tip (one test-file append collision resolved), 37/37 execute tests, one verified 10M CPU proof; waiting for the owner to push it to the PR branch and un-draft |
| `dimitryB/risc0` draft PR #1 | `codex/hemi-macos-metal-wp6` | `11025fe0e` | WP6 archive; unmerged, its miss documented in the PR |
| `hemilabs/risc0` PR #30 (open, draft); #31 and #32 (closed) | `codex/upstream-final-segment-padding`, `-locked-circom`, `-native-helper-lifetime` | single commits on `8007a9972` | #30 is a real fix for upstream and is to be replaced by the r2 rebase; #31 and #32 only mattered alongside a Mac-backend PR and were closed unmerged on 2026-11-07 |

Sealed checkpoints: `baseline`, `wp1-baseline`, `wp2-baseline`, `wp3-baseline`, `wp4-baseline`, `wp5-baseline`, `wp1b-baseline`, `wp8lite-checkpoint` (the pre-merge worker, `625589c3…`), `upstream-merge-checkpoint` and `post-diagnosis-final-checkpoint` (the published worker, `c3428c37…`, manifest `c9e2ea29…`). The original-pin verifier binary is unchanged throughout (`2bcdcb7f…`). The miner's root `Cargo.lock` is restored after every local-engine build and keeps upstream's Git-source pins; the engine lock gained only already-present dependency edges. The retired fold-only path was deleted outright early in the phase (engine `fca0c0995`, miner `7d2dae1`), so the branch no longer carries dead code behind guards.

## Reproduction

From `/Users/dbur/dev/zk-miner` on the M5 Max, with the project-local toolchain (Rust 1.93.0, guest r0.1.88.0, protoc 36.2 under `target/macos/bootstrap`), the engine checked out at `881513e02` in `/Users/dbur/dev/risc0`, the native Groth16 bundle under `target/macos/native-wrap/assets`, and the dedicated Colima profile present only for the Linux feature checks:

```sh
export HEMI_MINER_DIR=/Users/dbur/dev/zk-miner
source scripts/macos/env.sh
export PROTOC=$HEMI_MINER_DIR/target/macos/bootstrap/protoc-36.2/bin/protoc
export CARGO_NET_OFFLINE=true
ENGINE_DIR=/Users/dbur/dev/risc0
ENGINE_REV=881513e02

# local engine override + worker build with tests; the build ends with one po2-14 proof that warms the driver cache
python3 scripts/macos/local-engine.py "$ENGINE_DIR" target/macos/local-engine.toml --revision "$ENGINE_REV"
python3 scripts/macos/build-worker.py --mode native-rv32im-metal --engine "$ENGINE_DIR" --config target/macos/local-engine.toml --revision "$ENGINE_REV" --tests

# correctness ladder on the built worker
python3 scripts/macos/test-native-worker.py --engine "$ENGINE_DIR" --worker-kind native-rv32im-metal --segment-hal metal --recursion-hal metal --segment-check-cpu --repeat 1 --proof-only
python3 scripts/macos/test-native-worker.py --engine "$ENGINE_DIR" --worker-kind native-rv32im-metal --segment-hal metal --recursion-hal metal --limit-controls --repeat 1 --proof-only
python3 scripts/macos/test-native-worker.py --engine "$ENGINE_DIR" --worker-kind native-rv32im-metal --segment-hal metal --recursion-hal metal --kernel-pressure --repeat 1 --proof-only

# controlled reading on the sealed published worker; the script waits out the 15-minute idle gap and checks thermal state
python3 scripts/macos/phase2-run-post-diagnosis-reading.py --last-campaign-result <previous result.json> --acceptance-build-manifest target/macos/phase2/upstream-diagnosis-acceptance-prebuild/acceptance-build.json
python3 scripts/macos/phase2-collect-post-diagnosis-reading.py target/macos/phase2/controlled-post-diagnosis/<run-id>

# paired comparison of two sealed workers (the older one driven through the test-only v5 adapter)
python3 scripts/macos/phase2-run-upstream-diagnosis-pairs.py --baseline-archive target/macos/phase2/wp8lite-checkpoint --expected-baseline-sha256 625589c35f6583bfaf4510a00d6f98dceb781b004064500ce38b44b42d659be7 --label a --last-activity-file <last build or campaign manifest> --acceptance-build-manifest target/macos/phase2/upstream-diagnosis-acceptance-prebuild/acceptance-build.json
python3 scripts/macos/phase2-collect-upstream-diagnosis-pairs.py target/macos/phase2/upstream-diagnosis-pairs/<run-id>
```

Runtime selectors for the validated configuration: `RISC0_SEGMENT_HAL=metal`, `RISC0_RECURSION_HAL=metal`, `ZKMINER_RISC0_WRAP=native`; po2 20, pipelining and the 8 GiB reserve (`ZKMINER_MACOS_HOST_RESERVE_GIB`, 8..64) are the defaults. Each package's paired campaign and collector are `scripts/macos/phase2-run-<wp>-pairs.py` and `phase2-collect-<wp>-pairs.py`, run against the sealed archive the script names; `phase2-host-snapshot.py` records the load, pressure and thermal state the protocol requires. Evidence JSON binds source hashes, worker and verifier SHAs, engine commit and raw run directories, so a reproduction is checked by hash rather than by eye.

## Open items and decisions

1. **Identity profile, then the WP8 decision** — one hour with the existing timers to learn how identity_p254's ~12 s splits between Poseidon254 hashing and transforms/evaluator. Hybrid WP8 (hashing on CPU, rest on the Metal recursion HAL; 2–3 days, low risk) if hashing is under about half; full WP8 (Poseidon254 kernel on BN254 limbs; about a week, high risk) otherwise. Identity + wrap is 17.5 s of the 49.5 s proof and the only route to the desirable 45 s line, which the owner has rated desirable, not required. Needs the owner's go-ahead; the published worker is the right one to profile.
2. **Owner actions on GitHub** — push `codex/upstream-final-segment-padding-r2` (`d354733d6`) to the #30 branch or open a fresh PR, mark it ready for review and answer the #33 note with the proof evidence (#31 and #32 were closed unmerged on 2026-11-07; their branches stay on the fork); close the fork's WP7 PR as merged by branch; leave the WP6 draft PR as the record of the miss or close it.
3. **Merge and publish** — fold `codex/hemi-macos-metal-phase2` into `codex/hemi-macos-metal` in both repos, and decide whether the Mac backend itself goes to hemilabs as an opt-in PR (#30 is the prerequisite; the diff is now against upstream's current tip).
4. **Housekeeping** — delete the `zkmac` account created for the cold reading if it is not wanted; find the roughly one-second tick in the completion path that quantizes complete latency (48 of 86 readings across the last four campaigns sit at x.5 ± 0.1 s, and the 46M pairs land at exactly ±1.0 s) and use the worker-reported total or the stage sum for any gate of a second or less.
5. **Not pursued, by decision** — a 40 GiB large-memory profile for po2 21 with pipelining (projected 52–55 s at 10M); GPU witness generation and accumulation (62k lines of generated code); any further CPU/GPU overlap scheme; the WP6 archive beyond its draft PR.

What the phase did not measure: any machine other than the 128 GiB M5 Max, cold start on another GPU generation (the fresh-account reading is one observation on this machine), and a direct comparison against an NVIDIA card; the 5070 Ti figures remain estimates from hemilabs' published per-segment timings on other GPUs.
