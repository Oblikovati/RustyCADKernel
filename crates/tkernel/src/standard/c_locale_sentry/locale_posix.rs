#![allow(unsafe_code)]

use libc::locale_t;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::OnceLock;

/// Owns the shared C locale for process lifetime; POSIX locale objects are immutable after creation.
#[derive(Debug)]
struct SharedCLocale {
    handle: locale_t,
}

// SAFETY: the locale handle is initialized once, never mutated or freed, and POSIX permits its
// immutable locale object to be activated concurrently by separate threads.
unsafe impl Send for SharedCLocale {}
// SAFETY: same invariant as Send; access only passes the private handle to uselocale.
unsafe impl Sync for SharedCLocale {}

static C_LOCALE: OnceLock<SharedCLocale> = OnceLock::new();

trait LocaleOperations {
    type Handle: Copy;

    fn c_locale(&self) -> Self::Handle;
    fn use_locale(&self, requested: Self::Handle) -> Self::Handle;
}

#[derive(Debug)]
struct LocaleScope<Operations: LocaleOperations> {
    operations: Operations,
    previous_locale: Operations::Handle,
    _thread_bound: PhantomData<Rc<()>>,
}

impl<Operations: LocaleOperations> LocaleScope<Operations> {
    fn with_operations(operations: Operations) -> Self {
        let previous_locale = operations.use_locale(operations.c_locale());
        Self {
            operations,
            previous_locale,
            _thread_bound: PhantomData,
        }
    }
}

impl<Operations: LocaleOperations> Drop for LocaleScope<Operations> {
    fn drop(&mut self) {
        let _ignored_restore_result = self.operations.use_locale(self.previous_locale);
    }
}

#[derive(Debug)]
struct NativeLocaleOperations;

impl LocaleOperations for NativeLocaleOperations {
    type Handle = locale_t;

    fn c_locale(&self) -> Self::Handle {
        c_locale_handle()
    }

    fn use_locale(&self, requested: Self::Handle) -> Self::Handle {
        unsafe { libc::uselocale(requested) }
    }
}

#[derive(Debug)]
pub(super) struct ThreadLocaleScope {
    _scope: LocaleScope<NativeLocaleOperations>,
}

impl ThreadLocaleScope {
    pub(super) fn enter() -> Self {
        Self {
            _scope: LocaleScope::with_operations(NativeLocaleOperations),
        }
    }
}

fn c_locale_handle() -> locale_t {
    C_LOCALE
        .get_or_init(|| SharedCLocale {
            handle: create_c_locale(),
        })
        .handle
}

fn create_c_locale() -> locale_t {
    unsafe { libc::newlocale(libc::LC_ALL_MASK, c"C".as_ptr(), std::ptr::null_mut()) }
}

#[cfg(test)]
mod tests {
    use super::{LocaleOperations, LocaleScope, ThreadLocaleScope, c_locale_handle};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Debug)]
    struct FakeLocaleOperations(Rc<RefCell<FakeLocaleState>>);

    #[derive(Debug)]
    struct FakeLocaleState {
        c_locale: usize,
        current_locale: usize,
        failed_requests: Vec<usize>,
        requests: Vec<usize>,
    }

    impl LocaleOperations for FakeLocaleOperations {
        type Handle = usize;

        fn c_locale(&self) -> usize {
            self.0.borrow().c_locale
        }

        fn use_locale(&self, requested: usize) -> usize {
            let mut state = self.0.borrow_mut();
            state.requests.push(requested);
            if requested == 0 {
                return state.current_locale;
            }
            if state.failed_requests.contains(&requested) {
                return 0;
            }
            std::mem::replace(&mut state.current_locale, requested)
        }
    }

    #[test]
    fn nested_scopes_activate_the_same_cached_handle() {
        let outer = ThreadLocaleScope::enter();
        let first = unsafe { libc::uselocale(std::ptr::null_mut()) };
        let inner = ThreadLocaleScope::enter();
        let second = unsafe { libc::uselocale(std::ptr::null_mut()) };
        assert_eq!(first, c_locale_handle());
        assert_eq!(second, first);
        drop(inner);
        assert_eq!(unsafe { libc::uselocale(std::ptr::null_mut()) }, first);
        drop(outer);
    }

    #[test]
    fn null_cached_locale_follows_uselocale_query_and_ignored_restore_flow() {
        let operations = fake_operations(0, 19, vec![]);
        let sentry = LocaleScope::with_operations(operations.clone());
        assert_eq!(operations.0.borrow().current_locale, 19);
        drop(sentry);
        let state = operations.0.borrow();
        assert_eq!(state.requests, [0, 19]);
        assert_eq!(state.current_locale, 19);
    }

    #[test]
    fn activation_error_is_retained_and_drop_uses_native_null_query_semantics() {
        let operations = fake_operations(23, 19, vec![23]);
        let sentry = LocaleScope::with_operations(operations.clone());
        drop(sentry);
        let state = operations.0.borrow();
        assert_eq!(state.requests, [23, 0]);
        assert_eq!(state.current_locale, 19);
    }

    #[test]
    fn drop_ignores_a_restore_error_like_the_native_destructor() {
        let operations = fake_operations(23, 19, vec![19]);
        let sentry = LocaleScope::with_operations(operations.clone());
        drop(sentry);
        let state = operations.0.borrow();
        assert_eq!(state.requests, [23, 19]);
        assert_eq!(state.current_locale, 23);
    }

    fn fake_operations(
        c_locale: usize,
        current_locale: usize,
        failed_requests: Vec<usize>,
    ) -> FakeLocaleOperations {
        FakeLocaleOperations(Rc::new(RefCell::new(FakeLocaleState {
            c_locale,
            current_locale,
            failed_requests,
            requests: Vec::new(),
        })))
    }
}
