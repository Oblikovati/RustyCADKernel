use rustycad_tkernel::ncollection::LinearVector;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
struct CopyControl {
    copies: Cell<usize>,
    assignments: Cell<usize>,
    drops: Cell<usize>,
    panic_at_copy: Cell<usize>,
}

struct CopyProbe {
    value: i32,
    control: Rc<CopyControl>,
}

impl Clone for CopyProbe {
    fn clone(&self) -> Self {
        let copy = self.control.copies.get() + 1;
        self.control.copies.set(copy);
        assert_ne!(copy, self.control.panic_at_copy.get(), "copy {copy} failed");
        Self {
            value: self.value,
            control: Rc::clone(&self.control),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.control
            .assignments
            .set(self.control.assignments.get() + 1);
        self.value = source.value;
    }
}

impl Drop for CopyProbe {
    fn drop(&mut self) {
        self.control.drops.set(self.control.drops.get() + 1);
    }
}

fn vector_with_values(
    capacity: usize,
    values: &[i32],
    control: &Rc<CopyControl>,
) -> LinearVector<CopyProbe> {
    let mut vector = LinearVector::with_capacity(capacity);
    for value in values {
        vector.append(CopyProbe {
            value: *value,
            control: Rc::clone(control),
        });
    }
    vector
}

fn assert_failed_tail_state(
    destination: &LinearVector<CopyProbe>,
    control: &CopyControl,
    failure: std::thread::Result<()>,
) {
    assert!(failure.is_err());
    assert_eq!((destination.capacity(), destination.len()), (4, 1));
    assert_eq!(destination[0].value, 10);
    assert_eq!((control.copies.get(), control.assignments.get()), (2, 1));
    assert_eq!(control.drops.get(), 1);
}

#[test]
fn tail_copy_panic_keeps_common_assignment_and_drops_completed_tail() {
    let control = Rc::new(CopyControl::default());
    control.panic_at_copy.set(2);
    let source = vector_with_values(3, &[10, 20, 30], &control);
    let mut destination = vector_with_values(4, &[1], &control);
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        destination.assign(&source);
    }));
    assert_failed_tail_state(&destination, &control, failure);
    drop(destination);
    drop(source);
    assert_eq!(control.drops.get(), 5);
}
