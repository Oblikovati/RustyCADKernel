// Rust translation of the static MakeCircle, MakeOffset and VertexFromNode
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parallels of the elements of the spine before trimming: circles around the
//! vertices, offset curves of the edges extended at their free ends, and the vertices
//! of the bisector nodes lying on the offset.
use super::GeneratedMap;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dOffsetCurve};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::{CurveType, JoinType};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Point3};
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
use rustycad_tktopalgo::mat::MatNode;
use std::f64::consts::PI;

/// The vertices of the bisector nodes on the offset, keyed by node identity
/// (`NCollection_DataMap<Handle(MAT_Node), TopoDS_Shape>`).
#[derive(Default)]
pub(super) struct NodeVertices {
    entries: Vec<(Handle<MatNode>, Vertex)>,
}

impl NodeVertices {
    /// The vertex of a node, if made.
    fn find(&self, node: &Handle<MatNode>) -> Option<Vertex> {
        self.entries
            .iter()
            .find(|(n, _)| n.as_ptr() == node.as_ptr())
            .map(|(_, v)| v.clone())
    }
}

/// `MakeCircle(E, V, F, Offset, Map, RefPlane)`: the circle of radius |Offset| around
/// the vertex, starting along the outer normal of the edge ending there (clockwise for
/// a negative offset), mapped to the vertex.
pub(super) fn make_circle(
    e: &Edge,
    v: &Vertex,
    f_face: &Face,
    offset: f64,
    map: &mut GeneratedMap,
    ref_plane: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    let pc = BRepTool::curve_on_face(e, f_face)?;
    let gc = pc.pcurve.get().ok_or_else(null_pcurve)?;
    let (p, mut dx) = if e.orientation() == Orientation::Forward {
        gc.d1(pc.last)?
    } else {
        gc.d1(pc.first)?
    };
    if e.orientation() == Orientation::Forward {
        dx.reverse();
    }
    let axis = Axis2d::new(p, Direction2::from_vector(dx)?);
    let circ: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(axis, offset.abs(), offset < 0.)?).cast();
    let oe: Shape = BRepLibMakeEdge::from_pcurve(&circ, ref_plane)?
        .edge()?
        .into_shape();
    let mut ll: List<Shape> = List::default();
    ll.append(oe);
    map.add(v.as_ref().clone(), ll);
    Ok(())
}

/// Whether each end of the edge may be extended: not an end of an open result.
fn extensible_ends(e: &Edge, is_open_result: bool, ends: &[Vertex; 2]) -> (bool, bool) {
    if !is_open_result {
        return (true, true);
    }
    let (v1, v2) = topexp::vertices(e, false);
    let free = |v: &Vertex| !(v.is_same(&ends[0]) || v.is_same(&ends[1]));
    (free(&v1), free(&v2))
}

/// The range of the offset circle: extended by a fifth of the missing angle at each
/// free end for the Arc join, else a whole turn from the middle (or from a fixed end)
/// of the missing angle.
fn circle_range(
    (f, l): (f64, f64),
    the_join_type: JoinType,
    (first, last): (bool, bool),
) -> (f64, f64) {
    let delta: f64 = 2. * PI - l + f;
    let (mut f, mut l) = (f, l);
    if the_join_type == JoinType::Arc {
        if first {
            f -= 0.2 * delta;
        }
        if last {
            l += 0.2 * delta;
        }
        return (f, l);
    }
    if first && last {
        f = l + delta / 2.;
        l = f + 2. * PI;
    } else if first {
        f = l;
        l = f + 2. * PI;
    } else if last {
        l = f + 2. * PI;
    }
    (f, l)
}

/// The range of the offset line: extended by its length at each free end for the Arc
/// join, to infinity for the Intersection join.
fn line_range(
    (f, l): (f64, f64),
    the_join_type: JoinType,
    (first, last): (bool, bool),
) -> (f64, f64) {
    let delta: f64 = l - f;
    let arc: bool = the_join_type == JoinType::Arc;
    let f = if !first {
        f
    } else if arc {
        f - delta
    } else {
        -precision::infinite()
    };
    let l = if !last {
        l
    } else if arc {
        l + delta
    } else {
        precision::infinite()
    };
    (f, l)
}

/// The offset of a circle p-curve, `None` when the offset reaches the concave side
/// radius.
fn circle_offset(
    g2d: &Handle<dyn Geom2dCurve>,
    ac: &Geom2dAdaptorCurve,
    an_offset: f64,
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let c1 = ac.circle()?;
    let axes = c1.axis();
    let (xd, yd) = (axes.x_direction(), axes.y_direction());
    let crossed: f64 = xd.x() * yd.y() - xd.y() * yd.x();
    let signe: f64 = if crossed > 0. { -1. } else { 1. };
    if an_offset * signe >= c1.radius() - precision::confusion() {
        return Ok(None);
    }
    let ahc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(g2d)?).cast();
    let off = Adaptor2dOffsetCurve::with_offset(&ahc, an_offset)?;
    Ok(Some(
        Handle::new(Geom2dCircle::from_circ2d(off.circle()?)).cast(),
    ))
}

