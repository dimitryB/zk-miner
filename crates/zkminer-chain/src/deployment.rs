//! The deployed HemiProve stack zkminer talks to, and its startup verification (handoff §2, §13
//! test 1-2).
//!
//! Before the first transaction zkminer proves it is talking to the AUDITED code: every proxy is the
//! audited ERC-1967 proxy, every implementation and router facet it routes through is audited code,
//! and the RISC Zero adapter wraps the audited risc0-ethereum v3.0 verifier whose `SELECTOR()` is the
//! seal prefix zkminer emits. The pins below are copied verbatim from the deployed contracts repo
//! (`script/lib/AuditedCorePins.sol`, `script/lib/VerifierPins.sol` at zkmarket c8333aa) and are
//! DEPLOYMENT-INDEPENDENT: an implementation's pin is its runtime keccak with its immutables zeroed,
//! so the same audited code verifies at any address. Only the addresses and the scan floor are
//! per-network.

use alloy::primitives::{address, b256, keccak256, Address, B256};
use alloy::providers::Provider;
use anyhow::{bail, Context, Result};

use crate::auction::ProofSystem;
use crate::client::ChainClient;
use zkminer_contracts::bindings::{
    IHemiProveAux, IHemiProveRegistry, IHemiProveStaking, IRiscZeroSelectable, IVerifierAdapter,
};

/// Audited runtime pins (zkmarket c8333aa `script/lib/AuditedCorePins.sol`, `VerifierPins.sol`).
pub mod pins {
    use super::*;
    /// OpenZeppelin ERC1967Proxy (no immutable) — every core proxy.
    pub const ERC1967_PROXY: B256 =
        b256!("46a010df31eb642b47284650e80051e8f6239a0c995003d230b437810bc4b9b4");
    /// HemiProve router implementation (its one immutable, UUPS `__self`, zeroed).
    pub const ROUTER_IMPL: B256 =
        b256!("fe19d149183c16c2c5917745002d824fb94dc5f1766d8ecf2f1e2ab4c5f6d2aa");
    /// HemiProveStaking implementation (`__self` zeroed).
    pub const STAKING_IMPL: B256 =
        b256!("8f2769b1ac7e443981de10adba5962fd42552bd14f10006b5f30921eb70aea94");
    /// HemiProveRegistry implementation (`__self` zeroed).
    pub const REGISTRY_IMPL: B256 =
        b256!("f6ea9179aa36f691770372da03089f89bd0f6b0e52a7976ad749167f9b4388b2");
    /// ProgramRegistry implementation (`__self` zeroed).
    pub const PROGRAM_REGISTRY_IMPL: B256 =
        b256!("2b4b0456f6994a01cf2adf63ad65a2b1073cbfe3d1000cc8417beb0a9c7d5203");
    /// The five router facets (no immutable): HemiProveCore, Fulfill, Aux, FulfillCycle, AuxGov.
    pub const FACETS: [(&str, B256); 5] = [
        ("HemiProveCore", b256!("96787200886d4ae567581ca248d382ffbd333c161a8f44a361573d376a024cff")),
        ("HemiProveFulfill", b256!("5c18136573c55777c269943877a5965785fda93a82160a594b52e9cbecfaaffc")),
        ("HemiProveAux", b256!("f7dd82f12ef38cb7ab190611497898d07b77d583043cb0d604bab0de05ecb3ba")),
        ("HemiProveFulfillCycle", b256!("60675be0e8b7698b9c432039668443a867b30776cd7548fcbcad98c034118bcc")),
        ("HemiProveAuxGov", b256!("500035cdde2939e214b1ab1d92e04328c9d75291f9e1081c07da6f4c0be0b879")),
    ];
    /// RiscZeroGroth16VerifierAdapter runtime with its `nativeVerifier` immutable zeroed at
    /// [`RISC0_GROTH16_ADAPTER_IMMUTABLES`].
    pub const RISC0_GROTH16_ADAPTER: B256 =
        b256!("dc362a202817f67a52ef910bf022660f3723a4b7fc5e451f9799be71fb7b0a89");
    pub const RISC0_GROTH16_ADAPTER_IMMUTABLES: [usize; 3] = [101, 526, 1245];
    /// risc0-ethereum v3.0 RiscZeroGroth16Verifier runtime (immutables filled, raw keccak).
    pub const RISC0_V3_0_GROTH16_VERIFIER: B256 =
        b256!("0d2be77d8f4614c7c80be0da5f3b2a836b72b8650071d14006fbf43ecf34cf29");
    /// Its `SELECTOR()`: the 4-byte prefix of every risc0 3.0 Groth16 seal it accepts.
    pub const RISC0_V3_0_SELECTOR: [u8; 4] = [0x73, 0xc4, 0x57, 0xba];
}

