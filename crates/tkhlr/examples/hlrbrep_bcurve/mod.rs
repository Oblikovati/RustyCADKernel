//! Every declared BCurveTool member is exercised on real geometry.
mod fixtures;
use super::common::{attempt, end, flag, int, pnt, real, show, xyz};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkhlr::hlr_brep::BCurveTool as Tool;
use rustycad_tkmath::{geom_abs::Shape, gp::Point3};
pub fn case(v: &[f64]) {
    attempt(|| {
        let curve = fixtures::curve(v[0] as i32, v[1])?;
        metadata(&curve);
        evaluation(&curve, v[2]);
        conics(&curve);
        poles(&curve, [v[5] as i32, v[6] as i32, v[7] as i32, v[8] as i32]);
        attempt(|| {
            int(Tool::nb_samples(&curve, v[3], v[4])?);
            Ok(())
        });
        Ok(())
    });
    end();
}
fn metadata(curve: &BRepAdaptorCurve) {
    attempt(|| metadata_parameters(curve));
    attempt(|| {
        flag(Tool::is_closed(curve)?);
        flag(Tool::is_periodic(curve)?);
        Ok(())
    });
    show(Tool::period(curve), real);
    for query in [Tool::degree, Tool::nb_poles, Tool::nb_knots] {
        show(query(curve), int);
    }
    show(Tool::is_rational(curve), flag);
    for shape in [Shape::C0, Shape::C1, Shape::C2, Shape::C3, Shape::CN] {
        attempt(|| intervals(curve, shape));
    }
}
fn metadata_parameters(curve: &BRepAdaptorCurve) -> Result<(), Failure> {
    real(Tool::first_parameter(curve)?);
    real(Tool::last_parameter(curve)?);
    int(Tool::get_type(curve)? as i32);
    int(Tool::continuity(curve)? as i32);
    Ok(())
}
fn intervals(curve: &BRepAdaptorCurve, shape: Shape) -> Result<(), Failure> {
    let count = Tool::nb_intervals(curve, shape)?;
    int(count);
    let mut bounds = Array1::with_bounds(-1, count - 1).unwrap();
    Tool::intervals(curve, &mut bounds, shape)?;
    for index in bounds.lower()..=bounds.upper() {
        real(bounds[index]);
    }
    Ok(())
}
fn evaluation(curve: &BRepAdaptorCurve, t: f64) {
    attempt(|| {
        pnt(Tool::value(curve, t)?);
        pnt(Tool::d0(curve, t)?);
        Ok(())
    });
    show(Tool::d1(curve, t), derivative1);
    show(Tool::d2(curve, t), derivative2);
    show(Tool::d3(curve, t), derivative3);
    for n in [0, 1, 2, 3, 5] {
        show(Tool::dn(curve, t, n), |v| xyz(*v.xyz()));
    }
    for tolerance in [-1., 0., 1e-7, 2.] {
        show(Tool::resolution(curve, tolerance), real);
    }
}
fn derivative1((point, vector): (Point3, rustycad_tkmath::gp::Vector3)) {
    pnt(point);
    xyz(*vector.xyz());
}
fn derivative2(
    (point, first, second): (
        Point3,
        rustycad_tkmath::gp::Vector3,
        rustycad_tkmath::gp::Vector3,
    ),
) {
    derivative1((point, first));
    xyz(*second.xyz());
}
fn derivative3((point, first, second, third): rustycad_tkg3d::adaptor3d::Adaptor3dD3) {
    derivative2((point, first, second));
    xyz(*third.xyz());
}
fn conics(curve: &BRepAdaptorCurve) {
    attempt(|| {
        let c = Tool::line(curve)?;
        pnt(c.location());
        xyz(c.direction().xyz());
        Ok(())
    });
    attempt(|| {
        let c = Tool::circle(curve)?;
        pnt(c.location());
        real(c.radius());
        Ok(())
    });
    conic_radii(curve);
    show(Tool::bezier(curve).and_then(|c| c.eval_d0(0.5)), pnt);
    show(Tool::bspline(curve).and_then(|c| c.eval_d0(0.5)), pnt);
}
fn conic_radii(curve: &BRepAdaptorCurve) {
    attempt(|| {
        let c = Tool::ellipse(curve)?;
        pnt(c.location());
        real(c.major_radius());
        real(c.minor_radius());
        Ok(())
    });
    attempt(|| {
        let c = Tool::hyperbola(curve)?;
        pnt(c.location());
        real(c.major_radius());
        real(c.minor_radius());
        Ok(())
    });
    attempt(|| {
        let c = Tool::parabola(curve)?;
        pnt(c.location());
        real(c.focal());
        Ok(())
    });
}

