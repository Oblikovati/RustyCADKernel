//! BRepGraph oracle, part 5, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_tool.cxx: every `BRepGraph_Tool` member, then
//! the `br_bgraph_walk` case.
use super::brep::{Scene, scene};
use super::brep_graph::{action, curve, index, pcurve, surface};
use super::brep_graph_explore::explore;
use super::brep_graph_walk::{children, dense, edit, related, reverse};
use super::brep_report::{pnt, pnt2d};
use super::common::{Reader, attempt, end, flag, int, real, show};
use rustycad_tkbrep::brep_graph::tool::{self, CoEdgeUsage, FaceUsage, VertexUsage};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorTransformedCurve, GeomAdaptorTransformedSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

fn curve_adaptor(a: GeomAdaptorTransformedCurve) -> Result<(), Failure> {
    let has = a.is_curve_on_surface() || a.geom_curve().is_some();
    flag(!a.is_curve_on_surface());
    flag(has);
    if has {
        attempt(|| {
            let (f, l) = (a.first_parameter()?, a.last_parameter()?);
            real(f);
            real(l);
            pnt(a.value(0.5 * (f + l))?);
            Ok(())
        });
    }
    Ok(())
}
fn surface_adaptor(a: GeomAdaptorTransformedSurface) -> Result<(), Failure> {
    let has = a.geom_surface_original().is_some();
    flag(has);
    flag(a.has_trsf());
    if has {
        attempt(|| {
            let (u1, u2) = (a.first_u_parameter()?, a.last_u_parameter()?);
            let (v1, v2) = (a.first_v_parameter()?, a.last_v_parameter()?);
            [u1, u2, v1, v2].into_iter().for_each(real);
            pnt(a.value(0.25, 0.75)?);
            Ok(())
        });
    }
    Ok(())
}
fn pcurve_adaptor(a: Geom2dAdaptorCurve) -> Result<(), Failure> {
    flag(a.is_initialized());
    if a.is_initialized() {
        attempt(|| {
            let (f, l) = (a.first_parameter()?, a.last_parameter()?);
            real(f);
            real(l);
            pnt2d(a.value(0.5 * (f + l))?);
            Ok(())
        });
    }
    Ok(())
}
fn curve_on_surface(c: Handle<Adaptor3dCurveOnSurface>) -> Result<(), Failure> {
    flag(c.is_null());
    if let Some(c) = c.get() {
        attempt(|| {
            let (f, l) = (c.first_parameter()?, c.last_parameter()?);
            real(f);
            real(l);
            pnt(c.value(0.5 * (f + l))?);
            Ok(())
        });
    }
    Ok(())
}
fn bounds((u1, u2, v1, v2): (f64, f64, f64, f64)) {
    [u1, u2, v1, v2].into_iter().for_each(real);
}

fn vertices(g: &BRepGraph, l: &Location) {
    use tool::vertex as t;
    for i in 0..g.topo().vertices().nb() {
        let v = VertexId::new(i);
        pnt(t::pnt(g, v));
        real(t::tolerance(g, v));
        int(t::nb_edges(g, v) as i32);
        let usage = VertexUsage {
            def_id: v,
            location: l.clone(),
            orientation: Orientation::Forward,
        };
        pnt(t::pnt_usage(g, &usage));
    }
    for i in 0..=g.refs().vertices().nb() {
        let r = VertexRefId::new(i);
        let u = t::usage(g, r);
        flag(u.is_valid());
        index(u.def_id);
        int(u.orientation as i32);
        pnt(t::pnt_ref(g, r));
        real(t::tolerance_ref(g, r));
    }
    end();
}

fn edge_lookups(g: &BRepGraph, e: EdgeId) {
    use tool::edge as t;
    let v0 = tool::vertex::usage(g, t::start_vertex_id(g, e)).def_id;
    let v1 = tool::vertex::usage(g, t::end_vertex_id(g, e)).def_id;
    index(t::find_by_vertices(g, v0, v1, false));
    index(t::find_by_vertices(g, v1, v0, false));
    index(t::find_by_vertices(g, v1, v0, true));
    let mut faces: Vec<FaceId> = g.topo().edges().faces_of(e).collect();
    faces.push(FaceId::new(g.topo().faces().nb()));
    for f in faces {
        index(t::find_co_edge_id(g, e, f));
        index(t::find_pcurve_co_edge_id(g, e, f));
        for o in [Orientation::Forward, Orientation::Reversed] {
            index(t::find_co_edge_id_oriented(g, e, f, o));
            index(t::find_pcurve_co_edge_id_oriented(g, e, f, o));
        }
        flag(t::is_seam_on_face(g, e, f));
    }
}

fn edges(s: &Scene, g: &BRepGraph) {
    use tool::edge as t;
    for i in 0..g.topo().edges().nb() {
        let e = EdgeId::new(i);
        real(t::tolerance(g, e));
        flag(t::degenerated(g, e));
        flag(t::is_closed(g, e));
        let (first, last) = t::range(g, e);
        real(first);
        real(last);
        index(t::start_vertex_id(g, e));
        index(t::end_vertex_id(g, e));
        flag(t::has_curve(g, e));
        curve(s, &t::curve(g, e));
        attempt(|| curve_adaptor(t::curve_adaptor(g, e)?));
        int(t::nb_faces(g, e) as i32);
        flag(t::is_manifold(g, e));
        flag(t::is_boundary(g, e));
        edge_lookups(g, e);
    }
    end();
}

