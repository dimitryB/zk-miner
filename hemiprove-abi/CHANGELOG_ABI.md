# ABI Changelog — 2026-10-28 vs. any prior Hemi testnet deploy

This document lists every **wire-format** change since the last time your indexer / frontend / prover daemon regenerated its ABIs. Read this before repointing at the new deploy. Anything not listed here has stable signatures.

Categories: 🔴 **breaking** (client rebuild required), 🟡 **new** (opt-in, additive), ⚪ **cosmetic** (documentation, gas).

**Standing rule: router enums are append-only and frozen in what callbacks and views carry.** The members of every router enum that reaches a callback or a view (`JobStatus`, `VerifierTrust`, `AdapterStatus`, `CycleAttestationMode`, `CurveType`; `ProofContext`, `JobStatusView`, `getJobStatusView`, `verifiedResults`) keep their names and ordinals: a member is never removed, renamed or reordered. A new state either maps onto an existing member in every existing view and context, or ships behind a new selector; a new member appended to a type an existing view or callback returns is 🔴 breaking for deployed callbacks and is listed here as such. The ABI files carry enums as `uint8` (`scripts/gen-abi.sh --check` cannot see a member change), so every entry spells members out by hand, and `test/fixes/EnumOrdinalPin.t.sol` pins the ordinals. Consumers decode them tolerantly: read the raw `uint8`, treat a value you do not know as "not ended" or "not trusted" (never as success), and never let an unknown value revert a path that must complete (the `src/base` templates do this: `_jobStatusRaw`, `UnknownJobStatus(uint8)`, the verbatim calldata forward of `onProofVerified`).

---

> **Deployment.** Every Hemi testnet (743111) stack deployed before the new deployment is **abandoned**, including all eight BatchCycleCommittee proxies (`deployment-hemi-testnet.csv`). Nothing is migrated from them. Integrators repoint at the new deployment, which follows the mainnet procedure (DEPLOY_RUNBOOK.md, "Fresh deploy"). Its addresses are **TBD until the 743111 dress rehearsal completes**, and will then be in the "New deployment" rows of `deployment-hemi-testnet.csv`. The ABI files in this directory match the commit; no address in this changelog is a live deployment.

## 🟡 Additive — 2026-10-29 (743111: TestHemiToken, an owner-minted test HEMI)

No change to a deployed protocol contract and no pin move.

- **New:** `src/testnet/TestHemiToken.sol`, the 743111 stand-in for HEMI: "Test HEMI", symbol tHEMI, 18 decimals, an
  OpenZeppelin ERC-20 with no transfer hooks, fees or pause. Only its owner mints (`mint(address,uint256)`,
  `mintBatch(address[],uint256[])`, `LengthMismatch` on unequal arrays); ownership moves in two steps
  (`transferOwnership`, `acceptOwnership`). The constructor `(owner, initialHolder, initialSupply)` mints the initial
  supply and reverts `MainnetDeployRefused` on chain 43111. Integrators use it like any ERC-20 (approve the router and
  Staking); testers get tokens from the owner. The old free-mint tHEMI `0x60525eEBB1BD630Fb093d433675E28221DE1E698` is
  not the new deployment's token.
- DEPLOY_RUNBOOK.md pre-deploy step 2 deploys it before Day 0 from a key other than the deployer. Its tests:
  `test/testnet/TestHemiToken.t.sol` (12, including Deploy.s.sol's HEMI check accepting it on 743111).

## ⚪ No wire or ABI change — 2026-10-29 (743111: the role Safes may share signers with SHARED_SIGNERS=1)

No `src/` change and no pin move; scripts, docs and tests only.

- **Why.** The testnet deployment may be run by a team too small for five independent signer sets. The contracts and
  the deploy scripts require each role to be its own Safe (different addresses), and only
  `scripts/check-safe-config.sh` compares the Safes' signer sets.
- **`scripts/check-safe-config.sh --snapshot`** takes `SHARED_SIGNERS=1` on 743111 only (refused on every other chain)
  and writes `"sharedSigners": true` into the signing record. With it, an overlap between the role Safes' signer sets
  is a `NOTE` instead of a `DRIFT`; two roles on one Safe stay a `DRIFT`, and every other check applies. A record
  carrying `sharedSigners` on any other chain is a `DRIFT`. `scripts/safe-tx.py record` passes the variable through.
- **DEPLOY_RUNBOOK.md**: "Shared signers on the 743111 testnet" under "Roles" (a different `--salt` per Safe, the L1
  guardian's own salt, the rehearsal report notes that independence was not rehearsed), the `record` command and the
  pre-deploy abort list. `scripts/test-check-safe-config.sh` 115 checks.

## ⚪ No wire or ABI change — 2026-10-29 (the 743111 deployment procedure: verify refuses a mode change; the Day-0 registration confirmed on chain)

No `src/` change and no pin move; scripts, docs and tests only.

- **TransferOwnership `verify`** refuses a Registry `CycleAttestationModeUpdated` log since the deploy, like every other
  owner setter, also when the mode is set back to where it was registered: every job submitted in between keeps the mode
  it snapshotted. A registration logs its mode in `AdapterRegistered` and still passes, so the deployer's Day-0
  registrations on 743111 do. A rehearsed Committee-mode enablement through `setCycleAttestationMode` now fails `verify`
  for good, like the other owner-setter drills.
- **`verify`'s launch gates** no longer rest on the current `name()` alone: on 43111 an adapter whose `nativeVerifier()`
  has OpenVM's pinned v2.0.2 runtime, or whose `name()` is the older builds' "OpenVM", is the OpenVM adapter (refused;
  on 743111 refused under any psId but `OPENVM_HALO2_V1`), and an adapter named "RiscZero-PoVW" is the PoVW adapter.
- **TransferOwnership phases on 43111 and 743111** require the deployment record, as `verify` does. A host without it
  (a signer building the Phase 2/3 file) passes `NO_DEPLOY_RECORD=1`; the phase then runs unbound and prints a NOTE.
- **`DeployVerifiers.s.sol` `registerByDeployer()`** binds `REGISTRY` to the deployment record (`DEPLOY_RECORD`, else the
  default path of `HEMI_PROVE`) and prints the Registry it acts on. A run that registers prints each registration as
  simulated (forge prints before it sends) and no `EXPECTED_ADAPTERS`; the re-run without `--broadcast` that reads
  "ok already registered" for all three ids prints the value to export (DEPLOY_RUNBOOK.md "Day 0", step 3a.4).
- **`Deploy.s.sol`'s printed next steps** on 743111 name step 3a (`registerByDeployer()`) and the `EXPECTED_ADAPTERS`
  export.
- **DEPLOY_RUNBOOK.md**: step 3a takes `EXPECTED_ADAPTERS` from the confirming re-run; a Day-0 incident rule (from
  step 3a to Day 1, step 10, a pause and never a disable, blacklist, switch or mode change); signers' hosts and the
  record; a rehearsal-order item and exit criterion 11: a dry run of the 43111 Day-1 registration ops on a 43111 fork
  before mainnet. `deployment-hemi-testnet.csv` has one row per launch verifier and adapter.
- **Harness.** `scripts/test-fresh-deploy.sh` 429 checks.

## ⚪ No wire or ABI change — 2026-10-29 (the L1 hatch's checks: the portal's pause, an L2 outage, track's verdict)

No `src/` change and no pin move; scripts, docs and tests only.

- **`scripts/l1-hatch-tx.py check`.** The portal's `paused()` is a NOTE, not a refusal: OptimismPortal 2.5.0 (both L1
  portals run one implementation) pauses withdrawals only, accepts deposits while paused, and the derivation includes
  them. In its place an `eth_call` of the exact `depositTransaction` from the L1 sender (the L1 guardian Safe, or the
  pauser key) at the L1 head must succeed; it also refuses any other L1-side revert (a gas limit above the portal's
  resource-metering cap, for one). When the L2 endpoint does not answer (nothing listening, an HTTP error, a JSON-RPC
  error that is not the call's own revert), the L2 checks not yet made become NOTEs and `check` concludes on the L1
  half: exit 0, with an OK line that says the L2 half was not read. An answer that is wrong still refuses.
- **Aliasing.** The tool follows the portal's rule: the L2 sender is aliased exactly when `msg.sender != tx.origin`. A
  pauser key that carries an EIP-7702 delegation is a NOTE (its own deposit arrives unaliased); a pauser with other
  contract code, and an `L1_GUARDIAN` that carries a delegation, are refused.
- **`scripts/l1-hatch-tx.py track`** requires `ROUTER` and judges every deposit against `ROUTER`, `L1_GUARDIAN` and
  `RELAY`: only `emergencyPause()` or `emergencyCancelPendingUpgrade(opHash)` to `ROUTER` from `alias(L1_GUARDIAN)`,
  and `relay.pause(ROUTER)` to `RELAY`, can pass. Any other deposit (another target, call, sender or relayed target) is
  exit 2 from the L1 receipt alone, whatever landed. A pause passes only with the router's own `Paused` log in its L2
  receipt; `RPC` unset is exit 3 (the landing unchecked), no longer exit 0. The router's `paused()` at the L2 head is
  printed last.
- **DEPLOY_RUNBOOK.md**: "Router guardian and its L1 hatch" (the portal's pause and aliasing rule, step 1's sign-nothing
  conditions and the L2-outage rule, step 3's exit codes, the drill's expectation) and drill (c) (step 1 and the pass
  condition).
- **Harness.** `scripts/test-l1-hatch-tx.sh` 76 checks; it takes `PORT_RANGE`.

## ⚪ No wire or ABI change — 2026-10-29 (the schema example copies its reviewed schema at the pin and decodes in one pass)

No change to a deployed contract and no pin move: the example below is not deployed and has no ABI file; docs and
tests otherwise.

### 🟡 Example: `SchemaValidatedCallback`
- **Why.** The example compared the registry's `getIOSchemaHash(programId, 0)` with its pin at every `requestProof`.
  Once the reviewed copy is held, that comparison adds nothing to what a result is decoded with, and it let the
  entry's controller (first-come registrant or family admin) refuse every later request with one `setIOSchema`
  (another schema, or none) until the example's code changed. Its callback also made one self-call per field, each
  re-parsing the schema: the example's own Oxygen schema (8 ABI fields) needed about 570,000 gas, over the 500,000
  `MAX_CALLBACK_GAS_LIMIT`, so such a job paid the prover and stored nothing.
- **New:** owner-only `pinSchema(bytes32 programId, bytes schema)`: stores the reviewed bytes (unless a copy is held)
  and pins their keccak256, with `SchemaHashPinned`; the registry is not read.
- **Changed: `pinSchemaHash(bytes32 programId, bytes32 schemaHash)`** takes the copy: without one held for
  `schemaHash`, it copies the registry's current output schema once, only if it hashes to `schemaHash`, and otherwise
  reverts `SchemaHashMismatch(bytes32 programId, bytes32 pinned, bytes32 actual)` (`actual` 0: no schema) with nothing
  changed; 0 still unpins. Both pins revert with SchemaLib's error for bytes it cannot parse.
- **Changed: `requestProof`** never reads the registry: it reverts `SchemaNotPinned(bytes32)` without a pin and no
  longer reverts `SchemaHashMismatch`. Results decode with the stored copy of the job's hash, as before.
- **Changed: the callback** decodes every field with one `decodeWithSchema` self-call (`FieldDecoded` unchanged: the
  same fields, order, names and values). Measured with cold state under the cap: Oxygen about 330,000 gas on a
  program's first result and 110,000 after; a first result settles with up to 13 ABI fields with 10-byte names, 11
  with 63-byte names.
- **NatSpec.** `jobSchemaHash` is deleted when the job's result is handled, or when the job is cleared after it ended
  unhandled (`cancelProof`, `clearEndedJob`). The example does not read a program's registry status (a revoked
  program's jobs still settle; the owner unpins), and says why.
- **Docs.** `IO_SCHEMA_INTEGRATION_GUIDE.md` ("Pin the schema you decode with": the copy is taken at the pin and the
  registry never read at a request; a `requestProof` argument as the pin only on an owner-only request; registry status
  advisory; "Decode every field in one call"; the `MyConsumer` sample stores the reviewed bytes at deploy) and
  `abi/INTERACTION_GUIDE.md` §5 ("Authenticate every callback").
- **Tests.** `test/fixes/SchemaCallbackOwnJobs.t.sol` 22, `test/fixes/SchemaCallbackRegistryScenarios.t.sol` 14 (the
  real ProgramRegistry: swaps around a request and around a pin, a controller who keeps another schema, revocation and
  re-registration, re-entry at the pin), `test/fixes/SchemaCallbackGas.t.sol` 4 (Oxygen, 96 configurations of 5-7
  fields with names of 1-63 bytes, the limits), `test/invariant/SchemaCallbackPinInvariant.t.sol` 3 (a stateful model
  against the real registry and router: 9,600 invariant calls, 12 walks of 1,500 steps and one of 3,000).

## ⚪ No wire or ABI change — 2026-10-29 (`verify` through both endpoints; the setups behind each proof system; locks claimed in a catch-up burst; doc corrections)

No pin move: scripts, docs, tests and two source comments (every deploy-profile runtime and initcode byte-identical).

- **`verify` through both endpoints on 43111.** Since its scans are topic-filtered (2026-10-28, "TransferOwnership `verify` reads only owner events"), `verify`'s no-log checks no longer share the must-have scan, as the 2026-10-21 entry states they do: each event has its own query, so an endpoint that wrongly answers `[]` to one event's topic query hides that event's owner actions from a single run (the ProgramRegistry's `FeesWithdrawn`, `ProgramRevoked`, `ProgramKeyReserved` and `ExpiredBondClaimed` among them, which no head read shows). On 43111 `verify` runs a second time through the independent `RPC_2`, with `FOUNDRY_NO_STORAGE_CACHING=true` (forge keys its fork state cache by chain and block, not by endpoint), and both runs must pass (`DEPLOY_RUNBOOK.md`, pre-broadcast gate G2 and "Signing on 43111"). `scripts/check-runbook-commands.sh` requires that run in a document that runs `verify`, and the variable on it.
- **`LOG_CHUNK`** of TransferOwnership `verify` is a decimal block count: `script/lib/VerifyLogScanner.sol` refuses anything but digits (it read `1e3` as 1,000 and hex as its value) and 0, as before.
- **The setups.** `DEPLOY_RUNBOOK.md` threat-model residual 18 lists the setup behind each governance id. `SP1_PLONK_V1` has no circuit-specific ceremony: it rests on a universal KZG reference string, whose vendored points (`G1_SRS`, `G2_SRS_1` of `PlonkVerifier.sol`) equal the transcript of the last participant of Aztec's Ignition ceremony. One SP1 program settles under both SP1 ids, so a consumer that accepts either id rests on both setups: to rest on one, it accepts only that `proofSystemId` (`abi/INTERACTION_GUIDE.md` §3 "One SP1 program, two ids" and §5).
- **Corrections.** The 2026-10-28 PLONK entry and the guide say that a FULL `expectedJournalHash` hashes the job's `proofSystemId` (a hash made with the other SP1 id or a retired id never settles), that a submit under a retired id with the proof-nonce marker reverts `InvalidJournalBinding(6)` before `AdapterNotActive`, and that `scripts/safe-tx.py write register-adapter` refuses a non-canonical id unless `--allow-new-psid`. The guide's SP1 seal checks list the native verifiers' own reverts (`InvalidExitCode`, `PublicInputNotInField`, the PLONK verifier's `Error(string)` such as "wrong proof size").
- **The router's stall page.** `STALL: re-pause now, then extend` (`scripts/bcc-monitor.sh` stall-check) and the runbook ("Lifting a router or Registry pause", "Monitor reference", "Stall procedure") say what happens to a lock claimed after the lift's anchor and before the re-pause: in the extension file it is reset to its claim moment; left out, it is slashable once the lift's grace ends if its deadline passed in the burst.
- **`scripts/check-verifier.sh`** prints the wanted revert of a seal the verifier accepted as its error name and hex, not as a Python bytes literal.
- **Source comments.** `HemiProveBase._proofNonce` and `getProofNonce` say that the nonce word is public input 4 of both SP1 wrappers.
- **Harnesses.** `scripts/test-check-verifier.sh` 45 checks (now under `PORT_RANGE=<lo>-<hi>`), `scripts/test-bcc-monitor.sh` 396, `scripts/check-runbook-commands.sh --self-test` 149 expectations; forge `test/script/VerifyLogScannerChunk.t.sol` (4).

## ⚪ No wire or ABI change — 2026-10-29 (the reference daemons' claim retry set)

No `src/` change and no pin move; scripts, docs and tests only.

- **Why.** A claim refused for free stake waited in the claim retry set until the key's available collateral rose
  above the mark taken at the refusal, which normal operation never brings about (a settled lock returns the
  collateral to its peak; a slash or a release penalty lowers the peak for good). The entries of jobs that had ended
  stayed, and a requester could fill the set at no lasting cost (jobs of the daemon's own descriptor at 500%
  collateral, cancelled for a full refund once refused): every later job that needed the set (a ramped job first
  seen below the reservation, every `job-submitter.sh` job among them) was dropped with an `ERROR_LOG` line and no
  page.
- **`script/lib/prover-lib.sh`** (both daemons): a full set, or a caller's full share, evicts its oldest
  stake-marked entry for the new job (an `ERROR_LOG` line names it); each retry pass reads the job of one
  stake-marked entry, in turn (one `jobs()` call, no claim simulation), and drops the entry once the job is no longer
  Open; `PAGE: RETRY-FULL` (at most hourly; every refused job in `ERROR_LOG`) when a full set has no stake-marked
  entry to evict. A poll whose new-job scan stopped for a lock entering its `LOCK-LATE` window runs no retry pass:
  the next poll serves that lock first. The bound on a lock's wait is stated with the retry pass (the scan's budget
  plus one job, then the retry pass's budget plus one entry).
- **`script/demo-prover.sh`**: in fixture mode without `FIXTURE_SEAL`, an SP1 consumer's seal is its wrapper's
  4-byte prefix and 32 zero bytes (mock verifiers only), not `0x0000`, which the wrapper check refuses; an SP1
  consumer with the default `PROVE_CMD` is refused before the RISC Zero startup build.
- **Docs.** `abi/INTERACTION_GUIDE.md` §3a and DEPLOY_RUNBOOK.md ("Rehearsal provers", the daemons' residuals): the
  eviction and the liveness read, `RETRY-FULL` and what to do on it, the lock bound with the retry pass.
- **Harness.** `scripts/test-prover-daemons.sh` 505 checks: §1 (the eviction, `RETRY-FULL`, the liveness read, the
  retry pass after a `LOCK-LATE` stop), §43 (the retry pass at its default budget inside the lock bound), §44 (256
  jobs of 16 callers refused for free stake and cancelled at the shipped caps: later jobs claimed; `RETRY-FULL`), §45
  (a release penalty below the marks), §46 (demo-prover and SP1 consumers).

## ⚪ No wire or ABI change — 2026-10-28 (the guardian vetoes one op per file; two or more with one cancel-all)

No `src/` change and no pin move; scripts, docs and tests only.

- **Why.** `GT.guardianCancel` reverts for an op that is no longer pending, and a Safe batch reverts whole (GS013, its
  nonce kept). A compromised owner could therefore make a guardian's multi-op veto file revert at will by cancelling
  one of its ops just before it landed, then propose that op again, while the other ops reached their execute.
  `GT.guardianCancelAll` cannot be made to revert.
- **`scripts/safe-tx.py write veto`** takes exactly one `--op-hash` for the GT guardian too; two or more are refused
  (exit 2, no file), naming `write veto-all`. A veto whose op is no longer pending is refused at `build` / `exec` with
  the same advice.
- **`scripts/check-proposals.sh`**: the VETO summary says one op per veto file and one `veto-all` for two or more GT
  ops.
- **DEPLOY_RUNBOOK.md**: "On VETO" and the guardian veto deadline send two or more ops to `veto-all`, and a veto
  refused because the owner cancelled its op first to a fresh review and `veto-all`; guardian revocation step 1 pauses
  the router with a pauser key (the guardian Safe's pause file only as a fallback: if another pause lands first it
  keeps the Safe's nonce and holds step 2 behind it).
- **Harnesses.** `scripts/test-safe-tx.sh` 359 checks, `scripts/test-check-proposals.sh` 297.

## ⚪ No wire or ABI change — 2026-10-28 (the stall extension and lift procedure; the Safe signer's view; the 743111 Day-0 registration)

No `src/` change and no pin move; scripts, docs and tests only.

- **One stall rule.** Pause, then `extendLocksForDowntime` in its own Safe transaction, then every lift (router or
  Registry `unpause()`, adapter re-enable, `executePendingReactivation`, `unblacklistVerifier`) last.
  `scripts/check-lift-tx.sh` ABORTs a file that holds an extension and any lift, in either order (a skip is an event,
  not a revert, so in a shared batch a skipped lock stays silent until the lift has landed). `scripts/safe-tx.py write
  extend` takes `--downtime-end` equal to the router's pause start and refuses any other; before writing, it simulates
  each extension by a read-only `eth_call` from the GT and prints, per id, the extension or the skip code and when the
  lock becomes slashable; ids that would skip code 5 are written with their start raised past the credited window. The
  router's stall check offers the re-pause after `PROVER_GRACE` too ("re-pause now, then extend") and fires on every
  lift kind, not only the router's own unpause.
- **The signer's view.** `scripts/safe-tx.py` decodes upgrade data at any depth (the combined router op, ProgramRegistry
  upgrades) and refuses a lift inside upgrade data, which the lift gate cannot see there.
- **Guardian review.** `scripts/check-proposals.sh` judges an `undeprecateAdapter` against the `EXPECTED_ADAPTERS`
  entry's mode and economics too (a VETO on a difference); removing a shipped emergency selector is a VETO outside an
  announced, pending guardian revocation.
- **743111 Day 0.** `script/DeployVerifiers.s.sol` `registerByDeployer()` registers `RISC_ZERO_GROTH16_V1`,
  `SP1_GROTH16_V1` and `SP1_PLONK_V1` from the deployer key before the Day-1 handoff (pins checked first, mode None,
  the 743111 economics; refused on 43111), so testnet jobs can be proved from Day 0 (DEPLOY_RUNBOOK.md "Day 0", step
  3a). `OPENVM_HALO2_V1` is registered by a GT op after Day 1, as every 43111 adapter is.
- **Deploy.s.sol.** On 43111 a `MAX_TOTAL_STAKED` above 10 billion HEMI (an amount scaled by 1e18 twice) is refused,
  and the summary prints the cap in HEMI.
- **Harnesses.** `scripts/test-check-lift-tx.sh` 72, `test-safe-tx.sh` 353, `test-check-proposals.sh` 296,
  `test-bcc-monitor.sh` 395, `test-fresh-deploy.sh` 414 checks; each takes `PORT_RANGE=<lo>-<hi>` for its anvil ports.

## ⚪ No wire or ABI change — 2026-10-28 (reference daemons: the new-job scan stops after every job; a claim refused for free stake is retried)

No `src/` change and no pin move; scripts, docs and tests only.

- **Why.** The scan budget and the `LOCK-LATE` stop were checked only at block boundaries, so one block of jobs carrying
  a prover's own descriptor (anyone can submit them: priced below its reservation and refunded later, or cancelled
  ahead of each claim through the public mempool) held the lock loop for the whole block, minutes on a public
  endpoint, with no `LOCK-LATE` page meanwhile. And a claim the router refused for free stake (`Panic(0x01)`) was
  skipped for good.
- **Per-job stops.** The new-job scan checks `SCAN_BUDGET_SEC` and the `LOCK-LATE` crossing after every job it handles
  (its claim attempt included) and stops before the next one, inside a block if need be; the cursor holds below that
  block, and the next pass skips the jobs it already handled with no RPC read (`STATE_DIR/scan-done`: `<block>
  <jobId>`, entries past the cursor only). A flood keeps a lock waiting at most the budget plus one job. The claim
  retry pass stops the same way (with no poll sleep after a `LOCK-LATE` stop).
- **Free stake.** `Panic(0x01)` (Staking `lockCollateral`, reached by `claimJob`, `claimJobDelegated` and
  `claimAndFulfillAssigned`) and `PendingUnstakeConflict` are a claim refused for free stake: the job goes to the claim
  retry set marked with the key's `getAvailableCollateral`, is re-checked by `eth_call` once that rises (a lock
  settles, a top-up, a cancelled unstake), and `PAGE: STAKE-SHORT` fires at most hourly. Any other `Panic` code stays
  unclassified.
- **Harness.** `scripts/test-prover-daemons.sh` 478 checks: §1 (the per-job stops, the retry pass, the stake class),
  §42 (e) and a new §43 (one block of 50 own-descriptor jobs priced below the reservation or cancelled while a lock is
  late: the lock is paged and fulfilled before the block's last job, each job read once; a claim refused for free
  stake, claimed once a lock of the key settles). `PORT_RANGE=<lo>-<hi>` confines its anvil and proxy ports.

## ⚪ No wire or ABI change — 2026-10-28 (TransferOwnership `verify` reads only owner events; deploy-script hygiene)

No `src/` change and no pin move; scripts, docs and tests only.

- **`verify`'s log scans are topic-filtered.** `script/lib/VerifyLogScanner.sol` made one unfiltered `eth_getLogs`
  scan per contract (router, Registry, Staking, BCC, ProgramRegistry) and held every answer in one frame. The
  ProgramRegistry's `createApplication` and `updateApplicationMetadata` are free, so a flood of their logs, under
  0.02 ETH of L2 gas, pushed `verify` past the runbook's 4 GiB `FOUNDRY_MEMORY_LIMIT` (an empty revert, no `ok`
  lines). Every scan now makes one query per contract, event and `LOG_CHUNK` range, each for an event only the owner
  or an initializer makes the contract emit; logs anyone can write are never fetched, and an answer holding a log
  outside its filter is refused. The deploy's own logs still come from the same queries, so an endpoint that answers
  [] everywhere fails `verify`. The runbook keeps `FOUNDRY_GAS_LIMIT` / `FOUNDRY_MEMORY_LIMIT` on every `verify` line
  as headroom.
- **The clocks gate's forge limits.** `scripts/check-pause-clocks-upgrade.sh` runs forge with
  `FOUNDRY_GAS_LIMIT=9223372036854775807 FOUNDRY_MEMORY_LIMIT=4294967296` on its own command line (its `JobSubmitted`
  scan grows with every job). `scripts/check-runbook-commands.sh` holds the `forge script` lines of the scripts
  themselves (not only the runbook's) to the same rule, for `verify` and for `CheckPauseClocksUpgrade.s.sol`.
- **TransferOwnership binds every address to the deployment record.** With a record at hand, every phase refuses a
  record of another chain and a `TIMELOCK`, `HEMI_PROVE`, `REGISTRY` or `STAKING` other than the record's (before:
  `PROGRAM_REGISTRY` only).
- **Deploy.s.sol.** A simulation (no `--broadcast`) writes no deployment record (it prints the path the broadcast
  writes). `PROGRAM_REGISTRY_FEE` between 1 wei and 0.001 HEMI is refused (a fee typed in whole HEMI), and the summary
  prints the fee in HEMI. The printed next steps and the usage of Deploy, DeployBCC and TransferOwnership state both
  fee fields on each broadcast.
- **Harness.** `scripts/test-fresh-deploy.sh` (the simulation's record, the printed fee fields, a
  `TIMELOCK` other than the record's, and on 743111 a 40,000-log flood on the ProgramRegistry and the router after
  which `verify` passes under forge's default limits); `scripts/test-check-pause-clocks-upgrade.sh` 96 (the wrapper's
  own limits against a caller exporting tiny ones, with a control); `test/script` forge tests for the filtered scanner
  (a flood, an endpoint ignoring the filter, an all-empty backend), the record binding and the fee floor. Both
  harnesses take `PORT_RANGE=<lo>-<hi>` for their anvil ports.

## ⚪ No wire or ABI change — 2026-10-28 (the schema-pin example, the L1-hatch tool, the local CI mirror, rehearsal docs)

No change to a deployed contract and no pin move: the example below is not deployed and has no ABI file; scripts, docs
and tests otherwise.

### 🟡 Example: `SchemaValidatedCallback` decodes with a reviewed schema
- **Why.** A program's registry entry is controlled by its registrant (first come) or family admin, who can change the
  schema in any block, including around a consumer's request inside one block. A pin read from the registry at the
  request (the guide's earlier advice) is then the controller's, and the result is misdecoded or refused.
- **New:** owner-only `pinSchemaHash(bytes32 programId, bytes32 schemaHash)` (the keccak256 of the schema reviewed
  off-chain; 0 unpins) with `SchemaHashPinned(bytes32 indexed programId, bytes32 schemaHash)`; views
  `pinnedSchemaHash(bytes32)`, `schemaCopy(bytes32 schemaHash)`, `jobSchemaHash(bytes32 jobId)` and
  `storedSchemaHash(bytes32 programId)`.
- **`requestProof`** reverts `SchemaNotPinned(bytes32 programId)` without a pin and
  `SchemaHashMismatch(bytes32 programId, bytes32 pinned, bytes32 actual)` when the registry's current
  `getIOSchemaHash(programId, 0)` (or the bytes it copies at the first request) is not the pinned hash; the job records
  the hash. The callback and `getStoredFieldValue` decode with the stored copy of that hash, never with the registry's
  live schema. `NoSchemaRegistered(bytes32)` is removed. A re-pin applies to later requests only.
- **Docs.** `IO_SCHEMA_INTEGRATION_GUIDE.md` ("Pin the schema you decode with") and `abi/INTERACTION_GUIDE.md` §5
  ("Authenticate every callback"): pin a hash reviewed off-chain, compare it at the request, decode with the stored copy.

### The router's L1 hatch: `scripts/l1-hatch-tx.py`
- `plan`, `check` and `track` for the L1 guardian's `OptimismPortal.depositTransaction` (the router's `emergencyPause()`
  or `emergencyCancelPendingUpgrade(opHash)` from `alias(L1_GUARDIAN)`) and for a pauser EOA's `relay.pause(ROUTER)`
  deposit. The portals are pinned per L2: 743111 `0xB6f9579980aE46f61217A99145645341E49E2516` on Sepolia, 43111
  `0x39a0005415256B9863aFE2d55Edcf75ECc3A4D7e` on Ethereum (Hemi's published contract addresses, checked against each
  chain's `SystemConfig.optimismPortal()`). It never signs or sends: the L1 guardian Safe signs the written file in
  Safe{Wallet}. `track` derives each deposit's L2 transaction hash and reports its landing and event.
- `DEPLOY_RUNBOOK.md` "Router guardian and its L1 hatch": the L1 Safe created and checked like the L2 Safes
  (`scripts/create-safe.sh`, `scripts/check-safe-config.sh --creation` on the L1), the transaction, its signing, the
  drill; drill (c) is now a `relay.pause` by L1 deposit with a pass condition. New variable `L1_RPC`.

### The local CI mirror: `scripts/ci-local.sh`
- Runs `test.yml` (`pr`) and `nightly.yml` (`nightly`) on the host, each step with its profile and environment. It
  refuses to run when its step table differs from the workflows (`--parity`), or on a Foundry other than 1.5.1 at its
  commit; the 1.8.5 daemon step checks the tarball's sha256 before unpacking it. `test.yml` gains two steps:
  `./scripts/test-l1-hatch-tx.sh` and `./scripts/test-ci-local.sh` (the parity check and the mirror's self-test).

### Docs
- `DEPLOY_RUNBOOK.md`: an archive of `safe-tx/` after each run that writes it (Day 0, Day 1; `archive_safe_tx`); the
  signing host's `cast` checked by its release attestation (`gh attestation verify`, pinned to Foundry v1.5.1's
  commit; offline with `--bundle` and `--custom-trusted-root`); "Rehearsal order and exit criteria"; a second 743111
  endpoint for `RPC_2`; the competitive drills under third-party interference.
- `abi/INTERACTION_GUIDE.md` §3a: a proving market or service is a broadcast (prove first only on your own hardware;
  post a market request only after the lock is mined, its deadline inside `D + G`); step 7a takes the `sp1-prover`
  crate archive with its crates.io checksum; step 9 builds cargo-openvm at tag v2.0.2's commit (`--rev`). OpenVM is
  registered on 743111 by a GT op after the Day-1 handoff, not at Day 0.

## 🔴 Breaking — 2026-10-28 (SP1 PLONK; every proof-system id and adapter named after its wrapper)

Fresh deploy only, like every release since 2026-10-13: the entries below still apply in full. Two changes. **SP1 PLONK** joins the launch set: SP1 proofs wrapped in PLONK, verified by Succinct's vendored v6.1.0 PLONK verifier, under a new proof-system id. And **every proof-system id, its constant and its adapter now name the zkVM and the wrapper of its seals** (a proof system's wrapper fixes the seal format, so one id never accepts two kinds of seal): the old ids, constants and adapter contracts are gone, with no alias. No struct, enum, event or selector of the router, Registry, Staking, ProgramRegistry, GT, BCC or relay changes; the router's only behaviour change is that the proof-nonce marker is accepted on both SP1 ids. Rebuild adapter decoders from the regenerated `abi/*VerifierAdapter.abi.json`.

### 🔴 Wire format: the proof-system ids

Every job's `proofSystemId`, every `registerAdapter` / Registry call, `JobSubmitted`'s indexed `proofSystemId`, `verifiedResults` and `ProofContext` carry the new values, and so does every FULL `expectedJournalHash`: it hashes the job's `proofSystemId` (`keccak256(abi.encode(JOURNAL_PREDICATE_DOMAIN_V1, proofSystemId, programId, journal))`). Recompute every precomputed FULL hash with the new id, once per SP1 id if a statement is served under both: a FULL hash made with an old id, or with the other SP1 id, never settles (the proof of exactly that statement reverts `JournalMismatch` at fulfil). FIELDS hashes and the binding markers do not include the id. A submit under an old id is refused, nothing being registered under it: `AdapterNotActive(proofSystemId)` once the earlier submit checks pass, and before that `InvalidJournalBinding(6)` with the proof-nonce marker (accepted under `SP1_GROTH16_V1` and `SP1_PLONK_V1` only, so no longer under the retired `keccak256("sp1.v1")`) or `DescriptorTooLarge` for a descriptor over the cap.

| Proof system | Before | Since 2026-10-28 |
|---|---|---|
| RISC Zero Groth16 receipts | `RISC_ZERO_V1 = keccak256("risczero.v1")` = `0xe78e7460b61c60a2e4c61b929873b09e221b4c874388e613da921a41eaaeb602` | `RISC_ZERO_GROTH16_V1 = keccak256("risczero.groth16.v1")` = `0xf8fe527fde36a1f428dcd6b89b775a345a829f12ff8f54ba488149a8dc3960c5` |
| SP1, Groth16-wrapped | `SP1_V1 = keccak256("sp1.v1")` = `0x138d8f8d44b49258e0776fa8268f760a61913d981da657b843ce72f9a9f40b74` | `SP1_GROTH16_V1 = keccak256("sp1.groth16.v1")` = `0x023973b8ade217f37321f4b8d736956d5d075aee2500f6edd67d63f4ef51429d` |
| SP1, PLONK-wrapped | none | **new**: `SP1_PLONK_V1 = keccak256("sp1.plonk.v1")` = `0x1ea5c35f3bde6fda536b672488334cd2e98e3ff554f1835725ba35ce0cf3579f` |
| OpenVM Halo2 EVM proofs (743111 only) | `OPENVM_V1 = keccak256("openvm.v1")` = `0x8dca07c28f1102950638b8b05872a8e381168fee7e5e165524851abd2ba27d5b` | `OPENVM_HALO2_V1 = keccak256("openvm.halo2.v1")` = `0xc2f840cd747be40c46b043261812b681bb030ae49b44faf1249979380dca5155` |

`CUSTOM_VERIFIER = keccak256("custom")` is unchanged.

### 🔴 Wire format: the adapters

| Before | Since 2026-10-28 | `name()` |
|---|---|---|
| `SP1VerifierAdapter` (`abi/SP1VerifierAdapter.abi.json`), `name()` "SP1" | `SP1Groth16VerifierAdapter` (`abi/SP1Groth16VerifierAdapter.abi.json`), under `SP1_GROTH16_V1` | "SP1 Groth16" |
| none | **new** `SP1PlonkVerifierAdapter` (`abi/SP1PlonkVerifierAdapter.abi.json`), under `SP1_PLONK_V1` | "SP1 PLONK" |
| `RiscZeroVerifierAdapter`, "RiscZero" | `RiscZeroGroth16VerifierAdapter`, under `RISC_ZERO_GROTH16_V1` | "RISC Zero Groth16" |
| `OpenVmVerifierAdapter`, "OpenVM" | `OpenVmHalo2VerifierAdapter`, under `OPENVM_HALO2_V1` (743111 only) | "OpenVM Halo2" |
| `RiscZeroVerifierAdapterPoVW`, "RiscZero-PoVW" | `RiscZeroGroth16PoVWVerifierAdapter` (not published, deployed by nothing, refused by `verify` on 43111 and 743111 by its name) | "RISC Zero Groth16 PoVW" |

The functions, errors and events of each adapter are unchanged (`verifyProof`, `verifyProofBatch`, `nativeVerifier`, `name`, `version`; OpenVM's `programIdOf` and its two errors). The two SP1 adapters share their code (`src/adapters/SP1AdapterBase.sol`: programId = the program vkey, the seal passed verbatim to the native verifier's `verifyProof`) and differ only in `name()`; their runtime pins differ, so each SP1 id accepts only its own adapter over its own verifier.

### 🟡 SP1 PLONK: the verifier, the adapter, the router rule

- **The verifier** is Succinct's v6.1.0 PLONK verifier, vendored like the Groth16 one: `src/vendors/sp1/SP1VerifierPlonk.sol` and `PlonkVerifier.sol` from sp1-contracts tag v6.1.1 (commit `d3629729c3216eb51bd4859d027a8eb729399fa4`, `contracts/src/v6.1.0/`; identical at tag v6.1.0 and on `main`). Circuit v6.1.0, the circuit of every SP1 SDK from 6.1.0 to 6.10.0, so it verifies the PLONK proofs of the same SDKs as the Groth16 verifier: `VERSION()` `"v6.1.0"`, `VERIFIER_HASH()` **`0x5a093a2fcb46394f5cadfe55c44d4d572fad9cec7aeb38026b0278322ef07fac`** (sha256 of the SDK's `plonk_vk.bin`, `test/fixtures/verifiers/sp1-v6.1.0-plonk_vk.bin`), `VK_ROOT()` the Groth16 verifier's (`0x002f850e…`). Both upstream files declare `contract SP1Verifier`: import them under aliases where both are needed. It differs from the Groth16 wrapper only in its base verifier, its `VERIFIER_HASH` and the decoding of the proof after byte 100: the seal is the 4-byte selector `0x5a093a2f`, the exit code, the VK root, the proof nonce (bytes 68..100, public input 4, as in a Groth16 seal), then the PLONK proof, 964 B in all. A Groth16 seal sent to it reverts `WrongVerifierSelector(0x4388a21c, 0x5a093a2f)` and the reverse `WrongVerifierSelector(0x5a093a2f, 0x4388a21c)`: one id never settles with the other wrapper's seal (`VerifierCallFailed` at fulfil).
- **The adapter** `SP1PlonkVerifierAdapter` wraps it under `SP1_PLONK_V1`: `programId` is the program vkey (the same vkey as under `SP1_GROTH16_V1`: one SP1 program verifies under both ids), `publicValues` the SP1 public values, `proofBytes` the seal, `extraVerifierData` unused.
- **The router rule** (`HemiProveCore`, submit): the proof-nonce marker (`journalFields` slot 0 = `0xfffffffe`) is accepted on `SP1_GROTH16_V1` and `SP1_PLONK_V1`, and refused on every other id with `InvalidJournalBinding(6)` as before. The fulfil check is unchanged (`HemiProveFulfill._checkBinding`: `proofBytes[68:100]` against `getProofNonce(jobId, job.prover).nonce`, `ProofNonceMismatch(expected, actual)`), and holds for PLONK because both wrappers read their public input 4 from that word. The OpenVM rules (reason 7, `OPENVM_PUBLIC_VALUES_BYTES` = 32) apply to `OPENVM_HALO2_V1`.
- **Registration.** `SP1_PLONK_V1` joins the launch set on **743111 and 43111**, beside `SP1_GROTH16_V1`, in mode None with the same economics (`DEPLOY_RUNBOOK.md`, "SP1 registration"); OpenVM stays on 743111 only. Its native `verifyProof` costs about 280,000 gas; a FULL-bound `fulfillJob` with a real proof 639,000 gas, against 578,000 for a Groth16 seal (`test/integration/SP1PlonkProofs.t.sol`). Both sit far below the router's 4,000,000-gas verifier budget, so the gas preflight and `script/lib/FulfillGas.sol` need no new parameter.
- **ProgramRegistry: no change.** An entry lists one `proofSystemId`, and the router never reads the ProgramRegistry: a job's own `proofSystemId` picks its adapter. A program registered under one SP1 id is served under the other too (`abi/INTERACTION_GUIDE.md` §3 and §11, `IO_SCHEMA_INTEGRATION_GUIDE.md`, `DEPLOY_RUNBOOK.md` "ProgramRegistry").

### Bytecode, sizes and pins

- **`AUDITED_CORE_RUNTIME_KECCAK.CORE`** = `0x96787200886d4ae567581ca248d382ffbd333c161a8f44a361573d376a024cff` (was `0x8d8f6e4297307d3cca386b4342d53ec515c5c15427a441deb9c75fcbf795e14d`): the reason-6 rule. `HemiProveCore` deploy-profile runtime 22,903 B. The other 11 pins, `AUDITED_BCC_RUNTIME_KECCAK` and `AUDITED_RELAY_RUNTIME_KECCAK` are unchanged; the other facets change only in comments (the deploy profile carries no metadata). No storage change (`.storage-layout/` unchanged).
- **Verifier pins** (`script/lib/VerifierPins.sol`): new `SP1_V6_1_0_PLONK_RUNTIME` = `0xd41d21f0ede6aad839e0aaac997eea8e168333bc5d15fb5f520e385dcb227f47` (8,625 B, no immutable; byte-identical to a deploy-profile build of the verbatim upstream files; `test/fixtures/verifiers/sp1-v6.1.0-plonk.deploy.hex`) and `SP1_V6_1_0_PLONK_VERIFIER_HASH` = `0x5a093a2f…`. `SP1_V6_1_0_VERIFIER_HASH` is renamed `SP1_V6_1_0_GROTH16_VERIFIER_HASH` (same value, `0x4388a21c…`); `SP1_V6_1_0_GROTH16_RUNTIME`, the RISC Zero pins and the OpenVM verifier pins are unchanged.
- **Adapter pins**, renamed after their wrapper and moved with the `name()` strings (the runtime with its `nativeVerifier` immutable zeroed):

| Pin | Value | Size | Immutable offsets |
|---|---|---|---|
| `SP1_GROTH16_VERIFIER_ADAPTER_RUNTIME` (was `SP1_VERIFIER_ADAPTER_RUNTIME`, `0x56bf1a04…`, 1,843 B) | `0x99d18017af86518b2b86cd982aeadc69ed9c5c9611a52ccae7b637eb1b0c636d` | 1,851 B | 101, 527, 1165 |
| `SP1_PLONK_VERIFIER_ADAPTER_RUNTIME` (new) | `0xa8306b163225c75ecbf056f4b1de173fd7a1130ee61c5ba23c72b5f7f05c2ce0` | 1,849 B | 101, 527, 1165 |
| `RISC_ZERO_GROTH16_VERIFIER_ADAPTER_RUNTIME` (was `RISC_ZERO_VERIFIER_ADAPTER_RUNTIME`, `0x30f1011b…`, 1,930 B) | `0xdc362a202817f67a52ef910bf022660f3723a4b7fc5e451f9799be71fb7b0a89` | 1,939 B | 101, 526, 1245 |
| `OPENVM_HALO2_VERIFIER_ADAPTER_RUNTIME` (was `OPENVM_VERIFIER_ADAPTER_RUNTIME`, `0x6f68a310…`, 1,859 B) | `0x089268776de65a642931b72266d9424624a785dfad7d1763ff8a0c83c140cbeb` | 1,865 B | 111, 1657 |

  The runtime fixtures are `test/fixtures/verifiers/{sp1-groth16,sp1-plonk,risc0-groth16,openvm-halo2}-verifier-adapter.deploy.hex`. `VerifierPins.adapterProblem(adapter, verifier, psId)` and `adapterPin(psId)` take the canonical psId (the `bool sp1` form and `openVmAdapterProblem` are gone).
- **Anchors** (`scripts/check-verifier-anchors.sh`): fifteen vendored files (src/vendors/sp1: 5); the 35 VK and SRS constants of `PlonkVerifier.sol` decoded from `plonk_vk.bin` (sha256 = the PLONK `VERIFIER_HASH()`), both wrappers' `VK_ROOT()` equal, the PLONK fixture proofs' prefix `0x5a093a2f`.

### Scripts and tools (no ABI change)

- **`script/DeployVerifiers.s.sol`**: one entry point per id, `sp1Groth16()`, `sp1Plonk()` (new), `risc0Groth16()`, `openvmHalo2()` (were `sp1()`, `risc0()`, `openvm()`); they read `SP1_GROTH16_VERIFIER`, `SP1_PLONK_VERIFIER`, `RISC_ZERO_GROTH16_VERIFIER`, `OPENVM_HALO2_VERIFIER` and print `<PREFIX>_VERIFIER=` / `<PREFIX>_ADAPTER=` with that prefix (were `SP1_`, `RISC_ZERO_`, `OPENVM_`).
- **`scripts/check-verifier.sh`**: kinds `sp1-groth16`, `sp1-plonk` (new), `risc0-groth16`, `openvm-halo2` (were `sp1`, `risc0`, `openvm`). `sp1-plonk` checks the PLONK pins, both real PLONK fixtures, a flipped public value (`InvalidProof()`, `0x09bde339`) and the real Groth16 fixture refused `WrongVerifierSelector`; `sp1-groth16` refuses the real PLONK fixture the same way; a verifier of the other wrapper is named as such.
- **TransferOwnership `verify`**: `SP1_PLONK_V1` is in the checked set (Unregistered unless listed in `EXPECTED_ADAPTERS`); where registered, its `nativeVerifier()` answers the PLONK `VERIFIER_HASH()` and has the PLONK runtime keccak, and it is the pinned `SP1PlonkVerifierAdapter` ("ok verifier pins", "ok adapter pins"). The OpenVM launch gate matches the name "OpenVM Halo2"; the PoVW refusal the name "RISC Zero Groth16 PoVW".
- **`scripts/check-proposals.sh`** judges `SP1_PLONK_V1` adapter ops with `check-verifier.sh sp1-plonk`; **`scripts/safe-tx.py`** names a canonical `--ps-id` label's constant (`sp1.plonk.v1` = `SP1_PLONK_V1`), refuses `write register-adapter` of any other id (exit 2, no file: a mistyped label registers a psId no requester reaches) unless `--allow-new-psid` is given (then a NOTE), and refuses `write register-adapter` of `OPENVM_HALO2_V1` on 43111.
- **Reference daemons** (`script/lib/prover-lib.sh` `pl_sp1_mode`, `pl_seal_problem`): the job's id picks the SP1 proving mode, and a seal whose 4-byte prefix is not that wrapper's (a PLONK seal for an `SP1_GROTH16_V1` job, or the reverse) never leaves the host: `job-prover.sh` refuses to start with one, `demo-prover.sh` refuses such a `FIXTURE_SEAL` at start, does not claim with such a generated proof, runs its `PROVE_CMD` with `SP1_PROOF_MODE` (groth16 / plonk) and `HEMIPROVE_PROOF_SYSTEM_ID`, and refuses the default RISC Zero `PROVE_CMD` for an SP1 consumer.
- **Dev and demo scripts**: `DeployTestnet.s.sol` and `DeployViaFactory.s.sol` register `SP1_PLONK_V1` too; `LiveE2E.s.sol` runs 3 jobs of each of `RISC_ZERO_GROTH16_V1`, `SP1_GROTH16_V1` and `SP1_PLONK_V1`; `DemoHashChain.s.sol` takes `PROOF_SYSTEM` = `risczero-groth16` (default), `sp1-groth16` or `sp1-plonk`, `DemoInteraction.s.sol` `risczero-groth16` or `sp1-groth16`; `RegisterOxygen.s.sol` `addSP1Variant(registry, vkey, proofSystemId)` takes `SP1_GROTH16_V1` or `SP1_PLONK_V1`.
- **Harnesses**: `scripts/test-check-verifier.sh` 45 checks, `scripts/test-check-proposals.sh` 275, `scripts/test-safe-tx.sh` 327 on 1.4.1 (334 in all), `scripts/test-prover-daemons.sh` 457, `scripts/test-fresh-deploy.sh` 378 (`SP1_PLONK_V1` registered on both chains, `verify` with its entry, a real PLONK proof settled), `scripts/test-check-verifier-anchors.sh` 31.

### Downstream impact

- **Integrators and indexers**: replace the four ids and the constant names; decode adapters from the regenerated `abi/*VerifierAdapter.abi.json` (the old `SP1VerifierAdapter`, `RiscZeroVerifierAdapter` and `OpenVmVerifierAdapter` files are gone). A job's `proofSystemId` now also names its wrapper: an SP1 requester chooses `SP1_GROTH16_V1` or `SP1_PLONK_V1` for the same `programId` (`abi/INTERACTION_GUIDE.md` §3). A filter on `JobSubmitted.proofSystemId` for SP1 jobs lists both ids. A stored or precomputed FULL `expectedJournalHash` is recomputed with the new id (it hashes the `proofSystemId`; one per SP1 id).
- **Provers**: prove in the mode the job's id names, Groth16 for `SP1_GROTH16_V1` and PLONK for `SP1_PLONK_V1`, and check the seal's prefix (`0x4388a21c` or `0x5a093a2f`) before claiming and before fulfilling (prover checklist step 7). The proof nonce works the same way in both modes (step 7a). RISC Zero and OpenVM provers change only the ids.
- **Operators**: the renamed `DeployVerifiers` entry points, `check-verifier.sh` kinds, environment variables, `EXPECTED_ADAPTERS` labels (`sp1.groth16.v1`, `sp1.plonk.v1`, `risczero.groth16.v1`, `openvm.halo2.v1`) and `--ps-id` labels; the `SP1_PLONK_V1` registration on both chains (`DEPLOY_RUNBOOK.md`, "SP1 registration").
- **Unchanged**: every router, Registry, Staking, ProgramRegistry, GT, BCC and relay selector, event, error and struct; `CUSTOM_VERIFIER`; the SP1 Groth16, RISC Zero and OpenVM verifiers and their pins; the binding modes and markers, the nonce derivation (`getProofNonce`) and the fulfil checks; storage.

## 🔴 Breaking — 2026-10-27 (GovernanceTimelock: the guardian's cancel-all; only the GT pin moves)

Fresh deploy only: the GT is not a proxy, and `Deploy.s.sol` deploys it. Only the GovernanceTimelock changes (its ABI, `abi/GovernanceTimelock.abi.json`, and its runtime): no router, facet, Registry, Staking, ProgramRegistry, BCC, relay, adapter or verifier byte moves, and the ProgramRegistry entry below still applies in full. Breaking for anything that derives a GT op's state from its logs: an op can now end with no `OperationExecuted` or `OperationCancelled` of its own.

### 🔴 Wire format

| Item | Before (2026-10-26) | Since 2026-10-27 |
|---|---|---|
| `guardianCancelAll()` | none | **new**, `0x093b5326`, guardian only (`NotGuardian` otherwise): increments `cancelEpoch` and emits `AllOperationsCancelled`. The guardian is the `GuardianPauseRelay`, so the guardian Safe sends `relay.forward(abi.encodeCall(GT.guardianCancelAll, ()))` (`forward(bytes)`, `0xd948d468`, wrapping the 4 bytes `0x093b5326`; the relay is unchanged) |
| `cancelEpoch() returns (uint48)` | none | **new** view, `0xde45c641`: 0 at deploy, one higher per cancel-all |
| `AllOperationsCancelled(uint48 epoch)` | none | **new** event, topic0 `0xaf34645a1d93bf743f58c2215ca4aa5d2d497528432fd00fdecda3e2f7e5744a`, no indexed topic: every op proposed before it is void; `epoch` is the new `cancelEpoch()` |
| `pendingOps(bytes32 opHash) returns (uint48)` | the public mapping's getter | the same selector `0x7385df35` and output, now an explicit view: 0 also for an op proposed before the last cancel-all (the ABI names its argument `opHash`; it was unnamed) |

### 🔴 Behaviour (same wire)

- **A voided op** (proposed before a `guardianCancelAll`): `pendingOps` reads 0; `execute`, `cancel` and `guardianCancel` revert `OperationNotPending`; `propose` accepts it again at once (a new `readyAt`, a new 24h review). No `OperationCancelled` is emitted per voided op: the one `AllOperationsCancelled` stands for all of them. An op proposed after the cancel-all is pending as usual. An expired op is voided like any other.
- **A pending guardian revocation** (`proposeRevokeGuardian`) is not an op: a cancel-all leaves it, and the owner's 14-day path is unchanged.
- **Why.** The guardian's veto is the control against a compromised owner Safe, and the 14-day revocation argument (`DEPLOY_RUNBOOK.md`, "Guardian revocation") assumes it cancels every hostile op within 24h. One owner transaction can propose about a thousand distinct ops (`transferOwnership(x_i)` of Staking, for instance), while each guardian veto is a signing round and Hemi's per-block DA cap holds `guardianCancel` calls to about 600 per block: one veto per op cannot keep pace, and a surviving op executes. One `guardianCancelAll` voids any number of ops in one transaction (each proposal carries the cancel epoch it was proposed in; the call costs under 50,000 gas through the relay whatever the number, `test/governance/GovernanceTimelockCancelAll.t.sol`). A rogue guardian gains nothing it lacks with per-op cancels, which already let it cancel every op; the owner proposes again the legitimate ops a cancel-all voids.
- **An op's state from the logs**: live only while its last `OperationProposed` follows the last `AllOperationsCancelled` and no `OperationExecuted` or `OperationCancelled` follows it; equivalently, `pendingOps(opHash) != 0`.

### Bytecode, sizes and pins

- **GovernanceTimelock** deploy-profile runtime 3,861 B (was 3,557), initcode 4,769 B; ci profile 3,918 B (`scripts/check-eip170.sh`, both profiles).
- **`AUDITED_CORE_RUNTIME_KECCAK.GOVERNANCE_TIMELOCK`** = `0x6fcaec97c0131e448951cc4039bf668410123eb66f4e33253a3701789ebdfe9f` (was `0x6d83c655…63ce`); the other 11 pins, `AUDITED_BCC_RUNTIME_KECCAK` and `AUDITED_RELAY_RUNTIME_KECCAK` are unchanged. A deployment record's `timelockCodehash` follows the new build.
- **Storage**: the GT is not upgradeable and has no layout file (`scripts/check-storage-layout.sh` covers the five UUPS proxies, unchanged). The public `pendingOps` mapping became a private mapping of `(readyAt, epoch)` read through the `pendingOps` view, beside the new `cancelEpoch`.

### Scripts and docs (no ABI change)

- **`scripts/check-proposals.sh`**: an op proposed before the latest `AllOperationsCancelled` is void (its `pendingOps` must read 0; anything else is exit 1), one `ok` line per cancel-all names its block, epoch and transaction and the ops still void from it, and the `AllOperationsCancelled` logs must carry the epochs 1 to `cancelEpoch()` (exit 1 otherwise: a lagging log backend). **GT FLOOD**: more than `GT_FLOOD_THRESHOLD` (default 20) unexpired live GT ops that need the guardian (a VETO, NOTE or UNJUDGED line each) is a VETO, `GT FLOOD: …`, with an `ACTION GT FLOOD` line naming the remedy, one `write veto-all`.
- **`scripts/safe-tx.py`**: `write veto-all` (guardian Safe only, no flags): one `relay.forward(GT.guardianCancelAll())`. Its review, as in `build`, `sign` and `exec`, labels the call `CANCEL-ALL` and lists the ops pending at the pinned block (the first 20 by name, then a count) with the NOTE that every one is voided, the owner's legitimate ops included. The list never blocks the veto: when the GT's logs cannot be read (an endpoint that cannot return a flood's logs, endpoints that disagree on them) a `NOTE` stands in its place and the file is still written. `exec` of a GT veto now confirms it took effect at the receipt's block, on every endpoint it reads: each `guardianCancel`'s op reads 0 in `pendingOps`, and each `guardianCancelAll` emitted `AllOperationsCancelled` and raised `cancelEpoch()` (else REFUSED, after the send). `scripts/safe-tx-hash.sh`'s independent decode knows `guardianCancelAll()`.
- **`scripts/gt-op.sh`**: a voided op is `STATE=CANCELLED`, `CANCELLED_BY=GUARDIAN`, `CANCEL_ALL_EPOCH=<epoch>` (only the guardian can call `guardianCancelAll`), never live or inconsistent. The readers that only call `pendingOps` (`scripts/bcc-ops.sh`, `scripts/bcc-ops/`, `scripts/check-lift-tx.sh`, `DeployBCC.s.sol`, `TransferOwnership.s.sol`) need no change: a voided op reads as not pending, which it is.
- **`DEPLOY_RUNBOOK.md`**: the `veto-all` file, the GT FLOOD procedure ("Guardian review of proposals", on a GT FLOOD), the veto deadline against a flood ("Guardian revocation"), the defended-against row, the compromised-owner and rogue-guardian residuals, and the event classification.
- **Harnesses**: `scripts/test-check-proposals.sh` 273 checks (was 252: the cancel-all, a voided op proposed again, `gt-op.sh` on a voided op, a log backend that drops `AllOperationsCancelled`, the FLOOD at 21 ops and not at 20, `GT_FLOOD_THRESHOLD`); `scripts/test-safe-tx.sh` 326 on 1.4.1, 333 in all (was 312, 319: `write veto-all` written, decoded by both tools, signed and executed through the real Safe 1.4.1 and the relay, three owner ops voided; `exec`'s confirmation of both vetoes; a failed list of the pending ops leaves a NOTE and the file is still written).

### Downstream impact

- **Indexers and GT watchers:** decode `AllOperationsCancelled(uint48)` and treat every op proposed before it as cancelled by the guardian; read `pendingOps(opHash)` for a single op.
- **Guardian operators:** on a GT FLOOD, one `scripts/safe-tx.py write veto-all` instead of per-op vetoes ("Guardian review of proposals").
- **Owner operators:** after a cancel-all, propose again the legitimate ops it voided.
- **Everyone else:** nothing; no other contract changes.

## 🔴 Breaking — 2026-10-27 (ProgramRegistry: consent for family and application roles and links; OpenVM prover steps)

Fresh deploy only, like 2026-10-26, whose entry below still applies in full. Only the ProgramRegistry changes: no router, facet, Registry, Staking, adapter or verifier byte moves. Rebuild ProgramRegistry decoders from the regenerated `abi/ProgramRegistry.abi.json`.

### 🔴 Wire format

| Item | Before (2026-10-26) | Since 2026-10-27 |
|---|---|---|
| `transferFamilyAdmin(bytes32,address)` | one-step family admin transfer, `0xb4e7147a` | **removed**: `proposeFamilyAdminTransfer` + `acceptFamilyAdminTransfer` only |
| `transferApplicationAdmin(bytes32,address)` | one-step application admin transfer, `0x56e13c28` | **removed**: `proposeApplicationAdminTransfer` + `acceptApplicationAdminTransfer` only |
| `linkFamilyToApplication(bytes32 applicationId, bytes32 familyId)` (same selector) | either admin linked alone | both admins consent: the family admin's call offers, new event `FamilyLinkOffered(bytes32 indexed applicationId, bytes32 indexed familyId)` (`0x1b0eb50f…`); the application admin's call accepts and links (`FamilyLinkedToApplication`); one address holding both roles links in one call; the application admin without an open offer for that application reverts `FamilyLinkNotOffered(bytes32 familyId, bytes32 applicationId)` (`0x974a01f2`) |

- **No role is pushed.** With the one-step transfers gone, a family or an application changes admin only when the new admin accepts, so nobody can hand a family (with programs planted in it) or an application to an address that never took it.

### 🔴 Behaviour (same wire)

- **Family admin co-control ends at a registrant transfer.** The family admin may write to a program in its family (metadata, storage URIs, schemas, build info, deprecation, successor) only until the program's registrant is transferred to anyone else; an `acceptRegistrant` by the family admin restores it, and `clearRevokedProgram` resets it. Before: every family admin, present or future, co-controlled every program in the family, including one transferred to another registrant. Programs whose registrant never changed keep the co-control (a rotated family admin still curates them).
- **Offers.** An offer names one application and the family admin who made it: it stays open until a link consumes it or another offer replaces it, and counts only while its maker administers the family (a family admin rotation suspends it, a rotation back revives it; a change of application admin does not affect it). There is no cancel and no getter (the `FamilyLinkOffered` event shows it): to withdraw an offer, replace it, or unlink if it is accepted. Either admin can unlink at any time.
- **Constant-cost unlink.** `unlinkFamilyFromApplication` keeps an index per family instead of scanning the application's list from its start, so its cost no longer grows with the list (before: about 2,400 gas per family ahead, so about 12,500 earlier families pushed it past the 30M block).
- **Sizes, storage and pins.** Deploy-profile runtime 22,267 B (was 22,430). Three mappings appended at slots 39-41 (`__gap` 8 → 5; `.storage-layout/ProgramRegistry.json`). `AUDITED_CORE_RUNTIME_KECCAK.PROGRAM_REGISTRY` = `0x2b4b0456f6994a01cf2adf63ad65a2b1073cbfe3d1000cc8417beb0a9c7d5203` (was `0x6ebdad6c…`); the other 11 pins are unchanged.

### Scripts and docs (no ABI change)

- **`script/RegisterOxygen.s.sol`.** `run` checks that the broadcaster controls everything it finds already done (the program's registrant and family, the family admin, the application admin, the family's link), that the family lists no program another account registered and the application no family another account administers, and reverts otherwise, since the Oxygen ids are public and first-come. Its checks run in the simulation, so it is broadcast with `--slow` (a front-run then stops the run at the first reverted transaction). `addNewRiscZeroVersion` only registers; a second run, `deprecateForNewVersion(registry, newImageId)`, deprecates v1.0 once the new id is on chain as this account's.
- **OpenVM prover steps** (`abi/INTERACTION_GUIDE.md` §3a steps 2a and 9, §12; `DEPLOY_RUNBOOK.md` OpenVM step 6 and residual 6): without `--exe`, `cargo openvm commit`, `run` and `prove` build the working directory's crate, running its build scripts and procedural macros, and rustup and cargo read `rust-toolchain.toml` and `.cargo/config.toml` from the working directory and its parents before any flag. Provers check and prove from the requester's published `.vmexe` and `openvm.toml`, kept as plain files, from a working directory of their own and never inside or under a requester-supplied tree (`--config` with `--manifest-path`/`--target-dir` on `keygen`; `--exe`, `--config` and the same two on `commit`, `run` and `prove evm`; `verify evm` with explicit paths; a target directory per job), and launch any source rebuild (SP1 and RISC Zero included, and a registry entry's `buildCommand`) inside a disposable sandbox with no key. Requesters publish those two files.

### Downstream impact

- **Indexers:** decode `FamilyLinkOffered`; a `FamilyLinkedToApplication` now means both admins agreed. Drop the one-step transfer selectors.
- **Tools and integrations** that called `transferFamilyAdmin` or `transferApplicationAdmin`: propose, then have the new admin accept. A family admin linking into someone else's application: offer, then ask that application's admin to accept.
- **OpenVM provers and requesters:** the steps above.

## 🔴 Breaking — 2026-10-26 (proof binding to the job and its prover, one union pause clock, the downtime extension, one-transaction claim and fulfil for reserved jobs, core and peripheral hardening, ProgramRegistry and OpenVM ABIs, solc 0.8.38)

Fresh deploy only, like every release since 2026-10-13: nothing here is an upgrade of an existing stack. Rebuild every decoder from the regenerated `abi/*.abi.json` (`scripts/gen-abi.sh`): the downtime extension is renamed and its events change, the Registry loses its pause clocks, the router gains a facet-table upgrade op and new views, `HemiProveFulfill` gains `claimAndFulfillAssigned`, and `abi/OpenVmVerifierAdapter.abi.json` is new. No enum member changes (the enum-freeze rule in the header holds), no struct a client encodes changes (`JobDescriptor`, `AuctionConfig`, the 27-output `jobs()` tuple, `JobStatusView`, `AdapterEntry` and `ProofContext` are unchanged), and every claim, submit and fulfil selector is unchanged. Integrator-facing semantics are in `abi/INTERACTION_GUIDE.md`; operator procedures (the recovery order after a stall, the lift rules, adapter re-enables) in `DEPLOY_RUNBOOK.md`.

### 🔴 Wire format

| Item | Before (2026-10-17) | Since 2026-10-26 |
|---|---|---|
| The downtime extension (AuxGov, owner and GT emergency selector) | `extendDeadlinesForDowntime(bytes32[] jobIds, address[] expectedProvers, uint40[] expectedLockDeadlines, uint40 downtimeStart, uint40 extension, string reason)`, `0xcea77f03` | **removed**; **`extendLocksForDowntime(bytes32[] jobIds, address[] expectedProvers, uint40[] expectedLockDeadlines, uint40 downtimeStart, uint40 downtimeEnd, string reason) returns (uint256 applied)`**, `0x13b4e19a`, router paused only (below). The type string is the old one, so the name had to change: a pending GT op, the GT emergency pair or a decoder built for the old call cannot run the new meaning |
| Its events | `DeadlinesExtended(bytes32[] jobIds, uint40 extension, string reason)` (topic0 `0xdb57c041…`), `DeadlineExtensionSkipped(bytes32 indexed jobId)` (`0x187ec837…`) | **removed**; per job `DeadlineExtended(bytes32 indexed jobId, uint40 extension, uint40 lockDeadline)` (`0xfc76162e…`) or `DeadlineExtensionSkipped(bytes32 indexed jobId, uint8 reason)` (`0x09650061…`), and once per call `DowntimeCredited(uint40 downtimeStart, uint40 downtimeEnd, uint256 applied, string reason)` (`0x90aefdef…`) |
| Registry pause clocks | `pauseCredit(bytes32,address)` `0x0fad2262`, `getPauseAccounting()` `0xbb3fdefe`, `getKeyPauseAccounting(bytes32)` `0x988d80a6`, event `PauseClockSet(bytes32 indexed key, uint40, uint40)` (`0xb888e2c3…`) | **removed** from the Registry: the router holds the one union clock (below). `pauseTimestamp()` stays (the live Registry pause start, else 0). The router's own `getPauseAccounting()` and `PauseClockSet(uint40,uint40)` are unchanged |
| Registry reseed | `reseedVerifierSwitchDelayIfCorrupted()` `0x138bc0f5`, events `VerifierSwitchDelayReseeded(uint32,uint32)` (`0x26850d88…`), `TransferEpochRebased(uint32,uint32)` (`0xb8f8eb44…`), error `NotInitialized()` `0x87138d5c` | **removed** (a fresh deploy initializes the delay; the read-time floor stays) |
| `_claimJobExternal` (Core, router self-call only) | `(bytes32, address, uint96, uint96)`, `0x1aba486b` | `(bytes32 jobId, address prover, uint96 priceFloor, uint96 premiumFloor, bool reservedOnly) returns (uint96 price)`, `0xc1c4348d` (the claim leg of `claimAndFulfillAssigned` passes `true`; `claimJobBatch` and `slashAndClaim` pass `false`). Callable only by the router itself, as before |
| `SlashClaimFulfillPartialRevert(bytes32 indexed jobId, address indexed prover, bytes reason)` | in every router ABI file (topic0 `0x1a9f56be…`), never emitted | **removed** (event; the error of the same name, never in an ABI file, too). Supersedes the "Kept for ABI compat" line of an earlier entry |
| `ZeroAmount()` in `HemiProveAuxGov.abi.json` | raised by the old extension (`extension == 0`) | gone from that file (the router union keeps it: other facets raise it) |
| ProgramRegistry | 10 version / image / application views, 22 events, 3 errors that could only return zero or never fire | **removed** (list in "ProgramRegistry" below) |

- **Old selectors fail closed.** The router has no route for `0xcea77f03` or `0x1aba486b` (`Error("Unknown selector")`); a call to a removed Registry or ProgramRegistry function reverts. Nothing is overloaded.
- **Enum members:** none added, removed or reordered (`JobStatus`, `VerifierTrust`, `AdapterStatus`, `CycleAttestationMode`, `CurveType`, `EpochStatus`, the ProgramRegistry and BatchCycleCommittee enums: as in the 2026-10-17 build).
- **Router selector table** (`script/lib/RouterSelectors.sol`, installed by every deploy script): 98 selectors, core 19, fulfill 8, aux 24, fulfillCycle 14, auxGov 33 (was 89: 19, 7, 24, 7, 32).

### 🟡 New entries (all additive)

| Contract (ABI file) | Entry | Selector / topic0 |
|---|---|---|
| Router (`HemiProve`, `HemiProveRouter`) | `proposeUpgradeWithFacets(address newImplementation, bytes data, FacetTable table)` | `0x1799bc36` |
| | `upgradeWithFacets(address newImplementation, bytes data, FacetTable table)` (`onlyOwner`, after the timelock) | `0xab5a177c` |
| | struct `FacetTable(address coreImpl, address fulfillImpl, address auxImpl, bytes4[] coreSelectors, bytes4[] fulfillSelectors, bytes4[] auxSelectors, bytes4[] selectorsToRemove, address[] extraImpls, bytes4[][] extraSelectors)`: the nine `updateFacets` arguments | |
| Every router ABI file (declared in `HemiProveBase`) | events `UnionClockSet(uint40 anchor, uint40 total, bool registryPaused)`, `FirstPauseMarked(uint40 firstPauseA, uint40 firstPauseR)`, `KeyClockSet(bytes32 indexed key, bool active, uint40 a, uint40 t, uint40 c, uint24 epoch, uint40 prevEnd)`, `KeyEpochCut(bool routerBase, uint24 epoch, uint40 firstPause)` | `0xfba0d2de…`, `0x6b1f95f3…`, `0x691740e6…`, `0xbc5d1458…` |
| Core | error `NotReserved(bytes32 jobId)` | `0x33529975` |
| Fulfill | `claimAndFulfillAssigned(bytes32 jobId, uint96 priceFloor, uint96 premiumFloor, JobDescriptor descriptor, bytes publicValues, bytes proofBytes) returns (uint96 price)` | `0xce6eda56` |
| | errors `JournalPrefixMismatch(bytes32 jobId, address prover)`, `ProofNonceMismatch(uint256 expected, uint256 actual)` | `0x912b2e79`, `0x07f69e19` |
| FulfillCycle | `getProofNonce(bytes32 jobId, address prover) returns (uint256 nonce, uint32[4] limbs)` | `0x8162527d` |
| | `previewSpeedBonus(bytes32 jobId) returns (uint256)`; `totalPaidToProvers() returns (uint256)` | `0xb9e3f446`; `0x81c61d70` |
| | `getUnionClock() returns (UnionClock, ClockMarks)`, `getKeyClock(bytes32 key) returns (KeyClock)`, `getKeyEpochCut(bool routerBase, uint24 epoch) returns (uint40)`, `clockOf(bytes32 proofSystemId, address customVerifier) returns (uint256)` | `0x64127e06`, `0xcf58a9af`, `0x91403728`, `0x093e7cea` |
| Aux | error `SoftLockCollateralOverflow(uint256 collateral)`: a bid whose soft-lock collateral is above `type(uint96).max` (was `Error("SoftLockCollateralOverflow")`) | `0xec3aeb81` |
| AuxGov | `extendLocksForDowntime(...)` (above); `syncRegistryWindow(bytes32 key)` (Registry only) | `0x13b4e19a`; `0xe2cc91b2` |
| | errors `InvalidDowntimeWindow(uint40 downtimeStart, uint40 downtimeEnd, uint40 pauseStart)`, `NotRegistry(address caller)` | `0x864ea71c`, `0x631a294a` |
| Registry | `clockSink() returns (address)`, `setClockSink(address sink)` (owner, once), event `ClockSinkSet(address indexed sink)`; errors `ClockSinkAlreadySet(address)`, `ClockSinkNotSet()`, `InvalidClockSink(address)` | `0x4503ee53`, `0x08300ef4`, `0x9cc95688…`; `0x7cfdac88`, `0x458552f2`, `0x8018a158` |
| | `undeprecateAdapter(bytes32 proofSystemId)` (owner), event `AdapterUndeprecated(bytes32 indexed proofSystemId)`, error `AdapterNotDeprecated(bytes32)` | `0x496c60be`, `0x84afcc6e…`, `0x12f9b06e` |
| Staking | event `StakeReduced(address indexed prover, uint128 amount, uint128 newTotal, uint128 newPendingUnstake, bool slashed)` | `0x7a4dcebd…` |
| GovernanceTimelock | error `TargetHasNoCode(address target)` | `0xfff23361` |
| HashChainConsumer (the `HemiProveConsumer` template API) | `clearEndedJob(bytes32 jobId) returns (bool cleared)`, event `ProofDropped(bytes32 indexed jobId)`, error `UnknownJobStatus(uint8 status)` | `0xd5007b67`, `0x07cdf534…`, `0xeed8819d` |
| ProgramRegistry | error `ImplementationHasNoCode(address)` (`proposeUpgrade`) | `0x9e172b1d` |
| **`OpenVmVerifierAdapter`** (new file) | `programIdOf(bytes32 appExeCommit, bytes32 appVmCommit) returns (bytes32)`; errors `OpenVmVerifierDataLength(uint256 length)`, `OpenVmProgramIdMismatch(bytes32 programId, bytes32 commitsHash)`; the adapter surface (`verifyProof`, `verifyProofBatch`, `nativeVerifier`, `name()` = `"OpenVM"`, `version()` = 2) | `0x3042086f`; `0xaf77c084`, `0xdb486570` |

`InvalidJournalBinding(uint8 reason)` (`0x7725587f`) gains reasons 6 and 7 (below); an integrator that names the reasons adds them.

### 🟡 Proof binding: a proof made for one job and one prover (opt-in)

- **Why.** A proof of a statement-bound job binds the statement, not the prover: anyone who sees a journal and seal (a fulfil in the public mempool, a reverted transaction) could claim a job with the same statement and settle with the copy. Two opt-in markers in `journalFields` slot 0 (the top 32 bits) now tie a proof to the job and the prover that holds its lock. Without a marker nothing changes.
- **`JOURNAL_PREFIX_MARKER` = `0xffffffff`** (any proof system but `OPENVM_V1`): the journal must start with `abi.encode(jobId, job.prover)`, 64 bytes, the prover word strictly `bytes32(uint256(uint160(prover)))`. FULL and FIELDS then apply to the rest of the journal (`publicValues[64:]`): the FULL hash is over that slice, and FIELDS ranges sit in slots 1 to 7 measured from byte 64, ending within `MAX_JOURNAL_BYTES - 64` = 32,704. A mismatch, or a journal shorter than 64 bytes, reverts `JournalPrefixMismatch(jobId, prover)`. The guest writes the prefix with `commit_slice` of the ABI encoding first, never `commit`.
- **`PROOF_NONCE_MARKER` = `0xfffffffe`** (`SP1_V1` only): the seal's bytes 68 to 100 (the SP1 v6 Groth16 nonce word) must equal the job's nonce, four 30-bit limbs of `B = keccak256(abi.encode(PROOF_BINDING_DOMAIN_V1, block.chainid, router, jobId, prover))` packed at 31-bit spacing (limb 0 most significant). `getProofNonce(jobId, prover)` returns the nonce and the four limbs, exactly `with_proof_nonce`'s argument; it is a pure derivation that answers for any job id and address. A mismatch reverts `ProofNonceMismatch(expected, actual)` (`actual` = 0 for a seal shorter than 100 bytes). Check `seal[68:100]` against `getProofNonce(jobId, yourStakedProver).nonce` before you claim or prove first: `Registry.verifyProof` does not check the nonce. Stock sp1-prover 6.1.0 to 6.10.0 drop the nonce on the local CPU and CUDA paths (the proof carries nonce 0) and the Succinct prover network passes none, so a nonce-bound job needs a fixed or patched local prover (`abi/INTERACTION_GUIDE.md`).
- **"The prover" is `job.prover`**, the staked lock holder, never an operator key; for `claimAndFulfillAssigned` it is the caller.
- **Order.** Both checks run at fulfil (every path: `fulfillJob`, `fulfillJobHashed`, `fulfillJobBatch`, `claimAndFulfillAssigned`) before the statement binding, the typed governance errors, the gas preflight and the verifier call, so a wrong proof costs no verifier gas. The whole journal still goes to the verifier, the callback, `JobFulfilled` and `verifiedResults`.
- **Submit.** The marker is read first: the nonce marker outside `SP1_V1` reverts `InvalidJournalBinding(6)`, the prefix marker on `OPENVM_V1` `InvalidJournalBinding(7)`. A marker with no statement binding (`expectedJournalHash` 0) is NONE and keeps NONE's rules (an assigned prover, no bidding; `InvalidJournalBinding(3)` / `(4)`). Both words were refused in slot 0 before, so the markers extend the accepted set; nothing accepted before changes meaning.
- **Callbacks** under the prefix receive the whole journal: the job's own output starts at byte 64 (the consumer templates' NatSpec says so).
- **Constants** (`src/libraries/Constants.sol`, no getters): `PROOF_BINDING_DOMAIN_V1 = keccak256("HemiProve.ProofBinding.v1")`, `JOURNAL_PREFIX_MARKER`, `PROOF_NONCE_MARKER`, `JOURNAL_PREFIX_BYTES = 64`, `SP1_NONCE_OFFSET = 68`, `SP1_NONCE_LIMB_MASK = 0x3fffffff`.

### 🔴 OpenVM: a published adapter, `programId` names program and VM, reason 7

- **`OpenVmVerifierAdapter`** wraps OpenVM's v2.0 EVM verifier (openvm-solidity-sdk v2.0 `v2.0-base`; OpenVM v2.0.0 to v2.0.2). The verifier is OpenVM's own published build (runtime 19,749 B, pinned in `script/lib/VerifierPins.sol`); its sources are vendored under `src/vendors/openvm` and not compiled here. **743111: registered by a GT op after the Day-1 ownership handoff** (`DEPLOY_RUNBOOK.md`, "OpenVM registration"). `OPENVM_V1 = keccak256("openvm.v1")` is registered over the adapter in mode None: `script/DeployVerifiers.s.sol --sig 'openvm()'` deploys the published verifier (its initcode and runtime checked against the pins) and the adapter, the GT registers them, and `TransferOwnership.s.sol` `verify` accepts the pinned adapter over the pinned verifier under `OPENVM_V1` only. **43111: refused.** `verify` fails on `OPENVM_V1` and on any adapter whose `name()` is `"OpenVM"`, under any psId.
- **Encoding.** `extraVerifierData = abi.encode(bytes32 appExeCommit, bytes32 appVmCommit)`, exactly 64 bytes (`OpenVmVerifierDataLength` otherwise); **`programId = keccak256(extraVerifierData)`** = `programIdOf(appExeCommit, appVmCommit)` (`OpenVmProgramIdMismatch` otherwise); `publicValues` exactly 32 bytes; `proofBytes` the EVM proof's accumulator and proof, 1,760 bytes. Earlier OpenVM adapter sources used `programId = appExeCommit`: a proof of the same executable under another app VM configuration (another set of extensions or opcode handlers) would have verified, so `programId` now names program and VM together, like an SP1 verifying key or a RISC Zero image id.
- **Submit (behaviour, no ABI entry): `InvalidJournalBinding(7)`.** The OpenVM verifier takes exactly 32 public-value bytes, so `OPENVM_V1` submits refuse the journal-prefix marker and any FIELDS range ending past byte 32 (`OPENVM_PUBLIC_VALUES_BYTES = 32`; reason 7 replaces 5 for OpenVM). Such jobs were accepted before but could never be fulfilled. The nonce marker on OpenVM stays reason 6. The reason order is now: 6 or 7 (the marker), then 1, 3, 4 (NONE), then 2, then 5 or 7 (the ranges). OpenVM jobs bind the statement only (FULL over the 32-byte journal, or FIELDS within it): an OpenVM proof can serve any job with the same statement.

### 🔴 One union pause clock, held by the router

- **The clock.** A job's clock now measures the **union** of the windows that block it: the router pause, the Registry pause (adapter jobs), its adapter's Disabled window (adapter jobs) or its custom verifier's blacklist window, each extended by `RESUME_GRACE` (15 min). A second inside overlapping windows is credited once, a joint or staggered lift leaves one grace after the last lift, and job-local time never runs backwards. Before, the router and Registry clocks were summed: overlaps were credited twice, job-local time could run backwards over a joint lift, and the lift rules (joint lifts, router-only relay targets, overlap pages) existed to contain that. `getJobExtras(jobId).pauseCredit`, `getJobStatusView` (deadlines, `slashOpensAt`, `timeRemaining`) and every deadline check read the union value; with no overlap the value equals the old sum.
- **The Registry keeps no clock.** At each of its transitions (pause, unpause, a real disable, a re-enable, a blacklist, an unblacklist) it calls `router.syncRegistryWindow(key)` (`NotRegistry(caller)` from anyone else), and the router re-reads the condition (`paused()`, `isBlacklisted`, the adapter status) and updates its mirror. The router's own pause path makes no external call (the relay and L1-hatch pauses keep a fixed gas cost). The Registry learns the router once: `setClockSink(router)`, owner only, refused twice (`ClockSinkAlreadySet`) and unless the sink has code and its `registry()` is this Registry (`InvalidClockSink`); until it is set, every Registry transition (pause, unpause, disable, re-enable, blacklist, unblacklist) reverts `ClockSinkNotSet()`. The deploy scripts set it.
- **Views.** `clockOf(psId, address(0))` for an adapter job, `clockOf(CUSTOM_VERIFIER, verifier)` for a custom-verifier job: a job's credit since its base is `clockOf(...) - pauseBase` (`getJobExtras`). `getUnionClock()`, `getKeyClock(key)` (key = psId, or `bytes32(uint256(uint160(verifier)))`) and `getKeyEpochCut` expose the state; every field is reconstructible from `UnionClockSet`, `KeyClockSet`, `FirstPauseMarked`, `KeyEpochCut` and the router's `PauseClockSet`. `cancelStaleJob` measures the 7-day disabled time from the router's key clock (was `Registry.getKeyPauseAccounting(key).anchor`).
- **Gas.** A cold `relay.pause(router)` in a fresh transaction costs about 92,000 gas (89,400 for the first pause ever); the relay pause test pins it at 135,000 or less. Pause tooling uses a 150,000 gas limit.

### 🔴 The downtime extension: exact, bounded, while paused

- `extendLocksForDowntime(jobIds, expectedProvers, expectedLockDeadlines, downtimeStart, downtimeEnd, reason)` credits an uncredited outage (a sequencer stall) to the Locked jobs that lived through it. It runs **only while the router is paused** (`ExpectedPause` otherwise), and stays a GT emergency selector (`script/lib/GtEmergencySelectors.sol` pair 2; 11 pairs).
- **Batch reverts:** `InvalidDowntimeWindow(start, end, pauseStart)` unless `downtimeStart < downtimeEnd <= ` the router pause start and the window is at most 24 h (`MAX_DEADLINE_EXTENSION`; a 0 start is refused by the bound); `MaxBatchSizeExceeded` (more than 10 ids); `ArrayLengthMismatch`.
- **Per id**, the first failing check skips it with `DeadlineExtensionSkipped(jobId, reason)` (constants `EXTEND_SKIP_*`): 1 not Locked; 2 the lock changed (`prover` or the raw `lockDeadline` differs from the expected values: another lock, a duplicate id, an earlier extension); 3 already extended (once per lock; reset at every reopen); 4 locked after the window; 5 the window touches a window the job's clock already credits; 6 the lock was past its slash-open point at the window start. Otherwise the router extends the lock by exactly `downtimeEnd - max(downtimeStart, lock start)`, stores it in `totalExtensionApplied`, emits `DeadlineExtended(jobId, extension, newLockDeadline)`, and returns the count applied; `DowntimeCredited` once per call.
- **Gone:** the caller-chosen `extension`, the `min(fulfillmentTimeout, 7 d)` cumulative cap, the `downtimeStart = 0` bypass of the lateness test, and the batch-wide `DeadlinesExtended`. After the extension a lock stands where it stood at the window start: the same time to its deadline, the same prover grace, the same speed-bonus fraction.
- **Recovery order after a stall:** pause the router, extend while paused, lift last (`DEPLOY_RUNBOOK.md`). There is no backdated pause. Open jobs keep the uncredited stall (Dutch ramp, bid rounds and exclusivity advance through it; no slash).

### 🟡 One-transaction claim and fulfil for a reserved job

- **`claimAndFulfillAssigned(jobId, priceFloor, premiumFloor, descriptor, publicValues, proofBytes)`** claims an Open job reserved to the caller and fulfils it in the same transaction; returns the claim price; emits `JobClaimed`, then `JobFulfilled`.
- **Reserved to the caller at inclusion:** a NONE job whose assigned prover is the caller, or a job whose assigned prover is the caller inside its live exclusivity window (job-local time before `rampUpStart + exclusivityDuration`). Any other Open job reverts `NotReserved(jobId)` (an open job, no window or an ended one, a window for another prover); a NONE job assigned to another prover reverts `ExclusivityActive`; a live bid or bid round `BiddingActive`, as `claimJob`. Direct only (an operator key is a third party); no batch variant.
- **The legs.** The claim leg is `claimJob`'s rules, order and exact floors (`ClaimBelowFloor`, nothing locked; floors from an `eth_call` of this function or of `claimJob(jobId, 0, 0)` at a mined block, and `getPremiumCap` there). Collateral is locked and released in the transaction. The fulfil leg is `fulfillJob`'s with the caller as the lock's prover (descriptor hash, journal cap, binding, governance gates, gas preflight, verifier, settlement); any failure reverts both. The lock is fresh, so the speed bonus is the whole premium budget left. It saves about 45,000 gas and one block against `claimJob` then `fulfillJob`.
- **When it is safe.** The proof travels in the mempool while the job is still Open: it is the broadcast rule's one exception. Nobody else can lock a reserved job, but an unbound proof can be taken: the job's caller can cancel the Open job ahead of the transaction and resubmit the statement for another prover, and a window job's transaction held past the window end reverts `NotReserved` and leaves the job, and its published proof, open to anyone. Use it for a job with a binding marker (bound to the job and its prover), or for a requester you trust, and then for a window job only with at least 60 s of the window left in job-local time. Otherwise `claimJob` alone, wait for the receipt, then fulfil. The caller of the Open job can always cancel ahead of it and keep the result: the prover loses the proving work, never stake.
- **Gas limit:** the fulfil formula (`abi/INTERACTION_GUIDE.md` §9) plus the claim leg, which runs before the fulfil preflight; `script/lib/FulfillGas.sol` adds 300,000 (a cold claim leg measures about 95,000).

### 🔴 Core behaviour (same wire)

- **Minimums.** Submit and `setPreset` refuse `0 < exclusivityDuration < 60 s` (`InvalidExclusivityDuration`, `MIN_EXCLUSIVITY_DURATION`) and `0 < maxBidExtension < 60 s` (`InvalidBidConfig()`, `MIN_SNIPE_GUARD`). 0 stays valid for both.
- **Codeless callbacks.** A callback address without code is not called; the fulfil emits `CallbackFailed(jobId, callback, "")` and settles. A callback is called at most once per job: in the transaction that fulfils it, if the address has code; never for a cancelled or unfulfilled job, never at slash, release or reopen, never retried.
- **`JobReleased`.** `timeHeld` counts from the lock: `now - (lockDeadline - fulfillmentTimeout - totalExtensionApplied)` (an extension no longer shortens it); `operator` is always `address(0)` (the delegated-release branch is gone; the field stays).
- **psId after a cancel.** A cancelled job keeps `jobProofSystems(jobId)`, and `getJobStatusView(jobId).proofSystemId` (and its trust) is the job's psId for cancelled and fulfilled jobs.
- **`totalPaidToProvers()`**: every prover payout, `Σ JobFulfilled.proverPayout + Σ CycleCollateralSettled.toProver` (keeper rewards and resolver bounties excluded; an IOU credit counts at settlement). **`previewSpeedBonus(jobId)`**: a Locked job's bonus at the current job-local time, an Open job's premium cap, else 0.
- **Typed reverts.** Submit refuses a blacklisted custom verifier with `BlacklistedVerifier(verifier)` (`0x5b0b12c4`), the error claim, bid and fulfil raise, instead of `Error("Blacklisted")`; a collateral above `type(uint96).max` at claim reverts `CollateralOverflow(collateral)` (`0xb52efcbc`), the error submit raises, instead of `Error("CollateralOverflow")` (a job submit accepted never reaches it); a bid whose soft-lock collateral is above `type(uint96).max` reverts the new `SoftLockCollateralOverflow(collateral)` (`0xec3aeb81`) instead of `Error("SoftLockCollateralOverflow")` (the bound submit checks for the job's maximum price).
- **Removed constants** (source only): `RELEASE_PENALTY_BPS` (unused since the flat release penalty; `MIN_RELEASE_PENALTY_BPS` is the live knob), `DELEGATED_RELEASE_WINDOW`, `MAX_DELEGATED_RELEASES_PER_WINDOW`, `MAX_TOTAL_DELEGATED_RELEASES_PER_PROVER_PER_WINDOW`.

### 🔴 Registry: adapter lifecycle

- **A re-enable restores the status the adapter had.** `reactivateAdapter` (instant branch) and `executePendingReactivation` set a re-enabled adapter that was Deprecated before its disable back to **Deprecated** (its successor and marks kept), not Active; `AdapterReactivated` is unchanged. **`undeprecateAdapter(psId)`** (owner only, not a GT emergency selector) returns a Deprecated adapter to Active and clears the marks (`AdapterUndeprecated`; `AdapterNotDeprecated` otherwise); it never touches a Disabled adapter. To undo a guardian deprecation: `[reactivateAdapter(psId, 0), undeprecateAdapter(psId)]`.
- **The instant re-enable no longer refuses a blacklisted adapter address** (so the recovery batch runs in any order); the swap branch, its execute and `registerAdapter` still refuse one.
- **Initializer versions:** router 3 (the next migration is `reinitializer(4)`), Registry 1, Staking 1.

### 🟡 Peripheral contracts

- **GovernanceTimelock.** `execute` and `emergencyExecute` revert `TargetHasNoCode(target)` for a target without code (after the pending, ready and expiry checks; the op stays pending and can run once code exists, within the expiry). `propose`, `cancel`, `guardianCancel` and `setEmergencySelector` are unchanged, so an op for a CREATE2 address can still be queued. `relay.pause(target)` re-throws the error verbatim instead of reporting a pause that paused nothing.
- **HemiProveStaking.** `StakeReduced(prover, amount, newTotal, newPendingUnstake, slashed)` after every slash (`slashed` true) and non-zero release penalty, after the defensive unstake cap. Per prover: `totalStaked = Σ Staked.amount − Σ Withdrawn.amount − Σ StakeReduced.amount`, with `newTotal` as a checkpoint; `newPendingUnstake` sets the pending unstake.
- **GuardianLib.** A guardian candidate's ERC-165 answer counts only as the canonical word 1; any other non-zero word is false, so the multisig check reverts its typed `GuardianMustImplementERC1271(candidate)` instead of an empty revert.
- **Consumer templates** (`src/base`, published through `HashChainConsumer.abi.json`). `HemiProveCallbackBase.onProofVerified` forwards its calldata unchanged to `_onProofVerifiedExternal` (signature and selector unchanged), so an enum value the base does not know no longer reverts every inheritor (about 585 gas cheaper per callback on ProofResultStore). `HemiProveConsumer` decodes `getJobStatusView`'s status as a raw `uint8` (`_jobStatusRaw`); `_jobStatus` reverts `UnknownJobStatus(status)` for a value outside the enum. `clearEndedJob(jobId)` (anyone) clears the bookkeeping of the contract's own job once it is Fulfilled or Cancelled, through the `_onJobEnded` hook (`ProofCancelled` or the new `ProofDropped`), never for the job whose callback is running (a transient guard); `_cancelProof` returns `ended`, and `HashChainConsumer.cancelJob`'s output is renamed `ended` (same type). ProofResultStore's ABI is unchanged.

### 🔴 ProgramRegistry

- **Deployed with the core.** `script/Deploy.s.sol` deploys the ProgramRegistry last in the fresh deploy (implementation and `ERC1967Proxy`, no linked library), initialized with the HEMI token and `PROGRAM_REGISTRY_FEE` (HEMI wei, at most 100e18; required on 43111, default 0 on 743111). The deployment record (still version 2) gains five keys: `programRegistry` (the proxy, the address integrators use), `programRegistryImpl`, `programRegistryImplCodehash`, `programRegistryCodehash` and `programRegistryFee` (a decimal string); a record without them reads as "no ProgramRegistry". TransferOwnership hands it to the GT with the router, Registry and Staking (`PROGRAM_REGISTRY` in every phase; the Phase 2 and 3 files carry four calls), and `verify` checks its owner, token, fee, implementation pin (`PROGRAM_REGISTRY`, the 12th core pin), proxy pin, initializer version 1, no pending upgrade, and its logs since the deploy. `script/DeployProgramRegistry.s.sol` is for dev chains only (31337, or a devnet opted in with `ALLOW_DEV_DEPLOY_ON`).
- **Removed functions** (each could only return zero or empty data: nothing writes their storage): `getVersionInfo(bytes32)` `0xe2676263`, `getProgramVersionsPaginated(bytes32,uint256,uint256)` `0x1c1b534e`, `getVersionImagesPaginated(bytes32,uint256,uint256)` `0x3cceef8e`, `getImageVersion(bytes32)` `0x811680e9`, `findImageByProofSystem(bytes32,bytes32)` `0xa8b7f685`, `getImageHierarchy(bytes32)` `0xa4273b0b`, `getVersionFieldLayout(bytes32,uint8)` `0x9e1c1734`, `getVersionImagesDetailed(bytes32)` `0x850d131c`, `getApplicationProgramsPaginated(bytes32,uint256,uint256)` `0xb9a29ca3`, `getProgramApplication(bytes32)` `0x48b53c6b`.
- **Removed events** (never emitted): `ProgramRetired`, `ProgramSuccessorUpdated_v2`, `ProgramRevoked_v2`, `VersionCreated`, `VersionDeprecated`, `VersionSuccessorUpdated`, `VersionRevoked`, `VersionFieldLayoutSet`, `ImageRegistered`, `ImageDeprecated`, `ImageSuccessorUpdated`, `ImageRevoked`, `AdoptionProposed`, `AdoptionAccepted`, `AdoptionCancelled`, `ImageUnadopted`, `ProgramLinkProposed`, `ProgramLinkCancelled`, `ProgramLinkedToApplication`, `ProgramUnlinkedFromApplication`, `EncodingModeOverrideSet`, `EncodingModeOverrideCleared`.
- **Removed errors** (never raised by ProgramRegistry): `FieldIndexOutOfBounds(uint8,uint8)` `0xa26394b7`, `PublicValuesTooShort(uint256,uint256)` `0x66b34057`, `InvalidEncodingMode()` `0x5107cbba`.
- **No linked library.** `SchemaLib.validateSchema` is compiled in, so the schema errors `setIOSchema` can raise are in the contract's own ABI (same entries as before); `ProgramRegistryHelpers` is gone. The implementation fits EIP-170 (22,430 B).
- **Behaviour.** `proposeUpgrade` refuses an implementation without code (`ImplementationHasNoCode`); `clearRevokedProgram` also clears the program's I/O schemas, their hashes and its build hashes (no per-schema event: reset them on `RevokedProgramCleared`). Every other function, event and error is unchanged; storage is unchanged (the never-written slots stay reserved).

### Storage

- Router: `uint256 _totalPaidToProvers` appended at slot 48, `__gap` 3 → 2 slots (the footprint end does not move); the union clock lives in a new ERC-7201 namespace, `hemiprove.storage.UnionClock` at `0x02af81880bd94886601329cd9894613688505329d85ab29569391e0797742000`.
- Registry: `_pauseTotal` and `_keyClock` retired in place (`__deprecated_pauseTotal`, `__deprecated_keyClock`), `address _clockSink` at slot 8, `__gap` 39 → 38.
- Staking, BatchCycleCommittee and ProgramRegistry layouts unchanged. No layout waiver.

### Bytecode, sizes and pins

- **Compiler: solc 0.8.38** in every profile (was 0.8.28; 0.8.38 fixes the seven known 0.8.28 bugs). Deploy profile otherwise unchanged (cancun, via-IR, runs 1, no CBOR metadata).
- **Deploy-profile runtime sizes** (was, in the 2026-10-17 build): router 21,715 B (19,713), Core 22,851 (21,956), Fulfill 22,631 (21,249; its policy limit is 23,552), Aux 22,109 (21,979), AuxGov 17,925 (13,709), FulfillCycle 9,919 (7,538), Registry 11,170 (11,971), Staking 11,852 (11,644), GovernanceTimelock 3,557 (3,490), ProofResultStore 1,230 (1,486), BatchCycleCommittee 23,924 (23,922), GuardianPauseRelay 1,580 (1,581), ProgramRegistry 22,430 (24,879, above EIP-170), OpenVmVerifierAdapter 1,859 (new). The router and its facets keep at least 921 B of headroom under their policy limits (Fulfill, 23,552 B).
- **`AUDITED_CORE_RUNTIME_KECCAK`** (`script/lib/AuditedCorePins.sol`, the authority; **12 pins**, was 11): the new `PROGRAM_REGISTRY` pin covers the ProgramRegistry implementation (UUPS `__self` zeroed, like the core implementations; `ERC1967_PROXY` covers its proxy). Every earlier pin moves except `ERC1967_PROXY`. ROUTER `0xfe19d149183c16c2c5917745002d824fb94dc5f1766d8ecf2f1e2ab4c5f6d2aa`, REGISTRY `0xf6ea9179aa36f691770372da03089f89bd0f6b0e52a7976ad749167f9b4388b2`, STAKING `0x8f2769b1ac7e443981de10adba5962fd42552bd14f10006b5f30921eb70aea94`, CORE `0x8d8f6e4297307d3cca386b4342d53ec515c5c15427a441deb9c75fcbf795e14d`, FULFILL `0x5c18136573c55777c269943877a5965785fda93a82160a594b52e9cbecfaaffc`, AUX `0xf7dd82f12ef38cb7ab190611497898d07b77d583043cb0d604bab0de05ecb3ba`, FULFILL_CYCLE `0x60675be0e8b7698b9c432039668443a867b30776cd7548fcbcad98c034118bcc`, AUX_GOV `0x500035cdde2939e214b1ab1d92e04328c9d75291f9e1081c07da6f4c0be0b879`, GOVERNANCE_TIMELOCK `0x6d83c655fdc6b6b34320ff0f62bfb6a98347a0d90bcec196cacc2f2afdc363ce`, ERC1967_PROXY `0x46a010df31eb642b47284650e80051e8f6239a0c995003d230b437810bc4b9b4` (unchanged), PROOF_RESULT_STORE `0x7a17c5b70f7bb81ffdc4f3f35c0cf0b3ad8a38744383c3699cfb18ae92979752`, PROGRAM_REGISTRY `0x6ebdad6c980afc26f846739f6b0220e991a0b88e90a456a6e40304b63a21ac5b` (new). Deploy.s.sol checks the code it deploys, and `verify` the chain, against all twelve on 43111 and 743111.
- **BatchCycleCommittee and GuardianPauseRelay**: no source change (the relay's NatSpec only); both runtime pins move with the compiler: `AUDITED_BCC_RUNTIME_KECCAK` `0x750c801ec6b4c9265b1afba7204187a9c81dd0c81bfc03ee39640b65cc3209a4`, `AUDITED_RELAY_RUNTIME_KECCAK` `0x5a82a221d1ea20fefd46e39142ffb68d01949fa9643a7af652803dace440ed0d`.
- **Verifier and adapter pins** (`script/lib/VerifierPins.sol`) move with the compiler: SP1 v6.1.0 Groth16 verifier `0x6e1a354c…` (5,463 B), RISC Zero v3.0 `0x0d2be77d…`, `SP1VerifierAdapter` `0x56bf1a04…` (1,843 B, immutables at 101, 527, 1165), `RiscZeroVerifierAdapter` `0x30f1011b…` (1,930 B, 101, 526, 1245); new: the OpenVM v2.0 verifier (OpenVM's published build: runtime `0xc164e9a2…`, 19,749 B; initcode `0xf0009263…`) and `OpenVmVerifierAdapter` `0x6f68a310…` (1,859 B, 111, 1651). `SP1_V6_1_0_VERIFIER_HASH` (`0x4388a21c…`) and the RISC Zero parameters digest are unchanged; the SP1 verifier accepts proofs of every SP1 SDK from 6.1.0 to 6.10.0 (circuit v6.1.0).

### Scripts and operations (no ABI change)

- **Reference prover daemons** (`script/lib/prover-lib.sh`): `JournalPrefixMismatch` and `ProofNonceMismatch` release (a proof made for another job or prover), `NotReserved` skips the job. The daemons never claim marker jobs and keep the two-step path: claim, wait for the receipt, then fulfil; they never send `claimAndFulfillAssigned`.
- **Forge scripts.** `script/lib/FulfillGas.sol`: `forClaimAndFulfillAssigned` (the fulfil formula plus `CLAIM_LEG_GAS` = 300,000) and `claimAndFulfillAtQuote`, which sends the one-transaction path only for a job assigned to the broadcaster that is NONE-bound (reserved for its whole life; its caller must be trusted) or carries a binding marker. `script/TestFulfill.s.sol --sig 'claimAndFulfill()'` claims and fulfils a `script/job-submitter.sh` drill job (NONE, assigned to the broadcaster, submitted by the operator) in one transaction; `claim()` then `run()` stay the two-run path.
- **Deploy and `verify`.** Besides the ProgramRegistry and OpenVM items above, `verify` checks that the Registry's `clockSink()` is the router with exactly one `ClockSinkSet`, that every proxy's initializer version and `Initialized` logs are the deploy's (router 3, Registry, Staking, BatchCycleCommittee and ProgramRegistry 1), and names a router `upgradeWithFacets` executed after the deploy as such.
- **Operator tools.** `scripts/check-proposals.sh` judges the combined router op, `undeprecateAdapter`, `OPENVM_V1` adapter ops (VETO on 43111) and ProgramRegistry upgrades against the `PROGRAM_REGISTRY` pin; `scripts/safe-tx.py` writes the combined router upgrade (`router-upgrade --combined`) and `undeprecate`, and `write reenable` names a psId the re-enable leaves Deprecated; `scripts/check-verifier.sh openvm <verifier> --adapter <adapter>` checks the OpenVM verifier and adapter before registration; `scripts/bcc-monitor.sh` pulse-scan pages on `AdapterUndeprecated` and router-op-audit decodes the combined op; the lift gate accepts the recovery batch `[unblacklistVerifier, reactivateAdapter]` in either order; the pause-clock fork check covers `upgradeWithFacets`.
- **CI gates.** The zero-floor gate covers `claimAndFulfillAssigned` (Solidity calls, and `cast send` lines whose signature carries the descriptor tuple); the old-ABI gate refuses `_claimJobExternal(bytes32,address,uint96,uint96)` and `0x1aba486b`. A reflection test requires every router-native function with arguments to be refused as a facet selector, except the 11 keyed mapping getters.

### Downstream impact

- **Provers:** nothing changes for unbound jobs. For a marker job, check the prefix or the seal's nonce word (`getProofNonce`) before you claim; `JournalPrefixMismatch` and `ProofNonceMismatch` mean your proof is for another job or prover (re-prove or release). For a job reserved to you, `claimAndFulfillAssigned` saves a block when it is safe (above). OpenVM jobs: `programId = keccak256(abi.encode(appExeCommit, appVmCommit))`, 32-byte journals.
- **Callers / SDK:** opt into a binding marker to tie proofs to your job and its prover (any proof system but OpenVM for the prefix, SP1 for the nonce); `exclusivityDuration` and `maxBidExtension` are 0 or at least 60 s; a callback must be a contract.
- **Indexers:** decode the renamed extension events, `StakeReduced`, `AdapterUndeprecated`, `ClockSinkSet`, the union-clock events and `ProofDropped`; drop the Registry `PauseClockSet(bytes32,…)`, `DeadlinesExtended`, `DeadlineExtensionSkipped(bytes32)`, the reseed events and the ProgramRegistry events above; `JobClaimed` and `JobFulfilled` can share one transaction; `JobReleased.operator` is always 0.
- **Operators:** `extendLocksForDowntime` while the router is paused, then lift; `undeprecateAdapter` after a re-enable of a deprecated adapter; the combined upgrade op (`proposeUpgradeWithFacets` / `upgradeWithFacets`); 150,000 gas for a pause; `PROGRAM_REGISTRY_FEE` for the deploy and `PROGRAM_REGISTRY` on every TransferOwnership line; the regenerated pins (`DEPLOY_RUNBOOK.md`).

---

## ⚪ No wire or ABI change — 2026-10-23 (reference daemons under a job flood, drill jobs, guardian revocation step 2, `verify` limits)

No `src/` change and no pin move; scripts, docs and tests only.

### The reference daemons' new-job scan
- **Why.** Anyone can submit jobs. A flood of them kept the scan chasing the head, so a lock waiting for a later poll (a
  Registry pause, an adapter disable, a transient fulfil failure) was never served again and was slashed, with no page.
- **The scan budget.** The new-job scan runs at most `SCAN_BUDGET_SEC` per pass (default min(`POLL_INTERVAL`, 20 s)),
  stopping only at block boundaries, and stops early when a tracked lock enters its `LOCK_LATE_SEC` window; the locks
  are then served again without the poll's sleep.
- **The pre-filter.** job-prover skips a job built for another descriptor, or whose `maxPrice` is below
  `MIN_CLAIM_PRICE_WEI`, from its `JobSubmitted` data (words 1 and 4), with no RPC read.
- **`PAGE: SCAN-BEHIND`.** The scan ended short of the head `SCAN_BEHIND_POLLS` (10) polls in a row, or is more than
  `SCAN_BEHIND_BLOCKS` (an hour of blocks) behind; at most hourly. Every failed scan and every job that holds the
  cursor now writes `ERROR_LOG`.

### Drill and smoke jobs
- **Why.** They bound the public fixture statement with no assigned prover, and the fixture's seal is public, so anyone
  could claim them first and the operator's prover never held the lock a drill needs.
- `script/job-submitter.sh` requires `ASSIGNED_PROVER` and submits NONE-bound jobs assigned to that prover. TestFulfill
  (`descriptor(store, prover)`, `plan(address)`, `planFull(address)`) and LiveE2E build the same descriptor for their
  broadcaster, and job-prover builds its descriptor for its own key. A third party's claim reverts `ExclusivityActive`.
- The router refuses bids on NONE jobs, so the competitive "Claim floors" drill runs job-prover with
  `JOB_BINDING=full` (a FULL descriptor for its key).

### Guardian revocation, step 2
- `scripts/safe-tx.py write r19-step2` requires `--part`: `--part disable` writes only the adapter disables and reads no
  bids; `--part blacklist` writes only the verifier blacklist and prints its entry count. The disable file is sent
  first; the blacklist file, at the next Safe nonce, cannot be signed before it executes. The blacklist grows with the
  live bids anyone can place, so it no longer holds the disables back.

### `verify` limits
- `TransferOwnership.s.sol --sig 'verify()'` reads every router and Staking log since the deploy, which grows with
  chain activity and exhausts forge's default gas and memory. Every runbook `verify` command now sets
  `FOUNDRY_GAS_LIMIT=9223372036854775807 FOUNDRY_MEMORY_LIMIT=4294967296`, and `scripts/check-runbook-commands.sh`
  refuses a `verify` line without them.

## ⚪ No wire or ABI change — 2026-10-22 (operator scripts and docs: fees on Hemi, the L2 liveness gate, guardian review, daemon watchdogs)

No `src/` change and no pin move; scripts, docs and tests only.

### Fees on Hemi
- **Why.** No runbook broadcast stated its fees, and Foundry's defaults do not work on Hemi:
  - the default tip is 1 wei, and 43111 never mines a tip of 2 wei or less;
  - during a DA fill the default tip is the filler's own, so the filler holds a pause out for one wei more;
  - a tip without a max fee is refused by cast, and a max fee alone keeps the 1-wei tip.
- **The policy** ("Fees on Hemi" in the runbook): both fee fields on every send.
  - Routine: tip 0.01 gwei, max fee 1 gwei.
  - Urgent (the pause ACTIONs, guardian revocation step 1 when minutes matter): tip 100 gwei, max fee 200 gwei, `--gas-limit 100000` and an explicit nonce; re-sent every block at the same nonce with both fees raised at least 12.5%.
  - `GUARDIAN_PAUSER_MIN_BALANCE`: at least 0.065 ETH per key (the last re-send's admission cost) plus the 43111 L1-fee term, on at least two keys.
- **ACTION lines.** `check-l2-lag.sh` and `bcc-monitor.sh stall-check` append `priority_gas_price=` and `gas_price=` (env `PAUSE_PRIORITY_GAS_PRICE`, `PAUSE_GAS_PRICE`) to the ACTION line; the fields before them are unchanged. A `SEND:` line follows: the full `cast send` with `--gas-limit` (`PAUSE_GAS_LIMIT`, 100,000) and a `<NONCE>` to fill in.
- **A fixed gas limit**: cast estimates gas against the `pending` state. A same-nonce re-send of a pause is therefore estimated where the first has applied: a BCC re-send would be mined out of gas with nothing paused, and a router re-send refused.
- **`scripts/safe-tx.py exec`:**
  - `--priority-gas-price` and `--gas-price` (defaults 0.01 and 1 gwei), and `--tx-nonce` for the executor's transaction nonce (`--nonce` stays the Safe's);
  - its own gas limit, from an estimate at `latest` + 25%;
  - refusals before sending: a fee the chain would leave unmined (a tip below 2 × `eth_maxPriorityFeePerGas`, a max fee below 10 × the base fee), a bad nonce, a sender with unmined transactions;
  - an `EXEC` line with the fees, both nonces and the gas limit.
- **The runbook fee gate.** `scripts/check-runbook-commands.sh` fails a broadcast (`cast send`, `cast mktx`, `forge script --broadcast`, `forge create`, `create-safe.sh`) without both fee fields, and an exported fee variable.
- **Rehearsal.** It records what 743111 does with a 1-wei tip. `deployment-hemi-testnet.csv` drops its stale "2 gwei" row.

### L2 liveness gate
- **The receipt scan.**
  - `check-l2-lag.sh` reports the lag and head readings whatever its receipt scan does, and during a lag page an unreadable backlog reads as the action level.
  - It skips Hemi's system types 0x7c, 0x7d and 0x7e.
  - Receipts are fetched in 20-block batches, halved when the node answers "response too large" (op-geth's 25 MB cap).
  - The harness uses real 43111 and 743111 receipts, and drill (f) runs the script live, read-only, on both chains.
- **Before an ACTION**: the script reads `GT.guardian()`, the relay's `timelock()` and its pause target. When every answering endpoint shows the pause would revert, it prints no ACTION and pages the cause and the L1 hatch.
- **While the router is paused**: `HELD` with exit 3 instead of a page every minute. `PAGE SEQ-WINDOW` fires at half the sequencing window (6 h), then for 300 s each hour.
- **The throttle assumption.** Every reading names op-batcher's default 3.2 MB quadratic throttle and when it was confirmed (`THROTTLE_CONFIRMED_AT`). On 43111 an unset or stale (31 days) date is exit 3, never OK.
- **`PAGE CHAIN-CHANGED`** (exit 2; the lag gate still evaluates): Jovian (`isJovian()`, `daFootprintGasScalar()`), nonzero blob or operator fee scalars, a GasPriceOracle or L1Block version change, extraData, `gasLimit`, a receipt type outside the known set.
- **`RPC_2`** is required on 43111 unless `--allow-single-rpc` is passed.
- **Exits:** 0 OK, 2 PAGE, 1 tooling, and the new 3 (HELD or the throttle unconfirmed), which is never a lift signal. Precedence 2 > 1 > 3 > 0.

### Guardian review
- **EVM errors.** An EVM execution error on a probe of a proposal-supplied address (invalid opcode, out of gas) is judged like a revert: a VETO.
  - Every op is judged on its own; one a failed read stops prints `UNJUDGED`.
  - Exit precedence is 2 > 1 > 3 > 0.
  - An exit 1 is re-run at once, and the second in a row pages (Day 1 step 9, "Monitoring and alerting").
- **Registry upgrade data** is judged as GT ops on the Registry, and pending verifier switches are scanned. Staking and BCC upgrade data are not judged this way yet.
- **The owner rule.** Once the GT has owned a contract, any other `owner()` is a VETO whatever `pendingOwner()` says.
- **`EXPECT_OP_BLOCK`**, required with `EXPECT_OP`: the triggering `OperationProposed` must be in exactly that block. `dual-rpc.sh` passes it and defaults `MIN_BLOCK` to it.
- **Tests:** the swap path and a pending guardian transfer.

### Safe tooling and the lift gate
- **`write reenable`** re-enables by default only what was disabled or blacklisted since the revocation was first proposed (or `--since-block`). Older entries, a psId with a pending swap and a still-blacklisted adapter are NOTEs.
- **`check-lift-tx.sh`** simulates a batch in its execution order in one call and decodes revert reasons. The guardian revocation drill uses two agreed blocks, `PROPOSE_BLOCK` and `EXECUTE_BLOCK`.
- **A verifier swap's `GT.execute`** can be signed through `safe-tx.py`.
- **Tests** at the shipped 600 s safe-lag default.
- **`logs()`** refuses a `null` answer.

### Reference daemons and provers
- **Watchdogs.** Every poll that leaves a lock unsettled ends in one helper, so every line before a certain slash carries a page key:
  - `PAGE: LOCK-LATE` within `LOCK_LATE_SEC` (default `PROVER_GRACE_SEC` / 2) of the lock's last safe moment, whatever the cause (every 300 s per lock), and once past it;
  - `PAGE: LOCK-STUCK` at first sight for a cause the daemon cannot settle (an unknown revert, a release that would revert, demo-prover's lost proof);
  - `PAGE: LOCK-LOST` when a lock is found slashed;
  - `PAGE: RPC-DOWN` after `SCAN_STALL_POLLS` (10) unreadable polls, with an `ERROR_LOG` line on every one; `RPC_TIMEOUT_SEC` is exported as `ETH_RPC_TIMEOUT`;
  - the rate limiter persists in `STATE_DIR/pages`, so a restart does not repeat pages.
  The alert rule is updated in the daemon headers, the runbook's rehearsal notes and `INTERACTION_GUIDE.md` §3a. Rehearsal drill (g) exercises it.
- **Defaults under test**: `SAFE-LAG` at 600 s and `PENDING-UNMINED` at 10 blocks, inside `pl_send` and on anvil (`pl_settings` makes the defaults readable without a chain).
- **Pricing a release.** `INTERACTION_GUIDE.md` §3a and §6:
  - find what holds the fulfil first; a black-hole endpoint is released through the second endpoint at an ordinary fee;
  - price the same-nonce release (about 225,000 gas × (base fee + 1.1 × tip)) against the 95% of the collateral it saves, and send it only when it is cheaper;
  - set `TIP_CEILING_WEI` from what the lock is worth, not the 100-gwei default.
  The `PENDING-UNMINED` page prints that price, §9 gives the cold release gas, and the Oxygen guide follows.

### Tests and CI
- **Forge:** no change (6,110).
- **No new CI step.** The harness comments in `.github/workflows/test.yml` carry the new counts.
- **Harness counts:**

  | Harness | Before | Now |
  |---|---|---|
  | proposals | 171 | 207 |
  | lift | 53 | 60 |
  | Safe signing | 251 | 278 |
  | two-endpoint | 55 | 59 |
  | L2 liveness | 39 | 74 |
  | monitor | 355 | 372 |
  | runbook gate (self-test) | 107 | 130 |
  | prover daemons (1.5.1 and 1.8.5) | 388 | 408 |

## ⚪ No wire or ABI change — 2026-10-21 (verifier adapter pins, block freshness, Safe writers, L2 liveness; scripts, docs and tests only)

**No wire, ABI or bytecode change**: every `abi/*.abi.json` is unchanged, as are the 11 core runtime pins, the BCC pin, the relay pin and the verifier pins; the adapter pins are new constants. Clients built for the 2026-10-17 ABI need no rebuild.

### Verifier adapters
- **Adapter pins.** `script/lib/VerifierPins.sol` pins the deploy-profile runtimes of `SP1VerifierAdapter` (`0xc8cc0d2b…`, 1,844 bytes) and `RiscZeroVerifierAdapter` (`0xb25d3119…`, 1,931 bytes). The one immutable, `nativeVerifier`, is zeroed at its three pinned offsets and must name the verifier that `nativeVerifier()` returns.
- **Where they are required:**
  - TransferOwnership `verify`, for a registered `SP1_V1` or `RISC_ZERO_V1` in any status ("ok adapter pins");
  - `scripts/check-verifier.sh --adapter`;
  - `scripts/check-proposals.sh`, which VETOes a `registerAdapter`, `reactivateAdapter` or `executePendingReactivation` under those ids whose adapter misses the pin.
- **What is refused.** An EIP-7702 designator, an EIP-1167 clone or an ERC-1967 proxy is refused by name: a 7702 EOA delegating to the genuine adapter, or a proxy over it, would pass every other check, then accept a garbage proof after one re-delegation or upgrade.
- **`check-verifier.sh`.** `--adapter` is required: a missing, empty or malformed value exits 1. A tampered fixture is now also sent through the adapter.
- **`script/DeployVerifiers.s.sol`** (new) deploys a verifier and its adapter, or an adapter over an existing verifier. It requires both pins in simulation, before any broadcast.
- **Anchors.** The normaliser of `scripts/check-verifier-anchors.sh` keeps whitespace inside string literals. The ten anchors were re-derived from upstream and are unchanged.

### Governance review and monitors
- **Block freshness** (`scripts/dual-rpc.sh`, `scripts/safe-tx.py`):
  - `MAX_BLOCK_AGE` (1,800 s): an agreed block older than that pages, followed by the leading endpoint's own verdict, marked NOT CROSS-CHECKED;
  - `MIN_BLOCK` / `MIN_BLOCK_WAIT` (300 s): a run an event triggers reviews at that block or later.
- **`check-proposals.sh`:**
  - `EXPECT_OP`: the triggered review must see its op. Plus hourly runs on 43111.
  - Router upgrade data must be empty or exactly one direct `proposeUpdateFacets(F)`, else a VETO naming each element. Once the GT has owned the router, Registry or Staking, `owner()` must be the GT.
  - A failed or `null` read exits 1, and every getter must answer exactly one word.
  - The creation check runs on every rotation's new Safe: GT `transferOwnership`, a new relay controller or non-relay guardian, the router's guardian and elector, a treasury.
  - The adapter pins.
- **`dual-rpc.sh` merge.** A verdict plus a failure keeps the verdict and names the failed endpoint; an unknown exit code is a failure. The harness now runs it over check-proposals, check-safe-config, check-lift-tx and check-pause-clocks-upgrade.
- **Fork cache.** `check-pause-clocks-upgrade.sh` and `dual-rpc.sh` turn off forge's fork state cache, so the `RPC_2` run reads its own endpoint. The clocks check tests DIFF first, and a second RESULT line exits 1.
- **`check-lift-tx.sh`** ABORTs while the safe head lags the head by more than `SAFE_LAG_PAGE_SECS` (600 s), and on a reverted relay `timelock()`.
- **`gt-op.sh`** attributes a cancel from the receipt's Safe events, then the trace, then a direct call. It prints `UNKNOWN` when these are ambiguous.
- **`VerifyLogScanner`.** Its no-log checks share the must-have scan, so an empty log backend cannot pass them.

### Safe tooling
- **User-Agent.** `scripts/safe-tx.py` sends its own `User-Agent`, because the Hemi RPCs answer HTTP 403 to Python-urllib's.
- **Reads at the head.** With `RPC_2`, the nonce is re-read at each head after the cross-check, and `exec` simulates at the heads. A fresh exec, a consumed nonce and a lift right after its pause are now judged correctly.
- **New `write` kinds**: `revoke-guardian`, `cancel-revocation`, `owner-pause`, `reenable` and `router-upgrade --step propose|queue|upgrade|facets`. `router-upgrade` builds the 89-selector facet table from `abi/` and checks it against `script/lib/RouterSelectors.sol`.
- **Safe version.** `check-safe-config.sh`, `safe-tx.py` and `create-safe.sh` accept only 1.4.1 on 743111 and 43111, and `record` installs only a record whose check passed.
- **Transaction service** (43111). `status` records each service confirmation in the ledger, and pages on a deleted proposal and on any delegate.
- **Text fixes:**
  - the scratch-Safe check before 43111 Day 0 sets `.chainId = "43111"` and creates the Safe with `create-safe.sh --version 1.4.1`;
  - the EXECUTED line names the Safe and the safeTxHash.

### L2 liveness and the DA throttle
- **`scripts/check-l2-lag.sh`** (new) reads the safe-head lag (`ts(latest) − ts(safe)`) and the unposted DA backlog (the receipts' `l1GasUsed / 16` over (safe, head]).
  - It pages above 600 s of lag and at 2.5 MB of backlog.
  - At 2.75 MB it prints the `relay.pause(ROUTER)` ACTION for a pauser key.
  - Exits: 0 OK, 2 PAGE, 1 tooling.
- **The procedure** ("L2 liveness and the DA throttle") says who runs the check (every minute, on both endpoints), when to pause and how, and when to lift. **It is a hard gate for 43111**.
- **Threat model, residual 15 (the DA throttle)** is rewritten with:
  - Hemi's batcher gap history and the stall-fill timeline;
  - a competitor's profit, a pre-loaded backlog and L1 forks;
  - the no-stall case: a backlog can build without a stall, and fulfils above about 8.5-17 KB of estimated DA are then filtered whatever they tip.
- **Rehearsal drill (f)** measures the throttle and the batcher's drain on 743111.
- **The daemons** page `SAFE-LAG` and `PENDING-UNMINED`.
- **Provers.** `INTERACTION_GUIDE.md` §6 "The DA throttle": keep fulfils small, tip on short deadlines, check `paused()`, and release at the same nonce.

### Reference daemons
- **`pl_lookback`**:
  - one retry at the same size before halving;
  - the window regrows after 4 successes;
  - the recovery runs in 60-s slices and serves locks between them;
  - `PAGE: LOOKBACK-SLOW` after 10 min.
- **New pages**: `PAGE: SAFE-LAG` when the safe head trails by more than 600 s; `PAGE: PENDING-UNMINED` when an accepted fulfil or release is not mined 10 blocks later.
- **Selectors** are hashed once per process.
- **RPC URLs** in pages and errors are reduced to scheme and host.

### Docs
- **Rehearsal:**
  - `DEPLOY_RECORD` and `REGISTRY_DEPLOY_BLOCK` are recorded at Day 0 step 1;
  - the guardian revocation drill order is fixed.
- **Safe creation.** The creating key is not the deployer's.
- **Registration.** The SP1 and RISC Zero registration steps use `DeployVerifiers.s.sol` and give the 743111 adapter economics.

### Tests and CI
- **Test count.** Forge tests: 6,101 → 6,110 (4 shards, 0 skipped).
  - **New:** 4 in `VerifierPins221i` (the adapter pins, `DeployVerifiers.s.sol`).
  - **New:** 5 in `TransferOwnershipSnapshot`:
    - `verify` refuses an accept-all, a backdoored and a one-byte-off adapter, an ERC-1967 proxy before and after its upgrade, an EIP-1167 clone and an EIP-7702 delegation;
    - `VerifyLogScanner`'s no-log checks.
- **New CI step:** the L2 liveness monitor against a mock RPC (39 checks).
- **Harness fixes:** the Safe signing harness runs the lift gate at the maximum safe-head lag (anvil's safe head trails a day after its 24 h warps; `test-check-lift-tx.sh` tests the ABORT itself), and the daemon harness's second anvil (§33) retries five ports, as its first one does.
- **Harness counts:**

  | Harness | Before | Now |
  |---|---|---|
  | verifier registration | 19 | 32 |
  | verifier anchors (self-test) | 21 | 22 |
  | proposals | 133 | 171 |
  | lift | 48 | 53 |
  | clock upgrade | 54 | 58 |
  | Safe signing | 201 | 251 |
  | Safe creation and configuration | 104 | 111 |
  | two-endpoint | 18 | 55 |
  | L2 liveness (new) | — | 39 |
  | fresh deploy | 306 | 309 |
  | prover daemons (1.5.1 and 1.8.5) | 373 | 388 |

## ⚪ No wire or ABI change — 2026-10-20 (Safe creation and signing, a second RPC endpoint, verifier pins, daemon fixes; scripts, docs and tests only)

**No wire, ABI or bytecode change**: every `abi/*.abi.json` is unchanged, as are the 11 core runtime pins, the BCC pin and the relay pin. Clients built for the 2026-10-17 ABI need no rebuild.

### Safes: creation and configuration
- **`scripts/create-safe.sh`** (new) creates each Safe through the canonical factory, on the SafeL2 singleton with a plain `setup`. It then checks the creation and prints the `creationTx` and `creationBlock` the signing record keeps ("Creating the Safes").
- **`scripts/check-safe-config.sh`** now checks how each Safe was created, not only its views:
  - the canonical SafeProxy runtime (1.4.1 or 1.5.0);
  - exactly one `ProxyCreation` from the canonical factory, with a canonical singleton;
  - a `setup` initializer that is none or the canonical SafeToL2Setup, whose `ChangedMasterCopy` must name the SafeL2;
  - and a direct page on any module execution.
- **The history replay.** It replays every Safe transaction since creation: a DELEGATECALL other than to MultiSendCallOnly pages, and so does a signer outside the roster. It also has `--creation <SAFE> <TX>` and `AT_BLOCK`, which replaces `BLOCK`.
- **The treasury.** A treasury Safe run by the owner signers is exempt from the independent-signer-sets rule and the one-address check.
- **`DeployChecks.safeProblem`** (Deploy, TransferOwnership `verify`) requires VERSION 1.4.1, its **SafeL2** singleton and its fallback handler.
- **SafeL2 is now required for every recorded Safe** on both chains: it logs each transaction's operation and signatures, which the replay reads and which Hemi's Safe transaction service indexes. **The Safes are 1.4.1 on both chains**: the preflight and `verify` refuse a well-formed 1.5.0 Safe, and `scripts/create-safe.sh` warns on a 1.5.0 creation on 743111 or 43111. The signing and monitoring tools still take both versions; CI signs end to end on 1.4.1 and runs `check-safe-config.sh` on both.

### Safe signing: `scripts/safe-tx.py`
- **New subcommands:**
  - `write <veto | router-pause | lift | r19-step2 | register-adapter>` writes the hand-written Safe files from the signing record. It is deterministic at one block, so two signers write byte-identical files.
  - `hash` is the scratch-Safe check before 43111 Day 0.
  - `record` writes the signing record atomically.
- **Second endpoint.** With `RPC_2` set, every read is made on both endpoints at one pinned block (`AT_BLOCK`) and compared: chain id, configuration, version, nonce, stack, GT ops, logs, the hash views, the simulation and the exec receipt. On 43111 a missing `RPC_2` is refused unless `--allow-single-rpc`.
- **Nonce.** `--nonce` is required for `sign` and `exec`, so a lying RPC cannot pick it.
- **Ledger and status.** The ledger path must be absolute, and a missing ledger exits 1 (it used to print OK). On 43111, `status` also reads the Hemi Safe transaction service and pages on any unexecuted proposal with a confirmation.
- **Trezor.** `sign -- --trezor` is refused before the device is asked: Foundry's Trezor signer has no EIP-712 support. Sign with a Ledger; a Trezor can only execute.
- **Tests.** `scripts/safe-tx-hash.sh`'s MISMATCH (exit 2) and every WARN are now asserted. `scripts/test-safe-tx.sh` has 201 checks (was 268): 194 on Safe 1.4.1 (was 133), and a 1.5.0 pass that checks only the deploy's refusal of its Safes.

### Governance review and monitors
- **`scripts/dual-rpc.sh`** (new) runs a governance tool on `RPC` and `RPC_2` at one pinned block whose hash both report, and pages on any disagreement. `AT_BLOCK` pins every read of `check-proposals.sh`, `check-lift-tx.sh`, `check-pause-clocks-upgrade.sh`, `check-safe-config.sh` and bcc-monitor's governance checks. A second, independent endpoint is required on 43111 ("Independent RPC").
- **`scripts/gt-op.sh`** (new) prints a GT op's state, `READY_AT`, expiry and `CANCELLED_BY`.
- **An expired revocation** (past `readyAt + EXPIRY`) is EXPIRED in `check-proposals.sh` and relay-check, never "finalizable".
- **A facet update queued through a router upgrade's data** is decoded and judged instead of a false VETO. New procedure: "Router upgrade and facet update" (order, pause, the clocks gate on both ops, the 48h + 24h timeline).
- **From-blocks** are refused when the contract already has code at the block before, and are checked against `DEPLOY_RECORD`, which 43111 and 743111 now require. `VerifyLogScanner` refuses a scan that starts above the head.
- **pulse-scan** pages `CLOCK` (a clock write without its event) and `EXTEND` (a deadline extension that repeats an id).
- **Guardian revocation texts.** The staker notice says fulfil first. Requesters get their exits during an announced revocation, the guardian procedure gains "unpause Staking", and `check-proposals.sh` NOTEs an owner's Staking `pause()`.

### Verifiers and registration
- **Pins.** `script/lib/VerifierPins.sol` pins the native verifier runtimes of `SP1_V1` (`0xfe91722d…`) and `RISC_ZERO_V1` (`0xac13d438…`), and TransferOwnership `verify` requires them. `verify` also refuses a RISC Zero 2.0/2.1 `SELECTOR` under any psId and a `VERIFIER_HASH` reply that is not one word. A test now asserts all 22 rows of the SP1 advisory table against an independent list.
- **CI anchors.** `scripts/check-verifier-anchors.sh` (CI) anchors `src/vendors/sp1` and `src/vendors/risc0` to upstream: the sources and the SP1 VK.
- **Registration.** `scripts/check-verifier.sh` runs the registration fixture calls (`sp1`, `risc0`). There is a new runbook "RISC Zero registration" step.
- **Provers.**
  - Checklist step 2a: verify that the guest is the job's `programId` (`compute_image_id` / `vk.bytes32`) before running it, not the binary you were given. A decoy with the same journal fails at fulfil.
  - SP1 SDK 6.1.0 to 6.9.0, and risc0-zkvm 3.0.x (Groth16; guests ≥ 3.0.3); re-check on every upgrade.
  - The Groth16 trusted setup is named as a trust root (residual 18).

### Reference daemons
- **Signature cache.** It is seeded with the lib's errors at start, which removes cast 1.8.x's online selector lookups (8 locks: 121 s per poll down to about 1.3 s).
- **`pl_lookback`** retries with backoff and window halving, then pages `LOOKBACK-FAILED`; it never exits silently.
- **Toolchain.** The gate re-runs every poll, and a change pages `TOOLCHAIN-CHANGED`.
- **Revert data.** `pl_revert_data` takes the last data field.
- **`job-submitter.sh`** defaults `MAX_PRICE_WEI` to 20 HEMI and refuses a cap below `MIN_CLAIM_PRICE_WEI`.
- **Harness.** The daemon harness has a per-run timeout. `scripts/test-prover-daemons.sh` has 373 checks on Foundry 1.5.1 and 1.8.5.

### Docs
- **Lifecycle corrections** (`abi/INTERACTION_GUIDE.md` §1 to §8 and §11), each backed by `test/b221i/LifecycleDocs221i.t.sol`:
  - a release within 24h of a slash re-stamps the cancel delay;
  - there is no "operator's release";
  - the release window ends 1 s before the fulfil window;
  - which exits clear exclusivity;
  - three submit and bid validations that were not documented;
  - protocol fees accrue in the router until `withdrawProtocolFees`;
  - the bid-deadline view in a resume-grace tail;
  - repeated ids in `extendDeadlinesForDowntime`.
- **Chain assumptions:**
  - Hemi's `seq_window_size` on 43111 is the OP default, 3,600 L1 blocks (12h).
  - The HEMI bridge's ProxyAdmin is assumed to be a multisig: chain-level trust outside the design (threat model, residual 5).
- **Governance settings:**
  - **Safe 1.4.1** on both chains, enforced by the preflight and `verify` ("Safes" above).
  - **The GT guardian keeps its three `unpause()` pairs**: fast guardian lifts, with a rogue guardian's lift override and pulses for its tenure (up to 14 days) accepted (threat model, residual 17). The 11 emergency pairs are unchanged.
- **Hemi's DA throttle** (threat model, residual 15). The settings are now recorded.
  - **The policy.** With no backlog, op-batcher's defaults cap each 12-s block at 130,000 estimated DA bytes. Under a 3.2 to 12.8 MB unposted backlog the caps tighten to 2,000 per block and 150 per transaction, which holds out any proof-carrying fulfil.
  - **History.** The effective cap was about 260,000 until a Hemi op-geth fix (743111 2026-07-29, 43111 2026-08-18).
  - **Cost.** A hold-out by DA filling costs about 6.1M gas per block against 30M: about 5x cheaper in gas, not "4-22x".
  - **Still before mainnet:** Hemi ops' confirmation and a safe-head-lag monitor.
- **Other notes:** fresh mainnet deployer keys; the 43111 L1 fee (pre-deploy checklist step 6).

### Tests and CI
- **Test count.** Forge tests: 6,122 → 6,101 (4 shards, 267 suites, 0 skipped).
  - **Removed:** 89 empty `test_v1a6_deprecated_*` bodies.
  - **New:** `test/b221i/` (VerifierPins, RiscZeroVerifierVersion, RealVerifierBinding, ProductionComposition, LifecycleDocs) and `test/script/VerifyLogScannerFromBlock221i.t.sol`.
  - **Fixed:** the CycleToleranceBounds access check can now fail.
- **New CI steps:** verifier anchors and their self-test; deploy-profile verifier pins and launch composition; the verifier registration e2e; Safe creation and configuration e2e (104); the two-endpoint e2e (18).
- **Harness counts:**

  | Harness | Before | Now |
  |---|---|---|
  | proposals | 92 | 133 |
  | monitor | 336 | 355 |
  | lift | 45 | 48 |
  | clock upgrade | 44 | 54 |
  | fresh deploy | 299 | 306 |

## ⚪ No wire or ABI change — 2026-10-19 (guardian revocation delay, SP1 verifier v6.1.0, daemon toolchains, the reservation price unit, Safe signing; one GT constant, only the GT pin moves)

**No wire or ABI change**: every `abi/*.abi.json` is unchanged (`scripts/gen-abi.sh --check`), and clients built for the 2026-10-17 ABI need no rebuild. **One bytecode change**: `GovernanceTimelock.REVOKE_GUARDIAN_DELAY`, so the `GOVERNANCE_TIMELOCK` pin in `script/lib/AuditedCorePins.sol` moves (`0xd86dd9e0…db51e4` → `0x0ba78110…c27104`); the other ten `AUDITED_CORE_RUNTIME_KECCAK` pins, `AUDITED_BCC_RUNTIME_KECCAK` and `AUDITED_RELAY_RUNTIME_KECCAK` are unchanged. The vendored SP1 verifier is a separate deployment, not a pinned core contract. Everything else is scripts and docs.

### The GT's guardian revocation delay

**`GovernanceTimelock.REVOKE_GUARDIAN_DELAY`: 6 hours → 14 days**. Fresh deploy only: the GT is not a proxy, and `Deploy.s.sol` deploys it. Every `abi/*.abi.json` is unchanged (the `REVOKE_GUARDIAN_DELAY()` getter and the revocation events keep their signatures). The GT runtime pin `GOVERNANCE_TIMELOCK` in `script/lib/AuditedCorePins.sol` moves from `0xd86dd9e0…db51e4` to `0x0ba78110…c27104`; the other ten `AUDITED_CORE_RUNTIME_KECCAK` pins, `AUDITED_BCC_RUNTIME_KECCAK` and `AUDITED_RELAY_RUNTIME_KECCAK` are unchanged (the `GuardianPauseRelay` edit is a comment). A deployment record's `timelockCodehash` follows the new build.
- **`RevokeGuardianProposed(uint48 readyAt)`**: `readyAt` is the proposal's block timestamp + 1,209,600 s (was + 21,600 s). `finalizeRevokeGuardian()` is accepted from `readyAt` to `readyAt + EXPIRY` (7 days), as before.
- **Why.** `proposeRevokeGuardian` is a direct owner call the guardian cannot veto. At 6 hours a compromised owner Safe could revoke the guardian and swap the Staking authority (a 24h GT op, then 48h) 78 hours after the event, before any staker could leave (7 days). At 14 days the earliest drain is 384 hours after it (against a guardian that vetoes within its 24h deadline), after every staker leg the delay reaches (the longest: 338 h, a partial request pending under a full cap), with 46 h of margin. The rule, the legs it does not reach (a bid kept to its round end then won into a 7-day lock) and the guardian procedure: `DEPLOY_RUNBOOK.md`, "Guardian revocation"; tests: `test/governance/GovernanceTimelockR19.t.sol`.
- **Stakers and integrators**: a `RevokeGuardianProposed` starts your exit window (`INTERACTION_GUIDE.md` §12). Indexers: subscribe to it.
- **Tooling**: `scripts/check-proposals.sh` takes `ALLOW_REVOCATION=<readyAt>` and `scripts/bcc-monitor.sh relay-check` takes `--allow-revocation READY_AT` for the one announced revocation, so its two weeks do not page on every run; any other pending revocation still pages. `scripts/test-bcc-ops.sh` reads the delay from the GT (294 checks), `scripts/test-bcc-monitor.sh` 336, `scripts/test-check-proposals.sh` 92.
- **The cost**: a rogue GT guardian keeps its veto and emergency selectors for up to 14 days plus detection instead of 6 hours (no theft path; `DEPLOY_RUNBOOK.md`, threat model, residual 17).

### The vendored SP1 verifier is v6.1.0

`src/vendors/sp1/` (`SP1VerifierGroth16.sol` = contract `SP1Verifier`, `Groth16Verifier.sol`, `ISP1Verifier.sol`) is now succinctlabs/sp1-contracts tag **v6.1.1** `contracts/src/v6.1.0/` (commit `d3629729c3216eb51bd4859d027a8eb729399fa4`, = main on 2026-10-19), circuit **v6.1.0**: `VERSION()` `"v6.1.0"`, `VERIFIER_HASH()` `0x4388a21c687fdd5f218d7e3d13190cac4c5355818d3605fd5fb811df468ee696`, `VK_ROOT()` `0x002f850ee998974d6cc00e50cd0814b098c05bfade466d28573240d057f25352`. It was sp1-contracts v6.0.0 (`0x0e78f4db…`, `0x008cd56e…`), the circuit of SP1 SDK 6.0.0 to 6.0.2, which advisory GHSA-63x8-x938-vx33 (CVE-2026-40323, high: the V6 recursion circuit's row-count binding gap; fixed in 6.1.0) covers, and which refused every proof of SDK ≥ 6.1.0. Nothing in the HemiProve core imports `src/vendors/sp1`, so no core, BCC or relay pin depends on it. The verifier's ABI is unchanged too (same functions and errors); it is a new deployment with a new code hash.
- **Provers / SDKs** (`abi/INTERACTION_GUIDE.md` §3, §3a prover checklist step 7, §12): SP1 jobs need **SP1 SDK ≥ 6.1.0** in Groth16 mode (tested range 6.1.0 to 6.9.0: pin it and re-check on every SDK upgrade, §3a step 7); the seal (`SP1ProofWithPublicValues::bytes()`, 356 B) starts with `0x4388a21c`. A proof of SDK 6.0.x (`0x0e78f4db…`), a Plonk proof or a TEE-wrapped one reverts `VerifierCallFailed` at fulfil.
- **Operators** (`DEPLOY_RUNBOOK.md`, "After Day 1", "SP1 registration"): deploy the verifier from the deploy-profile build (runtime keccak `0xfe91722d73cdb3b78308da48b1b835e777231157bf114fc6023e7be266034fbf`), check `VERSION()` / `VERIFIER_HASH()` / `VK_ROOT()` before the GT op, and record its code hash in the `SP1_V1` `EXPECTED_ADAPTERS` entry. Never reuse the old stack's v6.0.0 verifier. TransferOwnership `verify` refuses, on 43111 and 743111, any adapter whose `nativeVerifier()` answers an advisory-affected SP1 `VERIFIER_HASH()` (`script/lib/Sp1Advisories.sol`: v6.0.0 Groth16 and Plonk, every release before v5.0.0) and prints "ok SP1 verifiers".
- **Tests**: `test/fixtures/SP1TestReceipt.sol` is a real circuit v6.1.0 proof (sp1-sdk 6.2.4, guest multiply(17, 23); provenance in its header); the v6.0.0 vector moved to `test/fixtures/SP1VersionReceipts.sol` (`SP1V600Receipt`, a negative vector) beside a second independent v6.1.0 proof. `test/integration/SP1VerifierVersion.t.sol` pins the constants, accepts both v6.1.0 proofs, refuses the v6.0.0 one (`WrongVerifierSelector`) and every one-bit change of the public values, program key and proof fields.

### The reference daemons on Foundry 1.5.1 and 1.8.5

`script/lib/prover-lib.sh`, `script/job-prover.sh`, `script/demo-prover.sh`. The 2026-10-18 daemons worked only on Foundry 1.5.1: under cast 1.8 (`cast block --json` now wraps its output in an envelope) they read every head as block 0, so a fresh-`STATE_DIR` restart recovered no live lock and every lock was slashed; under a nightly, cast's stderr banner was parsed as lock data.
- **Toolchain gate**: the daemons start only on a tested Foundry (`cast`, and `forge` for job-prover): 1.5.1 or 1.8.5. `ALLOW_UNTESTED_FOUNDRY=1` starts anyway and writes `PAGE: TOOLCHAIN` to `ERROR_LOG`. `FOUNDRY_DISABLE_NIGHTLY_WARNING=1` is exported for every cast and forge call.
- **Reads**: every chain read is `cast rpc --raw` or a `cast call` with a return signature (byte-identical output on 1.5.1 and 1.8.5). Success data comes from stdout only; a field that does not parse is a retry, never a lock state, and a tracked lock that cannot be read pages.
- **Refusals and pages**: a head at block 0 or timestamp 0 off the dev chains (31337, 1337) refuses to start (`PAGE: HEAD-ZERO`); a fresh `STATE_DIR` with `LOCK_LOOKBACK_BLOCKS=0`, or a recovery from block 0 while the head is above the lookback, pages `LOOKBACK`.
- **Prover hosts** (`abi/INTERACTION_GUIDE.md` §3a "Prover-host toolchain"; `DEPLOY_RUNBOOK.md`, toolchain): install Foundry 1.5.1 or 1.8.5 by version. Your own tooling must not parse `cast block|tx|estimate --json` or `cast receipt … status`.
- **Tests and CI**: `scripts/test-prover-daemons.sh` has 321 checks (was 274). CI runs it twice: on Foundry 1.5.1, then with every daemon on a checksummed 1.8.5 (`DAEMON_FOUNDRY_BIN`).

### The reservation price is in HEMI wei

**`MIN_CLAIM_PRICE_WEI` is HEMI wei** (the job's token), never an ETH amount. The earlier sizing, "this key's ETH cost per job, about 2.4e14 wei", was off by the HEMI/ETH rate (12,500 to 200,000 times too low), which would re-open the own-descriptor drain. `abi/INTERACTION_GUIDE.md` §3a gives the conversion and sizes it for 300-s jobs (the daemons have no timeout filter), with an example: 15 HEMI at 50,000 HEMI/ETH. The 2026-10-17 entry's `MIN_CLAIM_PRICE_WEI` line below carries the unit note.
- **Startup pages**: `RESERVATION-UNSET` (0 on 743111 or 43111) and `RESERVATION-UNIT` (between 0 and 0.1 HEMI); the banner prints the reservation in HEMI.
- **Optional cover monitor**: with `HEMI_PER_ETH` and `HEMI_PER_ETH_EXPIRES` set (and `COVER_GAS_PER_JOB`, default 1,000,000; `COVER_CHECK_SEC`, default 3600), the daemon compares the reservation with the gas a job costs this key (base fee + the 300-s hold-out tip, plus twice the L1 fee bound) and pages `RESERVATION-LOW` or `RESERVATION-RATE-EXPIRED`. It never changes what is claimed.

### Safe signing

The 743111 Safe tool showed only each call's target, selector and length (no device hashes, no warning on a call to the Safe itself), and on 43111 the Hemi Safe service cannot decode the protocol's own calls. One compromised signer who also executes, plus one who trusts the display, could enable a module. This change ships three tools that share no code, and a signing procedure for each chain (`DEPLOY_RUNBOOK.md`, "Signing on 743111", "Signing on 43111"). Safe 1.4.1 and 1.5.0 are supported (1.3.0 is refused); since 2026-10-20 the deploy accepts only 1.4.1.
- **`scripts/safe-tx.py`** (Python standard library; `cast` only to sign and send) builds, signs and executes a Safe transaction from a Transaction Builder batch, an ops-script call list or a Safe proposal. Before any signature it prints the chain id, the Safe (role, version, owners, threshold), the nonce, every SafeTx field, a full recursive decode of every call (through `relay.forward`, the GT's `propose`, `execute` and `emergencyExecute`, router `multicall` and MultiSend) with record labels on every address, the **domain hash and message hash** a hardware wallet shows, and the **safeTxHash**. The domain hash and the safeTxHash must equal the Safe's own `domainSeparator()` and `getTransactionHash()`. It refuses, unless a named flag allows it: DELEGATECALL to anything but the canonical MultiSendCallOnly (`--allow-delegatecall`), a call to the Safe itself or another recorded Safe, or any module, owner, threshold, guard or fallback-handler change (`--allow-safe-config`), gas-refund fields (`--allow-gas-refund`), a future nonce (`--allow-future-nonce`), unknown targets or selectors. Always refused: another chain id, a stale nonce, a non-zero value, a non-canonical singleton. A router or Registry lift runs `scripts/check-lift-tx.sh` with the record's addresses. `sign` records each signature; `status` lists signatures whose nonce is still live and `burn` builds the rejection that voids them (the abandoned-signature rule).
- **`scripts/safe-tx-hash.sh`** (bash + `cast`): an independent recomputation of the three hashes from the raw fields, with its own decode and warnings. On 43111 every signer compares its safeTxHash with the Safe{Wallet} proposal and the other signers out of band before signing.
- **`scripts/check-safe-config.sh`**: each recorded Safe's owners, threshold, modules, guards, fallback handler, singleton and version against the record and the pre-deploy checklist's hygiene rules, and no Safe's signers reaching another's threshold. `--snapshot` writes the record (Day 1 step 11); hourly and on every Safe configuration event, exit 2 = `DRIFT` (page), exit 1 = a tooling error.
- **Tests and CI**: `scripts/test-safe-tx.sh` (268 checks) runs the fresh deploy's Safe files, a router pause, a guardian lift and a guardian veto through real 2-of-3 Safes on Safe 1.4.1 and 1.5.0 bytecode (`test/fixtures/safe/`), checks that the three hash sources agree on every case, and exercises every refusal and flag. `scripts/check-runbook-commands.sh` checks the runbook's `safe-tx.py` calls against the tool's own grammar.

## ⚪ Cosmetic — 2026-10-18 (proposal review, daemon send path, rehearsal drills, doc corrections; no wire, ABI or bytecode change)

**No wire, ABI or bytecode change**: every `abi/*.abi.json` is unchanged (`scripts/gen-abi.sh --check`), and so are the 11 `AUDITED_CORE_RUNTIME_KECCAK` pins, `AUDITED_BCC_RUNTIME_KECCAK` and `AUDITED_RELAY_RUNTIME_KECCAK` (the `src/` edits are comments only; the deploy profile builds with `bytecode_hash = "none"` and `cbor_metadata = false`). Clients built for the 2026-10-17 ABI need no rebuild.
- **Proposal review** (`scripts/check-proposals.sh`). It judges configuration as well as code: a Staking or BCC authority must be the live router `HEMI_PROVE` (address(0) is the cancel); a committee must be a BCC proxy on the pinned runtime, owned by the GT, authorizing the router, with no pending owner, authority or upgrade and the router's token; a facet update must leave the expected dispatch table (`abi/<Facet>` minus `abi/HemiProve`, = `RouterSelectors` 19/7/24/7/32) on pinned facets with nothing else routed, with one line per facet proposal; every BCC a proposal can touch is in scope (the env BCC, the live, pending and proposed committees, `PREVIOUS_BCC`, any BCC proxy a GT op targets); an upgrade-type call to an unknown target and an ownership transfer out of the GT are VETOs. New exit 3: NOTE lines (calls it does not judge: routine setters, lifts, a guardian rotation to a new controller), no VETO; page on exit 3 as on exit 2 and decide the NOTE lines within the op's 24h. The OK line now reads "every proposal judged: …". `scripts/test-check-proposals.sh`: 86 checks (was 34).
- **Reference daemons** (`script/lib/prover-lib.sh`, `script/job-prover.sh`, `script/demo-prover.sh`). Step-down: with the hold-out floor on, a refused send's step-down (at most 40 steps of ÷1.125) ends with one signature at exactly `TIP_START_WEI` (at most 41 steps; unchanged when 40 reach it). A late fulfil or release that the balance pays at `TIP_START_WEI` is now sent instead of being `REFUSED` at every poll until the slash. `REFUSED` now means that signature was refused; its page names it and, for a balance refusal only, the balance the claim gate waits for. Governance reverts: a pause, disable or blacklist revert (`EnforcedPause`, `RegistryPaused`, `AdapterDisabled`, `BlacklistedVerifier`) of the claim simulation or of the `verifyProof` pre-check holds the log cursor instead of skipping the job for good, and a Registry pause no longer deletes retry-set entries (`submitJob` ignores the Registry pause). `scripts/test-prover-daemons.sh`: 274 checks (was 240).
- **Rehearsal drills** (`DEPLOY_RUNBOOK.md`, "Rehearsal drills (743111)"). "Claim floors" and "Bid-phase pause exit" are rewritten so they can be run on 743111 (`CLAIM_COMPETITIVE=1` and a reservation price; `BiddingActive`, then the high-water line; `ClaimBelowFloor` from a hand-sent `cast call` by a third staked key; the bid-phase `gracePauseRelease` call before and after 15 min + 1 s of a ~20-min pause), and the daemon drills follow the send-path and cursor fixes.
- **Doc corrections**, in place in the 2026-10-17 entry below, `abi/INTERACTION_GUIDE.md` and `DEPLOY_RUNBOOK.md`: **residual 16(g)**: k overlapping clocks rewind job-local time by (k − 1) × (overlap + `RESUME_GRACE`), the `reactivateAdapter`, `executePendingReactivation` and `unblacklistVerifier` tails included (ops rule: keep a reactivation or unblacklist more than `RESUME_GRACE` away from a router or Registry lift; a claim retry set must survive about 30 min of falling prices after a three-clock lift); **16(h)**: a round that had already ended at the pause start stays bound for the whole pause, however long; **16(j)**: pause credit earned after the last bid does not move `wallBidDeadline`; **16(d)**: a pause that has run longer than 15 min, not "15 min or more"; a third party's locking `settleBid` needs the round's job-local end and an unpaused router; `GracePauseDelayNotMet`'s parameter is `requiredAfter`, the last second that reverts; live bids in a verifier swap `refundBid` inside the 48h window; the 31,484 B seal room is the smallest of the tabled rows; the stall procedure's late router re-pause saves only locks whose deadline was still ahead at the pause start (also the `bcc-monitor.sh stall-check` PAGE text; its prefix is unchanged); what `verify` attests excludes storage written at construction; `ImplementationHasNoCode` is `code.length > 0` and accepts an EIP-7702 delegated EOA, with the guardian's pin check as the control and the bytecode deliberately unchanged.
- **CI.** `scripts/test-chain-override.sh` no longer leaks its two anvils (their pids now reach the EXIT trap).

---

## 🔴 Breaking — 2026-10-17 (claim floors, binding bids, no premium on competitive jobs, minimum bid rounds, the 7-day bid-phase pause exit)

Fresh deploy only, like the 2026-10-13 to 2026-10-15 changes. Rebuild every decoder from the regenerated `abi/*.abi.json` (`scripts/gen-abi.sh`): every claim selector, `slashAndClaim`, the live bidder's exit and `JobStatusView` change. Integrator-facing semantics are in `abi/INTERACTION_GUIDE.md` (§3a prove first with exact floors, §4 claim floors and binding bids, §5 the premium, §6 the exits and the `slashAndClaim` rescue, §8 events); operator rules and the residuals in `DEPLOY_RUNBOOK.md` ("Rehearsal notes", threat model, residuals 13 and 16, "Guardian review of proposals", "Regression coverage and CI").

### 🔴 Wire format

| Item | Before (2026-10-15) | Since 2026-10-17 |
|---|---|---|
| `claimJob` (Core) | `claimJob(bytes32 jobId)`, `0x5fcec5c6` | `claimJob(bytes32 jobId, uint96 priceFloor, uint96 premiumFloor) returns (uint96 price)`, `0xb2f51925` |
| `claimJobDelegated` (Core) | `(bytes32 jobId, address prover)`, `0x0d45f121` | `(bytes32 jobId, address prover, uint96 priceFloor, uint96 premiumFloor) returns (uint96 price)`, `0x363e6708` |
| `claimJobBatch` (Core) | `(bytes32[] jobIds)`, `0xedba853e` | `(bytes32[] jobIds, uint96[] priceFloors, uint96[] premiumFloors) returns (bool[] results)`, `0xf100ca00` |
| `_claimJobExternal` (Core, self-call only) | `(bytes32, address)`, `0x30cc563e` | `(bytes32 jobId, address prover, uint96 priceFloor, uint96 premiumFloor) returns (uint96 price)`, `0x1aba486b` |
| `slashAndClaim` (Fulfill) | `(bytes32 jobId)`, `0x2068a8f5` | `slashAndClaim(bytes32 jobId, uint96 priceFloor) returns (uint96 price)`, `0x5cb21b7e` |
| The live bidder's exit (Aux) | `forfeitBid(bytes32)`, `0x02ef46c6` | **removed**; **`refundBid(bytes32 jobId)`**, `0x427163dc`, free and only while a governance cause holds (`RouterSelectors.aux()` still has 24 selectors) |
| `getJobSnapshots(bytes32 jobId)` (FulfillCycle) | none | **added**, `0xc06e305c`: `(uint256 releasePenaltyBps, uint256 minCollateral, uint256 collateralBps)`, routed (`RouterSelectors.fulfillCycle()` has 7 selectors) |
| `getJobStatusView(bytes32)` (`0xbc4971c6`, selector unchanged) | `JobStatusView`, 23 members | **22 members**: `uint96 totalProverReward` (the 7th, between `bonusAmount` and `settledPrice`) is gone and every later member moves up one position |
| `HemiProveStaking.forfeitCollateral(address,uint128)` (`0xeb851d53`, HemiProve only) | the forfeit burn | **removed** (`IHemiProveStaking` too) |
| event `BidForfeited(bytes32 indexed jobId, address indexed bidder, uint96 burnedCollateral)` | emitted by the forfeit | **removed** from every ABI file |

- **Old selectors fail closed.** The router has no route for `0x5fcec5c6`, `0x0d45f121`, `0xedba853e`, `0x30cc563e`, `0x2068a8f5` or `0x02ef46c6`: a call reverts `Error("Unknown selector")`. Nothing is overloaded, so an unmigrated client cannot claim without a floor by accident.
- **New errors.** `HemiProveCore.abi.json` gains `ClaimBelowFloor(bytes32 jobId, uint96 price, uint96 priceFloor, uint256 premiumCap, uint96 premiumFloor)` (`0xc8cea62a`), `SpeedPremiumOnCompetitiveJob()` (`0x0134e701`) and `CustomVerifierIsDelegated(address verifier)` (`0xc2b70ea0`); `HemiProveAux.abi.json` `NoGovernanceCause(bytes32 jobId)` (`0x2bc6444e`); `HemiProve.abi.json`, `HemiProveAuxGov.abi.json`, `HemiProveRegistry.abi.json` and `HemiProveStaking.abi.json` `ImplementationHasNoCode(address implementation)` (`0x9e172b1d`); `HemiProveRouter.abi.json`, the union, all five. `slashAndClaim` bubbles `ClaimBelowFloor` from its Core self-call, so decode its reverts with the union. `ClaimBelowFloor.premiumCap` is always the live cap (`getPremiumCap`), also when `premiumFloor` is 0.
- **Reused errors, new conditions.** `InvalidBidConfig()` (below); `GracePauseDelayNotMet(bytes32 jobId, uint40 requiredAfter)` (`0x63f4a133`) with `requiredAfter = pauseStart + 7 days` on the bid-phase `gracePauseRelease` (`requiredAfter` is the ABI's name, which this entry gave as `readyAt`; it is the last second that reverts, so the exit opens at `requiredAfter + 1`, and inside the first 15 min the shared check reports `pauseStart + 15 min` first: retry at the value + 1 and re-read the error); `ArrayLengthMismatch(uint256, uint256)` for `claimJobBatch`'s floor arrays.
- **Constants** (`src/libraries/Constants.sol`, file-level, no getters): `MIN_UNDERBID_WINDOW = 5 minutes` and `MIN_BID_PAUSE_EXIT_DELAY = 7 days`.
- **Storage.** No layout change (`.storage-layout/` unchanged). `settlementTimestamp` and `competitiveBidStartTime` are still written and readable; no premium path reads them.

### 🔴 Claim floors: every lock states the lowest price it accepts

- **The rule.** Every lock (`claimJob`, `claimJobDelegated`, each `claimJobBatch` entry, and the claim leg of `slashAndClaim`) lands with `settledPrice ≥ priceFloor` and a premium cap `≥ premiumFloor` at inclusion, or reverts `ClaimBelowFloor` with nothing locked (in `claimJobBatch` only that entry fails). The test runs after the price is computed and before any collateral is locked. The claim returns the price.
- **Why.** A claim is priced at inclusion. Any reset ordered ahead of a pending claim used to lock it at the lower price: a bid round that ends without a lock (`submitBid` + the forfeit in one block), or a floor bid, `settleBid` after the round and a zero-hold `releaseJob` (about 5 HEMI at the 5% penalty), or a free pause exit. With a floor such a reset only delays the claim, which reverts and is re-sent.
- **The floors are exact (tolerance 0).** `priceFloor` = the price an `eth_call` of `claimJob(jobId, 0, 0)` from the claiming address returns at a **mined** block; `premiumFloor` = `getPremiumCap(jobId)` at that same block. Never `jobs(jobId).minPrice`, a `pending` read, 0, or `getCurrentPrice` (it reads 0 or the frozen ceiling while a bid is live or the job is Locked). No tolerance either, also in a `RESUME_GRACE` tail: once a router pause has run longer than 15 min (not "15 min or more": the exit opens at the pause start + 15 min + 1 s) every lock holder has a free exit (the Locked `gracePauseRelease`), so any tolerance is extractable for free. An honest claim still lands: inclusion only climbs the Dutch ramp, and the cap never rises (`speedPremium` has one writer, the submit, and `premiumUsedPpm` only grows), so an exact premium floor never reverts spuriously. A floor of 0 is accepted on-chain (a `minPrice = 0` job is legitimately priced 0) but leaves the claim repriceable; the CI grep gate refuses literal-0 floors in everything operators run (below).
- **`ClaimBelowFloor` and `BiddingActive` mean wait and retry.** Re-quote at a later mined block; never chase the price down. Keep a per-job price high-water mark (the highest floor you sent for the job), never claim below it, and forget it only on your own lock, a terminal job or `MAX_REOPEN_COUNT`, never because another prover locked the job (a sybil's settle is such a lock). Never ratchet the premium floor.
- **`claimJobBatch`** takes floor arrays parallel to `jobIds`: `ArrayLengthMismatch(n, length)` for either, after `EmptyBatch` and `MaxBatchSizeExceeded`. A `false` entry carries no reason (no reason array, to save Core bytes): re-simulate that entry alone with `claimJob` before classifying it.
- **`claimJobDelegated`.** The operator (PERM_CLAIM-trusted) chooses the floors. A delegated claim without floors is repriceable like a direct one: a settle + release reset leaves `lastReopenTimestamp` 0, so the 24h delegated-claim cooldown does not fire.
- **`slashAndClaim(jobId, priceFloor)`** binds the rescue to the lock the rescuer observed. Floor = P(`elapsedAtLock`) of that lock: the value an `eth_call` of `slashAndClaim` returns at a mined block at or after `slashOpensAt`, or `AuctionMath.computePrice(minPrice, maxPrice, rampUpPeriod, curveType, elapsedAtLock)` from `jobs(jobId)` read while that lock is live. Compute it once and persist it: a later bid drop can zero `jobs(jobId).elapsedAtLock` (below). The claim leg's premium floor is 0 (the slash spends the whole budget). A rescue still pending across a later lock (a keeper slashes the observed lock first, a sybil wins a `minPrice` round, that lock expires) now reverts `ClaimBelowFloor` and the slash unwinds; before 2026-10-17 it locked at the later lock's lower price. If a keeper slashed first, fall back to `claimJob(jobId, floor, 0)`.
- **Typed self-calls.** `claimJobBatch`, `slashAndClaim` and `cancelJob`'s auto-slash encode their router self-calls with `abi.encodeCall`, so a signature drift no longer compiles (`_executeSlashExternal` keeps `0xc36c643d`); a CI grep gate refuses string-encoded or untyped self-calls in `src/core`.
- **Calldata.** `claimJob(jobId, P, Q)` is 64 B and `slashAndClaim(jobId, f)` 32 B longer than before. The unsafe, never-send `multicall([claimJob(jobId, P, Q), fulfillJob])` with a 64 KiB descriptor and a full `MAX_JOURNAL_BYTES` journal leaves ≥ 31,484 B for the seal, now the smallest room of the tabled rows (`test/fixes/DescriptorCap.t.sol`: wrapper budget 352 B, headroom 64,252 B, 66,564 B measured; "of any row" was too wide: the untabled, equally unsafe `multicall([claimJobBatch([jobId], [P], [Q]), fulfillJob])`, whose claim leg is 292 B instead of 100 B, leaves ≥ 31,292 B, and was already below the earlier bound); `multicall([slashAndClaim(jobId, f), fulfillJob])` is 932 B fixed and 99,585 B before the seal (≥ 31,487 B). The safe rows are unchanged (smallest room 31,551 B). The earlier "≥ 31,519 B" bound no longer holds for the unsafe rows.

### 🔴 Binding bids: `refundBid` replaces the forfeit

- **A live bid is binding.** A bid round ends only in a lock (`settleBid`) or an outbid, unless a governance cause holds (the adapter Disabled or Unregistered; for a `CUSTOM_VERIFIER` job the verifier blacklisted) or a router or Registry pause has run longer than 7 d (below). The 2026-10-15 forfeit is gone, both branches, with `Staking.forfeitCollateral` and `BidForfeited`: its burn branch (100% of the soft-lock burned before the bid deadline, `reopenCount + 1`, `lastReopenTimestamp` stamped, exclusivity and the assigned prover cleared) let any bidder end a live round at a price nobody stood behind, which repriced every pending claim and cut the round. This supersedes the 2026-10-13 rule "forfeit only before the bid deadline".
- **`refundBid(jobId)`** (Aux, routed, `0x427163dc`; no pause gate, like `releaseJob`, `graceRelease*` and `gracePauseRelease`, and unlike `settleBid`, whose governance-cause branch is `whenNotPaused`: under a router pause with a cause, `refundBid` is the bidder's only refund; this entry said "like every exit"): the live bidder's free refund while a cause holds, with exactly `settleBid`'s governance-cause predicate. `JobNotOpen(jobId)` unless Open; `NotJobProver(jobId, msg.sender)` unless the caller is the live bidder (also when no bid is live); `NoGovernanceCause(jobId)` without a cause. The soft-lock is unlocked (nothing burned), the post-activity anchor is stamped, the Dutch ramp resumes at the best live bid with the round marker cleared, and `BidInvalidated(jobId, bidder)` is emitted.
- **The self-cooldown is keyed on the wall clock.** The bidder's own governance-cause refund (`refundBid`, or its own `settleBid` under a cause) stamps its 1h release cooldown on the job only before `wallBidDeadline` (`block.timestamp < wallBidDeadline`, read before the drop). Before it only the bidder can take the refund (a voluntary exit); from it anyone gets the identical refund through `settleBid` without a cooldown, and the cause freezes job-local time, so the 2026-10-15 rule cooled an honest winner settling after the wall deadline while a sybil's identical settle was not cooled. A third party's settle never cools the bidder. The bid-phase `gracePauseRelease` still stamps it.
- **A codeless custom verifier is no longer a cause** (at `settleBid` or `refundBid`). Under EIP-6780 (cancun) code a bidder saw at `submitBid` cannot vanish later: only the caller's own same-transaction CREATE2/SELFDESTRUCT verifier can be codeless, and that must not be a free bid drop. Submit now refuses EIP-7702 designators (below), and `submitBid` and the claim paths still refuse a codeless verifier (`CustomVerifierHasNoCode`).
- **The walk-away** from a live bid is: wait for the round to end, `settleBid` (anyone may, once the round has ended in job-local time and the router is unpaused: `getJobStatusView(jobId).bidDeadline`, which equals `wallBidDeadline` unless pause credit accrued after the last bid (a pause, or a resume-grace tail still running at the bid); this entry said "once the wall clock passes `wallBidDeadline`", but a locking settle needs the job-local deadline and `settleBid` is `whenNotPaused`), then fulfil or `releaseJob` at the snapshotted penalty, `releasePenaltyBps × max(S × collateralBps / 10,000, minCollateral) / 10,000` for a winning bid S, all from `getJobSnapshots` (5% by default; governance can lower the penalty floor to 1%). `cancelJob` still reverts `BidsActive` while a bid is live; `cancelStaleJob` is not a bidder exit (it needs a Disabled adapter, a blacklisted verifier or `MAX_REOPEN_COUNT`).
- **The settle top-up is gone.** Settle locks the bid's soft-lock as the lock collateral: the soft-lock (`BidMath`) and the settle requirement (`AuctionMath.computeCollateral`) are the same formula on the same per-job snapshots, and the settled price is the bid. The top-up's failure invalidation (`BidInvalidated` at settle) was unreachable and is gone.
- `cancelStaleJob`'s job age loses the forfeit's `lastReopenTimestamp` stamp (`max(lastReopenTimestamp, rampUpStart + elapsedAtLock)`; `lastReopenTimestamp` is now stamped only by a slash (there is no operator's release), a direct release clears it unless one was set in the last 24h (then it re-stamps it), and `graceRelease*` clears it).

### 🔴 No speed premium on competitive jobs

- Submit reverts `SpeedPremiumOnCompetitiveJob()` when `underbidWindow > 0` and `speedPremium != 0`, on every submit path (the presets carry no bid config). Premium jobs are Dutch-only: every lock of a premium job is claim-won, `u = speedPremium × remaining / fulfillmentTimeout`, and the 2026-10-15 cumulative budget is exactly the one-lock model: claim-won holds h1, …, hk then a fulfil at h pay `max(0, speedPremium × (1 − (h1 + … + hk + h) / T))`. `getPremiumCap` is 0 on a competitive job.
- The bid-won denominator `fulfillmentTimeout + (settlementTimestamp − firstBidTime)` is gone, and with it a gap: a bid round that ended without a lock was never charged to the budget, so a sybil could claim right after the drop and collect the full claim-won premium.
- **Superseded:** the 2026-10-15 premium entry's bid-won `D` and its worked row, and the 2026-10-13 bid-won budget remarks (below).

### 🔴 Minimum bid rounds

- A competitive submit (`underbidWindow > 0`) reverts `InvalidBidConfig()` unless `MIN_UNDERBID_WINDOW (300 s) ≤ underbidWindow ≤ maxBiddingDuration ≤ MAX_BIDDING_DURATION (7 d)` and `maxBidExtension ≤ 24 h`. The `require` string "maxBiddingDuration required" is gone (0 is `InvalidBidConfig`), and so is the first-deadline cap at `firstBidTime + maxBiddingDuration` (dead code under the ordered bounds).
- Every round's first deadline is exactly `firstBidTime + underbidWindow` in job-local time; pauses and joint-lift tails only lengthen it. Anti-snipe extensions stay capped at `firstBidTime + maxBiddingDuration`, so they need `maxBiddingDuration > underbidWindow` (with equality they are no-ops: the caller's choice).
- Hygiene and defence in depth, not an economic control: every settle-based reset takes ≥ 300 s of job-local time (about 25 Hemi blocks), so resetting a pending direct claim needs 25 consecutive blocks without it plus being ordered ahead of it in the 26th. It does not bound the job kill (ten claim + release pairs in one block), governance-cause drops, joint-lift rewinds or clients that chase. Hemi timestamps follow block height, so a back-dated catch-up burst after a sequencer stall compresses 300 s of chain time into seconds.

### 🔴 The bid-phase pause exit needs a pause longer than 7 days

- `gracePauseRelease` on an Open job (the live bidder drops its bid) reverts `GracePauseDelayNotMet(jobId, pauseStart + 7 days)` until the pause has run longer than `MIN_BID_PAUSE_EXIT_DELAY` = 7 d (inside the first 15 min the shared check reports `pauseStart + 15 min`). It reverts at exactly `pauseStart + 7 d` and succeeds one second later. `pauseStart` is the earlier of the counting pause starts (the router; the Registry too for an adapter job) and re-anchors at each new pause, so pulses never accumulate. The round-ended check (`BidDeadlinePassed`) follows it as before. The Locked branch keeps `MIN_GRACE_PAUSE_DELAY` = 15 min.
- **Why.** A router pause can be fired with no timelock by a hot pauser key through the GuardianPauseRelay (the router is an allowlisted relay target) and by the routine stall procedure. With the 15-min bid exit one such pause let every live leader drop for free and a sybil take each job at no more than its own bid in the lift block, while pending honest underbids reverted `JobNotOpen`. Now a routine pause (stall, hot key, the 24h GT unpause op) only freezes a live round: at the lift it resumes with its remaining job-local time plus the pause credit and `RESUME_GRACE`, and pending underbids land. A pause longer than 7 d (equal to `MAX_BIDDING_DURATION`) is a protocol halt, and the exit then frees the soft-lock of a round still live at the pause start; a round that had already ended at the pause start stays soft-locked for the whole pause, however long (`BidDeadlinePassed`, unchanged since 2026-10-13), and its winner settles after the lift.
- **A router pause is still not loss-free** (residual 16(d)). After 15 min of any router pause (or a Registry pause, for adapter jobs) every lock holder has the penalty-free Locked exit: a free reset of any lock to P(`elapsedAtLock`), which only delays floored claims (hence tolerance 0), and a free Dutch premium strip of `sp × h / T`, charged at the pause start and refunded to the caller (pending exact premium floors revert). `GuardianPauseRelay`'s NatSpec says so now (comment-only; its runtime pin is unchanged).

### 🟡 Smaller changes

- A bid drop that moves `rampUpStart` forward (`_resumeAtBestBid`: `refundBid`, the governance-cause refund in `settleBid`, the bid-phase `gracePauseRelease`) zeroes `jobs(jobId).elapsedAtLock`. `cancelStaleJob`'s base point `rampUpStart + elapsedAtLock` is then at or before the wall time of the drop, and for a governance-cause drop at or before the key clock's anchor `since`, so the stale cancel opens at `since + 7 days + 1`. The 2026-10-15 build left a stale `elapsedAtLock` that dated the base point into the future without bound. Read a lock's `elapsedAtLock` while that lock is live.
- Submit refuses a `CUSTOM_VERIFIER` whose code is an EIP-7702 delegation designator (23 bytes, `0xef0100 ‖ impl`): `CustomVerifierIsDelegated(verifier)`. Its key holder could clear and restore the code at will (a claim veto, an unfulfillable lock, and before 2026-10-17 a free bid drop while codeless). A deployed contract's code cannot start with `0xEF` (EIP-3541), so there are no false positives; on a chain without EIP-7702 the check never fires.
- `getJobSnapshots(jobId)`: the job's submit-time release penalty (bps), collateral floor and effective collateral bps, each decoded from its +1-offset snapshot (or the live value for a pre-snapshot job) exactly as the release, claim, bid and settle paths read them. For pricing bids and the binding-bid walk-away, and for monitoring residual costs. Not a floor tolerance (floors use tolerance 0).
- `JobStatusView.totalProverReward` (`settledPrice + bonusAmount` for a Locked job) is dropped: it omitted the speed bonus and the protocol fee, and `bonusAmount` has been 0 since 2026-10-13. Read `settledPrice`, `getPremiumCap` and `getJobSnapshots`.
- Timelocked proposals must name code: `ImplementationHasNoCode(implementation)` at propose and again at execute or accept, for the router (`proposeUpgrade` / `upgradeToAndCall`; `proposeUpdateFacets` / `updateFacets`, every non-zero facet), the Registry and Staking (`proposeUpgrade` / `executeUpgrade`), Staking `proposeAuthorizedHemiProve` / `acceptAuthorizedHemiProve`, and AuxGov `proposeBatchCycleCommittee` / `acceptBatchCycleCommittee` (address(0) is still the cancel). A proposal that commits only to an empty address (a factory's next CREATE address) gave the guardian's veto window nothing to review. The bootstrap setters (`setBatchCycleCommittee` before a committee exists, `setAuthorizedHemiProve`) are not checked. The BatchCycleCommittee is untouched (its bytecode stays frozen at pin `0x5d424f24…71ff`): its proposals are reviewed off-chain with `scripts/check-proposals.sh` (below), which is also the only check that a proposal names a pinned build.

### Bytecode and pins

Deploy-profile runtime sizes (was, in the 2026-10-15 build): router 19,713 B (19,452), Core 21,956 (21,261), Aux 21,979 (23,044), Fulfill 21,249 (21,378), FulfillCycle 7,538 (6,837), AuxGov 13,709 (13,675), Registry 11,971 (11,898), Staking 11,644 (11,802). Every facet keeps at least 2,597 B of EIP-170 headroom (Aux). BatchCycleCommittee (23,922 B) and GuardianPauseRelay (1,581 B) are byte-identical: `AUDITED_BCC_RUNTIME_KECCAK` and `AUDITED_RELAY_RUNTIME_KECCAK` do not move. All eight `AUDITED_CORE_RUNTIME_KECCAK` pins move (the router, Registry and AuxGov through `ImplementationHasNoCode`), and the set gains the GovernanceTimelock, the ERC1967Proxy and ProofResultStore (11 pins, below).

### Scripts and operations (no ABI change)

- **Reference prover daemons** (`script/job-prover.sh`, `script/demo-prover.sh`, `script/lib/prover-lib.sh`). **The 2026-10-15 daemons must not run against this deploy** (their claim selector is unrouted) and the rehearsal does not run them.
  - **Claims.** `claimJob(bytes32,uint96,uint96)` at exact floors (the `eth_call` price at a mined block and `getPremiumCap` there; tolerance 0), never below `MIN_CLAIM_PRICE_WEI` (the reservation price, decimal wei of any size; default 0: set it; HEMI wei, the job's token, never an ETH amount: `INTERACTION_GUIDE.md` §3a sizes it) or the job's persisted high-water mark (`STATE_DIR/claim-highwater`). `ClaimBelowFloor` and `BiddingActive` put the job in a bounded claim retry set (`STATE_DIR/retry`: `CLAIM_RETRY_CAP` 256 entries, `CLAIM_RETRY_PER_CALLER` 16, `CLAIM_RETRY_BUDGET` 4 re-checks per poll, strictly after the key's own locks) and the log cursor advances; a claim refused for funds holds the cursor (the funds gate re-reads the balance before every claim). `CLAIM_COMPETITIVE=0` (the default) skips competitive jobs. After a mined claim `JobClaimed.settledPrice` must be ≥ the floor sent (else `PAGE: CANARY`). Wei values are compared as decimal strings (the 2026-10-15 `max()` overflowed int64). The daemons send no batch and no `slashAndClaim` rescue.
  - **Sends.** pl_send re-publishes the live signature every block until mined or the give-up, also at the tip ceiling (no latch), and also the last accepted signature while a publish fails; it escalates tip and maxFee by 12.5% only on a real miss (one of its signatures was pooled), at most one step per 12 s of wall time; it caps maxFee at `RPC_TX_FEE_CAP_WEI` / gasLimit (default 1 ETH, op-geth's `--rpc.txfeecap`); a refused bump falls back to the last accepted signature (`ESCALATION-CAPPED`), a refused accepted one steps down (`STEP-DOWN`), a send refused before any acceptance returns (`REFUSED`) and holds new claims; fulfils and releases start no lower than the hold-out floor `HOLDOUT_FLOOR_WEI` / (blocks left × (block gas − gasLimit)) (`HOLDOUT_FLOOR_WEI` default 1 ETH); it signs locally; the wall clock never ends an accepted send (a halted chain keeps it pooled for the catch-up). The 2026-10-15 text "holding it out costs about 1.125^150 times the starting tip" ignored every cap: see the cost model in `abi/INTERACTION_GUIDE.md` §6.
  - **Ownership.** One sender per staked key: a host-local instance lock (`INSTANCE_LOCK_DIR`, default `~/.hemiprove`; pid plus boot id and start time) refuses a second daemon on the same key and host. Every `JobClaimed` of the key is adopted, but a lock whose descriptor the daemon cannot build (a settled bid, a hand, rescue or delegated claim) gets one `FOREIGN` line and is never fulfilled, released or sent for; `InvalidDescriptorHash` releases only a lock the daemon claimed itself.
  - **Release rule by job type.** For a `CUSTOM_VERIFIER` job only `isBlacklisted(verifier)` is a governance cause (a Registry pause neither blocks nor credits it): its `VerifierCallFailed` releases (5%) unless the verifier is blacklisted. An adapter that reads Unregistered is no cause either (released, and paged).
  - **Recovery.** A fresh `STATE_DIR` always recovers the key's locks from `LOCK_LOOKBACK_BLOCKS` back ((7 d + 24 h + `PROVER_GRACE` + 7 d) / block time); `FROM_BLOCK` moves only the new-job scan.
  - **New env**: `RPC_TX_FEE_CAP_WEI`, `CAP_RETRY_BLOCKS` (25), `HOLDOUT_FLOOR_WEI`, `BLOCK_GAS_LIMIT` (the head's), `MIN_CLAIM_PRICE_WEI`, `CLAIM_COMPETITIVE`, `CLAIM_RETRY_CAP`, `CLAIM_RETRY_BUDGET`, `CLAIM_RETRY_PER_CALLER`, `INSTANCE_LOCK_DIR`; bash ≥ 4.4. Alert on `PAGE` (`CANARY` lines included), `HEAD-STALE`, `ESCALATION-CAPPED`, `STEP-DOWN`, `REFUSED`, `FOREIGN` and `LOCK-LOST` in `ERROR_LOG`. CI: `scripts/test-prover-daemons.sh`, 240 checks on anvil (was 85), with an op-geth-like proxy for the 10% replacement rule, "already known" and the fee cap.
  - Still open (pre-mainnet): one RPC endpoint for the head, reads and publishes (multi-endpoint publishing with per-endpoint classification and `HEAD-STALE` failover), an L1-fee-aware balance reserve, the claim TTL-cancel (a claim the daemon stops waiting for can stay pooled and land later at its embedded floor, which was ≥ the high-water mark when signed: the mark binds new sends only), and a durable, non-serial send lane.
- **Forge scripts.** Every forge-script claim passes the exact floors `script/lib/FulfillGas.sol` `quoteClaim` simulates (a `claimJob(jobId, 0, 0)` by the claiming account at the forked block, rolled back, and `getPremiumCap` there), never a literal 0: `claimAtQuote` quotes and claims in one call (`TestFulfill.s.sol` `claim()`, which prints `CLAIM_PRICE_FLOOR` and `CLAIM_PREMIUM_FLOOR`; `LiveE2E.s.sol`, `DemoInteraction.s.sol`, `V3A2Cycle.s.sol`, `V3A3Cycles.s.sol`, `V3A9SlashReopen.s.sol`, `RealGroth16.s.sol`), and `slashFloor` gives the `slashAndClaim` rescue floor (computePrice at the live lock's `elapsedAtLock`). A live-chain claim step (`requireClaimable`) refuses a job that is not Open or whose descriptor hash is not the one its fulfil run rebuilds, and takes job ids from the **mined** `JobSubmitted` logs of the submit run's receipts, never from a simulated `submitJob` return: `LiveE2E.s.sol` `runSubmitClaim` is split into `runSubmit` and `runClaim(address,bytes32[])` (then `runFulfillLocked`, `checkFulfilled`), and `DemoInteraction.s.sol` `step2_submitJob` prints the command that reads `JOB_ID` from its receipt. `DemoBootstrap.s.sol` takes the dev-chain gate (`HemiScript._requireDevChain`): it refuses 743111 and 43111, where its price seed is an owner setter that fails `verify` for good. `scripts/test-fresh-deploy.sh`: 299 checks (was 286).
- **Deploy, verify and ops tooling.**
  - The router is a stall re-pause target: DeployBCC's `2-guardian-relay.json` adds `relay.setPauseTarget(ROUTER, true)`, relay-check takes `--allow-target "$ROUTER"`, and a second stall broadcaster runs `stall-check` on the router. On the router, `stall-check` withholds the ACTION only while the Registry is paused at the head (a Registry `RESUME_GRACE` tail or a rising key clock is a `NOTE stall-check:` line), runs its key scan after printing the ACTION, keeps a lossy ACTION until the router's `RESUME_GRACE` anchor + `PROVER_GRACE` − 60 s, and pages an `OVERLAP` once. `pulse-scan` takes `--page-from-block S` (each overlap pages once) and pairs the Registry's global clock with every adapter clock. `scripts/test-bcc-monitor.sh`: 330 checks (was 301).
  - `AUDITED_CORE_RUNTIME_KECCAK` has 11 pins: the eight core runtimes, the GovernanceTimelock, the OpenZeppelin ERC1967Proxy (every core and BCC proxy) and ProofResultStore (its `hemiProve` zeroed). `Deploy.s.sol` checks the GT, the proxies and the store it deploys; the deployment record is version 2 (it carries their runtime keccaks; a version-1 record is refused); TransferOwnership `verify` checks them, prints "ok GT, proxies and store", and replays `EmergencySelectorUpdated` to require exactly the expected GT emergency selectors (the 11 pairs, plus (BCC, `pause()`) after the relay install, plus (BCC, `unpause()`) only with `OPTION_A=1`). On 743111 DeployBCC requires the BCC and relay pins too. The runtime pins and the log replay do not attest storage a constructor wrote without a log (GT emergency pairs, a router facet-table entry, stakes, adapters, store results, a pre-queued GT op): checking each creation transaction's input against the pinned creation code and the expected constructor arguments is pre-mainnet work, and until then the deploy host and the checkout are trusted.
  - `HemiScript._requireRpcChain`: every production entry point and the dev-script gate refuse a set `FOUNDRY_CHAIN_ID`, and an RPC whose `eth_chainId` differs from `block.chainid` (`--chain` / `--chain-id` override `block.chainid` in the simulation while forge still sends to the RPC's chain). `scripts/test-chain-override.sh`: 14 checks.
  - TransferOwnership `verify` accepts the "Abandoned run" rewire with `PREVIOUS_BCC=<old BCC>`: exactly `BatchCycleCommitteeUpdated(0, PREVIOUS_BCC)`, `Proposed(BCC)` (each re-proposal with one `Cancelled(BCC)`), `Accepted(PREVIOUS_BCC, BCC)` and `Updated(PREVIOUS_BCC, BCC)`, for a GT-owned `PREVIOUS_BCC` that authorizes this router and is not the live committee.
  - `scripts/check-proposals.sh`: the guardians' review of every live GT op and every pending router, Registry, Staking and BCC proposal; it VETOes (exit 2) one that names a codeless, unpinned or undecodable implementation. `scripts/test-check-proposals.sh`: 34 checks on anvil.
- **CI gates.** `scripts/check-grep-gates.sh` (and `scripts/test-check-grep-gates.sh`, 58 checks): the single writer of `job.speedPremium`; no claim or bid ABI from before 2026-10-17 (the old signatures and selectors, the forfeit names, the void 2-arg daemon form `claimJob(bytes32,uint96)` and `PriceBelowFloor`) in `src/`, `script/`, `scripts/`, `test/` and the published docs; no literal-0 claim floor in `script/`, `scripts/` (not the `test-*.sh` harnesses) or a doc's `cast send` line; no string-encoded or untyped self-call in `src/core`. The allow-list is `scripts/grep-gates.allow` plus inline `grep-gate: allow <gate>` markers; this changelog is allowed whole (history). New CI steps also run the deploy-profile `InfraPins221fTest` (the GT, proxy and store pins on the shipped bytecode), `scripts/test-check-proposals.sh` and `scripts/test-chain-override.sh`. Regression suites: 14 new files in `test/fixes/` (176 tests), 2 in `test/script/` (37) and the invariant base (`test/invariant/SM221fInvariant.t.sol`, 8 tests per PR; the nightly `test/invariant-long/SM221fCampaign.t.sol`, so `ci-forge-test.sh campaigns 4`), with the whole test tree migrated in the same change; the forge suite runs 6,087 tests (`DEPLOY_RUNBOOK.md`, "Regression coverage and CI").

### Residuals (summary)

The remaining limitations are in `DEPLOY_RUNBOOK.md`, threat model, residual 16 (16(a) to 16(m)); in short:
- **16(a)** a settle-based reset (a floor bid, ≥ 300 s, `settleBid` + zero-hold `releaseJob`, about 5 HEMI at 5%, 1 HEMI at the 1% floor) only delays floored claims; floorless or chasing claims, direct and delegated, are repriced;
- **16(b)** the job kill: ten claim + release pairs in one block (about 50 HEMI at 5% on a floor-bound collateral) or one floor-bid round plus nine pairs; the caller is refunded (`cancelStaleJob`);
- **16(c)** the lift race, narrowed to governance causes and pauses longer than 7 d;
- **16(d)** a router pause longer than 15 min (or a Registry pause, for adapter jobs; was "of 15 min or more": the exit opens at the pause start + 15 min + 1 s) still gives every lock holder a penalty-free exit: free resets (delay only for floored claims) and free Dutch premium strips (refunded to the caller);
- **16(e)** floor-bid blocking (residual 7), **16(f)** the caller's cancel + identical resubmit hold-up, **16(g)** overlapping lifts (a joint router + Registry lift, or either one together with a reactivation or unblacklist of the job's adapter or verifier) rewind job-local time by (k − 1) × (overlap + `RESUME_GRACE`) for k clocks, (k − 1) × `RESUME_GRACE` of it while claims are open (exact floors revert and are retried; this entry gave only the k = 2 joint lift), **16(h)** bidders are bound through any pause shorter than 7 d (after a joint pause, about (k − 1) × the overlap again past the lift; a round that had ended at the pause start, for the whole pause), **16(i)** the Dutch premium hold-and-release strip, **16(j)** a third party's governance-cause refund of a live leader once the wall clock passes `wallBidDeadline` under a cause (pause credit earned after the last bid does not move it; this entry said "of an honest winner during a long disable"), **16(k)** the EIP-7702 and EIP-6780 assumptions (confirm both on 743111 and 43111), **16(l)** off-chain floor correctness is load-bearing, **16(m)** a pending `slashAndReopen` can slash a later expired lock.

### Superseded statements (summary)

Marked in place below: the 2026-10-15 wire-format row and entry of the forfeit, the forfeit cooldown and the forfeit branches of the 2026-10-15 bid-round entry, the bid-won `D` of the 2026-10-15 premium entry and its worked row, the 2026-10-15 "1.125^150" hold-out cost, the 2026-10-15 daemons' "the log cursor never skips a job", the 2026-10-15 "Bidders" and checklist advice; the 2026-10-13 "Forfeit only before the bid deadline" and its bid-won premium remarks; the "Not changed" list's `claimJob(bytes32)` and `slashAndClaim(bytes32)`.

### Downstream impact

- **Provers:** claim with `claimJob(jobId, priceFloor, premiumFloor)` at the exact floors of an `eth_call` at a mined block, with a reservation price and a per-job high-water mark; on `ClaimBelowFloor` or `BiddingActive` wait and retry. One sender per staked key: never run two daemons, a bidding tool, rescues or hand sends on one key. The rescue is `slashAndClaim(jobId, floor)` alone, the floor computed once from the observed lock and persisted, with `claimJob(jobId, floor, 0)` as the fallback. Prove first only for callers you trust (residual 16(f)). Do not run the 2026-10-15 daemons.
- **Bidders:** a bid is binding. `refundBid` refunds only under a governance cause; otherwise outbid, or settle and fulfil or release at the snapshotted penalty (`getJobSnapshots`). A pause shorter than 7 d does not free a bid.
- **Callers / SDK:** competitive jobs carry `speedPremium = 0`, `300 ≤ underbidWindow ≤ maxBiddingDuration ≤ 7 d`, and `maxBiddingDuration > underbidWindow` for anti-snipe extensions; `getPremiumCap` is 0 on them. Never default a floor to 0 or seed it from `minPrice`. A custom verifier cannot be an EIP-7702 delegated EOA.
- **Keepers:** `slashAndClaim` takes the floor of the lock you observed; `slashAndReopen` is unchanged.
- **Indexers:** drop `BidForfeited`; `refundBid` emits `BidInvalidated`; regenerate `JobStatusView` (22 members); `getJobSnapshots` is new.
- **Operators:** build with the regenerated pins (11 `AUDITED_CORE_RUNTIME_KECCAK` entries); allowlist the router on the relay and run its stall broadcaster; run `scripts/check-proposals.sh` on every proposal; never pass `--chain` / `--chain-id` or export `FOUNDRY_CHAIN_ID`; the rehearsal provers run the 2026-10-17 daemons, one sender per staked key (`DEPLOY_RUNBOOK.md`).

---

## 🔴 Breaking — 2026-10-15 (prover grace, cumulative premium budget, bid rounds resume at the best bid, `forfeitBid`, typed fulfil errors)

Fresh deploy only, like the 2026-10-13 and 2026-10-14 changes: no proxy is upgraded from a deployed impl, and the router's own storage namespace moves (below). Rebuild every decoder from the regenerated `abi/*.abi.json` (`scripts/gen-abi.sh`). Integrator-facing semantics are in `abi/INTERACTION_GUIDE.md` (§3a the broadcast rule, §4 bidding, §5 the revert order and the premium budget, §6 the windows, the exits and inclusion delay, §9 script gas, §12); operator rules in `DEPLOY_RUNBOOK.md` (threat model, "Lifting a router or Registry pause", "Regression coverage and CI").

### 🔴 Wire format

| Item | Before | Since 2026-10-15 |
|---|---|---|
| `HemiProveAux.forfeitBid(bytes32 jobId)` | none: the forfeit was `releaseJob`'s Open-job branch | **added**, `0x02ef46c6`, routed at the router (`RouterSelectors.aux()` has 24 selectors). **Removed on 2026-10-17** (binding bids): `refundBid(bytes32)` `0x427163dc` takes its slot |
| `releaseJob(bytes32)` (`0x3268974c`, Fulfill, selector unchanged) | Locked: the prover's release; Open: the live bidder's forfeit | **Locked only**: any other status, or a caller that is not the lock's prover, reverts `JobNotLocked(jobId)` |
| `getJobStatusView(bytes32)` (`0xbc4971c6`, selector unchanged) | `JobStatusView`, 22 fields | 23 fields: trailing **`uint40 slashOpensAt`** |
| `getBidState(bytes32)` (`0xdc0b8385`, selector unchanged) | `BidState`, 6 members | 7 members: trailing **`uint40 wallBidDeadline`** |
| `bidStates(bytes32)` (`0x08acc217`) | 6 values | unchanged: 6 values (frozen like `jobs()`; read `wallBidDeadline` with `getBidState`) |

- Both structs are static, so each new field is one more 32-byte word at the end of the return data. A decoder that reads only a prefix (`HemiProveConsumer` reads `status`) keeps working; regenerate full decoders.
- **Errors.** `HemiProveFulfill.abi.json` gains `RegistryPaused(bytes32)` (`0x98179f9d`), `AdapterDisabled(bytes32)` (`0x00fee884`) and `BlacklistedVerifier(address)` (`0x5b0b12c4`), now raised by every fulfil path (below), and loses `BidDeadlinePassed(bytes32)` (`0x5667357f`), which only Aux raises now (`forfeitBid`, a late underbid, the bid-phase `gracePauseRelease`). `HemiProveRouter.abi.json`, the union, already had all four: only the position of `BidDeadlinePassed` moves. A fulfil no longer reverts `Error("Blacklisted")`.
- **Constant** `PROVER_GRACE = 30 minutes` (`src/libraries/Constants.sol`; a file-level constant, no getter).
- **Storage.** `BidState.wallBidDeadline` fills free bytes of the struct's third slot (offset 22); every other member keeps its slot and offset (`.storage-layout/HemiProve.json` re-baselined for it). `ROUTER_STORAGE_LOCATION` moves (below).

### 🔴 Prover grace: `PROVER_GRACE` = 30 min after the lock deadline

With D = `lockDeadline`, G = `PROVER_GRACE` and s = `TIMESTAMP_SLACK` = 30 s, every test below is in job-local time (`jobNow = block.timestamp − pauseCredit`), i.e. against `D + pauseCredit` on the wall clock; the raw `block.timestamp` fast-fails shift with it:

| Path | Before 2026-10-15 | Since 2026-10-15 |
|---|---|---|
| `fulfillJob`, `fulfillJobHashed`, `fulfillJobBatch` (per job) | `jobNow ≤ D` | `jobNow ≤ D + G` ("Past deadline" after); the speed bonus is 0 from D on |
| `releaseJob` of a Locked job | `jobNow < D` | `jobNow < D + G` ("At or past deadline" after); the flat release penalty is unchanged |
| `slashAndReopen`, `slashAndClaim`, the `cancelJob` auto-slash | `jobNow > D + s` | `jobNow > D + G + s` ("Not expired yet" before) |
| The "late" test of a slash during a disable or blacklist | `jobNow > D + s` | `jobNow > D + G + s`: a lock inside its grace is not slashable during the window (`UseGraceRelease*`); it has the free exit instead (next row) |
| `cancelJob`'s Locked expiry (`JobNotExpired` before it) | `jobNow > D + s` | `jobNow > D + G + s` |
| `graceRelease`, `graceReleaseWithDescriptor`, `gracePauseRelease` (`JobExpiredBeforePause`) and the `gracePauseRelease` overlap guard: job-local time at the window start | `jobNow ≤ D + s` | `jobNow ≤ D + G + s`, the slash-open point: a lock frozen by a disable, a blacklist or a pause is either slashable or has the penalty-free exit, never neither (below) |
| `extendDeadlinesForDowntime`'s lateness test | `D + s` | **unchanged**: an extension never revives a lock that missed D |
| `topUpDeposit` on a Locked job | `jobNow ≤ D` | **unchanged** |
| `getJobStatusView.lockDeadline`, `timeRemaining` | end at D | **unchanged**; new `slashOpensAt` |

- **`slashOpensAt`**: for a Locked job, `lockDeadline + pauseCredit + PROVER_GRACE + 30 + 1` (saturated to `uint40`), the first wall-clock second at which a slash or `cancelJob`'s auto-slash can land, a lower bound while a pause clock runs; 0 for any other status. Keepers schedule from it (or from `jobs(id).lockDeadline + getJobExtras(id).pauseCredit + 1,831`), not from `D + 31 s`.
- **Inside its grace** a lock can fulfil (paid the price; speed bonus 0) or `releaseJob` (the flat penalty, 5% by default, burned).
- **The free exits end where the slash opens.** A lock at job-local `jobNow ≤ D + G + s` when a disable, a blacklist or a router or Registry pause begins keeps its penalty-free exit: `graceRelease*` (anyone, during the disable or blacklist) or `gracePauseRelease` (its prover or authorized operator, once the pause has lasted 15 min). A lock past `D + G + s` then has none (`JobExpiredBeforePause`) and is slashed: during a disable or blacklist at once, after a pause once it lifts (a pause blocks every slash). The window freezes job-local time, so free exits kept at `D + s` beside the grace would strand a lock inside `(D + s, D + G + s]` at the window start: neither slashable (`UseGraceRelease*`, `JobNotExpired`) nor grace-releasable for the whole window, with only its prover's 5% `releaseJob`, and in the last 30 s, `(D + G, D + G + s]`, nothing at all (fulfil and release closed, the slash not open). A long or permanent disable would hold the collateral and the caller's deposit. The exit is a small gift to a prover that missed D, bounded by the 30-min grace + 30 s and by the governance window. `extendDeadlinesForDowntime` still treats a lock past `D + s` as late, and `topUpDeposit` and `timeRemaining` still end at D.
- **Events.** `JobReleased.timeHeld` (wall-clock time since the lock) can now exceed `fulfillmentTimeout` by the grace too (it already could by pause credit). `JobFulfilled.speedBonus` is 0 for a fulfil in the grace.
- **Why.** On Hemi's public, tip-ordered blocks an observer could keep an on-time fulfil or release out of blocks until D, slash at `D + 31 s` and settle with the copied proof. G is 150 blocks at 12 s: against a sender that re-signs at the same nonce with a tip at least 12.5% higher for every block it misses, keeping it out for the whole grace costs about 1.125^150 (≈ 4.7 × 10^7) times the victim's starting tip per 30M-gas block (until the victim's tip ceiling). **Superseded on 2026-10-17:** that bound ignored every cap (the sender's tip ceiling, its balance and the RPC's fee cap) and the blocks left at the first dispatch; the cost is a sum over the blocks left, bounded by the sender's effective ceiling (`abi/INTERACTION_GUIDE.md` §6, the cost model).
- **What remains** (threat model residuals): the speed bonus is 0 from D, so delaying a fulfil's inclusion until D still strips the premium (the caller's refund grows by it); an incumbent bidder can still try to keep underbids out of blocks until the bid deadline; and a fulfil that lands after `D + G` reverts and publishes its proof (`INTERACTION_GUIDE.md` §6). Tip escalation is the control.

### 🔴 Fulfil: typed governance errors and the full revert order

Before the gas preflight and the verifier call, a fulfil now names the governance state that blocks it: for an adapter job `RegistryPaused(jobId)` while the Registry is paused, then `AdapterDisabled(psId)` while the adapter is Disabled (both were a bare `VerifierCallFailed()` from inside the Registry call); for a `CUSTOM_VERIFIER` job `BlacklistedVerifier(verifier)` (was `Error("Blacklisted")`). Each is temporary and credited: keep the lock (`gracePauseRelease` / `graceRelease*` are the free exits; fulfil after the lift). `VerifierCallFailed()` stays fixed and selector-free: it means the verifier (or the Registry call) reverted, returned fewer than 32 bytes or ran out of its gas budget, and never carries the verifier's revert data. The full order for `fulfillJob` (`fulfillJobHashed` the same; `fulfillJobBatch` per job, after its `EmptyBatch`, `MaxBatchSizeExceeded` and `ArrayLengthMismatch` checks):
1. `EnforcedPause()` (the router is paused);
2. `JobNotLocked(jobId)`, then `NotJobProver(jobId, sender)`;
3. "Past deadline" (`jobNow > lockDeadline + PROVER_GRACE`);
4. `InvalidDescriptorHash(expected, actual)`;
5. `JournalTooLarge(length, 32768)`, then `JournalMismatch(expected, actual)`;
6. adapter jobs: `RegistryPaused(jobId)`, then `AdapterDisabled(psId)`; `CUSTOM_VERIFIER` jobs: `BlacklistedVerifier(verifier)`;
7. `InsufficientGasForFullSettlement()` (the preflight, §9 of the guide);
8. the verifier call: `VerifierCallFailed()`; for an adapter job then `InsufficientGasForSettlement()` (the post-verify check).

**Superseded:** the 2026-10-13 revert order below (marked there). `VerifierCallFailed` no longer stands for a Registry pause or an adapter disable, so it is not a reason to release on its own (`INTERACTION_GUIDE.md` §5).

### 🔴 Speed premium: one cumulative budget per job

`Job.premiumUsedPpm` (same slot, same `uint24`) is now the **sum** of the shares of `speedPremium` (sp) that the job's earlier locks used, clamped at `PREMIUM_PPM` = 1,000,000. The 2026-10-13 cap kept only the largest single lock's share, so a hold split across k sybil locks kept (k − 1)/k of the premium. Let u(t) be the live lock's **uncapped** bonus at job-local time t, the 2026-10-13 formula: `sp × remaining / D`, with `remaining = min(effective lockDeadline − t, fulfillmentTimeout)` (0 from the deadline on) and `D = fulfillmentTimeout` for a claim-won lock or `fulfillmentTimeout + (settlementTimestamp − firstBidTime)` for a bid-won lock (**superseded:** a competitive job carries no premium since 2026-10-17, so every premium lock is claim-won and `D = fulfillmentTimeout`). Let `used = sp − getPremiumCap(jobId)` (rounded up).
- **Fulfil** pays `speedBonus = max(0, u(now) − used)`, saturating: a spent budget pays 0 and never reverts. A job's first lock (`used = 0`) is paid exactly u(now), as before. `JobFulfilled.speedBonus` is that value; the protocol fee is charged on it as before.
- **Every voluntary Locked → Open** (`releaseJob`, `graceRelease`, `graceReleaseWithDescriptor`, the Locked `gracePauseRelease`) charges the releasing lock's share: `premiumUsedPpm = min(1e6, premiumUsedPpm + ceil((sp − u(tRef)) × 1e6 / sp))`. tRef is the reopen time in job-local time (frozen during a disable or blacklist window, so its start for `graceRelease*`), and the pause start for `gracePauseRelease` (`jobNow + overlap`, as on 2026-10-13). A **slash** sets 1e6, as before.
- **`getPremiumCap(jobId)`** (`0x41b0a464`, unchanged) = `sp × (1e6 − premiumUsedPpm) / 1e6`: the budget **left**. No fulfil pays more, and it never rises. A bid-won lock is paid less still: its u is at most `sp × fulfillmentTimeout / D`, so a zero-hold release of a bid-won lock charges `(D − fulfillmentTimeout) / D` of sp. (Since 2026-10-17 no bid-won premium lock exists: `getPremiumCap` is 0 on a competitive job.)
- **Result.** Claim-won holds h1, …, hk that end in releases, then a fulfil after hold h, pay `max(0, sp × (1 − (h1 + … + hk + h) / T))` (T = `fulfillmentTimeout`): the same as one lock held for the total. A zero-hold claim-won release charges 0. A release with nothing left to earn (at the deadline or in the grace) spends the whole budget, so one late hold-and-release strips the premium from every later prover (accepted: it costs the flat release penalty).

Worked numbers (sp = 100 HEMI, T = 1h unless noted):

| Sequence | `getPremiumCap` after the releases | The fulfil pays | Paid before 2026-10-15 |
|---|---|---|---|
| One lock, fulfilled 54 min in | 100 | 10 | 10 |
| Lock A held 27 min and released; lock B fulfils 27 min in | 55 | 10 | 55 |
| Holds of 10 and 15 min, both released; the third lock fulfils 5 min in | 58.33 | 50.00 | 75 |
| A zero-hold release; the next lock fulfils h in | 100 | u(h), as a first lock | u(h) |
| A 59-min hold released; the next lock fulfils 1 min in | 1.67 | 0 | 1.67 |
| Two holds released 1 s before their deadlines | 0.0278, then 0 | 0 | 0.0278 |
| A release inside the grace (u = 0), or any slash | 0 | 0 | no grace before 2026-10-15; 0 after a slash |
| Bid-won, sp = 10, T = 3,600 s, D = T + 300 s: a zero-hold release, or a release 1,000 s in (superseded on 2026-10-17: submit refuses this job) | 9.23 (7.69% used), or 6.67 (33.33% used) | at most the cap | — |
| Dutch hold loop, sp = 1,000, T = 2h, ten 1h holds by two identities | 0 after the second hold | 0 | 500 |

- **Prove first** now means: prove, claim alone, then fulfil once the claim is mined (the broadcast rule, `INTERACTION_GUIDE.md` §3a; Downstream impact below). Waiting for the receipt costs `sp × (blocks between the claim and the fulfil) × 12 s / fulfillmentTimeout`: `sp × 12 s / T` when the fulfil lands in the next block.
- **Superseded:** the 2026-10-13 statements "The bonus a fulfil earns is `min(computed, cap)`", "At every Locked → Open it drops to the bonus the lock could still have earned", and "A fresh lock that is proved before it is claimed (claim and fulfil in one transaction) still earns the whole remaining cap" (below).

### 🔴 Bid rounds: the round marker is cleared, and dropped rounds resume at the best bid

- **Voluntary reopens clear the marker.** `releaseJob`, `graceRelease`, `graceReleaseWithDescriptor` and the Locked `gracePauseRelease` of a bid-won lock now clear `competitiveBidStartTime(jobId)`, as a slash already did. The job still resumes at `elapsedAtLock = e*(S)` (since 2026-10-13), so a direct `claimJob` right after the reopen locks at `P(e*(S)) ≤ S`, and the Dutch ramp rises from there only until someone claims or bids. The kept marker (2026-10-13) banned every direct claim (`BiddingActive`) while the ramp climbed toward `maxPrice`, and only a bid could stop it: a floor-bid win plus a release (or a free bid drop) let a coalition re-auction a job at up to `maxPrice`.
- **Dropped bids resume at the best live bid and clear the marker.** A bid dropped without a lock (the governance-cause invalidation in `settleBid`, the early governance-cause refund included; the settle top-up failure; the bid-phase `gracePauseRelease`; and both `forfeitBid` branches; since 2026-10-17 the forfeit and the settle top-up are gone and `refundBid` is such a drop) sets the ramp so that `jobNow − rampUpStart = e = min(firstBidTime − rampUpStart, AuctionMath.elapsedForPrice(minPrice, maxPrice, rampUpPeriod, curveType, bestBid))`: the round's frozen first-bid point, or the ramp point of the best live bid (rounded down) if that is lower. A direct claim then locks at `P(e) ≤` the best bid, and the next bid opens a new round (a new `firstBidTime` and marker). Before, the ramp resumed at the first-bid point (at or above every bid of the round) with the marker kept. As before, `rampUpStart` never moves back, `exclusivityDuration` shrinks by the actual forward shift (saturating at 0), and a lapsed exclusivity window is never re-armed. A forward shift also zeroes `elapsedAtLock` since 2026-10-17.
- So the marker is set only while a round is live or its winner holds the lock. On an Open job, `BiddingActive(jobId)` from a direct claim now means a live bid.
- **Cooldown.** The live bidder's own governance-cause refund (its `settleBid`, early under a governance cause or after the deadline) stamps its 1h release cooldown on the job, like the bid-phase `gracePauseRelease` and `forfeitBid`. A third party's invalidation does not cool the bidder down. **Superseded:** since 2026-10-17 the bidder's own refund (its `settleBid` or `refundBid`) is cooled only before `wallBidDeadline`.
- **Re-climb delay.** A winning floor bid (`e*(minPrice) = 0`) released at once puts the ramp back at `minPrice`. The job is claimable there at once, but a prover whose cost is above `minPrice` waits for the ramp to climb back, up to `rampUpPeriod` (≤ 7 days). Delay only (no theft; the caller can cancel after a direct release). Callers set `minPrice` near the expected clearing price and keep ramps short.
- **Superseded:** the 2026-10-13 reopen-pricing "Round marker" (a voluntary reopen keeps it) and its `BiddingActive` trigger "direct claim after a voluntary reopen of a bid-won lock"; the 2026-10-13 competitive-bidding "Round marker" (kept through forfeits, invalidations and the bid-phase exit) and "Frozen Dutch clock" ("after a forfeit or an invalidation the ramp resumes from there").

### 🔴 `forfeitBid` split from `releaseJob`

**Superseded on 2026-10-17:** `forfeitBid`, its burn and `BidForfeited` are removed (binding bids); `refundBid(bytes32)` is the free refund under a governance cause only, and a codeless verifier is no longer a cause. The rest of this entry is history.
- **`forfeitBid(bytes32 jobId)`** (Aux, routed): the live bidder of an Open job walks away from its bid. `JobNotOpen(jobId)` unless the job is Open; `NotJobProver(jobId, msg.sender)` unless the caller is the live bidder (also when no bid is live). No pause gate (an exit, like `releaseJob`).
  - **While a governance cause holds** (the adapter Disabled or Unregistered; for a `CUSTOM_VERIFIER` job the verifier blacklisted or without code: exactly `settleBid`'s governance-cause predicate), at any time: the free refund. The soft-lock is unlocked, not burned; no `reopenCount` or `lastReopenTimestamp` write; the bidder's 1h cooldown is stamped; `BidInvalidated(jobId, bidder)`.
  - **Otherwise, before the job-local bid deadline only:** the forfeit as before. 100% of the soft-lock is burned by Staking, `reopenCount + 1`, `lastReopenTimestamp` stamped, exclusivity and the assigned prover cleared, the 1h cooldown, `BidForfeited(jobId, bidder, burnedCollateral)`. At or after the deadline it reverts `BidDeadlinePassed(jobId)` (the winner settles, then `releaseJob` at 5%).
  - Both branches resume the ramp at the best bid and clear the marker (above).
- **`releaseJob(bytes32)`** is Locked only. Two traps of the 2026-10-13 build are gone: a `settleBid` that refunded (a governance cause) instead of locking, followed by `releaseJob` in a second transaction, burned the whole soft-lock; and during a disable the bidder's documented exit burned 100% where `settleBid` refunded for free.
- **Superseded:** the 2026-10-13 "Forfeit only before the bid deadline" (`releaseJob` by the live bidder) and the 2026-10-13 events entry's `BidDeadlinePassed` trigger "forfeit after the bid deadline": both are `forfeitBid` now.

### 🟡 `settleBid`: third parties wait for the wall-clock deadline of the last bid

- `BidState.wallBidDeadline` stores, at every bid, the wall-clock-effective bid deadline `bidDeadline + (block.timestamp − jobNow)`: the value `BidSubmitted.newBidDeadline` emits. Pause credit earned after that bid does not move it.
- A `settleBid` by anyone but the live bidder reverts `BidDeadlineNotPassed(jobId, wallBidDeadline)` while `block.timestamp < wallBidDeadline`. It compared the job-local `bidDeadline` with the wall clock, which a job with pause credit reaches `pauseCredit` seconds early: during a disable a third party could strip a live bid mid-round. The error's `deadline` is that wall-clock value on this refusal only; the job-local check a locking settle needs still reports the stored `bidDeadline`. A normal settle is never blocked: credit only grows, so `jobNow ≥ bidDeadline` implies `block.timestamp ≥ wallBidDeadline`.
- The live bidder's own early governance-cause refund is unchanged.
- **Superseded:** the 2026-10-13 "anyone else must wait until the raw stored `bidDeadline`", and the 2026-10-13 views bullet's "`BidDeadlineNotPassed.deadline` is the raw job-local value" for that refusal.

### 🟡 Demo: `HashChainVerified` carries the job's seed and iterations

`HashChainConsumer._handleProofResult` read `params.seed` and `params.iterations` through a storage pointer after `delete jobParams[jobId]`, so every `HashChainVerified(jobId, seed, result, iterations, prover)` carried seed 0 (an indexed topic) and iterations 0. It now copies both before the delete and emits the job's values. ABI unchanged; the stored results (`verifiedChains`, `getVerifiedResult`) were always right. Indexers keyed by the seed topic now match.

### Storage layout

- **`ROUTER_STORAGE_LOCATION`** (the router-native ERC-7201 namespace: the facet table, the pending upgrade, the guardian, elector and L1-guardian state) = `0xf192ad0f65eb226caa114f18119f08eea77db5a16d3b39236bbb6b01706efe00`, the value its `erc7201:hemiprove.router` annotation derives to (`keccak256(abi.encode(uint256(keccak256("hemiprove.router")) − 1)) & ~bytes32(uint256(0xff))`; `cast index-erc7201 hemiprove.router`). It was `0xe144d3b356f1f751996a404643b9c7f5a13ad3d5e6d67430b559d00f06f15400`, which no ERC-7201 tool derives from that annotation. Fresh deploy only: an older router proxy upgraded to this build would read an empty namespace. `broadcast/*.json` keep the old value (history); `test/fixes/RouterStorageLayoutPin.t.sol` pins the equality.
- `BidState.wallBidDeadline` (above). The Registry, Staking, BCC and ProgramRegistry layouts are unchanged.

### Scripts and operations (no ABI change)

- **Reference prover daemons** (`script/job-prover.sh`, `script/demo-prover.sh`, sharing `script/lib/prover-lib.sh`; demo and testnet references). They claim alone and wait for the receipt, and fulfil only after a `jobs(id)` read at a block at or after the receipt's shows the job Locked by them: no claim+fulfil multicall, no pipelined nonces. Every fulfil and release is re-signed at the same nonce with a tip and maxFee at least 12.5% higher for every block it misses (`TIP_BUMP_PERMILLE` ≥ 1125), from `TIP_START_WEI` (default `max(2 × eth_maxPriorityFeePerGas, 0.01 gwei)`) up to `TIP_CEILING_WEI` (default 100 gwei); `GAS_PRICE_WEI` is retired and refused. The prover's Locked set is rebuilt every poll (a lock file plus its `JobClaimed` logs), and each lock is retried until Fulfilled or until `lockDeadline + pauseCredit + PROVER_GRACE_SEC − 2 × BLOCK_TIME_SEC`. They release only before that point and only on an allow-list: `JournalMismatch`, `JournalTooLarge`, `InvalidDescriptorHash`, a local defect of the proof, and `VerifierCallFailed` only once `registry.paused()` and the adapter status (custom: `isBlacklisted`), re-read at the same block, show no governance cause. `RegistryPaused`, `AdapterDisabled`, `BlacklistedVerifier`, `EnforcedPause`, the grace-exit hints, `PendingSettlement`, `Error("Blacklisted")` and unknown reverts are kept; errors without revert data (429, 5xx, timeouts) are retried with backoff and never released; `InsufficientGasFor*Settlement` raises the job's gas limit by 1,000,000. The log cursor never skips a job (not quite: a job skipped at its first scan or reopened later is revisited only through the claim retry set of 2026-10-17; the rest is pre-mainnet work). `demo-prover.sh` takes the proof-gen checkout from `PROOF_GEN_DIR` (no default; the directory, cargo and Docker are checked at startup) and in generate mode proves before it claims. CI: `scripts/test-prover-daemons.sh` (85 checks on anvil). **Superseded:** the 2026-10-17 daemons claim with floors and re-broadcast without a latch (the 2026-10-17 entry); never run these against a router of 2026-10-17 or later (their claim selector is unrouted).
- **Forge-script fulfils.** `forge script` sizes every broadcast transaction at the simulated gas used × 1.3 (`gas_estimate_multiplier` is 130 by default, even without the flag), far below the router's 4.6M + `callbackGasLimit` preflight: a script fulfil reverted `InsufficientGasForFullSettlement` on-chain after its claim had landed, while the script's checks of the simulation passed. New `script/lib/FulfillGas.sol`: every script fulfil carries an explicit `{gas: N}`, N = 4,600,000 + `callbackGasLimit` (0 means 200,000) + 250,000 + 16 × calldata bytes + 8 × journal bytes + 500,000 (about 5.56M for a default-callback job with 1 KB of calldata), and `requireOwnLock` refuses a fulfil unless the chain already shows the job Locked by the broadcaster. On live chains claim and fulfil are two separate `forge script` runs: `TestFulfill.s.sol` (`--sig 'claim()'`, then `run()`), `LiveE2E.s.sol` (`runSubmitClaim`, `runFulfillLocked`, `checkFulfilled`; `runSubmitClaim` is split into `runSubmit` and `runClaim` since 2026-10-17), `DemoInteraction.s.sol` (`step4_claim`, then `step5_fulfill`). `scripts/test-fresh-deploy.sh` adds a one-job fulfil smoke on anvil.
- **TransferOwnership `verify`.** It checks every owner-settable scalar of the router and the Registry against `Deploy.s.sol`'s constants and the `initialize()` defaults (`script/lib/CoreDefaults.sol`; the router's pending-cycle-collateral cap against `MAX_PENDING_CYCLE_COLLATERAL` when exported), and each adapter's Registry economics against three new `EXPECTED_ADAPTERS` fields: `<psId>:<adapter>:<mode>:<adapter codehash>:<nativeVerifier codehash>:<feeRateBps>:<minStake wei>:<defaultCollateralRatio>`, 8 fields, required on 43111 and 743111. It fails on any owner-setter event of the router, the Registry or Staking since the deployment record's `fromBlock` other than the deploy's own (`script/lib/VerifySetterLogs.sol`), and on any `PresetUpdated` name missing from `EXPECTED_PRESETS`. So `verify` passes at the handoff, and any later owner setter (a fee, a blacklist, a disable, a price seed) fails it from then on.
- **Core runtime pins.** `script/lib/AuditedCorePins.sol` pins `AUDITED_CORE_RUNTIME_KECCAK` for the router, Registry and Staking implementations and the five facets (immutables zeroed). `Deploy.s.sol` checks the code it deploys before broadcasting, and `verify` checks the chain, on 43111 and 743111; the CI gate `scripts/check-core-pins.sh` (regenerate with `--write`) holds the pins to the build and asserts the deploy profile's resolved settings. `FOUNDRY_PROFILE=deploy` is enforced on 743111 as on 43111 (`Deploy.s.sol`, `DeployBCC.s.sol`), so the rehearsal certifies the mainnet bytecode. `foundry.toml`'s deploy-profile `optimizer_details` block is gone (its snake_case keys were ignored, so its custom Yul step string never reached solc; every deploy-profile runtime and initcode is byte-identical without it), and `scripts/check-no-transient.sh` refuses `transient` state variables while solc is 0.8.28 (SOL-2026-1).
- **743111 guards.** `DeployTestnet.s.sol` and `DeployViaFactory.s.sol` refuse 743111, and refuse `ALLOW_DEV_DEPLOY_ON=743111` or `=43111` on every chain. On 743111 `Deploy.s.sol` sets `MAX_TOTAL_STAKED` to the unfillable 2^200 (and refuses a lower value), and `verify` requires at least that: the free-mint rehearsal token could fill any real cap, and since 2026-10-13 a full cap makes unstake requests irrevocable. The Staking cap is not rehearsed on 743111.
- **Relay pause targets and lifts.** Only the router is a relay pause target for the hot pauser keys, never the Registry: a Registry pause beside a router pause sums the two clocks, so job-local time runs backwards for adapter jobs. `DeployBCC.s.sol` refuses the Registry or Staking in `OLD_PAUSE_TARGETS`; `relay-check` and `selector-check` page a Registry target even when it is passed as allowed; `stall-check` prints no ACTION for the Registry, and on the router pages `OVERLAP` while a Registry clock runs; `pulse-scan` pages `OVERLAP` whenever the router clock and the Registry's global clock or an adapter or verifier key clock rose at the same time (`--keys-from-block`); `scripts/check-lift-tx.sh` aborts a router `unpause()` while the Registry is paused (and the reverse) unless the same transaction lifts both. The on-chain fix (credit the union of the windows, not their sum) is not implemented yet.

### Superseded statements (summary)

Marked in place in the entries below: the 2026-10-13 bonus-burn entry's revert order, the multicall size row, the round marker, the premium cap, the forfeit, `graceRelease*` keeping the marker, the late threshold of a slash during a disable and of the `gracePauseRelease` overlap guard (the slash-open point), the third-party settle deadline, the events entry's triggers and the downstream advice ("release at once", "a margin of at least 2 blocks", "claim and fulfil in one transaction"); the 2026-10-13 input-binding entry's competitive-bidding round marker and frozen clock, the bid-phase `gracePauseRelease` ramp, the deadline list of the pause-clock entry, and its `JobExpiredBeforePause` threshold.

### Downstream impact

- **Provers:** never broadcast a journal or seal for a job before your own lock is in an earlier, mined block: claim alone, wait for the receipt, re-read `jobs(id).prover == you`, then fulfil (no claim+fulfil multicall, no pipelined nonces; a reverted transaction publishes its calldata; `eth_call` shows the seal to your RPC operator). Fulfil as soon as the proof exists and escalate the tip on the same nonce; the fulfil is accepted until `lockDeadline + PROVER_GRACE` (bonus 0 after the deadline). Keep the lock on `RegistryPaused`, `AdapterDisabled`, `BlacklistedVerifier` and `EnforcedPause`; release only on the allow-list. The rescue is `slashAndClaim` alone (or `claimJob` if a keeper slashed first), then the fulfil after inclusion. Give forge-script fulfils an explicit `{gas: N}`.
- **Bidders:** `forfeitBid` replaces `releaseJob` on an Open job; during a governance cause it refunds for free. (Superseded on 2026-10-17: a bid is binding; `refundBid` under a governance cause only.)
- **Callers / SDK:** the premium is one budget per job (`getPremiumCap` is what is left); set `minPrice` near the expected clearing price and keep ramps short on competitive jobs.
- **Keepers:** slash from `getJobStatusView(id).slashOpensAt`, not `D + 31 s`.
- **Indexers:** regenerate `JobStatusView` (23 fields) and `BidState` (7 members); `HashChainVerified` now carries real values; `BidDeadlineNotPassed.deadline` is wall-clock on a third party's early settle.
- **Operators:** the router is the only relay pause target; lift the router and the Registry in one transaction; run `verify` at the handoff with 8-field `EXPECTED_ADAPTERS`; the core pins and the deploy profile on 743111; leave `MAX_TOTAL_STAKED` unset on 743111 (`DEPLOY_RUNBOOK.md`).

---

## 🔴 Breaking — 2026-10-14 (no EIP-712 domain, `cancelStaleJob` job age, example callback gate)

Fresh deploy only, like the 2026-10-13 changes. Regenerate decoders from `abi/*.abi.json`.

### Removed

- **`eip712Domain()`** (`0x84b0196e`) and the event **`EIP712DomainChanged()`** (topic0 `0x0a6387c9ea3628b88a633bb4f3b151770f70085117a15f9bf3787cda53f13d31`) leave the router ABI (`HemiProve.abi.json`, `HemiProveRouter.abi.json`) and every facet ABI. `HemiProveBase` no longer inherits `EIP712Upgradeable`: nothing signs or verifies typed data (no relayed fulfil path), so the router has no EIP-712 domain. A call to `eip712Domain()` (or `DOMAIN_SEPARATOR()`) on the router reverts `"Unknown selector"`; both selectors stay **reserved** (`RouterReservedSelector` at `updateFacets`), so no facet can ever answer a domain query for the router. Sequential storage is unchanged (the EIP-712 state lived in the ERC-7201 namespace `openzeppelin.storage.EIP712`).
- Unused declarations (never in an ABI): error `InvalidSignature()`, constant `ERC_1271_GAS_LIMIT`.

### 🔴 `cancelStaleJob`: job age counts from the reopen

The job-age half of the stale check now measures 7 days from the job's last submit or reopen: `max(lastReopenTimestamp, rampUpStart + elapsedAtLock)` from `jobs(jobId)` (was `max(lastReopenTimestamp, rampUpStart)`). Every reopen (release, slash, `graceRelease*`, `gracePauseRelease`) writes `rampUpStart = now − elapsedAtLock`, so after a reopen that leaves `lastReopenTimestamp` 0 (a direct release, a grace exit) a job was cancellable `elapsedAtLock` early, and at once when its lock had been taken 7 days or more into its ramp and the adapter had been disabled for 7 days. Submit-time jobs are unaffected (`elapsedAtLock = 0`); the disabled-time requirement and the `MAX_REOPEN_COUNT` immediate cancel are unchanged. No signature change. Keepers that predict the earliest stale cancel must use the new expression.

### 🟡 Example: `SchemaValidatedCallback` acts only on its own jobs

`src/examples/SchemaValidatedCallback.sol` now inherits `HemiProveConsumer` (with `Ownable`): constructor `(hemiProve, hemiToken, registry, owner)`; jobs are submitted with the owner-only `requestProof(JobDescriptor, AuctionConfig) returns (bytes32)` (the descriptor's `callbackContract` must be the example, else `CallbackNotThisContract(address)`), paid from its HEMI balance; `cancelProof(bytes32) returns (bool)` and `recoverTokens(address,uint256)` are owner-only. A result reaches the handler only for a recorded job whose caller is the example; any other job naming it returns false with no state change (`CallbackFailed` at the router). It previously stored and decoded the result of any job naming it, so a stranger's job (accept-all custom verifier, forged journal) could overwrite `storedPublicValues` / `lastJobPerProgram`. The example is not deployed and has no published ABI file.

---

## 🔴 Breaking — 2026-10-13 (bonus burned at the source, journal cap, reopen pricing, premium cap, late-lock rules)

Fresh deploy only, like the input-binding change below: no proxy is upgraded from a deployed impl. Rebuild every decoder from the regenerated `abi/*.abi.json` (`scripts/gen-abi.sh`). Integrator-facing semantics are in `abi/INTERACTION_GUIDE.md` (§3a binding cookbook and pre-claim checks, §5 payout and premium cap, §6 exits and their costs, §9 gas); operator rules in `DEPLOY_RUNBOOK.md`.

### 🔴 Changed functions

| Function | Old | New |
|---|---|---|
| `HemiProveAuxGov.extendDeadlinesForDowntime` | `(bytes32[] jobIds, uint40 extension, string reason)` `0xd43564ee` | `(bytes32[] jobIds, address[] expectedProvers, uint40[] expectedLockDeadlines, uint40 downtimeStart, uint40 extension, string reason)` `0xcea77f03` |
| `HemiProveAux.getPresetDeposit(bytes32)` (`0xbfa102b6`, selector unchanged) | `returns (uint96)` | `returns (uint256 depositRequired)` |
| `ProofResultStore.results(bytes32)` (`0x4c6b25b1`, selector unchanged) | `(bytes32 programId, bytes32 proofSystemId, address prover, uint40 fulfilledAt, bytes32 publicValuesHash, bytes publicValues)` | `(bytes32 publicValuesHash, address prover, uint40 fulfilledAt)` |

- **`extendDeadlinesForDowntime`** binds each id to the lock it was built for. Per id it emits `DeadlineExtensionSkipped(jobId)` and moves on (no batch revert) when the job is not Locked, `jobs(id).prover != expectedProvers[i]`, the **raw** `jobs(id).lockDeadline != expectedLockDeadlines[i]` (another lock, or an extension already applied: re-running a batch is a no-op), the cumulative cap `min(fulfillmentTimeout, MAX_FULFILLMENT_TIMEOUT)` would be exceeded (was a whole-batch revert, "Cumulative extension exceeded"), or the lock was already late at `downtimeStart` (its job-local time at `downtimeStart`, taken as `jobNow − (now − downtimeStart)`, past `lockDeadline + 30 s`: a defaulted lock is never revived; `downtimeStart = 0` disables this test). The batch reverts on arrays of different lengths (`ArrayLengthMismatch`), on `downtimeStart > block.timestamp`, and as before on a zero or too large extension and on more than `MAX_BATCH_SIZE` ids. `DeadlinesExtended` is unchanged and still lists the whole batch. The function stays a **GT emergency selector** (`script/lib/GtEmergencySelectors.sol` follows the symbol), so the GT guardian can call it instantly through `relay.forward`. Build a batch from `jobs(id).prover` and the raw `jobs(id).lockDeadline` read at one block; never extend for a window a pause clock already credits. **Superseded on 2026-10-26:** replaced by `extendLocksForDowntime` (router paused only, an exact per-lock extension, once per lock, at most 24 h, reason-coded skips).
- **`getPresetDeposit`** returns `maxPrice + speedPremium` as `uint256` (it truncated to `uint96`). Decoders change the output type only.
- **`ProofResultStore`** stores the journal hash, not the journal: the old store ran out of the 200k default callback gas for journals of 128 B or more, so `hasResult` stayed false while the job settled. `getPublicValues(bytes32)` (`0xb86fbee9`) is **removed**; `matches(bytes32 jobId, bytes publicValues) returns (bool)` (`0x6706f0ea`) is **added** (true iff a result is stored and `keccak256(publicValues)` equals its hash); `hasResult(bytes32)` and `ResultStored` are unchanged. Readers take the journal bytes from `JobFulfilled.publicValues` and check them with `matches`. A store now fits the default 200k `callbackGasLimit` for every journal up to `MAX_JOURNAL_BYTES`; do not set less than 100k. ProofResultStore is not deployed on any live chain yet, so nothing migrates.

### 🟡 Added

- **View** `HemiProveFulfillCycle.getPremiumCap(bytes32 jobId) returns (uint256)` (`0x41b0a464`), routed at the router (`RouterSelectors.fulfillCycle()` now has 6 selectors): the speed premium still payable on the job, `speedPremium × (PREMIUM_PPM − premiumUsedPpm) / PREMIUM_PPM`. `getJobStatusView` keeps reporting the raw `speedPremium`, and `getJobExtras` keeps its 3-value tuple.
- **Struct field** `Job.premiumUsedPpm` (`uint24`, struct slot index 6, offset 29, in the free bytes after `speedPremium`): the parts per million of `speedPremium` no longer payable. 0 at submit; only ever raised. It is not in the 27-field `jobs()` tuple: read the cap with `getPremiumCap`.
- **Constants** `MAX_JOURNAL_BYTES = 32_768` and `PREMIUM_PPM = 1_000_000`.
- **Error** `JournalTooLarge(uint256 length, uint256 max)` (`0x7617f2d9`).
- **`InvalidJournalBinding` reason `5`**: a used FIELDS slot whose `offset + length` exceeds `MAX_JOURNAL_BYTES`.

### Removed

- Constants `FULFILL_TYPEHASH` and `SUBMIT_AND_FULFILL_TYPEHASH`, and the unused internal `_verifyProverSignature`. They were never used on-chain and suggested a relayed fulfil path that does not exist: every fulfil path requires `msg.sender == job.prover`.
- `ProofResultStore.getPublicValues(bytes32)` (above).

### 🔴 Every prover-derived bonus is burned at the source (all jobs)

What the input-binding change did for `CUSTOM_VERIFIER` jobs now holds for **every** job, adapter jobs included:
- **Slash.** The bonus share of a slash (33/98, about 34%, of what is left after the keeper reward) is added to the slash's burn: `distributeSlash` is called with a zero bonus amount, and `JobReopened.burnedAmount` includes the share. Nothing is added to `bonusAmount`. The keeper reward is unchanged: `max(2% × collateral, MIN_KEEPER_REWARD = 0.5 HEMI)` to a keeper other than the job's caller, burned (`keeperReward == 0`) when the keeper is the caller. A self-slash by the defaulting prover still earns the keeper share.
- **Release.** The release penalty's bonus half, which Staking still sends to the router, is forwarded to the burn destination in the same call (an IOU in `withdrawableBalances[burnDestination]` if the sink refuses). The release penalty is therefore **fully burned**: a releasing prover's net cost doubles from 2.5% to 5% of `lockedCollateral` at the default `MIN_RELEASE_PENALTY_BPS`.
- **Consequences.** `bonusAmount` is 0 for every job of this deploy. It stays in `Job`, `ProofContext`, `JobClaimed`, `JobFulfilled`, `JobReopened.newBonusAmount`, `JobReleased.newBonusAmount`, `JobCancelled.bonusBurned` and `JobStatusView` (shapes unchanged). The reopen bonus as a rescue incentive is gone; the resumed Dutch ramp remains the incentive. The fulfil payout is `settledPrice + speedBonus − protocolFee`.
- **Superseded:** the input-binding entry's bullet "`CUSTOM_VERIFIER` jobs: neither the slash bonus share nor the release-penalty bonus share enters `bonusAmount`" below now describes every job, and `JobReleased.newBonusAmount` is 0 rather than "unchanged".

### 🔴 Journal cap on every fulfil path; FIELDS slots must end within it

- `fulfillJob`, `fulfillJobHashed` and `fulfillJobBatch` (also inside `multicall`) revert `JournalTooLarge(length, 32768)` when `publicValues.length > MAX_JOURNAL_BYTES`, for every prover, contract provers included. The check and the journal binding now run **first**, before the verifier call (the binding moved from after the verifier; it is pure over the hash-checked descriptor and the calldata journal, so nothing it guards changed): a mismatching journal reverts `JournalMismatch` without spending the verifier budget, and the binding's FIELDS cost is spent before the router's gas preflight is measured. Revert order after the status, prover, deadline and descriptor checks: `JournalTooLarge` → `JournalMismatch` → (`CUSTOM_VERIFIER` jobs: "Blacklisted") → `InsufficientGasForFullSettlement` → the verifier call. **Superseded on 2026-10-15:** `JournalTooLarge` → `JournalMismatch` → (adapter jobs: `RegistryPaused(jobId)` → `AdapterDisabled(psId)`; `CUSTOM_VERIFIER` jobs: `BlacklistedVerifier(verifier)`, was Error("Blacklisted")) → `InsufficientGasForFullSettlement` → the verifier call (`VerifierCallFailed()` only for a verifier revert); the deadline check runs to `lockDeadline + PROVER_GRACE`.
- Every submit path reverts `InvalidJournalBinding(5)` for a FIELDS word with a used slot whose `offset + length > 32,768`. Unused slots (length 0) keep "offset ignored". A FULL journal cannot be measured at submit: a FULL job whose journal exceeds the cap can never be fulfilled by anyone (like an impossible binding, threat model residual 6; with the bonus burned, the caller gains at most a sybil keeper's share).
- Why 32 KiB. op-geth refuses transactions over 131,072 B. With the 64 KiB descriptor cap, the worst fulfil transaction (a `multicall([slashAndClaim, fulfillJob])`) is 99,553 B before the seal, which leaves at least 31,519 B for the seal on every path, 17.9 times the largest governance seal (an OpenVM Halo2 proof, 1,760 B). The table is in `INTERACTION_GUIDE.md` §3. The cap also bounds the FIELDS buffer (8 × the journal), the callback copy and the `JobFulfilled` log. **Since 2026-10-15:** never send that multicall: it publishes the seal before the lock is mined (anyone staked copies it from the mempool). It stays in the table as a size bound only.
- `MAX_DESCRIPTOR_BYTES` stays 65,536. The input-binding entry's statement that the descriptor cap alone keeps every fulfil under the limit ("≥ 64,275 B left for the journal and the seal") was wrong: nothing bounded the journal.

### 🔴 Bid-won locks reopen at a price ≤ the winning bid; voluntary reopens re-auction

- **Settle.** `settleBid` now sets `elapsedAtLock` to `e*(S)`, the largest ramp point in `[0, rampUpPeriod]` whose Dutch price does not exceed the winning bid `S` (rounded down, computed by bisection over the same price function every claim and view uses; `AuctionMath.elapsedForPrice`). Every reopen of that lock (release, slash, `gracePauseRelease`, `graceRelease*`) resumes the ramp there, at a price ≤ `S` (it resumed at the round's first-bid ceiling, ≥ every bid of the round), and the Dutch ramp rises from that point as usual. `JobReopened.elapsedAtLock` of a bid-won lock reports `e*(S)`.
- **Round marker.** A voluntary reopen of a bid-won lock (`releaseJob`, `gracePauseRelease`, `graceRelease`, `graceReleaseWithDescriptor`) **keeps** `competitiveBidStartTime(jobId)`: a direct `claimJob` / `claimJobDelegated` / `claimJobBatch` reverts `BiddingActive(jobId)`, and the job must be re-auctioned (bid ceiling from `P(e*(S))`). A slash clears the marker, so `slashAndReopen` leaves the job directly claimable and `slashAndClaim` locks at `P(e*(S)) ≤ S`. Claim-won locks are unchanged (no marker). **Superseded on 2026-10-15:** every voluntary reopen clears the marker too, so the job is directly claimable at `P(e*(S)) ≤ S` at once.
- **Superseded:** in the competitive-bidding section of the input-binding entry below, "every Locked → Open reopen clears it" and "at settle `elapsedAtLock = firstBidTime − rampUpStart`".

### 🔴 Speed premium: one budget per job across reopens

- The bonus a fulfil earns is `min(computed, cap)`, where `computed` is the input-binding entry's formula on the live lock (`speedPremium × remaining / D`; `remaining` = the effective time left, at most `fulfillmentTimeout`; `D = fulfillmentTimeout`, or for a bid-won lock `fulfillmentTimeout + (settlementTimestamp − firstBidTime)`) and `cap = getPremiumCap(jobId)`. **Superseded on 2026-10-15:** the payout is `max(0, computed − used)`, with `used = speedPremium − cap`. Since 2026-10-17 no competitive job carries a premium, so `D = fulfillmentTimeout` always.
- The cap starts at `speedPremium` and only falls. At every Locked → Open it drops to the bonus the lock could still have earned at that moment: at the reopen for `releaseJob`, `graceRelease` and `graceReleaseWithDescriptor` (job-local time is frozen from the disable or blacklist start, so that is the value then), at the pause start for `gracePauseRelease`, and to 0 on any slash (`slashAndReopen`, `slashAndClaim`, the `cancelJob` auto-slash). The stored cap is rounded down, never above the payable bonus. Releasing to a sybil that re-locks and fulfils no longer collects a fresh full premium. **Superseded on 2026-10-15:** each Locked → Open adds the releasing lock's uncapped share `speedPremium − computed` to the budget used (clamped); the cap is the budget left.
- `JobFulfilled.speedBonus` is the capped value; the protocol fee is charged on `settledPrice + speedBonus` as before. A fresh lock that is proved before it is claimed (claim and fulfil in one transaction) still earns the whole remaining cap: the premium pays lock-to-fulfil latency. **Superseded on 2026-10-15:** never send claim and fulfil in one transaction; claim alone and fulfil after the receipt (it costs about `speedPremium × 12 s / fulfillmentTimeout`).
- **Superseded:** the speed-premium section of the input-binding entry below ("computed on the effective time left") gains the cap.

### 🔴 Forfeit only before the bid deadline; a finished round must settle

- `releaseJob` by the live bidder of an Open job (the forfeit, 100% of the soft-lock burned) reverts `BidDeadlinePassed(jobId)` once job-local time has reached the bid deadline (`jobNow >= bidDeadline`). After it, the winner calls `settleBid` and then `releaseJob`: 5% of the lock, fully burned, and the job is re-auctioned from a ceiling ≤ its bid. **Superseded on 2026-10-15:** the forfeit is `forfeitBid(jobId)` (`releaseJob` is Locked only, `JobNotLocked` on an Open job), it is free while a governance cause holds, and a released bid-won job is directly claimable at ≤ its bid. **Superseded on 2026-10-17:** there is no forfeit; a live bid is binding (`refundBid` only under a governance cause), and the walk-away is settle, then `releaseJob`.
- `gracePauseRelease` by the live bidder of an Open job reverts `BidDeadlinePassed(jobId)` when job-local time at the pause start had reached the bid deadline: a round that had ended must settle, and the Locked rules apply after the lift.

### 🔴 Late locks, grace releases and overlapping pauses

- **Late locks stay slashable during a disable or blacklist.** `slashAndReopen`, `slashAndClaim` and the `cancelJob` auto-slash refuse with `UseGraceRelease` / `UseGraceReleaseWithDescriptor` only while the lock is **not** late in job-local time. A lock that was already past `lockDeadline + 30 s` (**since 2026-10-15:** `lockDeadline + PROVER_GRACE + 30 s`) when its adapter was disabled (or its verifier blacklisted) is slashed as usual by `slashAndReopen`; `cancelJob` on it slashes instead of reverting `CancelSlashFailed` (which remains for an auto-slash refused by a router pause or, for adapter jobs, a Registry pause). `slashAndClaim` of such a lock still reverts as a whole during the window: its claim leg refuses the disabled adapter (`AdapterDisabled`) or the blacklisted verifier (`BlacklistedVerifier`). The error order for a live lock is unchanged.
- **`graceRelease` / `graceReleaseWithDescriptor` refuse a late lock** with `JobExpiredBeforePause(jobId)`. A live lock exits as before (callable by anyone while the window lasts), and now keeps its round marker if it was bid-won and lowers the premium cap. **Superseded on 2026-10-15:** it clears the marker, and charges the lock's share of the premium budget at the window start; "late" is past `lockDeadline + PROVER_GRACE + 30 s` at the window start, the slash-open point.
- **Overlap guard in `gracePauseRelease`.** While both the router and the Registry are paused (adapter jobs), the liveness test uses `jobNow + overlap`, with `overlap = now − max(router pause start, Registry pauseTimestamp())`: a lower bound of the job-local time at the pause start. A prover that was late before an overlapping or staggered pause reverts `JobExpiredBeforePause` (it could exit penalty-free before). **Superseded on 2026-10-15:** late is past `lockDeadline + PROVER_GRACE + 30 s` at the pause start (was `lockDeadline + 30 s`). The same value sets the premium cap and the bid-phase deadline check.
- **CUSTOM_VERIFIER slashes ignore a Registry pause.** `RegistryPausedUseGracePauseRelease` no longer fires for `CUSTOM_VERIFIER` jobs, which receive no Registry pause credit and cannot `gracePauseRelease` under a Registry-only pause: an expired custom lock is slashable (and auto-cancellable) during a Registry-only pause.
- **Superseded:** in the pause-clock section of the input-binding entry below, "A lock that was already late when the adapter was disabled still reverts `CancelSlashFailed`" (it is now slashed), and the `JobExpiredBeforePause` trigger list (it also fires from `graceRelease*` and for the dual-pause overlap).

### 🟡 Bidding: early refunds, codeless verifiers, exclusivity

- The governance-cause bid refund (adapter disabled or unregistered, verifier blacklisted or codeless) before the **raw** bid deadline is available to the **live bidder only**: a third party's early `settleBid` reverts `BidDeadlineNotPassed(jobId, deadline)` until the raw deadline. A live bidder whose bid is not invalid still reverts on the job-local deadline. **Superseded:** the input-binding entry's settle bullet below ("needs only the raw stored `bidDeadline` to have passed"). **Superseded on 2026-10-15:** anyone else waits until the wall clock passes `BidState.wallBidDeadline`, the wall-clock deadline of the last bid (the stored job-local value is reached `pauseCredit` seconds early).
- `submitBid` reverts `CustomVerifierHasNoCode(verifier)` for a custom verifier without code, and `settleBid` treats a verifier that has lost its code as a governance-cause invalidation (`BidInvalidated`, soft-lock refunded). **Superseded:** a codeless verifier is no longer a cause since 2026-10-17 (EIP-6780; submit refuses EIP-7702 designators); `submitBid` still refuses one.
- An exclusivity window that has lapsed is never re-armed: a bid drop (governance invalidation, top-up failure, bid-phase `gracePauseRelease`) shifts the ramp and shortens `exclusivityDuration` by the same amount, and `settleBid` re-expresses the window against the lock's ramp position, so a reopen restores exactly the time the assigned prover had left at settle (that is, a reopen by `gracePauseRelease`; a release, a `graceRelease*` or a slash of the assigned prover clears the window instead), or nothing if the window had lapsed or the winner is not the assigned prover. `jobs(jobId).exclusivityDuration` can therefore change at a bid drop and at settle. **Since 2026-10-15** a bid drop resumes the ramp at the best live bid (a larger shift); the window still shrinks by the actual shift.

### 🟡 `cancelStaleJob`

- A job at `MAX_REOPEN_COUNT` (it can never be locked again) is cancellable by **anyone, at once** (the caller is refunded; `bonusAmount`, 0 for new jobs, is burned).
- Otherwise staleness is measured in **disabled time**: besides the disabled adapter or blacklisted verifier and the job age (7 days since both `lastReopenTimestamp` and `rampUpStart`, as before), the key clock's anchor (`getKeyPauseAccounting(psId)`, or the verifier key for `CUSTOM_VERIFIER` jobs; the start of the live window) must be more than `STALE_JOB_CANCEL_DELAY` = 7 days old ("Stale delay not met" before). A brief false-alarm disable no longer lets anyone cancel week-old jobs. **Superseded on 2026-10-26:** the anchor is the router's key clock (`getKeyClock(key)`); the Registry keeps no clock.

### 🟡 Governance, staking and the Registry

- `GuardianEqualsElector(account)` has new triggers: the router refuses an elector equal to the L1 guardian or to its L2 alias (`applyL1ToL2Alias(l1Guardian)`, which holds the full guardian powers) in `proposeGuardianElector`, `executeGuardianElectorTransfer`, `initializeL1Guardian`, `proposeL1Guardian` and `executeL1GuardianTransfer`.
- `Staking.cancelUnstake()` counts a pending unstake back as active stake only up to the room left under `maxTotalStaked`; the rest stays pending at its original request time (the request time is cleared only when everything counts back). With no room it reverts `MaxTotalStakedExceeded(activeStake + pending, max)`. `UnstakeCancelled(prover, amount)` reports the amount counted back, which can be less than the pending amount. **Superseded:** the input-binding entry's Staking-cap bullet below ("a `cancelUnstake` re-counts its amount and may leave active stake above the cap").
- `Registry.registerAdapter` reverts "Invalid proofSystemId" for `bytes32(0)`, `CUSTOM_VERIFIER` and any address-shaped id (`uint256(psId) >> 160 == 0`): adapter psIds share the Registry's key-clock space with the global clock (key 0) and the custom-verifier clocks.
- `setPreset` reverts `InvalidConfig()` for a preset with `laxCallbackMode = true` (such a preset made `submitJobSimple` revert for every callback-less descriptor).
- `Staking.executeUpgrade` no longer writes `_executingUpgrade` after `upgradeToAndCall` (that write ran with the outgoing implementation's layout). No ABI change.
- The stamping `_pause` / `_unpause` overrides, the router clock writer and `event PauseClockSet(uint40 anchor, uint40 total)` moved from `HemiProve` into `HemiProveBase`, so no facet can flip `paused()` without stamping the clock. No behaviour change at the router; the per-facet ABI files now also list `PauseClockSet(uint40,uint40)` (it is still emitted only at the router address). The per-facet files also gain the errors their new code raises (Aux `CustomVerifierHasNoCode`, AuxGov `ArrayLengthMismatch` and `InvalidConfig`, Fulfill `BidDeadlinePassed` and `JournalTooLarge`); `HemiProveRouter.abi.json`, already the union, changes only by `JournalTooLarge`, `getPremiumCap` and the two changed signatures above.

### 🟡 Consumer templates (`src/base`, `src/demo`)

Callbacks are observational, and anyone can submit a job that names any contract as its callback, with any `programId` and an accept-all custom verifier.
- `HemiProveConsumer` passes a job to `_handleProofResult` only if `pendingJobs[jobId]` is set **and** `context.caller == address(this)` **and** the new `_authorizeJob(ProofContext calldata) internal view virtual returns (bool)` hook returns true (default true; override it to require `trust == GovernanceApproved`, a proof system or a programId). A foreign job returns `false` (`CallbackFailed`) without touching the consumer's state; an own job leaves `pendingJobs` before the hook runs, so one that `_authorizeJob` refuses also returns `false` and is no longer pending. New internal helpers `_cancelProof(bytes32 jobId) returns (bool cancelled)` (calls the router's `cancelJob`; a router revert passes through; returns `false` and keeps the job pending when the cancel only slashed an expired lock, i.e. the job reopened) and `_jobStatus(bytes32)` (reads `getJobStatusView(jobId).status` through the new minimal interface `IHemiProveJobControl`), and the event `ProofCancelled(bytes32 indexed jobId)`.
- `HashChainConsumer` (published ABI; rebuild its decoder from the regenerated file): `createJob(bytes32,uint32)` and `createJob()` are restricted to the owner or `keeper()` (`NotOwnerOrKeeper(address)`; new `setKeeper(address)`, owner only, and event `KeeperUpdated(address indexed keeper)`); `iterations` is capped at `MAX_ITERATIONS = 100_000`, also for `setDefaultIterations` (`TooManyIterations(uint32 iterations, uint32 max)`); new owner-only `cancelJob(bytes32 jobId) returns (bool cancelled)` (deposit refunded to the consumer; `ProofCancelled` on success).
- `OxygenConsumer` (demo, no published ABI file): `proveNextBlock(uint64 targetHeight, bytes32 blockHash)` replaces `proveNextBlock(uint64)`; owner or `keeper()` only (`NotOwnerOrKeeper(address)`), and `blockHash` is the canonical hash of the block at `targetHeight` in the journal's byte order (`ZeroBlockHash()` for 0). Each job pins `(0, 96)` and `(128, 96)` of the journal (`INTERACTION_GUIDE.md` §3a); the callback no longer checks difficulty continuity. `cancelCurrentJob()` now returns `(bool cleared)` and clears the tracking only once the job can no longer settle (a router revert passes through; `false` when the cancel only slashed an expired lock). New `keeper()`, `setKeeper(address)` and events `KeeperUpdated(address indexed keeper)`, `CurrentJobCleared(bytes32 indexed jobId)`. Removed: error `DifficultyMismatch(bytes4,bytes4)`, constant `DIFFICULTY_ADJUSTMENT_INTERVAL`, event `JobCancelFailed(bytes32)`.
- `ProofResultStore`: above (changed functions).
- The commitment templates (`InputCommitmentCallback`, `PrivateInputCallback`) and `IHemiProveCallback` document the observational model: bind the commitment with a FIELDS pin, check `msg.sender` is the router and the job is your own.

### Events: shapes unchanged, semantics

- `JobReopened.burnedAmount` includes the bonus share; `JobReopened.newBonusAmount`, `JobReleased.newBonusAmount`, `JobClaimed.bonusAmount`, `JobFulfilled.bonusAmount` and `JobCancelled.bonusBurned` are 0 for jobs of this deploy.
- `JobReopened.elapsedAtLock` of a bid-won lock is `e*(S)`.
- `JobFulfilled.speedBonus` is capped by the premium cap (**since 2026-10-15** `max(0, computed − used)`, at most `getPremiumCap`). `JobFulfilled.protocolFee` is the fee **net of the committee diversion** (as before; the conservation identity is `proverPayout + callerRefund + protocolFee + diverted == depositedAmount + bonusAmount`).
- `UnstakeCancelled.amount` is the amount counted back.
- `DeadlineExtensionSkipped(jobId)` also reports prover or deadline mismatches, over-cap ids and ids late at `downtimeStart`.
- New error triggers: `BidDeadlinePassed` (forfeit after the bid deadline; bid-phase `gracePauseRelease` after it), `JobExpiredBeforePause` (`graceRelease*` of a late lock; the dual-pause overlap), `CustomVerifierHasNoCode` (`submitBid`), `GuardianEqualsElector` (elector vs the L1 guardian and its alias), `MaxTotalStakedExceeded` (`cancelUnstake` with no room), `InvalidConfig` (`setPreset` with `laxCallbackMode`), `BiddingActive` (direct claim after a voluntary reopen of a bid-won lock), "Invalid proofSystemId" (`registerAdapter`), `JournalTooLarge`, `InvalidJournalBinding(5)`. Removed triggers: `UseGraceRelease*` for a late lock, `RegistryPausedUseGracePauseRelease` for `CUSTOM_VERIFIER` jobs, the "Cumulative extension exceeded" batch revert. **Superseded on 2026-10-15:** the forfeit trigger of `BidDeadlinePassed` is `forfeitBid`, and `BiddingActive` no longer fires after a voluntary reopen.

### Storage layout

`Job.premiumUsedPpm` fills free bytes of `Job` slot 6: `Job` stays 256 B (8 slots) and every older member keeps its slot and offset. `.storage-layout/HemiProve.json` is re-baselined; the Registry, Staking, BCC and ProgramRegistry layouts are unchanged. `ProofResultStore.Result` shrinks to 2 slots (not a proxy).

### Downstream impact

- **Provers:** before `claimJob`, fetch the full witness and run the guest: check the journal against the binding, its size (≤ 32,768 B), that descriptor + journal + seal fit one transaction (`INTERACTION_GUIDE.md` §3), and that proving fits the timeout; release at once on a deterministic failure (5%, burned) rather than wait for a slash (100%). Read `getPremiumCap`, not `speedPremium`. Size the fulfil gas limit by the §9 formula or `eth_estimateGas`; never `--gas-estimate-multiplier`. A forfeit is only possible before the bid deadline. Keep a margin of at least 2 blocks against the effective deadline. **Superseded on 2026-10-15:** release only on the allow-list (`INTERACTION_GUIDE.md` §5), never on `RegistryPaused`, `AdapterDisabled` or `BlacklistedVerifier`; never broadcast a proof before your own lock is mined; fulfil as soon as the proof exists and escalate the tip instead of trusting a fixed margin (the fulfil is accepted until `lockDeadline + PROVER_GRACE`); `forge script` needs an explicit `{gas: N}`; the forfeit is `forfeitBid`.
- **Callers / SDK:** a reopen no longer funds a bonus; a released auction re-runs from a ceiling ≤ its winning bid (**since 2026-10-15** it is directly claimable at that price). Use the binding cookbook (`INTERACTION_GUIDE.md` §3a). For NONE jobs set `minPrice == maxPrice`.
- **Indexers:** bonus fields are 0; partial `UnstakeCancelled`; `getPresetDeposit` and `ProofResultStore.results` output types; `getPremiumCap`.
- **Operators:** the new `extendDeadlinesForDowntime` call shape; the router and Registry lift rules and the pre-execution clock check (`DEPLOY_RUNBOOK.md`, "Lifting a router or Registry pause").

---

## 🔴 Breaking — 2026-10-13 (input binding, pause-aware router deadlines, penalty routing, bidding, speed fee)

Fresh deploy only: no proxy is upgraded from a deployed impl. Rebuild every decoder from the regenerated `abi/*.abi.json` (`scripts/gen-abi.sh`). Integrator-facing semantics are in `abi/INTERACTION_GUIDE.md` (§3a binding, §4–§8); operator rules in `DEPLOY_RUNBOOK.md`.

### 🔴 Input binding: `JobDescriptor` 10 fields, `HashedDescriptor` 8 fields — every submit and fulfill selector changes

```
struct JobDescriptor {            // 10 fields
    bytes32 programId; bytes32 proofSystemId; address callbackContract; address assignedProver; bytes32 tag;
    bytes inputData; bytes callbackExtraData; bytes extraVerifierData;
    bytes32 expectedJournalHash;
    bytes32 journalFields;        // <-- NEW
}
struct HashedDescriptor {         // 8 fields
    bytes32 programId; bytes32 proofSystemId; address callbackContract; address assignedProver; bytes32 tag;
    bytes32 inputDataHash;
    bytes32 expectedJournalHash;  // <-- NEW (was hard-coded to 0 on the hashed path)
    bytes32 journalFields;        // <-- NEW
}
```

| Function | Old selector | New selector |
|---|---|---|
| `submitJob(JobDescriptor,AuctionConfig,CycleConfig,CompetitiveBidConfig)` | `0xaea9834b` | `0xc773d0f8` |
| `submitJobSimple(JobDescriptor,bytes32)` | `0xb723f1cc` | `0x6c6064e0` |
| `submitJobBatch(JobDescriptor[],AuctionConfig[],CycleConfig[],CompetitiveBidConfig[])` | `0x40e92097` | `0xd820b15d` |
| `submitJobDefault(bytes32,bytes32,bytes,address)` → `submitJobDefault(bytes32,bytes32,bytes,address,bytes32 expectedJournalHash,bytes32 journalFields)` | `0x122587cd` | `0x26abbec5` |
| `fulfillJob(bytes32,JobDescriptor,bytes,bytes)` | `0x61f0cf29` | `0xf925a173` |
| `fulfillJobHashed(bytes32,HashedDescriptor,bytes,bytes,bytes,bytes)` | `0x8696ea17` | `0x80f765f2` |
| `fulfillJobBatch(bytes32[],JobDescriptor[],bytes[],bytes[])` | `0xc368885b` | `0xae956f27` |
| `graceReleaseWithDescriptor(bytes32,JobDescriptor)` | `0x75f220da` | `0xbc68cd4c` |

**`descriptorHash`** is now 10 words: `keccak256(abi.encode(programId, proofSystemId, callbackContract, assignedProver, tag, keccak256(inputData), keccak256(callbackExtraData), keccak256(extraVerifierData), expectedJournalHash, journalFields))`. The hashed path re-derives it from `HashedDescriptor` with the same two binding words. `jobId` is unchanged in form (`keccak256(abi.encode(chainid, router, jobIndex, descriptorHash))`).

**Binding modes** (derived; no mode byte). The binding is the **only payout gate**: it is checked after the verifier accepts and before any state write, payment or callback, on `fulfillJob`, `fulfillJobHashed` and `fulfillJobBatch`. Callbacks are observational (`CallbackFailed` only). Since the journal cap (the bonus-burn entry above), the binding runs before the verifier call.

| `expectedJournalHash` | `journalFields` | Mode | Check at fulfill |
|---|---|---|---|
| ≠ 0 | 0 | FULL | `keccak256(abi.encode(JOURNAL_PREDICATE_DOMAIN_V1, proofSystemId, programId, publicValues)) == expectedJournalHash` (the 2026-07-10 formula, unchanged) |
| ≠ 0 | ≠ 0 | FIELDS | `keccak256(concatenation of the used publicValues ranges, in slot order) == expectedJournalHash`, i.e. `keccak256(abi.encodePacked(field0, field1, …))` of the raw pinned bytes; no domain tag |
| 0 | 0 | NONE | none; allowed only with `assignedProver != 0` |
| 0 | ≠ 0 | — | rejected at submit |

`journalFields` packing: 8 slots × 32 bits, slot `i` = bits `[255 − 32i .. 224 − 32i]` (slot 0 in the high bits), slot = `(uint16 offset << 16) | uint16 length` over `publicValues`; length 0 = unused slot (offset ignored). Ranges may overlap and need not be sorted.

**New error** `InvalidJournalBinding(uint8 reason)` (`0x7725587f`), at submit on every submit entry point: `1` `journalFields != 0` with `expectedJournalHash == 0`; `2` FIELDS with no used slot (`uint256(journalFields) & JOURNAL_FIELDS_LENGTH_MASK == 0`, which would bind the empty string); `3` NONE without an `assignedProver` (so `submitJobDefault` cannot create a NONE job); `4` NONE with a `CompetitiveBidConfig`.

**`JournalMismatch(bytes32 expected, bytes32 actual)`** now also covers FIELDS: `actual` is the computed FIELDS hash, or `0` when a used range ends past `publicValues.length` (never truncated).

**New constant** `JOURNAL_FIELDS_LENGTH_MASK = 0x0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff`. **`JOURNAL_PREDICATE_DOMAIN_V1`** (unchanged value) separates (proofSystemId, programId) pairs in FULL mode only; it is **not** replay protection.

**NONE jobs:** only `assignedProver` can ever lock the job (every claim path reverts `ExclusivityActive(jobId, assignedProver)` for anyone else, for the job's whole life, including after every reopen); no bids; `excludeFromEwma` is forced. A NONE job released by its prover cannot be claimed by anyone for the 1h release cooldown.

**No event change for the input binding.** `JobSubmitted` still carries only `descriptorHash`: read both binding words from the submit calldata, as for the rest of the descriptor. The job's NONE flag is readable through `getJobExtras(jobId)` (below).

**Same-statement reuse is accepted by design**: the binding is to the statement, not to the job, the chain or the prover. A proof of the same statement settles every job that binds it, including on another chain.

**Superseded:** the 2026-07-10 journal-predicate entry below says the hashed path is "opt-out only" (a predicate job could not use `fulfillJobHashed`); since 2026-10-13 it supports every mode. "Zero is a valid value — pass `bytes32(0)` if the caller doesn't want a predicate" no longer holds without an `assignedProver`.

### 🔴 Router and Registry pause clocks; pause-aware deadlines

- **Removed** router `localPauseTimestamp()` (`0x3365359e`).
- **Added** router `getPauseAccounting() returns (uint40 anchor, uint40 total)` (`0xbb3fdefe`, router-native; same selector and shape as the BatchCycleCommittee's). Registry `getPauseAccounting()` (its global clock), `getKeyPauseAccounting(bytes32 key) returns (uint40 anchor, uint40 total)` (`0x988d80a6`; key = psId for an adapter's Disabled windows, `bytes32(uint256(uint160(verifier)))` for a custom verifier's blacklist windows) and `pauseCredit(bytes32 psId, address customVerifier) returns (uint256)` (`0x0fad2262`). `Registry.pauseTimestamp()` keeps its semantics (the start of the live pause, else 0). **Superseded on 2026-10-26:** the three Registry clock views are removed; the router holds one union clock (`clockOf`, `getUnionClock`, `getKeyClock`).
- **Added** `getJobExtras(bytes32 jobId) returns (uint256 pauseCredit, uint40 pauseBase, bool assignedOnly)` (`0xec6eb4ec`, FulfillCycle facet, at the router). `pauseCredit` is 0 for terminal jobs.
- **New events:** router `PauseClockSet(uint40 anchor, uint40 total)` (topic0 `0x5c53b373…`, identical to the BCC's: filter by address); Registry `PauseClockSet(bytes32 indexed key, uint40 anchor, uint40 total)` (topic0 `0xb888e2c3…`; key 0 = the global clock).
- **Semantics.** Each clock runs at 1 s/s while paused (router, Registry, adapter Disabled, verifier blacklisted) and for `RESUME_GRACE` = 15 min after each lift. A Locked job's effective deadline is `lockDeadline + pauseCredit`: router clock + Registry clock + its adapter's clock (adapter jobs), or router clock + its verifier's clock (`CUSTOM_VERIFIER` jobs), counted from the job's last submit, lock or reopen. Fulfill, slash, release, `topUpDeposit` and `cancelJob`'s expiry check (`JobNotExpired`, before its auto-slash) use it; an atomic `[unpause, slashAndReopen, pause]` reverts "Not expired yet". The Dutch ramp, the exclusivity window and bid windows run in the same job-local time. **Since 2026-10-15:** fulfil and release run until `lockDeadline + PROVER_GRACE` (30 min), and the slash and `cancelJob`'s expiry wait until 30 s after that.
- **`cancelJob` on a Locked job whose adapter was disabled (or whose verifier was blacklisted) before its effective deadline** now reverts `JobNotExpired` (was `CancelSlashFailed`): the disable window's clock stops the job's time, so the lock never expires while it lasts. Use `graceRelease` / `graceReleaseWithDescriptor`. A lock that was already late when the adapter was disabled still reverts `CancelSlashFailed` (the auto-slash refuses with `UseGraceRelease`). **Superseded by the bonus-burn entry above:** a lock that was already late is slashed (the auto-slash proceeds during the disable), and `graceRelease*` refuses it with `JobExpiredBeforePause`.
- **Views.** `jobs(jobId).lockDeadline` and `rampUpStart` stay **raw** (as of the last base point). For an Open or Locked job that is not mid-settlement, `getJobStatusView` reports `lockDeadline` and `bidDeadline` as effective wall-clock deadlines (raw + pause credit; 0 stays 0), `timeRemaining` in job-local time, and `auctionTimeElapsed` / `currentAuctionPrice` on the paused ramp, frozen at the round's first bid; terminal and pending-settlement jobs report raw values. `getCurrentPrice` follows the same paused, bid-frozen ramp. `BidSubmitted.newBidDeadline` is wall-clock-effective at emission; `BidDeadlineNotPassed.deadline` is the raw job-local value, and so are the stored `bidStates(jobId)` / `getBidState(jobId)` `bidDeadline` and `firstBidTime` and the `competitiveBidStartTime(jobId)` round marker (add `getJobExtras(jobId).pauseCredit` for wall-clock values). While a clock runs, every effective deadline is a lower bound. **Since 2026-10-15:** a third party's early `settleBid` reports the wall-clock `wallBidDeadline` in `BidDeadlineNotPassed`, `getBidState` returns it, and `getJobStatusView` adds `slashOpensAt`.
- **New errors:** `RegistryPaused(bytes32 jobId)` (`0x98179f9d`: `claimJob`, `claimJobDelegated`, `claimJobBatch` (per job), `submitBid` and `settleBid` while the Registry is paused, except `CUSTOM_VERIFIER` jobs; `slashAndClaim` never gets there, its slash leg reverts `RegistryPausedUseGracePauseRelease` first); `JobExpiredBeforePause(bytes32 jobId)` (`0x807791cf`: `gracePauseRelease` for a lock already past `lockDeadline + 30 s` in job-local time, i.e. late before the pause; since the bonus-burn entry also `graceRelease*` of a late lock and the dual-pause overlap guard; **since 2026-10-15** past `lockDeadline + PROVER_GRACE + 30 s`, the slash-open point).
- **`gracePauseRelease`** (no signature change): only the job's prover or its still-authorized `claimingOperator` (`PERM_CLAIM`, not frozen, current nonce; `NotJobProver` otherwise); no `reopenCount` bump; keeps exclusivity, the assigned prover and `lastReopenTimestamp`; stamps the prover's 1h release cooldown. The countdown starts at the router pause's start or, for adapter jobs, the earlier of the router and Registry pause starts; a custom-verifier job under a Registry-only pause reverts "Not paused". New bid-phase exit: during a pause the current bidder of an Open job may call it to drop its bid without a forfeit (soft-lock unlocked, Dutch ramp resumed from the first bid, bidder's 1h cooldown stamped, no `reopenCount` or `lastReopenTimestamp` write); `GracePauseReleased(jobId, bidder, softLocked, address(0))`. The caller may then `cancelJob` at once. A terminal job, or an Open job without a live bid, reverts `JobNotLocked`. **Superseded on 2026-10-15:** the bid-phase exit resumes the ramp at the best live bid and clears the round marker (the job is directly claimable at ≤ that bid).
- **AuxGov `extendDeadlinesForDowntime`** skips ids that are not Locked and emits the new `DeadlineExtensionSkipped(bytes32 indexed jobId)` (topic0 `0x187ec837…`) instead of reverting the batch. `DeadlinesExtended` still lists the whole batch; subtract the skipped ids. **Superseded by the bonus-burn entry above:** new signature, every id bound to its expected prover and raw deadline, and more skip reasons.
- **Superseded:** the ⚪ entry "`gracePauseRelease` now uses MIN of pause timestamps" below reads `hemiProve.localPauseTimestamp()`, which is gone: use `min(router getPauseAccounting().anchor while paused, registry.pauseTimestamp())` (the router's anchor is the pause start while it is paused); the Registry term counts for adapter jobs only.

### 🔴 Prover-derived value never reaches the caller

- **`JobCancelled(bytes32 indexed jobId, address indexed caller, uint256 refundAmount, uint256 bonusBurned)`** — was `(…, uint256 refundAmount, bool bonusIncluded)`: new topic0 `0xa96b38c2…` (was `0xc6241b0a…`). A cancel (`cancelJob`, `cancelJobBatch`, `cancelStaleJob`, and the cancel after the max-reopen auto-slash) refunds `depositedAmount` plus the cycle commit collateral; the prover-derived `bonusAmount` is paid to the burn destination and reported as `bonusBurned`. `jobs().bonusAmount` keeps its value in the terminal record (informational).
- **`JobReopened.keeperReward == 0`** when the keeper is the job's caller (`slashAndReopen`, `slashAndClaim`, or the `cancelJob` auto-slash); that share is in `burnedAmount`.
- **`CUSTOM_VERIFIER` jobs** (**since the bonus-burn entry:** every job)**:** neither the slash bonus share nor the release-penalty bonus share enters `bonusAmount`. The slash share is burned at the source (added to the slash's burn, so `JobReopened.burnedAmount` includes it); the release share, which Staking still sends to the router, is forwarded by the router to the burn destination (IOU fallback), and `JobReleased.newBonusAmount` stays unchanged. `lockCollateralBps` is capped at `MAX_CUSTOM_VERIFIER_COLLATERAL_BPS` = 10,000 (`InvalidCollateralBps(bps)`), and 0 now defaults to 10,000 for them (was 15,000).
- **Descriptor cap:** `inputData + callbackExtraData + extraVerifierData` ≤ `MAX_DESCRIPTOR_BYTES` = 65,536 on every submit path; new error `DescriptorTooLarge(uint256 size, uint256 max)` (`0x9d00d843`). Since the bonus-burn entry the journal is capped too (`MAX_JOURNAL_BYTES` = 32,768).

### 🔴 Competitive bidding

- **`BidForfeited(bytes32 indexed jobId, address indexed bidder, uint96 burnedCollateral)`** — `newBonusAmount` dropped: new topic0 `0xb4f18e0f…` (was `0x6c042ad0…`). The forfeited soft-lock is burned by `Staking.forfeitCollateral` (to its burn destination, IOU fallback); it never enters the job's bonus or the router. The forfeiter's 1h release cooldown is stamped. **Since the bonus-burn entry** a forfeit is possible only before the job-local bid deadline (`BidDeadlinePassed` after it); **since 2026-10-15** through `forfeitBid(jobId)`, free while a governance cause holds.
- **Release cooldown** now also blocks `submitBid` (`ReleaseCooldownActive(jobId, prover, cooldownEnd)`), and is stamped by a release, a forfeit and `gracePauseRelease`.
- **Round marker:** `claimJob` reverts `BiddingActive(jobId)` for the rest of a round that has received a bid (until the job next reopens from Locked), not only while a bid is live. `competitiveBidStartTime(jobId)` is now set at a round's first bid (it was set at settle) and kept through forfeits, invalidations and the bid-phase `gracePauseRelease` exit; every Locked → Open reopen clears it. **Superseded by the bonus-burn entry:** a voluntary reopen of a bid-won lock keeps it (the job is re-auctioned); only a slash clears it. **Superseded on 2026-10-15:** every Locked → Open and every bid drop clears it; it is set only while a round is live or its winner holds the lock.
- **Frozen Dutch clock:** while bids are live the ceiling is the price at the round's first bid; after a forfeit or an invalidation the ramp resumes from there; at settle `elapsedAtLock = firstBidTime − rampUpStart`. **Superseded by the bonus-burn entry:** at settle `elapsedAtLock = e*(settledPrice)`, so no reopen prices above the winning bid. **Superseded on 2026-10-15:** after a bid drop the ramp resumes at the best live bid, or the first-bid point if lower.
- **Bid floor:** `submitBid` reverts the new `BidBelowMinPrice(uint96 bidAmount, uint96 minPrice)` (`0x005304ce`) when `bid < minPrice`. `minPrice` is the caller's reserve.
- **Settle:** no stake re-check of the winner (the soft-lock secures the bid; a stake drop is not a free exit). Governance-cause invalidations (adapter disabled or unregistered, verifier blacklisted) still refund with `BidInvalidated`, and so does a failed collateral top-up at settle. The governance-cause refund needs only the **raw** stored `bidDeadline` to have passed on the wall clock (the disable stops the job's own clock), and it also works during a Registry pause (**since the bonus-burn entry** the live bidder may take it before the raw deadline; anyone else waits for it, and a codeless verifier also counts as a governance cause); a settle that locks needs the job-local deadline. Bid-time eligibility counts stake net of the bidder's pending unstake. **Since 2026-10-15:** "the raw stored `bidDeadline`" is `wallBidDeadline` for anyone but the live bidder.
- **Bid timing bounds at submit:** `underbidWindow` and `maxBiddingDuration` ≤ 7 days, `maxBidExtension` ≤ 24 hours; new error `InvalidBidConfig()` (`0x9cea76f7`).
- **Overbid:** a bid ≥ the current bid reverts `BidDecrementTooSmall(0, minimum)` instead of a `Panic(0x11)` (`minimum` = the larger of `minBidDecrement` and `MIN_BID_DECREMENT_BPS` = 1% of the current bid); a too-small underbid still reports its actual decrement.

### 🔴 Speed premium

`JobFulfilled.protocolFee` is charged on `settledPrice + speedBonus` (the committee diversion takes its share of the larger fee). The speed bonus is computed on the effective time left (**since the bonus-burn entry** capped by the per-job premium cap, `getPremiumCap`). `speedPremium ≤ maxPrice` at submit and in `setPreset`: new error `InvalidSpeedPremium(uint96 speedPremium, uint96 maxPrice)` (`0xd8529c39`).

### 🟡 Router guardian ≠ elector; Staking cap

- New error `GuardianEqualsElector(address account)` (`0x3e64daec`, GuardianLib) from `initializeGuardian`, `reBootstrapGuardian`, `proposeGuardian`, `executeGuardianTransfer`, `proposeGuardianElector` and `executeGuardianElectorTransfer`. Since the bonus-burn entry also for an elector equal to the L1 guardian or its alias (five more sites).
- `Staking.stake` caps **active** stake: `MaxTotalStakedExceeded(newTotal, max)` now reports `totalStakedGlobal − totalPendingUnstake + amount`; new view `totalPendingUnstake()`. The cap gates `stake` only: a `cancelUnstake` re-counts its amount and may leave active stake above the cap. **Superseded by the bonus-burn entry:** `cancelUnstake` counts back only up to the remaining room and reverts when there is none. The cap is a deploy input (`MAX_TOTAL_STAKED`). A forfeit's burn that the burn destination refuses becomes an IOU (`SlashDistributionDeferred(burnDestination, …)` from Staking).

### 🟡 Getters relocated (no ABI change at the router)

`jobs`, `bidStates`, `bidConfigs`, `cycleConfigs`, `presets`, `priceHistories`, `operatorAuthorizations`, `pendingCycleSettlements` and `auditRecords` answer at the router with unchanged signatures and outputs, now dispatched to the Core facet (they were router-native auto-getters). `jobs(bytes32)` keeps its 27-field tuple; the two new `Job` fields (`pauseBase`, `assignedOnly`) are only in `getJobExtras`. Per-facet ABI files: the nine getters now appear in `HemiProveCore.abi.json` only.

### Storage layout

Fresh deploy: the router and Registry layouts change (`localPauseTimestamp` / `_pauseTimestamp` → pause-clock anchor + total; the nine getters' mappings renamed `_jobs`, … with unchanged slots; Registry `_keyClock`; Staking `totalPendingUnstake`). `Job` stays 8 slots (`assignedOnly` and `pauseBase` fill free bytes). Baselines re-generated with documented waivers (`.storage-layout/layout-waivers.txt`).

### Downstream impact

- **Indexers:** re-key `JobCancelled` and `BidForfeited` (new topic0s); add `PauseClockSet` (router: same signature as the BCC's, filter by address; Registry: keyed), `DeadlineExtensionSkipped`; decode the 10-field descriptor from submit calldata (and the 8-field `HashedDescriptor` from hashed fulfills); compute effective deadlines from `getJobExtras` / `getJobStatusView`, never from `jobs().lockDeadline` alone.
- **Callers / SDK:** every job needs a binding (FULL or FIELDS) unless it names its prover; build `journalFields` and the FIELDS hash over the exact committed journal bytes (zkVM framing included: RISC Zero `env::commit` serde pads each byte to a u32 word, SP1 `io::commit` is bincode, `commit_slice` / ABI journals are raw); set `minPrice` as the reserve on competitive jobs; keep descriptors ≤ 64 KiB; `speedPremium ≤ maxPrice`.
- **Provers:** recompute the binding before claiming; budget FIELDS gas (worst case ~0.82M); use effective deadlines; expect the release cooldown on bids; a forfeit is burned. `forge script` fulfils need `--gas-estimate-multiplier 800` or more (the router wants ~4.8M gas left at its preflight; `eth_estimateGas` is fine). **Superseded by the bonus-burn entry:** the FIELDS worst case was understated (about 1.32M for 64 KiB pins; journals are now capped at 32 KiB), and a multiplier does not size the limit reliably: use the gas formula in `INTERACTION_GUIDE.md` §9, an explicit gas limit or `eth_estimateGas`.
- **Keepers:** a slash before the effective deadline reverts "Not expired yet"; the caller earns no keeper reward on its own job.

---

## 🟡 New — 2026-10-12 (router guardian Safe check; custom verifiers without code)

### Router guardian: standard Safes are accepted (`initializeGuardian`, `reBootstrapGuardian`, `proposeGuardian`, `executeGuardianTransfer`)

No signature change. The guardian check used to require `supportsInterface(0x1626ba7e) == true`, which no standard Safe answers: the Safe CompatibilityFallbackHandler (v1.3.0 and v1.4.1) implements `isValidSignature(bytes32,bytes)` but does not declare ERC-1271 through ERC-165. Every standard Safe was rejected with `GuardianMustImplementERC1271`, so the router guardian could not be armed on a live chain. A candidate is now accepted if it is Safe-shaped (`getThreshold() >= 2`, and `getOwners()` returns at least that many owners) **or** declares ERC-1271 through ERC-165. A 1-of-n Safe is rejected. The errors are unchanged: `GuardianMustBeContract` for an EOA, `GuardianMustImplementERC1271` for any other contract.

### `CUSTOM_VERIFIER` jobs: a verifier must have code, and must answer

- 🔴 `submitJob*` and `claimJob` revert the new error `CustomVerifierHasNoCode(address verifier)` when the job's custom verifier has no code: an EOA, `address(0)`, a precompile, or an undeployed or other-chain address. Decoders of the router and `HemiProveCore` ABIs should add it.
- 🔴 `fulfillJob` / `fulfillJobHashed` revert `VerifierCallFailed()` when the verifier call succeeds with less than 32 bytes of returndata (for example a silent fallback). Before, such a call read as a passing `cycleCount == 0`, so a verifier with no code or a silent fallback accepted any proof.

---

## ⚪ Cosmetic — 2026-10-12 (guard unpaused-arm clause; GovernanceTimelock.setGuardian)

### `BatchCycleCommittee.upgradeToAndCall` — an unpaused grace longer than its total now reverts `PauseClockDropped`

No signature change. The guard now evaluates an explicit clause when the committee is **unpaused** after upgrade `data`. It rejects `anchor > block.timestamp + total`: a running grace longer than the total frozen time, for which `P` would be negative.
- Such a `data` (for example one writing `(now + 5000, P0)` with `P0 < 5000`) used to be rejected only by the pause-clock arithmetic, as `Panic(0x11)`.
- It now reverts `PauseClockDropped(impl)` by name. On the GovernanceTimelock path that is `ExecutionFailed(PauseClockDropped(impl))`. Decoders that matched the Panic for this case should match `PauseClockDropped`.
- The unpaused arm is now `anchor < anchor0 || (anchor != anchor0 && anchor <= now) || anchor > now + total`. It is evaluated before `P` is read, so the guard never reaches a pause-clock Panic for a post-`data` clock.
- A *pre*-`data` clock outside forward histories still fails closed with a Panic, at the `P` read before `data`.

BCC deploy runtime 23,922 B (+25 against the entry below). The new pinned runtime keccak is in DEPLOY_RUNBOOK.md ("Pinned values").

### `GovernanceTimelock.setGuardian(address)` — clears a pending guardian revocation

No signature change. If a revocation is pending (`pendingRevokeGuardianReadyAt != 0`), `setGuardian(newGuardian)` (a 24h self-op) now zeroes it and emits `RevokeGuardianCancelled()` before `GuardianUpdated(old, new)`. This is symmetric with `transferOwnership`.
- Before this change a revocation proposed against the outgoing guardian outlived it. `finalizeRevokeGuardian()` is permissionless, so anyone could then remove the guardian just installed, for example a GuardianPauseRelay, the only instant BCC re-pause.
- Indexers: a `RevokeGuardianCancelled` can now come from the execute of a `setGuardian` op.
- If the new guardian should go as well, the owner proposes a new revocation.

## ⚪ Cosmetic — 2026-10-11 (BatchCycleCommittee upgrade guard)

### `BatchCycleCommittee.upgradeToAndCall` — zero-grace clock rewrites in `data` now revert

No signature change. When the committee is unpaused after upgrade `data`, the guard used to require only `anchor >= ` the pre-upgrade anchor. From an unpaused pre-state whose anchor is `<= now` (never paused, or past the grace), it therefore accepted three shapes that leave the clock unpaused at `(now, P)` with no grace: a pause and a zero-grace unpause both inside `data`; a stamping `_pause()` closed by the bare OZ `_unpause()`; and an event-less `(now, P)` checkpoint. `P` never changed, but the last `PauseClockSet` read as a pause, so replaying the logs drifted from `getPauseAccounting()` with no bound. These now revert `PauseClockDropped(impl)`: a `data` that rewrites the anchor and leaves the committee unpaused must leave a grace (`anchor > now`). `''`, `pause()`, `unpause()` and a full `_pause(); _unpause();` on a conformant successor are unaffected. BCC deploy runtime 23,897 B (+37).

**Correction.** An earlier version of this entry said that the grace-end-block bare `_pause()` (the GuardianPauseRelay entry below) is the only accepted `data` that breaks the `PauseClockSet` record. It is not.
- The guard checks state, not events. Any event-less write of a state that conformant `data` could leave also passes.
- Example: an unpaused `(now + x, P + x)` written without a `PauseClockSet`. That is exactly the state a conformant `_pause(); _unpause();` with grace x stores, so `P` and the views stay correct.
- Replaying `PauseClockSet` logs then lags `getPauseAccounting()` by up to x, until the next clock write.
- Monitor M7 (`clock-check`) pages on it: the getter no longer equals the last `PauseClockSet`.
- Both shapes are documented residuals (DEPLOY_RUNBOOK.md, "Residuals").

---

## 🟡 New — 2026-10-11 (GuardianPauseRelay; published ABI files)

### New contract `GuardianPauseRelay` → `GuardianPauseRelay.abi.json`

`src/governance/GuardianPauseRelay.sol`, non-upgradeable (no owner, no upgrade path; a new controller means a new relay). It is installed as the GovernanceTimelock guardian by the 24h op `GT.setGuardian(relay)`, and `script/DeployBCC.s.sol` can deploy it. It splits the guardian role: the controller (the guardian Safe) keeps every guardian power through `forward`, and hot pauser keys can only make the timelock pause an allowlisted target, so the stall re-pause no longer depends on Safe nonces or pre-signed Safe transactions.

- Controller only: `forward(bytes data) returns (bytes)` calls the timelock with `data` as its guardian (`guardianCancel`, `emergencyExecute` of any timelock-allowlisted selector, …) and bubbles the timelock's revert data unchanged. `setPauser(address,bool)` and `setPauseTarget(address,bool)` take effect at once, and setting the current value succeeds and emits again.
- Pauser only: `pause(address target)` runs `GT.emergencyExecute(target, pause())` for a target on the relay's allowlist; the timelock's own (target, `0x8456cb59`) emergency selector must also be set.
- Views: `timelock()`, `controller()` (immutables), `isPauser(address)`, `isPauseTarget(address)`, `PAUSE_SELECTOR()` (`0x8456cb59`).
- Events: `PauserSet(address indexed pauser, bool enabled)`, `PauseTargetSet(address indexed target, bool allowed)`, `PauseRelayed(address indexed target, address indexed pauser)`. `PauseRelayed` follows the timelock's `EmergencyExecuted` in the same transaction and attributes the pause to a pauser key.
- Errors: `ZeroAddress()`, `ControllerIsTimelock()`, `TimelockHasNoCode()`, `ControllerIsTimelockOwner()` (constructor checks), `NotController()`, `NotPauser()`, `PauseTargetNotAllowed(address)`.
- Timelock reverts (`NotGuardian()` after a guardian revocation, `SelectorNotEmergency()` when the timelock de-lists the pause selector, `ExecutionFailed(bytes)`, …) come out of the relay verbatim but are not in its ABI, because they come from an external call rather than a linked library. Decode them with `GovernanceTimelock.abi.json`; `ZeroAddress()` has the same selector in both.
- Monitoring: index the relay's three events, and alert on `PauserSet(…, true)` and `PauseTargetSet(…, true)`, which widen what a pauser key can do.

### Published ABI files

No signature changes in existing contracts here: these items correct what `scripts/gen-abi.sh` publishes.

- **Linked-library errors are back in `ProgramRegistry.abi.json`.** `SchemaLib.validateSchema` (called by `setIOSchema`) and `ProgramRegistryHelpers.injectEncodingMode` are external library functions. They run by DELEGATECALL, so their reverts come out of the ProgramRegistry proxy unchanged, but solc leaves them out of ProgramRegistry's own ABI. The 2026-10-10 regeneration therefore dropped seven errors that `setIOSchema` can revert with, and `InvalidEncodingMode()` had never been published. The generator now appends the error and event entries of every library in an artifact's `linkReferences` to that contract's published ABI:
  - raised through `setIOSchema` → `SchemaLib.validateSchema`: `SchemaTruncated()`, `InvalidSchemaVersion(uint8)`, `InvalidEncoding(uint8)`, `ZeroFieldCount()`, `InvalidFieldType(uint8)`, `InvalidByteSize(uint8)`, `FieldNameTooLong(uint8)`;
  - raised through `ProgramRegistryHelpers.injectEncodingMode` (the field-layout view): `InvalidEncodingMode()`;
  - declared by SchemaLib but raised only by its internal `decode`/`extractField`/`getRef`, which ProgramRegistry never calls: `FieldIndexOutOfBounds(uint8,uint8)`, `PublicValuesTooShort(uint256,uint256)`. ProgramRegistry cannot revert with these two. The merge is a superset by design, because reachability cannot be read reliably from the artifacts: a selector scan of the deploy-profile SchemaLib runtime finds only 5 of the 7 reachable errors.
- **`GovernanceTimelock.abi.json` is published** (new file; the contract is unchanged). The timelock owns BCC and the router. Safe and monitoring tooling (`propose`/`execute`, `emergencyExecute(BCC, pause())`, `setEmergencySelector`, `ExecutionFailed(bytes)` wrapping `PauseClockDropped`, `OperationProposed`) can use it instead of a hand-typed ABI.
- **The published set is derived, not hand-kept**: every non-abstract contract in `src/core/` and `src/governance/` (the list `scripts/check-eip170.sh --print-required` prints, which now includes GuardianPauseRelay), plus the adapters, ProofResultStore, HashChainConsumer, IHemiProveCallback and the mocks, plus the router union. `scripts/gen-abi.sh --check` also fails on an `abi/*.abi.json` that is no longer generated, and `scripts/test-gen-abi.sh` tests the generator in CI.

Per-file delta: signature diff of each file as published before this change against this change, using the method described under "Regenerated non-BCC ABI files" below.

#### `ProgramRegistry.abi.json` — 220 → 230 entries (+10 / −0)

- Errors added (10): `FieldIndexOutOfBounds(uint8,uint8)`, `FieldNameTooLong(uint8)`, `InvalidByteSize(uint8)`, `InvalidEncoding(uint8)`, `InvalidEncodingMode()`, `InvalidFieldType(uint8)`, `InvalidSchemaVersion(uint8)`, `PublicValuesTooShort(uint256,uint256)`, `SchemaTruncated()`, `ZeroFieldCount()`

#### `GovernanceTimelock.abi.json` — 0 → 50 entries (+50 / −0)

- Functions added (23): `DELAY()`, `EXPIRY()`, `REVOKE_GUARDIAN_DELAY()`, `acceptOwnership()`, `cancel(bytes32)`, `cancelRevokeGuardian()`, `emergencyExecute(address,bytes)`, `emergencySelectors(address,bytes4)`, `execute(address,bytes)`, `finalizeRevokeGuardian()`, `guardian()`, `guardianCancel(bytes32)`, `hashOperation(address,bytes)`, `owner()`, `pendingOps(bytes32)`, `pendingOwner()`, `pendingRevokeGuardianReadyAt()`, `propose(address,bytes)`, `proposeRevokeGuardian()`, `renounceOwnership()`, `setEmergencySelector(address,bytes4,bool)`, `setGuardian(address)`, `transferOwnership(address)`
- Events added (10): `EmergencyExecuted(address,bytes)`, `EmergencySelectorUpdated(address,bytes4,bool)`, `GuardianUpdated(address,address)`, `OperationCancelled(bytes32)`, `OperationExecuted(bytes32,address,bytes)`, `OperationProposed(bytes32,address,bytes,uint48)`, `OwnershipTransferStarted(address,address)`, `OwnershipTransferred(address,address)`, `RevokeGuardianCancelled()`, `RevokeGuardianProposed(uint48)`
- Errors added (15): `CannotTargetSelf()`, `DataTooShort()`, `ExecutionFailed(bytes)`, `GuardianEqualsOwner()`, `NotGuardian()`, `NotSelf()`, `OperationAlreadyPending()`, `OperationExpired()`, `OperationNotPending()`, `OperationNotReady()`, `OwnableInvalidOwner(address)`, `OwnableUnauthorizedAccount(address)`, `RevocationExpired()`, `SelectorNotEmergency()`, `ZeroAddress()`
- `constructor` added
- `receive` added

#### `GuardianPauseRelay.abi.json` — 0 → 20 entries (+20 / −0)

- Functions added (9): `PAUSE_SELECTOR()`, `controller()`, `forward(bytes)`, `isPauseTarget(address)`, `isPauser(address)`, `pause(address)`, `setPauseTarget(address,bool)`, `setPauser(address,bool)`, `timelock()`
- Events added (3): `PauseRelayed(address,address)`, `PauseTargetSet(address,bool)`, `PauserSet(address,bool)`
- Errors added (7): `ControllerIsTimelock()`, `ControllerIsTimelockOwner()`, `NotController()`, `NotPauser()`, `PauseTargetNotAllowed(address)`, `TimelockHasNoCode()`, `ZeroAddress()`
- `constructor` added

### ⚪ `BatchCycleCommittee.upgradeToAndCall` — forward-dated stamp now reverts `PauseClockDropped`

No signature change. Upgrade `data` that leaves the committee paused with a stamp in the future (`anchor > block.timestamp`) used to revert with `Panic(0x11)` from the pause-clock arithmetic. It now reverts `PauseClockDropped(impl)` (on the GovernanceTimelock path, `ExecutionFailed(PauseClockDropped(impl))`), because the guard evaluates the frozen-horizon clause first. Decoders that matched the Panic for this case should match `PauseClockDropped`.

One accepted `data` emits no `PauseClockSet`: a successor's bare OZ `_pause()` executed in the block whose timestamp equals the anchor of the last unpause (the grace-end block). The stored clock is then exactly what a stamp would write, so the views stay correct, but replaying `PauseClockSet` logs diverges from `getPauseAccounting()` until the next clock write. Monitor M1 pages on it (a `Paused` without a `PauseClockSet` in the same transaction).

---

## 🔴 Breaking — 2026-10-10 (BatchCycleCommittee pause-aware deadlines)

Most of this entry changes **semantics**, not signatures: existing BatchCycleCommittee (BCC) functions keep their selectors, but the time they compare against has changed. Committee operators must adopt the keeper rule below **before** their first commit on a pause-aware BCC, which is every BCC of the new deployment (DEPLOY_RUNBOOK.md, "Keeper-rule precondition (hard)"). The router-facing `IBatchCycleCommittee` interface is unchanged.

### Epoch deadlines are enforced in epoch-local time

Before this change a BCC pause froze `commit`/`reveal` (`whenNotPaused`) but not the deadlines, so a pause that spanned a reveal deadline let `finalize()` slash honest committers who had been locked out. The BCC now credits pause time to every epoch:

- `P` ("frozen seconds") rises at 1 s/s **while paused** and for the first `RESUME_GRACE` seconds **after each unpause**, and stays constant otherwise. A re-pause inside a grace discards the rest of that grace.
- Each epoch records `P` at creation. Its **credit** is `P(now) − P(creation)`, which counts only pauses and graces after the epoch was created. Its **effective** deadline is the stored raw deadline + credit.
- `commit`, `reveal`, `finalize` and `advanceEpoch`, and the epoch roll-over inside `addJobToEpoch`/`requeueVoidedJobs`, all compare against effective deadlines. `EpochTransitioned` therefore follows the effective deadlines, and `JobResult.attestedAt` is later than the **effective** revealEnd. It can be much later than the raw revealEnd.
- **Every unpause delays every in-flight epoch by the grace applied**, including a pause+unpause in the same block (a pulse). The grace applied is `anchor − block.timestamp` of that unpause's `PauseClockSet` log (900 s in the current BCC).
- `RESUME_GRACE = 900 s` (15 min) in the current BCC. Later impls may change it within `[300 s, EPOCH_REVEAL_WINDOW = 3600 s]`, and a change applies only to unpauses after it. **Never hard-code 900.** The constant is `internal` and has no getter, so read the grace from each `PauseClockSet`.
- A window that closed before a pause stays closed. With no pause, every check is identical to the pre-pause-aware one.
- Stored deadlines are unchanged: `_createNewEpoch` and the Committing transition still write raw wall-clock values.

### `getEpochTiming`, `getEpochCollectionEnd`, `getEpochStatus` — SAME SIGNATURES, new values

- `getEpochTiming(e)` and `getEpochCollectionEnd(e)` return **effective** deadlines: raw value + `getEpochPauseCredit(e)`. As before, `commitEnd`/`revealEnd` read 0 until the epoch has been materialized into Committing. In every non-terminal stored state, effective revealEnd = effective collectionEnd + `EPOCH_COMMIT_WINDOW` + `EPOCH_REVEAL_WINDOW`.
- `getEpochStatus(e)` is evaluated in epoch-local time. At the block it is read in, it matches enforcement exactly.
- **Frozen predicate:** `paused() || getPauseAccounting().anchor > block.timestamp`. While it holds, every non-zero effective deadline of a stored-non-terminal epoch is a **lower bound** that advances 1 s/s. If there is no further pause, the final value is:
  - during a grace: the reported value + `(anchor − now)`;
  - while paused: the reported value + the grace applied at the unpause + the rest of the pause.

  This applies to every boundary not yet reached, including window openings (the reveal window opens at effective commitEnd + 1). A non-zero effective deadline already `< block.timestamp` is closed for good, even while frozen. While paused with `anchor == 0` (an unstamped pause carried over from a pre-pause-aware impl), the values do not advance and the final value is the reported value + the grace applied at the unpause.
- Values for stored Finalized/Voided epochs are constant: their credit is fixed at settlement (`settledCredit`). An epoch settled under a pre-pause-aware impl reads credit 0, so it reports raw values. Nonexistent ids read credit 0.
- An epoch that `getEpochStatus` reports Voided while it is still stored Collecting (0 jobs, past its effective collectionEnd) reports live values until it is materialized. A settled deadline can fall inside a pause even when the window closed before that pause began. To audit an enforced cut-off, use log replay (below), not these views.

### `CommitteeStakingAgeTooYoung(member, joinTimestamp, minAge)` — `minAge` is now an epoch-local age

The revert data is byte-identical: `minAge` is still `COMMITTEE_MIN_STAKING_AGE = 86400` s. Eligibility is now checked as `t ≥ joinTimestamp + 86400`, where `t` is the epoch's local time. In wall-clock terms a member becomes eligible at `joinTimestamp + COMMITTEE_MIN_STAKING_AGE + getEpochPauseCredit(epochId)`. `joinTimestamp + minAge ≤ now` is still necessary, but it is no longer sufficient. A pause therefore cannot change who may commit into an epoch. This relies on `COMMITTEE_MIN_STAKING_AGE > EPOCH_COLLECTION_WINDOW + EPOCH_COMMIT_WINDOW`, and that relation is fixed for the life of the proxy.

### `EpochCreated(epochId, collectionEnd)` — `collectionEnd` stays RAW

`JobAddedToEpoch(e, …)` can arrive up to `getEpochPauseCredit(e)` seconds after `EpochCreated(e).collectionEnd`. The job list of `e` is final only once `getEpochStatus(e) ≥ Committing`. Never schedule from the raw event value.

### `pause()` no longer reverts when already paused

`BatchCycleCommittee.pause()` is idempotent:

| State before | Effect | Logs |
|---|---|---|
| unpaused | pause, and stamp the clock | `Paused(account)`, then `PauseClockSet(ts, P)` |
| paused by a pre-pause-aware impl (`anchor == 0`) | stamp only | `PauseClockSet(ts, total)`, **no** `Paused` |
| paused and stamped | silent no-op | none |

`GovernanceTimelock.emergencyExecute(BCC, pause())` therefore **succeeds** while BCC is paused. Read `paused()`, and never infer state from a revert. This deliberately diverges from HemiProve/Registry/Staking, whose `pause()` still reverts `EnforcedPause`. `unpause()` is unchanged: it still reverts `ExpectedPause` when BCC is not paused, and it is now followed by `PauseClockSet(ts + grace, P + grace)`.

### `voidVulnerableInFlightEpochOnUpgradeV2(uint256[])` — re-callable; skips decided epochs

- No longer `reinitializer(4)`. It emits no `Initialized(4)` and consumes no initializer version. It stays `onlyOwner whenPaused` and may be called any number of times during one migration pause, for example to chunk a long list or to re-run with ids that a failed gate reports. A repeat call on already-voided ids is a no-op.
- It skips **already-decided** epochs: stored Revealing with `revealCount == commitCount ≥ quorum`. `BatchMigrationVoided.voided < supplied` can therefore also mean that a supplied id was already decided. The other skip reasons are unchanged: terminal, `quorum ≤ 17`, duplicate, and `0`.
- Indexers: an `EpochVoided` emitted by V2 can carry `revealCount ≥ quorum` (e.g. 19 of 20 committers revealed with quorum 18 — not everyone revealed, so the epoch is not "decided" and is still voided). Attribute voids in the same tx as `BatchMigrationVoided` to migration; never infer "missed quorum" from a void.
- Each voided epoch writes `settledCredit`, including the in-progress pause, in the same write as its status.
- Gas: about 3M for a full epoch (100 jobs × 20 members), so at most about 10 full epochs fit in a 30M block. Split long lists across calls.

### `BatchCycleCommittee.NotInitialized` error — REMOVED (BCC only)

Its two guards were unreachable (`owner() == address(0)` behind `onlyOwner`). Drop the selector from the **BCC** error decoder only. HemiProveRegistry and ProgramRegistry still declare and raise `NotInitialized()` (`0x87138d5c`), and both of their ABIs list it, so a client with one merged error table must keep the selector.

### `finalize(epochId)` — legacy-quorum branch

For an epoch whose snapshotted quorum is above `COMMITTEE_MAX_QUORUM_PER_JOB = 17` (only an impl from before the current per-job quorum clamp could write one), `finalize` compares the snapshot against the **unclamped** live quorum `max(activeMemberCount / 2, 5)`, so the effective quorum is `min(snapshot, max(activeMemberCount / 2, 5))`. It no longer uses `min(snapshot, clamped live ≤ 17)`. Epochs with a snapshot ≤ 17 behave as before. This is defence in depth: the migration pause and V2 remain the control.

### Router: `HemiProveAuxGov.rescuePendingCycleSettlement(bytes32)` — SAME SIGNATURE, new refusal

`rescuePendingCycleSettlement(jobId)` now also reverts **`"Committee epoch live"`** while `pending.committee.getEpochStatusForJob(jobId)` returns 1, 2 or 3 (stored Collecting, Committing or Revealing), regardless of pause state:

- **Fails open:** the check is skipped if that call reverts or returns fewer than 32 bytes, as with a non-BCC committee.
- **Retired-committee escape:** the check is skipped only if `pending.committee != batchCycleCommittee` (the snapshotted committee has been retired from the router) **and** that committee's `paused()` returns true at execution.

Ops tooling that rescues pending cycle settlements must follow the rescue rule in DEPLOY_RUNBOOK.md (`scripts/bcc-ops.sh rescue-check`).

### Keeper rule (normative) — committee operator bots

This is the normative text; the BCC NatSpec carries only the frozen predicate and the lower-bound semantics. It is a **hard precondition** of committee onboarding: committer operators implement rules 0, 1a–1c, 2 and 6 before their first commit, and no Committee-mode adapter is registered until the confirmations are in (DEPLOY_RUNBOOK.md, "Keeper-rule onboarding and the Committee-mode adapter gate"). There is no upgrade rollout from the pre-pause-aware BCC: every BCC is a fresh deploy.

0. *One block.* Each decision reads everything at one block tag N (`latest`, never `pending`): `paused`, `getPauseAccounting`, `getEpochStatus`, `getEpochTiming`, `getEpochJobs`, `getEpochCounters` and every `eth_call`. Compare deadlines only with `ts(N)`, never with a host or wall clock. RPCs that forward to the sequencer reject sends while it is down, so every broadcast is a retry loop.
1a. *Reveal gating.* Send no reveal calldata for `e` to any endpoint until a `getEpochStatus(e)` read has returned Revealing. This covers `eth_call`, `eth_estimateGas`, `eth_createAccessList`, `eth_sendRawTransaction`, and library auto-estimation or retry wrappers. Poll readiness with views only. Once Revealing has been observed, `commit()` for `e` is closed for good. The gating read must come from a node the operator trusts.
1b. Then broadcast only if, at N, `!paused()` and an `eth_call` of the exact reveal succeeds. (A reverted reveal tx is still public.)
1c. *Commit.* Build `commitHash` only from `getEpochJobs(e)` read at a block where `getEpochStatus(e) == Committing`; from then on the list is final. Persist `(list, counts, salt)` with the hash. Before every (re)broadcast, re-check that `getEpochCounters(e).jobCount` equals the hashed length, and rebuild on mismatch. An `eth_call` of `commit` does not validate the hash.
2. Never pre-sign, schedule or give up based on an effective deadline `≥ ts(N)` read while frozen, or on the raw `EpochCreated.collectionEnd`. A non-zero effective deadline `< ts(N)` is closed for good, even while frozen. After every `Unpaused`, re-read at a block with `ts(N) ≥ anchor`. An upgrade whose `data` is `pause()` counts as a pause.
3. Never apply margin-based skips while frozen.
4. A member is eligible now iff an `eth_call` of `commit(e, h)` from the member at N does not revert `CommitteeStakingAgeTooYoung`. The ETA (a lower bound while frozen) is `joinTimestamp + 86400 + credit(e)`, where `credit(e) = getEpochCollectionEnd(e) − EpochCreated(e).collectionEnd`. That equals `getEpochPauseCredit(e)` on the pause-aware BCC and 0 on pre-pause-aware impls. Decode `minAge` as an epoch-local age.
5. Do not knowingly submit an L1-deposit commit or reveal while BCC is paused. If one lands while paused (`EnforcedPause`), resubmit after `Unpaused`. (There is no rule 5b: the censorship-bound grace option was not adopted.)
6. *Classify outcomes at one N.*
   - **Stop:** `getEpochStatus(e) ∈ {Finalized, Voided}`; `AlreadyRevealed`; `AlreadyCommitted`; `NoCommitment`; `EpochNotRevealing` while `getEpochStatus(e) == Revealing` and `!paused()`, or `EpochNotCommitting` while `getEpochStatus(e)` is Revealing (both closed for good: local time never goes back).
   - **Retry:** `EnforcedPause`; `EpochNotRevealing` or `EpochNotCommitting` while the status is earlier than the target phase (this means "not yet open", never "missed"); `CommitteeStakingAgeTooYoung`.
   - **Bot fault (page, do not retry):** `CommitHashMismatch` or `"Must reveal for all epoch jobs"`.

*Pre-pause-aware fallback:* if `getPauseAccounting()` reverts with empty data, then frozen := `paused()`, credit = 0, and the views are final. Rule 1b is the only broadcast gate. An error while evaluating the frozen predicate or the credit never suppresses or delays a reveal. Bots built this way run unchanged across an upgrade.

Operationally: re-check status at least every 5 min until the epoch is terminal. Treat `EpochNotCommitting`/`EpochNotRevealing` received at or after a raw opening time as retryable. Never time a reveal from a `commitEnd` read before the upgrade, and never send it as a blind fixed-gas broadcast.

---

## 🟡 New — 2026-10-10 (BatchCycleCommittee pause clock)

### `BatchCycleCommittee.getPauseAccounting() external view returns (uint40 anchor, uint40 total)`

The pause-clock snapshot. With `paused()` read in the same block:
- paused: `P = total + (anchor != 0 ? now − anchor : 0)`. `anchor` is the pause start; `0` means an unstamped pause carried over from a pre-pause-aware impl.
- unpaused: `P = total − max(anchor − now, 0)`. `anchor` is the grace end of the last unpause; `total` already includes that grace in full.

`(0, 0)` means no clock write yet. The frozen predicate is `paused() || anchor > now`. The getter is callable by anyone in every state. Its selector and `(anchor, total)` prefix are fixed for the life of the proxy, because the upgrade guard reads it through the proxy.

### `BatchCycleCommittee.getEpochPauseCredit(uint256 epochId) external view returns (uint256)`

Seconds of pause and grace credited to `epochId`: effective deadline = stored raw deadline + this value. It is 0 for nonexistent ids, and fixed at settlement for Finalized/Voided epochs (0 if settled under a pre-pause-aware impl).

### New event

- `BatchCycleCommittee.PauseClockSet(uint40 anchor, uint40 total)` — emitted by the proxy on every clock write, with exactly the stored values, in the same frame. `anchor == block.timestamp` marks a pause or a legacy stamp. `anchor > block.timestamp` marks an unpause, whose grace is `anchor − block.timestamp`. It always follows the OZ `Paused`/`Unpaused` log in the same tx, except for a legacy stamp, which has no `Paused`. The signature and meaning are fixed for the life of the proxy.

### New error

- `BatchCycleCommittee.PauseClockDropped(address impl)` — `upgradeToAndCall` refused `impl`. Either it does not expose `getPauseAccounting()` through the proxy (every pre-pause-aware impl, so **rollback to a pre-pause-aware impl is impossible**), or `P` changed at the upgrade instant, or the frozen horizon was shortened. On the GovernanceTimelock path it surfaces as `ExecutionFailed(abi.encodeWithSelector(PauseClockDropped.selector, impl))`.

`PauseClockNotQuiescent` is **not** part of the current BCC. It is reserved for a future clock-removal upgrade.

### Log replay (indexers)

The views are authoritative. To replay `P` from logs:

- Derive `P` only from BCC `PauseClockSet` logs, never from OZ `Paused`/`Unpaused`.
- Index `PauseClockSet`, `EpochCreated` and `EpochTransitioned` **from the proxy's deployment block**. Use only logs from successful receipts, drop logs marked `removed: true`, and read `ts` from the block header by `blockHash`.
- Order by `(blockNumber, logIndex)`, and let `L` be the last `PauseClockSet` before position x:
  - `L.anchor == ts(L)`: `P(t) = L.total + (t − L.anchor)`;
  - `L.anchor > ts(L)`: `P(t) = L.total − max(L.anchor − t, 0)`;
  - no `L`: `P = 0`.
- `base(e)` is `P` at `EpochCreated(e)`.
- After the `EpochTransitioned(e, Finalized=4 | Voided=5)` log `S(e)`, `credit(e) = P(ts(S(e))) − base(e)` and stays constant. Key on `EpochTransitioned`: the empty-epoch void emits nothing else. Before `S(e)`, `credit = P(t) − base(e)`.
- An index that starts after the deployment block is invalid for every existing epoch unless it is seeded: seed the clock from `getPauseAccounting()` + `paused()`, and the per-epoch base and settled bits from archive reads.

### Storage layout (upgrade-safe)

- `BatchCycleCommittee` — new `PauseClock _pauseClock` struct at slot 28 (`anchor` at offset 0, `total` at offset 5). `__gap[12]` → `[11]` (slots 29–39), so the footprint still ends at slot 39. Slot 27 (`__deprecated_pausedAt`) stays dead.
- `Epoch` struct (still 1 slot, now 31/32 B) — appends `uint40 pauseCreditBase` (offset 21) and `uint40 settledCredit` (offset 26). Epochs that existed before the upgrade read 0 in both fields.

### ABI JSON regenerated

`abi/BatchCycleCommittee.abi.json` is regenerated in this change by `scripts/gen-abi.sh` (`scripts/gen-abi.sh --check` lists any stale ABI file). It had been stale since the repository's initial commit: it lacked, among others, `BatchMigrationVoided`, `voidVulnerableInFlightEpochOnUpgradeV2`, `reseedFormerSinksOnUpgrade`, `markFormerSink` and `rescuePendingPayout`, and still listed the removed `QuorumUnreachable`. Rebuild BCC decoders from the regenerated file, not from a diff against the old one.

### Regenerated non-BCC ABI files

The same regeneration rewrote nine more files. All of them except ProgramRegistry's seven removed errors are **catch-up, not new on-chain surface**. abi/ had been stale since the repository's initial commit, and this change's source edits outside BCC add no ABI entry (HemiProveAuxGov gained only a private helper). Clients that built decoders from the old files must still add these entries. The seven ProgramRegistry errors were dropped by a generator gap and are restored on 2026-10-11 (see the entry above).

Generated mechanically: for each file, a signature-level diff (`name(canonical input types)` for every function, event and error) of the file before this change against the file after it. No entry changed its outputs, state mutability or indexed flags. The `HemiProveRouter` additions are exactly the union of the HemiProve, HemiProveAux and HemiProveAuxGov additions. `AdapterNotActive(bytes32)` left HemiProveAux but stays in the router, because HemiProveCore still declares it.

#### `HemiProve.abi.json` — 167 → 226 entries (+59 / −0)

- Functions added (25): `auxImpl()`, `cancelPendingGuardianElectorRotation()`, `cancelPendingGuardianRotation()`, `cancelPendingL1GuardianRotation()`, `coreImpl()`, `emergencyCancelPendingUpgrade(bytes32)`, `emergencyPause()`, `executeGuardianElectorTransfer()`, `executeGuardianTransfer()`, `executeL1GuardianTransfer()`, `expectedL1EscapeHatchSender(address)`, `fulfillImpl()`, `guardian()`, `guardianElector()`, `initializeGuardian(address,address)`, `initializeL1Guardian(address)`, `l1Guardian()`, `l1GuardianAlias()`, `pendingGuardian()`, `pendingGuardianElector()`, `pendingL1Guardian()`, `proposeGuardian(address)`, `proposeGuardianElector(address)`, `proposeL1Guardian(address)`, `reBootstrapGuardian(address,address)`
- Events added (14): `FormerBurnDestinationRegistered(address)`, `GuardianElectorRotationCancelled(address,address)`, `GuardianElectorRotationExecuted(address,address)`, `GuardianElectorRotationProposed(address,uint48)`, `GuardianEmergencyPaused(address,bool,address)`, `GuardianEmergencyUpgradeCancelled(address,bytes32,bool,address)`, `GuardianReBootstrapped(address,address,address)`, `GuardianRotationCancelled(address,address)`, `GuardianRotationExecuted(address,address)`, `GuardianRotationProposed(address,uint48)`, `L1GuardianInitialized(address)`, `L1GuardianRotationCancelled(address,address)`, `L1GuardianRotationExecuted(address,address)`, `L1GuardianRotationProposed(address,uint48)`
- Errors added (20): `GuardianAlreadyInitialized(address)`, `GuardianElectorAlreadyInitialized(address)`, `GuardianElectorNotInitialized()`, `GuardianMustBeContract(address)`, `GuardianMustImplementERC1271(address)`, `GuardianRotationAlreadyPending()`, `GuardianRotationExpired(uint48,uint48)`, `GuardianRotationNotReady(uint48)`, `L1GuardianAddressZero()`, `L1GuardianAlreadyInitialized(address)`, `L1GuardianRotationAlreadyPending()`, `L1GuardianRotationExpired(uint48,uint48)`, `L1GuardianRotationNotReady(uint48)`, `L2GuardianNotInitialized()`, `NoGuardianRotationPending()`, `NoL1GuardianRotationPending()`, `NotGuardianElector(address,address)`, `NotGuardianOrL1Alias(address,address,address)`, `ReBootstrapRequiresPriorInit(uint64)`, `RouterReservedSelector(bytes4)`

#### `HemiProveCore.abi.json` — 182 → 183 entries (+1 / −0)

- Events added (1): `FormerBurnDestinationRegistered(address)`

#### `HemiProveFulfill.abi.json` — 151 → 152 entries (+1 / −0)

- Events added (1): `FormerBurnDestinationRegistered(address)`

#### `HemiProveFulfillCycle.abi.json` — 136 → 137 entries (+1 / −0)

- Events added (1): `FormerBurnDestinationRegistered(address)`

#### `HemiProveAux.abi.json` — 183 → 184 entries (+2 / −1)

- Events added (1): `FormerBurnDestinationRegistered(address)`
- Errors added (1): `BidsActive(bytes32)`
- Errors removed (1): `AdapterNotActive(bytes32)`

#### `HemiProveAuxGov.abi.json` — 176 → 179 entries (+3 / −0)

- Functions added (2): `isFormerBurnDestination(address)`, `markFormerBurnDestination(address)`
- Events added (1): `FormerBurnDestinationRegistered(address)`

#### `HemiProveRouter.abi.json` — 306 → 368 entries (+62 / −0)

- Functions added (27): `auxImpl()`, `cancelPendingGuardianElectorRotation()`, `cancelPendingGuardianRotation()`, `cancelPendingL1GuardianRotation()`, `coreImpl()`, `emergencyCancelPendingUpgrade(bytes32)`, `emergencyPause()`, `executeGuardianElectorTransfer()`, `executeGuardianTransfer()`, `executeL1GuardianTransfer()`, `expectedL1EscapeHatchSender(address)`, `fulfillImpl()`, `guardian()`, `guardianElector()`, `initializeGuardian(address,address)`, `initializeL1Guardian(address)`, `isFormerBurnDestination(address)`, `l1Guardian()`, `l1GuardianAlias()`, `markFormerBurnDestination(address)`, `pendingGuardian()`, `pendingGuardianElector()`, `pendingL1Guardian()`, `proposeGuardian(address)`, `proposeGuardianElector(address)`, `proposeL1Guardian(address)`, `reBootstrapGuardian(address,address)`
- Events added (14): `FormerBurnDestinationRegistered(address)`, `GuardianElectorRotationCancelled(address,address)`, `GuardianElectorRotationExecuted(address,address)`, `GuardianElectorRotationProposed(address,uint48)`, `GuardianEmergencyPaused(address,bool,address)`, `GuardianEmergencyUpgradeCancelled(address,bytes32,bool,address)`, `GuardianReBootstrapped(address,address,address)`, `GuardianRotationCancelled(address,address)`, `GuardianRotationExecuted(address,address)`, `GuardianRotationProposed(address,uint48)`, `L1GuardianInitialized(address)`, `L1GuardianRotationCancelled(address,address)`, `L1GuardianRotationExecuted(address,address)`, `L1GuardianRotationProposed(address,uint48)`
- Errors added (21): `BidsActive(bytes32)`, `GuardianAlreadyInitialized(address)`, `GuardianElectorAlreadyInitialized(address)`, `GuardianElectorNotInitialized()`, `GuardianMustBeContract(address)`, `GuardianMustImplementERC1271(address)`, `GuardianRotationAlreadyPending()`, `GuardianRotationExpired(uint48,uint48)`, `GuardianRotationNotReady(uint48)`, `L1GuardianAddressZero()`, `L1GuardianAlreadyInitialized(address)`, `L1GuardianRotationAlreadyPending()`, `L1GuardianRotationExpired(uint48,uint48)`, `L1GuardianRotationNotReady(uint48)`, `L2GuardianNotInitialized()`, `NoGuardianRotationPending()`, `NoL1GuardianRotationPending()`, `NotGuardianElector(address,address)`, `NotGuardianOrL1Alias(address,address,address)`, `ReBootstrapRequiresPriorInit(uint64)`, `RouterReservedSelector(bytes4)`

#### `HemiProveRegistry.abi.json` — 88 → 95 entries (+7 / −0)

- Functions added (3): `pendingReactivationEpoch(bytes32)`, `reseedVerifierSwitchDelayIfCorrupted()`, `transferEpoch()`
- Events added (2): `TransferEpochRebased(uint32,uint32)`, `VerifierSwitchDelayReseeded(uint32,uint32)`
- Errors added (2): `NotInitialized()`, `PendingReactivationCrossOwnership(bytes32,uint32,uint32)`

#### `ProgramRegistry.abi.json` — 226 → 220 entries (+1 / −7)

- Events added (1): `RegistrantTransferCancelled(bytes32,address)`
- Errors removed (7): `FieldNameTooLong(uint8)`, `InvalidByteSize(uint8)`, `InvalidEncoding(uint8)`, `InvalidFieldType(uint8)`, `InvalidSchemaVersion(uint8)`, `SchemaTruncated()`, `ZeroFieldCount()`

### Gas

Per-path gas deltas of `addJobToEpoch` (paths (a)–(d)), commit, reveal and finalize, each measured never-paused, post-grace and in-grace against the vendored pre-pause-aware impl (`test/fixes/BCCPauseAwareGas.t.sol`). Metric: minimum forwarded gas for which the call succeeds, deploy profile, cold (the default profile matches within ~100 gas, deltas within 1 gas). Post-grace cost equals never-paused exactly; in-grace adds 131 gas on every path; pause history (1 s, 24h, 10×24h, 900 per-block sandwiches) does not change any figure.

| Path | pre-pause-aware | pause-aware never / post-grace | Δ | pause-aware in-grace | Δ |
|---|---|---|---|---|---|
| `addJobToEpoch` (a) first epoch | 196,559 | 199,025 | +2,466 | 199,156 | +2,597 |
| `addJobToEpoch` (b) Collecting → Committing + create | 189,566 | 192,345 | +2,779 | 192,476 | +2,910 |
| `addJobToEpoch` (c) 0 jobs → Voided + create | 186,288 | 189,409 | +3,121 | 189,540 | +3,252 |
| `addJobToEpoch` (d) steady | 138,309 | 140,997 | +2,688 | 141,128 | +2,819 |
| `commit` + transition | 143,444 | 146,076 | +2,632 | 146,207 | +2,763 |
| `commit` | 120,477 | 123,108 | +2,631 | 123,239 | +2,762 |
| `reveal` + transition | 119,642 | 122,315 | +2,673 | 122,446 | +2,804 |
| `finalize` → Finalized | 166,320 | 169,491 | +3,171 | 169,622 | +3,302 |
| `finalize` → Voided | 273,030 | 276,197 | +3,167 | 276,328 | +3,298 |

The bound is ≤ +3.5k gas per `addJobToEpoch` path, and the minimum forwarded gas for which `addJobToEpoch` returns true must be ≤ 215k on every path, under the router's `COMMITTEE_ADD_JOB_GAS_LIMIT = 300_000`. Both hold: the largest `addJobToEpoch` delta is +3,252 (path (c) in-grace) and the worst minimum forwarded gas is 199,156 (about 1.5x under the cap).

### Downstream impact

- **Committee operator bots**: **critical**. Adopt the keeper rule above before your first commit. Rules 1a–1c are no-ops under a pre-pause-aware impl. Never treat `EpochNotCommitting`/`EpochNotRevealing` as "missed" while the status is earlier than the target phase.
- **Indexer**: subscribe to `PauseClockSet`. Expose effective deadlines from the views, or replay them as above. Treat `EpochCreated.collectionEnd` as raw. Attribute every void in a transaction that also contains `BatchMigrationVoided` to the migration: those voids emit no `MemberSlashed`, and their `EpochVoided` can carry `revealCount ≥ quorum`, which a natural void never does.
- **Frontend**: show effective deadlines. While the clock is frozen, label deadlines as "at least" and show the pause state.
- **Ops tooling**: BCC `pause()` is idempotent, so read `paused()`. Router rescues must pass `rescue-check` and can now revert `"Committee epoch live"`. BCC upgrades must use the runbook sequence: the upgrade from any pre-pause-aware impl carries data `pause()` = `0x8456cb59`.
- **Prover daemon**: no code changes. Mode-1 (Committee) settlements may now arrive later, after the credited pause and grace.

---

## 🔴 Breaking — 2026-07-10 (journal predicate)

### `JobDescriptor` struct gains 9th field

```
struct JobDescriptor {
    bytes32 programId;
    bytes32 proofSystemId;
    address callbackContract;
    address assignedProver;
    bytes32 tag;
    bytes inputData;
    bytes callbackExtraData;
    bytes extraVerifierData;
    bytes32 expectedJournalHash;   // <-- NEW
}
```

**Semantics**: optional outcome commitment.
- `expectedJournalHash == bytes32(0)`: opt-out. The soft-callback semantics of 2026-07-09 apply unchanged (verifier acceptance is terminal for payment). Superseded on 2026-10-13: a zero hash with `journalFields == 0` is the NONE mode, allowed only with an `assignedProver` (`InvalidJournalBinding(3)` otherwise); a nonzero hash is FULL or FIELDS (the 2026-10-13 input-binding entry).
- `expectedJournalHash != bytes32(0)`: predicate mode. After the verifier accepts, the router computes `keccak256(abi.encode(JOURNAL_PREDICATE_DOMAIN_V1, proofSystemId, programId, publicValues))` and requires it to equal the committed value. Mismatch reverts `JournalMismatch(expected, actual)` before any state writes, payment, or callback dispatch.

### Descriptor hash preimage extended

`descriptorHash` (and the equivalent hash reconstructed inside `fulfillJobHashed`) now includes the new field as the 9th `abi.encode` slot:

```
keccak256(abi.encode(
    programId,
    proofSystemId,
    callbackContract,
    assignedProver,
    tag,
    keccak256(inputData),
    keccak256(callbackExtraData),
    keccak256(extraVerifierData),
    expectedJournalHash                // <-- NEW
))
```

Any off-chain code (SDK, indexer, prover daemon) that reconstructs `descriptorHash` MUST include the new field. Zero is a valid value — pass `bytes32(0)` if the caller doesn't want a predicate.

### `fulfillJobHashed` fast-path — opt-out only (superseded on 2026-10-13: every binding mode)

The `fulfillJobHashed` reconstruction fixes the 9th slot to `bytes32(0)`. A job submitted with `expectedJournalHash != 0` cannot be fulfilled through this entrypoint — the reconstructed hash diverges from the stored `descriptorHash` and reverts `InvalidDescriptorHash`. Provers on predicate-enforced jobs must use full `fulfillJob`.

### New error

- `JournalMismatch(bytes32 expected, bytes32 actual)` — predicate check failed at fulfill. Attribution: prover proved the wrong thing (the prover chose the witness).

### New constant

- `JOURNAL_PREDICATE_DOMAIN_V1 = keccak256("HemiProve.JournalPredicate.v1")` — domain tag baked into the predicate preimage. Prevents cross-program / cross-adapter journal replay. Version-suffixed so a future structured-predicate format can coexist.

### No storage layout impact

Job struct unchanged (27 fields, 8 slots). `HemiProveStorage.__gap` unchanged. `expectedJournalHash` lives only on the calldata descriptor and folds into the already-stored `descriptorHash` field.

### Downstream impact

- **Indexer**: extend descriptor decoder to 9 fields; expose `expectedJournalHash` (surface as "predicate" or "outcome commitment"). Optionally index by `expectedJournalHash` to support "jobs seeking this outcome" queries.
- **Frontend**: form field for optional journal-hash commitment; default zero with a warning banner explaining the opt-out (soft-callback) semantics. For deterministic use cases, provide a helper that computes `keccak256(abi.encode(DOMAIN_V1, psId, programId, expectedPublicValues))`.
- **Prover daemon**: **critical** — before claiming a job with non-zero `expectedJournalHash`, simulate the proof off-chain and verify the predicate would pass. Claiming without simulation exposes the prover to collateral slash if the caller's committed journal cannot be produced. This is a known residual (an impossible binding) — no on-chain mitigation is possible (the predicate is the point).
- **SDK**: publish `computePredicate(psId, programId, publicValues) -> bytes32` helper matching the on-chain formula.

---

## 🔴 Breaking — 2026-07-09 (soft callback)

### Removed events (4)

The multi-outcome callback event set is retired. Settlement no longer forks on callback return semantics, so these decoders can be dropped:

- `CallbackForceAccepted(bytes32 indexed jobId, address indexed callback, uint8 rejectionCount)`
- `CallbackForceAcceptedSplit(bytes32 indexed jobId, address indexed callback, uint96 proverShare, uint96 callerRefundExtra)`
- `CallbackRevertAccepted(bytes32 indexed jobId, address indexed callback, bytes retBytes)`
- `CallbackRejected(bytes32 indexed jobId, address indexed callback, uint8 currentCount)`

### Sole surviving callback-failure event

- `CallbackFailed(bytes32 indexed jobId, address indexed callback, bytes reason)` — now the **only** callback-failure signal. Observational only; does **not** gate settlement. Fires on: bool false return, revert, OOG, returnbomb, non-canonical bool ≠ 0.

### Semantics change — callback no longer gates settlement

Callback outcome (accept / reject / revert / OOG / returnbomb) is decoupled from job settlement. On cryptographic verifier accept the job unconditionally transitions to `Fulfilled` and the prover is paid `settledPrice - protocolFee`. There is no retry loop; a single dispatch is attempted per fulfill.

The historical 3-strike loop + `settledPrice/2` force-accept split path (a grief primitive) is **DELETED**.

### Frozen-but-dead fields (storage-preserved for upgrade safety)

- `Job.callbackRejections` (index 24) — frozen in the 27-field struct, never incremented.
- `AuctionConfig.laxCallbackMode` — accepted at submit for descriptor-hash compat, ignored at settlement.
- `HemiProveStorage.jobLaxCallbackMode` — mapping retained, never read.
- `Constants.MAX_CALLBACK_REJECTIONS` — retained as documented-dead constant for test-import compat.

### Downstream impact

- **Indexer**: drop the 4 removed event handlers; ensure `CallbackFailed` is subscribed and treated as observational. `JobFulfilled` remains the authoritative settlement signal.
- **Frontend**: collapse the prior 4-state callback UI into a single "callback failed" indicator; remove force-accept / split-payout affordances.
- **Prover daemon**: no code changes required — prover payment guarantee tightened (full pay on verifier accept, no half-payment risk).
- **SDK**: no signature changes on fulfill / claim / submit paths — pure event and semantics delta.

---

## 🔴 Breaking — 2026-07-08 (staking age by block, verifier switch delay, `MIN_STAKING_AGE` removed)

### `ProverStake.depositTimestamp` → `depositBlock` — field rename + SEMANTIC CHANGE

The trailing field of the `ProverStake` struct was renamed **and its meaning changed**:

- Old: `uint40 depositTimestamp` — stored `uint40(block.timestamp)`; consumed by claim as `block.timestamp >= depositTimestamp + MIN_STAKING_AGE`.
- New: `uint40 depositBlock` — stores `uint40(block.number)`; consumed by claim as `block.number > depositBlock` (strict `>`).

**Impact**: `HemiProveStaking.getProverStake(prover).depositTimestamp` no longer exists — access it as `.depositBlock`. Any downstream (zkui, zkminer, indexer) that read this field for "when did this prover stake" now gets a **block number**, not a timestamp. Convert via chain block time if you need a timestamp back.

### `HemiProveRegistry.reactivateAdapter(psId, newAdapter)` — behavior split (SAME SIGNATURE)

Signature unchanged, but semantics diverge:

- `newAdapter == address(0)` OR `newAdapter == entry.adapter` → **instant** recovery re-enable (unchanged behavior).
- `newAdapter != entry.adapter` → **PROPOSES** a swap with `_verifierSwitchDelay` (default 48h). Adapter stays `Disabled` during the delay. Apply via new `executePendingReactivation(psId)`.

**Impact**: any ops tooling that previously called `reactivateAdapter(psId, someNewAddress)` expecting immediate re-enable now must call `executePendingReactivation` after the delay elapses. Cancel via `cancelPendingReactivation(psId)` inside the window.

### `MIN_STAKING_AGE` constant — REMOVED

The 1-hour claim-time staking-age gate has been retired. `Constants.sol` no longer exports `MIN_STAKING_AGE`. Any script or test that imports this constant will fail to compile.

Replacement: no claim-time delay knob exists; the enforcement is `block.number > depositBlock` (implicit, no constant). Capital-commitment now lives at exit via `MIN_POST_ACTIVITY_COOLDOWN = 1 hours` (new constant) gating `withdrawUnstaked`.

---

## 🟡 New — 2026-07-08 (post-activity cooldown, verifier switch delay)

### `HemiProveStaking.markProverActivity(address) external onlyHemiProve`

Called by HemiProve on every settlement path (`_fulfillJobInternal` success, `_releaseLockedJob`, `_executeSlash`, `_forfeitBid`) to bump the post-activity cooldown anchor. Not user-callable.

### `HemiProveRegistry.executePendingReactivation(bytes32) external onlyOwner`

Applies a proposed verifier switch after `_verifierSwitchDelay` has elapsed. Emits `AdapterReactivated(psId, newAdapter)`.

### `HemiProveRegistry.cancelPendingReactivation(bytes32) external onlyOwner`

Cancels a proposed verifier switch during the delay window. Emits `VerifierSwitchCancelled(psId, rejectedAdapter)`.

### `HemiProveRegistry.setVerifierSwitchDelay(uint32) external onlyOwner`

Updates `_verifierSwitchDelay`. Bounded `[MIN_VERIFIER_SWITCH_DELAY, MAX_VERIFIER_SWITCH_DELAY]` = `[1h, 30d]`. Values below 1h revert `VerifierSwitchDelayTooShort`; values above 30d revert `VerifierSwitchDelayTooLong`. Emits `VerifierSwitchDelayUpdated`.

### View helpers on `HemiProveRegistry`

- `verifierSwitchDelay() external view returns (uint32)`
- `pendingReactivation(bytes32) external view returns (address newAdapter, uint40 readyAt)`

### New events

- `HemiProveRegistry.VerifierSwitchProposed(bytes32 indexed psId, address indexed newAdapter, uint40 readyAt)`
- `HemiProveRegistry.VerifierSwitchCancelled(bytes32 indexed psId, address indexed rejectedAdapter)`
- `HemiProveRegistry.VerifierSwitchDelayUpdated(uint32 oldDelay, uint32 newDelay)`

### New errors

- `StakingBlockDelayNotMet(address prover, uint40 depositBlock)` — same-block claim attempt.
- `PostActivityCooldownActive(address prover, uint40 unlockAt)` — withdraw before post-activity cooldown elapses.
- `VerifierSwitchDelayNotElapsed(bytes32 psId, uint40 readyAt)` — `executePendingReactivation` called too early.
- `NoPendingVerifierSwitch(bytes32 psId)` — execute/cancel with nothing pending.
- `VerifierSwitchDelayTooLong(uint32 requested, uint32 max)` — setter above 30d cap.
- `VerifierSwitchDelayTooShort(uint32 requested, uint32 min)` — setter below 1h floor.

---

## Storage layout (upgrade-safe)

- `HemiProveStorage` — unchanged (still `__gap[4]`).
- `HemiProveStaking` — added `_lastProverActivityAt` mapping; `__gap[37]` → `[36]`.
- `HemiProveRegistry` — added `_pendingReactivations` mapping + `_verifierSwitchDelay` uint32; `__gap[43]` → `[41]`.
- `ProverStake` struct field rename only (slot layout byte-identical).

Regenerate `.storage-layout/*.json` before shipping.

---

## 🔴 Breaking — 2026-07-07 (`initialize`, `updateFacets`, struct fields, removed fast paths, events and errors)

### `HemiProve.initialize` — 14 params → 15 params

Added `burnDestination` as the 15th positional arg (was previously deployed via a separate `setBurnDestination` call). Deploy scripts already handle this; frontend/indexer only relevant if they simulate initialize.

### `updateFacets` / `proposeUpdateFacets` — 6 args → 9 args

`extraImpls[]` and `extraSelectors[][]` added for FulfillCycle + AuxGov extra-facet support. If your ops tooling calls these, regenerate the ABI.

### Struct field additions

Any indexer decoding these structs via a static ABI must rebuild:

| Struct | Old fields | New fields | Notes |
|---|---:|---:|---|
| `CycleConfig` | 5 | **8** | Adds `overshootBps`, `undershootPenaltyBps`, `resolverRewardBps` for tolerance-band saturation |
| `PendingCycleSettlement` | 8 | **11** | Adds cycle-committee fields |
| `AuctionConfig` | 10 | **11** | Adds `laxCallbackMode` (caller opts prover out of callback-OOG slashing) |

### `PrivateInputCallback.commitments` getter signature changed

Was: `commitments(bytes32 jobId) view returns (bytes32)`
Now: `commitments(bytes32 jobId, address submitter) view returns (bytes32)`

**Selector changes.** Anyone reading commitments must pass both keys. Store commit is now keyed on `msg.sender`, verify against `context.caller` at proof-check time. Closes a mempool front-run DoS.

### `HemiProveFulfill.claimAndFulfillJob` and `submitAndFulfillJob` — **REMOVED**

The two atomic fastpath selectors are gone entirely. Deploy scripts do not register them. Any tooling that called them must switch to the standard two-step flow:
- `submitJob` → `claimJob` → `fulfillJob` (`claimJob(jobId, priceFloor, premiumFloor)` since 2026-10-17)
- `submitJob` → `fulfillJobHashed` (if you want the hashed-descriptor gas optimization)

### `HemiProveRegistry.setEwmaExcluded` and `setMaxEwmaBootstrapPrice` — governance-only

Previously `excludeFromEwma` was a caller-supplied `AuctionConfig` field. Now it's forced by governance via `setEwmaExcluded(bytes32 proofSystemId, bytes32 programId, bool excluded)`. Caller-set field is silently overwritten. Frontend: hide the checkbox.

### Removed events (5)

These were never actually emitted on any live deploy — pure ABI slimming. Safe to drop from event decoders:

- `JobBatchFulfilled`
- `BatchSettlementFailed`
- `GraceBidReleased`
- `CycleCommitExceeded`
- `OperatorCapBelowLocked`

**Kept for ABI compat**: `SlashClaimFulfillPartialRevert` (no longer emitted but signature preserved). **Superseded on 2026-10-26:** removed from every ABI file.

### Removed custom errors (12)

Pure ABI slimming; none were reachable. Drop from selector→error-name decoder tables:

- `InsufficientCollateral`, `InvalidBidConfig`, `MixedProofSystems`, `CustomVerifierCannotBatch`
- `DelegatedReleaseLimitExceeded`, `DelegatedReleaseTotalLimitExceeded`
- `SignatureExpired`, `OperatorFrozen`, `NotPaused`
- `CycleCollateralTooSmall`, `InvalidCycleAttestationMode`, `InvalidParameterBounds`

### Removed staking APIs

`stakeForDelegated` and `requestUnstakeDelegated` on `HemiProveStaking` were dead code and are gone. Prover self-staking only. Selectors will 404.

### `deprecateAdapter` signature

Now requires **two** args: `deprecateAdapter(bytes32 proofSystemId, bytes32 successorId)`. Old single-arg version reverts.

### `rescueBalance` semantics

New requirement: `to == from`. Rescue can only send balance back to the address that owns it. `rescueBalance(to, otherAddr)` reverts with `"to must equal from"`.

---

## 🟡 New (additive; safe to ignore initially)

### `event FeeRateSnapshotted(bytes32 indexed jobId, uint16 feeRateBps)`

Emitted in the same tx as `JobSubmitted`. Pin this per-jobId for accurate fee accounting off-chain — the protocol fee at fulfill will use *this* rate, not the current `Registry.feeRateBps` (so a fee change cannot sandwich a job). Indexers should join `JobSubmitted` and `FeeRateSnapshotted` on `jobId`.

### `_effectiveFeeRateBpsAt(bytes32 jobId, bytes32 proofSystemId)` — internal view helper

Not called externally, but if you build a future atomic facet, use this helper (in `HemiProveBase`) instead of re-implementing the snapshot decode. Keeps the fee-rate snapshot from regressing.

### Governance timelock introduced for these setters

The following setters now go through a 48h `TimelockController`-style propose/execute flow, so calling them directly reverts unless the timelock has already scheduled the change:

- `HemiProveStaking.setAuthorizedHemiProve`
- Registry / BCC / ProgramRegistry UUPS `upgradeToAndCall`
- `HemiProveAuxGov.setBatchCycleCommittee`

Off-chain must submit via `proposeXxx` then wait 48h then `finalizeXxx`. Ownership-transfer scripts document this in `DEPLOY_RUNBOOK.md`.

### New `HemiProveAuxGov` selectors

Two selectors bring AuxGov from 28 to 30 selectors:
- `setEwmaExcluded(bytes32 proofSystemId, bytes32 programId, bool excluded)`
- `setMaxEwmaBootstrapPrice(uint96 maxPrice)`

### `getEffectiveFeeRateBps(bytes32 jobId, bytes32 proofSystemId)` — public equivalent

Not added yet; if you need the snapshot fee off-chain, decode from `FeeRateSnapshotted` event.

### Job field: `_releasePenaltyBpsAtSubmit[jobId]` and `_feeRateBpsAtSubmit[jobId]`

Both are `internal` mappings with `+1` offset encoding (a stored 0 = a pre-snapshot job, fall back to live rate). Not directly readable but the derived event (`FeeRateSnapshotted`) exposes the value.

---

## ⚪ Cosmetic / behavioral

### `_forfeitBid` now bumps `reopenCount`, stamps `lastReopenTimestamp`, resets `competitiveBidStartTime[jobId]`

> Superseded in part on 2026-10-13: a forfeit no longer resets `competitiveBidStartTime[jobId]` (the round marker; `claimJob` reverts `BiddingActive` for the rest of the round), and it burns the soft-lock instead of crediting the job's bonus (the 2026-10-13 input-binding entry). The `reopenCount` bump and the `lastReopenTimestamp` stamp stay. Since 2026-10-15 the forfeit is `forfeitBid`, and it clears the round marker again, after resuming the ramp at the best live bid.

Off-chain grief-detection heuristics that track "auction restart via forfeit" now see `reopenCount++` for every forfeit. Bounded by `MAX_REOPEN_COUNT = 10`. Fresh side-effect: cancelJob's `MIN_REOPEN_CANCEL_DELAY = 24h` clock now applies to forfeit cycles too (capped at 9×24h = 216h before the cap-bypass kicks in and cancel goes through immediately).

### `HemiProveStaking.applyReleasePenalty` / `forfeitCollateral` / `distributeSlash` now `nonReentrant`

Defense-in-depth. Dormant against current hookless HEMI ERC20; only observable if a future ERC777-style token migration or malicious `burnDestination` calls back into Staking. No behavior change on happy path.

### EWMA warmup (samples #2-#5 use 150% ceiling instead of 400%)

Reduces early-lifecycle price ratchet. Advisory only (`suggestPriceRange` view). Any indexer that recomputes EWMA off-chain needs to mirror the `sampleCount < EWMA_WARMUP_SAMPLES (5)` branch.

### `gracePauseRelease` now uses MIN of pause timestamps when both paused (`localPauseTimestamp()` removed on 2026-10-13)

Previously used MAX (later timestamp). Now uses MIN (earlier). Effect: countdown to release can only advance, never regress, when both registry and local pause are active. Off-chain callers computing "when can I gracePauseRelease" should switch to `min(registry.pauseTimestamp(), hemiProve.localPauseTimestamp())`. Since 2026-10-13: `localPauseTimestamp()` is gone. Use the router's `getPauseAccounting().anchor` while the router is paused, and `registry.pauseTimestamp()` for adapter jobs only (a `CUSTOM_VERIFIER` job ignores a Registry pause). A pause that is lifted and re-taken restarts the countdown at the new pause start.

### ProgramRegistry: 3 non-paginated getters removed

- `getProgramVersions(bytes32 programKey)` — use `getProgramVersionsPaginated(programKey, 0, type(uint256).max)`
- `getVersionImages(bytes32 versionId)` — use `getVersionImagesPaginated(...)`
- `getApplicationPrograms(bytes32 applicationId)` — use `getApplicationProgramsPaginated(...)`

Also removed: `getEffectiveIOSchemaHash(imageId, kind)` — compute `keccak256(getEffectiveIOSchema(imageId, kind))` client-side.

### ProgramRegistry: 10 `public constant` → `internal constant` (auto-getters removed)

`MAX_NAME_LENGTH`, `MAX_DESCRIPTION_LENGTH`, `MAX_SCHEMA_URI_LENGTH`, `MAX_URI_LENGTH`, `MAX_SCHEMA_SIZE`, `SCHEMA_KIND_INPUT`, `MAX_BATCH_SIZE`, `MIN_COMMIT_BOND`, `COMMIT_REVEAL_DELAY`, `COMMIT_EXPIRY`.

Kept public: `MAX_URIS_PER_PROGRAM`, `MAX_REGISTRATION_FEE`. Frontend should hardcode the removed values from source (they haven't changed) or query them one-time from a source snapshot.

### Deploy profile metadata strip

`FOUNDRY_PROFILE=deploy` sets `bytecode_hash = "none"` and `cbor_metadata = false`. Runtime bytecode is ~53 bytes shorter but Etherscan / Sourcify auto-verification degrades to partial match. Manual source verification (upload flattened source + settings) still works.

### `ProgramRegistryHelpers` external library added

New standalone library at 945 B. `ProgramRegistry` DELEGATECALLs into it for `injectEncodingMode` + `paginateMemory`. Deploy scripts auto-link via Forge's `new X()` pattern. If you deploy `ProgramRegistry` via raw bytecode blob (bypassing Forge's linker), you MUST manually patch the `__$hash$__` placeholder.

---

## Not changed (stable signatures — no rebuild needed)

> This list predates 2026-10-13: `submitJob`, `fulfillJob` and `fulfillJobHashed` changed selector with the 10-field `JobDescriptor` and the 8-field `HashedDescriptor` (see the 2026-10-13 input-binding entry). The rest still holds, except the `Job` field count (annotated below).

- `submitJob(descriptor, auction, cycle, bid) → bytes32`
- `claimJob(bytes32)` (**changed:** `claimJob(bytes32,uint96,uint96) returns (uint96)` since 2026-10-17; so did `claimJobDelegated`, `claimJobBatch` and `slashAndClaim`)
- `fulfillJob(bytes32, descriptor, publicValues, proof)`
- `fulfillJobHashed(bytes32, hashedDesc, callbackExtraData, extraVerifierData, publicValues, proof)`
- `slashAndReopen(bytes32)`, `slashAndClaim(bytes32)`, `releaseJob(bytes32)`, `cancelJob(bytes32)` (since 2026-10-15 `releaseJob` keeps its selector but is Locked only, and the forfeit is the new `forfeitBid(bytes32)`; since 2026-10-17 `slashAndClaim` is `slashAndClaim(bytes32,uint96) returns (uint96)` and `forfeitBid` is removed (`refundBid`))
- `stake(prover, uint128)`, `requestUnstake(uint128)`, `withdrawUnstaked()`
- `getProverStake(address)`, `getProverStats(address)`
- All `AdapterEntry` fields unchanged
- All `BidState`, `ProverStake`, `ProverStats` field orderings preserved (since 2026-10-15 `BidState` gains a trailing 7th member, `wallBidDeadline`, in free bytes; `getBidState` returns it, the 6-value `bidStates` getter does not)
- `Job` struct still 27 packed fields across 8 storage slots — critical for storage-layout upgrade safety. Since the input binding (2026-10-13): 29 fields (`assignedOnly`, `pauseBase` added in free bytes), still 8 slots; the `jobs()` getter keeps the 27-field tuple. Since the premium cap (2026-10-13): 30 fields (`premiumUsedPpm`, free bytes of slot 6), still 8 slots; `jobs()` unchanged.

---

## Recommended off-chain rebuild checklist

1. **Regenerate ABIs** from the JSON files in this `/abi/` directory. Merge the 5 facet ABIs into a single "router ABI" attached to the `HEMIPROVE` address (see `HemiProveRouter.abi.json`).
2. **Update struct decoders** for `CycleConfig`, `PendingCycleSettlement`, `AuctionConfig` (5→8, 8→11, 10→11 fields respectively).
3. **Add `FeeRateSnapshotted` handler** to the event pipeline; join with `JobSubmitted` on jobId.
4. **Remove decoders** for the 5 removed events + 12 removed errors (safe; they never fired).
5. **Update `deprecateAdapter` calls** to the 2-arg form.
6. **Update `rescueBalance` calls** to `to == from`.
7. **Remove any `claimAndFulfillJob` / `submitAndFulfillJob` calls** — switch to the standard 3-step flow.
8. **Skip `stakeForDelegated` / `requestUnstakeDelegated`** — provers self-stake only.
9. **Set explicit `gasLimit >= 5_000_000` on all `fulfillJob` transactions** — see INTERACTION_GUIDE.md §9. Superseded on 2026-10-13: 5,000,000 is below the router's preflight for larger callbacks and calldata; size the limit by the §9 formula (≥ 4,600,000 + `callbackGasLimit` + ~250k + calldata gas + the FIELDS cost for adapter jobs) or with `eth_estimateGas`.
10. **Cache the descriptor struct** at submit time; pass the *identical* struct at fulfill time (hash-checked).
11. **(2026-10-10, 2026-10-11) Every regenerated ABI file:** rebuild each decoder from the regenerated file, not from a diff against an old copy. This covers `BatchCycleCommittee`, `HemiProveRouter` and the per-facet files, `HemiProveRegistry`, `ProgramRegistry` (now with its linked-library errors) and the new `GovernanceTimelock` and `GuardianPauseRelay` files. The per-file deltas are under "Regenerated non-BCC ABI files" (2026-10-10) and in the 2026-10-11 GuardianPauseRelay entry. For BatchCycleCommittee, add decoders for `PauseClockSet`, `PauseClockDropped`, `getPauseAccounting` and `getEpochPauseCredit`, and drop `NotInitialized` from the BCC decoder only (HemiProveRegistry and ProgramRegistry still raise it). Switch every deadline computation to effective deadlines (lower bounds while the clock is frozen).
12. **(2026-10-10) Committee bots:** implement keeper rules 0, 1a–1c, 2 and 6 and confirm to the BCC owner before your first commit (hard precondition).
13. **(2026-10-13) Descriptors and events:** re-encode every `JobDescriptor` (10 fields) and `HashedDescriptor` (8 fields) and every submit/fulfill selector; give every job a binding (FULL or FIELDS) unless it names its prover; re-key `JobCancelled` and `BidForfeited`; read deadlines through `getJobExtras` / `getJobStatusView`; replace `localPauseTimestamp()` with `getPauseAccounting()`.
14. **(2026-10-13) Journal, premium and exits:** check journal size (≤ `MAX_JOURNAL_BYTES` = 32,768), binding and transaction size before claiming; read `getPremiumCap`; expect `bonusAmount == 0`; re-encode `extendDeadlinesForDowntime` (6 arguments) and decode `getPresetDeposit` as `uint256` and `ProofResultStore.results` as 3 fields; forfeit only before the bid deadline.
15. **(2026-10-15) Locks, bids and gas:** never broadcast a journal or seal before your own lock is mined (claim alone, wait for the receipt, re-read `jobs(id).prover`, then fulfil); escalate the tip on the same nonce; fulfil until `lockDeadline + PROVER_GRACE` and slash from `getJobStatusView().slashOpensAt`; decode `JobStatusView` (23 fields) and `BidState` (7 members); treat `RegistryPaused`, `AdapterDisabled` and `BlacklistedVerifier` at fulfil as keep-the-lock; forfeit with `forfeitBid` (removed on 2026-10-17: `refundBid`); read `getPremiumCap` as the budget left; give forge-script fulfils an explicit `{gas: N}`.
16. **(2026-10-17) Floors, bids and pins:** re-encode `claimJob`, `claimJobDelegated`, `claimJobBatch` and `slashAndClaim` with floors and pass the exact floors of an `eth_call` at a mined block (never 0, `minPrice` or `getCurrentPrice`); treat `ClaimBelowFloor` and `BiddingActive` as wait-and-retry; replace the forfeit with `refundBid` (a governance cause only) or settle + release; drop `BidForfeited` and `forfeitCollateral`; decode `JobStatusView` (22 members) and `getJobSnapshots`; submit competitive jobs with `speedPremium = 0` and `300 ≤ underbidWindow ≤ maxBiddingDuration ≤ 7 d`; run the 2026-10-17 (or later) daemons only, one sender per staked key.

---

*Generated 2026-07-07. Live devnet cycles verified with real Groth16 proof.*
