//! The BRep_Tool reports of a scene, the Rust twin of
//! tools/oracle/tkbrep_probe_brep_report.cxx. A query that reaches the planar
//! projection (Standard_NotImplemented here, a projected curve natively) prints `P`.
use super::brep::{Scene, id, scene};
use super::common::{Reader, end, failure, flag, int, masked, real, show, text, word};
use super::graph::{chain, desc};
use rustycad_tkbrep::brep::{
    BRepCurveOnClosedSurface, BRepCurveOnSurface, BRepCurveRepresentation, BRepGCurve,
    BRepPointRepresentation, BRepTEdge, BRepTFace, BRepTVertex, BRepTool, EdgePCurve,
};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::toploc::Location;

fn pnt(p: Point3) {
    real(p.x());
    real(p.y());
    real(p.z());
}
fn pnt2d(p: Point2) {
    real(p.x());
    real(p.y());
}

fn representation_geometry(s: &Scene, c: &dyn BRepCurveRepresentation) {
    let g = &s.geometry;
    show(c.curve3d(), |h| int(id(&g.curves, &h)));
    show(c.surface(), |h| int(id(&g.surfaces, &h)));
    show(c.pcurve(), |h| int(id(&g.pcurves, &h)));
    show(c.pcurve2(), |h| int(id(&g.pcurves, &h)));
    show(c.polygon3d(), |h| int(id(&g.polygons3d, &h)));
    show(c.polygon(), |h| int(id(&g.polygons2d, &h)));
    show(c.polygon2(), |h| int(id(&g.polygons2d, &h)));
    show(c.triangulation(), |h| int(id(&g.triangulations, &h)));
    show(c.polygon_on_triangulation(), |h| int(id(&g.nodes, &h)));
    show(c.polygon_on_triangulation2(), |h| int(id(&g.nodes, &h)));
    show(c.surface2(), |h| int(id(&g.surfaces, &h)));
    show(c.location2(), |l| chain(&s.graph, &l));
    show(c.continuity(), |c| int(c as i32));
}

fn representation(s: &Scene, handle: &Handle<dyn BRepCurveRepresentation>) {
    let c = handle.get().unwrap();
    word(c.dynamic_type().name());
    let kinds = [
        c.is_curve3d(),
        c.is_curve_on_surface(),
        c.is_regularity(),
        c.is_curve_on_closed_surface(),
        c.is_polygon3d(),
        c.is_polygon_on_triangulation(),
        c.is_polygon_on_closed_triangulation(),
        c.is_polygon_on_surface(),
        c.is_polygon_on_closed_surface(),
    ];
    kinds.into_iter().for_each(flag);
    chain(&s.graph, &c.location());
    representation_geometry(s, c);
    if let Some(gc) = handle.cast::<dyn BRepGCurve>().get() {
        real(gc.first());
        real(gc.last());
        // D0 of a null 3D curve is a null dereference natively.
        let null_curve3d = gc.is_curve3d() && gc.curve3d().is_ok_and(|c| c.is_null());
        if !precision::is_infinite(gc.first()) && !null_curve3d {
            show(gc.d0(gc.first()), pnt);
        }
    }
    if let Some(cs) = handle.cast::<BRepCurveOnSurface>().get() {
        let (a, b) = cs.uv_points();
        pnt2d(a);
        pnt2d(b);
    }
    if let Some(cc) = handle.cast::<BRepCurveOnClosedSurface>().get() {
        let (a, b) = cc.uv_points2();
        pnt2d(a);
        pnt2d(b);
    }
}

fn point_representation(s: &Scene, p: &dyn BRepPointRepresentation) {
    let g = &s.geometry;
    word(p.dynamic_type().name());
    flag(p.is_point_on_curve());
    flag(p.is_point_on_curve_on_surface());
    flag(p.is_point_on_surface());
    real(p.parameter());
    show(p.parameter2(), real);
    chain(&s.graph, &p.location());
    show(p.curve(), |h| int(id(&g.curves, &h)));
    show(p.pcurve(), |h| int(id(&g.pcurves, &h)));
    show(p.surface(), |h| int(id(&g.surfaces, &h)));
}

