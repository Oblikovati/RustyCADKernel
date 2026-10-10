//! BRepFill package probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_brepfill_package.cxx (the records are documented there).
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_fill;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;

/// The program of a request, or its failure printed and the line ended.
fn program(r: &mut Reader<'_>) -> Option<TpShapes> {
    match tp_shapes(r) {
        Ok(t) => Some(t),
        Err(error) => {
            print!("{} ", error.exception_type());
            end();
            None
        }
    }
}

/// The shape of the program at a table index.
fn at(t: &TpShapes, index: i32) -> Result<Shape, Failure> {
    t.shapes
        .get(index as usize)
        .cloned()
        .ok_or_else(|| Failure::out_of_range("bl_probe: shape index"))
}

/// `bf_face program e1 e2`.
pub fn face_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (e1, e2) = (r.integer(), r.integer());
    attempt(|| {
        let face =
            brep_fill::ruled_face(&topods::edge(&at(&t, e1)?)?, &topods::edge(&at(&t, e2)?)?)?;
        shape(&face)
    });
    end();
}

/// `bf_shell program w1 w2`.
pub fn shell_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (w1, w2) = (r.integer(), r.integer());
    attempt(|| {
        let shell =
            brep_fill::ruled_shell(&topods::wire(&at(&t, w1)?)?, &topods::wire(&at(&t, w2)?)?)?;
        shape(&shell)
    });
    end();
}

/// `bf_axe program spine profile tol`.
pub fn axe_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (spine, profile, tol) = (r.integer(), r.integer(), r.real());
    attempt(|| {
        let (axe, on) = brep_fill::axe(&at(&t, spine)?, &topods::wire(&at(&t, profile)?)?, tol)?;
        pnt(axe.location());
        dir(axe.direction());
        dir(axe.x_direction());
        dir(axe.y_direction());
        flag(on);
        Ok(())
    });
    end();
}

/// `bf_search_origin program wire P Dir tol`.
pub fn search_origin_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let w = r.integer();
    let p = r.pnt();
    let xyz = r.xyz();
    let d = Vector3::new(xyz.x(), xyz.y(), xyz.z());
    let tol = r.real();
    attempt(|| {
        let mut wire = topods::wire(&at(&t, w)?)?;
        brep_fill::search_origin(&mut wire, p, d, tol)?;
        shape(&wire)
    });
    end();
}

/// `bf_compute_acr program wire`.
pub fn compute_acr_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let w = r.integer();
    attempt(|| {
        let wire = topods::wire(&at(&t, w)?)?;
        let mut n: i32 = 0;
        let mut explorer = WireExplorer::with_wire(&wire)?;
        while explorer.more() {
            n += 1;
            explorer.next()?;
        }
        int(n);
        let mut acr: Array1<'static, f64> =
            Array1::with_bounds(0, n).map_err(|_| Failure::out_of_range("acr"))?;
        brep_fill::compute_acr(&wire, &mut acr)?;
        for value in acr.iter() {
            real(*value);
        }
        Ok(())
    });
    end();
}

/// `bf_insert_acr program wire n (cut)... prec`.
pub fn insert_acr_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let w = r.integer();
    let n = r.integer();
    let mut cuts: Array1<'static, f64> = Array1::with_bounds(1, n).unwrap_or_default();
    for i in 1..=n {
        let _ = cuts.set_value(i, r.real());
    }
    let prec = r.real();
    attempt(|| {
        shape(&brep_fill::insert_acr(&topods::wire(&at(&t, w)?)?, &cuts, prec)?.into_shape())
    });
    end();
}
