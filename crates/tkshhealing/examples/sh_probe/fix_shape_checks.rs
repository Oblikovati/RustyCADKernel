//! Analysis and wire-repair operations of the `sh_fix_shape` probe, the Rust twin of
//! tools/oracle/tkshhealing_probe_fix_shape_checks.cxx (which documents the requests).
use super::brep_lib_common::shape;
use super::common::{Reader, attempt, flag, int, real};
use super::fix_shape::STATUSES;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::{Array2, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::shape_analysis::{
    PinFaceEdgeMap, ShapeAnalysisCheckSmallFace, ShapeAnalysisEdge, ShapeAnalysisFreeBoundData,
    ShapeAnalysisFreeBoundsProperties, SplittingVertexEdges, SplittingVertexParameters,
};
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixSplitTool, ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

/// Family 3: a face on the Bezier surface of `nu nv (pole)*`.
pub(super) fn bezier_face(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let (nu, nv) = (r.integer(), r.integer());
    let mut poles: Array2<'_, Point3> =
        Array2::with_bounds(1, nu, 1, nv).map_err(|e| e.into_failure("probe poles"))?;
    for i in 1..=nu {
        for j in 1..=nv {
            poles
                .set_value(i, j, r.pnt())
                .map_err(|e| e.into_failure("probe poles"))?;
        }
    }
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomBezierSurface::new(&poles)?).cast();
    Ok(BRepBuilderAPIMakeFace::from_surface(&surface, 1e-7)?
        .face()?
        .into_shape())
}

/// Family 4: a face on the XY plane whose wire only collects `n (sh_unify edge)*`; without
/// a wire for n = 0.
pub(super) fn raw_face(r: &mut Reader<'_>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let count = r.integer();
    for _ in 0..count {
        builder.add(&mut wire, &super::unify::edge(r)?.into_shape())?;
    }
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    builder.make_face_on_surface(&mut face, &plane, 1e-7)?;
    if count > 0 {
        builder.add(&mut face, &wire.into_shape())?;
    }
    Ok(face.into_shape())
}

fn edge_digest(edge: &Edge) -> Result<(), Failure> {
    flag(!edge.is_null());
    if edge.is_null() {
        return Ok(());
    }
    shape(edge)
}

fn shapes_of(s: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, kind, ShapeEnum::Shape)?.collect())
}

fn check_statuses(a: &ShapeAnalysisCheckSmallFace) {
    STATUSES.iter().for_each(|&x| flag(a.status_strip(x)));
    STATUSES.iter().for_each(|&x| flag(a.status_pin(x)));
    STATUSES.iter().for_each(|&x| flag(a.status_twisted(x)));
    STATUSES.iter().for_each(|&x| flag(a.status_split_vert(x)));
}

fn split_vertices(a: &mut ShapeAnalysisCheckSmallFace, face: &Face) -> Result<(), Failure> {
    let mut edges = SplittingVertexEdges::default();
    let mut params = SplittingVertexParameters::default();
    let mut all = Compound::default();
    Builder.make_compound(&mut all);
    int(a.check_splitting_vertices(face, &mut edges, &mut params, &mut all)?);
    shape(&all)?;
    for vertex in shapes_of(face, ShapeEnum::Vertex)? {
        flag(edges.is_bound(&vertex));
        if !edges.is_bound(&vertex) {
            continue;
        }
        for edge in edges.find(&vertex)?.iter() {
            shape(edge)?;
        }
        params.find(&vertex)?.iter().for_each(|&p| real(p));
    }
    Ok(())
}

fn pin_face(a: &mut ShapeAnalysisCheckSmallFace, face: &Face, tolerance: f64) {
    let mut map = PinFaceEdgeMap::default();
    attempt(|| {
        flag(a.check_pin_face(face, &mut map, tolerance)?);
        int(map.extent() as i32);
        Ok(())
    });
}

