//! Shared mutation of transients: a change through one handle is seen through all.
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use std::{panic, sync::Barrier, thread};

/// A transient with one mutable field, as a Geom2d or Poly class keeps it.
struct SharedRadius {
    base: StandardTransient,
    radius: SharedState<f64>,
}
rustycad_tkernel::impl_transient!(SharedRadius, "SharedRadius", StandardTransient, base);
impl SharedRadius {
    fn shared(radius: f64) -> Handle<Self> {
        Handle::new(Self {
            base: StandardTransient::default(),
            radius: SharedState::new(radius),
        })
    }
}

#[test]
fn write_through_one_handle_is_read_through_another() {
    let first = SharedRadius::shared(1.0);
    let second = first.clone();
    *second.radius.write() = 5.0;
    assert_eq!(*first.radius.read(), 5.0);
}

#[test]
fn snapshot_copies_state_without_aliasing() {
    let state = SharedState::new(vec![1.0, 2.0]);
    let copy = state.snapshot();
    state.write().push(3.0);
    assert_eq!(copy, vec![1.0, 2.0]);
    assert_eq!(state.snapshot(), vec![1.0, 2.0, 3.0]);
}

#[test]
fn default_and_const_construction_hold_the_initial_state() {
    static ORIGIN: SharedState<i32> = SharedState::new(7);
    assert_eq!(*ORIGIN.read(), 7);
    assert_eq!(*SharedState::<i32>::default().read(), 0);
}

#[test]
fn write_on_one_thread_is_visible_on_another() {
    let object = SharedRadius::shared(1.0);
    let writer = object.clone();
    thread::spawn(move || *writer.radius.write() = 2.5)
        .join()
        .unwrap();
    assert_eq!(*object.radius.read(), 2.5);
}

#[test]
fn concurrent_writers_serialize_every_update() {
    let counter = SharedState::new(0_u32);
    thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| (0..1000).for_each(|_| *counter.write() += 1));
        }
    });
    assert_eq!(*counter.read(), 8000);
}

#[test]
fn readers_on_several_threads_do_not_block_each_other() {
    // Every reader holds its guard across the barrier; exclusive reads would deadlock.
    let state = SharedState::new(3.0);
    let barrier = Barrier::new(4);
    thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                let guard = state.read();
                barrier.wait();
                assert_eq!(*guard, 3.0);
            });
        }
    });
}

#[test]
fn poisoned_lock_is_recovered_with_its_state() {
    let state = SharedState::new(4);
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut guard = state.write();
        *guard = 9;
        panic!("poison the lock");
    }));
    assert!(result.is_err());
    assert_eq!(*state.read(), 9);
    *state.write() = 10;
    assert_eq!(state.snapshot(), 10);
}
