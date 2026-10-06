//! Ownership indexing must not replace canonical native link or destruction order.
use rustycad_tkernel::ncollection::List;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
struct RegistryDropTrace {
    number: usize,
    trace: Rc<RefCell<Vec<usize>>>,
}
impl Drop for RegistryDropTrace {
    fn drop(&mut self) {
        self.trace.borrow_mut().push(self.number);
    }
}
fn traced_list(
    range: std::ops::Range<usize>,
    trace: &Rc<RefCell<Vec<usize>>>,
) -> List<RegistryDropTrace> {
    range
        .map(|number| RegistryDropTrace {
            number,
            trace: trace.clone(),
        })
        .collect()
}
#[test]
fn owner_registry_growth_and_splice_keep_native_reversed_clear_order() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut first = traced_list(0..64, &trace);
    let mut second = traced_list(64..128, &trace);
    first.append_list(&mut second);
    assert!(second.is_empty());
    first.reverse();
    first.clear();
    assert_eq!(*trace.borrow(), (0..128).rev().collect::<Vec<_>>());
    assert!(first.is_empty());
}
#[test]
fn owner_registry_repeated_head_removal_preserves_remaining_chain() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut list = traced_list(0..128, &trace);
    for number in 0..128 {
        assert_eq!(list.first().unwrap().number, number);
        assert_eq!(list.len(), 128 - number);
        list.remove_first().unwrap();
        assert_eq!(trace.borrow().last(), Some(&number));
    }
    assert_eq!(*trace.borrow(), (0..128).collect::<Vec<_>>());
    assert!(list.is_empty());
}

#[test]
fn indexed_traversal_keeps_visit_order_and_search_miss_state() {
    let mut list: List<usize> = (0..128).collect();
    let mut visited = Vec::new();
    list.retain(|number| {
        visited.push(*number);
        true
    });
    assert_eq!(visited, (0..128).collect::<Vec<_>>());
    assert!(!list.remove_value(&128));
    assert!(list.remove_value(&127));
    assert_eq!(
        list.iter().copied().collect::<Vec<_>>(),
        (0..127).collect::<Vec<_>>()
    );
}
#[test]
fn indexed_removal_repairs_freed_remembered_predecessor() {
    let mut list: List<i32> = [1, 2, 3, 4].into_iter().collect();
    let mut removed_previous = list.cursor();
    removed_previous.next(&list);
    let mut cursor = removed_previous;
    cursor.next(&list);
    let remembered = cursor.previous_node();
    list.remove(&mut removed_previous).unwrap();
    list.remove(&mut cursor).unwrap();
    assert_eq!(cursor.previous_node(), remembered);
    assert_eq!(cursor.value(&list), Some(&4));
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [1, 4]);
}
