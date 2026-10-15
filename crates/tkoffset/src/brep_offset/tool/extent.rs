// Rust translation of BRepOffset_Tool::ExtentFace with the static ExtentEdge and SelectEdge
// of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extension of a face to its neighbour caps: the edges facing a cap replaced by
//! the sections with the enlarged cap, their neighbours extended in tangency, and the
//! wires rebuilt on the new edges and vertices.
use super::enlarge::EnlargeFaceOptions;
use super::inter2d::{project_vertex_on_edge, try_parameter};
use super::pcurves::build_pcurves;
use super::topology::{EdgeNeighbours, VertexEdges};
use super::{BRepOffsetTool, live};
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api;
use rustycad_tkgeombase::geom_lib::extend_curve_to_point;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Plane3, Point2, Point3};

/// A map of shapes to shapes (`ConstShapes`, `ToBuild`, `Build`).
pub type ShapeImages = DataMap<Shape, Shape, ShapeMapHasher>;

/// `ExtentEdge(F, EF, E, NE)`: an empty copy of the edge for a conic or a line; else
/// the edge whose p-curve on `EF` is the p-curve on `F` extended in tangency to the
/// bounds of the surface (within 100).
fn extent_edge(f: &Face, ef: &Face, e: &Edge) -> Result<Edge, Failure> {
    let ce = BRepAdaptorCurve::with_edge(e)?;
    let mut ne = topods::edge(&e.empty_copied()?)?;
    let typ = ce.get_type()?;
    if matches!(
        typ,
        CurveType::Line
            | CurveType::Circle
            | CurveType::Ellipse
            | CurveType::Hyperbola
            | CurveType::Parabola
    ) {
        return Ok(ne);
    }
    let p_max = 1.0e2;
    let (s, _) = BRepTool::surface_with_location(f)?;
    let (umin, umax, vmin, vmax) = live(&s, "surface")?.bounds();
    let bounds = [
        umin.max(-p_max),
        umax.min(p_max),
        vmin.max(-p_max),
        vmax.min(p_max),
    ];
    let c2d = BRepTool::curve_on_face(e, f)?.pcurve;
    let pf2d = target_point(&c2d, ce.first_parameter()?, bounds, -1.)?;
    let pl2d = target_point(&c2d, ce.last_parameter()?, bounds, 1.)?;
    let xoy = Plane3::default();
    let cc = geom_api::to_3d(&c2d, &xoy)?;
    let mut ext_c: Handle<dyn GeomBoundedCurve> = cc.cast();
    if ext_c.is_null() {
        return Ok(ne);
    }
    extend_curve_to_point(&mut ext_c, Point3::new(pf2d.x(), pf2d.y(), 0.), 1, false)?;
    extend_curve_to_point(&mut ext_c, Point3::new(pl2d.x(), pl2d.y(), 0.), 1, true)?;
    let cne2d = geom_api::to_2d(&ext_c.cast::<dyn GeomCurve>(), &xoy)?;
    let builder = BRepBuilder::default();
    builder.make_edge(&mut ne)?;
    builder.update_edge_pcurve_on_face(&ne, &cne2d, ef, BRepTool::edge_tolerance(e)?)?;
    let c = live(&cne2d, "pcurve")?;
    builder.range(&ne, c.first_parameter(), c.last_parameter(), false)?;
    ne.set_orientation(e.orientation());
    Ok(ne)
}

/// The point of the tangent line at the parameter where it meets the nearest bound,
/// backward (`sign` -1) or forward (`sign` 1).
fn target_point(
    c2d: &Handle<dyn Geom2dCurve>,
    u: f64,
    [umin, umax, vmin, vmax]: [f64; 4],
    sign: f64,
) -> Result<Point2, Failure> {
    let (p, mut tang) = live(c2d, "pcurve")?.d1(u)?;
    let (mut tx, mut ty) = (precision::infinite(), precision::infinite());
    if tang.x().abs() > precision::confusion() {
        tx = ((umax - p.x()) / tang.x())
            .abs()
            .min(((umin - p.x()) / tang.x()).abs());
    }
    if tang.y().abs() > precision::confusion() {
        ty = ((vmax - p.y()) / tang.y())
            .abs()
            .min(((vmin - p.y()) / tang.y()).abs());
    }
    tang *= tx.min(ty);
    Ok(Point2::new(
        p.x() + sign * tang.x(),
        p.y() + sign * tang.y(),
    ))
}

