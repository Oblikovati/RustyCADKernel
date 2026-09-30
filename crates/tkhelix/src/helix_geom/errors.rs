// Rust support for the native exceptions of ModelingAlgorithms/TKHelix
// at 3d097a0328e71b826377d4814ab05ec3c3d23871 (docs/helix-design.md section 3).

//! The native exceptions that TKHelix members raise, as `Failure` values carrying the
//! native exception type.
use rustycad_tkernel::ncollection::ArrayError;
use rustycad_tkernel::standard::Failure;

/// `Standard_ConstructionError`; `construction_error("HelixGeom_HelixCurve::Load: ...")`.
pub(crate) fn construction_error(message: &str) -> Failure {
    Failure::with_type("Standard_ConstructionError", message, "")
}

/// `Standard_NotImplemented`; `not_implemented("HelixGeom_HelixCurve::Resolution")`.
pub(crate) fn not_implemented(message: &str) -> Failure {
    Failure::with_type("Standard_NotImplemented", message, "")
}

/// `Standard_DomainError`; `domain_error("HelixGeom_HelixCurve::Period")`.
pub(crate) fn domain_error(message: &str) -> Failure {
    Failure::with_type("Standard_DomainError", message, "")
}

/// `Standard_NullObject` of a null handle dereference; `null_object("aC1")`.
pub(crate) fn null_object(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{what}: null handle; expected an object"),
        "",
    )
}

/// The exception a native `NCollection_Array1` access raises (Standard_OutOfRange for an
/// index, Standard_RangeError for bounds); `array.value(1).map_err(array_failure("T"))`.
pub(crate) fn array_failure(member: &'static str) -> impl Fn(ArrayError) -> Failure {
    move |error: ArrayError| error.into_failure(member)
}

/// The x86-64 truncating conversion `(int)value` (`cvttsd2si`): NaN and values outside
/// the `int` range give the "integer indefinite" `INT_MIN`, where Rust `as` saturates;
/// `c_int_truncation(2.9) == 2`.
pub(crate) fn c_int_truncation(value: f64) -> i32 {
    // 2^31 is exact; every double in (-2^31 - 1, 2^31) truncates into the int range.
    let limit: f64 = 2147483648.0;
    if value.is_nan() || value >= limit || value <= -limit - 1.0 {
        return i32::MIN;
    }
    value as i32
}

#[cfg(test)]
mod tests {
    use super::c_int_truncation;

    #[test]
    fn c_int_truncation_matches_cvttsd2si() {
        assert_eq!(c_int_truncation(2.9), 2);
        assert_eq!(c_int_truncation(-2.9), -2);
        assert_eq!(c_int_truncation(f64::NAN), i32::MIN);
        assert_eq!(c_int_truncation(3.0e10), i32::MIN);
        assert_eq!(c_int_truncation(-3.0e10), i32::MIN);
        assert_eq!(c_int_truncation(-2147483648.5), i32::MIN);
        assert_eq!(c_int_truncation(2147483647.5), i32::MAX);
    }
}
