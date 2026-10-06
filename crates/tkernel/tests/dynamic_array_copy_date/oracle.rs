use super::super::*;
use std::fmt::Write;
trait ObservedValue {
    fn observed_value(&self) -> i32;
}
impl ObservedValue for CopyDateItem {
    fn observed_value(&self) -> i32 {
        self.value
    }
}
impl ObservedValue for i32 {
    fn observed_value(&self) -> i32 {
        *self
    }
}
struct AllocationRows {
    allocator: BaseAllocator,
    live: usize,
    allocations: usize,
    frees: usize,
}
impl AllocationRows {
    fn new(allocator: BaseAllocator) -> Self {
        Self {
            allocator,
            live: 0,
            allocations: 0,
            frees: 0,
        }
    }
    fn snapshot(&mut self) -> [usize; 2] {
        let live = live_allocations(&self.allocator);
        self.allocations += live.saturating_sub(self.live);
        self.frees += self.live.saturating_sub(live);
        self.live = live;
        [self.allocations, self.frees]
    }
}
fn values<'a, T: ObservedValue + 'a>(items: impl Iterator<Item = &'a T>) -> String {
    let joined = items
        .map(|item| item.observed_value().to_string())
        .collect::<Vec<_>>()
        .join(",");
    if joined.is_empty() {
        "-".to_owned()
    } else {
        joined
    }
}
struct CopyCase<T> {
    target: DynamicArray<T>,
    observations: Rc<CopyObservations>,
    allocations: AllocationRows,
}
impl<T: ObservedValue> CopyCase<T> {
    fn new(observations: Rc<CopyObservations>) -> Self {
        let allocator = BaseAllocator::accumulating(4096).unwrap();
        Self {
            target: DynamicArray::with_allocator(2, Some(allocator.clone())),
            observations,
            allocations: AllocationRows::new(allocator),
        }
    }
    fn counters(&mut self, partial: usize) -> [usize; 7] {
        let [allocated, freed] = self.allocations.snapshot();
        [
            self.target.size(),
            self.target.storage_extension().block_count(),
            allocated,
            freed,
            self.observations.attempts.get(),
            self.observations.drops.get(),
            partial,
        ]
    }
    fn row(&mut self, name: &str, partial: &[CopyDateItem]) -> String {
        let mut output = name.to_owned();
        for count in self.counters(partial.len()) {
            write!(output, " {count}").unwrap();
        }
        writeln!(
            output,
            " {} {}",
            values(self.target.iter()),
            values(partial.iter())
        )
        .unwrap();
        output
    }
    fn cleared_row(&mut self, name: &str) -> String {
        self.target.clear(true);
        self.row(&format!("{name}_cleared"), &[])
    }
}
fn recover_row(
    name: &str,
    fixture: &mut CopyCase<CopyDateItem>,
    error: DynamicArrayCopyFailure<CopyDateItem>,
) -> String {
    assert!(error.has_incomplete_block());
    assert_eq!(error.failure().message(), "copy failure");
    let mut output = fixture.row(&format!("{name}_failed"), error.incomplete_values());
    let failure = error.into_failure();
    assert_eq!(failure.message(), "copy failure");
    output.push_str(&fixture.row(&format!("{name}_recovered"), &[]));
    output
}
fn nontrivial_rows(name: &str, fail_at: usize) -> String {
    let (source, observations) = source_items(3);
    let mut fixture = CopyCase::new(observations);
    let result = fixture
        .target
        .storage_extension()
        .copy_date_with(&source, |item| item.copy_construct(fail_at));
    let mut output = match result {
        Ok(()) => fixture.row(&format!("{name}_copied"), &[]),
        Err(error) => recover_row(name, &mut fixture, error),
    };
    output.push_str(&fixture.cleared_row(name));
    assert_eq!(
        source.iter().map(|item| item.value).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    output
}
fn trivial_source(used: i32, blocks: usize) -> DynamicArray<i32> {
    let mut source = DynamicArray::with_increment(2);
    for value in 1..=used {
        source.append(value);
    }
    source
        .storage_extension()
        .ensure_block_count(blocks, size_of::<i32>())
        .unwrap();
    source
}
fn trivial_rows(name: &str, used: i32, blocks: usize) -> String {
    let source = trivial_source(used, blocks);
    let mut fixture = CopyCase::new(Rc::new(CopyObservations::default()));
    fixture
        .target
        .storage_extension()
        .copy_date_trivial(&source)
        .unwrap();
    let mut output = fixture.row(&format!("{name}_copied"), &[]);
    output.push_str(&fixture.cleared_row(name));
    output
}
#[test]
fn native_copy_date_rows_join_allocations_publication_and_recovery() {
    let mut actual = String::new();
    for (name, fail_at) in [
        ("constructor", 0),
        ("first_failure", 1),
        ("partial_failure", 2),
        ("second_block_failure", 3),
    ] {
        actual.push_str(&nontrivial_rows(name, fail_at));
    }
    actual.push_str(&trivial_rows("trivial", 3, 3));
    actual.push_str(&trivial_rows("empty_retained", 0, 3));
    actual.push_str(&trivial_rows("empty", 0, 0));
    assert_eq!(
        actual,
        include_str!("../../../../tools/oracle/tkernel_dynamic_copy_date.expected")
    );
}
