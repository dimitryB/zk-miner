# Executed validation — 2026-10-31

Gate 1 passed: genuine complete CPU-path proofs, including final Groth16,
accepted by an independently built unchanged Hemi verifier. Gate 2 passed:
real miner WorkerPool discovery/IPC/admission, multiple complete proofs,
wrapping timeout/cleanup, restart and shutdown. This is a bounded experimental
local CPU slice with an emulated wrapping helper. Gate 3 hybrid Metal and
network acceptance are **not run**. The whole specification is not complete.

## Source/build evidence

The source/hardware audit was written before broad implementation. Original
Hemi and upstream checkouts remain clean and unchanged. The implementation
engine worktree uses the exact miner pin; upstream HEAD was never substituted.
The original CPU check failed because `xcrun metal` was invoked on a machine
without the Metal compiler (`target/macos/check-original.log`). Explicit
Metal feature propagation fixed this; CPU check/build then succeeded.

Actual successful build commands (from the miner root after sourcing env.sh):

```sh
cargo +1.93.0 build --release --features prove --manifest-path tools/macos-proof/Cargo.toml --config target/macos/local-engine.toml --target-dir target/macos/cpu
python3 scripts/macos/build-guest.py fibonacci --config target/macos/local-engine.toml
python3 scripts/macos/build-guest.py sha256-chain --config target/macos/local-engine.toml
python3 scripts/macos/build-guest.py segment-boundary --config target/macos/local-engine.toml --reset-lock
sh scripts/macos/build-verifier.sh
python3 scripts/macos/build-worker.py --config target/macos/local-engine.toml --engine /Users/dbur/.codex/worktrees/hemi-macos-cpu/risc0
cargo +1.93.0 build --locked --release -p zkminer-cli --bin zkminer --target-dir target/macos/miner
```

`file` identifies harness, worker, miner and verifier as Mach-O arm64.
Source metadata verifies 17 local worker engine packages and 10 unchanged
Git verifier packages. Feature trees are in `target/macos/{worker,verifier}-tree.txt`.
The worker build saves its local lock and restores the root Git lock; its only
root lock additions are existing libc/risc0-binfmt dependencies, with
no version upgrades. Verifier tests use original circuit/verifier parameters.
`zkminer --help` is the only CLI operation used after the native miner build.
A final rebuild initially used the wrong package name `-p zkminer` and failed;
the actual package is `zkminer-cli`, binary `zkminer`.

## Genuine proof attempts

Every attempt, including failures, is recorded in `target/macos/runs/results.jsonl`.
Full artifacts include intermediate receipts, complete receipt/seal/journal,
proof/verification JSON and stage logs. Independent expected journals are
computed from arithmetic/hash inputs, not copied from generated receipts.

| Run prefix | Workload / po2 | Outcome |
|---|---|---|
| d5c3df7d | Fibonacci / 18, raw ELF | Failed: malformed ProgramBinary; fixed by packing authentic ELF + pinned V1COMPAT kernel |
| facaf950 | Fibonacci(20) / 18 | Complete Groth16, independent positive and 3 negative checks passed |
| 9cc541fc | Fibonacci(20) / 18 | Second consecutive complete proof, same independent checks passed |
| d811a606 | SHA-256-chain(10) / 18 | Complete Groth16, independent checks passed |
| 03aeaeb7 | Segment boundary / 14 | Verified 22-segment composite, then missing `lift_rv32im_v2_13.zkr`; incomplete proof, failed |
| f2bd8b66 | Segment boundary / 18 | Complete independently valid Groth16, but only one segment; failed multi-segment coverage |
| a519537b | Segment boundary / 16 | Five segments, complete Groth16, independent positive and 3 negative checks passed; 241.9996 seconds |

Fibonacci image ID:
`6f8159836d1511443408b77a590c2920fce31042249e71551bf998e7c7ea1466`;
expected journal `6d1a000000000000`. All successful independent checks reject
wrong image, altered journal and corrupted seal. SHA-256 expected journal:
`43f66f29297b60a3a6a09f54f383bf01a44d6c3baa727796983637a32fcb0583`.
The pinned recursion archive supplies lift sizes 14..24, not 13. No circuit,
key, archive or verifier substitution was made to bypass that limitation.

Smoke commands executed:

```sh
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 2 --workload fibonacci
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 1 --workload sha256-chain
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 1 --workload segment-boundary --po2 14
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 1 --workload segment-boundary --po2 18
python3 scripts/macos/smoke.py --context colima-hemi-macos-cpu --repeat 1 --workload segment-boundary --po2 16
```

## Real worker acceptance

