// Rust translation of the static helpers ComputeAveragePlaneAndMaxDeviation,
// UpdateSectionEdge, FindEdgeCloseToBisectorPlane, FindMiddleEdges and FindCommonVertex
// (on two edges) (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometric choices among the section edges: average planes, the edges closest to
//! the bisector plane and the trimming of a section edge at a new vertex.
use super::gap::vertex_edge_map;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::UniformAbscissa;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Plane3, Point3, Vector3};
use std::f64::consts::PI;

/// The average plane of a wire and the largest distance of its sample points to it.
pub(super) struct AveragePlane {
    /// The plane (`None` when the points are on a line; the native plane is then left
    /// unchanged).
    pub(super) plane: Option<Plane3>,
    /// The largest deviation, -1 when the points are on a line.
    pub(super) deviation: f64,
}

/// `ComputeAveragePlaneAndMaxDeviation(aWire, thePlane, IsSingular)`: the inertia plane
/// of 40 uniform points per edge (of the 41 of the distribution) and the largest
/// distance of the points to it.
pub(super) fn compute_average_plane_and_max_deviation(
    a_wire: &Shape,
) -> Result<AveragePlane, Failure> {
    const N: i32 = 40;
    let nedges: i32 = a_wire.nb_children();
    let mut pnts: Array1<'static, Point3> =
        Array1::with_bounds(1, nedges * N).map_err(crate::brep_fill::shape_table::table_error)?;
    let mut ind: i32 = 1;
    let mut iter = ShapeIterator::new(a_wire, true, true);
    while iter.more() {
        let a_curve = BRepAdaptorCurve::with_edge(&topods::edge(iter.value()?)?)?;
        let curve: &dyn Adaptor3dCurve = &a_curve;
        let distribution = UniformAbscissa::with_nb_points(curve, N + 1, -1.)?;
        for i in 1..=N {
            *pnts
                .change_value(ind)
                .map_err(crate::brep_fill::shape_table::table_error)? =
                a_curve.value(distribution.parameter(i)?)?;
            ind += 1;
        }
        iter.next();
    }
    let inertia = geom_lib::axe_of_inertia(&pnts, 1e-7)?;
    if inertia.is_singular {
        return Ok(AveragePlane {
            plane: None,
            deviation: -1.,
        });
    }
    let the_plane = Plane3::from_position(Axis3::from_axis2(inertia.axe));
    let max_deviation: f64 = pnts.iter().fold(0., |max, p| {
        if the_plane.distance(*p) > max {
            the_plane.distance(*p)
        } else {
            max
        }
    });
    Ok(AveragePlane {
        plane: Some(the_plane),
        deviation: max_deviation,
    })
}

/// `UpdateSectionEdge(theEdge, theConstVertex, theVertex, theParam)`: the edge cut at
/// `the_param`, its end other than `the_const_vertex` replaced by `the_vertex`.
pub(super) fn update_section_edge(
    the_edge: &Edge,
    the_const_vertex: &Vertex,
    the_vertex: &mut Vertex,
    the_param: f64,
) -> Result<(), Failure> {
    let mut f_edge: Edge = Edge::unchecked(the_edge.oriented(Orientation::Forward));
    let (v1, v2) = topexp::vertices(&f_edge, false);
    let (or_of_vertex, another_vertex) = if the_const_vertex.is_same(&v1) {
        (Orientation::Reversed, v2)
    } else {
        (Orientation::Forward, v1)
    };
    let builder = BRepBuilder::default();
    let (mut fpar, mut lpar) = BRepTool::range(&f_edge)?;
    if or_of_vertex == Orientation::Forward {
        fpar = the_param;
    } else {
        lpar = the_param;
    }
    builder.range(&f_edge, fpar, lpar, false)?;
    f_edge.set_free(true)?;
    builder.remove(&mut f_edge, &another_vertex)?;
    the_vertex.set_orientation(or_of_vertex);
    builder.add(&mut f_edge, the_vertex)
}

