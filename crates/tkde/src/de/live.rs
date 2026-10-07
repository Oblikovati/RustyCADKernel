//! The null-handle check of the DE ports: a native dereference of a null handle is a
//! `Standard_NullObject` failure in Rust.
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, exceptions};

/// Borrows the object of a handle, or returns the `Standard_NullObject` that a native
/// dereference of the null handle stands for; `live(&wrapper, "DE_Wrapper::Load")?`.
///
/// ```
/// use rustycad_tkde::de::{DeWrapper, live};
/// use rustycad_tkernel::standard::Handle;
/// let empty: Handle<DeWrapper> = Handle::default();
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