fn co_edge_usage(s: &Scene, g: &BRepGraph, c: CoEdgeId, l: &Location) {
    use tool::edge as t;
    let u = CoEdgeUsage {
        def_id: c,
        location: l.clone(),
        orientation: Orientation::Reversed,
    };
    show(t::curve_usage(g, &u), |found| curve(s, &found));
    attempt(|| curve_adaptor(t::curve_adaptor_usage(g, &u)?));
    attempt(|| pcurve_adaptor(tool::co_edge::pcurve_adaptor_usage(g, &u)?));
    let own = tool::co_edge::face_of(g, c);
    for f in [own, FaceId::new(0)] {
        if f.is_valid_for(g.topo().faces().nb()) {
            attempt(|| curve_on_surface(t::curve_on_surface(g, &u, f)?));
        }
    }
}

fn co_edges(s: &Scene, g: &BRepGraph, l: &Location) {
    use tool::co_edge as t;
    for i in 0..g.topo().co_edges().nb() {
        let c = CoEdgeId::new(i);
        int(t::orientation(g, c) as i32);
        flag(t::is_reversed(g, c));
        index(t::edge_of(g, c));
        index(t::face_of(g, c));
        index(t::seam_pair(g, c));
        flag(t::is_seam(g, c));
        flag(t::has_pcurve(g, c));
        show(t::same_parameter(g, c), flag);
        flag(t::same_range(g, c));
        pcurve(s, &t::pcurve(g, c));
        let (first, last) = t::range(g, c);
        real(first);
        real(last);
        show(t::uv_points(g, c), |(a, b)| {
            pnt2d(a);
            pnt2d(b);
        });
        attempt(|| pcurve_adaptor(t::pcurve_adaptor(g, c)?));
        co_edge_usage(s, g, c, l);
    }
    end();
}

fn face(s: &Scene, g: &BRepGraph, f: FaceId, l: &Location) {
    use tool::face as t;
    let u = FaceUsage {
        def_id: f,
        location: l.clone(),
        orientation: Orientation::Forward,
    };
    let range = (0.0, 1.0, -1.0, 2.0);
    real(t::tolerance(g, f));
    flag(t::has_surface(g, f));
    show(t::outer_wire(g, f), index);
    surface(s, &t::surface(g, f));
    attempt(|| surface_adaptor(t::surface_adaptor(g, f)?));
    attempt(|| surface_adaptor(t::surface_adaptor_usage(g, &u)?));
    attempt(|| surface_adaptor(t::surface_adaptor_bounded(g, f, range)?));
    attempt(|| surface_adaptor(t::surface_adaptor_usage_bounded(g, &u, range)?));
    int(t::nb_wires(g, f) as i32);
    bounds(t::bounds(g, f));
}

fn faces(s: &Scene, g: &BRepGraph, l: &Location) {
    use tool::face as t;
    for i in 0..g.topo().faces().nb() {
        face(s, g, FaceId::new(i), l);
    }
    for i in 0..=g.refs().faces().nb() {
        let r = FaceRefId::new(i);
        let u = t::usage(g, r);
        flag(u.is_valid());
        index(u.def_id);
        int(u.orientation as i32);
        real(t::tolerance_ref(g, r));
        flag(t::has_surface_ref(g, r));
        show(t::outer_wire_ref(g, r), index);
        surface(s, &t::surface_ref(g, r));
        attempt(|| surface_adaptor(t::surface_adaptor_ref(g, r)?));
        attempt(|| surface_adaptor(t::surface_adaptor_ref_bounded(g, r, (0.0, 1.0, -1.0, 2.0))?));
        int(t::nb_wires_ref(g, r) as i32);
        bounds(t::bounds_ref(g, r));
    }
    end();
}

fn wires_shells(g: &BRepGraph) {
    use tool::{shell, wire};
    for i in 0..g.topo().wires().nb() {
        let w = WireId::new(i);
        flag(wire::is_closed(g, w));
        int(wire::nb_co_edges(g, w) as i32);
        int(wire::nb_distinct_edges(g, w) as i32);
        index(wire::face_of(g, w));
        show(wire::is_outer(g, w), flag);
    }
    for i in 0..=g.refs().wires().nb() {
        let r = WireRefId::new(i);
        let u = wire::usage(g, r);
        flag(u.is_valid());
        index(u.def_id);
        int(u.orientation as i32);
        flag(wire::is_closed_ref(g, r));
        int(wire::nb_co_edges_ref(g, r) as i32);
        int(wire::nb_distinct_edges_ref(g, r) as i32);
        index(wire::face_of_ref(g, r));
        show(wire::is_outer_ref(g, r), flag);
    }
    for i in 0..g.topo().shells().nb() {
        let x = ShellId::new(i);
        flag(shell::is_closed(g, x));
        int(shell::nb_faces(g, x) as i32);
    }
    for i in 0..=g.refs().shells().nb() {
        let r = ShellRefId::new(i);
        let u = shell::usage(g, r);
        flag(u.is_valid());
        index(u.def_id);
        int(u.orientation as i32);
        flag(shell::is_closed_ref(g, r));
        int(shell::nb_faces_ref(g, r) as i32);
    }
    end();
}

/// `bt_tool`: usages are placed at the scene's last location.
fn tool_report(s: &Scene, g: &BRepGraph) {
    let l = s.graph.loc(s.graph.locs.len() as i32 - 1);
    vertices(g, &l);
    edges(s, g);
    co_edges(s, g, &l);
    faces(s, g, &l);
    wires_shells(g);
}

/// `br_bgraph_walk script nactions (action)... nedits (edit)...`.
pub fn bgraph_walk_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let mut g = BRepGraph::new();
    for _ in 0..r.integer() {
        action(&mut g, &s, &mut r);
    }
    for _ in 0..r.integer() {
        edit(&s, &mut g, &mut r);
    }
    end();
    dense(&g);
    children(&g);
    reverse(&g);
    related(&g);
    explore(&s, &g);
    tool_report(&s, &g);
}