fn has_curve(edge: &Edge) -> Result<bool, Failure> {
    Ok(!BRepTool::curve(edge)?.0.is_null())
}

fn edge_pairs(a: &ShapeAnalysisCheckSmallFace, face: &Face, prec: f64) -> Result<(), Failure> {
    let edges: Vec<Edge> = shapes_of(face, ShapeEnum::Edge)?
        .iter()
        .map(topods::edge)
        .collect::<Result<_, _>>()?;
    for pair in edges.windows(2) {
        if !has_curve(&pair[0])? || !has_curve(&pair[1])? {
            continue;
        }
        let mut d = -1.0;
        for tolerance in [prec, -1.0] {
            attempt(|| {
                flag(a.check_strip_edges(&pair[0], &pair[1], tolerance, &mut d)?);
                real(d);
                Ok(())
            });
        }
        for (first, second, c1, c2, tolerance) in [(0, 1, 5., 7., prec), (1, 0, 4., 40., -1.)] {
            attempt(|| {
                flag(a.check_pin_edges(&pair[first], &pair[second], c1, c2, tolerance)?);
                Ok(())
            });
        }
    }
    Ok(())
}

fn strip_checks(a: &mut ShapeAnalysisCheckSmallFace, face: &Face, prec: f64) {
    let (mut e1, mut e2) = (Edge::default(), Edge::default());
    attempt(|| {
        flag(a.check_single_strip(face, &mut e1, &mut e2, prec)?);
        edge_digest(&e1)?;
        edge_digest(&e2)
    });
    let mut d = -1.0;
    attempt(|| {
        flag(a.find_strip_edges(face, &mut e1, &mut e2, prec, &mut d)?);
        real(d);
        edge_digest(&e1)?;
        edge_digest(&e2)
    });
}

fn surface_checks(a: &mut ShapeAnalysisCheckSmallFace, face: &Face) {
    let (mut row, mut sense) = (-7, -7);
    attempt(|| {
        flag(a.check_pin(face, &mut row, &mut sense)?);
        int(row);
        int(sense);
        Ok(())
    });
    let (mut u, mut v) = (-7.0, -7.0);
    attempt(|| {
        flag(a.check_twisted(face, &mut u, &mut v)?);
        real(u);
        real(v);
        Ok(())
    });
}

fn face_checks(a: &mut ShapeAnalysisCheckSmallFace, face: &Face, prec: f64) -> Result<(), Failure> {
    strip_checks(a, face, prec);
    surface_checks(a, face);
    attempt(|| split_vertices(a, face));
    pin_face(a, face, prec);
    pin_face(a, face, -1.0);
    edge_pairs(a, face, prec)?;
    check_statuses(a);
    Ok(())
}

/// Op 16: the ShapeAnalysis_CheckSmallFace checks on each face.
pub(super) fn small_face_checks(s: &Shape, prec: f64) -> Result<(), Failure> {
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    analyzer.set_tolerance(prec);
    for face in shapes_of(s, ShapeEnum::Face)? {
        face_checks(&mut analyzer, &topods::face(&face)?, prec)?;
    }
    Ok(())
}

fn bound_record(record: &Handle<ShapeAnalysisFreeBoundData>) -> Result<(), Failure> {
    let d = record
        .get()
        .ok_or_else(|| Failure::new("probe: null free-bound record"))?;
    shape(&d.free_bound())?;
    [d.area(), d.perimeter(), d.ratio(), d.width()]
        .into_iter()
        .for_each(real);
    int(d.nb_notches());
    for i in 1..=d.nb_notches() {
        shape(&d.notch(i)?.into_shape())?;
        real(d.notch_width(i)?);
    }
    Ok(())
}

