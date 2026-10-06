use super::super::*;
use std::fmt::Write;
struct RetainedFixture {
    target: DynamicArray<ParameterItem>,
    counts: Rc<ParameterCounts>,
    allocator: BaseAllocator,
    live: usize,
    allocated: usize,
    freed: usize,
}
impl RetainedFixture {
    fn new(increment: usize) -> Self {
        let allocator = BaseAllocator::accumulating(4096).unwrap();
        Self {
            target: DynamicArray::with_allocator(increment, Some(allocator.clone())),
            counts: Rc::new(ParameterCounts::default()),
            allocator,
            live: 0,
            allocated: 0,
            freed: 0,
        }
    }
    fn append_range(&mut self, first: i32, count: i32) -> Vec<*mut ParameterItem> {
        for value in first..first + count {
            self.target.append(ParameterItem::new(value, &self.counts));
        }
        raw_initialized_slots(&mut self.target)
    }
    unsafe fn copy_parameters(&mut self, increment: usize, adopt: bool, same_owner: bool) {
        let owner = if same_owner {
            self.allocator.clone()
        } else {
            BaseAllocator::accumulating(4096).unwrap()
        };
        let source = DynamicArray::with_allocator(increment, Some(owner));
        // Caller keeps old raw slots live until explicit destruction or valid publication.
        unsafe {
            self.target
                .storage_extension()
                .copy_parameters_retained(&source, adopt)
                .unwrap()
        };
        assert_eq!(self.target.allocator(), Some(&self.allocator));
    }
    fn snapshot(&mut self) -> [usize; 7] {
        let live = allocator_live(&self.allocator);
        self.allocated += live.saturating_sub(self.live);
        self.freed += self.live.saturating_sub(live);
        self.live = live;
        [
            self.target.size(),
            self.target.storage_extension().block_count(),
            self.target.storage_extension().available_size(),
            self.allocated,
            self.freed,
            self.counts.constructed.get(),
            self.counts.destroyed.get(),
        ]
    }
    unsafe fn row(&mut self, name: &str, slots: &[*mut ParameterItem]) -> String {
        let mut output = name.to_owned();
        for value in self.snapshot() {
            write!(output, " {value}").unwrap();
        }
        // Only original live slots are supplied, including forgotten but not yet destroyed values.
        let values = unsafe { raw_values(slots) };
        writeln!(output, " {values}").unwrap();
        output
    }
}
fn raw_initialized_slots(target: &mut DynamicArray<ParameterItem>) -> Vec<*mut ParameterItem> {
    let used = target.size();
    let mut storage = target.storage_extension();
    let increment = storage.available_size() / storage.block_count();
    (0..used)
        .map(|index| {
            let base = storage.change_block_at(index / increment).unwrap();
            // Derive raw pointers after construction, without retaining &mut-return tags across appends.
            unsafe { base.add(index % increment) }
        })
        .collect()
}
fn joined_values(values: impl Iterator<Item = i32>) -> String {
    let output = values
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    if output.is_empty() {
        "-".to_owned()
    } else {
        output
    }
}
unsafe fn raw_values(slots: &[*mut ParameterItem]) -> String {
    // Caller retained live raw objects; safe array length may intentionally be zero.
    joined_values(slots.iter().map(|pointer| unsafe { (**pointer).value }))
}
unsafe fn restore_fixture(fixture: &mut RetainedFixture, used: usize) {
    // The caller proves the current-stride prefix still consists of the original live objects.
    unsafe {
        fixture
            .target
            .storage_extension()
            .set_used_size(used)
            .unwrap()
    };
}
fn equal_rows() -> String {
    let mut fixture = RetainedFixture::new(2);
    let slots = fixture.append_range(1, 3);
    let mut output = unsafe { fixture.row("equal_before", &slots) };
    unsafe { fixture.copy_parameters(2, true, true) };
    output.push_str(&unsafe { fixture.row("equal_forgotten", &slots) });
    unsafe { restore_fixture(&mut fixture, 3) };
    assert_eq!(retained_values(&fixture.target), [1, 2, 3]);
    output.push_str(&unsafe { fixture.row("equal_restored", &slots) });
    fixture.target.clear(true);
    output.push_str(&unsafe { fixture.row("equal_cleared", &[]) });
    output
}
unsafe fn destroy_forgotten(slots: &[*mut ParameterItem]) {
    for pointer in slots {
        // Each slot was unpublished, remains live, and is destroyed exactly once here.
        unsafe { DynamicArrayStorageExtension::destroy_at(*pointer) };
    }
}
fn smaller_rows() -> String {
    let mut fixture = RetainedFixture::new(4);
    let slots = fixture.append_range(1, 6);
    let mut output = unsafe { fixture.row("smaller_before", &slots) };
    unsafe { fixture.copy_parameters(2, false, false) };
    output.push_str(&unsafe { fixture.row("smaller_forgotten", &slots) });
    unsafe { destroy_forgotten(&slots) };
    output.push_str(&unsafe { fixture.row("smaller_destroyed_old", &[]) });
    let reused = fixture.append_range(11, 5);
    assert_eq!(reused[0], slots[0]);
    output.push_str(&unsafe { fixture.row("smaller_reused", &reused) });
    fixture.target.clear(true);
    output.push_str(&unsafe { fixture.row("smaller_cleared", &[]) });
    output
}
fn regrow_rows() -> String {
    let mut fixture = RetainedFixture::new(4);
    let slots = fixture.append_range(1, 3);
    unsafe { fixture.copy_parameters(2, true, true) };
    let mut output = unsafe { fixture.row("regrow_shrunk", &slots) };
    unsafe { fixture.copy_parameters(4, true, true) };
    output.push_str(&unsafe { fixture.row("regrow_expanded", &slots) });
    unsafe { restore_fixture(&mut fixture, 3) };
    assert_eq!(retained_values(&fixture.target), [1, 2, 3]);
    output.push_str(&unsafe { fixture.row("regrow_restored", &slots) });
    fixture.target.clear(true);
    output.push_str(&unsafe { fixture.row("regrow_cleared", &[]) });
    output
}
fn empty_rows() -> String {
    let mut fixture = RetainedFixture::new(4);
    fixture
        .target
        .storage_extension()
        .ensure_block_count(3, size_of::<ParameterItem>())
        .unwrap();
    unsafe { fixture.copy_parameters(2, true, true) };
    let mut output = unsafe { fixture.row("empty_retained_copied", &[]) };
    let reused = fixture.append_range(21, 2);
    output.push_str(&unsafe { fixture.row("empty_retained_reused", &reused) });
    fixture.target.clear(true);
    output.push_str(&unsafe { fixture.row("empty_retained_cleared", &[]) });
    output
}
struct OwnerFixture {
    target: DynamicArray<i32>,
    owner: Option<BaseAllocator>,
    first: *mut i32,
    second: *mut i32,
}
fn owner_target(old: BaseAllocator) -> DynamicArray<i32> {
    let mut target = DynamicArray::with_allocator(2, Some(old));
    for value in 1..=3 {
        target.append(value);
    }
    target
}
impl OwnerFixture {
    fn new(old: BaseAllocator, owner: Option<BaseAllocator>) -> Self {
        let mut target = owner_target(old);
        let first = target.storage_extension().change_block_at(0).unwrap();
        let second = target.storage_extension().change_block_at(1).unwrap();
        let source = DynamicArray::with_allocator(2, owner.clone());
        // Primitive slots remain live and are republished before cleanup.
        unsafe {
            target
                .storage_extension()
                .copy_parameters_retained(&source, true)
                .unwrap()
        };
        Self {
            target,
            owner,
            first,
            second,
        }
    }
    fn row(&mut self, name: &str, values: &str) -> String {
        let count = self.target.storage_extension().block_count();
        let stable =
            count != 0 && self.target.storage_extension().change_block_at(0).unwrap() == self.first;
        let owner = self.target.allocator() == self.owner.as_ref();
        format!(
            "{name} {} {count} {} {} {} {values}\n",
            self.target.size(),
            self.target.storage_extension().available_size(),
            u8::from(owner),
            u8::from(stable)
        )
    }
    fn forgotten_values(&self) -> String {
        // Constructor retained all three initialized primitive slots, even though Size is zero.
        unsafe { joined_values([*self.first, *self.first.add(1), *self.second].into_iter()) }
    }
    fn restored_row(&mut self, name: &str) -> String {
        unsafe { self.target.storage_extension().set_used_size(3).unwrap() };
        let values = joined_values(self.target.iter().copied());
        self.row(&format!("{name}_restored"), &values)
    }
    fn cleared_row(&mut self, name: &str) -> String {
        self.target.clear(true);
        self.row(&format!("{name}_cleared"), "-")
    }
}
fn owner_rows(name: &str, old: BaseAllocator, new: Option<BaseAllocator>) -> String {
    let mut fixture = OwnerFixture::new(old, new);
    let values = fixture.forgotten_values();
    let mut output = fixture.row(&format!("{name}_forgotten"), &values);
    output.push_str(&fixture.restored_row(name));
    output.push_str(&fixture.cleared_row(name));
    output
}
fn compatible_owner_rows() -> String {
    let mut output = owner_rows(
        "common_to_heap",
        BaseAllocator::fresh_common(),
        Some(BaseAllocator::fresh_heap()),
    );
    output.push_str(&owner_rows(
        "heap_to_common",
        BaseAllocator::fresh_heap(),
        Some(BaseAllocator::fresh_common()),
    ));
    output.push_str(&owner_rows(
        "heap_to_null",
        BaseAllocator::fresh_heap(),
        None,
    ));
    output
}
#[test]
fn native_retained_parameter_rows_preserve_bytes_lifetimes_and_compatible_owners() {
    let mut actual = equal_rows();
    actual.push_str(&smaller_rows());
    actual.push_str(&regrow_rows());
    actual.push_str(&empty_rows());
    actual.push_str(&compatible_owner_rows());
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkernel_dynamic_retained_parameters.expected")
    );
}
