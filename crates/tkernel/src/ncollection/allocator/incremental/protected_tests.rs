//! Read-only field assertions bind native protected growth to the actual next-block state.
use super::IncEngine;

#[test]
fn protected_growth_updates_every_native_level_without_allocating() {
    let fixture = include_str!(
        "../../../../../../tools/oracle/tkernel_allocator_protected_operations.expected"
    );
    for size in [2048, 12288, 196608, 786432, 1000001, 12582912] {
        let mut engine = IncEngine::new(size);
        engine.increase_block_size();
        let row = format!("grow-{size} {}", engine.block_size);
        assert!(
            fixture.lines().any(|expected| expected == row),
            "native growth row {row}"
        );
        assert_eq!(engine.block_count, 0);
        assert!(engine.blocks.is_empty());
        assert_eq!(engine.live, 0);
    }
}
