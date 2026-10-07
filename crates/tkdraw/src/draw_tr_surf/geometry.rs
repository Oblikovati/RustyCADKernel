//! Access to the geometry handles of the DrawTrSurf drawables.
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};

/// The object of `handle`; Standard_NullObject when it is null (a drawable restored
/// from an unreadable record, which the C++ code would dereference).
pub(crate) fn geometry<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Result<&T, Failure> {
    handle
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "null geometry in a drawable"))
}
