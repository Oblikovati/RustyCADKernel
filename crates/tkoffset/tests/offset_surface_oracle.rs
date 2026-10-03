//! Replays the pinned native BRepOffset::Surface oracle with exact finite bits.
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface, GeomElementarySurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkoffset::brep_offset::surface;

#[test]
fn all_native_surface_cases_match() {
    let cases: Vec<&str> = include_str!("fixtures/offset_surface.txt")
        .lines()
        .collect();
    assert_eq!(cases.len(), 131);
    for line in cases {
        compare_native_case(line);
    }
}

fn compare_native_case(line: &str) {
    let fields: Vec<(&str, &str)> = line
        .split_whitespace()
        .map(|field| field.split_once('=').unwrap())
        .collect();
    let input = input_surface(fields[0].1, fields[1].1 == "1");
    let amount: f64 = fields[2].1.parse().unwrap();
    let (result, status) = surface(&input, amount, fields[3].1 == "1").unwrap();
    assert_eq!(format!("{status:?}"), fields[4].1, "{line}");
    assert_eq!(result.dynamic_type().unwrap().name(), fields[5].1, "{line}");
    for (key, expected) in &fields[6..] {
        compare_surface_field(&result, key, expected, line);
    }
}

fn compare_surface_field(result: &Handle<dyn GeomSurface>, key: &str, expected: &str, line: &str) {
    if key == "basis" {
        assert_eq!(basis_name(result), expected, "{line}");
        return;
    }
    if key == "point_error" {
        assert!(result.value(0.3, 0.4).is_err(), "{line}");
        return;
    }
    compare_numbers(&numeric_surface_field(result, key), expected, line);
}

fn numeric_surface_field(result: &Handle<dyn GeomSurface>, key: &str) -> Vec<f64> {
    if key == "bounds" {
        let (u1, u2, v1, v2) = result.bounds();
        return vec![u1, u2, v1, v2];
    }
    if key == "point" {
        let point = result.value(0.3, 0.4).unwrap();
        return vec![point.x(), point.y(), point.z()];
    }
    vec![scalar_surface_field(result, key)]
}

fn scalar_surface_field(result: &Handle<dyn GeomSurface>, key: &str) -> f64 {
    match key {
        "direct" => f64::from(
            result
                .cast::<dyn GeomElementarySurface>()
                .position()
                .direct()
                .unwrap(),
        ),
        "radius" => analytic_radius(result),
        "angle" => result.cast::<GeomConicalSurface>().semi_angle(),
        "major" => result.cast::<GeomToroidalSurface>().major_radius(),
        "minor" => result.cast::<GeomToroidalSurface>().minor_radius(),
        "offset" => result.cast::<GeomOffsetSurface>().offset(),
        _ => panic!("unexpected native field {key}; expected a known numeric surface property"),
    }
}

fn compare_numbers(actual: &[f64], expected: &str, line: &str) {
    let expected: Vec<f64> = expected
        .split(',')
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{line}: {actual} != {expected}"
        );
    }
}

fn analytic_radius(result: &Handle<dyn GeomSurface>) -> f64 {
    match result.dynamic_type().unwrap().name() {
        "Geom_CylindricalSurface" => result.cast::<GeomCylindricalSurface>().radius(),
        "Geom_ConicalSurface" => result.cast::<GeomConicalSurface>().ref_radius(),
        "Geom_SphericalSurface" => result.cast::<GeomSphericalSurface>().radius(),
        kind => panic!("unexpected radius owner {kind}; expected cylinder, cone or sphere"),
    }
}

fn basis_name(result: &Handle<dyn GeomSurface>) -> &'static str {
    let offset = result.cast::<GeomOffsetSurface>();
    if !offset.is_null() {
        return offset.basis_surface().dynamic_type().unwrap().name();
    }
    result
        .cast::<GeomRectangularTrimmedSurface>()
        .basis_surface()
        .dynamic_type()
        .unwrap()
        .name()
}

fn input_surface(kind: &str, direct: bool) -> Handle<dyn GeomSurface> {
    let mut axis = Axis3::default();
    if !direct {
        axis.y_reverse();
    }
    match kind {
        "plane" => Handle::new(GeomPlane::new(axis)).cast(),
        "cylinder" => Handle::new(GeomCylindricalSurface::new(axis, 2.0).unwrap()).cast(),
        "cone" => Handle::new(GeomConicalSurface::new(axis, 0.4, 2.0).unwrap()).cast(),
        "sphere" => Handle::new(GeomSphericalSurface::new(axis, 2.0).unwrap()).cast(),
        "torus" => Handle::new(GeomToroidalSurface::new(axis, 5.0, 2.0).unwrap()).cast(),
        "spindle" => Handle::new(GeomToroidalSurface::new(axis, 2.0, 3.0).unwrap()).cast(),
        "trimmed" => trimmed_cylinder(axis),
        "bezier" => planar_bezier(),
        _ => panic!("unexpected input {kind}; expected a native fixture surface"),
    }
}

fn trimmed_cylinder(axis: Axis3) -> Handle<dyn GeomSurface> {
    let basis: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(axis, 2.0).unwrap()).cast();
    let trimmed =
        GeomRectangularTrimmedSurface::new(&basis, (0.2, 1.0), (-1.0, 2.0), (true, true)).unwrap();
    Handle::new(trimmed).cast()
}

fn planar_bezier() -> Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for u in 1..=2 {
        for v in 1..=2 {
            poles
                .set_value(u, v, Point3::new((u - 1) as f64, (v - 1) as f64, 0.0))
                .unwrap();
        }
    }
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