/// `SelectEdge(F, EF, E, LInt)`: of several section edges, the one whose ends are the
/// nearest to the ends of the edge (the last of equal sums).
fn select_edge(e: &Edge, l_int: &mut List<Shape>) -> Result<(), Failure> {
    let mut du = 1.0e100;
    let mut ge = Shape::default();
    let (fst, lst) = BRepTool::range(e)?;
    let ad1 = BRepAdaptorCurve::with_edge(e)?;
    let (p_first, p_last) = (ad1.value(fst)?, ad1.value(lst)?);
    for ei in l_int.iter() {
        let ei_edge = topods::edge(ei)?;
        let (fst, lst) = BRepTool::range(&ei_edge)?;
        let ad2 = BRepAdaptorCurve::with_edge(&ei_edge)?;
        let tmp = ad2.value(fst)?.distance(&p_first) + ad2.value(lst)?.distance(&p_last);
        if tmp <= du {
            du = tmp;
            ge = ei.clone();
        }
    }
    l_int.clear();
    l_int.append(ge);
    Ok(())
}

/// The state of `ExtentFace` shared by its steps.
struct ExtentState<'a> {
    f: &'a Face,
    ef: Face,
    side: State,
    tol_conf: f64,
    build: ShapeImages,
}

impl BRepOffsetTool {
    /// `ExtentFace(F, ConstShapes, ToBuild, Side, TolConf, NF)`: the face extended to its
    /// caps (`ToBuild` binds each edge facing a cap to the cap): its surface enlarged
    /// (the p-curves of its edges and their images moved onto it), the facing edges
    /// replaced by sections with the enlarged caps, their neighbours extended, the new
    /// vertices intersected or projected, and the wires rebuilt (`ConstShapes` binds
    /// the images); a partial face when an intersection vertex is missing;
    /// `BRepOffsetTool::extent_face(&face, &mut const_shapes, &mut to_build, State::Out, 1e-7, &mut nf)?`.
    pub fn extent_face(
        f: &Face,
        const_shapes: &mut ShapeImages,
        to_build: &mut ShapeImages,
        side: State,
        tol_conf: f64,
        nf: &mut Face,
    ) -> Result<(), Failure> {
        let mut ef = Face::default();
        let surface_change =
            Self::en_large_face(f, &mut ef, EnlargeFaceOptions::with_change_geom(true))?;
        *nf = topods::face(&ef.empty_copied()?)?;
        nf.set_orientation(Orientation::Forward);
        if surface_change {
            move_pcurves_to_enlarged(f, &ef, const_shapes)?;
        }
        let mut state = ExtentState {
            f,
            ef,
            side,
            tol_conf,
            build: ShapeImages::default(),
        };
        let mut exp = Explorer::new(
            &f.oriented(Orientation::Forward),
            ShapeEnum::Wire,
            ShapeEnum::Shape,
        )?;
        while exp.more() {
            let w = topods::wire(exp.current())?;
            exp.next()?;
            if !extent_wire(&mut state, &w, const_shapes, to_build, nf)? {
                return Ok(());
            }
        }
        nf.set_orientation(f.orientation());
        brep_tools::update(nf.as_ref())
    }
}