fn poles(curve: &BRepAdaptorCurve, [pl, pu, wl, wu]: [i32; 4]) {
    let mut poles = Array1::with_bounds(pl, pu).unwrap();
    let mut weights = Array1::with_bounds(wl, wu).unwrap();
    for i in pl..=pu {
        poles[i] = Point3::new(7., 8., 9.);
    }
    for i in wl..=wu {
        weights[i] = 17.;
    }
    attempt(|| Tool::poles(curve, &mut poles));
    for i in pl..=pu {
        pnt(poles[i]);
    }
    attempt(|| Tool::poles_and_weights(curve, &mut poles, &mut weights));
    for i in pl..=pu {
        pnt(poles[i]);
    }
    for i in wl..=wu {
        real(weights[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inverse_span_sampling_caps_and_floors() {
        let curve = fixtures::curve(7, 0.).unwrap();
        for (first, last, expected) in [
            (0., 1., 6),
            (0., 0.01, 50),
            (0., 100., 2),
            (1., 0., 2),
            (0., 0., 50),
        ] {
            assert_eq!(Tool::nb_samples(&curve, first, last).unwrap(), expected);
        }
        for (kind, expected) in [(0, 2), (5, 7), (1, 10)] {
            let curve = fixtures::curve(kind, 0.).unwrap();
            assert_eq!(Tool::nb_samples(&curve, 0., 1.).unwrap(), expected);
        }
    }
    #[test]
    fn zero_curve_span_retains_pinned_nan_integer_cast() {
        let curve = fixtures::curve(10, 0.).unwrap();
        assert_eq!(Tool::nb_samples(&curve, 0., 0.).unwrap(), i32::MIN);
        assert_eq!(Tool::nb_samples(&curve, 0., 1.).unwrap(), 2);
    }
    #[test]
    fn located_rational_poles_use_native_indices() {
        let curve = fixtures::curve(8, 2.).unwrap();
        let mut poles = Array1::with_bounds(2, 3).unwrap();
        let mut weights = Array1::with_bounds(2, 3).unwrap();
        Tool::poles_and_weights(&curve, &mut poles, &mut weights).unwrap();
        assert_eq!(poles[2], Point3::new(3., 4., 2.));
        assert_eq!(poles[3], Point3::new(4., 5., 7.));
        assert_eq!((weights[2], weights[3]), (2., 1.));
    }
    #[test]
    fn pole_failure_keeps_prefix_and_leaves_weights_unwritten() {
        let curve = fixtures::curve(5, 0.).unwrap();
        let mut poles = Array1::with_bounds(1, 5).unwrap();
        let mut weights = Array1::with_bounds(1, 4).unwrap();
        for i in 1..=5 {
            poles[i] = Point3::new(7., 8., 9.);
        }
        for i in 1..=4 {
            weights[i] = 17.;
        }
        let failure = Tool::poles_and_weights(&curve, &mut poles, &mut weights).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        assert_eq!(poles[4], Point3::new(3., 0., 16.));
        assert_eq!(poles[5], Point3::new(7., 8., 9.));
        assert_eq!(weights[1], 17.);
    }
    #[test]
    fn non_spline_output_buffers_remain_unchanged() {
        let curve = fixtures::curve(1, 0.).unwrap();
        let mut poles = Array1::with_bounds(-2, -1).unwrap();
        let mut weights = Array1::with_bounds(9, 10).unwrap();
        for i in -2..=-1 {
            poles[i] = Point3::new(7., 8., 9.);
        }
        for i in 9..=10 {
            weights[i] = 17.;
        }
        Tool::poles_and_weights(&curve, &mut poles, &mut weights).unwrap();
        assert_eq!(poles[-1], Point3::new(7., 8., 9.));
        assert_eq!(weights[10], 17.);
    }
}
