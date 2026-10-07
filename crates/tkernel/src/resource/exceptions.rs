// Rust translation of FoundationClasses/TKernel/Resource/Resource_NoSuchResource.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Resource_NoSuchResource`, the one exception class of the Resource package.
use crate::standard::Failure;

/// Raises `Resource_NoSuchResource` (a `Standard_NoSuchObject`), the failure of a lookup
/// of an absent resource; `resource::no_such_resource("Units.Length")`.
pub fn no_such_resource(message: &str) -> Failure {
    Failure::with_type("Resource_NoSuchResource", message, "")
}

/// `(class, direct base)` of DEFINE_STANDARD_EXCEPTION(Resource_NoSuchResource,
/// Standard_NoSuchObject), so a catch by Standard_NoSuchObject or Standard_DomainError
/// also catches it (`Failure::is_kind`).
pub(crate) const RESOURCE_HIERARCHY: &[(&str, &str)] =
    &[("Resource_NoSuchResource", "Standard_NoSuchObject")];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_such_resource_is_a_no_such_object() {
        let failure = no_such_resource("missing.key");
        assert_eq!(failure.exception_type(), "Resource_NoSuchResource");
        assert_eq!(failure.message(), "missing.key");
        assert!(failure.is_kind("Standard_NoSuchObject"));
        assert!(failure.is_kind("Standard_DomainError"));
        assert!(!failure.is_kind("Standard_RangeError"));
    }
}
