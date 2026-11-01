//! End-to-end tests of zkminer against a local fork of the live Hemi testnet (743111).
//!
//! anvil forks 743111, so the fork runs the real HemiProve v2 deployment that zkminer's startup
//! checks pin. Each test then sets up, on the fork only:
//! - tHEMI for a submitter and a prover, minted as the token's owner;
//! - the prover's stake;
//! - the fibonacci guest in the ProgramRegistry, with a storage URI this test serves over HTTP;
//! - one job per binding case (see [`Case`]): three the miner should claim and three it must not.
//!
//! and runs the `zkminer` binary against the fork. anvil impersonates every sender, so setting up
//! needs no key.
//!
//! - `watch_only_reports_every_job`: `zkminer run --watch-only` as the prover. Checks each job's
//!   verdict, that exactly the claimable jobs would be claimed, and that nothing was sent. CPU only.
//! - `mines_the_claimable_jobs`: `zkminer run` with the prover's key: real claims, GPU proofs and
//!   fulfils, on the fork. Runs only with `ZKMINER_LOCALNET_MINE=1`.
//!
//! Both are ignored by default: they need anvil (Foundry), the release risc0 worker, and, on the
//! first run at a fork block, the testnet RPC (anvil caches what it fetches under
//! `~/.foundry/cache/rpc`, so later runs at the same block are quick):
//!
//! ```text
//! cargo build --release -p zkminer-prove-risc0 --features cuda --bin zkminer-prove-risc0-cuda
//! cargo test -p zkminer-cli --test localnet -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Environment:
//! - `ZKMINER_LOCALNET_FORK_URL`: the 743111 RPC to fork (default: the public testnet RPC).
//! - `ZKMINER_LOCALNET_FORK_BLOCK`: the block to fork at (default [`FORK_BLOCK`]; `latest` for the
//!   head).
//! - `ZKMINER_LOCALNET_RISC0_WORKER`: the risc0 worker (default:
//!   `target/release/zkminer-prove-risc0-cuda`).
//! - `ANVIL`: the anvil binary (default: `anvil` on `PATH`).
//! - `ZKMINER_LOCALNET_ECHO=1`: echo zkminer's log while it runs.
//! - `ZKMINER_LOCALNET_KEEP=1`: keep the run directory (logs, config) after a pass; a failed run
//!   always keeps it.
//! - `ZKMINER_LOCALNET_EXPECT_CLAIMS=0`: in the watch test, don't require the would-claim verdicts
//!   (for a host whose capacity or memory gates refuse every claim).
//! - `ZKMINER_LOCALNET_MINE=1`, `ZKMINER_LOCALNET_MINE_TIMEOUT_SECS` (default 1800): the mining
//!   test.
#![cfg(unix)]

use alloy::primitives::aliases::{U40, U96};
use alloy::primitives::{keccak256, Address, Bytes, B256, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolEvent;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zkminer_chain::auction::risc_zero_groth16_v1_id;
use zkminer_chain::binding::{fields_hash, full_hash};
use zkminer_chain::deployment::HEMI_TESTNET_743111 as DEPLOYMENT;
use zkminer_config::ZkMinerConfig;
use zkminer_contracts::bindings::{
    AuctionConfig, CompetitiveBidConfig, CycleConfig, IHemiProveCore, IHemiProveFulfill,
    IHemiProveStaking, IProgramRegistry, ITestnetToken, JobDescriptor, ResourceEstimate,
    StorageURI, IERC20,
};

/// The 743111 block the fork starts from by default, after the deployment's setup. Move it forward
/// whenever `zkminer_chain::deployment::HEMI_TESTNET_743111` changes: the fork would otherwise run
/// the old deployment, and zkminer's startup checks refuse it.
const FORK_BLOCK: u64 = 7_116_900;
const DEFAULT_FORK_URL: &str = "https://testnet.rpc.hemi.network/rpc";
const HEMI: u128 = 1_000_000_000_000_000_000;
/// Every job proves fibonacci(FIB_N).
const FIB_N: u32 = 10;
/// Every job is fixed-price at this.
const JOB_PRICE: u128 = 5 * HEMI;
const PROVER_STAKE: u128 = 1_000 * HEMI;
/// How long the watch test waits for every verdict.
const WATCH_TIMEOUT: Duration = Duration::from_secs(300);

/// The binding cases. Each is one job, all on the same program and input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Case {
    /// FULL: `keccak256(abi.encode(domain, psId, programId, journal))` of the real journal.
    Full,
    /// FIELDS: the journal's 8 bytes as one range.
    Fields,
    /// FULL over a journal the guest does not commit: the miner must refuse it before claiming.
    FullWrongJournal,
    /// NONE, assigned to the prover.
    NoneForUs,
    /// NONE, assigned to another prover: reserved for it.
    NoneForAnother,
    /// FULL with the journal-prefix marker, which this build does not support.
    PrefixMarker,
}

