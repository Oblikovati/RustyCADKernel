//! Rust side of the Standard.hxx memory cases (tools/oracle/tkernel_standard_services_memory.cxx).
use crate::common::{guarded, hex, next};
use rustycad_tkernel::standard::memory;
use std::str::SplitWhitespace;

/// "services_standard_allocate <size>": zero check of the calloc'ed block.
pub fn allocate_case(input: &mut SplitWhitespace<'_>) -> String {
    let size: usize = next(input);
    guarded(memory::allocate(size).map(|block| {
        let checked = &block.as_slice()[..size.min(4096)];
        format!("zero={}", u8::from(checked.iter().all(|byte| *byte == 0)))
    }))
}

/// "services_standard_allocate_optimal <size>": null or live.
pub fn allocate_optimal_case(input: &mut SplitWhitespace<'_>) -> String {
    let block = memory::allocate_optimal(next(input));
    let text = if block.is_some() { "live" } else { "null" };
    memory::free(block);
    text.into()
}

/// "services_standard_reallocate <initial|-> <new>": the prefix realloc keeps.
pub fn reallocate_case(input: &mut SplitWhitespace<'_>) -> String {
    let first = input.next().expect("initial size");
    let new_size: usize = next(input);
    let old: usize = if first == "-" {
        0
    } else {
        first.parse().expect("size")
    };
    let mut slot = (first != "-").then(|| memory::allocate(old).expect("initial block"));
    if let Some(block) = slot.as_mut() {
        block
            .as_mut_slice()
            .iter_mut()
            .enumerate()
            .for_each(|(i, byte)| *byte = (i + 1) as u8);
    }
    let result = memory::reallocate(&mut slot, new_size);
    guarded(result.map(|()| {
        let bytes = slot.as_ref().expect("reallocated block").as_slice();
        hex(&bytes[..old.min(new_size).min(32)])
    }))
}

/// "services_standard_aligned <size> <alignment>": null or the alignment check.
pub fn aligned_case(input: &mut SplitWhitespace<'_>) -> String {
    let size: usize = next(input);
    let alignment: usize = next(input);
    let Some(block) = memory::allocate_aligned(size, alignment) else {
        return "null".into();
    };
    format!("aligned={}", u8::from(block.address() % alignment == 0))
}

/// "services_standard_misc": allocator type, Purge and the nulling template Free overloads.
pub fn misc_case(_: &mut SplitWhitespace<'_>) -> String {
    let mut block = memory::allocate(8).ok();
    let mut aligned = memory::allocate_aligned(8, 16);
    memory::free_and_null(&mut block);
    memory::free_aligned_and_null(&mut aligned);
    format!(
        "type={} purge={} nulled={}",
        memory::get_allocator_type() as i32,
        memory::purge(),
        u8::from(block.is_none() && aligned.is_none())
    )
}