fn vertex_report(s: &Scene, v: &Vertex) {
    show(BRepTool::pnt(v), pnt);
    real(BRepTool::vertex_tolerance(v).unwrap());
    let tv = v.tshape().cast::<BRepTVertex>();
    let tv = tv.get().unwrap();
    real(tv.tolerance());
    pnt(tv.pnt());
    let points = tv.points();
    int(points.len() as i32);
    for p in &points {
        point_representation(s, p.get().unwrap());
    }
}

/// `br_cos`: one curve on surface result.
fn cos(s: &Scene, r: &rustycad_tkbrep::brep::EdgeCurveOnSurface) {
    int(id(&s.geometry.pcurves, &r.pcurve));
    int(id(&s.geometry.surfaces, &r.surface));
    chain(&s.graph, &r.location);
    real(r.first);
    real(r.last);
}

fn polygon_on_surface_result(
    s: &Scene,
    r: &(
        Handle<rustycad_tkmath::poly::Polygon2D>,
        Handle<dyn GeomSurface>,
        Location,
    ),
) {
    int(id(&s.geometry.polygons2d, &r.0));
    int(id(&s.geometry.surfaces, &r.1));
    chain(&s.graph, &r.2);
}

fn polygon_on_triangulation_result(
    s: &Scene,
    r: &(
        Handle<rustycad_tkmath::poly::PolygonOnTriangulation>,
        Handle<rustycad_tkmath::poly::Triangulation>,
        Location,
    ),
) {
    int(id(&s.geometry.nodes, &r.0));
    int(id(&s.geometry.triangulations, &r.1));
    chain(&s.graph, &r.2);
}

fn edge_lists(s: &Scene, e: &Edge) {
    cos(s, &BRepTool::first_curve_on_surface(e).unwrap());
    for i in 0..5 {
        cos(s, &BRepTool::curve_on_surface_by_index(e, i).unwrap());
    }
    polygon_on_surface_result(s, &BRepTool::first_polygon_on_surface(e).unwrap());
    for i in 0..4 {
        polygon_on_surface_result(s, &BRepTool::polygon_on_surface_by_index(e, i).unwrap());
    }
    polygon_on_triangulation_result(s, &BRepTool::first_polygon_on_triangulation(e).unwrap());
    for i in 0..4 {
        polygon_on_triangulation_result(
            s,
            &BRepTool::polygon_on_triangulation_by_index(e, i).unwrap(),
        );
    }
}

fn curve_value(c: &dyn GeomCurve, u: f64) {
    show(c.value(u), pnt);
}

fn edge_curves(s: &Scene, e: &Edge) {
    let g = &s.geometry;
    let found = BRepTool::curve_with_location(e).unwrap();
    int(id(&g.curves, &found.curve));
    chain(&s.graph, &found.location);
    real(found.first);
    real(found.last);
    let (c, f, l) = BRepTool::curve(e).unwrap();
    let cid = id(&g.curves, &c);
    int(cid);
    if cid == -2 {
        let c = c.get().unwrap();
        if !precision::is_infinite(f) {
            curve_value(c, f);
        }
        if !precision::is_infinite(l) {
            curve_value(c, l);
        }
    }
    let (p, l) = BRepTool::polygon3d(e).unwrap();
    int(id(&g.polygons3d, &p));
    chain(&s.graph, &l);
}

fn edge_report(s: &Scene, e: &Edge) {
    let te = e.tshape().cast::<BRepTEdge>();
    let te = te.get().unwrap();
    real(BRepTool::edge_tolerance(e).unwrap());
    real(te.tolerance());
    flag(BRepTool::same_parameter(e).unwrap());
    flag(BRepTool::same_range(e).unwrap());
    flag(BRepTool::degenerated(e).unwrap());
    flag(BRepTool::is_geometric_edge(e).unwrap());
    let (f, l) = BRepTool::range(e).unwrap();
    real(f);
    real(l);
    edge_curves(s, e);
    edge_lists(s, e);
    flag(BRepTool::has_continuity(e).unwrap());
    int(BRepTool::max_continuity(e).unwrap() as i32);
    let curves = te.curves();
    int(curves.len() as i32);
    for c in &curves {
        representation(s, c);
    }
}

