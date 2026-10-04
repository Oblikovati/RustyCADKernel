//! Public `Perform` comparisons for curved localized faces and shared-context reuse.
use crate::int_tools::BeanFaceIntersector;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface, GeomSurface};
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

#[test]
fn public_perform_matches_curved_surface_native_ranges_and_repeat_calls() {
    for surface in ["bezier", "bspline", "bezier_curved", "bspline_curved"] {
        for located in [false, true] {
            assert_public_case(surface, located);
        }
    }
}

fn assert_public_case(surface: &str, located: bool) {
    let name = case_name(surface, located);
    let edge = crossing_edge(located);
    let face = curved_face(surface, located);
    let mut solver = BeanFaceIntersector::from_shapes(&edge, &face).unwrap();
    solver.set_bean_parameters(0.0, 2.0);
    solver.set_surface_parameters(0.0, 1.0, 0.0, 1.0);
    for pass in 0..2 {
        solver.perform().unwrap();
        assert_native_result(&solver, &name, pass);
    }
}

fn assert_native_result(solver: &BeanFaceIntersector, name: &str, pass: usize) {
    assert!(solver.is_done(), "{name} pass {pass}");
    let expected = native_result(name, pass);
    assert_eq!(
        solver.result().length() as usize,
        expected.len(),
        "{name} pass {pass}"
    );
    for (index, (first, last)) in expected.into_iter().enumerate() {
        let actual = solver.result().iter().nth(index).unwrap();
        assert!((actual.first() - first).abs() <= 1e-12, "{name} first");
        assert!((actual.last() - last).abs() <= 1e-12, "{name} last");
    }
}

fn native_result(name: &str, pass: usize) -> Vec<(f64, f64)> {
    include_str!("../../../../../../tools/oracle/tkbo_bean_localized_perform.expected")
        .lines()
        .filter_map(|line| parse_native_result(line, name, pass))
        .collect()
}

fn parse_native_result(line: &str, name: &str, pass: usize) -> Option<(f64, f64)> {
    let mut fields = line.split_whitespace();
    if fields.next()? != name || fields.next()?.parse::<usize>().ok()? != pass {
        return None;
    }
    if fields.next()?.parse::<i32>().ok()? != 1 || fields.next()?.parse::<usize>().ok()? != 1 {
        return None;
    }
    Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?))
}

fn case_name(surface: &str, located: bool) -> String {
    if located {
        format!("{surface}_located")
    } else {
        surface.to_owned()
    }
}

fn crossing_edge(located: bool) -> Edge {
    let offset = if located {
        Vector3::new(10.0, -4.0, 3.0)
    } else {
        Vector3::default()
    };
    let first = Point3::new(0.5, 0.5, -1.0).translated(offset);
    let last = Point3::new(0.5, 0.5, 1.0).translated(offset);
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}

fn curved_face(surface: &str, located: bool) -> Face {
    let geometry = match surface {
        "bezier" => bezier_surface(),
        "bspline" => bspline_surface(),
        "bezier_curved" => curved_bezier_surface(),
        _ => curved_bspline_surface(),
    };
    let mut face =
        BRepBuilderAPIMakeFace::from_surface_bounds(&geometry, (0.0, 1.0, 0.0, 1.0), 1e-7)
            .unwrap()
            .face()
            .unwrap();
    if located {
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(10.0, -4.0, 3.0));
        face.set_location(&Location::new(transform), true).unwrap();
    }
    face
}

fn bezier_surface() -> rustycad_tkernel::standard::Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    poles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    poles[(1, 2)] = Point3::new(0.0, 1.0, 0.0);
    poles[(2, 2)] = Point3::new(1.0, 1.0, 0.0);
    rustycad_tkernel::standard::Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}

fn bspline_surface() -> rustycad_tkernel::standard::Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 6, 1, 6).unwrap();
    for u in 1..=6 {
        for v in 1..=6 {
            let x = f64::from(u - 1) / 5.0;
            let y = f64::from(v - 1) / 5.0;
            poles[(u, v)] = Point3::new(x, y, 0.08 * x * y);
        }
    }
    let (knots, multiplicities) = cubic_knots();
    let spline = GeomBSplineSurface::new(
        &poles,
        (&knots, &multiplicities),
        (&knots, &multiplicities),
        (3, 3),
        (false, false),
    )
    .unwrap();
    rustycad_tkernel::standard::Handle::new(spline).cast()
}

fn curved_bezier_surface() -> rustycad_tkernel::standard::Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for u in 1..=3 {
        for v in 1..=3 {
            let x = f64::from(u - 1) / 2.0;
            let y = f64::from(v - 1) / 2.0;
            let z = if u == 2 { 0.3 } else { 0.0 };
            poles[(u, v)] = Point3::new(x, y, z);
        }
    }
    rustycad_tkernel::standard::Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}

fn curved_bspline_surface() -> rustycad_tkernel::standard::Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 6, 1, 6).unwrap();
    for u in 1..=6 {
        for v in 1..=6 {
            let x = f64::from(u - 1) / 5.0;
            let y = f64::from(v - 1) / 5.0;
            let z = 0.24 * x * y + 0.12 * x * x + 0.08 * y * y;
            poles[(u, v)] = Point3::new(x, y, z);
        }
    }
    let (knots, multiplicities) = cubic_knots();
    let spline = GeomBSplineSurface::new(
        &poles,
        (&knots, &multiplicities),
        (&knots, &multiplicities),
        (3, 3),
        (false, false),
    )
    .unwrap();
    rustycad_tkernel::standard::Handle::new(spline).cast()
}

fn cubic_knots() -> (
    rustycad_tkernel::ncollection::Array1<'static, f64>,
    rustycad_tkernel::ncollection::Array1<'static, i32>,
) {
    let mut knots = rustycad_tkernel::ncollection::Array1::with_bounds(1, 3).unwrap();
    let mut multiplicities = rustycad_tkernel::ncollection::Array1::with_bounds(1, 3).unwrap();
    for (index, value) in [0.0, 0.5, 1.0].into_iter().enumerate() {
        knots[(index as i32) + 1] = value;
    }
    for (index, value) in [4, 2, 4].into_iter().enumerate() {
        multiplicities[(index as i32) + 1] = value;
    }
    (knots, multiplicities)
}
