//! Native custom-interface dispatch, without requiring Clone or Any on borrowed payloads.
use rustycad_tkernel::osd::{Parallel, ParallelIteratorInterface, destroy_parallel_iterator};
use std::any::Any;
use std::cell::Cell;
use std::sync::atomic::{AtomicI32, Ordering};

struct BorrowedPayload<'a> {
    values: &'a [i32],
}

struct BorrowedCursor<'a> {
    payload: &'a BorrowedPayload<'a>,
    position: usize,
    destroyed: &'a Cell<usize>,
}

impl<'a> BorrowedCursor<'a> {
    fn start(payload: &'a BorrowedPayload<'a>, destroyed: &'a Cell<usize>) -> Self {
        Self {
            payload,
            position: 0,
            destroyed,
        }
    }
}

impl Drop for BorrowedCursor<'_> {
    fn drop(&mut self) {
        self.destroyed.set(self.destroyed.get() + 1);
    }
}

impl<'a> ParallelIteratorInterface<'a> for BorrowedCursor<'a> {
    fn is_equal(&self, other: &dyn ParallelIteratorInterface<'a>) -> bool {
        other
            .comparison_value()
            .and_then(|value| value.downcast_ref::<usize>())
            == Some(&self.position)
    }

    fn increment(&mut self) {
        assert!(self.position < self.payload.values.len());
        self.position += 1;
    }

    fn clone_box(&self) -> Option<Box<dyn ParallelIteratorInterface<'a> + 'a>> {
        Some(Box::new(Self {
            payload: self.payload,
            position: self.position,
            destroyed: self.destroyed,
        }))
    }

    fn comparison_value(&self) -> Option<&dyn Any> {
        Some(&self.position)
    }
}

struct OpaqueCursor;

impl<'a> ParallelIteratorInterface<'a> for OpaqueCursor {
    fn is_equal(&self, _: &dyn ParallelIteratorInterface<'a>) -> bool {
        false
    }
    fn increment(&mut self) {}
    fn clone_box(&self) -> Option<Box<dyn ParallelIteratorInterface<'a> + 'a>> {
        Some(Box::new(Self))
    }
}

struct ForeignCursor(i32);

impl<'a> ParallelIteratorInterface<'a> for ForeignCursor {
    fn is_equal(&self, _: &dyn ParallelIteratorInterface<'a>) -> bool {
        false
    }
    fn increment(&mut self) {
        self.0 += 1;
    }
    fn clone_box(&self) -> Option<Box<dyn ParallelIteratorInterface<'a> + 'a>> {
        Some(Box::new(Self(self.0)))
    }
    fn comparison_value(&self) -> Option<&dyn Any> {
        Some(&self.0)
    }
}

#[test]
fn virtual_clone_outlives_source_wrapper_and_preserves_borrowed_payload() {
    let values = [17, 29, 41];
    let payload = BorrowedPayload { values: &values };
    let destroyed = Cell::new(0);
    let mut source = BorrowedCursor::start(&payload, &destroyed);
    let mut copy = source.clone_box().unwrap();
    assert!(source.is_equal(copy.as_ref()));
    source.increment();
    assert!(!source.is_equal(copy.as_ref()));
    drop(source);
    assert_eq!(destroyed.get(), 1);
    copy.increment();
    assert_eq!(
        copy.comparison_value().unwrap().downcast_ref::<usize>(),
        Some(&1)
    );
    destroy_parallel_iterator(copy);
    assert_eq!(destroyed.get(), 2);
}

#[test]
fn optional_and_foreign_comparison_states_are_not_equal() {
    let payload = BorrowedPayload { values: &[11] };
    let destroyed = Cell::new(0);
    let cursor = BorrowedCursor::start(&payload, &destroyed);
    assert!(!cursor.is_equal(&OpaqueCursor));
    assert!(!cursor.is_equal(&ForeignCursor(0)));
    assert!(OpaqueCursor.comparison_value().is_none());
    let mut foreign = ForeignCursor(0).clone_box().unwrap();
    foreign.increment();
    assert!(!foreign.is_equal(&cursor));
    assert!(!OpaqueCursor.clone_box().unwrap().is_equal(&cursor));
}

#[test]
fn parallel_for_each_still_accepts_borrowed_non_clone_payloads() {
    let values = [3, 5, 7];
    let payloads = values
        .chunks(2)
        .map(|values| BorrowedPayload { values })
        .collect::<Vec<_>>();
    let sum = AtomicI32::new(0);
    Parallel::for_each(
        payloads.iter(),
        |item| {
            sum.fetch_add(item.values.iter().sum::<i32>(), Ordering::Relaxed);
        },
        true,
        2,
    )
    .unwrap();
    assert_eq!(sum.load(Ordering::Relaxed), 15);
}
