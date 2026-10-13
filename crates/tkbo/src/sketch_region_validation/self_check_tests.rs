// SPDX-License-Identifier: GPL-2.0-only
//! Real finite tangent walking is unfinished without a fabricated FaceFace flag.
use super::outcome::StageCoverage;
use super::self_check::{observe_walking, walking_coverage};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::{
    adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle,
    geom::{GeomPlane, GeomSphericalSurface, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkgeomalgo::int_walk::IntWalkPWalking;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use std::f64::consts::{FRAC_PI_2, PI};

fn tangent_surface_pair() -> [Adaptor3dSurfaceHandle; 2] {
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(z_frame(0.3), 1.0).unwrap()).cast();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(z_frame(1.3))).cast();
    [
        Handle::new(
            GeomAdaptorSurface::with_bounds(&sphere, (0.0, 2.0 * PI, -FRAC_PI_2, FRAC_PI_2))
                .unwrap(),
        )
        .cast(),
        Handle::new(GeomAdaptorSurface::with_bounds(&plane, (-2.0, 2.0, -2.0, 2.0)).unwrap())
            .cast(),
    ]
}

fn tangent_walking() -> (IntWalkPWalking, [f64; 4]) {
    let [sphere, plane] = tangent_surface_pair();
    let values = [0.0, FRAC_PI_2, 0.0, 0.0];
    let mut start = Array1::with_bounds(1, 4).unwrap();
    for (index, value) in values.iter().enumerate() {
        start[index as i32 + 1] = *value;
    }
    let mut walk = IntWalkPWalking::new(&sphere, &plane, 1e-6, 1e-8, 0.01, 0.1).unwrap();
    walk.perform(&start).unwrap();
    (walk, values)
}

#[test]
fn real_unfinished_walking_cannot_be_complete_analytic_coverage() {
    let (walk, start) = tangent_walking();
    assert!(!walk.is_done());
    assert_eq!(
        walk.nb_points().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let witness = observe_walking(&walk, start);
    assert!(!witness.done);
    assert_eq!(witness.start, start);
    assert_eq!(walking_coverage(&witness), StageCoverage::Incomplete);
}

fn z_frame(z: f64) -> Axis3 {
    let origin = Point3::new(0.0, 0.0, z);
    let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
    Axis3::from_normal(origin, normal).unwrap()
}

#[test]
fn actual_outer_termination_keeps_each_freeform_source_incomplete() {
    let shape = converted_box();
    let findings = super::self_check::inspect_self_intersection(&shape, &[]);
    let freeform: Vec<_> = findings
        .iter()
        .filter(|finding| {
            matches!(
                &finding.cause,
                super::outcome::ValidationCause::AnalyticGap {
                    index: Some(_),
                    outer: Some(crate::bop_algo::FaceTermination::Terminated { .. }),
                    ..
                }
            )
        })
        .collect();
    assert_freeform_findings(&freeform);
}

fn assert_freeform_findings(freeform: &[&super::outcome::ValidationFinding]) {
    assert_eq!(freeform.len(), 6);
    assert!(
        freeform
            .iter()
            .all(|finding| finding.coverage == StageCoverage::Incomplete)
    );
    assert!(
        freeform
            .iter()
            .all(|finding| finding.affected.shape.shape_type().unwrap()
                == rustycad_tkg3d::top_abs::ShapeEnum::Face)
    );
}

#[test]
fn mapped_contacts_fail_strict_coverage_with_real_entity_contexts() {
    let shape = overlapping_solids();
    let findings = super::self_check::inspect_self_intersection(&shape, &[]);
    let contacts: Vec<_> = findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.cause,
                super::outcome::ValidationCause::AnalyticInterference { .. }
            )
        })
        .collect();
    assert_contact_findings(&contacts);
}

fn assert_contact_findings(contacts: &[&super::outcome::ValidationFinding]) {
    assert!(!contacts.is_empty());
    assert!(
        contacts
            .iter()
            .all(|finding| finding.coverage == StageCoverage::Failed)
    );
    assert!(
        contacts
            .iter()
            .all(|finding| !finding.affected.shape.is_null())
    );
    assert!(
        contacts
            .iter()
            .all(|finding| !finding.context.as_ref().unwrap().shape.is_null())
    );
}

fn overlapping_solids() -> rustycad_tkbrep::topods::Shape {
    use rustycad_tkbrep::{brep::BRepBuilder, topods::Compound};
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
    let mut left = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let mut right =
        BRepPrimAPIMakeBox::at_point(Point3::new(0.5, 0.5, 0.5), 1.0, 1.0, 1.0).unwrap();
    let mut shape = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut shape);
    builder.add(&mut shape, left.shape().unwrap()).unwrap();
    builder.add(&mut shape, right.shape().unwrap()).unwrap();
    shape.into_shape()
}

fn converted_box() -> rustycad_tkbrep::topods::Shape {
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
    use rustycad_tktopalgo::brep_builder_api::{
        BRepBuilderAPIMakeShape, BRepBuilderAPINurbsConvert,
    };
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let mut converted =
        BRepBuilderAPINurbsConvert::with_shape(maker.shape().unwrap(), false).unwrap();
    converted.shape().unwrap().clone()
}
