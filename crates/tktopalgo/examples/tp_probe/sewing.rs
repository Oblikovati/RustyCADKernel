//! BRepBuilderAPI_Sewing probe operation, the Rust twin of
//! tools/oracle/tktopalgo_probe_sewing.cxx (the request layout is documented there).
use super::brep_lib_common::shape as digest;
use super::brep_lib_package::guarded;
use super::common::{Reader, attempt, captured, flag, int, real};
use super::shape::{TpShape, tp_shape};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPISewing;

/// The MapShapes maps of the input and of the sewn shape, and whether the sewn indices
/// are replaced by presence flags (the order-free summary).
struct SewMaps {
    input: IndexedMapOfShape,
    sewn: IndexedMapOfShape,
    summary: bool,
}

/// `tp_sew_locate`.
fn locate(m: &SewMaps, s: &Shape) {
    if s.is_null() {
        int(-1);
        return;
    }
    int(m.input.find_index(s) as i32);
    if m.summary {
        int(i32::from(m.sewn.contains(s)));
    } else {
        int(m.sewn.find_index(s) as i32);
    }
    int(s.orientation() as i32);
}

/// `tp_sew_point`: the presence, point and tolerance of a vertex.
fn point_tuple(v: &Vertex) -> Result<Vec<u64>, Failure> {
    if v.is_null() {
        return Ok(vec![0]);
    }
    let p = BRepTool::pnt(v)?;
    let tol = BRepTool::vertex_tolerance(v)?;
    Ok(vec![
        1,
        p.x().to_bits(),
        p.y().to_bits(),
        p.z().to_bits(),
        tol.to_bits(),
    ])
}

/// The tuple of an edge: tolerance, degenerated flag, range and sorted end tuples.
fn edge_tuple(e: &Edge) -> Result<Vec<u64>, Failure> {
    let (f, l) = BRepTool::range(e)?;
    let mut t = vec![
        BRepTool::edge_tolerance(e)?.to_bits(),
        u64::from(BRepTool::degenerated(e)?),
        f.to_bits(),
        l.to_bits(),
    ];
    let (a, b) = topexp::vertices(e, false);
    let (mut pa, mut pb) = (point_tuple(&a)?, point_tuple(&b)?);
    if pb < pa {
        std::mem::swap(&mut pa, &mut pb);
    }
    t.extend(pa);
    t.extend(pb);
    Ok(t)
}

/// `tp_sew_summary`: the order-free summary of a shape.
fn sew_summary(s: &Shape) -> Result<(), Failure> {
    if s.is_null() {
        int(-1);
        return Ok(());
    }
    let maps = [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face].map(|kind| {
        let mut m = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(s, kind, &mut m).map(|_| m)
    });
    let [mv, me, mf] = maps;
    let (mv, me, mf) = (mv?, me?, mf?);
    int(mv.extent() as i32);
    int(me.extent() as i32);
    int(mf.extent() as i32);
    int(s.shape_type()? as i32);
    let mut vs: Vec<Vec<u64>> = Vec::new();
    for i in 1..=mv.extent() {
        vs.push(point_tuple(&topods::vertex(mv.find_key(i)?)?)?);
    }
    let mut es: Vec<Vec<u64>> = Vec::new();
    for i in 1..=me.extent() {
        es.push(edge_tuple(&topods::edge(me.find_key(i)?)?)?);
    }
    vs.sort();
    es.sort();
    for u in vs.iter().chain(es.iter()).flatten() {
        print!("{u} ");
    }
    Ok(())
}

/// `tp_sew_nm_vertices`: INTERNAL vertices added to edges of the record.
fn nm_vertices(r: &mut Reader<'_>, shape: &TpShape) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    for _ in 0..r.integer() {
        let mut e = shape.edges[r.integer() as usize].clone();
        let t = r.real();
        let (c, _, _) = BRepTool::curve(&e)?;
        let curve = c
            .get()
            .ok_or_else(|| Failure::new("tp_sew_nm_vertices: no 3d curve"))?;
        let mut v = Vertex::default();
        b.make_vertex_at(&mut v, curve.value(t)?, 1e-7)?;
        e.set_free(true)?;
        b.add(&mut e, &v.oriented(Orientation::Internal))?;
        e.set_free(false)?;
    }
    Ok(())
}

