//! Native CellFilter target construction order, default setup, and enum representation.
use rustycad_tkernel::ncollection::{BaseAllocator, CellAction, CellFilter, CellInspector, List};
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Debug)]
struct Target {
    number: i32,
    arena: BaseAllocator,
    copy_allocations: Rc<RefCell<Vec<usize>>>,
}
impl Clone for Target {
    fn clone(&self) -> Self {
        let failure = self.arena.reset(false).unwrap_err();
        let count = failure
            .message()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        self.copy_allocations.borrow_mut().push(count);
        Self {
            number: self.number,
            arena: self.arena.clone(),
            copy_allocations: Rc::clone(&self.copy_allocations),
        }
    }
}
#[derive(Default)]
struct Inspector {
    visited: Vec<i32>,
}
impl CellInspector for Inspector {
    type Target = Target;
    type Point = [f64; 1];
    const DIMENSION: usize = 1;
    fn coord(axis: usize, point: &Self::Point) -> f64 {
        point[axis]
    }
    fn is_equal(first: &Target, second: &Target) -> bool {
        first.number == second.number
    }
    fn inspect(&mut self, target: &Target) -> CellAction {
        self.visited.push(target.number);
        CellAction::Keep
    }
}
#[test]
fn target_copy_observes_preallocated_native_list_node() {
    let arena = BaseAllocator::incremental(1024);
    let copied = Rc::new(RefCell::new(Vec::new()));
    let mut target = Target {
        number: 7,
        arena: arena.clone(),
        copy_allocations: Rc::clone(&copied),
    };
    let mut filter = CellFilter::<Inspector>::with_allocator(1.0, Some(arena));
    filter.add(&target, &[0.0]);
    target.number = 8;
    filter.add(&target, &[0.0]);
    assert_eq!(&*copied.borrow(), &[2, 3]);
    let mut inspector = Inspector::default();
    filter.inspect(&[0.0], &mut inspector);
    assert_eq!(inspector.visited, [8, 7]);
}
#[test]
fn prepend_constructor_sees_allocation_and_failure_keeps_previous_chain() {
    let arena = BaseAllocator::incremental(1024);
    let mut list = List::with_allocator(arena.clone());
    assert_eq!(
        list.prepend_with(|| {
            assert!(
                arena
                    .reset(false)
                    .unwrap_err()
                    .message()
                    .starts_with("1 live")
            );
            42
        }),
        &42
    );
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        list.prepend_with(|| {
            assert!(
                arena
                    .reset(false)
                    .unwrap_err()
                    .message()
                    .starts_with("2 live")
            );
            panic!("injected target constructor failure");
        });
    }));
    assert!(failed.is_err());
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [42]);
    assert!(
        arena
            .reset(false)
            .unwrap_err()
            .message()
            .starts_with("1 live")
    );
}
#[test]
fn action_abi_matches_native_integer_enumeration() {
    assert_eq!(CellAction::Keep as i32, 0);
    assert_eq!(CellAction::Purge as i32, 1);
    assert_eq!(
        std::mem::size_of::<CellAction>(),
        std::mem::size_of::<i32>()
    );
    assert_eq!(
        std::mem::align_of::<CellAction>(),
        std::mem::align_of::<i32>()
    );
}
#[test]
fn default_constructor_can_be_initialized_before_use() {
    let mut filter = CellFilter::<Inspector>::default();
    let arena = BaseAllocator::incremental(1024);
    filter.reset(1.0, Some(arena.clone())).unwrap();
    let target = Target {
        number: 7,
        arena,
        copy_allocations: Rc::new(RefCell::new(Vec::new())),
    };
    filter.add(&target, &[0.0]);
    let mut inspector = Inspector::default();
    filter.inspect(&[0.0], &mut inspector);
    assert_eq!(inspector.visited, [7]);
}
