//! BRepAlgo package probe operations, the Rust twin of tools/oracle/tkbool_probe_brepalgo.cxx
//! (the records are documented there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag};
use super::shape::shape;
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The continuity of its enumeration value (C0..CN).
fn continuity_of(value: i32) -> Continuity {
    match value {
        1 => Continuity::G1,
        2 => Continuity::C1,
        3 => Continuity::G2,
        4 => Continuity::C2,
        5 => Continuity::C3,
        6 => Continuity::CN,
        _ => Continuity::C0,
    }
}

/// The program of a request, `None` (printed) when it fails.
fn program(r: &mut Reader<'_>) -> Option<TpShapes> {
    match tp_shapes(r) {
        Ok(t) => Some(t),
        Err(error) => {
            failed(error);
            None
        }
    }
}

/// `ba_concat program wire option angtol`.
pub fn concat_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (index, option, tol) = (r.integer(), continuity_of(r.integer()), r.real());
    attempt(|| {
        shape(brep_algo::concatenate_wire(&topods::wire(&at(&t, index)?)?, option, tol)?.as_ref())
    });
    end();
}

/// `ba_concat_c0 program wire`.
pub fn concat_c0_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let index = r.integer();
    attempt(|| shape(brep_algo::concatenate_wire_c0(&topods::wire(&at(&t, index)?)?)?.as_ref()));
    end();
}

/// `ba_convert program face angtol`.
pub fn convert_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (index, tol) = (r.integer(), r.real());
    let face = at(&t, index).and_then(|f| topods::face(&f));
    if let Ok(f) = &face {
        let wires: Vec<Shape> = Explorer::new(f.as_ref(), ShapeEnum::Wire, ShapeEnum::Shape)
            .map(|e| e.collect())
            .unwrap_or_default();
        for w in &wires {
            attempt(|| shape(brep_algo::convert_wire(&topods::wire(w)?, tol, f)?.as_ref()));
        }
    }
    attempt(|| shape(brep_algo::convert_face(&face.clone()?, tol)?.as_ref()));
    end();
}

/// `ba_valid program n arg... result closed geomctrl`.
pub fn valid_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let mut args: List<Shape> = List::default();
    for _ in 0..r.integer() {
        if let Ok(s) = at(&t, r.integer()) {
            args.append(s);
        }
    }
    let result = at(&t, r.integer()).unwrap_or_default();
    let (closed, geom) = (r.flag(), r.flag());
    attempt(|| brep_algo::is_valid(&result).map(flag));
    attempt(|| brep_algo::is_topologically_valid(&result).map(flag));
    attempt(|| brep_algo::is_valid_result(&args, &result, closed, geom).map(flag));
    end();
}
