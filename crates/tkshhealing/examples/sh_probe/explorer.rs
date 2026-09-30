//! ShapeExtend_Explorer probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_explorer.cxx (which documents the request).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, int};
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use super::wire_data::sh_shape;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::{HSequence, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_extend::{ShapeDispatch, ShapeExtendExplorer, ShapeHSequence};

/// `sh_ex_item`: the IsSame index and the orientation (-1 for a null shape).
fn item(t: &TpShapes, s: &Shape) {
    int(tp_same_index(t, s));
    int(if s.is_null() {
        -1
    } else {
        s.orientation() as i32
    });
}

/// `sh_ex_shape`: the item, then its type and its children.
fn new_shape(t: &TpShapes, s: &Shape) -> Result<(), Failure> {
    item(t, s);
    if s.is_null() {
        return Ok(());
    }
    int(s.shape_type()? as i32);
    int(s.nb_children());
    let mut it = ShapeIterator::new(s, true, true);
    while it.more() {
        item(t, it.value()?);
        it.next();
    }
    Ok(())
}

/// `sh_ex_list`: `n (index)...`, n = -1 for a null handle.
fn list(r: &mut Reader<'_>, t: &TpShapes) -> ShapeHSequence {
    let n = r.integer();
    if n < 0 {
        return Handle::default();
    }
    let seq = Handle::new(HSequence::default());
    for _ in 0..n {
        seq.append(sh_shape(t, r.integer()));
    }
    seq
}

/// `sh_ex_seq`: the length (-1 for null) and the items.
fn seq(t: &TpShapes, seq: &ShapeHSequence) {
    let Some(seq) = seq.get() else {
        int(-1);
        return;
    };
    seq.sequence(|s| {
        int(s.length());
        s.iter().for_each(|shape| item(t, shape));
    });
}

fn list_from_seq(r: &mut Reader<'_>, t: &TpShapes, ex: &ShapeExtendExplorer) {
    let source = list(r, t);
    let (pre, clear) = (r.integer(), r.flag());
    let mut out = List::default();
    if pre >= -1 {
        out.append(sh_shape(t, pre));
    }
    ex.list_from_seq(&source, &mut out, clear);
    int(out.extent());
    out.iter().for_each(|shape| item(t, shape));
}

fn dispatch(r: &mut Reader<'_>, t: &TpShapes, ex: &ShapeExtendExplorer) -> Result<(), Failure> {
    let source = list(r, t);
    let mask = r.integer();
    let mut out = ShapeDispatch::default();
    let fields = [
        &mut out.vertices,
        &mut out.edges,
        &mut out.wires,
        &mut out.faces,
        &mut out.shells,
        &mut out.solids,
        &mut out.compsols,
        &mut out.compounds,
    ];
    for (k, field) in fields.into_iter().enumerate() {
        if mask >> k & 1 == 1 {
            *field = Handle::new(HSequence::default());
            field.append(t.shapes[0].clone());
        }
    }
    ex.dispatch_list(&source, &mut out)?;
    for s in [
        &out.vertices,
        &out.edges,
        &out.wires,
        &out.faces,
        &out.shells,
        &out.solids,
        &out.compsols,
        &out.compounds,
    ] {
        seq(t, s);
    }
    Ok(())
}

fn sorted(r: &mut Reader<'_>, t: &TpShapes, ex: &ShapeExtendExplorer) -> Result<(), Failure> {
    let s = r.integer();
    let kind = ShapeEnum::ALL[r.integer() as usize];
    let (explore, compound) = (r.flag(), r.flag());
    let result = ex.sorted_compound(&sh_shape(t, s), kind, explore, compound)?;
    new_shape(t, &result)?;
    shape(&result)
}

fn run(r: &mut Reader<'_>, t: &TpShapes, ex: &ShapeExtendExplorer, op: i32) -> Result<(), Failure> {
    match op {
        0 => {
            let source = list(r, t);
            attempt(|| {
                let compound = ex.compound_from_seq(&source)?;
                new_shape(t, &compound)?;
                shape(&compound)
            });
        }
        1 => {
            let (s, exp) = (r.integer(), r.flag());
            seq(t, &ex.seq_from_compound(&sh_shape(t, s), exp)?);
        }
        2 => list_from_seq(r, t, ex),
        3 => {
            let source = list(r, t);
            let mut items = List::default();
            source.sequence(|s| s.iter().for_each(|shape| _ = items.append(shape.clone())));
            seq(t, &ex.seq_from_list(&items));
        }
        4 => {
            let s = r.integer();
            int(ex.shape_type(&sh_shape(t, s), r.flag())? as i32);
        }
        5 => sorted(r, t, ex)?,
        _ => dispatch(r, t, ex)?,
    }
    Ok(())
}

/// `sh_explorer program op ...`.
pub fn explorer_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shape program");
    let op = r.integer();
    attempt(|| run(&mut r, &t, &ShapeExtendExplorer::new(), op));
    end();
}