/// The p-curves of the edges of the face (and of their images) moved onto the surface
/// of the enlarged face.
fn move_pcurves_to_enlarged(
    f: &Face,
    ef: &Face,
    const_shapes: &ShapeImages,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let fforward = topods::face(&f.oriented(Orientation::Forward))?;
    let mut emap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(fforward.as_ref(), ShapeEnum::Edge, &mut emap)?;
    for i in 1..=emap.extent() {
        let ce = topods::edge(&emap.find_key(i)?.oriented(Orientation::Forward))?;
        let pc = BRepTool::curve_on_face(&ce, &fforward)?;
        if pc.pcurve.is_null() {
            continue;
        }
        let (mut first, mut last) = (pc.first, pc.last);
        let mut ecs = Edge::default();
        if let Ok(image) = const_shapes.find(ce.as_ref()) {
            ecs = topods::edge(image)?;
            (first, last) = BRepTool::range(&ecs)?;
        }
        let tol = BRepTool::edge_tolerance(&ce)?;
        if BRepTool::is_closed_on_face(&ce, &fforward)? {
            let c2r = BRepTool::curve_on_face(&topods::edge(&ce.reversed())?, &fforward)?;
            (first, last) = (c2r.first, c2r.last);
            builder.update_edge_pcurves_on_face(&ce, (&pc.pcurve, &c2r.pcurve), ef, tol)?;
            if !ecs.is_null() {
                builder.update_edge_pcurves_on_face(&ecs, (&pc.pcurve, &c2r.pcurve), ef, tol)?;
            }
        } else {
            builder.update_edge_pcurve_on_face(&ce, &pc.pcurve, ef, tol)?;
            if !ecs.is_null() {
                builder.update_edge_pcurve_on_face(&ecs, &pc.pcurve, ef, tol)?;
            }
        }
        builder.range(&ce, first, last, false)?;
        if !ecs.is_null() {
            builder.range(&ecs, first, last, false)?;
        }
    }
    Ok(())
}

