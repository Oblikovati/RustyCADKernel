//! Bounded observation preserves identity, orientation, location and ownership while propagating actual callback failures.
use super::*;
use rustycad_tkernel::standard::{Failure, Handle, raise_native_failure};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};
#[test]
fn reference_scope_preserves_topology_counts_and_all_shape_fields() {
    let topology: Handle<dyn TShape> = Handle::new(TCompound::new()).cast();
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(2., 3., 4.));
    let placement = Location::new(transform);
    Shape::with_reference_parts(&topology, &placement, Orientation::Reversed, |shape| {
        assert_eq!(shape.tshape().object_address(), topology.object_address());
        assert_eq!(shape.tshape().ref_count(), 1);
        assert!(shape.location().is_equal(&placement));
        assert_eq!(shape.orientation(), Orientation::Reversed);
        let copied = shape.clone();
        assert_eq!(topology.ref_count(), 2);
        assert!(copied.is_equal(shape));
        Ok(())
    })
    .unwrap();
    assert_eq!(topology.ref_count(), 1);
}
#[test]
fn reference_scope_releases_observation_after_result_or_typed_failure() {
    let topology: Handle<dyn TShape> = Handle::new(TCompound::new()).cast();
    let failure = Failure::out_of_range("callback index=0; expected 1..=1");
    let returned = Shape::with_reference_parts(
        &topology,
        &Location::default(),
        Orientation::Forward,
        |_| Err::<(), _>(failure.clone()),
    );
    assert_eq!(returned, Err(failure.clone()));
    let panicked = Shape::with_reference_parts(
        &topology,
        &Location::default(),
        Orientation::Forward,
        |_| Ok(raise_native_failure::<()>(Err(failure.clone()))),
    );
    assert_eq!(panicked, Err(failure));
    assert_eq!(topology.ref_count(), 1);
}
#[test]
fn reference_scope_resumes_unrelated_panics_and_supports_null_reference_reads() {
    let topology: Handle<dyn TShape> = Handle::default();
    let empty = Shape::with_reference_parts(
        &topology,
        &Location::default(),
        Orientation::External,
        |shape| Ok(shape.is_null()),
    );
    assert_eq!(empty, Ok(true));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Shape::with_reference_parts::<()>(
            &topology,
            &Location::default(),
            Orientation::External,
            |_| panic!("named foreign callback panic"),
        )
    }));
    assert!(panic.is_err());
}