impl Case {
    const ALL: [Case; 6] = [
        Case::Full,
        Case::Fields,
        Case::FullWrongJournal,
        Case::NoneForUs,
        Case::NoneForAnother,
        Case::PrefixMarker,
    ];

    fn claimable(self) -> bool {
        matches!(self, Case::Full | Case::Fields | Case::NoneForUs)
    }

    /// Submitted before the miner starts, so it finds the job in its startup backfill; the others
    /// are submitted once it runs, so its live monitor finds them. Each half has jobs it must claim
    /// and jobs it must refuse.
    fn before_start(self) -> bool {
        matches!(
            self,
            Case::Full | Case::FullWrongJournal | Case::NoneForAnother
        )
    }

    /// What the watch-only inspector's verdict for this job must contain.
    fn inspector_verdict(self) -> String {
        let quote = format!("claim quote {} HEMI", hemi(JOB_PRICE));
        match self {
            Case::Full | Case::Fields => format!("binding satisfied; {quote}"),
            Case::NoneForUs => format!("no journal binding; {quote}"),
            Case::FullWrongJournal => "the measured journal FAILS its binding".into(),
            Case::NoneForAnother => "reserved for prover".into(),
            Case::PrefixMarker => "its binding marker is not supported".into(),
        }
    }
}

fn fib(n: u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        (a, b) = (b, a.wrapping_add(b));
    }
    a
}

/// The fibonacci guest reads a `u32` and commits the `u64` result.
fn guest_input() -> Vec<u8> {
    FIB_N.to_le_bytes().to_vec()
}

fn guest_journal() -> Vec<u8> {
    fib(FIB_N).to_le_bytes().to_vec()
}

/// `fmt_hemi`'s format: whole HEMI and two decimals.
fn hemi(wei: u128) -> String {
    format!("{}.{:02}", wei / HEMI, (wei % HEMI) / (HEMI / 100))
}

