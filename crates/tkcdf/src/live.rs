//! Shared helpers of the OCAF ports: null-handle checks and the shared resource manager.
use rustycad_tkernel::{
    resource::ResourceManager,
    standard::{Failure, Handle, RuntimeTyped, SharedState, exceptions},
};
use std::sync::Arc;

/// `occ::handle<Resource_Manager>`: the TKernel `ResourceManager` is not a transient, so the
/// shared, mutable handle is an `Arc` over its `SharedState` (ocaf-design.md section 6).
pub type SharedResourceManager = Arc<SharedState<ResourceManager>>;

/// Borrows the object of a handle, or returns the `Standard_NullObject` that a native
/// dereference of the null handle stands for; `live(&handle, "CDM_Reference::myMetaData")?`.
///
/// ```
/// use rustycad_tkcdf::live;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let empty: Handle<StandardTransient> = Handle::default();
/// assert_eq!(live(&empty, "x").err().unwrap().exception_type(), "Standard_NullObject");
/// ```
pub fn live<'a, T: ?Sized + RuntimeTyped>(
    handle: &'a Handle<T>,
    what: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        exceptions::null_object(&format!(
            "{what}: null handle; expected a live {}",
            T::TYPE_NAME
        ))
    })
}
