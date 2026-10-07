//! The 2d fillets and chamfers of ChFi2d (`fl_chfi2d_*`); the twin of
//! tools/oracle/tkfillet_probe_chfi2d.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::brep_fillet_api::BRepFilletAPIMakeFillet2d;
use rustycad_tkfillet::chfi2d::{
    ChFi2dAnaFilletAlgo, ChFi2dBuilder, ChFi2dChamferAPI, ChFi2dFilletAPI, ChFi2dFilletAlgo,
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gc::MakeArcOfCircle;
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

/// `fl_edge`.
fn edge(e: &Edge) {
    flag(e.is_null());
    if e.is_null() {
        return;
    }
    int(e.orientation() as i32);
    attempt(|| {
        let c = BRepAdaptorCurve::with_edge(e)?;
        int(c.get_type()? as i32);
        let (f, l) = (c.first_parameter()?, c.last_parameter()?);
        real(f);
        real(l);
        pnt(c.value(0.5 * (f + l))?);
        Ok(())
    });
    let (v1, v2) = topexp::vertices(e, true);
    for v in [v1, v2] {
        flag(v.is_null());
        if !v.is_null() {
            pnt(BRepTool::pnt(&v).expect("vertex point"));
        }
    }
}

/// `fl_edge_spec`.
fn edge_spec(r: &mut Reader<'_>) -> Edge {
    let kind = r.integer();
    let (p1, pm, p2) = (r.pnt(), r.pnt(), r.pnt());
    if kind == 1 {
        let arc: Handle<dyn GeomCurve> = MakeArcOfCircle::through_points(p1, pm, p2)
            .and_then(|m| m.value())
            .expect("arc")
            .cast();
        return BRepBuilderAPIMakeEdge::from_curve(&arc)
            .and_then(|m| m.edge())
            .expect("arc edge");
    }
    BRepBuilderAPIMakeEdge::from_points(p1, p2)
        .and_then(|m| m.edge())
        .expect("segment")
}

/// `fl_chfi2d_fillet`: see tools/oracle/tkfillet_probe_chfi2d.cxx.
pub fn fillet_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mode = r.integer();
    let (e1, e2) = (edge_spec(r), edge_spec(r));
    let (radius, p, isol) = (r.real(), r.pnt(), r.integer());
    let pln = Plane3::default();
    let (mut r1, mut r2, mut f) = (Edge::default(), Edge::default(), Edge::default());
    attempt(|| {
        f = match mode {
            0 | 1 => {
                let mut api = ChFi2dFilletAPI::new();
                if mode == 0 {
                    api.init_wire(
                        &BRepBuilderAPIMakeWire::from_two_edges(&e1, &e2)?.wire()?,
                        &pln,
                    )?;
                } else {
                    api.init_edges(&e1, &e2, &pln)?;
                }
                flag(api.perform(radius)?);
                int(api.nb_results(p)?);
                api.result(p, &mut r1, &mut r2, isol)?
            }
            2 => {
                let mut algo = ChFi2dAnaFilletAlgo::with_edges(&e1, &e2, &pln)?;
                flag(algo.perform(radius)?);
                algo.result(&mut r1, &mut r2)
            }
            _ => {
                let mut algo = ChFi2dFilletAlgo::with_edges(&e1, &e2, &pln)?;
                flag(algo.perform(radius)?);
                int(algo.nb_results(p)?);
                algo.result(p, &mut r1, &mut r2, isol)?
            }
        };
        Ok(())
    });
    edge(&f);
    edge(&r1);
    edge(&r2);
    end();
}

/// `fl_chfi2d_chamfer`: see tools/oracle/tkfillet_probe_chfi2d.cxx.
pub fn chamfer_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mode = r.integer();
    let (e1, e2) = (edge_spec(r), edge_spec(r));
    let (l1, l2) = (r.real(), r.real());
    let (mut r1, mut r2, mut f) = (Edge::default(), Edge::default(), Edge::default());
    attempt(|| {
        let mut api = ChFi2dChamferAPI::new();
        if mode == 0 {
            api.init_wire(&BRepBuilderAPIMakeWire::from_two_edges(&e1, &e2)?.wire()?)?;
        } else {
            api.init_edges(&e1, &e2);
        }
        flag(api.perform()?);
        f = api.result(&mut r1, &mut r2, (l1, l2))?;
        Ok(())
    });
    edge(&f);
    edge(&r1);
    edge(&r2);
    end();
}

