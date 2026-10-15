//! The swept shapes of the local operations, checked against closed forms: a prism of a
//! square is a box, a revolution of a rectangle around an axis a tube sector of volume
//! `angle (R^2 - r^2) h / 2`, a drafted prism a frustum, a pipe along a line a box.
mod support;

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Face, Shape, Wire};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkfeat::loc_ope::{DPrism, LinearForm, Pipe, Prism, Revol, RevolutionForm};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};
use std::f64::consts::PI;
use support::{assert_close, count, face_centre, is_valid, sub_shapes, volume};

/// The closed polygon through `points`.
fn polygon(points: &[(f64, f64, f64)]) -> Wire {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for p in points {
        poly.add_point(Point3::new(p.0, p.1, p.2)).unwrap();
    }
    poly.close().unwrap();
    poly.wire().unwrap()
}

/// The planar face bounded by the closed polygon through `points`.
fn polygon_face(points: &[(f64, f64, f64)]) -> Face {
    BRepBuilderAPIMakeFace::from_wire(&polygon(points), true)
        .unwrap()
        .face()
        .unwrap()
}

fn square() -> Face {
    polygon_face(&[(0., 0., 0.), (10., 0., 0.), (10., 10., 0.), (0., 10., 0.)])
}

fn z_axis() -> Axis1 {
    Axis1::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
}

#[test]
fn prism_of_a_square_is_a_box_with_its_side_faces_mapped() {
    let base = square();
    let prism = Prism::with_vector(&base, Vector3::new(0., 0., 5.)).unwrap();
    let res = prism.shape().unwrap();
    assert_close(volume(res), 500., 1e-9);
    assert!(is_valid(res));
    for e in sub_shapes(&base, ShapeEnum::Edge) {
        assert_eq!(prism.shapes(&e).unwrap().len(), 1);
    }
    assert!(prism.shapes(res).is_err());
    assert_close(
        face_centre(&topods::face(prism.last_shape()).unwrap()).z(),
        5.,
        1e-9,
    );
    let mut curves = Sequence::default();
    prism.curves(&mut curves).unwrap();
    assert_eq!(curves.len(), 4 * 9 + 4);
    let baryc = prism.baryc_curve().unwrap();
    let p = baryc.get().unwrap().value(0.).unwrap();
    assert_close(p.x(), 5., 1e-9);
    assert_close(p.y(), 5., 1e-9);
}

#[test]
fn prism_with_translation_maps_the_original_edges() {
    let base = square();
    let prism =
        Prism::with_translation(&base, Vector3::new(0., 0., 5.), Vector3::new(0., 0., 2.)).unwrap();
    let res = prism.shape().unwrap();
    assert_close(volume(res), 500., 1e-9);
    assert_close(
        face_centre(&topods::face(prism.first_shape()).unwrap()).z(),
        2.,
        1e-9,
    );
    for e in sub_shapes(&base, ShapeEnum::Edge) {
        assert_eq!(prism.shapes(&e).unwrap().len(), 1);
    }
}

#[test]
fn prism_of_a_wire_is_an_open_shell_of_four_faces() {
    let wire = polygon(&[(0., 0., 0.), (10., 0., 0.), (10., 10., 0.), (0., 10., 0.)]);
    let prism = Prism::with_vector(&wire.into_shape(), Vector3::new(0., 0., 5.)).unwrap();
    assert_eq!(count(prism.shape().unwrap(), ShapeEnum::Face), 4);
    assert_close(support::area(prism.shape().unwrap()), 200., 1e-9);
}

#[test]
fn linear_form_sweeps_like_the_prism() {
    let base = square();
    let form = LinearForm::with_vector(
        &base,
        Vector3::new(0., 0., 3.),
        Point3::new(0., 0., 0.),
        Point3::new(0., 0., 3.),
    )
    .unwrap();
    assert_close(volume(form.shape().unwrap()), 300., 1e-9);
    assert!(LinearForm::new().shape().is_err());
}

/// The rectangle `[2, 4] x [0, 1]` in the XZ plane.
fn rectangle() -> Face {
    polygon_face(&[(2., 0., 0.), (4., 0., 0.), (4., 0., 1.), (2., 0., 1.)])
}

