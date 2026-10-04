//! The native ICurrentData slot, shared across schemas unless explicitly injected.
use crate::standard::Handle;
use crate::storage::Data;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

/// Shared current-write state for storage schemas. Clones share the slot; `Default`
/// creates an independent slot for isolated embedding and tests.
/// Example: `let isolated = SchemaContext::default();`.
#[derive(Clone, Default)]
pub struct SchemaContext {
    current: Arc<Mutex<Handle<Data>>>,
}

impl SchemaContext {
    /// OCCT's process-wide current-data slot (`Storage_Schema::ICurrentData`).
    /// Example: `let shared = SchemaContext::process_shared();`.
    pub fn process_shared() -> Self {
        static CURRENT: OnceLock<SchemaContext> = OnceLock::new();
        CURRENT.get_or_init(Self::default).clone()
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, Handle<Data>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
