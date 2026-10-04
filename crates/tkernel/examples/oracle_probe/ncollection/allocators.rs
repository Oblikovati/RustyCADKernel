//! NCollection allocators probe operations (twin of `tkernel_ncollection_allocators_probe.cxx`).
//!
//! Block identities are printed as positions and offsets, never raw addresses.
// The raw Free mirrors the native pointer API, which Rust marks unsafe.
#![allow(unsafe_code)]
use rustycad_tkernel::{
    ncollection::{
        ACC_DEFAULT_BLOCK_SIZE, BaseAllocator, Buffer, INC_DEFAULT_BLOCK_SIZE, OccAllocator,
        StandardAllocator,
    },
    standard::{Failure, dump::DumpStream},
};
use std::{ptr::NonNull, str::SplitWhitespace};

/// The output of an allocators operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    match op {
        "ncoll_allocators_inc" => Some(incremental(input)),
        "ncoll_allocators_acc" => Some(accumulating(input)),
        "ncoll_allocators_acc_extension" => Some(accumulating_extension()),
        "ncoll_allocators_aligned" => Some(aligned(input)),
        "ncoll_allocators_buffer" => Some(buffer(input)),
        "ncoll_allocators_occ_eq" => Some(occ_equality()),
        "ncoll_allocators_instance_identity" => Some(allocator_instance_identity()),
        _ => None,
    }
}

fn number(input: &mut SplitWhitespace<'_>) -> usize {
    input
        .next()
        .expect("numeric field")
        .parse()
        .expect("unsigned integer")
}

/// Labels incremental blocks by appearance order; see the native `IncBlockLabels`.
#[derive(Default)]
struct IncBlockLabels {
    starts: Vec<usize>,
    ends: Vec<usize>,
}

impl IncBlockLabels {
    fn label(&mut self, address: usize, size: usize) -> String {
        if let Some(index) = self.ends.iter().position(|end| *end == address) {
            self.ends[index] = address + size;
            return format!("b{index}+{}", address - self.starts[index]);
        }
        self.starts.push(address);
        self.ends.push(address + size);
        format!("b{}+0", self.starts.len() - 1)
    }
    fn rewind(&mut self) {
        self.ends.clone_from(&self.starts);
    }
}

fn incremental(input: &mut SplitWhitespace<'_>) -> String {
    let allocator = BaseAllocator::incremental(number(input));
    let count = number(input);
    let mut labels = IncBlockLabels::default();
    let mut out = String::new();
    for _ in 0..count {
        let step = input.next().expect("step");
        if step == "r0" || step == "r1" {
            allocator.reset(step == "r1").expect("no live owners");
            if step == "r1" {
                labels = IncBlockLabels::default(); // released blocks: addresses may be reused
            } else {
                labels.rewind();
            }
            out.push_str(&format!("{step} "));
            continue;
        }
        let size: usize = step[1..].parse().expect("allocation size");
        let address = allocator.allocate_raw(size).expect("allocation").as_ptr() as usize;
        out.push_str(&format!("{} ", labels.label(address, size)));
    }
    out
}

/// Chain position of the block holding `address` and the offset from its start.
fn acc_where(allocator: &BaseAllocator, address: usize) -> String {
    let block = allocator.accumulating_block(address);
    let chain = allocator.accumulating_chain();
    match block.and_then(|state| chain.iter().position(|item| *item == state)) {
        Some(position) => format!("{position}+{}", address - chain[position].address),
        None => "none".to_owned(),
    }
}

fn acc_chain(allocator: &BaseAllocator) -> String {
    let items: String = allocator
        .accumulating_chain()
        .iter()
        .map(|state| format!("{}:{};", state.free_size, state.alloc_count))
        .collect();
    format!("[{items}]")
}

fn accumulating(input: &mut SplitWhitespace<'_>) -> String {
    let allocator = BaseAllocator::accumulating(number(input)).expect("nonzero block size");
    let count = number(input);
    let mut addresses: Vec<NonNull<u8>> = Vec::new();
    let mut out = acc_chain(&allocator);
    for _ in 0..count {
        let step = input.next().expect("step");
        out.push(' ');
        if let Some(size) = step.strip_prefix('a') {
            let address = allocator
                .allocate_raw(size.parse().expect("size"))
                .expect("allocation");
            addresses.push(address);
            out.push_str(&acc_where(&allocator, address.as_ptr() as usize));
        } else {
            out.push_str(&acc_free(&allocator, step, &addresses));
        }
        out.push_str(&acc_chain(&allocator));
    }
    out
}

