use rustycad_tkernel::ncollection::{AllocatedBytes, BaseAllocator, PackedInteger, PackedMap};
use rustycad_tkernel::standard::Failure;
use std::collections::BTreeMap;
use std::sync::OnceLock;
mod lifecycle;
mod node_trace;
pub use lifecycle::{failed_placement, lifetimes};
const REQUESTS: [usize; 18] = [
    0,
    1,
    2,
    7,
    8,
    15,
    16,
    31,
    32,
    33,
    63,
    64,
    65,
    127,
    128,
    257,
    usize::MAX,
    usize::MAX - 31,
];
fn expected(label: &str, actual: &str) {
    static ROWS: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    let rows = ROWS.get_or_init(|| {
        include_str!("../../../../tools/oracle/tkernel_packed_map_object_storage.expected")
            .lines()
            .map(|line| line.split_once(' ').unwrap())
            .collect()
    });
    assert_eq!(actual, rows[label], "{label}");
}
fn reserve<T: PackedInteger>(array: bool, size: usize) -> Result<AllocatedBytes, Failure> {
    if array {
        PackedMap::<T>::allocate_array_storage(size)
    } else {
        PackedMap::<T>::allocate_object_storage(size)
    }
}
fn release<T: PackedInteger>(array: bool, owner: Option<AllocatedBytes>) {
    if array {
        PackedMap::<T>::free_array_storage(owner);
    } else {
        PackedMap::<T>::free_object_storage(owner);
    }
}
fn raw_result(owner: &mut AllocatedBytes) -> String {
    assert_eq!(owner.allocator(), &BaseAllocator::common());
    let zero = owner.as_slice().iter().all(|byte| *byte == 0);
    let aligned = owner.as_ptr().addr().is_multiple_of(16);
    for (index, byte) in owner.as_mut_slice().iter_mut().enumerate() {
        *byte = (index * 17 + 165) as u8;
    }
    let sum: u64 = owner.as_slice().iter().map(|byte| u64::from(*byte)).sum();
    format!(
        "ok|{}|{}|{}|{}|{sum}",
        owner.as_slice().len(),
        u8::from(!owner.as_ptr().is_null()),
        u8::from(aligned),
        u8::from(zero)
    )
}
pub fn raw<T: PackedInteger>(name: &str, array: bool) {
    for (index, size) in REQUESTS.into_iter().enumerate() {
        let actual = match reserve::<T>(array, size) {
            Ok(mut owner) => {
                let fields = raw_result(&mut owner);
                release::<T>(array, Some(owner));
                fields
            }
            Err(failure) => format!(
                "error|{}|{}",
                failure.exception_type(),
                std::str::from_utf8(failure.what()).unwrap()
            ),
        };
        expected(
            &format!("{name}:{}:{index}", if array { "array" } else { "scalar" }),
            &actual,
        );
    }
    release::<T>(array, None);
}
pub fn placement<T: PackedInteger>(name: &str) {
    let mut scratch = [165_u8; 128];
    let addresses = [
        scratch.as_mut_ptr(),
        scratch.as_mut_ptr().wrapping_add(1),
        std::ptr::null_mut(),
    ];
    for (index, size) in [0, 1, usize::MAX].into_iter().enumerate() {
        for (slot, address) in addresses.into_iter().enumerate() {
            let result = PackedMap::<T>::placement_storage(size, address);
            PackedMap::<T>::release_placement_storage(result, addresses[(slot + 1) % 3]);
            let actual = format!(
                "{}|{}",
                u8::from(result == address),
                u8::from(scratch == [165; 128])
            );
            expected(&format!("{name}:placement:{index}:{slot}"), &actual);
        }
    }
    zero_identity::<T>(name);
}
fn zero_identity<T: PackedInteger>(name: &str) {
    let first = reserve::<T>(false, 0).unwrap();
    let second = reserve::<T>(true, 0).unwrap();
    expected(
        &format!("{name}:zero_identity"),
        &u8::from(first.as_ptr() != second.as_ptr()).to_string(),
    );
    release::<T>(false, Some(first));
    release::<T>(true, Some(second));
}
