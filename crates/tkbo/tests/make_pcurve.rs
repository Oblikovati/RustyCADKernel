//! Native pcurve projection values and tolerance selection.
use rustycad_tkbo::{bop_tools::algo_tools2d, int_tools::Context};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomSurface, GeomSurfaceOfRevolution, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::f64::consts::PI;
struct ProjectionFixture {
    face: Face,
    curve: Handle<dyn GeomCurve>,
    range: (f64, f64),
}
#[test]
fn native_projection_and_tolerance_cases_match() {
    let fixtures: Vec<ProjectionFixture> = (0..5).map(fixture).collect();
    let context = Context::default();
    for row in include_str!("../../../tools/oracle/tkbo_make_pcurve.expected").lines() {
        let fields: Vec<&str> = row.split_whitespace().collect();
        let kind = fields[0].parse::<usize>().unwrap();
        replay(&fields, &fixtures[kind], &context);
    }
}
fn fixture(kind: usize) -> ProjectionFixture {
    let (surface, curve, range, bounds) = geometry(kind);
    let face = BRepBuilderAPIMakeFace::from_surface_bounds(&surface, bounds, 1e-7)
        .unwrap()
        .face()
        .unwrap();
    ProjectionFixture { face, curve, range }
}
type ProjectionGeometry = (
    Handle<dyn GeomSurface>,
    Handle<dyn GeomCurve>,
    (f64, f64),
    (f64, f64, f64, f64),
);
fn geometry(kind: usize) -> ProjectionGeometry {
    match kind {
        0 => (
            Handle::new(GeomPlane::new(Axis3::default())).cast(),
            line(
                Point3::new(-1., 0.5, 0.),
                Direction3::new(1., 0., 0.).unwrap(),
            ),
            (-2., 3.),
            (-2., 2., -2., 2.),
        ),
        1 => (
            Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.).unwrap()).cast(),
            line(
                Point3::new(2., 0., -1.),
                Direction3::new(0., 0., 1.).unwrap(),
            ),
            (-1., 3.),
            (0., 2. * PI, -2., 2.),
        ),
        2 => (
            Handle::new(GeomToroidalSurface::new(Axis3::default(), 5., 2.).unwrap()).cast(),
            Handle::new(GeomCircle::new(Axis2::default(), 7.).unwrap()).cast(),
            (-0.5, 7.),
            (0., 2. * PI, 0., 2. * PI),
        ),
        3 => revolution_geometry(),
        _ => bezier_geometry(),
    }
}
fn line(point: Point3, direction: Direction3) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(point, direction))).cast()
}
fn replay(fields: &[&str], fixture: &ProjectionFixture, context: &Context) {
    assert_eq!(
        fields[4], "ok",
        "unexpected native projection failure: {fields:?}"
    );
    let tolerances = [0., 1e-7, 1e-5, 1e-3, 2.];
    let mut tolerance = tolerances[fields[1].parse::<usize>().unwrap()];
    let cached = if fields[2] == "1" {
        Some(context)
    } else {
        None
    };
    let mut pcurve: Handle<dyn Geom2dCurve> = Handle::default();
    if fields[3] == "1" {
        let trimmed: Handle<dyn GeomCurve> = Handle::new(
            GeomTrimmedCurve::new(
                &fixture.curve,
                fixture.range.0,
                fixture.range.1,
                true,
                false,
            )
            .unwrap(),
        )
        .cast();
        algo_tools2d::make_pcurve_full_curve(
            &fixture.face,
            &trimmed,
            &mut pcurve,
            &mut tolerance,
            cached,
        )
        .unwrap();
    } else {
        algo_tools2d::make_pcurve_on_face(
            &fixture.face,
            &fixture.curve,
            fixture.range,
            &mut pcurve,
            &mut tolerance,
            cached,
        )
        .unwrap();
    }
    assert!(
        (tolerance - fields[5].parse::<f64>().unwrap()).abs() < 1e-14,
        "{fields:?}: {tolerance}"
    );
    assert_eq!(pcurve.first_parameter(), fields[6].parse::<f64>().unwrap());
    assert_eq!(pcurve.last_parameter(), fields[7].parse::<f64>().unwrap());
    let (first, last) = fixture.range;
    for (index, parameter) in [first, 0.5 * (first + last), last].into_iter().enumerate() {
        let expected = Point2::new(
            fields[8 + 2 * index].parse().unwrap(),
            fields[9 + 2 * index].parse().unwrap(),
        );
        assert!(
            pcurve.value(parameter).unwrap().distance(expected) < 1e-12,
            "{fields:?}"
        );
    }
}

fn revolution_geometry() -> ProjectionGeometry {
    let curve = line(
        Point3::new(2., 0., -1.),
        Direction3::new(0., 0., 1.).unwrap(),
    );
    let surface = Handle::new(
        GeomSurfaceOfRevolution::new(
            &curve,
            Axis1::new(Point3::default(), Direction3::new(0., 0., 1.).unwrap()),
        )
        .unwrap(),
    )
    .cast();
    (surface, curve, (-1., 3.), (0., 2. * PI, -2., 2.))
}
fn bezier_geometry() -> ProjectionGeometry {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for u in 1..=2 {
        for v in 1..=2 {
            *poles.change_value(u, v).unwrap() =
                Point3::new(f64::from(u - 1), f64::from(v - 1), 0.);
        }
    }
    let surface = Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    (
        surface,
        line(
            Point3::new(0., 0.5, 0.),
            Direction3::new(1., 0., 0.).unwrap(),
        ),
        (0., 1.),
        (0., 1., 0., 1.),
    )
}
mod edge_pcurve_cases;