fn accumulating_extension() -> String {
    let allocator = BaseAllocator::accumulating(4096).expect("block size");
    let original = allocator.allocate_raw(8).expect("original allocation");
    let mut extension = allocator
        .accumulating_extension()
        .expect("accumulator extension");
    let (old, before, new, extended) = extension_allocations(&mut extension, original);
    drop(extension);
    free_extension_allocation(&allocator, extended);
    let chain = acc_chain(&allocator);
    let miss_text = extension_miss(&allocator);
    free_extension_allocation(&allocator, original);
    format!("{old} {before} {new} {chain} {miss_text}")
}

fn extension_allocations(
    extension: &mut rustycad_tkernel::ncollection::AccAllocatorExtension<'_>,
    original: NonNull<u8>,
) -> (String, String, String, NonNull<u8>) {
    let address = original.as_ptr() as usize;
    let old = lookup_text(extension.find_block(address), address, 4096);
    let block = extension.allocate_new_block(4096).expect("new block");
    let before = format!("{}:{}", block.free_size, block.alloc_count);
    let pointer = extension
        .allocate_from_block(block.address, 5)
        .expect("extension allocation");
    let address = pointer.as_ptr() as usize;
    let new = lookup_text(extension.find_block(address), address, 4096);
    (old, before, new, pointer)
}

fn extension_miss(allocator: &BaseAllocator) -> String {
    let local = 0_i32;
    let lookup = allocator
        .accumulating_extension()
        .expect("accumulator extension")
        .find_block((&local as *const i32) as usize);
    if lookup.block.is_none() {
        "miss:1"
    } else {
        "hit"
    }
    .to_owned()
}

fn free_extension_allocation(allocator: &BaseAllocator, allocation: NonNull<u8>) {
    // SAFETY: this pointer is a live allocation tracked by this allocator exactly once.
    unsafe { allocator.free_raw(allocation) }.expect("release extension block allocation");
}

fn lookup_text(
    lookup: rustycad_tkernel::ncollection::AccBlockLookup,
    address: usize,
    size: usize,
) -> String {
    match lookup.block {
        Some(block) => format!(
            "hit:{}:{}:{}:{}",
            u8::from(lookup.key.value() == block.address / size),
            block.free_size,
            block.alloc_count,
            address - block.address
        ),
        None => format!(
            "miss:{}",
            u8::from(lookup.key.value() == address / size - 1)
        ),
    }
}

fn acc_free(allocator: &BaseAllocator, step: &str, addresses: &[NonNull<u8>]) -> String {
    let local = 0_i32;
    let address = match step {
        "x" => NonNull::from(&local).cast::<u8>(),
        _ => addresses[step[1..].parse::<usize>().expect("allocation index")],
    };
    // SAFETY: the corpus frees each live allocation of a block at most once before the block
    // is released; foreign addresses are detected without touching any block.
    match unsafe { allocator.free_raw(address) } {
        Ok(()) => "freed".to_owned(),
        Err(failure) => format!("throw {}", failure.exception_type()),
    }
}

fn aligned(input: &mut SplitWhitespace<'_>) -> String {
    let alignment = number(input);
    let allocator = BaseAllocator::aligned(alignment);
    let Some(address) = allocator.allocate_optimal_raw(number(input)) else {
        return "null".to_owned();
    };
    let is_aligned = alignment == 0 || (address.as_ptr() as usize).is_multiple_of(alignment);
    // SAFETY: address is a live allocation of this allocator.
    unsafe { allocator.free_raw(address) }.expect("heap free");
    format!("aligned {}", u8::from(is_aligned))
}

fn allocator_named(name: &str) -> Option<BaseAllocator> {
    match name {
        "common" => Some(BaseAllocator::common()),
        "heap" => Some(BaseAllocator::global_heap()),
        "inc" => Some(BaseAllocator::incremental(INC_DEFAULT_BLOCK_SIZE)),
        "acc" => Some(BaseAllocator::accumulating(ACC_DEFAULT_BLOCK_SIZE).expect("default size")),
        "null" => None,
        _ => Some(BaseAllocator::aligned(
            name[1..].parse().expect("alignment"),
        )),
    }
}

