//! ShapeFix_Root, ShapeFix_ShapeTolerance and ShapeFix_EdgeConnect probe operations, the
//! Rust twin of tools/oracle/tkshhealing_probe_fix_basic.cxx (which documents the requests).
use super::analysis_basic::shape_enum;
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, text};
use super::registrator::{maps, message};
use super::shapes::{TpShapes, tp_same_index, tp_shapes};
use super::wire_data::{sh_edge, sh_shape};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkernel::message::Gravity;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::{
    BasicMsgRegistrator, ShapeExtendBasicMsgRegistrator, ShapeExtendMsgRegistrator,
};
use rustycad_tkshhealing::shape_fix::{
    FixRoot, ShapeFixEdgeConnect, ShapeFixRoot, ShapeFixShapeTolerance, need_fix,
};

/// `sh_fr_send`.
fn send(root: &ShapeFixRoot, s: Option<&Shape>, k: i32, which: i32) {
    let m = message(k);
    match (s, which) {
        (Some(s), 0) => root.send_msg(s, &m, Gravity::Info),
        (Some(s), 1) => root.send_warning(s, &m),
        (Some(s), _) => root.send_fail(s, &m),
        (None, 0) => root.send_msg_on_shape(&m, Gravity::Info),
        (None, 1) => root.send_warning_on_shape(&m),
        (None, _) => root.send_fail_on_shape(&m),
    }
}

fn registrator(kind: i32) -> Handle<dyn BasicMsgRegistrator> {
    match kind {
        1 => Handle::new(ShapeExtendBasicMsgRegistrator::new()).cast(),
        2 => Handle::new(ShapeExtendMsgRegistrator::new()).cast(),
        _ => Handle::default(),
    }
}

/// `sh_fr_op`.
fn fixroot_op(r: &mut Reader<'_>, t: &TpShapes, root: &ShapeFixRoot, op: i32) {
    match op {
        0 => root.set_precision(r.real()),
        1 => root.set_min_tolerance(r.real()),
        2 => root.set_max_tolerance(r.real()),
        3 => real(root.limit_tolerance(r.real())),
        4 => {
            let other = ShapeFixRoot::new();
            other.set_precision(r.real());
            other.set_min_tolerance(r.real());
            other.set_max_tolerance(r.real());
            root.set(&other);
        }
        5 => root.set_context(&Handle::new(ShapeBuildReShape::new())),
        6 => root.set_msg_registrator(&registrator(r.integer())),
        7 => {
            let s = sh_shape(t, r.integer());
            let k = r.integer();
            send(root, Some(&s), k, r.integer());
        }
        8 => {
            let k = r.integer();
            send(root, None, k, r.integer());
        }
        _ => {
            let f = r.integer();
            flag(need_fix(f, r.flag()));
        }
    }
}

/// `sh_fixroot program nops (op)...`.
pub fn fixroot_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shapes");
    let root = ShapeFixRoot::new();
    for _ in 0..r.integer() {
        let op = r.integer();
        fixroot_op(&mut r, &t, &root, op);
    }
    real(root.precision());
    real(root.min_tolerance());
    real(root.max_tolerance());
    flag(root.context().is_null());
    let reg = root.msg_registrator();
    match reg.dynamic_type() {
        Some(kind) => text(kind.name().as_bytes()),
        None => text(b"null"),
    }
    if let Some(keeper) = reg.cast::<ShapeExtendMsgRegistrator>().get() {
        maps(&t, keeper, &[]);
    }
    end();
}

/// `sh_tolerances`: the tolerance of every vertex, edge and face of the table.
fn tolerances(t: &TpShapes) -> Result<(), Failure> {
    for s in &t.shapes {
        match s.shape_type()? {
            ShapeEnum::Vertex => real(BRepTool::vertex_tolerance(&topods::vertex(s)?)?),
            ShapeEnum::Edge => real(BRepTool::edge_tolerance(&topods::edge(s)?)?),
            ShapeEnum::Face => real(BRepTool::face_tolerance(&topods::face(s)?)?),
            _ => {}
        }
    }
    Ok(())
}

/// `sh_fixtol program nops (op)...`.
pub fn fixtol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shapes");
    let tool = ShapeFixShapeTolerance::new();
    for _ in 0..r.integer() {
        let op = r.integer();
        let s = sh_shape(&t, r.integer());
        if op == 0 {
            let (a, b, ty) = (r.real(), r.real(), shape_enum(r.integer()));
            show(tool.limit_tolerance(&s, a, b, ty), flag);
        } else {
            let (p, ty) = (r.real(), shape_enum(r.integer()));
            attempt(|| tool.set_tolerance(&s, p, ty));
        }
    }
    tolerances(&t).expect("probe tolerances");
    end();
}

/// `sh_ec_vertex`: the IsSame index, point and tolerance of a vertex.
fn ec_vertex(t: &TpShapes, v: &Vertex) -> Result<(), Failure> {
    int(tp_same_index(t, v));
    if !v.is_null() {
        pnt(BRepTool::pnt(v)?);
        real(BRepTool::vertex_tolerance(v)?);
    }
    Ok(())
}

/// `sh_ec_digest`: the vertex count and the ends of each edge of `s`.
fn ec_digest(t: &TpShapes, s: &Shape) -> Result<(), Failure> {
    for current in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&current)?;
        int(e.nb_children());
        let (a, b) = topexp::vertices(&e, false);
        ec_vertex(t, &a)?;
        ec_vertex(t, &b)?;
    }
    Ok(())
}

/// `sh_edgeconnect program nops (op)... s`.
pub fn edgeconnect_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shapes");
    let mut tool = ShapeFixEdgeConnect::new();
    for _ in 0..r.integer() {
        match r.integer() {
            0 => {
                let (a, b) = (r.integer(), r.integer());
                attempt(|| {
                    tool.add_edges(&sh_edge(&t, a)?, &sh_edge(&t, b)?);
                    Ok(())
                });
            }
            1 => {
                let s = sh_shape(&t, r.integer());
                attempt(|| tool.add_shape(&s));
            }
            2 => attempt(|| tool.build()),
            _ => tool.clear(),
        }
    }
    let s = sh_shape(&t, r.integer());
    attempt(|| ec_digest(&t, &s));
    end();
}
