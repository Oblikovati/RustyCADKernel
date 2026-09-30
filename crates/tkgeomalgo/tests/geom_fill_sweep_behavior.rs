//! Behavior tests of the GeomFill sweep classes (docs/geomalgo-design.md, section 14):
//! the elementary sweeps, the pipes, the section placement and the N-section law.
//! The numeric probes compare the same classes with OCCT bit for bit.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomCartesianPoint, GeomCircle, GeomCurve, GeomLine, GeomRectangularTrimmedSurface,
    GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_fill::*;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};

fn segment(origin: Point3, direction: Direction3, length: f64) -> Handle<dyn GeomCurve> {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(origin, direction)).cast();
    Handle::new(GeomTrimmedCurve::new(&line, 0.0, length, true, true).unwrap()).cast()
}

fn circle(centre: Point3, radius: f64) -> Handle<dyn GeomCurve> {
    Handle::new(GeomCircle::new(Axis2::new(centre, Direction3::Z).unwrap(), radius).unwrap()).cast()
}

fn fixed_law_on_z(path: &Handle<dyn GeomCurve>) -> Handle<dyn GeomFillLocationLaw> {
    let fixed =
        GeomFillFixed::new(Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)).unwrap();
    let trihedron: Handle<dyn GeomFillTrihedronLaw> = Handle::new(fixed).cast();
    let law: Handle<dyn GeomFillLocationLaw> =
        Handle::new(GeomFillCurveAndTrihedron::new(&trihedron)).cast();
    let adaptor: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(path).unwrap()).cast();
    law.get().unwrap().set_curve(&adaptor).unwrap();
    law
}

fn basis_name(surface: &Handle<dyn GeomSurface>) -> &'static str {
    let trimmed: Handle<GeomRectangularTrimmedSurface> = surface.cast();
    let basis = trimmed.get().map_or(surface.clone(), |t| t.basis_surface());
    basis.dynamic_type().unwrap().name()
}

#[test]
fn sweep_of_a_normal_circle_along_a_line_is_a_cylinder() {
    let path = segment(Point3::new(0.0, 0.0, 0.0), Direction3::Z, 4.0);
    let section: Handle<dyn GeomFillSectionLaw> = Handle::new(
        GeomFillUniformSection::new(&circle(Point3::new(0.0, 0.0, 0.0), 1.0), 0.0, 1.0).unwrap(),
    )
    .cast();
    let mut sweep = GeomFillSweep::new(&fixed_law_on_z(&path), true).unwrap();
    sweep
        .build(&section, GeomFillApproxStyle::Location, Shape::C2, 10, 30)
        .unwrap();
    assert!(sweep.is_done());
    assert_eq!(basis_name(&sweep.surface()), "Geom_CylindricalSurface");
}

#[test]
fn sweep_without_particular_cases_approximates() {
    let path = segment(Point3::new(0.0, 0.0, 0.0), Direction3::Z, 4.0);
    let section: Handle<dyn GeomFillSectionLaw> = Handle::new(
        GeomFillUniformSection::new(&circle(Point3::new(0.0, 0.0, 0.0), 1.0), 0.0, 1.0).unwrap(),
    )
    .cast();
    let mut sweep = GeomFillSweep::new(&fixed_law_on_z(&path), false).unwrap();
    sweep
        .build(&section, GeomFillApproxStyle::Location, Shape::C2, 10, 30)
        .unwrap();
    assert!(sweep.is_done());
    assert_eq!(basis_name(&sweep.surface()), "Geom_BSplineSurface");
    assert!(sweep.error_on_surface() < 1e-4);
    assert!(sweep.restriction(true).unwrap().get().is_some());
}

#[test]
fn tube_pipe_is_done_with_a_small_error() {
    let path = segment(Point3::new(0.0, 0.0, 0.0), Direction3::Z, 4.0);
    let mut pipe = GeomFillPipe::with_radius(&path, 0.5).unwrap();
    pipe.perform(1e-4, false, Shape::C1, 11, 30).unwrap();
    assert!(pipe.is_done());
    let surface = pipe.surface();
    let s = surface.get().unwrap();
    let (u1, u2, v1, v2) = s.bounds();
    let p = s.value(0.5 * (u1 + u2), 0.5 * (v1 + v2)).unwrap();
    let radial = (p.x() * p.x() + p.y() * p.y()).sqrt();
    assert!((radial - 0.5).abs() < 1e-3);
}

#[test]
fn blend_pipe_between_parallel_lines_is_a_cylinder() {
    let path = segment(Point3::new(0.0, 0.0, 0.0), Direction3::Z, 4.0);
    let c1 = segment(Point3::new(1.0, 0.0, 0.0), Direction3::Z, 4.0);
    let c2 = segment(Point3::new(0.0, 1.0, 0.0), Direction3::Z, 4.0);
    let mut pipe = GeomFillPipe::with_rails(&path, &c1, &c2, 1.0).unwrap();
    pipe.perform(1e-4, false, Shape::C1, 11, 30).unwrap();
    assert!(pipe.is_done());
    assert_eq!(basis_name(&pipe.surface()), "Geom_CylindricalSurface");
}

#[test]
fn section_placement_of_a_point_on_the_path_is_at_distance_zero() {
    let path = segment(Point3::new(0.0, 0.0, 0.0), Direction3::Z, 4.0);
    let point = GeomCartesianPoint::from_pnt(Point3::new(0.0, 0.0, 2.0));
    let mut placement =
        GeomFillSectionPlacement::new(&fixed_law_on_z(&path), &Handle::new(point).cast()).unwrap();
    placement.perform(1e-7).unwrap();
    assert!(placement.is_done());
    assert!(placement.distance() < 1e-12);
    assert!((placement.parameter_on_path() - 2.0).abs() < 1e-9);
}

#[test]
fn nsections_of_coaxial_circles_is_a_conical_law() {
    let sections = [
        circle(Point3::new(0.0, 0.0, 0.0), 1.0),
        circle(Point3::new(0.0, 0.0, 2.0), 2.0),
    ];
    let law = GeomFillNSections::with_params(&sections, &[0.0, 1.0]).unwrap();
    let mut error = -1.0;
    assert!(law.is_conical_law(&mut error).unwrap());
    assert_eq!(error, 0.0);
    let middle = law.circl_section(0.5).unwrap();
    let c: Handle<GeomCircle> = middle.cast();
    assert!((c.get().unwrap().radius() - 1.5).abs() < 1e-12);
}

#[test]
fn nsections_of_one_section_has_no_surface() {
    let sections = [circle(Point3::new(0.0, 0.0, 0.0), 1.0)];
    let law = GeomFillNSections::new(&sections).unwrap();
    assert!(law.bspline_surface().unwrap().is_null());
    assert_eq!(law.section_shape().unwrap(), (0, 0, 0));
}