fn short_id(id: B256) -> String {
    format!("{id}")[..10].to_string()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn env_flag(name: &str) -> bool {
    matches!(std::env::var(name).as_deref(), Ok("1") | Ok("true"))
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .expect("no free local port")
        .port()
}

/// A directory for one run: the miner's HOME, its config and every log.
struct RunDir(PathBuf);

impl RunDir {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("zkminer-localnet-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".zkminer")).expect("create the run directory");
        Self(dir)
    }

    /// Remove the directory after a pass, unless asked to keep it.
    fn finish(self) {
        if env_flag("ZKMINER_LOCALNET_KEEP") {
            eprintln!("localnet: run directory kept at {}", self.0.display());
        } else {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// anvil forking 743111; killed on drop.
struct Anvil {
    child: Child,
    url: String,
}

impl Drop for Anvil {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_anvil(dir: &Path) -> Anvil {
    let port = free_port();
    let fork_url =
        std::env::var("ZKMINER_LOCALNET_FORK_URL").unwrap_or_else(|_| DEFAULT_FORK_URL.into());
    let fork_block = match std::env::var("ZKMINER_LOCALNET_FORK_BLOCK").as_deref() {
        Ok("latest") => None,
        Ok(b) => Some(
            b.parse::<u64>()
                .expect("ZKMINER_LOCALNET_FORK_BLOCK is a block number"),
        ),
        Err(_) => Some(FORK_BLOCK),
    };
    let log = std::fs::File::create(dir.join("anvil.log")).expect("create anvil.log");
    let mut cmd = Command::new(std::env::var("ANVIL").unwrap_or_else(|_| "anvil".into()));
    cmd.args(["--fork-url", &fork_url])
        .args(["--chain-id", &DEPLOYMENT.chain_id.to_string()])
        .args(["--host", "127.0.0.1", "--port", &port.to_string()])
        .args(["--block-time", "1"])
        // Every setup transaction is sent from an address anvil signs for.
        .arg("--auto-impersonate")
        .args([
            "--retries",
            "10",
            "--fork-retry-backoff",
            "1000",
            "--timeout",
            "60000",
        ])
        .stdout(log.try_clone().unwrap())
        .stderr(log);
    if let Some(b) = fork_block {
        cmd.args(["--fork-block-number", &b.to_string()]);
    }
    let child = cmd
        .spawn()
        .expect("start anvil: install Foundry, or set ANVIL to its path");
    Anvil {
        child,
        url: format!("http://127.0.0.1:{port}"),
    }
}

/// Serve `body` to every GET on a local port; counts the requests.
fn serve_file(body: Vec<u8>) -> (String, Arc<AtomicUsize>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind the ELF server");
    let url = format!("http://{}/fibonacci.bin", listener.local_addr().unwrap());
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            counter.fetch_add(1, Ordering::SeqCst);
            // Read the request head (bounded) before answering.
            let mut head = Vec::new();
            let mut byte = [0u8; 1];
            while head.len() < 16 * 1024 && !head.ends_with(b"\r\n\r\n") {
                match stream.read(&mut byte) {
                    Ok(1) => head.push(byte[0]),
                    _ => break,
                }
            }
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        }
    });
    (url, hits)
}

/// The risc0 worker the miner measures (and in the mining test, proves) with.
fn risc0_worker() -> PathBuf {
    let path = std::env::var("ZKMINER_LOCALNET_RISC0_WORKER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root().join("target/release/zkminer-prove-risc0-cuda"));
    assert!(
        path.is_file(),
        "no risc0 worker at {}: build it (cargo build --release -p zkminer-prove-risc0 --features \
         cuda --bin zkminer-prove-risc0-cuda) or set ZKMINER_LOCALNET_RISC0_WORKER",
        path.display()
    );
    path
}

/// The fibonacci guest and its image id (the jobs' `programId`), from the worker that runs it.
fn export_guest(worker: &Path, dir: &Path) -> (Vec<u8>, B256) {
    let path = dir.join("fibonacci.bin");
    let out = Command::new(worker)
        .args(["--export-guest", "fibonacci"])
        .arg(&path)
        .output()
        .expect("run the risc0 worker");
    assert!(
        out.status.success(),
        "--export-guest failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let id: B256 = String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .expect("--export-guest prints the image id");
    (std::fs::read(&path).expect("read the exported guest"), id)
}

/// Wait for a mined receipt and require success.
macro_rules! mined {
    ($call:expr, $what:expr) => {{
        let what = $what;
        let receipt = $call
            .send()
            .await
            .unwrap_or_else(|e| panic!("{what}: {e}"))
            .get_receipt()
            .await
            .unwrap_or_else(|e| panic!("{what}: no receipt: {e}"));
        assert!(
            receipt.status(),
            "{what} reverted ({})",
            receipt.transaction_hash
        );
        receipt
    }};
}

/// A job submitted for one case.
struct Job {
    case: Case,
    id: B256,
}

/// The fork, set up: a staked prover, the guest registered and served, one job per case.
struct World {
    _anvil: Anvil,
    rpc_url: String,
    provider: DynProvider,
    prover: PrivateKeySigner,
    submitter: Address,
    another_prover: Address,
    program_id: B256,
    /// The jobs submitted so far.
    jobs: Vec<Job>,
    /// The first block after setup started: every log query starts here.
    from_block: u64,
    /// Requests the ELF server answered.
    elf_hits: Arc<AtomicUsize>,
}

impl World {
    async fn new(dir: &Path, worker: &Path) -> Self {
        let anvil = start_anvil(dir);
        let provider = ProviderBuilder::new()
            .connect_http(anvil.url.parse().unwrap())
            .erased();
        // The first call waits for the fork to load.
        let started = Instant::now();
        loop {
            match provider.get_chain_id().await {
                Ok(id) => {
                    assert_eq!(id, DEPLOYMENT.chain_id);
                    break;
                }
                Err(e) if started.elapsed() > Duration::from_secs(180) => {
                    panic!(
                        "anvil did not come up ({e}); see {}/anvil.log",
                        dir.display()
                    )
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }
        let from_block = provider.get_block_number().await.unwrap() + 1;

        let (elf, program_id) = export_guest(worker, dir);
        let (elf_url, elf_hits) = serve_file(elf.clone());

        let submitter = Address::from_word(keccak256(b"zkminer localnet submitter"));
        let another_prover = Address::repeat_byte(0x42);
        let prover = PrivateKeySigner::random();
        // tHEMI is minted by its owner (`owner()`), which anvil impersonates.
        let owner_word = provider
            .call(
                TransactionRequest::default()
                    .to(DEPLOYMENT.token)
                    .input(Bytes::from_static(&[0x8d, 0xa5, 0xcb, 0x5b]).into()),
            )
            .await
            .expect("read the token's owner");
        let token_owner = Address::from_word(B256::from_slice(&owner_word));
        for a in [token_owner, submitter, prover.address()] {
            let _: serde_json::Value = provider
                .raw_request(
                    "anvil_setBalance".into(),
                    (a, U256::from(100u128) * U256::from(HEMI)),
                )
                .await
                .expect("anvil_setBalance");
        }

        let token = ITestnetToken::new(DEPLOYMENT.token, &provider);
        let erc20 = IERC20::new(DEPLOYMENT.token, &provider);
        mined!(
            token
                .mint(submitter, U256::from(1_000 * HEMI))
                .from(token_owner),
            "mint tHEMI to the submitter"
        );
        mined!(
            token
                .mint(prover.address(), U256::from(PROVER_STAKE))
                .from(token_owner),
            "mint tHEMI to the prover"
        );

        mined!(
            erc20
                .approve(DEPLOYMENT.staking, U256::from(PROVER_STAKE))
                .from(prover.address()),
            "approve the stake"
        );
        mined!(
            IHemiProveStaking::new(DEPLOYMENT.staking, &provider)
                .stake(prover.address(), PROVER_STAKE)
                .from(prover.address()),
            "stake"
        );

        // Registration is permissionless; the allowance covers a registration fee, if any.
        mined!(
            erc20
                .approve(DEPLOYMENT.program_registry, U256::from(100 * HEMI))
                .from(submitter),
            "approve the registration fee"
        );
        mined!(
            IProgramRegistry::new(DEPLOYMENT.program_registry, &provider)
                .registerProgram(
                    program_id,
                    risc_zero_groth16_v1_id(),
                    B256::ZERO,
                    ResourceEstimate {
                        estimatedCycles: 0,
                        peakMemoryBytes: 0,
                        estimatedWallTimeS: 0,
                    },
                    "fibonacci (zkminer localnet)".into(),
                    String::new(),
                    String::new(),
                    vec![StorageURI {
                        storageType: 2, // HTTP
                        uri: elf_url,
                        contentHash: keccak256(&elf),
                    }],
                )
                .from(submitter),
            "register the guest"
        );

        mined!(
            erc20
                .approve(DEPLOYMENT.router, U256::from(1_000 * HEMI))
                .from(submitter),
            "approve the job deposits"
        );
        let mut world = World {
            rpc_url: anvil.url.clone(),
            _anvil: anvil,
            provider,
            prover,
            submitter,
            another_prover,
            program_id,
            jobs: Vec::new(),
            from_block,
            elf_hits,
        };
        for case in Case::ALL.into_iter().filter(|c| c.before_start()) {
            world.submit(case).await;
        }
        world
    }

    async fn submit(&mut self, case: Case) {
        let core = IHemiProveCore::new(DEPLOYMENT.router, &self.provider);
        let receipt = mined!(
            core.submitJob(
                descriptor(
                    case,
                    self.program_id,
                    self.prover.address(),
                    self.another_prover
                ),
                fixed_price(),
                no_cycles(),
                no_bidding(),
            )
            .from(self.submitter),
            format!("submit the {case:?} job")
        );
        let id = receipt
            .inner
            .logs()
            .iter()
            .find_map(|l| l.log_decode::<IHemiProveCore::JobSubmitted>().ok())
            .map(|l| l.inner.data.jobId)
            .expect("JobSubmitted in the submit receipt");
        self.jobs.push(Job { case, id });
    }

    /// Submit the jobs held back for the live monitor, once the miner's monitor runs.
    async fn submit_after_start(&mut self, miner: &mut Miner) {
        let started = Instant::now();
        while !miner
            .lines()
            .iter()
            .any(|l| l.contains("Job monitor started"))
        {
            assert!(
                !miner.exited(),
                "the miner exited before its job monitor started"
            );
            assert!(
                started.elapsed() < Duration::from_secs(120),
                "the miner's job monitor did not start within 120 s"
            );
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        for case in Case::ALL.into_iter().filter(|c| !c.before_start()) {
            self.submit(case).await;
        }
    }

    fn job(&self, case: Case) -> B256 {
        self.jobs.iter().find(|j| j.case == case).unwrap().id
    }

    async fn prover_nonce(&self) -> u64 {
        self.provider
            .get_transaction_count(self.prover.address())
            .await
            .unwrap()
    }

    /// Jobs with a log of `E` on the fork since setup began.
    async fn jobs_with_event<E: SolEvent>(&self) -> Vec<B256> {
        let filter = Filter::new()
            .address(DEPLOYMENT.router)
            .event_signature(E::SIGNATURE_HASH)
            .from_block(self.from_block);
        self.provider
            .get_logs(&filter)
            .await
            .unwrap()
            .iter()
            .filter_map(|l| l.topics().get(1).copied())
            .collect()
    }

    /// The miner's config: the fork, the 743111 contracts, and fast polling.
    fn write_config(&self, dir: &Path, worker: &Path, discover_worker: bool) -> PathBuf {
        let mut c = ZkMinerConfig::default();
        c.chain.rpc_url = self.rpc_url.clone();
        c.chain.chain_id = DEPLOYMENT.chain_id;
        c.chain.scan_from_block = Some(self.from_block);
        c.chain.monitor_poll_secs = Some(2);
        c.contracts.hemi_prove = DEPLOYMENT.router.to_string();
        c.contracts.hemi_prove_staking = DEPLOYMENT.staking.to_string();
        c.contracts.hemi_prove_registry = DEPLOYMENT.registry.to_string();
        c.contracts.hemi_token = DEPLOYMENT.token.to_string();
        c.contracts.program_registry = DEPLOYMENT.program_registry.to_string();
        // A 5 HEMI job is the whole market here: claim anything that pays.
        c.prover.min_profit_threshold = 0.0;
        c.prover.skip_benchmark_gate = true;
        if discover_worker {
            // One worker per GPU, from a directory holding only the risc0 worker.
            let provers = dir.join("provers");
            std::fs::create_dir_all(&provers).unwrap();
            let link = provers.join(worker.file_name().unwrap());
            let _ = std::fs::remove_file(&link);
            std::os::unix::fs::symlink(worker, &link).unwrap();
            c.prover.worker_search_paths = vec![provers.display().to_string()];
        } else {
            // Named outright, so a host without a GPU still gets the risc0 measure pool.
            c.prover
                .worker_binaries
                .insert("risc0".into(), worker.display().to_string());
        }
        let path = dir.join(".zkminer/config.toml");
        c.save(Some(&path)).expect("write the miner config");
        path
    }
}

fn descriptor(case: Case, program_id: B256, prover: Address, another: Address) -> JobDescriptor {
    let psid = risc_zero_groth16_v1_id();
    let journal = guest_journal();
    let slot0 = |word: u32| {
        let mut f = B256::ZERO;
        f.0[..4].copy_from_slice(&word.to_be_bytes());
        f
    };
    let (assigned, expected, fields) = match case {
        Case::Full => (
            Address::ZERO,
            full_hash(psid, program_id, &journal),
            B256::ZERO,
        ),
        Case::Fields => {
            // One range: offset 0, length 8.
            let f = slot0(8);
            (Address::ZERO, fields_hash(&journal, f), f)
        }
        Case::FullWrongJournal => {
            let wrong = (fib(FIB_N) + 1).to_le_bytes();
            (
                Address::ZERO,
                full_hash(psid, program_id, &wrong),
                B256::ZERO,
            )
        }
        Case::NoneForUs => (prover, B256::ZERO, B256::ZERO),
        Case::NoneForAnother => (another, B256::ZERO, B256::ZERO),
        Case::PrefixMarker => (
            Address::ZERO,
            full_hash(psid, program_id, &journal),
            slot0(0xffff_ffff),
        ),
    };
    JobDescriptor {
        programId: program_id,
        proofSystemId: psid,
        callbackContract: Address::ZERO,
        assignedProver: assigned,
        tag: keccak256(format!("zkminer localnet {case:?}")),
        inputData: guest_input().into(),
        callbackExtraData: Bytes::new(),
        extraVerifierData: Bytes::new(),
        expectedJournalHash: expected,
        journalFields: fields,
    }
}

fn fixed_price() -> AuctionConfig {
    AuctionConfig {
        minPrice: U96::from(JOB_PRICE),
        maxPrice: U96::from(JOB_PRICE),
        rampUpPeriod: U40::ZERO,
        curveType: 0,
        fulfillmentTimeout: U40::from(3_600u64),
        lockCollateralBps: U96::ZERO,
        speedPremium: U96::ZERO,
        exclusivityDuration: U40::ZERO,
        callbackGasLimit: 0,
        excludeFromEwma: false,
        laxCallbackMode: false,
    }
}

fn no_cycles() -> CycleConfig {
    CycleConfig {
        expectedCycles: 0,
        cycleCommitCollateral: U96::ZERO,
        snapshotMode: 0,
        snapshotToleranceBps: 0,
        snapshotFullBonusOvershootBps: 0,
        snapshotUndershootPenaltyBps: 0,
        snapshotFullPenaltyUndershootBps: 0,
        snapshotResolverRewardBps: 0,
    }
}

fn no_bidding() -> CompetitiveBidConfig {
    CompetitiveBidConfig {
        underbidWindow: U40::ZERO,
        maxBiddingDuration: U40::ZERO,
        maxBidExtension: U40::ZERO,
        minBidDecrement: U96::ZERO,
        bidExtensionCurveType: 0,
    }
}

/// A running `zkminer`, its log lines collected as they arrive; stopped on drop.
struct Miner {
    child: Child,
    lines: Arc<Mutex<Vec<String>>>,
}

impl Miner {
    /// Start a copy of the `zkminer` binary in `dir` (its own directory is searched for workers,
    /// so a copy finds only the ones the config names), with `dir` as its HOME.
    fn start(dir: &Path, config: &Path, args: &[&str], env: &[(&str, String)]) -> Self {
        let exe = dir.join("zkminer");
        std::fs::copy(env!("CARGO_BIN_EXE_zkminer"), &exe).expect("copy the zkminer binary");
        let mut cmd = Command::new(&exe);
        cmd.arg("--config")
            .arg(config)
            .arg("run")
            .args(args)
            .env("HOME", dir)
            // The guest is served from this machine.
            .env("ZKMINER_ELF_ALLOW_PRIVATE_URLS", "1")
            .env_remove("ZKMINER_PRIVATE_KEY")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("start zkminer");
        let lines = Arc::new(Mutex::new(Vec::new()));
        let echo = env_flag("ZKMINER_LOCALNET_ECHO");
        let log = Arc::new(Mutex::new(
            std::fs::File::create(dir.join("zkminer.log")).expect("create zkminer.log"),
        ));
        for stream in [
            Box::new(child.stdout.take().unwrap()) as Box<dyn Read + Send>,
            Box::new(child.stderr.take().unwrap()),
        ] {
            let (lines, log) = (lines.clone(), log.clone());
            std::thread::spawn(move || {
                for line in BufReader::new(stream).lines().map_while(Result::ok) {
                    let line = strip_ansi(&line);
                    if echo {
                        eprintln!("zkminer| {line}");
                    }
                    let _ = writeln!(log.lock().unwrap(), "{line}");
                    lines.lock().unwrap().push(line);
                }
            });
        }
        Miner { child, lines }
    }

    fn lines(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }

    fn exited(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_some()
    }

    /// SIGINT (the miner's drain), then wait up to `grace` before killing it. Returns whether it
    /// exited on its own.
    fn stop(&mut self, grace: Duration) -> bool {
        if self.exited() {
            return true;
        }
        let _ = Command::new("kill")
            .args(["-INT", &self.child.id().to_string()])
            .status();
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            if self.exited() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        false
    }
}

impl Drop for Miner {
    fn drop(&mut self) {
        self.stop(Duration::from_secs(20));
    }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The `WATCH <job>: …` verdicts logged for `id`.
fn watch_lines(lines: &[String], id: B256) -> Vec<String> {
    let tag = format!("WATCH {}: ", short_id(id));
    lines
        .iter()
        .filter_map(|l| l.find(&tag).map(|i| l[i + tag.len()..].to_string()))
        .collect()
}

/// Print the tail of the miner's log, for a failure.
fn dump_tail(dir: &Path, lines: &[String]) {
    eprintln!(
        "---- last zkminer log lines (full log: {}/zkminer.log) ----",
        dir.display()
    );
    for l in lines.iter().rev().take(80).rev() {
        eprintln!("{l}");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs anvil, the release risc0 worker and the 743111 RPC; see the module docs"]
async fn watch_only_reports_every_job() {
    let dir = RunDir::new("watch");
    let worker = risc0_worker();
    let mut world = World::new(&dir.0, &worker).await;
    let config = world.write_config(&dir.0, &worker, false);
    let expect_claims = std::env::var("ZKMINER_LOCALNET_EXPECT_CLAIMS").as_deref() != Ok("0");
    let nonce_before = world.prover_nonce().await;

    let mut miner = Miner::start(
        &dir.0,
        &config,
        &[
            "--watch-only",
            "--address",
            &world.prover.address().to_string(),
        ],
        &[],
    );
    world.submit_after_start(&mut miner).await;

    // Every case's verdict, and a would-claim for each claimable case.
    let started = Instant::now();
    let missing = loop {
        let lines = miner.lines();
        let mut missing = Vec::new();
        for job in &world.jobs {
            let verdicts = watch_lines(&lines, job.id);
            let want = job.case.inspector_verdict();
            if !verdicts.iter().any(|v| v.contains(&want)) {
                missing.push(format!("{:?}: a verdict containing {want:?}", job.case));
            }
            if expect_claims
                && job.case.claimable()
                && !verdicts.iter().any(|v| v.starts_with("WOULD CLAIM"))
            {
                missing.push(format!("{:?}: WOULD CLAIM", job.case));
            }
        }
        if missing.is_empty() || started.elapsed() > WATCH_TIMEOUT || miner.exited() {
            break missing;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    };
    let exited_early = miner.exited();
    let stopped = miner.stop(Duration::from_secs(60));
    let lines = miner.lines();

    let mut failures = missing;
    if exited_early {
        failures.push("zkminer exited before every verdict was in".into());
    }
    if !stopped {
        failures.push("zkminer did not exit within 60 s of SIGINT".into());
    }
    for job in &world.jobs {
        // Never a would-claim for a job the miner must refuse.
        if !job.case.claimable()
            && watch_lines(&lines, job.id)
                .iter()
                .any(|v| v.starts_with("WOULD CLAIM"))
        {
            failures.push(format!("{:?}: reported WOULD CLAIM", job.case));
        }
    }
    // Watch-only sends nothing: the prover's nonce is where setup left it, no job was claimed, and
    // the send guard never had anything to refuse.
    let nonce_after = world.prover_nonce().await;
    if nonce_after != nonce_before {
        failures.push(format!(
            "the prover's nonce moved from {nonce_before} to {nonce_after}"
        ));
    }
    let claimed = world.jobs_with_event::<IHemiProveCore::JobClaimed>().await;
    if !claimed.is_empty() {
        failures.push(format!("JobClaimed on the fork for {claimed:?}"));
    }
    if lines.iter().any(|l| l.contains("refusing to send")) {
        failures.push("the send guard refused a send".into());
    }
    // The guest came from its registry URI.
    if world.elf_hits.load(Ordering::SeqCst) == 0 {
        failures.push("the guest was never downloaded from its registry URI".into());
    }

    for job in &world.jobs {
        for v in watch_lines(&lines, job.id) {
            eprintln!("localnet: {:?} {}: {v}", job.case, short_id(job.id));
        }
    }
    if !failures.is_empty() {
        dump_tail(&dir.0, &lines);
        panic!(
            "watch-only localnet run failed (run directory {}):\n  {}",
            dir.0.display(),
            failures.join("\n  ")
        );
    }
    dir.finish();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs a GPU, anvil, the release risc0 worker and ZKMINER_LOCALNET_MINE=1"]
async fn mines_the_claimable_jobs() {
    if !env_flag("ZKMINER_LOCALNET_MINE") {
        eprintln!("localnet: skipping the mining test; set ZKMINER_LOCALNET_MINE=1 to run it");
        return;
    }
    let timeout = Duration::from_secs(
        std::env::var("ZKMINER_LOCALNET_MINE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_800),
    );
    let dir = RunDir::new("mine");
    let worker = risc0_worker();
    let mut world = World::new(&dir.0, &worker).await;
    let config = world.write_config(&dir.0, &worker, true);
    let key = format!("{}", world.prover.to_bytes());

    let mut miner = Miner::start(
        &dir.0,
        &config,
        &["--headless"],
        &[("ZKMINER_PRIVATE_KEY", key)],
    );
    world.submit_after_start(&mut miner).await;

    let want: Vec<B256> = Case::ALL
        .iter()
        .filter(|c| c.claimable())
        .map(|c| world.job(*c))
        .collect();
    let started = Instant::now();
    let fulfilled = loop {
        let fulfilled = world
            .jobs_with_event::<IHemiProveFulfill::JobFulfilled>()
            .await;
        if want.iter().all(|id| fulfilled.contains(id))
            || started.elapsed() > timeout
            || miner.exited()
        {
            break fulfilled;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    };
    let exited_early = miner.exited();
    let stopped = miner.stop(Duration::from_secs(120));
    let lines = miner.lines();

    let mut failures = Vec::new();
    if exited_early {
        failures.push("zkminer exited before the jobs were fulfilled".to_string());
    }
    if !stopped {
        failures.push("zkminer did not drain and exit within 120 s of SIGINT".into());
    }
    let claimed = world.jobs_with_event::<IHemiProveCore::JobClaimed>().await;
    for job in &world.jobs {
        let (c, f) = (claimed.contains(&job.id), fulfilled.contains(&job.id));
        eprintln!(
            "localnet: {:?} {}: claimed={c} fulfilled={f}",
            job.case,
            short_id(job.id)
        );
        if job.case.claimable() && !f {
            failures.push(format!("{:?}: not fulfilled", job.case));
        }
        if !job.case.claimable() && c {
            failures.push(format!("{:?}: claimed, but must never be", job.case));
        }
    }
    if !failures.is_empty() {
        dump_tail(&dir.0, &lines);
        panic!(
            "mining localnet run failed (run directory {}):\n  {}",
            dir.0.display(),
            failures.join("\n  ")
        );
    }
    dir.finish();
}
