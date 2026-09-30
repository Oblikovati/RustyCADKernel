//! Complete executable BSurfaceTool declarations, including native exceptions.
mod fixtures;
use super::common::{attempt, end, flag, int, pnt, real, show, xyz};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkhlr::hlr_brep::BSurfaceTool as Tool;
use rustycad_tkmath::geom_abs::Shape;
pub fn case(v: &[f64]) {
    attempt(|| {
        let s = fixtures::surface(v[0] as i32, v[1])?;
        metadata(&s);
        evaluation(&s, v[2], v[3]);
        geometry(&s);
        intervals(&s);
        trims(&s, v[4], v[5]);
        show(Tool::nb_samples_u(&s), int);
        show(Tool::nb_samples_v(&s), int);
        show(Tool::nb_samples_u_range(&s, v[4], v[5]), int);
        show(Tool::nb_samples_v_range(&s, v[4], v[5]), int);
        Ok(())
    });
    end();
}
fn metadata(s: &BRepAdaptorSurface) {
    metadata_bounds(s);
    metadata_spline(s);
    metadata_kind(s);
}
fn metadata_bounds(s: &BRepAdaptorSurface) {
    show(Tool::first_u_parameter(s), real);
    show(Tool::first_v_parameter(s), real);
    show(Tool::last_u_parameter(s), real);
    show(Tool::last_v_parameter(s), real);
    show(Tool::u_period(s), real);
    show(Tool::v_period(s), real);
    show(Tool::is_u_closed(s), flag);
    show(Tool::is_v_closed(s), flag);
    show(Tool::is_u_periodic(s), flag);
    show(Tool::is_v_periodic(s), flag);
}
fn metadata_spline(s: &BRepAdaptorSurface) {
    show(Tool::u_degree(s), int);
    show(Tool::v_degree(s), int);
    show(Tool::nb_u_poles(s), int);
    show(Tool::nb_v_poles(s), int);
    show(Tool::nb_u_knots(s), int);
    show(Tool::nb_v_knots(s), int);
    show(Tool::is_u_rational(s), flag);
    show(Tool::is_v_rational(s), flag);
}
fn metadata_kind(s: &BRepAdaptorSurface) {
    show(Tool::get_type(s), |v| int(v as i32));
    show(Tool::u_continuity(s), |v| int(v as i32));
    show(Tool::v_continuity(s), |v| int(v as i32));
}
fn evaluation(s: &BRepAdaptorSurface, u: f64, v: f64) {
    show(Tool::value(s, u, v), pnt);
    show(Tool::d0(s, u, v), pnt);
    derivatives(s, u, v);
    for (nu, nv) in [(0, 0), (1, 0), (0, 1), (2, 1), (1, 3)] {
        show(Tool::dn(s, u, v, nu, nv), |a| xyz(*a.xyz()));
    }
    for tolerance in [-1., 0., 1e-7, 2.] {
        show(Tool::u_resolution(s, tolerance), real);
        show(Tool::v_resolution(s, tolerance), real);
    }
}
fn derivatives(s: &BRepAdaptorSurface, u: f64, v: f64) {
    show(Tool::d1(s, u, v), |r| {
        print_derivative(r.point, &[r.d1u, r.d1v])
    });
    show(Tool::d2(s, u, v), |r| {
        print_derivative(r.point, &[r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv])
    });
    show(Tool::d3(s, u, v), |r| {
        print_derivative(
            r.point,
            &[
                r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv, r.d3u, r.d3v, r.d3uuv, r.d3uvv,
            ],
        )
    });
}
fn print_derivative(
    point: rustycad_tkmath::gp::Point3,
    derivatives: &[rustycad_tkmath::gp::Vector3],
) {
    pnt(point);
    for derivative in derivatives {
        xyz(*derivative.xyz());
    }
}
fn geometry(s: &BRepAdaptorSurface) {
    show(Tool::plane(s), |v| pnt(v.location()));
    show(Tool::cylinder(s), |v| real(v.radius()));
    show(Tool::cone(s), |v| real(v.semi_angle()));
    show(Tool::sphere(s), |v| real(v.radius()));
    show(Tool::torus(s), |v| real(v.major_radius()));
    show(Tool::axe_of_revolution(s), |v| {
        pnt(v.location());
        xyz(v.direction().xyz());
    });
    show(Tool::direction(s), |v| xyz(v.xyz()));
    show(Tool::basis_curve(s).and_then(|v| v.value(0.5)), pnt);
    show(Tool::bezier(s).and_then(|v| v.value(0.5, 0.5)), pnt);
    show(Tool::bspline(s).and_then(|v| v.value(0.5, 0.5)), pnt);
}
fn intervals(s: &BRepAdaptorSurface) {
    for shape in [Shape::C0, Shape::C1, Shape::C2, Shape::C3, Shape::CN] {
        attempt(|| direction_intervals(s, shape, true));
        attempt(|| direction_intervals(s, shape, false));
    }
}
fn direction_intervals(s: &BRepAdaptorSurface, shape: Shape, u: bool) -> Result<(), Failure> {
    let count = if u {
        Tool::nb_u_intervals(s, shape)?
    } else {
        Tool::nb_v_intervals(s, shape)?
    };
    int(count);
    let mut bounds = Array1::with_bounds(1, count + 1).unwrap();
    if u {
        Tool::u_intervals(s, &mut bounds, shape)?;
    } else {
        Tool::v_intervals(s, &mut bounds, shape)?;
    }
    for i in bounds.lower()..=bounds.upper() {
        real(bounds[i]);
    }
    Ok(())
}
fn trims(s: &BRepAdaptorSurface, first: f64, last: f64) {
    for trim in [Tool::u_trim, Tool::v_trim] {
        attempt(|| {
            let t = trim(s, first, last, 1e-7)?;
            real(t.first_u_parameter()?);
            real(t.last_u_parameter()?);
            real(t.first_v_parameter()?);
            real(t.last_v_parameter()?);
            pnt(t.value(0.5, 0.5)?);
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests;
