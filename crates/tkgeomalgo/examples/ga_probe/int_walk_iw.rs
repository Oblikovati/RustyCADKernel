//! IntWalk_IWalking.gxx probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_iwalk.cxx: the generic runs through its IntPatch
//! instance `IntPatchTheIWalking` (`HSurfaceTool`, `IntPatchTheSurfFunction` and
//! `IntPatchTheIWLineOfTheIWalking`).
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, xyz};
use super::int_surf::quadric;
use super::int_walk::{pnt2s, surface};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchTheIWLineOfTheIWalking, IntPatchTheIWalking, IntPatchTheSurfFunction,
};
use rustycad_tkgeomalgo::int_surf::{IntSurfInteriorPoint, IntSurfPathPoint};
use rustycad_tkgeomalgo::int_walk::TheIWFunction;
use rustycad_tkmath::gp::{Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

type SurfFunction = IntPatchTheSurfFunction;

fn evaluate(f: &mut SurfFunction, u: f64, v: f64) -> Result<(), Failure> {
    let mut x = Vector::with_bounds(1, 2)?;
    let mut fv = Vector::with_bounds(1, 1)?;
    let mut d = Matrix::with_bounds(1, 1, 1, 2).expect("probe matrix");
    (x[1], x[2]) = (u, v);
    f.checked_values(&x, &mut fv, &mut d)?;
    Ok(())
}

/// `ga_iw_path_point`.
fn path_point(r: &mut Reader, f: &mut SurfFunction) -> Result<IntSurfPathPoint, Failure> {
    let (u, v) = (r.real(), r.real());
    let (pass, tangent, nmult) = (r.flag(), r.flag(), r.integer());
    evaluate(f, u, v)?;
    let mut p = IntSurfPathPoint::with_value(f.point(), u, v);
    if !tangent {
        let d3 = TheIWFunction::direction3d(f)?;
        p.set_directions(d3, TheIWFunction::direction2d(f)?);
    }
    p.set_passing(pass);
    for _ in 0..nmult {
        let a = r.real();
        p.add_uv(a, r.real())?;
    }
    Ok(p)
}

/// `ga_iw_interior_point`.
fn interior_point(r: &mut Reader, f: &mut SurfFunction) -> Result<IntSurfInteriorPoint, Failure> {
    let (u, v) = (r.real(), r.real());
    evaluate(f, u, v)?;
    if TheIWFunction::is_tangent(f)? {
        return Ok(IntSurfInteriorPoint::with_value(
            f.point(),
            u,
            v,
            Vector3::new(0.0, 0.0, 0.0),
            Vector2::new(1.0, 0.0),
        ));
    }
    let d2 = TheIWFunction::direction2d(f)?;
    let d3 = TheIWFunction::direction3d(f)?;
    Ok(IntSurfInteriorPoint::with_value(
        f.point(),
        u,
        v,
        d3,
        Vector2::new(d2.x(), d2.y()),
    ))
}

pub fn line_report(l: &IntPatchTheIWLineOfTheIWalking) {
    let n = l.nb_points();
    int(n);
    for i in 1..=n {
        pnt2s(&l.value(i).expect("probe line point"));
    }
    flag(l.is_closed());
    flag(l.has_first_point());
    flag(l.has_last_point());
    show(l.first_point_index(), int);
    show(l.first_point(), |p| pnt(p.value()));
    show(l.last_point_index(), int);
    show(l.last_point(), |p| pnt(p.value()));
    int(l.nb_passing_point());
    for i in 1..=l.nb_passing_point() {
        let (a, b) = l.passing_point(i).expect("probe passing point");
        int(a);
        int(b);
    }
    let (v, idx) = l.tangent_vector();
    xyz(*v.xyz());
    int(idx);
    flag(l.is_tangent_at_begining());
    flag(l.is_tangent_at_end());
}

fn walk_report(w: &IntPatchTheIWalking) {
    flag(w.is_done());
    attempt(|| {
        let n = w.nb_lines()?;
        int(n);
        for i in 1..=n {
            line_report(w.value(i)?);
        }
        Ok(())
    });
    attempt(|| {
        let n = w.nb_single_pnts()?;
        int(n);
        for i in 1..=n {
            let p = w.single_pnt(i)?;
            pnt(p.value());
            let (u, v) = p.value2d()?;
            real(u);
            real(v);
        }
        Ok(())
    });
    attempt(|| w.value(0).map(|_| ()));
    attempt(|| w.value(w.nb_lines()? + 1).map(|_| ()));
    attempt(|| w.single_pnt(0).map(|_| ()));
}

/// `surface quadric tol eps defl step fill reversed mode npath [...] nloop [...]`.
pub fn walk_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = surface(&mut r)?;
        let q = quadric(&mut r);
        let (tol, eps, defl, step) = (r.real(), r.real(), r.real(), r.real());
        let (fill, rev, mode) = (r.flag(), r.flag(), r.integer());
        let mut f = SurfFunction::with_surfaces(&s, &q);
        f.set_tolerance(tol);
        let np = r.integer();
        let mut paths = Vec::new();
        for _ in 0..np {
            paths.push(path_point(&mut r, &mut f)?);
        }
        let nl = r.integer();
        let mut loops = Vec::new();
        for _ in 0..nl {
            loops.push(interior_point(&mut r, &mut f)?);
        }
        let mut w = IntPatchTheIWalking::new(eps, defl, step, fill)?;
        if mode != 2 {
            attempt(|| w.perform(&paths, &loops, &mut f, &s, rev));
        }
        if mode == 1 {
            attempt(|| w.perform_open(&paths, &mut f, &s, rev));
        }
        walk_report(&w);
        Ok(())
    });
    end();
}
