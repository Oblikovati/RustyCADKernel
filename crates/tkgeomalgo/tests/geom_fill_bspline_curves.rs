//! Translation of the pinned GeomFill_BSplineCurves_Test.cxx suite.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface};
use rustycad_tkgeomalgo::app_blend::AppBlendApprox;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillAppSurf, GeomFillBSplineCurves, GeomFillFillingStyle, GeomFillLine,
    GeomFillSectionGenerator,
};
#[path = "support/geom_fill_sections.rs"]
mod geom_fill_sections;
use geom_fill_sections::circle_and_ellipse;
#[path = "support/geom_fill_rows.rs"]
mod geom_fill_rows;
use geom_fill_rows::{reals, row};

/// The approximation path of the DRAW test bug13904 (the OCC13904 case of
/// GeomFill_BSplineCurves_Test.cxx): a circle and an ellipse skinned by
/// SectionGenerator -> AppSurf, whose result builds a B-spline surface.
#[test]
fn app_surf_between_circle_and_ellipse() -> Result<(), Failure> {
    let mut section = GeomFillSectionGenerator::new();
    for curve in circle_and_ellipse()? {
        section.add_curve(&curve)?;
    }
    section.perform(precision::p_confusion())?;
    let line = Handle::new(GeomFillLine::with_nb_points(2));
    let mut approx = GeomFillAppSurf::new(
        3,
        8,
        precision::confusion(),
        precision::p_confusion(),
        0,
        false,
    );
    approx.perform(line.get(), &mut section, false)?;
    assert!(approx.is_done());
    let [
        u_degree,
        v_degree,
        nb_u_poles,
        nb_v_poles,
        nb_u_knots,
        nb_v_knots,
    ] = approx.surf_shape()?;
    assert!(u_degree >= 3 && v_degree >= 1);
    assert!(nb_u_poles > 0 && nb_v_poles > 0 && nb_u_knots > 0 && nb_v_knots > 0);
    let surface = GeomBSplineSurface::with_weights(
        approx.surf_poles()?,
        approx.surf_weights()?,
        (approx.surf_u_knots()?, approx.surf_u_mults()?),
        (approx.surf_v_knots()?, approx.surf_v_mults()?),
        (approx.u_degree()?, approx.v_degree()?),
        (false, false),
    );
    assert!(surface.is_ok());
    Ok(())
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
