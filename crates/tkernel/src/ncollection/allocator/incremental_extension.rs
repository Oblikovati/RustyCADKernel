//! Protected incremental operations borrow the canonical engine under its existing mutex.
use super::{AllocatorKind, BaseAllocator, IncAllocatorObservation, IncEngine, locked};
use crate::standard::Failure;
use std::{ptr::NonNull, sync::MutexGuard};

/// Lock-scoped native incremental extension operations.
///
/// Release this guard before reentering its allocator; a second acquisition would deadlock.
/// ```
/// use rustycad_tkernel::ncollection::BaseAllocator;
/// let arena = BaseAllocator::incremental(4096);
/// let mut extension = arena.incremental_extension().unwrap();
/// extension.increase_block_size();
/// let address = extension.allocate_slow(8).unwrap();
/// ```
pub struct IncAllocatorExtension<'a> {
    engine: MutexGuard<'a, IncEngine>,
}
impl IncAllocatorExtension<'_> {
    /// Performs native protected allocation with list repair; `extension.allocate_slow(32)`.
    /// Returned raw bytes remain owned by the arena and do not acquire a typed-owner lease.
    pub fn allocate_slow(&mut self, size: usize) -> Option<NonNull<u8>> {
        self.engine.allocate_slow(size)
    }
    /// Advances the real next-block capacity using the native growth level; `extension.increase_block_size()`.
    pub fn increase_block_size(&mut self) {
        self.engine.increase_block_size();
    }
    /// Copies actual header topology under this guard; `let state = extension.observation();`.
    /// Pointer fields become numeric identities, and no writable native field escapes.
    pub fn observation(&self) -> IncAllocatorObservation {
        // Copy only while the guard excludes concurrent topology mutation.
        self.engine.observation()
    }

    /// Rewinds one validated native block without changing its list membership.
    /// `extension.reset_block(extension.observation().ordered_blocks)?;`
    /// Rust rejects all live typed/RAII leases; addresses outside the current ownership chain are also rejected.
    pub fn reset_block(&mut self, address: usize) -> Result<(), Failure> {
        // Membership and live leases are checked under this same guard.
        self.engine.reset_block(address)
    }

    /// Releases all actual blocks and restores native default capacity; `extension.clean()?;`.
    /// The native thread policy survives. Rust rejects this operation while safe owners remain.
    pub fn clean(&mut self) -> Result<(), Failure> {
        // Keep lease validation and deallocation atomic with respect to other handles.
        self.engine.clean_checked()
    }
}
impl BaseAllocator {
    /// Borrows the real protected incremental engine; `let access = arena.incremental_extension()?`.
    pub fn incremental_extension(&self) -> Result<IncAllocatorExtension<'_>, Failure> {
        let AllocatorKind::Incremental(engine) = &*self.inner else {
            return Err(Failure::with_message(
                "Standard_TypeMismatch",
                &format!(
                    "{} extension: expected NCollection_IncAllocator",
                    self.kind_name()
                ),
            ));
        };
        Ok(IncAllocatorExtension {
            engine: locked(engine),
        })
    }
}
