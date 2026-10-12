//! Scoped native edge topology is nonowning; explicit clones and checked failures retain ordinary semantics.
use super::{Edge, Face, Shape};
use crate::brep::BRepBuilder;
#[test]
fn scoped_edge_reads_preserve_identity_placement_and_explicit_clone_ownership() {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    Edge::with_shape_reference(&edge, |view| {
        assert!(view.is_equal(&edge));
        assert_eq!(edge.tshape().ref_count(), 1);
        let clone = view.clone();
        assert_eq!(edge.tshape().ref_count(), 2);
        drop(clone);
        assert_eq!(edge.tshape().ref_count(), 1);
        Ok(())
    })
    .unwrap();
    assert_eq!(edge.tshape().ref_count(), 1);
}
#[test]
fn scoped_edge_checks_wrong_kind_and_preserves_native_null_cast() {
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    let failure = Edge::with_shape_reference(&face, |_| Ok(())).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_TypeMismatch");
    Edge::with_shape_reference(&Shape::default(), |edge| {
        assert!(edge.is_null());
        Ok(())
    })
    .unwrap();
}
#[test]
fn scoped_edge_read_resumes_unrelated_failure_payloads() {
    let payload = std::panic::catch_unwind(|| {
        Edge::with_shape_reference(
            &Shape::default(),
            |_| -> Result<(), rustycad_tkernel::standard::Failure> {
                std::panic::panic_any(7u32);
            },
        )
    })
    .unwrap_err();
    assert_eq!(payload.downcast_ref::<u32>(), Some(&7));
}
#[test]
fn scoped_edge_read_preserves_typed_failure_and_releases_the_view() {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    let failure = Edge::with_shape_reference(
        &edge,
        |_| -> Result<(), rustycad_tkernel::standard::Failure> {
            rustycad_tkernel::standard::raise_native_failure(Err(
                rustycad_tkernel::standard::Failure::with_type(
                    "Standard_ProgramError",
                    "edge index=1; expected successful projection",
                    "",
                ),
            ))
        },
    )
    .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert_eq!(edge.tshape().ref_count(), 1);
}
#[test]
fn scoped_face_reads_preserve_ownership_and_check_native_kind() {
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    Face::with_shape_reference(&face, |view| {
        assert!(view.is_equal(&face));
        assert_eq!(face.tshape().ref_count(), 1);
        let owning = view.clone();
        assert_eq!(face.tshape().ref_count(), 2);
        drop(owning);
        Ok(())
    })
    .unwrap();
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    let failure = Face::with_shape_reference(&edge, |_| Ok(())).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_TypeMismatch");
}