/// `tp_sew_input`.
fn sew_input(sew: &mut BRepBuilderAPISewing, s: &Shape, mode: i32) -> Result<(), Failure> {
    if mode == 1 {
        return sew.add(s);
    }
    if mode == 2 {
        return sew.load(s);
    }
    if s.shape_type()? != ShapeEnum::Compound {
        return sew.add(s);
    }
    let mut first = true;
    let mut it = ShapeIterator::new(s, true, true);
    while let Ok(child) = it.value() {
        let child = child.clone();
        if first && mode == 3 {
            sew.load(&child)?;
        } else {
            sew.add(&child)?;
        }
        first = false;
        it.next();
    }
    Ok(())
}

/// `tp_sew_settings`.
fn settings(sew: &BRepBuilderAPISewing) {
    real(sew.tolerance());
    real(sew.min_tolerance());
    real(sew.max_tolerance());
    flag(sew.face_mode());
    flag(sew.floating_edges_mode());
    flag(sew.same_parameter_mode());
    flag(sew.local_tolerances_mode());
    flag(sew.non_manifold_mode());
}

/// `tp_sew_counts`.
fn counts(sew: &BRepBuilderAPISewing) {
    int(sew.nb_free_edges());
    int(sew.nb_multiple_edges());
    int(sew.nb_contigous_edges());
    int(sew.nb_degenerated_shapes());
    int(sew.nb_deleted_faces());
}

/// `tp_sew_couple`.
fn couple(sew: &BRepBuilderAPISewing, m: &SewMaps, i: i32) -> Result<(), Failure> {
    let list = sew.contigous_edge_couple(i)?;
    int(list.extent());
    for section in list.iter() {
        locate(m, section);
        let e = topods::edge(section)?;
        flag(sew.is_section_bound(&e));
        attempt(|| {
            locate(m, &sew.section_to_boundary(&e)?.into_shape());
            Ok(())
        });
    }
    Ok(())
}

/// `tp_sew_lists`.
fn lists(sew: &BRepBuilderAPISewing, m: &SewMaps) -> Result<(), Failure> {
    for i in 1..=sew.nb_free_edges() {
        locate(m, &sew.free_edge(i)?.into_shape());
    }
    for i in 1..=sew.nb_multiple_edges() {
        locate(m, &sew.multiple_edge(i)?.into_shape());
    }
    for i in 1..=sew.nb_contigous_edges() {
        locate(m, &sew.contigous_edge(i)?.into_shape());
        couple(sew, m, i)?;
    }
    for i in 1..=sew.nb_degenerated_shapes() {
        locate(m, sew.degenerated_shape(i)?);
    }
    for i in 1..=sew.nb_deleted_faces() {
        locate(m, &sew.deleted_face(i)?.into_shape());
    }
    Ok(())
}

/// Prints the located shape of an accessor, or its exception.
fn try_locate(m: &SewMaps, found: Result<Shape, Failure>) {
    attempt(|| {
        locate(m, &found?);
        Ok(())
    });
}