/// ERC-1967 implementation slot: `bytes32(uint256(keccak256("eip1967.proxy.implementation")) - 1)`.
const ERC1967_IMPL_SLOT: B256 =
    b256!("360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc");

/// A known deployment: the addresses an operator's config must name on that chain.
#[derive(Debug, Clone, Copy)]
pub struct KnownDeployment {
    pub name: &'static str,
    pub chain_id: u64,
    pub router: Address,
    pub staking: Address,
    pub registry: Address,
    pub token: Address,
    pub program_registry: Address,
    /// The deployment record's `fromBlock`: no HemiProve log exists below it, so every log scan
    /// (job discovery, lock recovery, descriptor reconstruction) is floored here.
    pub scan_from_block: u64,
    /// Raw runtime keccak of the stake token (no repo pin exists for the testnet token).
    pub token_codehash: B256,
}

/// Hemi testnet, deployed 2026-10-29 from zkmarket c8333aa (handoff §1-§2). Every older 743111 stack
/// — and the old free-mint tHEMI `0x60525eEB…E698` — is abandoned.
pub const HEMI_TESTNET_743111: KnownDeployment = KnownDeployment {
    name: "Hemi testnet (743111, 2026-10-29)",
    chain_id: 743111,
    router: address!("E1a492Dc6D57fCC2B6deb5B64087E69666cA470e"),
    staking: address!("44b2DDCfBe37900917E3750A4996BA007F4aA66E"),
    registry: address!("77fDb1DCE2224B75BA295f2Cdd87Ba5247d4BBc6"),
    token: address!("46955EdCF61b23a8c0b0d71428A9F1C07Bbf36cc"),
    program_registry: address!("CC761A929747A81e34517e1f4F98247da925cAb4"),
    scan_from_block: 7_095_745,
    token_codehash: b256!("536084d8a9d1773c86d8dd25aef9e619741e48e072dbbfdc30cad81bc2521aba"),
};

/// The known deployment on `chain_id`, if any. (Hemi mainnet, 43111, has none yet.)
pub fn known_deployment(chain_id: u64) -> Option<&'static KnownDeployment> {
    [&HEMI_TESTNET_743111]
        .into_iter()
        .find(|d| d.chain_id == chain_id)
}

/// keccak256 of `code` with every occurrence of the 20-byte `self_addr` zeroed — a UUPS
/// implementation's `__self` immutable is its own address, so this is its audited-pin hash.
pub fn keccak_self_zeroed(code: &[u8], self_addr: Address) -> B256 {
    let needle = self_addr.as_slice();
    let mut buf = code.to_vec();
    let mut i = 0;
    while i + needle.len() <= buf.len() {
        if &buf[i..i + needle.len()] == needle {
            buf[i..i + needle.len()].fill(0);
            i += needle.len();
        } else {
            i += 1;
        }
    }
    keccak256(&buf)
}

/// keccak256 of `code` with a 32-byte immutable slot zeroed at each offset (the repo's
/// `*_IMMUTABLES` byte offsets). `None` if an offset runs past the code.
pub fn keccak_offsets_zeroed(code: &[u8], offsets: &[usize]) -> Option<B256> {
    let mut buf = code.to_vec();
    for &off in offsets {
        buf.get_mut(off..off + 32)?.fill(0);
    }
    Some(keccak256(&buf))
}

/// Whether the operator opted out of the audited-code checks (e.g. a governance upgrade landed
/// before this zkminer shipped its pins). Never skips the chain-id, address or wiring checks.
fn code_pins_skipped() -> bool {
    std::env::var("ZKMINER_SKIP_CODE_PINS").ok().as_deref() == Some("1")
}

/// A proof system that was READ and is definitely not serveable right now (disabled, swapped to
/// unaudited code, a SELECTOR mismatch) — as opposed to an RPC failure, which says nothing. Only this
/// clears a previous "verified" verdict.
#[derive(Debug, Clone)]
pub struct ProofSystemRefused(pub String);

impl std::fmt::Display for ProofSystemRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ProofSystemRefused {}