/// The angle between an edge and the bisector plane: through its average plane, or
/// (for a straight edge) through its chord.
fn bisector_angle(an_edge: &Edge, the_axis: &Axis1) -> Result<f64, Failure> {
    let builder = BRepBuilder::default();
    let mut a_wire = Wire::default();
    builder.make_wire(&mut a_wire);
    builder.add(&mut a_wire, an_edge)?;
    let average = compute_average_plane_and_max_deviation(&a_wire)?;
    let Some(a_pln) = average.plane else {
        let ba_curve = BRepAdaptorCurve::with_edge(an_edge)?;
        let first_pnt = ba_curve.value(ba_curve.first_parameter()?)?;
        let last_pnt = ba_curve.value(ba_curve.last_parameter()?)?;
        let edge_dir = Direction3::from_vector(Vector3::between(first_pnt, last_pnt))?;
        let mut an_angle: f64 = edge_dir.angle(&the_axis.direction());
        if an_angle > PI / 2. {
            an_angle = PI - an_angle;
        }
        return Ok(PI / 2. - an_angle);
    };
    let an_angle: f64 = a_pln.axis().angle(*the_axis);
    Ok(if an_angle > PI / 2. {
        PI - an_angle
    } else {
        an_angle
    })
}

/// `FindEdgeCloseToBisectorPlane(theVertex, theComp, theAxis)`: the edge of the
/// compound at the vertex closest angularly to the bisector plane (the only one when
/// there is one), removed from the compound; null when the vertex is not in it.
pub(super) fn find_edge_close_to_bisector_plane(
    the_vertex: &Vertex,
    the_comp: &mut Compound,
    the_axis: &Axis1,
) -> Result<Edge, Failure> {
    let ve_map = vertex_edge_map(the_comp)?;
    let Some(edges) = ve_map.seek(the_vertex) else {
        return Ok(Edge::default());
    };
    let mut min_edge = Edge::default();
    if edges.extent() == 1 {
        min_edge = topods::edge(edges.first().unwrap_or(&Shape::default()))?;
    } else {
        let mut min_angle: f64 = f64::MAX;
        for e in edges.iter() {
            let an_edge: Edge = topods::edge(e)?;
            let an_angle: f64 = bisector_angle(&an_edge, the_axis)?;
            if an_angle < min_angle {
                min_angle = an_angle;
                min_edge = an_edge;
            }
        }
    }
    BRepBuilder::default().remove(the_comp, &min_edge)?;
    Ok(min_edge)
}

/// `FindMiddleEdges(theVertex1, theVertex2, theAxis, theComp, theElist)`: the chain of
/// edges closest to the bisector plane from the first to the second vertex (taken out
/// of the compound); false when the chain breaks.
pub(super) fn find_middle_edges(
    the_vertex1: &Vertex,
    the_vertex2: &Vertex,
    the_axis: &Axis1,
    the_comp: &mut Compound,
    the_elist: &mut List<Shape>,
) -> Result<bool, Failure> {
    let ve_map = vertex_edge_map(the_comp)?;
    if ve_map.extent() == 0 || !ve_map.contains(the_vertex1) || !ve_map.contains(the_vertex2) {
        return Ok(false);
    }
    let mut cur_vertex: Vertex = the_vertex1.clone();
    loop {
        let cur_edge: Edge = find_edge_close_to_bisector_plane(&cur_vertex, the_comp, the_axis)?;
        if cur_edge.is_null() {
            return Ok(false);
        }
        let (v1, v2) = topexp::vertices(&cur_edge, false);
        cur_vertex = if v1.is_same(&cur_vertex) { v2 } else { v1 };
        the_elist.append(cur_edge.into_shape());
        if cur_vertex.is_same(the_vertex2) {
            return Ok(true);
        }
    }
}

/// `FindCommonVertex(theFirstEdge, theLastEdge, theFirstVertex, theLastVertex,
/// theCommonVertex)`: the vertex shared by the two edges; when both ends are the same
/// vertex, the other end of the first edge if the last edge has it.
pub(super) fn find_common_vertex_edges(
    the_first_edge: &Edge,
    the_last_edge: &Edge,
    the_first_vertex: &Vertex,
    the_last_vertex: &Vertex,
) -> Option<Vertex> {
    if !the_first_vertex.is_same(the_last_vertex) {
        return topexp::common_vertex(the_first_edge, the_last_edge);
    }
    let (v1, v2) = topexp::vertices(the_first_edge, false);
    let (v3, v4) = topexp::vertices(the_last_edge, false);
    let other: Vertex = if v1.is_same(the_first_vertex) { v2 } else { v1 };
    (other.is_same(&v3) || other.is_same(&v4)).then_some(other)
}
