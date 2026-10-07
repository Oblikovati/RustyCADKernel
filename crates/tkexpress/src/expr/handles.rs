// Rust support for the `occ::handle` idioms of ModelingAlgorithms/TKExpress/Expr at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Handle helpers shared by every Expr class: the general handle type, checked
//! dereference (a null handle is the native crash, here a `Standard_NullObject`
//! failure) and the self pointer behind `occ::handle<T> me = this`.
use super::errors::null_object;
use super::general_expression::ExprGeneralExpression;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{
    Failure, Handle, RuntimeType, RuntimeTyped, StandardTransient, WeakHandle,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;

/// `occ::handle<Expr_GeneralExpression>`; `let sum: ExprHandle = ExprSum::new(&a, &b).cast();`.
pub type ExprHandle = Handle<dyn ExprGeneralExpression>;

/// Borrow the object of `handle`, or fail as the native null dereference does;
/// `let value: &ExprNumericValue = deref(&handle, "GetValue")?;`.
pub fn deref<'h, T: ?Sized + RuntimeTyped>(
    handle: &'h Handle<T>,
    member: &str,
) -> Result<&'h T, Failure> {
    handle.get().ok_or_else(|| {
        null_object(&format!(
            "{member}: null handle of {}; expected an object",
            T::TYPE_NAME
        ))
    })
}

/// True when `handle` is not null and its object is `T` or inherits from it, as
/// `handle->IsKind(STANDARD_TYPE(T))` (false for null, where native code would crash
/// only if it dereferenced); `is_kind::<ExprNumericValue>(&operand)`.
pub fn is_kind<T: ?Sized + RuntimeTyped>(handle: &ExprHandle) -> bool {
    handle.is_kind(T::static_type())
}

/// The descriptor test of [`is_kind`] for a known descriptor;
/// `is_kind_of(&operand, ExprSine::static_type())`.
pub fn is_kind_of(handle: &ExprHandle, descriptor: &RuntimeType) -> bool {
    handle.is_kind(descriptor)
}

/// The text of a name for an error message; `lossy(&AsciiString::new("x")) == "x"`.
pub fn lossy(name: &AsciiString) -> String {
    String::from_utf8_lossy(name.as_bytes()).into_owned()
}

/// The array `NCollection_Array1(1, n)` holding `values` in order;
/// `let values = one_based(vec![1.0, 2.0]);` gives bounds 1..2.
pub fn one_based<T: Default>(values: Vec<T>) -> Array1<'static, T> {
    let count: i32 = i32::try_from(values.len()).unwrap_or(i32::MAX);
    let mut array: Array1<'static, T> = Array1::with_bounds(1, count).unwrap_or_default();
    for (slot, value) in array.iter_mut().zip(values) {
        *slot = value;
    }
    array
}

/// The non-owning pointer of an object to its own handle, which OCCT reads as
/// `occ::handle<T> me = this`.
///
/// It is bound once, by the constructor that wraps the object in its first handle; a
/// `me` request on an object no handle owns fails as `Standard_Transient::This()` does.
#[derive(Default)]
pub struct ExprSelfLink {
    pointer: OnceLock<WeakHandle<StandardTransient>>,
}
impl ExprSelfLink {
    /// Remember `handle` as the handle of the object holding this link;
    /// `link.bind(&handle)`.
    pub fn bind<T: ?Sized + RuntimeTyped>(&self, handle: &Handle<T>) {
        let root: Handle<StandardTransient> = handle.cast();
        // A second bind keeps the first pointer: the object has one identity.
        let _ = self.pointer.set(root.downgrade());
    }
    /// A new handle to the object, projected to `U`; `let me: ExprHandle = link.me()?;`.
    pub fn me<U: ?Sized + RuntimeTyped>(&self) -> Result<Handle<U>, Failure> {
        let me: Handle<U> = self
            .pointer
            .get()
            .map(WeakHandle::upgrade)
            .unwrap_or_default()
            .cast();
        if me.is_null() {
            return Err(rustycad_tkernel::standard::exceptions::program_error(
                "Attempt to create handle to object created in stack, not yet constructed, or destroyed",
            ));
        }
        Ok(me)
    }
    /// The address of the object, the identity `this` compares by; null when unbound;
    /// `link.address() == other.object_address()`.
    pub fn address(&self) -> *const () {
        self.pointer
            .get()
            .map_or(std::ptr::null(), WeakHandle::object_address)
    }
}

/// Wrap a fresh object in its first handle and bind its self pointer;
/// `adopt(ExprNumericValue::build(1.0), |value| &value.link)`.
pub fn adopt<T: RuntimeTyped + rustycad_tkernel::standard::Transient>(
    object: T,
    link: fn(&T) -> &ExprSelfLink,
) -> Handle<T> {
    let handle: Handle<T> = Handle::new(object);
    if let Some(created) = handle.get() {
        link(created).bind(&handle);
    }
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_based_arrays_start_at_one() {
        let values: Array1<'static, f64> = one_based(vec![2.0, 3.0]);
        assert_eq!((values.lower(), values.upper()), (1, 2));
        assert_eq!(values[2], 3.0);
        assert_eq!(one_based::<f64>(Vec::new()).length(), 0);
    }

    #[test]
    fn an_unbound_link_refuses_me() {
        let link: ExprSelfLink = ExprSelfLink::default();
        assert!(link.address().is_null());
        let failure: Failure = link.me::<StandardTransient>().expect_err("unbound");
        assert_eq!(failure.exception_type(), "Standard_ProgramError");
    }
}
