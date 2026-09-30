//! The BRepApprox_SurfaceTool probe operation, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepapprox_tool.cxx.
use super::brep_approx::bra_face;
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, xyz};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Axis3;
use rustycad_tktopalgo::brep_approx::{BRepApproxSurface, BRepApproxSurfaceTool as Tool};

const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

/// `tp_bra_ax3`.
fn ax3(a: Axis3) {
    pnt(a.location());
    dir(a.direction());
    dir(a.x_direction());
}

/// The bounds of the intervals of one direction.
fn interval_bounds(
    n: i32,
    fill: impl FnOnce(&mut Array1<'static, f64>) -> Result<(), Failure>,
) -> Result<(), Failure> {
    int(n);
    let mut t: Array1<'static, f64> = Array1::with_bounds(1, n + 1).expect("probe array");
    fill(&mut t)?;
    t.iter().copied().for_each(real);
    Ok(())
}

/// `tp_bra_intervals`.
fn intervals(s: &BRepApproxSurface) {
    for c in SHAPES {
        attempt(|| {
            let n = Tool::nb_u_intervals(s, c)?;
            interval_bounds(n, |t| Tool::u_intervals(s, t, c))
        });
        attempt(|| {
            let n = Tool::nb_v_intervals(s, c)?;
            interval_bounds(n, |t| Tool::v_intervals(s, t, c))
        });
    }
}

/// `tp_bra_trims`.
fn trims(s: &BRepApproxSurface, (u1, u2, v1, v2): (f64, f64, f64, f64), tol: f64) {
    for trimmed in [Tool::u_trim(s, u1, u2, tol), Tool::v_trim(s, v1, v2, tol)] {
        attempt(|| {
            let t = trimmed?;
            real(t.first_u_parameter()?);
            real(t.last_u_parameter()?);
            real(t.first_v_parameter()?);
            real(t.last_v_parameter()?);
            Ok(())
        });
    }
}

/// `tp_bra_periods`.
fn periods(s: &BRepApproxSurface) -> Result<(), Failure> {
    flag(Tool::is_u_closed(s)?);
    flag(Tool::is_v_closed(s)?);
    flag(Tool::is_u_periodic(s)?);
    attempt(|| Tool::u_period(s).map(real));
    flag(Tool::is_v_periodic(s)?);
    attempt(|| Tool::v_period(s).map(real));
    Ok(())
}

/// `tp_bra_evaluate`.
fn evaluate(s: &BRepApproxSurface, u: f64, v: f64) {
    attempt(|| Tool::value(s, u, v).map(pnt));
    attempt(|| Tool::d0(s, u, v).map(pnt));
    attempt(|| {
        let d = Tool::d1(s, u, v)?;
        pnt(d.point);
        [d.d1u, d.d1v].iter().for_each(|x| xyz(*x.xyz()));
        Ok(())
    });
    attempt(|| {
        let d = Tool::d2(s, u, v)?;
        pnt(d.point);
        [d.d1u, d.d1v, d.d2u, d.d2v, d.d2uv]
            .iter()
            .for_each(|x| xyz(*x.xyz()));
        Ok(())
    });
    attempt(|| {
        let d = Tool::d3(s, u, v)?;
        pnt(d.point);
        [
            d.d1u, d.d1v, d.d2u, d.d2v, d.d2uv, d.d3u, d.d3v, d.d3uuv, d.d3uvv,
        ]
        .iter()
        .for_each(|x| xyz(*x.xyz()));
        Ok(())
    });
    for (nu, nv) in [(1, 0), (0, 1), (1, 1), (2, 0)] {
        attempt(|| Tool::dn(s, u, v, nu, nv).map(|d| xyz(*d.xyz())));
    }
}

/// The quadric accessors of `tp_bra_kind`.
fn quadrics(s: &BRepApproxSurface) {
    attempt(|| Tool::plane(s).map(|p| ax3(p.position())));
    attempt(|| {
        let c = Tool::cylinder(s)?;
        ax3(c.position());
        real(c.radius());
        Ok(())
    });
    attempt(|| {
        let c = Tool::cone(s)?;
        ax3(c.position());
        real(c.ref_radius());
        real(c.semi_angle());
        Ok(())
    });
    attempt(|| {
        let c = Tool::torus(s)?;
        ax3(c.position());
        real(c.major_radius());
        real(c.minor_radius());
        Ok(())
    });
    attempt(|| {
        let c = Tool::sphere(s)?;
        ax3(c.position());
        real(c.radius());
        Ok(())
    });
}

/// `tp_bra_kind`.
fn kind(s: &BRepApproxSurface, u1: f64, u2: f64) -> Result<(), Failure> {
    int(Tool::get_type(s)? as i32);
    quadrics(s);
    attempt(|| {
        let b = Tool::bezier(s)?;
        flag(b.is_null());
        if let Some(b) = b.get() {
            int(b.nb_u_poles());
            int(b.nb_v_poles());
        }
        Ok(())
    });
    attempt(|| {
        let b = Tool::bspline(s)?;
        flag(b.is_null());
        if let Some(b) = b.get() {
            int(b.nb_u_poles());
            int(b.nb_v_poles());
        }
        Ok(())
    });
    attempt(|| {
        let a = Tool::axe_of_revolution(s)?;
        pnt(a.location());
        dir(a.direction());
        Ok(())
    });
    attempt(|| Tool::direction(s).map(dir));
    attempt(|| Tool::basis_curve(s).map(|c| flag(c.is_null())));
    int(Tool::nb_samples_u(s));
    int(Tool::nb_samples_v(s));
    int(Tool::nb_samples_u_range(s, u1, u2));
    int(Tool::nb_samples_v_range(s, u1, u2));
    Ok(())
}

/// `bra_tool`: `face restr u1 u2 v1 v2 tol r3d n (u v)...`.
pub fn tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let f = bra_face(&mut r)?;
        let s = &f.s;
        let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
        let (tol, r3d) = (r.real(), r.real());
        real(Tool::first_u_parameter(s)?);
        real(Tool::last_u_parameter(s)?);
        real(Tool::first_v_parameter(s)?);
        real(Tool::last_v_parameter(s)?);
        intervals(s);
        trims(s, (u1, u2, v1, v2), tol);
        periods(s)?;
        attempt(|| Tool::u_resolution(s, r3d).map(real));
        attempt(|| Tool::v_resolution(s, r3d).map(real));
        kind(s, u1, u2)?;
        for _ in 0..r.integer() {
            let (u, v) = (r.real(), r.real());
            evaluate(s, u, v);
        }
        Ok(())
    });
    end();
}