The environment/command is reproduced in SETUP.md. Executed explicitly:
`cargo +1.93.0 test --locked -p zkminer-prover --test macos_worker -- --ignored --nocapture`.
Result: **1 passed, 0 failed, 0 ignored**, 744.21 seconds. Evidence directory
`target/macos/worker-acceptance-2`, log `target/macos/test-worker-acceptance-2.log`.
Four actual raw worker Groth16 outputs were independently verified, including
all three tamper checks each time. No mocked prover participates in this test.

Observed: forced Metal rejected; missing helper context rejected; capabilities
report canonical risc0, no GPU and two supported fixtures; protocol mismatch
rejected; real Execute returns cycles; invalid guest fails; three consecutive
Prove requests succeed; the 70-second deadline kills a request after a real
helper appears; no helper survives; the next request restarts and succeeds;
shutdown leaves no helper. One conservative memory-driven worker recycle
occurred before the third successful request. This is three consecutive
successful requests, not three proofs in a single uninterrupted worker process.
Proof times were 164.478, 159.875, 167.932 and 164.781 seconds.

The first acceptance attempt failed during startup because free-only memory
accounting ignored the filesystem cache. The corrected conservative estimate
is documented in DESIGN.md. That failure log is retained. During the passing
suite, process cleanup still used the conservative fallback when `/proc` was
absent; subsequent native proc_listpids cleanup was separately compiled and
tested with real owned processes. The passing full suite is not represented
as a run of those later process-enumeration edits.

## Focused checks and failures retained

Executed commands and latest results:

* `cargo +1.93.0 test --locked -p zkminer-prover-protocol --lib -- --test-threads=1`:
  31 passed, 0 ignored. Includes framing/roundtrips, EOF/oversize checks and
  bounded child-process execution. Linux-only ETXTBSY runtime test is compiled
  only on Linux; it is not a proof or a passed macOS test. Darwin errno mapping
  now tests 35 as EAGAIN and rejects 11 (EDEADLK) as transient.
* `cargo +1.93.0 test --locked -p zkminer-prover --lib worker::tests`:
  14 passed, 0 ignored; live descendant detection/kill, zombie/reap semantics,
  process-group ownership guards and synthetic proc regression fixtures.
* `cargo +1.93.0 test --locked -p zkminer-prover --lib memory::tests`:
  12 passed, 0 ignored; actual host observations plus admission arithmetic.
* `cargo +1.93.0 test --locked -p zkminer-prover --lib macos::tests`:
  2 passed, 0 ignored; native resource/PID identity and owned descendant cleanup.
* Acceptance guard experiments reject dev mode, Bonsai, skipped build/kernel
  settings and an upstream HEAD supplied to the local-source generator.
  See `evidence/guard-tests.json`.

Earlier focused runs failed on Linux zombie-group assumptions, Linux script
ETXTBSY behavior, reserve-specific test constants and a 150 ms new-script
execution timing assertion; poisoned test mutexes caused follow-on failures.
The timing test now invokes the installed shell directly with the same bound;
newly written script startup was the unrelated latency. Native enumeration
also exposed an exit race where BSD info disappears before a process leaves
the group; it now conservatively retains that enumerated member. All those
logs remain under `target/macos/test-*.log`; no failed proving is averaged away.

## Remaining gates / unexecuted tests

Metal compilation, kernel execution, CPU/Metal differential comparisons,
Metal-buffer lifetime/error tests, controlled five-run-per-mode benchmarking,
20-request extended stability, and Keccak-specific workload coverage are not
run. CPU benchmark/calibration marketplace ranking is not established. Full
CUDA/ROCm builds/runtime tests are not available on this Mac; platform guards
preserve their source paths but do not substitute for Linux CI. Contract-adapter
or deployed verifier acceptance, wallets, network jobs and transactions are
not run or authorized. The next Metal prerequisite is a compatible compiler;
the next stability prerequisite is the extended sequential suite with current
resource/lifecycle code, preserving every failure.

Final real-worker preflight check:
`cargo +1.93.0 test --locked -p zkminer-prover --test macos_worker native_worker_mode_preflight -- --ignored --nocapture`
passed (1 passed, 0 ignored; full proof test filtered out). This tests auto CPU
fallback, invalid mode and dev-mode rejection using the final process cleanup
and Darwin errno code. It is a preflight/lifecycle check, not another proof.

## Review artifacts

`evidence/validation.json` contains all seven harness attempts, four worker
proof verifications, hardware/guest provenance, helper artifact hashes and final
binary/lockfile/metadata hashes. `evidence/measurements.csv` is the timing ledger.
`evidence/hemi-engine.patch` is the separate six-file engine diff. The complete
miner diff, including new files, is saved locally as `target/macos/miner.patch`;
it is uncommitted and is not embedded recursively in the evidence bundle.
Raw receipts, seals and full logs remain under `target/macos` (about 10 GiB
including all toolchains/builds). Initial provenance remains in baseline.json.
Final binary hashes describe the final rebuild, not a claim that the earlier
full acceptance suite used later platform-test fixes. No source was committed.
