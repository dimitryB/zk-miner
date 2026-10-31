//! Isolated capability probe for the validated Metal Merkle-fold stage.
//!
//! The worker supervisor runs this in a bounded subprocess. A compiler, device,
//! or dispatch failure therefore cannot silently select a different proving mode.

use anyhow::{ensure, Context, Result};
use risc0_zkp::{
    core::digest::Digest,
    field::baby_bear::{BabyBear, BabyBearElem},
    hal::{cpu::CpuHal, metal::MetalHalPoseidon2, metal_fold::MetalPoseidon2Fold, Buffer, Hal},
};
use serde_json::{json, Value};

/// Compile the retained kernels and compare real, bounded Metal work with CPU.
/// This checks device usability; complete proof acceptance is a separate gate.
pub fn probe(staging_bytes: usize) -> Result<Value> {
    ensure!(
        (4096..=16 * 1024 * 1024).contains(&staging_bytes),
        "Metal probe staging budget must be between 4 KiB and 16 MiB"
    );
    let (device_name, max_buffer_bytes, recommended_working_set_bytes, unified_memory, suite) =
        metal::objc::rc::autoreleasepool(|| {
            let hal = MetalHalPoseidon2::new();
            (
                hal.device.name().to_owned(),
                hal.device.max_buffer_length() as u64,
                hal.device.recommended_max_working_set_size(),
                hal.device.has_unified_memory(),
                hal.get_hash_suite().clone(),
            )
        });
    ensure!(
        unified_memory,
        "Metal hybrid proving requires unified memory"
    );
    ensure!(
        max_buffer_bytes >= staging_bytes as u64,
        "Metal maximum buffer size is below the required staging budget"
    );
    ensure!(
        recommended_working_set_bytes >= staging_bytes as u64,
        "Metal recommended working set is below the required staging budget"
    );

    // At the default budget this exercises a full staging chunk and a tail,
    // with a nonzero CPU buffer offset and unchanged prefix/suffix sentinels. All digest words are reduced
    // Montgomery BabyBear values, matching the retained Poseidon2 representation.
    const OUTPUTS: usize = 4097;
    const OFFSET: usize = 3;
    const GUARD: usize = 5;
    let sentinel = Digest::from([BabyBearElem::new(17).as_u32_montgomery(); 8]);
    let mut initial = vec![sentinel; OFFSET + OUTPUTS * 4 + GUARD];
    for i in 0..OUTPUTS * 2 {
        initial[OFFSET + OUTPUTS * 2 + i] = Digest::from(std::array::from_fn(|j| {
            BabyBearElem::new((i as u32).wrapping_mul(37).wrapping_add(j as u32 * 73))
                .as_u32_montgomery()
        }));
    }
    let cpu = CpuHal::<BabyBear>::new(suite.clone());
    let hybrid =
        CpuHal::<BabyBear, _>::with_hash_fold(suite, MetalPoseidon2Fold::new(staging_bytes)?);
    let expected = cpu.copy_from_digest("metal_probe_expected", &initial);
    let actual = hybrid.copy_from_digest("metal_probe_actual", &initial);
    cpu.hash_fold(&expected.slice(OFFSET, OUTPUTS * 4), OUTPUTS * 2, OUTPUTS);
    hybrid.hash_fold(&actual.slice(OFFSET, OUTPUTS * 4), OUTPUTS * 2, OUTPUTS);
    let mut matched = false;
    expected.view(|expected| actual.view(|actual| matched = expected == actual));
    ensure!(
        matched,
        "Metal fold probe differs from CPU output or changed sentinels"
    );

    let stats = hybrid
        .metal_stats()
        .context("Metal probe produced no stage counters")?;
    let outputs_per_chunk = staging_bytes / (4 * std::mem::size_of::<Digest>());
    let expected_dispatches = OUTPUTS.div_ceil(outputs_per_chunk) as u64;
    ensure!(
        stats.completed_dispatches == expected_dispatches && stats.fold_calls == 1,
        "Metal probe did not complete the required GPU chunks"
    );
    ensure!(
        stats.output_digests == OUTPUTS as u64 && stats.input_digests == (OUTPUTS * 2) as u64,
        "Metal probe processed an unexpected amount of work"
    );
    ensure!(
        stats.peak_staging_bytes
            == outputs_per_chunk.min(OUTPUTS) * 4 * std::mem::size_of::<Digest>()
            && stats.staging_limit_bytes == staging_bytes,
        "Metal probe staging budget was not enforced"
    );

    Ok(json!({
        "device_name": device_name,
        "max_buffer_bytes": max_buffer_bytes,
        "recommended_working_set_bytes": recommended_working_set_bytes,
        "unified_memory": unified_memory,
        "completed_dispatches": stats.completed_dispatches,
        "stage": "segment_poseidon2_merkle_fold",
        "staging_limit_bytes": stats.staging_limit_bytes,
        "peak_staging_bytes": stats.peak_staging_bytes,
        "fold_calls": stats.fold_calls,
        "input_digests": stats.input_digests,
        "output_digests": stats.output_digests,
        "differential": "matched_cpu",
        "compiler": "runtime_source"
    }))
}
