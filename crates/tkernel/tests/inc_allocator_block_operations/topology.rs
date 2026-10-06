//! Each native graph row compares every canonical header field and all three actual heads.
use rustycad_tkernel::ncollection::{BaseAllocator, IncAllocatorBlock, IncAllocatorObservation};

fn identity(state: &IncAllocatorObservation, address: usize) -> usize {
    state
        .blocks
        .iter()
        .position(|block| block.address == address)
        .map_or(0, |index| index + 1)
}

fn block_rows(state: &IncAllocatorObservation) -> Vec<String> {
    state
        .blocks
        .iter()
        .map(|block| {
            format!(
                "{}:{}:{}:{}:{}",
                identity(state, block.address),
                block.current - block.address - size_of::<IncAllocatorBlock>(),
                block.available,
                identity(state, block.next),
                identity(state, block.ordered_next)
            )
        })
        .collect()
}

fn snapshot(arena: &BaseAllocator, mode: bool, step: usize) -> String {
    let state = arena.incremental_extension().unwrap().observation();
    let prefix = format!(
        "graph-{}-{step} {} {} {} {} {}",
        u8::from(mode),
        state.block_size,
        state.block_count,
        identity(&state, state.allocation_heap),
        identity(&state, state.used_heap),
        identity(&state, state.ordered_blocks)
    );
    std::iter::once(prefix)
        .chain(block_rows(&state))
        .collect::<Vec<_>>()
        .join(" ")
}

fn append_snapshot(arena: &BaseAllocator, mode: bool, rows: &mut Vec<String>) {
    let row = snapshot(arena, mode, rows.len());
    rows.push(row);
}

fn allocate_rows(arena: &BaseAllocator, mode: bool, rows: &mut Vec<String>, sizes: &[usize]) {
    for &size in sizes {
        arena.allocate_optimal_raw(size).unwrap();
        append_snapshot(arena, mode, rows);
    }
}

fn block_reset_rows(arena: &BaseAllocator, mode: bool, rows: &mut Vec<String>) {
    let state = arena.incremental_extension().unwrap().observation();
    for address in [state.used_heap, state.ordered_blocks] {
        arena
            .incremental_extension()
            .unwrap()
            .reset_block(address)
            .unwrap();
        append_snapshot(arena, mode, rows);
    }
    arena
        .incremental_extension()
        .unwrap()
        .allocate_slow(32)
        .unwrap();
    append_snapshot(arena, mode, rows);
    arena.reset(false).unwrap();
    append_snapshot(arena, mode, rows);
}

fn topology_rows(mode: bool) -> Vec<String> {
    let arena = BaseAllocator::incremental(2048);
    arena.set_thread_safe(mode).unwrap();
    let mut rows = vec![snapshot(&arena, mode, 0)];
    allocate_rows(
        &arena,
        mode,
        &mut rows,
        &[1600, 1700, 64, 64, 64, 0, 2048, 8, 8, 16000, 8, 8],
    );
    block_reset_rows(&arena, mode, &mut rows);
    allocate_rows(&arena, mode, &mut rows, &[16, 32, 64, 128, 2048, 10000]);
    arena.incremental_extension().unwrap().clean().unwrap();
    append_snapshot(&arena, mode, &mut rows);
    rows
}

#[test]
fn all_intrusive_graph_transitions_match_native_in_both_modes() {
    let actual = [topology_rows(false), topology_rows(true)]
        .concat()
        .join("\n")
        + "\n";
    let expected = include_str!("../../../../tools/oracle/tkernel_inc_allocator_topology.expected");
    assert_eq!(actual, expected);
}