fn face_report(s: &Scene, f: &Face) {
    let g = &s.geometry;
    let tf = f.tshape().cast::<BRepTFace>();
    let tf = tf.get().unwrap();
    let (sf, l) = BRepTool::surface_with_location(f).unwrap();
    int(id(&g.surfaces, &sf));
    chain(&s.graph, &l);
    let copy = BRepTool::surface(f).unwrap();
    let sid = id(&g.surfaces, &copy);
    int(sid);
    if sid == -2 {
        show(copy.get().unwrap().value(0.25, 0.75), pnt);
    }
    real(BRepTool::face_tolerance(f).unwrap());
    real(tf.tolerance());
    flag(BRepTool::natural_restriction(f).unwrap());
    flag(BRepTool::is_geometric_face(f).unwrap());
    for p in [0, 1, 2, 4, 8, 16, 17, 32, 48] {
        let (t, l) = BRepTool::triangulation(f, p).unwrap();
        int(id(&g.triangulations, &t));
        chain(&s.graph, &l);
    }
    let (ts, l) = BRepTool::triangulations(f).unwrap();
    int(ts.len() as i32);
    for t in &ts {
        int(id(&g.triangulations, t));
    }
    chain(&s.graph, &l);
    int(tf.nb_triangulations());
    int(id(&g.triangulations, &tf.active_triangulation()));
}

/// Whether a failure is the unported projection step.
fn is_projection(e: &Failure) -> bool {
    e.exception_type() == "Standard_NotImplemented"
}

/// `br_pcurve_result` for a Rust result.
fn pcurve_result(s: &Scene, r: Result<EdgePCurve, Failure>) {
    match r {
        Err(e) if is_projection(&e) => word("P"),
        Err(e) => failure(&e),
        Ok(r) => {
            int(id(&s.geometry.pcurves, &r.pcurve));
            real(r.first);
            real(r.last);
            flag(r.is_stored);
        }
    }
}

fn edge_on_surface(s: &Scene, e: &Edge, sf: &Handle<dyn GeomSurface>, l: &Location) {
    pcurve_result(s, BRepTool::curve_on_surface(e, sf, l));
    let plane = BRepTool::curve_on_plane(e, sf, l).map(|(pcurve, first, last)| EdgePCurve {
        pcurve,
        first,
        last,
        is_stored: false,
    });
    pcurve_result(s, plane);
    int(id(
        &s.geometry.polygons2d,
        &BRepTool::polygon_on_surface(e, sf, l).unwrap(),
    ));
    flag(BRepTool::is_closed_on_surface(e, sf, l).unwrap());
    let (f, la) = BRepTool::range_on_surface(e, sf, l).unwrap();
    real(f);
    real(la);
    show(BRepTool::uv_points(e, sf, l), |(a, b)| {
        pnt2d(a);
        pnt2d(b);
    });
}

fn edge_on_face(s: &Scene, e: &Edge, f: &Face) {
    pcurve_result(s, BRepTool::curve_on_face(e, f));
    int(id(
        &s.geometry.polygons2d,
        &BRepTool::polygon_on_face(e, f).unwrap(),
    ));
    flag(BRepTool::is_closed_on_face(e, f).unwrap());
    let (a, b) = BRepTool::range_on_face(e, f).unwrap();
    real(a);
    real(b);
    show(BRepTool::uv_points_on_face(e, f), |(a, b)| {
        pnt2d(a);
        pnt2d(b);
    });
}

fn is_type(shape: &Shape, kind: ShapeEnum) -> bool {
    !shape.is_null() && shape.shape_type().unwrap() == kind
}