/// Ops 17 and 18: ShapeAnalysis_FreeBoundsProperties on the actual or forecast bounds.
pub(super) fn free_properties(s: &Shape, maxtol: f64, forecast: bool) -> Result<(), Failure> {
    let mut p = ShapeAnalysisFreeBoundsProperties::new();
    if forecast {
        p.init_with_tolerance(s, maxtol, true, true);
    } else {
        p.init(s, false, false);
    }
    flag(p.perform()?);
    int(p.nb_closed_free_bounds());
    int(p.nb_open_free_bounds());
    for i in 1..=p.nb_closed_free_bounds() {
        bound_record(&p.closed_free_bound(i)?)?;
    }
    for i in 1..=p.nb_open_free_bounds() {
        bound_record(&p.open_free_bound(i)?)?;
    }
    Ok(())
}

fn wire_pass(w: &ShapeFixWire, op: i32, prec: f64, maxtol: f64) -> Result<bool, Failure> {
    match op {
        19 => {
            w.set_max_tail_width(maxtol);
            w.set_max_tail_angle(0.3);
            w.fix_tails()
        }
        20 => w.fix_notched_edges(),
        21 | 22 => {
            if op == 22 {
                w.set_remove_loop_mode(1);
            }
            w.fix_self_intersection()
        }
        23 => w.fix_gaps_3d(),
        24 => w.fix_gaps_2d(),
        25 | 29 => {
            w.set_fix_gaps_by_ranges(true);
            if op == 25 {
                w.fix_gaps_3d()
            } else {
                w.fix_gaps_2d()
            }
        }
        26 => w.fix_edge_curves(),
        30 => {
            w.fix_reorder(false)?;
            let first = w.fix_gaps_3d()?;
            Ok(w.fix_gaps_2d()? || first)
        }
        27 => w.fix_closed(prec),
        _ => w.perform(),
    }
}

fn wire_report(w: &ShapeFixWire) -> Result<(), Failure> {
    for &x in &STATUSES {
        flag(w.status_fix_tails(x));
        flag(w.status_notches(x));
        flag(w.status_self_intersection(x));
        flag(w.status_gaps_3d(x));
        flag(w.status_gaps_2d(x));
        flag(w.status_edge_curves(x));
        flag(w.status_closed(x));
    }
    flag(w.status_removed_segment());
    int(w.nb_edges());
    shape(&w.wire()?.into_shape())
}

fn wire_fix(w: &ShapeFixWire, op: i32, prec: f64, maxtol: f64) {
    FixRoot::set_context(w, &Handle::new(ShapeBuildReShape::new()));
    FixRoot::set_precision(w, prec);
    FixRoot::set_max_tolerance(w, maxtol);
    attempt(|| {
        flag(wire_pass(w, op, prec, maxtol)?);
        wire_report(w)
    });
}

/// Ops 19 to 30: one ShapeFix_Wire pass on each face wire, then on each free wire.
pub(super) fn wire_fixes(s: &Shape, prec: f64, maxtol: f64, op: i32) -> Result<(), Failure> {
    for face in shapes_of(s, ShapeEnum::Face)? {
        let face = topods::face(&face)?;
        for wire in shapes_of(&face, ShapeEnum::Wire)? {
            let w = ShapeFixWire::from_wire_face(&topods::wire(&wire)?, &face, prec)?;
            wire_fix(&w, op, prec, maxtol);
        }
    }
    for wire in Explorer::new(s, ShapeEnum::Wire, ShapeEnum::Face)? {
        let w = ShapeFixWire::new();
        w.load(&topods::wire(&wire)?)?;
        wire_fix(&w, op, prec, maxtol);
    }
    Ok(())
}

fn vertex_at(curve: &Handle<dyn GeomCurve>, t: f64, tolerance: f64) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, curve.value(t)?, tolerance)?;
    Ok(vertex)
}

fn split_pair(ok: bool, first: &Edge, second: &Edge, vertex: &Vertex) -> Result<(), Failure> {
    flag(ok);
    if !ok {
        return Ok(());
    }
    shape(first)?;
    shape(second)?;
    real(BRepTool::vertex_tolerance(vertex)?);
    Ok(())
}

