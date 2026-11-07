//! Descriptor hash computation — must match `_hashDescriptorFields` (HemiProveBase) exactly.
//!
//! ```solidity
//! keccak256(abi.encode(
//!     programId,
//!     proofSystemId,
//!     callback,
//!     assignedProver,
//!     tag,
//!     keccak256(inputData),
//!     keccak256(callbackExtraData),
//!     keccak256(extraVerifierData),
//!     expectedJournalHash,              // 9th slot, raw
//!     journalFields                     // 10th slot, raw
//! ))
//! ```

use alloy_primitives::{keccak256, Address, B256};

/// Compute the descriptor hash matching the on-chain `_hashDescriptorFields`.
///
/// `expected_journal_hash` and `journal_fields` are the job's journal binding (raw `bytes32`, NOT
/// pre-hashed). Both zero is a NONE job (no journal check; it requires an assigned prover).
#[allow(clippy::too_many_arguments)]
pub fn compute_descriptor_hash(
    program_id: B256,
    proof_system_id: B256,
    callback_contract: Address,
    assigned_prover: Address,
    tag: B256,
    input_data: &[u8],
    callback_extra_data: &[u8],
    extra_verifier_data: &[u8],
    expected_journal_hash: B256,
    journal_fields: B256,
) -> B256 {
    compute_descriptor_hash_from_hashed(
        program_id,
        proof_system_id,
        callback_contract,
        assigned_prover,
        tag,
        keccak256(input_data),
        callback_extra_data,
        extra_verifier_data,
        expected_journal_hash,
        journal_fields,
    )
}

/// Compute the descriptor hash from pre-hashed input data (the `fulfillJobHashed` path, whose
/// 8-field `HashedDescriptor` carries both binding words — the hashed path serves bound jobs too).
#[allow(clippy::too_many_arguments)]
pub fn compute_descriptor_hash_from_hashed(
    program_id: B256,
    proof_system_id: B256,
    callback_contract: Address,
    assigned_prover: Address,
    tag: B256,
    input_data_hash: B256,
    callback_extra_data: &[u8],
    extra_verifier_data: &[u8],
    expected_journal_hash: B256,
    journal_fields: B256,
) -> B256 {
    let callback_extra_data_hash = keccak256(callback_extra_data);
    let extra_verifier_data_hash = keccak256(extra_verifier_data);

    // abi.encode packs each value into a 32-byte word: 10 * 32 = 320 bytes.
    let mut encoded = Vec::with_capacity(320);
    encoded.extend_from_slice(program_id.as_slice());
    encoded.extend_from_slice(proof_system_id.as_slice());
    encoded.extend_from_slice(&address_to_word(callback_contract));
    encoded.extend_from_slice(&address_to_word(assigned_prover));
    encoded.extend_from_slice(tag.as_slice());
    encoded.extend_from_slice(input_data_hash.as_slice());
    encoded.extend_from_slice(callback_extra_data_hash.as_slice());
    encoded.extend_from_slice(extra_verifier_data_hash.as_slice());
    encoded.extend_from_slice(expected_journal_hash.as_slice());
    encoded.extend_from_slice(journal_fields.as_slice());

    keccak256(&encoded)
}

/// Convert an address to a left-padded 32-byte word (abi.encode format).
fn address_to_word(addr: Address) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[12..].copy_from_slice(addr.as_slice());
    word
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    fn sample(ejh: B256, jf: B256) -> B256 {
        compute_descriptor_hash(
            B256::ZERO,
            keccak256("risczero.groth16.v1"),
            address!("0000000000000000000000000000000000000001"),
            Address::ZERO,
            B256::ZERO,
            b"test input",
            b"",
            b"",
            ejh,
            jf,
        )
    }

    #[test]
    fn test_descriptor_hash_deterministic_and_covers_both_binding_words() {
        let ejh = B256::repeat_byte(0x99);
        let jf = B256::repeat_byte(0x11);
        assert_eq!(sample(ejh, jf), sample(ejh, jf));
        assert_ne!(sample(ejh, jf), sample(B256::ZERO, jf), "expectedJournalHash is hashed");
        assert_ne!(sample(ejh, jf), sample(ejh, B256::ZERO), "journalFields is hashed");
    }

    #[test]
    fn test_hashed_descriptor_matches() {
        let ejh = B256::repeat_byte(0x99);
        let jf = B256::repeat_byte(0x11);
        let hash_hashed = compute_descriptor_hash_from_hashed(
            B256::ZERO,
            keccak256("risczero.groth16.v1"),
            address!("0000000000000000000000000000000000000001"),
            Address::ZERO,
            B256::ZERO,
            keccak256(b"test input"),
            b"",
            b"",
            ejh,
            jf,
        );
        assert_eq!(sample(ejh, jf), hash_hashed);
    }
}
