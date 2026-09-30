//! Contap walking from boundary and interior seeds.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::contap_iwline::line_report;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::{
    int_surf::{IntSurfInteriorPoint, IntSurfPathPoint},
    int_walk::TheIWFunction,
};
use rustycad_tkhlr::contap::{ContapSurfFunction as SurfFunction, ContapTheIWalking as Walk};
use rustycad_tkmath::{
    gp::{Vector2, Vector3},
    math::{Matrix, Vector},
};
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

fn walk_report(w: &Walk) {
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

/// Trace original contour walks with both Perform overloads; `walking_case(numbers)`.
pub fn walking_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = super::contap_surface::surface_adaptor(&mut reader)?;
        let mut function = SurfFunction::new();
        function.set_surface(&surface)?;
        function.set_direction(reader.dir());
        function.set_tolerance(1e-7);
        let (epsilon, deflection, step) = (reader.real(), reader.real(), reader.real());
        let (fill, reversed, mode) = (reader.flag(), reader.flag(), reader.integer());
        let mut paths = Vec::new();
        for _ in 0..reader.integer() {
            paths.push(path_point(&mut reader, &mut function)?);
        }
        let mut loops = Vec::new();
        for _ in 0..reader.integer() {
            loops.push(interior_point(&mut reader, &mut function)?);
        }
        let mut walk = Walk::new(epsilon, deflection, step, fill)?;
        walk.set_tolerance(epsilon, deflection, step);
        if mode != 2 {
            attempt(|| walk.perform(&paths, &loops, &mut function, &surface, reversed));
        }
        if mode == 1 {
            attempt(|| walk.perform_open(&paths, &mut function, &surface, reversed));
        }
        walk_report(&walk);
        Ok(())
    });
    end();
}