struct SplitCase<'a> {
    tool: &'a ShapeFixSplitTool,
    edge: &'a Edge,
    curve: &'a Handle<dyn GeomCurve>,
    face: &'a Face,
    prec: f64,
}

fn split_sequence(case: &SplitCase<'_>, first: f64, last: f64) -> Result<(), Failure> {
    let v1 = vertex_at(case.curve, first, case.prec)?;
    let v2 = vertex_at(case.curve, last, case.prec)?;
    let context = Handle::new(ShapeBuildReShape::new());
    let mut pieces = Sequence::default();
    let mut num = -1;
    attempt(|| {
        let ok = case.tool.split_edge_sequence(
            case.edge,
            first,
            &v1,
            last,
            &v2,
            case.face,
            &mut pieces,
            &mut num,
            &context,
            case.prec,
            1e-9,
        )?;
        flag(ok);
        int(num);
        int(pieces.length());
        for piece in pieces.iter() {
            shape(piece)?;
        }
        shape(&context.apply(case.edge, ShapeEnum::Shape)?)
    });
    Ok(())
}

fn split_edge_cases(
    case: &SplitCase<'_>,
    (f, l): (f64, f64),
    (a, b): (f64, f64),
) -> Result<(), Failure> {
    let d = b - a;
    let (mut e1, mut e2) = (Edge::default(), Edge::default());
    let v = vertex_at(case.curve, (f + l) / 2., case.prec)?;
    attempt(|| {
        let ok = case.tool.split_edge(
            case.edge,
            a + 0.5 * d,
            &v,
            case.face,
            &mut e1,
            &mut e2,
            case.prec,
            1e-9,
        )?;
        split_pair(ok, &e1, &e2, &v)
    });
    let v = vertex_at(case.curve, (f + l) / 2., case.prec)?;
    attempt(|| {
        let ok = case.tool.split_edge_cut(
            case.edge,
            a + 0.3 * d,
            a + 0.6 * d,
            &v,
            case.face,
            &mut e1,
            &mut e2,
            case.prec,
            1e-9,
        )?;
        split_pair(ok, &e1, &e2, &v)
    });
    split_sequence(case, a + 0.2 * d, a + 0.7 * d)?;
    split_sequence(case, a, a + 0.5 * d)?;
    split_sequence(case, a + 0.5 * d, b)
}

fn cut_edge_cases(case: &SplitCase<'_>, (a, b): (f64, f64)) {
    let d = b - a;
    let mut line = false;
    for (end, cut) in [(a, a + 0.25 * d), (b, a + 0.75 * d)] {
        attempt(|| {
            flag(
                case.tool
                    .cut_edge(case.edge, end, cut, case.face, &mut line)?,
            );
            flag(line);
            shape(case.edge)
        });
    }
}

fn split_edge(
    tool: &ShapeFixSplitTool,
    edge: &Edge,
    face: &Face,
    prec: f64,
) -> Result<(), Failure> {
    let (curve, f, l) = BRepTool::curve(edge)?;
    let (pcurve, a, b) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    if curve.is_null() || pcurve.is_null() {
        return Ok(());
    }
    let case = SplitCase {
        tool,
        edge,
        curve: &curve,
        face,
        prec,
    };
    split_edge_cases(&case, (f, l), (a, b))?;
    cut_edge_cases(&case, (a, b));
    Ok(())
}

/// Op 32: ShapeFix_SplitTool on each face edge.
pub(super) fn split_tool(s: &Shape, prec: f64) -> Result<(), Failure> {
    let tool = ShapeFixSplitTool::new();
    for face in shapes_of(s, ShapeEnum::Face)? {
        let face = topods::face(&face)?;
        for edge in shapes_of(&face, ShapeEnum::Edge)? {
            split_edge(&tool, &topods::edge(&edge)?, &face, prec)?;
        }
    }
    Ok(())
}
