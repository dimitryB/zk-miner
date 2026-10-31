# Correctness-run measurements

These are observed CPU-path correctness timings, **not a controlled benchmark
or CPU/Metal comparison**. There was development/build activity and other host
load. No formal cold/warm separation, five-run-per-mode trial, thermal/power
sampling, sustained thermal test, or 20-request stability run was performed.
Do not infer a speedup, economic utility or unattended reliability.

Hardware: Apple M5 Max, 18 CPU/40 GPU cores, 128 GiB unified RAM, macOS 26.4.
No proving GPU kernels executed. Host threads: six Rayon; one active proof.
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
peak. The VM's configured 32 GiB is a ceiling, not observed use. Native worker
resource protection samples RSS every second but does not currently export a
high-water series; unknown measurements remain unknown. There is no GPU peak
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
load and independently verified outputs. The isolated one-stage hybrid harness now passes complete proof acceptance; worker hybrid mode remains disabled.


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
timings, sampled RSS, source/binary identities and retained failures. Five
alternating runs per mode, thermal controls and twenty-request worker stability
remain unrun. No performance improvement or production readiness is claimed.
