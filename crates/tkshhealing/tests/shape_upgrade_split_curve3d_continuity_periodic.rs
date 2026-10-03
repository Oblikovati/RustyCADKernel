//! Additional native cases for periodic knot traversal and ordered failed removals.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::{geom_abs::Shape, gp::Point3};
use rustycad_tkshhealing::shape_upgrade::{ShapeUpgradeSplitCurve3dContinuity, SplitCurve};

fn spline(
    poles: &[Point3],
    knots: &[f64],
    mults: &[i32],
    degree: i32,
    periodic: bool,
) -> Handle<dyn GeomCurve> {
    let curve = GeomBSplineCurve::new(
        &Array1::from_shared(poles, 1),
        &Array1::from_shared(knots, 1),
        &Array1::from_shared(mults, 1),
        degree,
        periodic,
    )
    .unwrap();
    Handle::new(curve).cast()
}

fn periodic_polygon() -> Handle<dyn GeomCurve> {
    let poles = [
        Point3::new(1., 0., 0.),
        Point3::new(0.309016994, 0.951056516, 0.),
        Point3::new(-0.809016994, 0.587785252, 0.),
        Point3::new(-0.809016994, -0.587785252, 0.),
        Point3::new(0.309016994, -0.951056516, 0.),
    ];
    spline(&poles, &[0., 1., 2., 3., 4., 5.], &[1; 6], 2, true)
}

fn periodic_triangle() -> Handle<dyn GeomCurve> {
    let poles = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(2., 0., 0.),
    ];
    spline(&poles, &[0., 1., 2., 3.], &[1; 4], 2, true)
}

fn zigzag() -> Handle<dyn GeomCurve> {
    let poles = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 0.),
        Point3::new(2., 1., 0.),
        Point3::new(2., 2., 0.),
    ];
    spline(&poles, &[0., 1., 2., 3., 4.], &[2, 1, 1, 1, 2], 1, false)
}

fn format_values(values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>) -> String {
    values.sequence(|seq| {
        seq.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    })
}

fn oracle_row(label: &str, curve: &Handle<dyn GeomCurve>, tolerance: f64) -> String {
    let splitter = ShapeUpgradeSplitCurve3dContinuity::new();
    splitter.init_curve(curve).unwrap();
    splitter.set_criterion(Shape::CN);
    splitter.set_tolerance(tolerance);
    let shared = splitter.split_values();
    splitter.compute().unwrap();
    let spline = splitter.curve().cast::<GeomBSplineCurve>();
    let (knots, periodic) = spline.get().map_or((-1, 0), |curve| {
        (curve.nb_knots(), i32::from(curve.is_periodic()))
    });
    let status = splitter.split_curve_data().with(|fields| fields.status);
    assert_eq!(
        shared.object_address(),
        splitter.split_values().object_address()
    );
    format!(
        "{label} {status} 1 {knots} {periodic} {}\n",
        format_values(&shared)
    )
}

fn periodic_copy() -> GeomBSplineCurve {
    let source = periodic_polygon().cast::<GeomBSplineCurve>();
    GeomBSplineCurve::copy_of(source.get().unwrap()).unwrap()
}

fn knot_rows(curve: &GeomBSplineCurve) -> String {
    (1..=curve.nb_knots())
        .map(|slot| {
            format!(
                "{}:{}",
                curve.knot(slot).unwrap(),
                curve.multiplicity(slot).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn direct_remove_row(index: i32) -> String {
    let curve = periodic_copy();
    let removed = curve.remove_knot(index, 0, 10.).unwrap();
    format!(
        "direct-remove-{index} {} {} {} {} {} {}\n",
        i32::from(removed),
        curve.nb_knots(),
        curve.nb_poles(),
        curve.first_u_knot_index().unwrap(),
        curve.last_u_knot_index().unwrap(),
        knot_rows(&curve)
    )
}

fn sequential_remove_rows(steps: i32) -> String {
    let curve = periodic_copy();
    let mut output = String::new();
    for step in 1..=steps {
        let removed = curve.remove_knot(2, 0, 10.).unwrap();
        output += &format!(
            "sequential-remove-{step} {} {} {} {}\n",
            i32::from(removed),
            curve.nb_knots(),
            curve.nb_poles(),
            knot_rows(&curve)
        );
    }
    output
}

#[test]
fn periodic_and_multiple_failed_knot_paths_match_native_oracle() {
    let mut actual = oracle_row("periodic-polygon", &periodic_polygon(), 1e-7);
    actual += &oracle_row("periodic-triangle-removal-2", &periodic_triangle(), 2.);
    actual += &oracle_row("multiple-failed-knots", &zigzag(), 1e-7);
    for index in 2..=5 {
        actual += &direct_remove_row(index);
    }
    actual += &sequential_remove_rows(3);
    assert_eq!(
        actual,
        include_str!(
            "../../../tools/oracle/tkshhealing_split_curve3d_continuity_periodic.expected"
        )
    );
}

#[test]
fn periodic_full_removal_reports_native_undefined_domain_gap() {
    let actual = oracle_row("periodic-polygon-full-removal", &periodic_polygon(), 10.);
    assert_eq!(actual, "periodic-polygon-full-removal 1 1 2 1 0 5 5\n");
}
