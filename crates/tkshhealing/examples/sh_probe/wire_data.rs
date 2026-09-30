//! ShapeExtend_WireData probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_wiredata.cxx (which documents the request).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, end, flag, int, show};
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;

/// `sh_shape`: the shape of index `i`, a null shape for -1.
pub fn sh_shape(t: &TpShapes, i: i32) -> Shape {
    usize::try_from(i).map_or_else(|_| Shape::default(), |i| t.shapes[i].clone())
}
/// `sh_edge`: TopoDS::Edge of the shape of index `i`.
pub fn sh_edge(t: &TpShapes, i: i32) -> Result<Edge, Failure> {
    topods::edge(&sh_shape(t, i))
}
/// `sh_wire`: TopoDS::Wire of the shape of index `i`.
pub fn sh_wire(t: &TpShapes, i: i32) -> Result<Wire, Failure> {
    topods::wire(&sh_shape(t, i))
}
/// `sh_face`: TopoDS::Face of the shape of index `i`.
pub fn sh_face(t: &TpShapes, i: i32) -> Result<Face, Failure> {
    topods::face(&sh_shape(t, i))
}

/// `sh_wd_edge`: the IsSame index of an edge in the table and its orientation.
fn wd_edge(t: &TpShapes, e: &Edge) {
    int(tp_same_index(t, e));
    int(e.orientation() as i32);
}

fn add_oriented(t: &TpShapes, wd: &ShapeExtendWireData, a: &[i32; 4]) -> Result<(), Failure> {
    match a[0] {
        0 => wd.add_oriented_edge(&sh_edge(t, a[1])?, a[2]),
        1 => wd.add_oriented_wire(&sh_wire(t, a[1])?, a[2]),
        _ => wd.add_oriented_shape(&sh_shape(t, a[1]), a[2]),
    }
}

fn other(t: &TpShapes, a: &[i32; 4]) -> Result<ShapeExtendWireData, Failure> {
    ShapeExtendWireData::from_wire(&sh_wire(t, a[0])?, a[1] != 0, a[2] != 0)
}

fn op_add(t: &TpShapes, wd: &ShapeExtendWireData, op: i32, a: &[i32; 4]) -> Result<(), Failure> {
    match op {
        1 => wd.add_edge(&sh_edge(t, a[0])?, a[1]),
        2 => wd.add_wire(&sh_wire(t, a[0])?, a[1]),
        3 => wd.add_shape(&sh_shape(t, a[0]), a[1]),
        4 => add_oriented(t, wd, a),
        _ => wd.set(&sh_edge(t, a[0])?, a[1]),
    }
}

fn op_other(t: &TpShapes, wd: &ShapeExtendWireData, op: i32, a: &[i32; 4]) -> Result<(), Failure> {
    match op {
        14 => wd.init(&other(t, a)?),
        15 => wd.add_wire_data(&other(t, a)?, a[3]),
        16 => wd.init(wd),
        _ => {
            let kept = wd.nonmanifold_edges();
            wd.clear();
            int(kept.sequence(|s| s.length()));
            Ok(())
        }
    }
}

/// `sh_wd_op`.
fn op(t: &TpShapes, wd: &ShapeExtendWireData, op: i32, a: &[i32; 4]) -> Result<(), Failure> {
    match op {
        0 => wd
            .init_wire(&sh_wire(t, a[0])?, a[1] != 0, a[2] != 0)
            .map(flag),
        1..=4 | 6 => op_add(t, wd, op, a),
        5 => wd.remove(a[0]),
        7 => wd.reverse(),
        8 => wd.reverse_on_face(&sh_face(t, a[0])?),
        9 => wd.set_last(a[0]),
        10 => wd.set_degenerated_last(),
        11 => wd.compute_seams(a[0] != 0),
        12 => {
            wd.clear();
            Ok(())
        }
        13 => {
            wd.set_manifold_mode(a[0] != 0);
            Ok(())
        }
        _ => op_other(t, wd, op, a),
    }
}

/// `sh_wd_operand_count`.
const OPERAND_COUNTS: [usize; 18] = [3, 2, 2, 2, 3, 1, 2, 0, 1, 1, 0, 1, 0, 1, 3, 4, 0, 0];

fn digest(t: &TpShapes, wd: &ShapeExtendWireData) -> Result<(), Failure> {
    flag(wd.manifold_mode());
    let n = wd.nb_edges();
    int(n);
    for i in 1..=n {
        wd_edge(t, &wd.edge(i)?);
    }
    let nm = wd.nb_non_manifold_edges();
    int(nm);
    let mut null_edge = false;
    for i in 1..=nm {
        wd_edge(t, &wd.nonmanifold_edge(i)?);
        null_edge |= wd.nonmanifold_edge(i)?.is_null();
    }
    for i in 1..=n {
        flag(wd.is_seam(i));
        int(wd.index(&wd.edge(i)?)?);
    }
    if n > 0 {
        wd_edge(t, &wd.edge(-1)?);
    }
    show(wd.edge(n + 1), |e| wd_edge(t, &e));
    flag(wd.nonmanifold_edge(-1)?.is_null());
    // A null non-manifold edge crashes Wire() natively: the wires are left out then.
    if null_edge && wd.manifold_mode() {
        return Ok(());
    }
    attempt(|| shape(&wd.wire()?.into_shape()));
    attempt(|| shape(&wd.wire_api_make()?.into_shape()));
    Ok(())
}

fn construct(r: &mut Reader<'_>, t: &TpShapes) -> Option<Handle<ShapeExtendWireData>> {
    if r.integer() == 0 {
        return Some(Handle::new(ShapeExtendWireData::new()));
    }
    let (w, chained, manifold) = (r.integer(), r.flag(), r.flag());
    let mut made = None;
    attempt(|| {
        made = Some(Handle::new(ShapeExtendWireData::from_wire(
            &sh_wire(t, w)?,
            chained,
            manifold,
        )?));
        Ok(())
    });
    made
}

/// `sh_wiredata program ctor nops (op)...`.
pub fn wiredata_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shape program");
    let wd = construct(&mut r, &t);
    let nops = r.integer();
    for _ in 0..nops {
        let code = r.integer();
        let mut a = [0; 4];
        for slot in a.iter_mut().take(OPERAND_COUNTS[code as usize]) {
            *slot = r.integer();
        }
        if let Some(wd) = &wd {
            attempt(|| op(&t, wd, code, &a));
        }
    }
    if let Some(wd) = &wd {
        attempt(|| digest(&t, wd));
    }
    end();
}
