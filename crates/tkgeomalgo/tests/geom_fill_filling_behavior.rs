//! Behavior of the GeomFill filling schemes and curve fillings (`GeomFill_Coons`,
//! `GeomFill_Stretch`, `GeomFill_Curved`, `GeomFill_Generator`,
//! `GeomFill_BezierCurves`, `GeomFill_BSplineCurves`) beyond the numeric probe.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomBoundedCurve, GeomCurve, GeomSurface,
};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillBSplineCurves, GeomFillBezierCurves, GeomFillCoons, GeomFillCurved,
    GeomFillFillingStyle, GeomFillGenerator, GeomFillStretch,
};
use rustycad_tkmath::gp::Point3;

fn row(points: &[(f64, f64, f64)]) -> Array1<'static, Point3> {
    let mut row = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (k, &(x, y, z)) in points.iter().enumerate() {
        row[k as i32 + 1] = Point3::new(x, y, z);
    }
    row
}

fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut row = Array1::with_bounds(1, values.len() as i32).unwrap();
    row.as_mut_slice().copy_from_slice(values);
    row
}

/// The rows of the flat square [0, 3] x [0, 3], three poles per side.
fn square() -> [Array1<'static, Point3>; 4] {
    [
        row(&[(0., 0., 0.), (1.5, 0., 0.), (3., 0., 0.)]),
        row(&[(3., 0., 0.), (3., 1.5, 0.), (3., 3., 0.)]),
        row(&[(0., 3., 0.), (1.5, 3., 0.), (3., 3., 0.)]),
        row(&[(0., 0., 0.), (0., 1.5, 0.), (0., 3., 0.)]),
    ]
}

#[test]
fn stretch_of_a_flat_square_is_its_bilinear_patch() -> Result<(), Failure> {
    let [p1, p2, p3, p4] = square();
    let stretch = GeomFillStretch::with_poles([&p1, &p2, &p3, &p4])?;
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(0, 0, 0, 0).unwrap();
    stretch.poles(&mut poles)?;
    assert_eq!(
        (poles.lower_row(), poles.col_length(), poles.row_length()),
        (1, 3, 3)
    );
    // P4 is the first row and P2 the last in the stretch layout.
    assert_eq!(poles[(1, 2)], Point3::new(0., 1.5, 0.));
    assert_eq!(poles[(2, 2)], Point3::new(1.5, 1.5, 0.));
    Ok(())
}

