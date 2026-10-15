//! The native exceptions of TKFeat as `Failure` values, each naming the member and the
//! expected state.
use rustycad_tkernel::standard::Failure;

/// `StdFail_NotDone` of a member called before a successful computation;
/// `not_done("LocOpe_CSIntersector::NbPoints")`.
pub(crate) fn not_done(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("{member}: the algorithm is not done; expected a successful Perform first"),
    )
}

/// `Standard_OutOfRange` of a 1-based index outside `[1, length]`;
/// `out_of_range("LocOpe_CSIntersector::Point", 3, 2)`.
pub(crate) fn out_of_range(member: &str, index: i32, length: i32) -> Failure {
    Failure::out_of_range(&format!(
        "{member}: index {index}; expected an index in [1, {length}]"
    ))
}

/// `Standard_ConstructionError` of a member given unusable input;
/// `construction_error("LocOpe_CSIntersector::Perform", "null shape")`.
pub(crate) fn construction_error(member: &str, detail: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", &format!("{member}: {detail}"))
}

/// `Standard_NullObject` of a member dereferencing a null handle or shape;
/// `null_object("LocOpe::SampleEdges", "edge without 3D curve")`.
pub(crate) fn null_object(member: &str, detail: &str) -> Failure {
    Failure::with_message("Standard_NullObject", &format!("{member}: {detail}"))
}

/// `Standard_NoSuchObject` of a lookup that found nothing;
/// `no_such_object("LocOpe_Gluer::DescendantFaces", "face not bound")`.
pub(crate) fn no_such_object(member: &str, detail: &str) -> Failure {
    Failure::with_message("Standard_NoSuchObject", &format!("{member}: {detail}"))
}

/// `Standard_DomainError` of a member called in a state that does not allow it;
/// `domain_error("LocOpe_Generator::Perform", "no generated shape")`.
pub(crate) fn domain_error(member: &str, detail: &str) -> Failure {
    Failure::with_message("Standard_DomainError", &format!("{member}: {detail}"))
}