impl ChainClient {
    /// Verify the configured stack before the first transaction (handoff §13 test 1): the chain is a
    /// known deployment's (or the operator explicitly allowed another), the router's own wiring names
    /// the configured Staking/Registry/token, and every proxy, implementation and router facet zkminer
    /// calls through is the audited code. Returns a human summary on success.
    pub async fn verify_deployment(&self) -> Result<Vec<String>> {
        let mut report = Vec::new();
        // 1. Known deployment: refuse a dead stack on a known chain.
        match known_deployment(self.chain_id) {
            Some(k) => {
                let mismatches: Vec<String> = [
                    ("hemi_prove (router)", self.hemi_prove, k.router),
                    ("hemi_prove_staking", self.hemi_prove_staking, k.staking),
                    ("hemi_prove_registry", self.hemi_prove_registry, k.registry),
                    ("hemi_token", self.hemi_token, k.token),
                    // Program metadata (ELF URIs) comes only from here: a missing or stale address
                    // would pass every other check and then refuse every program's ELF.
                    (
                        "program_registry",
                        self.program_registry.unwrap_or(Address::ZERO),
                        k.program_registry,
                    ),
                ]
                .into_iter()
                .filter(|(_, got, want)| got != want)
                .map(|(n, got, want)| format!("{n} is {got}, the live deployment's is {want}"))
                .collect();
                if !mismatches.is_empty() {
                    if std::env::var("ZKMINER_ALLOW_UNKNOWN_DEPLOYMENT").ok().as_deref() == Some("1") {
                        tracing::warn!(
                            "configured contracts differ from {} ({}) — continuing because \
                             ZKMINER_ALLOW_UNKNOWN_DEPLOYMENT=1",
                            k.name,
                            mismatches.join("; ")
                        );
                    } else {
                        bail!(
                            "the configured contracts are not {} — {}. Older stacks on this chain are \
                             abandoned; regenerate the config with `zkminer init --network testnet` (or set \
                             ZKMINER_ALLOW_UNKNOWN_DEPLOYMENT=1 to target another deployment on purpose)",
                            k.name,
                            mismatches.join("; ")
                        );
                    }
                } else {
                    report.push(format!("addresses: {}", k.name));
                }
            }
            None => report.push(format!(
                "addresses: no known deployment on chain {} — trusting the config",
                self.chain_id
            )),
        }

        // 2. Wiring: the router's own getters must name the configured contracts.
        let aux = IHemiProveAux::new(self.hemi_prove, &*self.provider);
        let staking = aux.staking().call().await.context("router.staking()")?;
        let registry = aux.registry().call().await.context("router.registry()")?;
        let token = aux.hemiToken().call().await.context("router.hemiToken()")?;
        let staking_token = IHemiProveStaking::new(self.hemi_prove_staking, &*self.provider)
            .hemiToken()
            .call()
            .await
            .context("Staking.hemiToken()")?;
        for (what, got, want) in [
            ("router.staking()", staking, self.hemi_prove_staking),
            ("router.registry()", registry, self.hemi_prove_registry),
            ("router.hemiToken()", token, self.hemi_token),
            ("Staking.hemiToken()", staking_token, self.hemi_token),
        ] {
            if got != want {
                bail!("{what} is {got} but the config names {want} — refusing to start");
            }
        }
        report.push("wiring: router.staking/registry/hemiToken and Staking.hemiToken match".into());

        // 3. Audited code.
        if code_pins_skipped() {
            tracing::warn!(
                "ZKMINER_SKIP_CODE_PINS=1: NOT verifying that the router, Staking, Registry and their \
                 facets run the audited code"
            );
            report.push("code: SKIPPED (ZKMINER_SKIP_CODE_PINS=1)".into());
            return Ok(report);
        }
        let mut proxies = vec![
            ("router", self.hemi_prove, pins::ROUTER_IMPL),
            ("Staking", self.hemi_prove_staking, pins::STAKING_IMPL),
            ("Registry", self.hemi_prove_registry, pins::REGISTRY_IMPL),
        ];
        if let Some(pr) = self.program_registry {
            proxies.push(("ProgramRegistry", pr, pins::PROGRAM_REGISTRY_IMPL));
        }
        for (name, proxy, impl_pin) in proxies {
            let code = self.provider.get_code_at(proxy).await.with_context(|| format!("{name} code"))?;
            if keccak256(&code) != pins::ERC1967_PROXY {
                bail!("{name} {proxy} is not the audited ERC1967Proxy (runtime keccak {})", keccak256(&code));
            }
            let slot = self
                .provider
                .get_storage_at(proxy, ERC1967_IMPL_SLOT.into())
                .await
                .with_context(|| format!("{name} implementation slot"))?;
            let implementation = Address::from_word(B256::from(slot));
            let icode = self
                .provider
                .get_code_at(implementation)
                .await
                .with_context(|| format!("{name} implementation code"))?;
            if keccak_self_zeroed(&icode, implementation) != impl_pin {
                bail!(
                    "{name} implementation {implementation} is not the audited code (pin {impl_pin}) — \
                     refusing to start (set ZKMINER_SKIP_CODE_PINS=1 only after verifying an upgrade)"
                );
            }
        }
        // Every selector zkminer sends to the router must route to an audited facet (or be served by
        // the pinned router implementation itself: getFacet == 0).
        use alloy::sol_types::SolCall;
        use zkminer_contracts::bindings::{IHemiProveCore as C, IHemiProveFulfill as F};
        let selectors: [[u8; 4]; 16] = [
            C::claimJobCall::SELECTOR,
            F::fulfillJobCall::SELECTOR,
            F::releaseJobCall::SELECTOR,
            F::graceReleaseCall::SELECTOR,
            IHemiProveAux::getJobStatusViewCall::SELECTOR,
            IHemiProveAux::jobsCall::SELECTOR,
            IHemiProveAux::getJobExtrasCall::SELECTOR,
            IHemiProveAux::getJobSnapshotsCall::SELECTOR,
            IHemiProveAux::getPremiumCapCall::SELECTOR,
            IHemiProveAux::bidConfigsCall::SELECTOR,
            IHemiProveAux::MIN_COLLATERAL_AMOUNTCall::SELECTOR,
            IHemiProveAux::pausedCall::SELECTOR,
            IHemiProveAux::getUnionClockCall::SELECTOR,
            IHemiProveAux::getKeyClockCall::SELECTOR,
            IHemiProveAux::jobProofSystemsCall::SELECTOR,
            C::withdrawCall::SELECTOR,
        ];
        let facet_sig = alloy::primitives::keccak256("getFacet(bytes4)");
        for sel in selectors {
            let mut data = facet_sig[..4].to_vec();
            let mut word = [0u8; 32];
            word[..4].copy_from_slice(&sel);
            data.extend_from_slice(&word);
            let tx = alloy::rpc::types::TransactionRequest::default()
                .to(self.hemi_prove)
                .input(data.into());
            let out = self.provider.call(tx).await.context("router.getFacet")?;
            if out.len() < 32 {
                bail!("router.getFacet(0x{}) returned {} bytes", alloy::hex::encode(sel), out.len());
            }
            let facet = Address::from_slice(&out[12..32]);
            if facet == Address::ZERO {
                continue; // served by the pinned router implementation itself
            }
            let fcode = self.provider.get_code_at(facet).await.context("facet code")?;
            let h = keccak256(&fcode);
            if !pins::FACETS.iter().any(|(_, p)| *p == h) {
                bail!(
                    "router selector 0x{} routes to {facet}, which is not an audited facet (keccak {h})",
                    alloy::hex::encode(sel)
                );
            }
        }
        if let Some(k) = known_deployment(self.chain_id) {
            if self.hemi_token == k.token {
                let tcode = self.provider.get_code_at(self.hemi_token).await.context("token code")?;
                if keccak256(&tcode) != k.token_codehash {
                    bail!("stake token {} has unexpected code (keccak {})", self.hemi_token, keccak256(&tcode));
                }
            }
        }
        report.push(
            "code: router/Staking/Registry proxies, implementations and routed facets are the audited code"
                .into(),
        );
        Ok(report)
    }