#[test]
fn mismatched_rows_raise_domain_error() {
    let [p1, p2, p3, _] = square();
    let short = row(&[(0., 0., 0.), (0., 3., 0.)]);
    let error = GeomFillCoons::with_poles([&p1, &p2, &p3, &short])
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    let empty = GeomFillCoons::new();
    assert_eq!(
        empty.nb_u_poles().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
}

/// A second, non-rational Init keeps the weights of the first (native behaviour).
#[test]
fn non_rational_init_keeps_the_weights() -> Result<(), Failure> {
    let [p1, p2, p3, p4] = square();
    let w = [
        reals(&[1., 2., 1.]),
        reals(&[1., 3., 1.]),
        reals(&[1., 2., 1.]),
        reals(&[1., 3., 1.]),
    ];
    let mut curved =
        GeomFillCurved::with_weights([&p1, &p2, &p3, &p4], [&w[0], &w[1], &w[2], &w[3]])?;
    assert!(curved.is_rational());
    curved.init([&p1, &p2, &p3, &p4])?;
    assert!(!curved.is_rational());
    let mut weights: Array2<'static, f64> = Array2::with_bounds(1, 1, 1, 1).unwrap();
    curved.weights(&mut weights)?;
    assert_eq!(weights.col_length(), 3);
    Ok(())
}

#[test]
fn generator_rules_two_sections() -> Result<(), Failure> {
    let mut generator = GeomFillGenerator::new();
    assert!(generator.surface().is_null());
    for z in [0.0, 2.0] {
        let poles = row(&[(0., 0., z), (1., 1., z), (2., 0., z)]);
        let knots = reals(&[0., 1.]);
        let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
        mults.init(3);
        let curve = GeomBSplineCurve::new(&poles, &knots, &mults, 2, false)?;
        generator.add_curve(&Handle::new(curve).cast())?;
    }
    generator.perform(1e-9)?;
    assert!(!generator.surface().is_null());
    Ok(())
}

fn bezier(points: &[(f64, f64, f64)]) -> Handle<GeomBezierCurve> {
    Handle::new(GeomBezierCurve::new(&row(points)).unwrap())
}

/// In the curved style, when the end of the first curve meets the start of the
/// second, the caller's first curve is reversed (native quirk), not its copy.
#[test]
fn curved_two_bezier_curves_reverse_the_callers_first_curve() -> Result<(), Failure> {
    let c1 = bezier(&[(2., 0., 0.), (1., 1., 0.), (0., 0., 0.)]);
    let c2 = bezier(&[(0., 0., 0.), (0., 2., 1.)]);
    GeomFillBezierCurves::with_two(&c1, &c2, GeomFillFillingStyle::CurvedStyle)?;
    assert_eq!(c1.start_point()?, Point3::new(0., 0., 0.));
    let c3 = bezier(&[(5., 5., 5.), (6., 5., 5.)]);
    let error = GeomFillBezierCurves::with_two(&c3, &c2, GeomFillFillingStyle::CurvedStyle)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    Ok(())
}

#[test]
fn unjoined_bezier_curves_raise_construction_error() {
    let c1 = bezier(&[(0., 0., 0.), (1., 0., 0.)]);
    let c2 = bezier(&[(1., 0., 0.), (1., 1., 0.)]);
    let c3 = bezier(&[(9., 9., 9.), (8., 9., 9.)]);
    let c4 = bezier(&[(0., 1., 0.), (0., 0., 0.)]);
    let error =
        GeomFillBezierCurves::with_four(&c1, &c2, &c3, &c4, GeomFillFillingStyle::StretchStyle)
            .err()
            .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

fn two_pole(a: (f64, f64, f64), b: (f64, f64, f64), last: f64) -> Handle<GeomBSplineCurve> {
    let (poles, weights, knots) = (row(&[a, b]), reals(&[1., 1.]), reals(&[0., last]));
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    mults.init(2);
    Handle::new(GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, 1, false).unwrap())
}

/// The OCC27704 case of GeomFill_BSplineCurves_Test.cxx (bug27775): two-pole
/// curves cannot give a Coons filling.
#[test]
fn occ27704_two_pole_curves_raise_construction_error() {
    let p1 = (-24033.3957701043, -6337.90755953146, -16577.8188547128);
    let p2 = (-23933.3957701044, -6337.90755953146, -16577.8362547128);
    let p3 = (-24033.3957701043, -6371.01755953146, -16577.8188547128);
    let p4 = (-23933.3957701044, -6371.01755953146, -16577.8362547128);
    let c1 = two_pole(p1, p2, 100.000001513789);
    let c2 = two_pole(p2, p1, 100.000001513789);
    let c3 = two_pole(p3, p1, 33.1099999999979);
    let c4 = two_pole(p3, p4, 100.000001513789);
    let error =
        GeomFillBSplineCurves::with_four(&c1, &c2, &c3, &c4, GeomFillFillingStyle::CoonsStyle)
            .err()
            .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn bspline_curves_fill_three_boundaries() -> Result<(), Failure> {
    let edge =
        |a: (f64, f64, f64), m: (f64, f64, f64), b: (f64, f64, f64)| -> Handle<GeomBSplineCurve> {
            let (poles, knots) = (row(&[a, m, m, b]), reals(&[0., 1.]));
            let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
            mults.init(4);
            Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 3, false).unwrap())
        };
    let c1 = edge((0., 0., 0.), (1., -0.2, 0.3), (2., 0., 0.));
    let c2 = edge((2., 0., 0.), (2.2, 1., 0.2), (2., 2., 0.));
    let c3 = edge((2., 2., 0.), (1., 2.3, 0.1), (0., 2., 0.));
    let fill = GeomFillBSplineCurves::with_three(&c1, &c2, &c3, GeomFillFillingStyle::CoonsStyle)?;
    let surface = fill.surface();
    assert_eq!(surface.nb_u_poles(), 4);
    assert!(!surface.is_u_periodic());
    Ok(())
}

/// `Reversed` gives a reversed copy and leaves the curve unchanged.
#[test]
fn bezier_curve_reversal_is_a_copy() -> Result<(), Failure> {
    let c = bezier(&[(0., 0., 0.), (1., 0., 0.)]);
    let reversed = c.reversed()?;
    assert_eq!(c.start_point()?, Point3::new(0., 0., 0.));
    assert_eq!(
        reversed.cast::<GeomBezierCurve>().start_point()?,
        Point3::new(1., 0., 0.)
    );
    Ok(())
}

#[test]
fn coons_needs_matching_weight_rows() {
    let [p1, p2, p3, p4] = square();
    let w = reals(&[1., 1., 1.]);
    let short = reals(&[1., 1.]);
    let error = GeomFillCoons::with_weights([&p1, &p2, &p3, &p4], [&w, &w, &w, &short])
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert!(GeomFillCoons::with_weights([&p1, &p2, &p3, &p4], [&w, &w, &w, &w]).is_ok());
}
