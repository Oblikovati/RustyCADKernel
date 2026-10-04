//! Differential coverage for independent U/V protected segment flags.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomGeometry, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_eval::{RepSurfaceDesc, RepSurfaceDescFull};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn surface() -> GeomBSplineSurface {
    let poles: Array2<'static, Point3> = poles();
    let knots: Array1<'static, f64> = values(&[0.0, 0.25, 0.5, 0.75, 1.0]);
    let mults: Array1<'static, i32> = integers(&[3, 1, 1, 1, 3]);
    GeomBSplineSurface::new(
        &poles,
        (&knots, &mults),
        (&knots, &mults),
        (2, 2),
        (false, false),
    )
    .unwrap()
}

fn poles() -> Array2<'static, Point3> {
    let mut grid: Array2<'static, Point3> = Array2::with_bounds(1, 6, 1, 6).unwrap();
    for row in 1..=6 {
        for column in 1..=6 {
            grid.set_value(
                row,
                column,
                Point3::new(
                    f64::from(row),
                    f64::from(column),
                    f64::from(row * column) * 0.1,
                ),
            )
            .unwrap();
        }
    }
    grid
}

fn values(source: &[f64]) -> Array1<'static, f64> {
    let mut result: Array1<'static, f64> = Array1::with_bounds(1, source.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(source);
    result
}

fn integers(source: &[i32]) -> Array1<'static, i32> {
    let mut result: Array1<'static, i32> = Array1::with_bounds(1, source.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(source);
    result
}

fn plane_representation() -> Handle<dyn RepSurfaceDesc> {
    let frame =
        Axis3::from_normal(Point3::default(), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    Handle::new(RepSurfaceDescFull::new(plane)).cast()
}

fn assert_case(flags: (bool, bool), expected: (i32, i32)) {
    let surface = surface();
    let representation = plane_representation();
    surface
        .set_eval_representation(representation.clone())
        .unwrap();
    assert!(surface.has_eval_representation());
    surface.resolution(1.0).unwrap();
    assert_cache_ready(&surface);
    surface
        .segment_with_flags((0.25, 0.75), (0.25, 0.75), (1.0e-12, 1.0e-12), flags)
        .unwrap();
    assert_case_output(&surface, flags, expected);
}

fn assert_case_output(surface: &GeomBSplineSurface, flags: (bool, bool), expected: (i32, i32)) {
    let expected_u_bounds = (
        if flags.0 { 0.25 } else { 0.75 },
        if flags.0 { 0.75 } else { 0.25 },
    );
    let expected_v_bounds = (
        if flags.1 { 0.25 } else { 0.75 },
        if flags.1 { 0.75 } else { 0.25 },
    );
    let (u1, u2, v1, v2) = surface.bounds();
    assert_eq!((u1, u2), expected_u_bounds);
    assert_eq!((v1, v2), expected_v_bounds);
    assert_eq!((surface.nb_u_poles(), surface.nb_v_poles()), expected);
    assert_eq!(surface.u_knots().as_slice(), &[0.25, 0.5, 0.75]);
    assert_eq!(surface.v_knots().as_slice(), &[0.25, 0.5, 0.75]);
    assert_eq!(surface.u_multiplicities().as_slice(), flag_mults(flags.0));
    assert_eq!(surface.v_multiplicities().as_slice(), flag_mults(flags.1));
    assert!(!surface.has_eval_representation());
    assert_cache_invalid(surface);
}

fn flag_mults(segment: bool) -> &'static [i32] {
    if segment { &[3, 1, 3] } else { &[1, 1, 1] }
}

fn assert_cache_invalid(surface: &GeomBSplineSurface) {
    let mut stream: DumpStream = DumpStream::default();
    surface.dump_json(&mut stream, -1).unwrap();
    assert!(stream.text().contains("\"MaxDerivInvOk\": 0"));
}

fn assert_cache_ready(surface: &GeomBSplineSurface) {
    let mut stream: DumpStream = DumpStream::default();
    surface.dump_json(&mut stream, -1).unwrap();
    assert!(stream.text().contains("\"MaxDerivInvOk\": 1"));
}

#[test]
fn segment_flags_apply_endpoint_multiplicities_independently() {
    for (flags, counts) in [
        ((false, false), (3, 3)),
        ((false, true), (3, 4)),
        ((true, false), (4, 3)),
        ((true, true), (4, 4)),
    ] {
        assert_case(flags, counts);
    }
}
