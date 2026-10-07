use rustycad_tkernel::osd::{ParallelIteratorInterface, ParallelUniversalIterator};
use std::any::Any;
use std::cell::{Cell, RefCell};

pub struct BorrowedPayload<'a> {
    pub values: &'a [i32],
}

#[derive(Default)]
pub struct Observations {
    pub trace: RefCell<String>,
    pub fail_clone: Cell<bool>,
    pub null_clone: Cell<bool>,
}

struct BorrowedCursor<'a> {
    payload: &'a BorrowedPayload<'a>,
    position: usize,
    observations: &'a Observations,
}

impl Drop for BorrowedCursor<'_> {
    fn drop(&mut self) {
        self.observations.trace.borrow_mut().push('d');
    }
}

impl<'a> ParallelIteratorInterface<'a> for BorrowedCursor<'a> {
    fn is_equal(&self, other: &dyn ParallelIteratorInterface<'a>) -> bool {
        self.observations.trace.borrow_mut().push('e');
        other
            .comparison_value()
            .and_then(|value| value.downcast_ref::<usize>())
            == Some(&self.position)
    }

    fn increment(&mut self) {
        self.observations.trace.borrow_mut().push('i');
        assert!(self.position < self.payload.values.len());
        self.position += 1;
    }

    fn clone_box(&self) -> Option<Box<dyn ParallelIteratorInterface<'a> + 'a>> {
        self.observations.trace.borrow_mut().push('c');
        assert!(!self.observations.fail_clone.get(), "fixture clone failure");
        if self.observations.null_clone.get() {
            return None;
        }
        Some(Box::new(Self {
            payload: self.payload,
            position: self.position,
            observations: self.observations,
        }))
    }

    fn comparison_value(&self) -> Option<&dyn Any> {
        Some(&self.position)
    }
}

pub fn cursor<'a>(
    payload: &'a BorrowedPayload<'a>,
    position: usize,
    observations: &'a Observations,
) -> ParallelUniversalIterator<'a> {
    ParallelUniversalIterator::from_interface(Some(Box::new(BorrowedCursor {
        payload,
        position,
        observations,
    })))
}

pub fn position(iterator: &ParallelUniversalIterator<'_>) -> usize {
    *iterator
        .value()
        .unwrap()
        .comparison_value()
        .unwrap()
        .downcast_ref::<usize>()
        .unwrap()
}
