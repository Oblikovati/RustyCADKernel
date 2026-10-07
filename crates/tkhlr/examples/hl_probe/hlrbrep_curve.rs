//! HLRBRep_Curve, the Rust twin of tools/oracle/tkhlr_probe_hlrbrep_curve.cxx.
use super::common::{Reader, attempt, end, int, pnt, pnt2d, real, show, xy};
use super::hlrbrep_fixtures::{circ2d, edge, elips2d, lin2d, projector};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkhlr::hlr_algo::init_min_max;
use rustycad_tkhlr::hlr_brep::{BCurveTool, Curve};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Vector2};
use std::sync::Arc;

fn curve_box(c: &mut Curve) {
    let (mut mn, mut mx) = ([0.; 16], [0.; 16]);
    init_min_max(1e100, &mut mn, &mut mx);
    attempt(|| {
        real(c.update(&mut mn, &mut mx)?);
        mn.iter().chain(mx.iter()).for_each(|v| real(*v));
        Ok(())
    });
    int(c.get_type() as i32);
    attempt(|| {
        real(c.first_parameter()?);
        real(c.last_parameter()?);
        Ok(())
    });
}

fn curve_at(c: &Curve, t: f64) {
    attempt(|| {
        let p2 = c.parameter_2d(t)?;
        real(p2);
        real(c.parameter_3d(p2)?);
        Ok(())
    });
    show(c.z(t), real);
    attempt(|| {
        let mut p = Point2::new(7., 8.);
        c.d0(t, &mut p)?;
        pnt2d(p);
        Ok(())
    });
    show(c.value(t), pnt2d);
    show(c.value_3d(t), pnt);
    attempt(|| {
        let (mut p, mut v) = (Point2::new(7., 8.), Vector2::new(1., 2.));
        c.d1(t, &mut p, &mut v)?;
        pnt2d(p);
        xy(v.xy());
        Ok(())
    });
    attempt(|| {
        let (mut p, mut v, mut w) = (
            Point2::new(7., 8.),
            Vector2::new(1., 2.),
            Vector2::new(3., 4.),
        );
        c.d2(t, &mut p, &mut v, &mut w)?;
        pnt2d(p);
        xy(v.xy());
        xy(w.xy());
        Ok(())
    });
}

fn curve_conics(c: &Curve) {
    for start in [true, false] {
        attempt(|| {
            let (p, d) = c.tangent(start)?;
            pnt2d(p);
            xy(d.xy());
            Ok(())
        });
    }
    show(c.line(), lin2d);
    show(c.circle(), circ2d);
    show(c.ellipse(), elips2d);
    real(c.hyperbola().major_radius());
    real(c.parabola().focal());
    xy(c.dn(0.5, 1).xy());
}

fn print_poles(poles: &Array1<'_, Point2>, weights: Option<&Array1<'_, f64>>) {
    poles.iter().for_each(|p| pnt2d(*p));
    weights
        .into_iter()
        .flat_map(|w| w.iter())
        .for_each(|x| real(*x));
}

fn curve_poles(c: &Curve) -> Result<(), Failure> {
    let t = BCurveTool::get_type(c.get_curve())?;
    if t != CurveType::BezierCurve && t != CurveType::BSplineCurve {
        return Ok(());
    }
    let n = c.nb_poles()?;
    let mut poles = Array1::with_bounds(1, n).expect("probe array");
    let mut w = Array1::with_bounds(1, n).expect("probe array");
    attempt(|| c.poles(&mut poles).map(|()| print_poles(&poles, None)));
    attempt(|| {
        c.poles_and_weights(&mut poles, &mut w)
            .map(|()| print_poles(&poles, Some(&w)))
    });
    if t != CurveType::BSplineCurve {
        return Ok(());
    }
    let k = c.nb_knots()?;
    let mut kn = Array1::with_bounds(1, k).expect("probe array");
    let mut mu = Array1::with_bounds(1, k).expect("probe array");
    attempt(|| c.knots(&mut kn).map(|()| kn.iter().for_each(|x| real(*x))));
    attempt(|| {
        c.multiplicities(&mut mu)
            .map(|()| mu.iter().for_each(|x| int(*x)))
    });
    let b = BCurveTool::bspline(c.get_curve())?;
    attempt(|| {
        c.poles_of(&b, &mut poles)
            .map(|()| print_poles(&poles, None))
    });
    attempt(|| {
        c.poles_and_weights_of(&b, &mut poles, &mut w)
            .map(|()| print_poles(&poles, Some(&w)))
    });
    Ok(())
}

/// `projector edge t1 t2 t3`: see `hb_curve_case`.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let projector = projector(&mut r)?;
        let edge = edge(&mut r)?;
        let mut c = Curve::new();
        c.set_projector(Some(Arc::new(projector)));
        c.set_curve(&edge)?;
        curve_box(&mut c);
        for _ in 0..3 {
            curve_at(&c, r.real());
        }
        curve_conics(&c);
        curve_poles(&c)
    });
    end();
}