#[test]
fn revol_of_a_rectangle_is_a_tube_sector() {
    let revol = Revol::with_axis(&rectangle(), z_axis(), PI / 2.).unwrap();
    let res = revol.shape().unwrap();
    // angle (R^2 - r^2) h / 2 = (pi / 2) (16 - 4) / 2
    assert_close(volume(res), 3. * PI, 1e-9);
    assert!(is_valid(res));
    let mut curves = Sequence::default();
    revol.curves(&mut curves).unwrap();
    assert_eq!(curves.len(), 4 * 9 + 4);
    let baryc = revol.baryc_curve().unwrap();
    let c = baryc.get().unwrap();
    let p = c.value(0.).unwrap();
    assert_close((p.x() * p.x() + p.y() * p.y()).sqrt(), 3., 1e-9);
}

#[test]
fn revol_with_offset_angle_starts_turned() {
    let revol = Revol::with_offset_angle(&rectangle(), z_axis(), PI / 2., PI / 2.).unwrap();
    assert_close(volume(revol.shape().unwrap()), 3. * PI, 1e-9);
    let first = face_centre(&topods::face(revol.first_shape()).unwrap());
    assert_close(first.x(), 0., 1e-9);
    assert_close(first.y(), 3., 1e-9);
}

#[test]
fn revolution_form_turns_a_full_tube() {
    let form = RevolutionForm::with_axis(&rectangle(), z_axis(), 2. * PI).unwrap();
    assert_close(volume(form.shape().unwrap()), 12. * PI, 1e-9);
}

#[test]
fn dprism_without_draft_is_a_box() {
    let dprism = DPrism::with_height(&square(), 5., 0.).unwrap();
    assert!(dprism.is_done());
    let res = dprism.shape().unwrap();
    assert_close(volume(res), 500., 1e-7);
    assert!(is_valid(res));
    let mut curves = Sequence::default();
    dprism.curves(&mut curves).unwrap();
    assert_eq!(curves.len(), 4 * 8);
}

#[test]
fn dprism_with_draft_is_a_frustum() {
    let (h, a) = (5_f64, 10_f64.to_radians());
    let dprism = DPrism::with_height(&square(), h, a).unwrap();
    let res = dprism.shape().unwrap();
    let (height, t) = (h * a.cos(), a.tan());
    // The section shrinks by t z on each side: (10 - 2 t z)^2 integrated over the height.
    let inward = 100. * height - 20. * t * height * height + 4. / 3. * t * t * height.powi(3);
    assert_close(volume(res), inward, 1e-6);
    assert!(is_valid(res));
}

#[test]
fn pipe_of_a_square_along_a_line_is_a_box() {
    let profile = polygon_face(&[(0., 0., 0.), (2., 0., 0.), (2., 2., 0.), (0., 2., 0.)]);
    let b = BRepBuilder::default();
    let mut spine = Wire::default();
    b.make_wire(&mut spine);
    let edge = rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_points(
        Point3::new(1., 1., 0.),
        Point3::new(1., 1., 10.),
    )
    .unwrap()
    .edge()
    .unwrap();
    b.add(&mut spine, &edge).unwrap();
    let mut pipe = Pipe::new(&spine, profile.as_ref()).unwrap();
    assert_close(volume(pipe.shape()), 40., 1e-7);
    assert!(is_valid(pipe.shape()));
    for e in sub_shapes(&profile, ShapeEnum::Edge) {
        assert_eq!(pipe.shapes(&e).unwrap().len(), 1);
    }
    let v = sub_shapes(&profile, ShapeEnum::Vertex).remove(0);
    assert_eq!(pipe.shapes(&v).unwrap().len(), 1);
    assert_eq!(
        pipe.shapes(&Shape::default())
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );
    // The pipe line of a straight spine is an untrimmed Geom_Line, which
    // GeomConvert::CurveToBSplineCurve refuses (as natively).
    let mut points = Sequence::default();
    points.append(Point3::new(1., 1., 0.));
    assert_eq!(
        pipe.curves(&points).err().unwrap().exception_type(),
        "Standard_DomainError"
    );
}
