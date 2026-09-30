//! GeomFill_Gordon and GeomFill_NetworkSurface probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_gordon.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::geom_api_surface::bsurface_out;
use super::ics_geometry::{geom_curve, params};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillGordon, GeomFillNetworkSurface, GordonApproximationMode,
};
use rustycad_tkmath::gp::Point3;

/// `ga_gordon_curves`: `n curve*n` (n >= 1) as a 1-based array of curves.
fn gordon_curves(r: &mut Reader) -> Array1<'static, Handle<dyn GeomCurve>> {
    let n = r.integer();
    let mut a: Array1<'static, Handle<dyn GeomCurve>> =
        Array1::with_bounds(1, n).expect("curve array");
    for i in 1..=n {
        a[i] = geom_curve(r);
    }
    a
}

/// `ga_gordon_bsplines`: `n curve*n` (n >= 1) as a 1-based array of B-spline curves.
fn gordon_bsplines(r: &mut Reader) -> Array1<'static, Handle<GeomBSplineCurve>> {
    let n = r.integer();
    let mut a: Array1<'static, Handle<GeomBSplineCurve>> =
        Array1::with_bounds(1, n).expect("curve array");
    for i in 1..=n {
        a[i] = geom_curve(r).cast();
    }
    a
}

/// `ga_gordon_print`: status, report, flags, approximation mode and surface.
fn gordon_print(g: &GeomFillGordon) {
    let rep = g.report();
    int(g.status() as i32);
    int(rep.status as i32);
    int(rep.failed_stage as i32);
    flag(rep.is_approximate);
    real(rep.max_contact_gap);
    real(rep.max_reparametrization_deviation);
    real(rep.max_profile_deviation);
    real(rep.max_guide_deviation);
    real(rep.max_approximation_deviation);
    flag(g.is_done());
    flag(g.is_approximate());
    flag(g.is_parallel_mode());
    int(g.get_approximation_mode() as i32);
    attempt(|| bsurface_out(&g.surface()?));
}

/// `mode parallel tolerance profiles guides performs`: GeomFill_Gordon.
pub fn gordon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let parallel = r.flag();
    let tol = r.real();
    let profiles = gordon_curves(&mut r);
    let guides = gordon_curves(&mut r);
    let performs = r.integer();
    let mut g = GeomFillGordon::new();
    g.set_parallel_mode(parallel);
    if mode == 1 {
        g.set_approximation_mode(GordonApproximationMode::AllowApproximateFallback);
    }
    attempt(|| g.init(&profiles, &guides, tol));
    gordon_print(&g);
    for _ in 0..performs {
        attempt(|| g.perform());
        gordon_print(&g);
    }
    end();
}

// The row-by-row points and weights on `rl ru cl cu` bounds.
fn network_grids(r: &mut Reader) -> (Array2<'static, Point3>, Array2<'static, f64>) {
    let (rl, ru, cl, cu) = (r.integer(), r.integer(), r.integer(), r.integer());
    let mut pts: Array2<'static, Point3> = Array2::with_bounds(rl, ru, cl, cu).expect("point grid");
    let mut ws: Array2<'static, f64> = Array2::with_bounds(rl, ru, cl, cu).expect("weight grid");
    for i in rl..=ru {
        for j in cl..=cu {
            pts[(i, j)] = r.pnt();
        }
    }
    for i in rl..=ru {
        for j in cl..=cu {
            ws[(i, j)] = r.real();
        }
    }
    (pts, ws)
}

/// `profiles guides np vparams ng uparams rl ru cl cu points weights tolerance uclosed
/// vclosed performs`: GeomFill_NetworkSurface.
pub fn network_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let profiles = gordon_bsplines(&mut r);
    let guides = gordon_bsplines(&mut r);
    let vparams = params(&mut r);
    let uparams = params(&mut r);
    let (pts, ws) = network_grids(&mut r);
    let tol = r.real();
    let closed = (r.flag(), r.flag());
    let performs = r.integer();
    let mut n = GeomFillNetworkSurface::new();
    n.init(
        (&profiles, &guides),
        (&vparams, &uparams),
        (&pts, &ws),
        tol,
        closed,
    );
    int(n.status() as i32);
    flag(n.is_done());
    for _ in 0..performs {
        n.perform();
        int(n.status() as i32);
        flag(n.is_done());
        attempt(|| bsurface_out(&n.surface()?));
    }
    end();
}
