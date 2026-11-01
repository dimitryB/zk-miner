# Controlled CPU/Metal Fibonacci comparison — completed 2026-11-01

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
