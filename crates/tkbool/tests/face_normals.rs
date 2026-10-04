//! Pinned normal vectors, oriented directions, output retention and native exceptions.
use rustycad_tkbool::top_ope_brep_tool::{min_duv, nggeom_f, nt};
use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
use rustycad_tkernel::{
    ncollection::Array2,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomBezierSurface, GeomConicalSurface, GeomPlane, GeomSphericalSurface, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use std::f64::consts::{FRAC_PI_2, PI};
#[test]
fn native_face_normals_match_analytic_singular_and_failed_cases() {
    for line in include_str!("../../../tools/oracle/tkbool_face_normals.expected").lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|field| field.parse().unwrap())
            .collect();
        let actual = observe(
            expected[0] as i32,
            expected[1] as usize,
            expected[2] as usize,
        );
        for (index, (actual, expected)) in actual.iter().zip(&expected[3..]).enumerate() {
            assert!(
                (actual - expected).abs() < 1e-12,
                "{line}: field {index}, {actual} != {expected}"
            );
        }
    }
}
fn observe(kind: i32, orientation: usize, point: usize) -> [f64; 9] {
    let mut face = fixture(kind);
    face.set_orientation(
        [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ][orientation],
    );
    let points = [
        Point2::new(0.3, 0.2),
        Point2::new(0., 0.),
        Point2::new(0.4, FRAC_PI_2),
        Point2::new(0.4, -FRAC_PI_2),
    ];
    let uv = if kind == 2 && point == 2 {
        Point2::new(0.4, 0.)
    } else {
        points[point]
    };
    let mut vector = Vector3::new(7., 8., 9.);
    let mut normal = Direction3::new(1., 0., 0.).unwrap();
    let a = normal_status(nggeom_f(uv, &face, &mut vector));
    let b = normal_status(nt(uv, &face, &mut normal));
    [
        a,
        b,
        min_duv(&face).unwrap(),
        vector.x(),
        vector.y(),
        vector.z(),
        normal.x(),
        normal.y(),
        normal.z(),
    ]
}
fn normal_status(result: Result<bool, Failure>) -> f64 {
    match result {
        Ok(done) => f64::from(done),
        Err(failure) => {
            assert_eq!(failure.exception_type(), "Standard_ConstructionError");
            -1.
        }
    }
}
fn fixture(kind: i32) -> Face {
    if kind >= 3 {
        return degenerate_fixture(kind);
    }
    let surface: Handle<dyn GeomSurface> = match kind {
        0 => Handle::new(GeomPlane::new(Axis3::default())).cast(),
        1 => Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.).unwrap()).cast(),
        _ => Handle::new(GeomConicalSurface::new(Axis3::default(), 0.35, 0.).unwrap()).cast(),
    };
    let (first, last) = if kind == 1 {
        (-FRAC_PI_2, FRAC_PI_2)
    } else {
        (-1., 1.)
    };
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0., 2. * PI, first, last), 1e-7)
        .unwrap()
        .face()
        .unwrap()
}
fn degenerate_fixture(kind: i32) -> Face {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for i in 1..=2 {
        for j in 1..=2 {
            *poles.change_value(i, j).unwrap() = if kind == 3 {
                Point3::default()
            } else {
                Point3::new(f64::from(i) * 1e-9, f64::from(j) * 1e-9, 0.)
            };
        }
    }
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
