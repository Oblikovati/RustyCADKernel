//! Named failing block source checks native metadata prefixes without inducing host OOM.
use super::{IncBlockLink, IncEngine};

struct FailingBlockSource;
impl FailingBlockSource {
    fn allocate(_size: usize) -> Option<IncBlockLink> {
        // A real backend refusal returns before any allocation or link publication.
        None
    }
}

fn failed_row(engine: &mut IncEngine, size: usize, label: &str) -> String {
    let before = engine.observation();
    let failed = engine
        .push_block_with(size, FailingBlockSource::allocate)
        .is_none();
    let after = engine.observation();
    assert_eq!(after.blocks, before.blocks);
    format!(
        "{label} {} {} {} {} {} {}",
        u8::from(failed),
        after.block_count,
        after.block_size,
        u8::from(before.ordered_blocks == after.ordered_blocks),
        u8::from(before.used_heap == after.used_heap),
        u8::from(before.allocation_heap == after.allocation_heap)
    )
}

fn first_failure_rows() -> Vec<String> {
    let mut engine = IncEngine::new(2048);
    let failure = failed_row(&mut engine, 4096, "first-failure");
    engine.allocate(8).unwrap();
    vec![
        failure,
        format!("first-retry {} {}", engine.block_count, engine.block_size),
    ]
}

fn fifth_failure_rows() -> Vec<String> {
    let mut engine = IncEngine::new(2048);
    for _ in 0..4 {
        engine.allocate(2048).unwrap();
    }
    let failure = failed_row(&mut engine, 8, "fifth-failure");
    engine.allocate(8).unwrap();
    vec![
        failure,
        format!("fifth-retry {} {}", engine.block_count, engine.block_size),
    ]
}

fn available_failure_rows() -> Vec<String> {
    let mut engine = IncEngine::new(2048);
    engine.allocate(8).unwrap();
    let failure = failed_row(&mut engine, 8192, "available-failure");
    engine.allocate(8).unwrap();
    vec![
        failure,
        format!(
            "available-reuse {} {}",
            engine.block_count, engine.block_size
        ),
    ]
}

#[test]
fn block_failure_preserves_exact_native_prefixes_and_existing_headers() {
    let actual = [
        first_failure_rows(),
        fifth_failure_rows(),
        available_failure_rows(),
    ]
    .concat();
    let fixture =
        include_str!("../../../../../../tools/oracle/tkernel_inc_allocator_failed_blocks.expected");
    assert_eq!(actual.join("\n") + "\n", fixture);
}

#[test]
fn checked_rust_overflow_rejects_before_mutating_native_attempt_state() {
    let mut engine = IncEngine::new(2048);
    engine.allocate(8).unwrap();
    let before = engine.observation();
    assert!(engine.allocate(usize::MAX).is_none());
    assert!(engine.allocate_aligned(usize::MAX, 64).is_none());
    assert_eq!(engine.observation(), before);
}
