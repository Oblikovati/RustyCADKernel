//! The native exceptions of the sewing classes (docs/topalgo-design.md section 11).
use rustycad_tkernel::standard::Failure;

/// Standard_OutOfRange, as the `_Raise_if` checks and the map accessors raise it.
pub(crate) fn out_of_range(member: &str, index: i32, extent: i32) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("{member}: index {index} is outside [1, {extent}]"),
        "",
    )
}

/// Standard_NoSuchObject, as `Standard_NoSuchObject_Raise_if` and `Find` raise it.
pub(crate) fn no_such_object(message: &str) -> Failure {
    Failure::with_type("Standard_NoSuchObject", message, "")
}

/// Standard_NullObject: a null handle or shape that the native code dereferences.
pub(crate) fn null_object(message: &str) -> Failure {
    Failure::with_type("Standard_NullObject", message, "")
}

/// Standard_ProgramError, as the `Standard_ProgramError_Raise_if` checks raise it.
pub(crate) fn program_error(message: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, "")
}

/// `NCollection_Array1<T>(1, n)` filled with `value`; Standard_RangeError for a
/// negative size, as the native constructor.
pub(crate) fn array1<T: Clone + Default>(
    n: i32,
    value: T,
) -> Result<rustycad_tkernel::ncollection::Array1<'static, T>, Failure> {
    let mut array = rustycad_tkernel::ncollection::Array1::with_bounds(1, n).map_err(|error| {
        Failure::with_type(
            "Standard_RangeError",
            &format!("NCollection_Array1(1, {n}): {error:?}"),
            "",
        )
    })?;
    array.init(value);
    Ok(array)
}
