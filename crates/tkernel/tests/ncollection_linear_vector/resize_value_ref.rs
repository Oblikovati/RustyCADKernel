use rustycad_tkernel::ncollection::LinearVector;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
struct CloneControl {
    attempts: Cell<usize>,
    drops: Cell<usize>,
    panic_at: Cell<usize>,
}

struct CloneProbe {
    value: i32,
    control: Rc<CloneControl>,
}

impl Clone for CloneProbe {
    fn clone(&self) -> Self {
        let attempt = self.control.attempts.get() + 1;
        self.control.attempts.set(attempt);
        assert_ne!(
            attempt,
            self.control.panic_at.get(),
            "clone {attempt} failed"
        );
        Self {
            value: self.value,
            control: Rc::clone(&self.control),
        }
    }
}

impl Drop for CloneProbe {
    fn drop(&mut self) {
        self.control.drops.set(self.control.drops.get() + 1);
    }
}

fn make_vector(
    capacity: usize,
    values: &[i32],
    control: &Rc<CloneControl>,
) -> LinearVector<CloneProbe> {
    let mut vector = LinearVector::with_capacity(capacity);
    for value in values {
        vector.append(CloneProbe {
            value: *value,
            control: Rc::clone(control),
        });
    }
    vector
}

fn assert_values(vector: &LinearVector<CloneProbe>, values: &[i32]) {
    assert_eq!(
        vector.iter().map(|item| item.value).collect::<Vec<_>>(),
        values
    );
}

#[test]
fn borrowed_fill_clones_each_new_slot_and_shrink_drops_only_removed_items() {
    let control = Rc::new(CloneControl::default());
    let mut vector = make_vector(4, &[1], &control);
    let fill = CloneProbe {
        value: 9,
        control: Rc::clone(&control),
    };
    vector.resize_with_value_ref(3, &fill);
    assert_eq!(vector.capacity(), 4);
    assert_eq!(control.attempts.get(), 2);
    assert_eq!(control.drops.get(), 0);
    assert_values(&vector, &[1, 9, 9]);
    vector.resize_with_value_ref(3, &fill);
    assert_eq!(control.attempts.get(), 2);
    vector.resize_with_value_ref(1, &fill);
    assert_eq!(control.attempts.get(), 2);
    assert_eq!(control.drops.get(), 2);
    assert_values(&vector, &[1]);
}

#[test]
fn borrowed_fill_reallocation_moves_existing_values_then_clones_each_tail() {
    let control = Rc::new(CloneControl::default());
    let mut vector = make_vector(1, &[1], &control);
    let fill = CloneProbe {
        value: 9,
        control: Rc::clone(&control),
    };
    vector.resize_with_value_ref(3, &fill);
    assert!(vector.capacity() >= 3);
    assert_eq!(control.attempts.get(), 2);
    assert_values(&vector, &[1, 9, 9]);
}

#[test]
fn borrowed_fill_clone_panic_rolls_back_and_drops_completed_tail() {
    let control = Rc::new(CloneControl::default());
    control.panic_at.set(2);
    let mut vector = make_vector(4, &[1], &control);
    let fill = CloneProbe {
        value: 9,
        control: Rc::clone(&control),
    };
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vector.resize_with_value_ref(4, &fill);
    }));
    assert!(failure.is_err());
    assert_eq!(vector.len(), 1);
    assert_eq!(vector.capacity(), 4);
    assert_eq!(control.attempts.get(), 2);
    assert_eq!(control.drops.get(), 1);
    assert_values(&vector, &[1]);
}

#[test]
fn owned_fill_wrapper_clones_new_slots_through_the_borrowed_path() {
    let control = Rc::new(CloneControl::default());
    let mut vector = make_vector(4, &[1], &control);
    vector.resize_with_value(
        3,
        CloneProbe {
            value: 9,
            control: Rc::clone(&control),
        },
    );
    assert_eq!(control.attempts.get(), 2);
    assert_values(&vector, &[1, 9, 9]);
}

#[test]
fn borrowed_fill_constructor_keeps_native_empty_and_growth_capacities() {
    let control = Rc::new(CloneControl::default());
    let fill = CloneProbe {
        value: 9,
        control: Rc::clone(&control),
    };
    let empty = LinearVector::with_value_ref(0, &fill);
    assert_eq!(empty.capacity(), 0);
    assert_eq!(control.attempts.get(), 0);
    let one = LinearVector::with_value_ref(1, &fill);
    assert_eq!(one.capacity(), 2);
    let many = LinearVector::with_value_ref(3, &fill);
    assert_eq!(many.capacity(), 3);
    assert_eq!(control.attempts.get(), 4);
    assert_values(&one, &[9]);
    assert_values(&many, &[9, 9, 9]);
}

#[test]
fn borrowed_fill_constructor_panic_drops_completed_rust_tail() {
    let control = Rc::new(CloneControl::default());
    control.panic_at.set(2);
    let fill = CloneProbe {
        value: 9,
        control: Rc::clone(&control),
    };
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        LinearVector::with_value_ref(4, &fill)
    }));
    assert!(failure.is_err());
    assert_eq!(control.attempts.get(), 2);
    assert_eq!(control.drops.get(), 1);
    drop(fill);
    assert_eq!(control.drops.get(), 2);
}

#[test]
fn owned_fill_constructor_uses_the_borrowed_native_capacity_path() {
    let control = Rc::new(CloneControl::default());
    let vector = LinearVector::with_value(
        1,
        CloneProbe {
            value: 9,
            control: Rc::clone(&control),
        },
    );
    assert_eq!(vector.capacity(), 2);
    assert_eq!(control.attempts.get(), 1);
    assert_values(&vector, &[9]);
}

#[test]
fn borrowed_fill_constructor_drops_every_successful_copy() {
    let control = Rc::new(CloneControl::default());
    let fill = CloneProbe {
        value: 9,
        control: Rc::clone(&control),
    };
    let vector = LinearVector::with_value_ref(3, &fill);
    assert_eq!(control.attempts.get(), 3);
    assert_eq!(control.drops.get(), 0);
    drop(vector);
    assert_eq!(control.drops.get(), 3);
    drop(fill);
    assert_eq!(control.drops.get(), 4);
}