fn edge_pairs(s: &Scene, e: &Edge) {
    let g = &s.graph;
    for l in g.locs.iter().take(3) {
        for sf in &s.geometry.surfaces {
            edge_on_surface(s, e, sf, l);
        }
        for t in &s.geometry.triangulations {
            int(id(
                &s.geometry.nodes,
                &BRepTool::polygon_on_triangulation(e, t, l).unwrap(),
            ));
            flag(BRepTool::is_closed_on_triangulation(e, t, l).unwrap());
        }
    }
    for b in g.shapes.iter().filter(|b| is_type(b, ShapeEnum::Face)) {
        let f = topods::face(b).unwrap();
        edge_on_face(s, e, &f);
        for c in g.shapes.iter().filter(|c| is_type(c, ShapeEnum::Face)) {
            let f2 = topods::face(c).unwrap();
            flag(BRepTool::has_continuity_on_faces(e, &f, &f2).unwrap());
            int(BRepTool::continuity_on_faces(e, &f, &f2).unwrap() as i32);
        }
    }
}

/// `br_vertex_at_end`: the vertex is a FORWARD or REVERSED vertex of the edge.
fn vertex_at_end(v: &Vertex, e: &Edge) -> bool {
    let mut found = Shape::default();
    let mut it = ShapeIterator::new(&e.oriented(Orientation::Forward), true, true);
    while let Ok(child) = it.value() {
        if v.is_same(child) && (found.is_null() || child.orientation() == v.orientation()) {
            found = child.clone();
        }
        it.next();
    }
    !found.is_null()
        && (found.orientation() == Orientation::Forward
            || found.orientation() == Orientation::Reversed)
}

fn vertex_pairs(s: &Scene, v: &Vertex) {
    let g = &s.graph;
    for b in g.shapes.iter().filter(|b| !b.is_null()) {
        if is_type(b, ShapeEnum::Face) {
            show(BRepTool::parameters(v, &topods::face(b).unwrap()), pnt2d);
        }
        if !is_type(b, ShapeEnum::Edge) {
            continue;
        }
        let e = topods::edge(b).unwrap();
        show(BRepTool::find_parameter(v, &e), |found| {
            flag(found.is_some());
            real(found.unwrap_or(0.0));
        });
        for c in g.shapes.iter().filter(|c| is_type(c, ShapeEnum::Face)) {
            let (sf, l) = BRepTool::surface_with_location(&topods::face(c).unwrap()).unwrap();
            match BRepTool::parameter_on_surface(v, &e, &sf, &l) {
                Err(err) if is_projection(&err) && !vertex_at_end(v, &e) => word("P"),
                r => show(r, real),
            }
        }
    }
}

fn pairs_report(s: &Scene) {
    for a in s
        .graph
        .shapes
        .iter()
        .filter(|a| is_type(a, ShapeEnum::Edge))
    {
        edge_pairs(s, &topods::edge(a).unwrap());
    }
    for a in s
        .graph
        .shapes
        .iter()
        .filter(|a| is_type(a, ShapeEnum::Vertex))
    {
        vertex_pairs(s, &topods::vertex(a).unwrap());
    }
}

fn shape_report(s: &Scene, sh: &Shape) {
    desc(&s.graph, sh);
    if sh.is_null() {
        return;
    }
    show(BRepTool::is_closed(sh), flag);
    for t in [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex] {
        show(BRepTool::max_tolerance(sh, t), real);
    }
    match sh.shape_type().unwrap() {
        ShapeEnum::Vertex => vertex_report(s, &topods::vertex(sh).unwrap()),
        ShapeEnum::Edge => edge_report(s, &topods::edge(sh).unwrap()),
        ShapeEnum::Face => face_report(s, &topods::face(sh).unwrap()),
        _ => {}
    }
}

/// `br_brep script`.
pub fn brep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    for sh in &s.graph.shapes {
        shape_report(&s, sh);
    }
    pairs_report(&s);
    for t in &s.geometry.triangulations {
        int(t.mesh_purpose() as i32);
    }
    end();
}

/// `br_brep_dump script`.
pub fn brep_dump_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    for sh in &s.graph.shapes {
        let Some(tshape) = sh.tshape().get() else {
            word("N");
            continue;
        };
        for depth in -1..=2 {
            let mut o = DumpStream::default();
            tshape.dump_json(&mut o, depth).unwrap();
            text(&masked(o.bytes()));
        }
    }
    end();
}
