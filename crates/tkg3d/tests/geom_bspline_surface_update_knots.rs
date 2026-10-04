//! Direct behavioral coverage for protected `Geom_BSplineSurface` knot refreshes.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomGeometry, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_eval::{RepSurfaceDesc, RepSurfaceDescFull};
use rustycad_tkmath::geom_abs::BSplKnotDistribution;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn surface() -> GeomBSplineSurface {
    let poles = poles();
    let u_knots = values(&[0.0, 0.25, 0.75, 1.0]);
    let u_mults = integers(&[3, 1, 2, 3]);
    let v_knots = values(&[0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
    let v_mults = integers(&[1, 1, 1, 1, 1, 1]);
    GeomBSplineSurface::new(
        &poles,
        (&u_knots, &u_mults),
        (&v_knots, &v_mults),
        (2, 3),
        (false, true),
    )
    .unwrap()
}

fn poles() -> Array2<'static, Point3> {
    let mut grid = Array2::with_bounds(1, 6, 1, 5).unwrap();
    for row in 1..=6 {
        for column in 1..=5 {
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

fn values(input: &[f64]) -> Array1<'static, f64> {
    let mut output = Array1::with_bounds(1, input.len() as i32).unwrap();
    output.as_mut_slice().copy_from_slice(input);
    output
}

fn integers(input: &[i32]) -> Array1<'static, i32> {
    let mut output = Array1::with_bounds(1, input.len() as i32).unwrap();
    output.as_mut_slice().copy_from_slice(input);
    output
}

fn plane_representation() -> Handle<dyn RepSurfaceDesc> {
    let frame =
        Axis3::from_normal(Point3::default(), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    Handle::new(RepSurfaceDescFull::new(plane)).cast()
}

fn assert_metadata(
    surface: &GeomBSplineSurface,
    representation: &Handle<dyn RepSurfaceDesc>,
    cache_valid: bool,
) {
    assert_distributions(surface);
    assert_sequences(surface);
    assert_periodicity(surface);
    assert_representation(surface, representation);
    assert_eq!(cache_ready(surface), cache_valid);
    assert_smoothness(surface);
}

fn assert_distributions(surface: &GeomBSplineSurface) {
    assert_eq!(
        surface.u_knot_distribution(),
        BSplKnotDistribution::NonUniform
    );
    assert_eq!(surface.v_knot_distribution(), BSplKnotDistribution::Uniform);
}

fn assert_sequences(surface: &GeomBSplineSurface) {
    assert_eq!(
        surface.u_knot_sequence().as_slice(),
        &[0.0, 0.0, 0.0, 0.25, 0.75, 0.75, 1.0, 1.0, 1.0]
    );
    assert_eq!(
        surface.v_knot_sequence().as_slice(),
        &[
            -0.6,
            -0.4,
            -0.19999999999999996,
            0.0,
            0.2,
            0.4,
            0.6,
            0.8,
            1.0,
            1.2,
            1.4,
            1.6
        ]
    );
}

fn assert_periodicity(surface: &GeomBSplineSurface) {
    assert!(surface.is_v_periodic() && !surface.is_u_periodic());
}

fn assert_representation(
    surface: &GeomBSplineSurface,
    representation: &Handle<dyn RepSurfaceDesc>,
) {
    assert!(
        surface.has_eval_representation()
            && surface.eval_representation() == representation.clone()
    );
}

fn assert_smoothness(surface: &GeomBSplineSurface) {
    let mut stream = DumpStream::default();
    surface.dump_json(&mut stream, -1).unwrap();
    let json = stream.text();
    assert!(json.contains("\"USmooth\": 0"));
    assert!(json.contains("\"VSmooth\": 4"));
}

fn cache_ready(surface: &GeomBSplineSurface) -> bool {
    let mut stream = DumpStream::default();
    surface.dump_json(&mut stream, -1).unwrap();
    stream.text().contains("\"MaxDerivInvOk\": 1")
}

#[test]
fn update_u_knots_recomputes_u_metadata_and_keeps_v_and_eval_rep() {
    let surface = surface();
    let representation = plane_representation();
    surface
        .set_eval_representation(representation.clone())
        .unwrap();
    surface.resolution(1.0).unwrap();
    assert_metadata(&surface, &representation, true);
    surface.update_u_knots().unwrap();
    assert_metadata(&surface, &representation, false);
}

#[test]
fn update_v_knots_recomputes_v_metadata_and_keeps_u_and_eval_rep() {
    let surface = surface();
    let representation = plane_representation();
    surface
        .set_eval_representation(representation.clone())
        .unwrap();
    surface.resolution(1.0).unwrap();
    assert_metadata(&surface, &representation, true);
    surface.update_v_knots().unwrap();
    assert_metadata(&surface, &representation, false);
}
