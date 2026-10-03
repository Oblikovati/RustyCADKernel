//! Native statistics, recording flags and copy-sharing behavior.
#[path = "support/shape_contents/mod.rs"]
mod fixture;
#[path = "support/shape_contents/observations.rs"]
mod observations;
use observations::assert_native;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Edge, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkmath::{
    gp::{Axis3, Cylinder3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisShapeContents;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
#[test]
fn shape_contents_counts_repeated_placements_as_native_does() {
    let mut contents = ShapeAnalysisShapeContents::new();
    contents.clear();
    assert_native("clear", &contents);
    let shape = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    contents.perform(&shape).unwrap();
    assert_native("box", &contents);
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &shape).unwrap();
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(10., 0., 0.));
    builder
        .add(
            &mut compound,
            &shape.moved(&Location::new(transform), true).unwrap(),
        )
        .unwrap();
    contents.perform(&compound).unwrap();
    assert_native("repeated-located-box", &contents);
}
#[test]
fn shape_contents_surface_kinds_and_seams_match_native() {
    let mut contents = ShapeAnalysisShapeContents::new();
    fixture::modes(&mut contents);
    let cylinder = Cylinder3::new(Axis3::default(), 1.).unwrap();
    let face =
        BRepBuilderAPIMakeFace::from_cylinder_bounds(cylinder, (0., std::f64::consts::TAU, 0., 2.))
            .unwrap()
            .face()
            .unwrap();
    contents.perform(&face).unwrap();
    assert_native("seam-cylinder", &contents);
    let mut axis = Axis3::default();
    axis.y_reverse();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(axis)).cast();
    contents.perform(&fixture::face(&plane)).unwrap();
    assert_native("indirect", &contents);
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let offset: Handle<dyn GeomSurface> =
        Handle::new(GeomOffsetSurface::new(&plane, 1., false).unwrap()).cast();
    contents.perform(&fixture::face(&offset)).unwrap();
    assert_native("offset-surface", &contents);
}
#[test]
fn shape_contents_spline_threshold_and_one_trim_unwrap_match_native() {
    let mut contents = ShapeAnalysisShapeContents::new();
    fixture::modes(&mut contents);
    let trimmed: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&fixture::spline(3), (0., 1.), (0., 2.), (true, true))
            .unwrap(),
    )
    .cast();
    contents.perform(&fixture::face(&trimmed)).unwrap();
    assert_native("trimmed-spline", &contents);
    contents
        .perform(&fixture::face(&fixture::spline(4096)))
        .unwrap();
    assert_native("spline-threshold", &contents);
    contents
        .perform(&fixture::face(&fixture::spline(4097)))
        .unwrap();
    assert_native("big-spline", &contents);
}
#[test]
fn shape_contents_curve_recordings_and_shared_copy_clear_match_native() {
    let mut contents = ShapeAnalysisShapeContents::new();
    fixture::modes(&mut contents);
    let face = fixture::curve_face();
    contents.perform(&face).unwrap();
    assert_native("curve-kinds", &contents);
    let saved = contents.trimmed_3d_sequence();
    contents.clear_flags();
    assert_native("clear-flags", &contents);
    let mut copy = contents.clone();
    copy.clear();
    assert_native("shared-clear", &contents);
    assert_eq!(saved.sequence(|s| s.len()), 0);
    contents.perform(&face).unwrap();
    assert_native("recording-disabled", &contents);
    *contents.modify_offest_surface_mode() = true;
    assert!(*contents.modify_offset_surface_mode());
}
#[test]
fn shape_contents_free_topology_matches_native() {
    let builder = BRepBuilder::default();
    let (mut wire, mut edge) = (Wire::default(), Edge::default());
    builder.make_wire(&mut wire);
    builder.make_edge(&mut edge).unwrap();
    builder.add(&mut wire, &edge).unwrap();
    let mut contents = ShapeAnalysisShapeContents::new();
    contents.perform(&wire).unwrap();
    assert_native("free-wire", &contents);
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut missing = fixture::face(&plane);
    builder.add(&mut missing, &wire).unwrap();
    contents.perform(&missing).unwrap();
    assert_native("missing-pcurve", &contents);
}

#[test]
fn shape_contents_bezier_surface_matches_native() {
    use rustycad_tkernel::ncollection::Array2;
    use rustycad_tkg3d::geom::GeomBezierSurface;
    use rustycad_tkmath::gp::Point3;
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = Point3::new(0., 0., 0.);
    poles[(2, 1)] = Point3::new(1., 0., 0.);
    poles[(1, 2)] = Point3::new(0., 1., 0.);
    poles[(2, 2)] = Point3::new(1., 1., 0.);
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    let mut contents = ShapeAnalysisShapeContents::new();
    fixture::modes(&mut contents);
    contents.perform(&fixture::face(&surface)).unwrap();
    assert_native("bezier-surface", &contents);
}