/// Replaces every `0xx<hex>` pointer text by `PTR`, like the native regex.
fn mask_pointers(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("0xx") {
        out.push_str(&rest[..start]);
        let tail = &rest[start + 3..];
        let digits = tail.find(|c: char| !c.is_ascii_hexdigit() || c.is_ascii_uppercase());
        let length = digits.unwrap_or(tail.len());
        out.push_str(if length == 0 { "0xx" } else { "PTR" });
        rest = &tail[length..];
    }
    out + rest
}

fn buffer_state(buffer: &Buffer) -> String {
    let mut stream = DumpStream::default();
    buffer.dump_json(&mut stream, -1).expect("in-memory dump");
    format!(
        "{} {} {{{}}}",
        u8::from(buffer.is_empty()),
        buffer.size(),
        mask_pointers(stream.text())
    )
}

/// Buffer construction, reallocation and release; a raised Free appends `throw <type>`.
fn buffer(input: &mut SplitWhitespace<'_>) -> String {
    let allocator = allocator_named(input.next().expect("allocator name"));
    let (size, resize) = (number(input), number(input));
    let mut out = String::new();
    if let Err(failure) = buffer_steps(allocator, size, resize, &mut out) {
        out.push_str(&format!("throw {}", failure.exception_type()));
    }
    out
}

fn buffer_steps(
    allocator: Option<BaseAllocator>,
    size: usize,
    resize: usize,
    out: &mut String,
) -> Result<(), Failure> {
    let mut buffer = Buffer::new(allocator, size)?;
    out.push_str(&format!("{} | ", buffer_state(&buffer)));
    let allocated = buffer.allocate(resize)?;
    out.push_str(&format!(
        "{} {} | ",
        u8::from(allocated),
        buffer_state(&buffer)
    ));
    buffer.free()?;
    out.push_str(&buffer_state(&buffer));
    Ok(())
}

fn occ_equality() -> String {
    let incremental = BaseAllocator::incremental(INC_DEFAULT_BLOCK_SIZE);
    let allocators = [
        OccAllocator::<i32>::default(),
        OccAllocator::new(BaseAllocator::common()),
        OccAllocator::new(BaseAllocator::global_heap()),
        OccAllocator::new(incremental),
    ];
    let mut out = String::new();
    for left in &allocators {
        for right in &allocators {
            let rebound = right.rebind::<f64>();
            let flags = [left == right, left != right, *left == rebound];
            out.extend(flags.map(|flag| if flag { '1' } else { '0' }));
        }
        out.push(' ');
    }
    let heap = BaseAllocator::global_heap();
    format!(
        "{out}{} {} {} {} {} {} {}{}",
        OccAllocator::<u8>::default().max_size(),
        OccAllocator::<i16>::default().max_size(),
        OccAllocator::<i32>::default().max_size(),
        OccAllocator::<f64>::default().max_size(),
        OccAllocator::<[u8; 24]>::default().max_size(),
        u8::from(StandardAllocator::<i32>::new() == StandardAllocator::<f64>::new()),
        u8::from(heap == BaseAllocator::global_heap()),
        u8::from(heap == BaseAllocator::common()),
    )
}

fn allocator_instance_identity() -> String {
    let base_one = BaseAllocator::fresh_common();
    let heap_one = BaseAllocator::fresh_heap();
    free_probe_allocation(&base_one);
    free_probe_allocation(&heap_one);
    allocator_identity_flags()
        .map(|flag| if flag { '1' } else { '0' })
        .iter()
        .collect()
}

fn allocator_identity_flags() -> [bool; 7] {
    let common = BaseAllocator::common();
    let heap = BaseAllocator::global_heap();
    let fresh_common = BaseAllocator::fresh_common();
    let second_common = BaseAllocator::fresh_common();
    let fresh_heap = BaseAllocator::fresh_heap();
    let second_heap = BaseAllocator::fresh_heap();
    [
        common == BaseAllocator::common(),
        heap == BaseAllocator::global_heap(),
        common != heap,
        fresh_common != common,
        fresh_common != second_common,
        fresh_heap != heap,
        fresh_heap != second_heap,
    ]
}

fn free_probe_allocation(allocator: &BaseAllocator) {
    let allocation = allocator
        .allocate_raw(8)
        .expect("fresh allocator allocation");
    // SAFETY: this allocation came from the same heap-policy handle and is released once.
    unsafe { allocator.free_raw(allocation) }.expect("release fresh allocator allocation");
}