/// The offset of a line p-curve.
fn line_offset(
    g2d: &Handle<dyn Geom2dCurve>,
    an_offset: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let ahc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(g2d)?).cast();
    let off = Adaptor2dOffsetCurve::with_offset(&ahc, an_offset)?;
    Ok(Handle::new(Geom2dLine::from_lin2d(off.line()?)).cast())
}

/// The trimmed offset p-curve of an edge (None for a circle consumed by the offset).
fn offset_curve(
    e: &Edge,
    f_face: &Face,
    offset: f64,
    the_join_type: JoinType,
    extend: (bool, bool),
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let an_offset: f64 = if e.orientation() == Orientation::Forward {
        -offset
    } else {
        offset
    };
    let pc = BRepTool::curve_on_face(e, f_face)?;
    let (g2d, f, l) = (pc.pcurve, pc.first, pc.last);
    let ac = Geom2dAdaptorCurve::with_range(&g2d, f, l)?;
    let trimmed = |cc: Handle<dyn Geom2dCurve>,
                   (f, l): (f64, f64)|
     -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Ok(Handle::new(Geom2dTrimmedCurve::new(&cc, f, l, true, true)?).cast())
    };
    match ac.get_type()? {
        CurveType::Circle => match circle_offset(&g2d, &ac, an_offset)? {
            Some(cc) => Ok(Some(trimmed(
                cc,
                circle_range((f, l), the_join_type, extend),
            )?)),
            None => Ok(None),
        },
        CurveType::Line => Ok(Some(trimmed(
            line_offset(&g2d, an_offset)?,
            line_range((f, l), the_join_type, extend),
        )?)),
        _ => {
            let g2d_t: Handle<dyn Geom2dCurve> = trimmed(g2d.clone(), (f, l))?;
            Ok(Some(
                Handle::new(Geom2dOffsetCurve::new(&g2d_t, an_offset, false)?).cast(),
            ))
        }
    }
}

/// `MakeOffset(E, F, Offset, Map, RefPlane, IsOpenResult, theJoinType, Ends)`: the
/// parallel edge of the edge on the reference plane, in its orientation, mapped to the
/// edge (none for a circle consumed by the offset).
#[allow(clippy::too_many_arguments)] // The native eight-argument function.
pub(super) fn make_offset(
    e: &Edge,
    f_face: &Face,
    offset: f64,
    map: &mut GeneratedMap,
    ref_plane: &Handle<dyn GeomSurface>,
    is_open_result: bool,
    the_join_type: JoinType,
    ends: &[Vertex; 2],
) -> Result<(), Failure> {
    let extend = extensible_ends(e, is_open_result, ends);
    let Some(g2d_oc) = offset_curve(e, f_face, offset, the_join_type, extend)? else {
        return Ok(());
    };
    let mut oe: Shape = BRepLibMakeEdge::from_pcurve(&g2d_oc, ref_plane)?
        .edge()?
        .into_shape();
    oe.set_orientation(e.orientation());
    let mut ll: List<Shape> = List::default();
    ll.append(oe);
    map.add(e.as_ref().clone(), ll);
    Ok(())
}

/// `VertexFromNode(aNode, Offset, PN, MapNodeVertex, VN)`: the vertex of a finite node
/// at the offset distance (made once per node at its point on Z = 0), `None` otherwise.
pub(super) fn vertex_from_node(
    a_node: &Handle<MatNode>,
    offset: f64,
    pn: Point2,
    map_node_vertex: &mut NodeVertices,
) -> Result<Option<Vertex>, Failure> {
    let node = a_node.get().ok_or_else(null_node)?;
    if node.infinite() || (node.distance() - offset).abs() >= precision::confusion() {
        return Ok(None);
    }
    if let Some(vn) = map_node_vertex.find(a_node) {
        return Ok(Some(vn));
    }
    let mut vn = Vertex::default();
    BRepBuilder::default().make_vertex_at(
        &mut vn,
        Point3::new(pn.x(), pn.y(), 0.),
        precision::confusion(),
    )?;
    map_node_vertex.entries.push((a_node.clone(), vn.clone()));
    Ok(Some(vn))
}

/// Standard_NullObject of a null p-curve.
fn null_pcurve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: null p-curve of a spine edge; expected a curve on the spine",
        "BRepFill_OffsetWire",
    )
}

/// Standard_NullObject of a null node.
fn null_node() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: null bisector node; expected a node",
        "BRepFill_OffsetWire",
    )
}
