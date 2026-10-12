//! Resource replacement in the real payload destructor must precede the current Free call.
#![allow(unsafe_code)]
use crate::ncollection::{
    AllocatorDispatchCode, AllocatorInvocation, AllocatorObject, AllocatorSource, BaseAllocator,
    Sequence,
};
use crate::standard::{Failure, Handle, SharedState, Transient};
use std::sync::Arc;

#[derive(Default)]
struct RetirementFreeRecorder {
    parent: AllocatorObject,
    free_counts: SharedState<Vec<i32>>,
}
crate::impl_transient!(RetirementFreeRecorder, "Martensite_RetirementFreeRecorder",
    AllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: both recorders allocate and free through the same Standard heap implementation.
unsafe impl AllocatorSource for RetirementFreeRecorder {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Counts and allocation provenance belong to this genuine embedded native base.
        &self.parent
    }
    fn dispatch_code(&self) -> AllocatorDispatchCode {
        AllocatorDispatchCode {
            free: retirement_recorder_free,
            ..Default::default()
        }
    }
}
struct RetirementResourceReplacement {
    selected: Arc<SharedState<BaseAllocator>>,
    replacement: BaseAllocator,
    retired: Arc<SharedState<bool>>,
    panic_after_replacement: bool,
}
impl Drop for RetirementResourceReplacement {
    fn drop(&mut self) {
        *self.selected.write() = self.replacement.storage_lease();
        *self.retired.write() = true;
        assert!(
            !self.panic_after_replacement,
            "retirement fixture requested payload failure"
        );
    }
}
struct RetirementCase {
    old_owner: Handle<RetirementFreeRecorder>,
    new_owner: Handle<RetirementFreeRecorder>,
    old: BaseAllocator,
    current: BaseAllocator,
    selected: Arc<SharedState<BaseAllocator>>,
    retired: Arc<SharedState<bool>>,
}
impl RetirementCase {
    fn new() -> Self {
        let old_owner = Handle::new(RetirementFreeRecorder::default());
        let new_owner = Handle::new(RetirementFreeRecorder::default());
        let old = BaseAllocator::from_handle(&old_owner);
        let current = BaseAllocator::from_handle(&new_owner);
        let selected = Arc::new(SharedState::new(old.storage_lease()));
        Self {
            old_owner,
            new_owner,
            old,
            current,
            selected,
            retired: Arc::new(SharedState::new(false)),
        }
    }
    fn detached(
        &self,
        panic_after_replacement: bool,
    ) -> super::super::SequenceNodeRetirement<RetirementResourceReplacement> {
        let mut sequence = Sequence::with_allocator(self.old.storage_lease());
        sequence.append(RetirementResourceReplacement {
            selected: self.selected.clone(),
            replacement: self.current.storage_lease(),
            retired: self.retired.clone(),
            panic_after_replacement,
        });
        let mut cursor = sequence.cursor();
        sequence.detach_cursor(&mut cursor).unwrap().unwrap()
    }
    fn retire(&self, node: super::super::SequenceNodeRetirement<RetirementResourceReplacement>) {
        // SAFETY: both actual resources accept the same Standard heap allocation.
        unsafe {
            node.retire_using(|| {
                assert!(*self.retired.read());
                self.selected.read().storage_lease()
            })
        };
    }
    fn check(&self, expected_panic: bool) {
        let node = self.detached(expected_panic);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.retire(node)));
        assert_eq!(result.is_err(), expected_panic);
        assert!(self.old_owner.free_counts.read().is_empty());
        assert_eq!(&*self.new_owner.free_counts.read(), &[2]);
    }
}
fn execute_current_resource_case(panic_after_replacement: bool) {
    let fixture = RetirementCase::new();
    fixture.check(panic_after_replacement);
}
#[test]
fn payload_replacement_selects_actual_current_free_without_per_node_ownership() {
    // Both allocator objects remain anchored while native-compatible callbacks execute.
    execute_current_resource_case(false);
}
#[test]
fn rust_payload_unwind_keeps_current_resource_allocation_cleanup() {
    // Rust's existing node cleanup remains defined; no C++ throwing-destructor parity claim.
    execute_current_resource_case(true);
}

fn record_retirement_free(source: &RetirementFreeRecorder) {
    source
        .free_counts
        .write()
        .push(source.transient_base().ref_count());
}
unsafe fn retirement_recorder_free(
    invocation: &AllocatorInvocation,
    pointer: *mut u8,
) -> Result<(), Failure> {
    // SAFETY: plain trace values cannot cause callbacks or resource retirement.
    crate::standard::raise_native_failure(unsafe { invocation.capture(record_retirement_free) });
    // SAFETY: fixture passes the original compatible Standard allocation pointer.
    unsafe { invocation.free_base(pointer) }
}
