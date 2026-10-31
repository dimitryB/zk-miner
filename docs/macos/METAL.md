# First Metal differential milestone

The CPU checkpoints are committed locally on new `codex/hemi-macos-metal`
branches: miner `3503563849661f0e99b5ad9c7da7229bd561ea4f`, engine
`e274e19ee6e77bcd5704ca25c9880bd143f0783c`. The engine remains a descendant of
Hemi `f8dcbdf8124a3f1528146bb01d539c55545cbb3d`; the miner's dependency pin,
verifier, circuits and keys are unchanged. Nothing has been pushed.

## Toolchain provisioning

The machine still has standalone CLT/SDK 26.4, with no Xcode.app and no
`xcrun metal` compiler. Apple's supported offline path requires full Xcode:
CLT does not include xcodebuild. Apple's current compatibility table lists
Xcode 26.4.1 as supporting this macOS 26.4 host; newer Xcode versions can require
a newer OS, so installing an arbitrary latest version is inappropriate.

Official sources:

* [CLT and xcodebuild availability](https://developer.apple.com/documentation/xcode/installing-the-command-line-tools/)
* [Xcode/macOS compatibility](https://developer.apple.com/xcode/system-requirements)
* [Additional Xcode components](https://developer.apple.com/documentation/xcode/downloading-and-installing-additional-xcode-components)
* [Runtime source compilation](https://developer.apple.com/documentation/metal/mtldevice/makelibrary%28source%3Aoptions%3A%29?language=objc)

No Xcode package, privileged component, Rosetta installation, global toolchain
selection or global shell configuration was installed/changed. A native probe
successfully used the existing MTLDevice runtime compiler, dispatched one GPU
kernel, and checked four outputs on the Apple M5 Max. This permits first
retained-kernel tests without installing an offline compiler. It is not an
assertion that `xcrun metal` now exists.

For a later offline toolchain: obtain a compatible Xcode from Apple Downloads,
use per-command `DEVELOPER_DIR` to its Contents/Developer directory, inspect
`xcodebuild -checkFirstLaunchStatus`, and ask the owner before privileged
first-launch installation. Apple documents `xcodebuild -downloadComponent
metalToolchain -exportPath ...` and `-importComponent metalToolchain ...`;
confirm syntax with that Xcode's help. No such installation was attempted.

## Explicit runtime feature

`risc0-zkp/metal-runtime` selects runtime compilation of the retained Metal
headers and seven kernel sources. Normal CPU defaults remain CPU-only. Normal
`metal` without `metal-runtime` still uses the existing offline build path;
there is no automatic fallback after an offline compiler failure.

The sys build assembles exact retained source, preserves licenses and #line
markers, and rejects unknown local includes. Shared field headers come from
risc0-build-kernel's existing embedded sources. The compiled source is exposed
for SHA-256 provenance. Fast math is disabled. Every dispatched command is
waited for and must report Completed; a per-HAL counter increments only then.
CPU fallback operations do not increment that counter.

The worker still refuses forced Metal. These tests do not select a Metal
segment prover, accelerate a complete proof, enable marketplace jobs or change
verification semantics. Prefix products, scatter and zeroize/copy-slice CPU
fallbacks are not represented as GPU acceleration. The incomplete optional
`metal_prefix_products` feature stays disabled.

## Reproduction

After reviewing the local engine revision/diff, from the miner root:

```sh
export HEMI_MINER_DIR=/Users/dbur/dev/zk-miner
source scripts/macos/env.sh
ENGINE_DIR=/Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
ENGINE_REV=$(git -C "$ENGINE_DIR" rev-parse HEAD)
sh scripts/macos/build-metal.sh "$ENGINE_DIR" "$ENGINE_REV"
python3 scripts/macos/test-metal.py --engine "$ENGINE_DIR" --mode basic
python3 scripts/macos/test-metal.py --engine "$ENGINE_DIR" --mode all
```

The build uses a separate target directory and lockfile. Source metadata must
contain exactly the local intended engine packages and the explicit runtime
Metal feature; the CPU source guard rejects this graph. The source-override
generator now accepts a committed descendant only with an explicit full
`--revision`, validates HEAD equality and baseline ancestry, and still rejects
upstream/wrong revisions. No broad dependency update is performed.

The test runner bounds time to 180 seconds and samples native RSS against a
4 GiB ceiling. Cases use small buffers; RSS is not a Metal allocation peak and
these limits do not establish a production GPU memory budget. Each attempt
gets a fresh directory and a ledger entry, including failures. Actual outcomes
and exact artifacts are recorded below when execution completes.

## Executed outcomes

Final committed engine: `130f98fb91f6c5ccb1d243a498bf303c760c6388`.
The final sealed build and run used this exact clean engine checkout; no working
diff. Actual commands were the build/test commands above, with mode `all`.
The build manifest binds the binary hash to engine path/HEAD/diff, harness
source/lock, generated overrides and resolved metadata. A changed binary or
source state is rejected before GPU execution. Matching inputs, an incorrect
engine path, and an altered temporary binary were tested explicitly.

* Initial retained-kernel basic addition: 257 elements matched CPU, one actual
  GPU completion.
* Initial boundary attempt failed because zero-length addition still dispatched.
* After buffer fixes, run `e9f0a59558fb4647b088f6a02f014b13` passed 188 cases,
  then failed empty-column Poseidon hashing (rows=1, columns=0). It completed
  371 commands, of which 370 belonged to successful cases; the failing command
  is not counted as correct work.
* After the Poseidon fix, run `8eedc773ef5d4f11a3c79a14bdf22a89` passed all 222
  cases with 404 completed commands.
* Final source-bound run `54482afeee90464dbca9fbc8183d5b06` again passed **222
  cases and 404 GPU completions** against committed engine `130f98fb9`.

Final compiled kernel-source SHA-256:
`2fc1eb3c8a00e4fb40f5055dbaef08074e4c16cbc4e06b89de2bfe1e57d1bcd0`.
Full case records are in `evidence/metal-final-cases.json`; provenance, failures,
source guards and CPU regression evidence are in `evidence/metal-validation.json`.
Raw per-attempt stdout/stderr and build logs remain under `target/macos`.

| Cases | Count | Evidence checked |
|---|---:|---|
| Add / copy | 43 / 42 | CPU equality, modular-u64 addition oracle, independent offsets, sentinels |
| Multiply by factor | 36 | Rust field and independent u64 modular oracle, edge factors |
| NTT roundtrip / expansion | 14 / 16 | CPU intermediates, original input recovery, offsets/sentinels |
| SHA rows / folds | 24 / 10 | Pinned CPU hash suite, empty and boundary columns, tree outputs |
| Poseidon rows / folds | 24 / 10 | Pinned CPU hash suite, empty and boundary columns, tree outputs |
| Empty buffers / range rejection and reuse / allocation lifetime | 1 each | No empty dispatch, five invalid host ranges rejected, reuse succeeds, one allocation across clones |

Empty and ownership-only cases report zero GPU work. Every nonempty kernel
case records completed commands; default CPU fallback methods are excluded.
Largest arithmetic data length is 4096 elements. The subsecond suite runs are
correctness checks, not timing comparisons. RSS samples caught only startup
and are unsuitable as peak-memory evidence.

## Fixes and compatibility

Changes are restricted to the opt-in Metal path: logical slice reads and raw
pointers honor offsets; range/size arithmetic rejects overflow; empty buffers
retain safe physical backing but empty operations submit no GPU work; Arc
tracks a shared allocation until its final HAL owner drops. Command completion
is checked before CPU reads. Host views are used serially after completion;
no asynchronous overlap or new buffer cache is introduced.

Poseidon now checks `col_size == 0` before the empty-input permutation, matching
pinned CPU unpadded_hash. No field constants, permutation rounds, circuits,
trusted setup, keys, verifier parameters or proof encoding changed. Existing
CPU Buffer::to_vec slice behavior was observed but not changed; tests read the
CPU reference through its view contract instead of repeating that defect.

The CPU-only harness was rebuilt with Metal absent from its feature closure.
Regression run `f738364b836b4e3a902203f6151f6276` completed a genuine Fibonacci
Groth16 proof in 166.672 seconds, accepted by the unchanged verifier, with
wrong-image/journal/seal rejection. The dedicated helper VM was stopped after
testing; no helper remained. Root miner/engine lockfiles stayed unchanged; the
isolated Metal lock introduces no dependency versions absent from the miner lock.

## Remaining scope

This establishes initial CPU/Metal primitive differentials, not a validated
hybrid prover (Gate 3 remains open). Next: connect one compatible generic Metal
stage to the pinned segment prover, declare that split explicitly, observe its
GPU commands, and require a full independently verified proof. Keep recursion,
identity and wrapping on their current CPU/helper implementations initially.
Add stage-specific buffer budgets, error-completion tests and additional FRI/
circuit coverage before exposing a worker mode. No production memory budget,
GPU-fault recovery, sustained stability, speedup or profitability is established.
Offline tooling still needs compatible Xcode provisioning and explicit approval
before any privileged setup. Network acceptance remains unrun and unauthorized.