/// The edges of a wire in explorer order of its FORWARD orientation.
fn wire_edges(w: &Wire) -> Result<Vec<Edge>, Failure> {
    let mut edges = Vec::new();
    let mut exp = Explorer::new(
        &w.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while exp.more() {
        edges.push(topods::edge(exp.current())?);
        exp.next()?;
    }
    Ok(edges)
}

/// One wire of `ExtentFace`: the edges, the vertices and the rebuilt wire added to the
/// new face; false when an intersection vertex is missing (the native early return).
fn extent_wire(
    state: &mut ExtentState,
    w: &Wire,
    const_shapes: &mut ShapeImages,
    to_build: &mut ShapeImages,
    nf: &mut Face,
) -> Result<bool, Failure> {
    let mut mve = VertexEdges::default();
    let (mut n_on_v1, mut n_on_v2) = (EdgeNeighbours::default(), EdgeNeighbours::default());
    BRepOffsetTool::map_vertex_edges(w.as_ref(), &mut mve)?;
    BRepOffsetTool::build_neighbour(w, state.f, &mut n_on_v1, &mut n_on_v2)?;
    let edges = wire_edges(w)?;
    let (mut l_int1, mut l_int2) = (List::default(), List::default());
    for e in &edges {
        drop_projected_edge(state, e, const_shapes, to_build, (&mut l_int1, &mut l_int2))?;
    }
    for e in &edges {
        build_section_edge(
            state,
            e,
            const_shapes,
            to_build,
            (&n_on_v1, &n_on_v2),
            (&mut l_int1, &mut l_int2),
        )?;
    }
    for e in &edges {
        if !build_edge_vertices(state, e, const_shapes, to_build, (&n_on_v1, &n_on_v2), &mve)? {
            return Ok(false);
        }
    }
    let nw = rebuild_wire(state, &edges, const_shapes, nf)?;
    BRepBuilder::default().add(nf, &nw.oriented(w.orientation()))?;
    Ok(true)
}

/// The first edge loop of a wire: an edge with an image needs no cap; an edge lying on
/// its cap (the projection onto the cap succeeds) needs no section either.
fn drop_projected_edge(
    state: &ExtentState,
    e: &Edge,
    const_shapes: &ShapeImages,
    to_build: &mut ShapeImages,
    (l_int1, l_int2): (&mut List<Shape>, &mut List<Shape>),
) -> Result<(), Failure> {
    if const_shapes.is_bound(e.as_ref()) {
        to_build.unbind(e.as_ref());
    }
    let Ok(cap) = to_build.find(e.as_ref()).cloned() else {
        return Ok(());
    };
    let mut loe = List::default();
    loe.append(e.as_ref().clone());
    let projected = BRepOffsetTool::try_project(
        &topods::face(&cap)?,
        &state.ef,
        &loe,
        l_int2,
        l_int1,
        state.side,
        state.tol_conf,
    )?;
    if projected && !l_int1.is_empty() {
        to_build.unbind(e.as_ref());
    }
    Ok(())
}

/// The second edge loop of a wire: the section of the enlarged face with the enlarged
/// cap replaces an edge facing a cap, and its free neighbours are extended.
fn build_section_edge(
    state: &mut ExtentState,
    e: &Edge,
    const_shapes: &ShapeImages,
    to_build: &mut ShapeImages,
    (n_on_v1, n_on_v2): (&EdgeNeighbours, &EdgeNeighbours),
    (l_int1, l_int2): (&mut List<Shape>, &mut List<Shape>),
) -> Result<(), Failure> {
    if const_shapes.is_bound(e.as_ref()) {
        to_build.unbind(e.as_ref());
    }
    let Ok(cap) = to_build.find(e.as_ref()).cloned() else {
        return Ok(());
    };
    let mut stop_face = Face::default();
    BRepOffsetTool::en_large_face(
        &topods::face(&cap)?,
        &mut stop_face,
        EnlargeFaceOptions::with_change_geom(false),
    )?;
    let null_face = Face::default();
    BRepOffsetTool::inter3d(
        &state.ef, &stop_face, l_int1, l_int2, state.side, e, &null_face, &null_face,
    )?;
    if l_int1.is_empty() {
        return Ok(());
    }
    if l_int1.len() > 1 {
        select_edge(e, l_int1)?;
    }
    let ne = topods::edge(l_int1.first().ok_or_else(empty_list)?)?;
    let tedge = edge_tshape(ne.as_ref(), "ExtentFace")?;
    tedge.set_tolerance(tedge.tolerance() * 10.);
    let oriented = if ne.orientation() == e.orientation() {
        Orientation::Forward
    } else {
        Orientation::Reversed
    };
    state.build.bind(e.as_ref().clone(), ne.oriented(oriented));
    for neighbours in [n_on_v1, n_on_v2] {
        let e_on_v = topods::edge(neighbours.find(e.as_ref())?)?;
        if !to_build.is_bound(e_on_v.as_ref())
            && !const_shapes.is_bound(e_on_v.as_ref())
            && !state.build.is_bound(e_on_v.as_ref())
        {
            let extended = extent_edge(state.f, &state.ef, &e_on_v)?;
            state.build.bind(
                e_on_v.as_ref().clone(),
                extended.oriented(Orientation::Forward),
            );
        }
    }
    Ok(())
}

/// The third edge loop of a wire: the new vertices of a built edge at both ends,
/// intersected with a built neighbour or projected; false when an intersection fails.
fn build_edge_vertices(
    state: &mut ExtentState,
    e: &Edge,
    const_shapes: &mut ShapeImages,
    to_build: &ShapeImages,
    (n_on_v1, n_on_v2): (&EdgeNeighbours, &EdgeNeighbours),
    mve: &VertexEdges,
) -> Result<bool, Failure> {
    if !state.build.is_bound(e.as_ref()) {
        return Ok(true);
    }
    let (v1, v2) = topexp::vertices(e, false);
    for (vertex, neighbours, first) in [(&v1, n_on_v1, true), (&v2, n_on_v2, false)] {
        let Some(v) = end_vertex(
            state,
            e,
            vertex,
            neighbours,
            first,
            const_shapes,
            to_build,
            mve,
        )?
        else {
            return Ok(false);
        };
        const_shapes.bind(vertex.as_ref().clone(), v.as_ref().clone());
        state.build.bind(vertex.as_ref().clone(), v.into_shape());
    }
    Ok(true)
}

/// The new vertex of a built edge at one end: the end of the built edge for a closed
/// one, the intersection with the built neighbour (or the projection of its known
/// image at a vertex of more than two edges), or the projection of the old vertex.
#[allow(clippy::too_many_arguments)]
fn end_vertex(
    state: &ExtentState,
    e: &Edge,
    vertex: &Vertex,
    neighbours: &EdgeNeighbours,
    first: bool,
    const_shapes: &ShapeImages,
    to_build: &ShapeImages,
    mve: &VertexEdges,
) -> Result<Option<Vertex>, Failure> {
    let built_e = topods::edge(state.build.find(e.as_ref())?)?;
    let ne_on_v = topods::edge(neighbours.find(e.as_ref())?)?;
    let intersect = state.build.is_bound(ne_on_v.as_ref())
        && (to_build.is_bound(e.as_ref()) || to_build.is_bound(ne_on_v.as_ref()));
    if !intersect {
        let mut v = vertex.clone();
        if let Ok(image) = const_shapes.find(vertex.as_ref()) {
            v = topods::vertex(image)?;
        }
        v.set_orientation(end_orientation(first, &built_e));
        if !try_parameter(e, &v, &built_e, state.tol_conf)? {
            project_vertex_on_edge(&v, &built_e, state.tol_conf)?;
        }
        return Ok(Some(v));
    }
    if e.is_same(&ne_on_v) {
        let end = if first {
            topexp::first_vertex(&built_e, false)
        } else {
            topexp::last_vertex(&built_e, false)
        };
        return Ok(Some(end));
    }
    if let Ok(image) = state.build.find(vertex.as_ref()) {
        let mut v = topods::vertex(image)?;
        if mve.find(vertex.as_ref())?.len() > 2 {
            v.set_orientation(end_orientation(first, &built_e));
            project_vertex_on_edge(&v, &built_e, state.tol_conf)?;
        }
        return Ok(Some(v));
    }
    let mut lv = List::default();
    let built_neighbour = topods::edge(state.build.find(ne_on_v.as_ref())?)?;
    BRepOffsetTool::inter2d(
        &state.ef,
        &built_e,
        &built_neighbour,
        &mut lv,
        precision::confusion(),
    )?;
    if lv.is_empty() {
        return Ok(None);
    }
    let take_first = (built_e.orientation() == Orientation::Forward) == first;
    let chosen = if take_first { lv.first() } else { lv.last() };
    Ok(Some(topods::vertex(chosen.ok_or_else(empty_list)?)?))
}

/// The orientation of an end vertex of a built edge: FORWARD at its first vertex,
/// REVERSED at its last, swapped on a REVERSED built edge.
fn end_orientation(first: bool, built_e: &Edge) -> Orientation {
    let o = if first {
        Orientation::Forward
    } else {
        Orientation::Reversed
    };
    if built_e.orientation() == Orientation::Reversed {
        return top_abs::reverse(o);
    }
    o
}

/// The fourth edge loop of a wire: the wire of the built edges between their new
/// vertices, the images and the original edges.
fn rebuild_wire(
    state: &mut ExtentState,
    edges: &[Edge],
    const_shapes: &mut ShapeImages,
    nf: &Face,
) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut nw = Wire::default();
    builder.make_wire(&mut nw);
    for e in edges {
        let ne = if state.build.is_bound(e.as_ref()) {
            let ne = rebuild_built_edge(state, e, const_shapes)?;
            state.build.unbind(e.as_ref());
            ne
        } else if let Ok(image) = const_shapes.find(e.as_ref()).cloned() {
            let mut ne = topods::edge(&image)?;
            build_pcurves(&ne, nf)?;
            if ne.orientation() == Orientation::Reversed {
                ne.set_orientation(top_abs::reverse(e.orientation()));
            } else {
                ne.set_orientation(e.orientation());
            }
            ne
        } else {
            const_shapes.bind(e.as_ref().clone(), e.oriented(Orientation::Forward));
            e.clone()
        };
        builder.add(&mut nw, ne.as_ref())?;
    }
    Ok(nw)
}