/// `fl_face_edges`.
fn face_edges(face: &Face) -> Result<(), Failure> {
    flag(face.is_null());
    if face.is_null() {
        return Ok(());
    }
    int(face.orientation() as i32);
    let mut edges = Vec::new();
    let mut ex = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex.more() {
        edges.push(topods::edge(ex.current())?);
        ex.next()?;
    }
    int(edges.len() as i32);
    edges.iter().for_each(edge);
    Ok(())
}

/// One builder operation on the current result.
fn builder_op(
    b: &mut ChFi2dBuilder,
    (op, a, bb, x, y): (i32, i32, i32, f64, f64),
) -> Result<(), Failure> {
    let (mut vs, mut es) = (IndexedMapOfShape::default(), IndexedMapOfShape::default());
    topexp::map_shapes_of_type(&b.result(), ShapeEnum::Vertex, &mut vs)?;
    topexp::map_shapes_of_type(&b.result(), ShapeEnum::Edge, &mut es)?;
    let v = |i: i32| -> Result<Vertex, Failure> {
        topods::vertex(vs.find_key(usize::try_from(i).unwrap_or(0))?)
    };
    let e = |i: i32| -> Result<Edge, Failure> {
        topods::edge(es.find_key(usize::try_from(i).unwrap_or(0))?)
    };
    let nth = |list: &[rustycad_tkbrep::topods::Shape], i: i32| -> Result<Edge, Failure> {
        let shape = usize::try_from(i - 1)
            .ok()
            .and_then(|k| list.get(k))
            .ok_or_else(|| Failure::out_of_range("index"))?;
        topods::edge(shape)
    };
    let removed = |v: Vertex| {
        flag(v.is_null());
        if !v.is_null() {
            pnt(BRepTool::pnt(&v).expect("vertex point"));
        }
    };
    match op {
        1 => edge(&b.add_fillet(&v(a)?, x)?),
        2 => edge(&b.add_chamfer(&e(a)?, &e(bb)?, x, y)?),
        3 => edge(&b.add_chamfer_angle(&e(a)?, &v(bb)?, x, y)?),
        4 => edge(&b.modify_fillet(&nth(b.fillet_edges(), a)?, x)?),
        5 => removed(b.remove_fillet(&nth(b.fillet_edges(), a)?)?),
        6 => edge(&b.modify_chamfer_angle(&nth(b.chamfer_edges(), a)?, &e(bb)?, x, y)?),
        7 => removed(b.remove_chamfer(&nth(b.chamfer_edges(), a)?)?),
        _ => edge(&b.modify_chamfer(&nth(b.chamfer_edges(), a)?, &e(bb)?, &e(bb)?, x, y)?),
    }
    Ok(())
}

/// `fl_make_fillet2d`: BRepFilletAPI_MakeFillet2d initialized on the reference face and
/// the face the builder modified.
fn make_fillet2d(face: &Face, modified: &Face) -> Result<(), Failure> {
    let mut api = BRepFilletAPIMakeFillet2d::new();
    api.init_modified(face, modified)?;
    int(api.status() as i32);
    api.build(&ProgressRange::default())?;
    flag(api.is_done());
    int(api.nb_fillet());
    int(api.nb_chamfer());
    for i in 1..=api.nb_curves() {
        edge(&topods::edge(api.new_edges(i)?.first().ok_or_else(
            || Failure::with_message("Standard_NoSuchObject", "NCollection_List::First"),
        )?)?);
    }
    let mut ex = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while ex.more() {
        let e = topods::edge(ex.current())?;
        flag(api.is_modified(&e));
        if api.has_descendant(&e) {
            edge(&topods::edge(
                api.modified(ex.current())?.first().ok_or_else(|| {
                    Failure::with_message("Standard_NoSuchObject", "NCollection_List::First")
                })?,
            )?);
            edge(&api.basis_edge(&api.descendant_edge(&e)?)?);
        }
        ex.next()?;
    }
    Ok(())
}

/// `fl_chfi2d_builder`: see tools/oracle/tkfillet_probe_chfi2d.cxx.
pub fn builder_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let n = r.integer();
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for _ in 0..n {
        poly.add_point(r.pnt()).expect("polygon point");
    }
    poly.close().expect("closed polygon");
    let wire = poly.wire().expect("polygon wire");
    let face = BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .and_then(|m| m.face())
        .expect("face");
    let mut b = ChFi2dBuilder::with_face(&face).expect("builder");
    int(b.status() as i32);
    for _ in 0..r.integer() {
        let op = (r.integer(), r.integer(), r.integer(), r.real(), r.real());
        attempt(|| builder_op(&mut b, op));
        int(b.status() as i32);
        int(b.nb_fillet());
        int(b.nb_chamfer());
    }
    attempt(|| face_edges(&b.result()));
    attempt(|| make_fillet2d(&face, &b.result()));
    end();
}