    /// The Registry's adapter for `ps` right now: `(adapter address, status)`. Cheap (one call); used
    /// to notice an adapter swap or disable between full [`Self::verify_proof_system`] runs.
    pub async fn adapter_entry(&self, ps: ProofSystem) -> Result<(Address, u8)> {
        let reg = IHemiProveRegistry::new(self.hemi_prove_registry, &*self.provider);
        let entry = reg
            .getAdapter(ps.id())
            .call()
            .await
            .with_context(|| format!("Registry.getAdapter({})", ps.label()))?;
        Ok((entry.adapter, entry.status))
    }

    /// Verify a proof system zkminer will serve (handoff §13 test 2): its adapter is registered and
    /// claimable (Active or Deprecated), it is the audited adapter code, and it wraps the audited native
    /// verifier whose seal prefix equals `seal_prefix` (the 4 bytes zkminer prepends). Returns the
    /// adapter it verified, so a later swap (`executePendingReactivation`) can be noticed.
    pub async fn verify_proof_system(&self, ps: ProofSystem, seal_prefix: [u8; 4]) -> Result<Address> {
        let reg = IHemiProveRegistry::new(self.hemi_prove_registry, &*self.provider);
        let entry = reg
            .getAdapter(ps.id())
            .call()
            .await
            .with_context(|| format!("Registry.getAdapter({})", ps.label()))?;
        // AdapterStatus: 0 Unregistered, 1 Active, 2 Deprecated (still claimable), 3 Disabled.
        if !matches!(entry.status, 1 | 2) {
            return Err(ProofSystemRefused(format!(
                "{} adapter status is {} (not Active/Deprecated)",
                ps.label(),
                entry.status
            ))
            .into());
        }
        if ps != ProofSystem::Risc0Groth16 {
            bail!("{}: no pinned verifier — zkminer does not serve it yet", ps.label());
        }
        let native = IVerifierAdapter::new(entry.adapter, &*self.provider)
            .nativeVerifier()
            .call()
            .await
            .context("adapter.nativeVerifier()")?;
        if !code_pins_skipped() {
            let acode = self.provider.get_code_at(entry.adapter).await.context("adapter code")?;
            if keccak_offsets_zeroed(&acode, &pins::RISC0_GROTH16_ADAPTER_IMMUTABLES)
                != Some(pins::RISC0_GROTH16_ADAPTER)
            {
                return Err(ProofSystemRefused(format!(
                    "{} adapter {} is not the audited RiscZeroGroth16VerifierAdapter",
                    ps.label(),
                    entry.adapter
                ))
                .into());
            }
            let vcode = self.provider.get_code_at(native).await.context("verifier code")?;
            if keccak256(&vcode) != pins::RISC0_V3_0_GROTH16_VERIFIER {
                return Err(ProofSystemRefused(format!(
                    "{} native verifier {native} is not the audited risc0-ethereum v3.0 verifier",
                    ps.label()
                ))
                .into());
            }
        }
        let selector = IRiscZeroSelectable::new(native, &*self.provider)
            .SELECTOR()
            .call()
            .await
            .context("verifier SELECTOR()")?;
        if selector.0 != seal_prefix {
            return Err(ProofSystemRefused(format!(
                "{} verifier SELECTOR() is 0x{} but zkminer's seals are prefixed 0x{} — every proof \
                 would revert VerifierCallFailed",
                ps.label(),
                alloy::hex::encode(selector),
                alloy::hex::encode(seal_prefix)
            ))
            .into());
        }
        Ok(entry.adapter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_zeroing_matches_the_repos_method() {
        // The UUPS __self immutable appears as the implementation's own 20-byte address inside
        // PUSH32 words; zeroing every occurrence must leave the rest of the code untouched.
        let me = Address::repeat_byte(0xAB);
        let mut code = vec![0x60, 0x80];
        code.extend_from_slice(&[0u8; 12]);
        code.extend_from_slice(me.as_slice());
        code.extend_from_slice(&[0x5b, 0x00]);
        code.extend_from_slice(me.as_slice());
        let mut expect = code.clone();
        for w in expect.windows(20).enumerate().filter(|(_, w)| *w == me.as_slice()).map(|(i, _)| i).collect::<Vec<_>>() {
            expect[w..w + 20].fill(0);
        }
        assert_eq!(keccak_self_zeroed(&code, me), keccak256(&expect));
        assert_ne!(keccak_self_zeroed(&code, me), keccak256(&code));
    }

    #[test]
    fn offset_zeroing_rejects_out_of_range() {
        let code = vec![1u8; 64];
        assert!(keccak_offsets_zeroed(&code, &[32]).is_some());
        assert!(keccak_offsets_zeroed(&code, &[33]).is_none());
    }

    #[test]
    fn the_743111_deployment_is_known() {
        let k = known_deployment(743111).expect("743111 known");
        assert_eq!(k.scan_from_block, 7_095_745);
        assert_eq!(k.router, address!("E1a492Dc6D57fCC2B6deb5B64087E69666cA470e"));
        assert!(known_deployment(43111).is_none(), "mainnet has no deployment yet");
    }
}