/// A built edge copied empty between its new vertices (a closed edge on its whole
/// range), oriented to the old edge, bound as the image of the old edge.
fn rebuild_built_edge(
    state: &ExtentState,
    e: &Edge,
    const_shapes: &mut ShapeImages,
) -> Result<Edge, Failure> {
    let (v1, v2) = topexp::vertices(e, false);
    let built = topods::edge(state.build.find(e.as_ref())?)?;
    let (f, l) = BRepTool::range(&built)?;
    let or = built.orientation();
    let nv1 = topods::vertex(const_shapes.find(v1.as_ref())?)?;
    let nv2 = topods::vertex(const_shapes.find(v2.as_ref())?)?;
    let mut u1 = BRepTool::parameter(
        &topods::vertex(&nv1.oriented(Orientation::Internal))?,
        &topods::edge(&built.oriented(Orientation::Internal))?,
    )?;
    let mut u2 = BRepTool::parameter(
        &topods::vertex(&nv2.oriented(Orientation::Internal))?,
        &topods::edge(&built.oriented(Orientation::Forward))?,
    )?;
    let mut ne = topods::edge(&built.empty_copied()?)?;
    ne.set_orientation(Orientation::Forward);
    let eps = precision::confusion();
    let builder = BRepBuilder::default();
    let add_range =
        |ne: &mut Edge, first: &Vertex, last: &Vertex, a: f64, b: f64| -> Result<(), Failure> {
            builder.add(ne, &first.oriented(Orientation::Forward))?;
            builder.add(ne, &last.oriented(Orientation::Reversed))?;
            builder.range(ne, a, b, false)
        };
    let reversed_image: bool;
    if nv1.is_same(&nv2) {
        (u1, u2) = if or == Orientation::Forward {
            (f, l)
        } else {
            (l, f)
        };
        if or == Orientation::Forward {
            if u1 > u2 {
                if (u1 - l).abs() < eps {
                    u1 = f;
                }
                if (u2 - f).abs() < eps {
                    u2 = l;
                }
            }
            add_range(&mut ne, &nv1, &nv2, u1, u2)?;
            reversed_image = false;
        } else {
            if u2 > u1 {
                if (u2 - l).abs() < eps {
                    u2 = f;
                }
                if (u1 - f).abs() < eps {
                    u1 = l;
                }
            }
            add_range(&mut ne, &nv2, &nv1, u2, u1)?;
            reversed_image = true;
        }
    } else if or == Orientation::Forward {
        if u1 > u2 {
            add_range(&mut ne, &nv2, &nv1, u2, u1)?;
        } else {
            add_range(&mut ne, &nv1, &nv2, u1, u2)?;
        }
        reversed_image = false;
    } else if u2 > u1 {
        add_range(&mut ne, &nv1, &nv2, u1, u2)?;
        reversed_image = false;
    } else {
        add_range(&mut ne, &nv2, &nv1, u2, u1)?;
        reversed_image = true;
    }
    if reversed_image {
        const_shapes.bind(e.as_ref().clone(), ne.oriented(Orientation::Reversed));
        ne.set_orientation(top_abs::reverse(e.orientation()));
    } else {
        const_shapes.bind(e.as_ref().clone(), ne.as_ref().clone());
        ne.set_orientation(e.orientation());
    }
    Ok(ne)
}

/// Standard_NoSuchObject of an empty list.
fn empty_list() -> Failure {
    Failure::with_message(
        "Standard_NoSuchObject",
        "BRepOffset_Tool::ExtentFace: empty list",
    )
}
