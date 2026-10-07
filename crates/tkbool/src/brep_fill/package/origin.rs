// Rust translation of BRepFill::SearchOrigin and the static MakeWire and CutEdge
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill::SearchOrigin`: a closed wire restarted at the point nearest to a given
//! point (cutting an edge there when needed) and run along a given direction.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_extrema::{BRepExtremaDistShapeShape, BRepExtremaSupportType};
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
use std::f64::consts::PI;

/// `MakeWire`: the closed FORWARD wire of the edges from `rangdeb` on (then from the
/// first one), or of the reversed edges walked backwards from `rangdeb`.
fn make_wire(edges: &[Edge], rangdeb: usize, forward: bool) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let n: usize = edges.len();
    let order: Vec<usize> = if forward {
        (rangdeb..=n).chain(1..rangdeb).collect()
    } else {
        (1..=rangdeb).rev().chain((rangdeb + 1..=n).rev()).collect()
    };
    for rang in order {
        let edge: &Edge = &edges[rang - 1];
        let added: Shape = if forward {
            edge.as_ref().clone()
        } else {
            edge.reversed()
        };
        builder.add(&mut wire, &added)?;
    }
    wire.set_orientation(Orientation::Forward);
    wire.set_closed(true)?;
    Ok(wire)
}

/// `CutEdge`: the two FORWARD pieces of an edge at `param` around a new vertex, the
/// first one starting at `vref` (the edge orientation is not kept, as natively).
fn cut_edge(edge: &Edge, param: f64, vref: &Vertex) -> Result<(Edge, Edge), Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let Some(geometry) = curve.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill CutEdge: an edge without 3d curve; expected a curve",
            "BRepFill",
        ));
    };
    let mut vi = Vertex::default();
    BRepBuilder::default().make_vertex_at(
        &mut vi,
        geometry.value(param)?,
        precision::confusion(),
    )?;
    let (vf, vl) = topexp::vertices(edge, false);
    let head =
        BRepLibMakeEdge::from_curve_vertices_params(&curve, &vf, &vi, first, param)?.edge()?;
    let tail =
        BRepLibMakeEdge::from_curve_vertices_params(&curve, &vi, &vl, param, last)?.edge()?;
    if vref.is_same(&vf) {
        return Ok((head, tail));
    }
    Ok((tail, head))
}

/// The origin found on the wire: an existing vertex, or a parameter on an edge where a
/// new vertex cuts it.
struct Origin {
    vertex: Vertex,
    cut: Option<(Edge, f64)>,
}

/// The wire point nearest to `p` (the first of the BRepExtrema solutions at the least
/// distance): its vertex, or the vertex of its edge within `tol` (the other end when
/// `p` is near the first one, a native quirk), else a cut of the edge.
fn nearest_origin(wire: &Wire, p: Point3, tol: f64) -> Result<Origin, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, p, tol)?;
    let dss = BRepExtremaDistShapeShape::with_shapes(&vertex, wire, Default::default())?;
    if !dss.is_done() {
        return Ok(Origin { vertex, cut: None });
    }
    let mut isol: i32 = 1;
    let mut dss_min: f64 = p.distance(&dss.point_on_shape2(isol)?);
    for iss in 2..=dss.nb_solution() {
        if dss_min > p.distance(&dss.point_on_shape2(iss)?) {
            dss_min = p.distance(&dss.point_on_shape2(iss)?);
            isol = iss;
        }
    }
    let support: Shape = dss.support_on_shape2(isol)?;
    if dss.support_type_shape2(isol)? == BRepExtremaSupportType::IsVertex {
        return Ok(Origin {
            vertex: topods::vertex(&support)?,
            cut: None,
        });
    }
    let edge: Edge = topods::edge(&support)?;
    let (vf, vl) = topexp::vertices(&edge, false);
    let mut dist: f64 = p.distance(&BRepTool::pnt(&vf)?);
    if dist < tol {
        vertex = vl.clone();
    }
    let d: f64 = p.distance(&BRepTool::pnt(&vl)?);
    if d < tol && d < dist {
        vertex = vf;
        dist = d;
    }
    if dist > tol {
        let theparam: f64 = dss.par_on_edge_s2(isol)?;
        return Ok(Origin {
            vertex,
            cut: Some((edge, theparam)),
        });
    }
    Ok(Origin { vertex, cut: None })
}

/// The edges of the wire in exploration order (the cut edge replaced by its two
/// pieces) and the rank of the origin edge (the last edge when the origin is not met).
fn origin_edges(wire: &Wire, origin: &Origin) -> Result<(Vec<Edge>, usize), Failure> {
    let mut edges: Vec<Edge> = Vec::new();
    let mut rangdeb: usize = 0;
    let mut exp = WireExplorer::with_wire(wire)?;
    while exp.more() {
        let edge: Edge = exp.current().clone();
        match &origin.cut {
            Some((eref, param)) if edge.is_same(eref) => {
                let (e1, e2) = cut_edge(&edge, *param, exp.current_vertex())?;
                edges.push(e1);
                edges.push(e2);
                rangdeb = edges.len();
            }
            _ => edges.push(edge),
        }
        if origin.cut.is_none() && origin.vertex.is_same(exp.current_vertex()) {
            rangdeb = edges.len();
        }
        exp.next()?;
    }
    if rangdeb == 0 {
        rangdeb = edges.len();
    }
    Ok((edges, rangdeb))
}

/// `BRepFill::SearchOrigin(W, P, Dir, Tol)`: the closed wire `W` rebuilt FORWARD from
/// its point nearest to `p`, running along `dir` when its tangent there makes an angle
/// of at most PI/2 with it, backwards otherwise. Standard_NoSuchObject for an open wire.
///
/// ```ignore
/// rustycad_tkbool::brep_fill::search_origin(&mut wire, point, direction, 1e-7)?;
/// ```
pub fn search_origin(w: &mut Wire, p: Point3, dir: Vector3, tol: f64) -> Result<(), Failure> {
    if !w.closed()? {
        return Err(Failure::with_type(
            "Standard_NoSuchObject",
            "BRepFill::SearchOrigin : the wire must be closed",
            "BRepFill",
        ));
    }
    // To avoid composing the orientations.
    w.set_orientation(Orientation::Forward);
    let origin: Origin = nearest_origin(w, p, tol)?;
    let (edges, rangdeb) = origin_edges(w, &origin)?;
    let Some(edge) = edges.get(rangdeb.wrapping_sub(1)) else {
        return Err(Failure::with_type(
            "Standard_OutOfRange",
            "BRepFill::SearchOrigin: a wire without edge; expected edges",
            "BRepFill",
        ));
    };
    let theparam: f64 = match &origin.cut {
        Some((_, param)) => *param,
        None => BRepTool::parameter(&origin.vertex, edge)?,
    };
    let (_, mut ve) = BRepAdaptorCurve::with_edge(edge)?.d1(theparam)?;
    if edge.orientation() == Orientation::Reversed {
        ve *= -1.;
    }
    let mut angle: f64 = ve.angle(dir)?;
    if angle > PI {
        angle = 2. * PI - angle;
    }
    *w = make_wire(&edges, rangdeb, angle <= PI / 2.)?;
    w.set_closed(true)
}
