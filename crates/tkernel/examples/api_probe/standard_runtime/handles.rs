//! Handle scripts and TColStd_HPackedMapOfInteger cases (tkernel_standard_runtime_handles_probe.cxx).
use super::bit;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::PackedMap;
use rustycad_tkernel::standard::{Handle, StandardPersistent, StandardTransient, occ};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use std::str::SplitWhitespace;
use std::sync::atomic::{AtomicI32, Ordering};

/// Destructions of [`RuntimeProbeCounted`] objects, as the native `Deleted()` counter.
static DELETED: AtomicI32 = AtomicI32::new(0);
#[derive(Default)]
struct RuntimeProbeCounted {
    base: StandardTransient,
}
impl_transient!(
    RuntimeProbeCounted,
    "RuntimeProbeCounted",
    StandardTransient,
    base
);
impl Drop for RuntimeProbeCounted {
    fn drop(&mut self) {
        DELETED.fetch_add(1, Ordering::Relaxed);
    }
}
#[derive(Default)]
struct RuntimeProbeDerived {
    base: RuntimeProbeCounted,
}
impl_transient!(
    RuntimeProbeDerived,
    "RuntimeProbeDerived",
    RuntimeProbeCounted,
    base
);

type Slots = [Handle<StandardTransient>; 3];

fn counted() -> Handle<StandardTransient> {
    Handle::new(RuntimeProbeCounted::default()).cast()
}
/// `slots[i] = slots[j]`, the C++ copy assignment.
fn copy_assign(slots: &mut Slots, i: usize, j: usize) {
    let source = slots[j].clone();
    slots[i].clone_from(&source);
}
/// One script operation, as the native `run_handle_op`.
fn run_handle_op(slots: &mut Slots, op: &str) {
    let digit = |at: usize| {
        op.as_bytes()
            .get(at)
            .map_or(0, |byte| usize::from(byte - b'0'))
    };
    let (i, j) = (digit(1), digit(2));
    match op.as_bytes()[0] {
        b'T' => slots[i] = counted(),
        b'D' => slots[i] = Handle::new(RuntimeProbeDerived::default()).cast(),
        b'P' => slots[i] = Handle::new(StandardPersistent::default()).cast(),
        b'c' => copy_assign(slots, i, j),
        // C++ move assignment swaps the two handles; a Rust move leaves nothing to observe.
        b'm' => slots.swap(i, j),
        b'x' => slots[i].nullify(),
        b'd' => slots[i] = occ::down_cast::<RuntimeProbeDerived, _>(&slots[j]).cast(),
        b's' => slots[i] = Handle::<StandardTransient>::down_cast(&slots[j]),
        b'p' => slots[i] = Handle::from_view(&slots[j].view::<StandardTransient>()),
        b'r' => slots[i].reset(RuntimeProbeCounted::default()),
        b'z' => slots[i].nullify(),
        _ => panic!("unknown handle op {op:?}"),
    }
}
/// Counts, null flags, pairwise equality and destructions, as the native `handle_state`.
fn handle_state(slots: &Slots) -> String {
    let mut state: String = slots
        .iter()
        .map(|slot| format!("{}.", slot.ref_count()))
        .collect();
    state.extend(slots.iter().map(|slot| bit(slot.is_null())));
    state.extend([
        ':',
        bit(slots[0] == slots[1]),
        bit(slots[0] != slots[2]),
        bit(slots[1] == slots[2].as_ptr()),
        ':',
    ]);
    state + &DELETED.load(Ordering::Relaxed).to_string()
}
/// "runtime_handle_script <op>...".
pub fn handle_script_case(input: &mut SplitWhitespace<'_>) -> String {
    DELETED.store(0, Ordering::Relaxed);
    let mut trace = String::new();
    {
        let mut slots: Slots = Default::default();
        for op in input.by_ref() {
            run_handle_op(&mut slots, op);
            trace += &(handle_state(&slots) + " ");
        }
    }
    format!("{trace}end:{}", DELETED.load(Ordering::Relaxed))
}
/// Buckets, Add through a second handle, Extent and Contains, as the native loop.
fn hpacked_fill(map: &Handle<HPackedMapOfInteger>, values: &[i32]) -> String {
    let holder = map.clone();
    let mut result = format!("{} ", map.map().bucket_count());
    result.extend(
        values
            .iter()
            .map(|value| bit(holder.change_map().add(*value))),
    );
    result += &format!(" {} ", map.map().extent());
    result.extend(values.iter().map(|value| bit(map.map().contains(*value))));
    result
}
/// "runtime_hpacked <nb buckets> <value>...".
pub fn hpacked_case(input: &mut SplitWhitespace<'_>) -> String {
    let buckets: i32 = crate::common::next(input);
    let map = Handle::new(HPackedMapOfInteger::new(buckets));
    let values: Vec<i32> = input
        .map(|token| token.parse().expect("integer value"))
        .collect();
    let result = hpacked_fill(&map, &values);
    let copy = HPackedMapOfInteger::from_map_ref(&map.map());
    copy.change_map().add(123_456_789);
    let source: PackedMap<i32> = map.map().clone();
    let moved = HPackedMapOfInteger::from_map(source);
    let dynamic = map.dynamic_type().expect("live map");
    format!(
        "{result} {}{} {} {} {}",
        copy.map().extent(),
        bit(map.map().contains(123_456_789)),
        moved.map().extent(),
        dynamic.name(),
        dynamic.parent().expect("parent").name()
    )
}