/// `tp_sew_ranges`.
fn ranges(sew: &BRepBuilderAPISewing, m: &SewMaps) {
    try_locate(m, sew.free_edge(0).map(|e| e.into_shape()));
    try_locate(
        m,
        sew.free_edge(sew.nb_free_edges() + 1)
            .map(|e| e.into_shape()),
    );
    try_locate(m, sew.free_edge(-1).map(|e| e.into_shape()));
    try_locate(
        m,
        sew.multiple_edge(sew.nb_multiple_edges() + 1)
            .map(|e| e.into_shape()),
    );
    try_locate(m, sew.contigous_edge(0).map(|e| e.into_shape()));
    attempt(|| {
        int(sew
            .contigous_edge_couple(sew.nb_contigous_edges() + 1)?
            .extent());
        Ok(())
    });
    try_locate(
        m,
        sew.degenerated_shape(sew.nb_degenerated_shapes() + 1)
            .cloned(),
    );
    try_locate(m, sew.deleted_face(0).map(|f| f.into_shape()));
    try_locate(
        m,
        sew.deleted_face(sew.nb_deleted_faces() + 1)
            .map(|f| f.into_shape()),
    );
}

/// `tp_sew_history`.
fn history(sew: &BRepBuilderAPISewing, m: &SewMaps) -> Result<(), Failure> {
    for i in 1..=m.input.extent() {
        let s = m.input.find_key(i)?;
        flag(sew.is_modified(s));
        locate(m, &sew.modified(s));
        attempt(|| {
            flag(sew.is_modified_sub_shape(s)?);
            Ok(())
        });
        try_locate(m, sew.modified_sub_shape(s));
        attempt(|| {
            flag(sew.is_degenerated(s)?);
            Ok(())
        });
        if s.shape_type()? == ShapeEnum::Edge {
            let e = topods::edge(s)?;
            try_locate(m, sew.which_face(&e, 1).map(|f| f.into_shape()));
            try_locate(m, sew.which_face(&e, 2).map(|f| f.into_shape()));
        }
    }
    Ok(())
}

/// The sewing of one request, printed after Perform.
fn sew_case(r: &mut Reader<'_>) -> Result<(), Failure> {
    let shape = tp_shape(r)?;
    nm_vertices(r, &shape)?;
    let tol = r.real();
    let (o1, o2, o3, o4) = (r.flag(), r.flag(), r.flag(), r.flag());
    let (face, floating, samepar) = (r.flag(), r.flag(), r.flag());
    let (mintol, maxtol, local) = (r.real(), r.real(), r.flag());
    let (mode, context, summary) = (r.integer(), r.flag(), r.flag());
    let mut sew = BRepBuilderAPISewing::new(tol, o1, o2, o3, o4);
    let ctx: Handle<dyn ReShape> = Handle::new(BRepToolsReShape::new()).cast();
    if context {
        sew.set_context(&ctx);
    }
    sew.set_face_mode(face);
    sew.set_floating_edges_mode(floating);
    sew.set_same_parameter_mode(samepar);
    if mintol >= 0.0 {
        sew.set_min_tolerance(mintol);
    }
    if maxtol >= 0.0 {
        sew.set_max_tolerance(maxtol);
    }
    sew.set_local_tolerances_mode(local);
    sew_input(&mut sew, &shape.shape, mode)?;
    flag(*sew.get_context() == ctx);
    attempt(|| {
        sew.perform(&ProgressRange::default())?;
        int(1);
        Ok(())
    });
    settings(&sew);
    counts(&sew);
    if summary {
        sew_summary(sew.sewed_shape())?;
    } else {
        digest(sew.sewed_shape())?;
    }
    let mut m = SewMaps {
        input: IndexedMapOfShape::default(),
        sewn: IndexedMapOfShape::default(),
        summary,
    };
    topexp::map_shapes(&shape.shape, &mut m.input, true, true);
    if !sew.sewed_shape().is_null() {
        topexp::map_shapes(sew.sewed_shape(), &mut m.sewn, true, true);
    }
    lists(&sew, &m)?;
    ranges(&sew, &m);
    history(&sew, &m)?;
    captured(|text| {
        let mut bytes: Vec<u8> = Vec::new();
        let result = sew.dump(&mut bytes);
        text.push_str(&String::from_utf8_lossy(&bytes));
        result
    });
    Ok(())
}

/// `bba_sew`.
pub fn sew_request(values: &[f64]) {
    guarded(values, sew_case);
}
