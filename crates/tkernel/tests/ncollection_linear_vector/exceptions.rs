use rustycad_tkernel::ncollection::LinearVector;
use std::cell::Cell;
use std::rc::Rc;

struct PanickingDefault {
    value: i32,
}

impl Default for PanickingDefault {
    fn default() -> Self {
        panic!("default construction for vector growth failed");
    }
}

struct CopyControl {
    attempts: Cell<usize>,
    dropped: Cell<usize>,
}

fn copy_control() -> Rc<CopyControl> {
    Rc::new(CopyControl {
        attempts: Cell::new(0),
        dropped: Cell::new(0),
    })
}

fn assert_panics(action: impl FnOnce()) {
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(action));
    assert!(failure.is_err());
}

struct PanickingClone {
    value: i32,
    control: Rc<CopyControl>,
}

impl Clone for PanickingClone {
    fn clone(&self) -> Self {
        let attempt = self.control.attempts.get() + 1;
        self.control.attempts.set(attempt);
        assert_ne!(
            attempt, 2,
            "injected tail copy failure at attempt {attempt}"
        );
        Self {
            value: self.value,
            control: Rc::clone(&self.control),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
    }
}

impl Drop for PanickingClone {
    fn drop(&mut self) {
        self.control.dropped.set(self.control.dropped.get() + 1);
    }
}

fn tracked_vector(values: &[i32], control: &Rc<CopyControl>) -> LinearVector<PanickingClone> {
    let mut vector = LinearVector::with_capacity(5);
    for &value in values {
        vector.append(PanickingClone {
            value,
            control: Rc::clone(control),
        });
    }
    vector
}

#[test]
fn resize_restores_logical_length_after_default_panics() {
    let mut vector = LinearVector::with_capacity(4);
    vector.append(PanickingDefault { value: 42 });
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| vector.resize(4)));

    assert!(failure.is_err());
    assert_eq!(vector.len(), 1);
    assert_eq!(vector[0].value, 42);
}

#[test]
fn resize_with_value_drops_completed_tail_after_clone_panics() {
    let control = copy_control();
    let mut vector = tracked_vector(&[42], &control);
    let fill = PanickingClone {
        value: 7,
        control: Rc::clone(&control),
    };
    assert_panics(|| {
        vector.resize_with_value(4, fill);
    });

    assert_eq!(vector.len(), 1);
    assert_eq!(vector[0].value, 42);
    assert_eq!(control.dropped.get(), 2);
}

#[test]
fn assign_keeps_size_but_retains_completed_common_item_assignment() {
    let control = copy_control();
    let source = tracked_vector(&[10, 20, 30], &control);
    let mut destination = tracked_vector(&[1], &control);
    assert_panics(|| {
        destination.assign(&source);
    });

    assert_eq!(destination.len(), 1);
    assert_eq!(destination[0].value, 10);
    assert_eq!(control.dropped.get(), 1);
}

struct RelocationProbe {
    value: i32,
    clones: Rc<Cell<usize>>,
}

impl Clone for RelocationProbe {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        Self {
            value: self.value,
            clones: Rc::clone(&self.clones),
        }
    }
}

#[test]
fn reserve_relocates_without_native_move_if_noexcept_copy_dispatch() {
    let clones = Rc::new(Cell::new(0));
    let mut vector = LinearVector::with_capacity(1);
    vector.append(RelocationProbe {
        value: 42,
        clones: Rc::clone(&clones),
    });

    vector.reserve(2);

    assert!(vector.capacity() >= 2);
    assert_eq!(vector[0].value, 42);
    assert_eq!(clones.get(), 0);
}
