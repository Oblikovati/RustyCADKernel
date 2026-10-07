//! Shared mutable ownership for the ChFiDS transient classes.
//!
//! The fillet builders keep `occ::handle<ChFiDS_SurfData>`, `occ::handle<ChFiDS_Stripe>`
//! and `occ::handle<ChFiDS_Spine>` in several containers at once and mutate the object
//! through any of them (`ChangeInterferenceOnS1()`, `ChangeSpine()`...). A [`Shared`]
//! value is that handle: clones share one object, and [`lock`] gives the mutable view.
//! A null native handle is an `Option::None` of it.
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// A shared, mutable object: one native handle; `shared(SurfData::new())`.
pub type Shared<T> = Arc<Mutex<T>>;

/// Wraps `value` in a new shared handle, as `new T(...)`; `let data = shared(SurfData::new());`.
pub fn shared<T>(value: T) -> Shared<T> {
    Arc::new(Mutex::new(value))
}

/// The mutable view of a shared object. A poisoned lock is recovered: the port never
/// panics while it holds a guard, so the state stays consistent; `lock(&data).surf()`.
pub fn lock<T: ?Sized>(handle: &Shared<T>) -> MutexGuard<'_, T> {
    handle.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether two handles hold the same object, as the native handle `==`;
/// `same_object(&a, &b)`.
pub fn same_object<T: ?Sized>(first: &Shared<T>, second: &Shared<T>) -> bool {
    Arc::ptr_eq(first, second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_one_object() {
        let first = shared(1_i32);
        let second = Arc::clone(&first);
        *lock(&second) = 5;
        assert_eq!(*lock(&first), 5);
        assert!(same_object(&first, &second));
        assert!(!same_object(&first, &shared(5)));
    }
}
