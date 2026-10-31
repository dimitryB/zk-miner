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

For the next formal benchmark, first run extended stability, then define an
identical supported guest/input/po2 and resource envelope for CPU and an actually
validated hybrid mode. Warm up and collect at least five complete alternating
runs per mode; record cold start, failures, actual stages, thermal/background
load and independently verified outputs. The isolated hybrid harness and subsequent native-worker milestone below pass
complete proof acceptance; extended stability and controlled measurements remain open.


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
