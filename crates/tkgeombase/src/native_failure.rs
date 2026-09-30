// Rust translation of ModelingData/TKGeomBase/FEmTool and AppParCurves (shared exception helpers)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Native exception values shared by the TKGeomBase ports that raise them from
//! container accesses (FEmTool, AppParCurves).
use rustycad_tkernel::ncollection::ArrayError;
use rustycad_tkernel::standard::Failure;

/// A native exception of class `kind` with `message`, without a stack trace;
/// `failure("Standard_DomainError", "FEmTool_Assembly::AddMatrix")`.
pub(crate) fn failure(kind: &'static str, message: &str) -> Failure {
    Failure::with_type(kind, message, "")
}

/// The exception an NCollection or math container raises for `error`: bad bounds
/// are Standard_RangeError (the allocating constructors), a bad index
/// Standard_OutOfRange and a size mismatch Standard_DimensionMismatch;
/// `array_failure(error, "FEmTool_Curve::Knots")`.
pub(crate) fn array_failure(error: ArrayError, context: &str) -> Failure {
    let kind: &'static str = match error {
        ArrayError::Bounds { .. } => "Standard_RangeError",
        ArrayError::Index { .. } => "Standard_OutOfRange",
        _ => "Standard_DimensionMismatch",
    };
    failure(kind, &format!("{context}: {error}"))
}

/// The exception of a native `NCollection_Array1(Lower, Upper)` allocation: bounds
/// with `Upper < Lower - 1` wrap the size to a huge value whose allocation fails
/// (natively a null buffer, then a crash on the first write); the port raises
/// Standard_OutOfMemory (AppDef_LinearCriteria, AppDef_Variational);
/// `allocation_failure(error, "AppDef_Variational")`.
pub(crate) fn allocation_failure(error: ArrayError, context: &str) -> Failure {
    match error {
        ArrayError::Bounds { .. } => {
            failure("Standard_OutOfMemory", &format!("{context}: {error}"))
        }
        other => array_failure(other, context),
    }
}
