// Rust port of DE_Wrapper::GlobalWrapper, SetGlobalWrapper and GlobalLoadMutex of
// DataExchange/TKDE/DE/DE_Wrapper.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The process-wide wrapper the format plugins register into (de-design.md section 3).
use super::DeWrapper;
use rustycad_tkernel::standard::Handle;
use std::sync::{Mutex, OnceLock, PoisonError, RwLock};

/// The native `THE_GLOBAL_CONFIGURATION()`: created on first use as an empty wrapper.
fn global_configuration() -> &'static RwLock<Handle<DeWrapper>> {
    static GLOBAL: OnceLock<RwLock<Handle<DeWrapper>>> = OnceLock::new();
    GLOBAL.get_or_init(|| RwLock::new(Handle::new(DeWrapper::new())))
}

impl DeWrapper {
    /// `GlobalWrapper()`: the process-wide wrapper (a new empty one until replaced);
    /// `DeWrapper::global_wrapper().get().is_some()`.
    pub fn global_wrapper() -> Handle<DeWrapper> {
        global_configuration()
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// `SetGlobalWrapper(theWrapper)`: replaces the process-wide wrapper; a null handle is
    /// ignored, as natively; `DeWrapper::set_global_wrapper(&Handle::new(DeWrapper::new()))`.
    pub fn set_global_wrapper(wrapper: &Handle<DeWrapper>) {
        if wrapper.is_null() {
            return;
        }
        *global_configuration()
            .write()
            .unwrap_or_else(PoisonError::into_inner) = wrapper.clone();
    }

    /// `GlobalLoadMutex()`: the lock plugin holders take while they register;
    /// `let _guard = DeWrapper::global_load_mutex().lock();`.
    pub fn global_load_mutex() -> &'static Mutex<()> {
        static LOAD_MUTEX: Mutex<()> = Mutex::new(());
        &LOAD_MUTEX
    }
}
