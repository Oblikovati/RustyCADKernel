// Rust translation of the static IsClosedShape, CreateNewNodes and IsMergedVertices
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The merge of chains of nearest nodes into new nodes, and the tests that forbid
//! gluing the two ends of one short closed chain of edges.
use super::{SewingShapeIndexedMap, SewingShapeListMap, SewingShapeMap};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::{Point3, Xyz};
use std::f64::consts::PI;

/// The static `IsClosedShape(theshape, v1, v2)`: the length of the edges of the shape
/// that start at `v1` or `v2` (up to the first one that also ends there) is more than
/// 1.2 pi times the distance of the two vertices.
pub(super) fn is_closed_shape(shape: &Shape, v1: &Shape, v2: &Shape) -> Result<bool, Failure> {
    let mut tot_length: f64 = 0.0;
    for e in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&e)?;
        if edge.is_null() {
            continue;
        }
        let (ve1, ve2) = topexp::vertices(&edge, false);
        if !ve1.is_same(v1) && !ve1.is_same(v2) {
            continue;
        }
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        let (c3d, first, last) = BRepTool::curve(&edge)?;
        if c3d.is_null() {
            continue;
        }
        let adaptor = GeomAdaptorCurve::with_curve(&c3d)?;
        tot_length += AbscissaPoint::length_between(&adaptor, first, last)?;
        if ve2.is_same(v1) || ve2.is_same(v2) {
            break;
        }
    }
    if tot_length <= 0.0 {
        return Ok(false);
    }
    let p1 = BRepTool::pnt(&topods::vertex(v1)?)?;
    let p2 = BRepTool::pnt(&topods::vertex(v2)?)?;
    Ok(p1.distance(&p2) < tot_length / (1.2 * PI))
}

/// The wires of the face that hold `e1` and `e2` (the first found for each).
fn wires_of(face: &Shape, e1: &Shape, e2: &Shape) -> Result<(Shape, Shape), Failure> {
    let (mut wire1, mut wire2) = (Shape::default(), Shape::default());
    let mut itw = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?;
    while itw.more() && (wire1.is_null() || wire2.is_null()) {
        let wire = itw.current().clone();
        let mut ite = ShapeIterator::new(&wire, false, true);
        while (wire1.is_null() || wire2.is_null())
            && let Ok(edge) = ite.value()
        {
            if wire1.is_null() && e1.is_same(edge) {
                wire1 = wire.clone();
            }
            if wire2.is_null() && e2.is_same(edge) {
                wire2 = wire.clone();
            }
            ite.next();
        }
        itw.next()?;
    }
    Ok((wire1, wire2))
}

/// The static `IsMergedVertices(face1, e1, e2, vtx1, vtx2)`: 1 when the two vertices
/// bound one short closed chain of the same wire (they may not be glued), -1 when the
/// edges are on different wires of the face, 0 otherwise; for a floating edge (null
/// face), 1 unless the edge is closed on the two vertices.
pub(super) fn is_merged_vertices(
    face1: &Shape,
    e1: &Shape,
    e2: &Shape,
    vtx1: &Shape,
    vtx2: &Shape,
) -> Result<i32, Failure> {
    if face1.is_null() {
        return Ok(i32::from(!is_closed_shape(e1, vtx1, vtx2)?));
    }
    let (wire1, wire2) = wires_of(face1, e1, e2)?;
    if wire1.is_null() || wire2.is_null() {
        return Ok(0);
    }
    if !wire1.is_same(&wire2) {
        return Ok(-1);
    }
    let mut it = ShapeIterator::new(&wire1, false, true);
    while let Ok(edge) = it.value() {
        let (ve1, ve2) = topexp::vertices(&topods::edge(edge)?, false);
        if (ve1.is_same(vtx1) && ve2.is_same(vtx2)) || (ve2.is_same(vtx1) && ve1.is_same(vtx2)) {
            return Ok(if is_closed_shape(edge, vtx1, vtx2)? {
                0
            } else {
                1
            });
        }
        it.next();
    }
    if !is_closed_shape(&wire1, vtx1, vtx2)? {
        return Ok(1);
    }
    let (v1, v2) = topexp::wire_vertices(&topods::wire(&wire1)?);
    let is_end_vertex =
        (v1.is_same(vtx1) && v2.is_same(vtx2)) || (v2.is_same(vtx1) && v1.is_same(vtx2));
    Ok(if is_end_vertex { 0 } else { 1 })
}

/// The chains of nearest nodes: old node -> new node, and new node -> old nodes.
#[derive(Default)]
struct NodeChains {
    old_new: SewingShapeMap,
    new_old: IndexedDataMapOfShapeListOfShape,
}

impl NodeChains {
    /// Links a pair of nearest nodes: joins their chains, extends one, or starts a new
    /// node.
    fn link(&mut self, oldnode1: &Shape, oldnode2: &Shape) -> Result<(), Failure> {
        let new1 = self.old_new.seek(oldnode1).cloned();
        let new2 = self.old_new.seek(oldnode2).cloned();
        match (new1, new2) {
            (Some(newnode1), Some(newnode2)) => {
                if newnode1.is_same(&newnode2) {
                    return Ok(());
                }
                self.new_old.change_from_key(&newnode1)?;
                let moved: Vec<Shape> = self
                    .new_old
                    .find_from_key(&newnode2)?
                    .iter()
                    .cloned()
                    .collect();
                for node2 in moved {
                    self.new_old
                        .change_from_key(&newnode1)?
                        .append(node2.clone());
                    *self.old_new.change_find(&node2)? = newnode1.clone();
                }
                self.new_old.remove_key(&newnode2);
            }
            (Some(newnode1), None) => {
                self.old_new.bind(oldnode2.clone(), newnode1.clone());
                self.new_old
                    .change_from_key(&newnode1)?
                    .append(oldnode2.clone());
            }
            (None, Some(newnode2)) => {
                self.old_new.bind(oldnode1.clone(), newnode2.clone());
                self.new_old
                    .change_from_key(&newnode2)?
                    .append(oldnode1.clone());
            }
            (None, None) => {
                let mut newnode = Vertex::default();
                BRepBuilder::default().make_vertex(&mut newnode);
                self.old_new
                    .bind(oldnode1.clone(), newnode.clone().into_shape());
                self.old_new
                    .bind(oldnode2.clone(), newnode.clone().into_shape());
                let mut lnodes = List::default();
                lnodes.append(oldnode1.clone());
                lnodes.append(oldnode2.clone());
                self.new_old.add(newnode.into_shape(), lnodes);
            }
        }
        Ok(())
    }
}

/// The static `CreateNewNodes(NodeNearestNode, NodeVertices, aVertexNode,
/// aNodeEdges)`: the pairs of nearest nodes (both in the map) are chained into new
/// nodes, each at the mean point of its vertices with a tolerance covering them; the
/// vertices and the edges move to the new nodes. False when no node was created.
pub(super) fn create_new_nodes(
    node_nearest_node: &SewingShapeIndexedMap,
    node_vertices: &IndexedDataMapOfShapeListOfShape,
    vertex_node: &mut SewingShapeIndexedMap,
    node_edges: &mut SewingShapeListMap,
) -> Result<bool, Failure> {
    let mut chains = NodeChains::default();
    for i in 1..=node_nearest_node.extent() {
        let oldnode1 = node_nearest_node.find_key(i)?;
        let oldnode2 = node_nearest_node.find_from_index(i)?;
        // Second node should also be in the map.
        if !node_nearest_node.contains(oldnode2) {
            continue;
        }
        chains.link(oldnode1, oldnode2)?;
    }
    if chains.new_old.is_empty() {
        return Ok(false);
    }
    for i in 1..=chains.new_old.extent() {
        let newnode = topods::vertex(chains.new_old.find_key(i)?)?;
        let old_nodes = chains.new_old.find_from_index(i)?;
        move_to_new_node(&newnode, old_nodes, node_vertices, vertex_node, node_edges)?;
    }
    Ok(true)
}

/// The vertices and edges of the old nodes moved to the new node, which takes their
/// mean point and a tolerance covering them.
fn move_to_new_node(
    newnode: &Vertex,
    old_nodes: &List<Shape>,
    node_vertices: &IndexedDataMapOfShapeListOfShape,
    vertex_node: &mut SewingShapeIndexedMap,
    node_edges: &mut SewingShapeListMap,
) -> Result<(), Failure> {
    let mut coordinates = Xyz::new(0., 0., 0.);
    let mut lvert: Vec<Vertex> = Vec::new();
    let mut medge = MapOfShape::default();
    let mut ledge: List<Shape> = List::default();
    for oldnode in old_nodes.iter() {
        for v in node_vertices.find_from_key(oldnode)?.iter() {
            let vertex = topods::vertex(v)?;
            *vertex_node.change_from_key(&vertex)? = newnode.clone().into_shape();
            coordinates += BRepTool::pnt(&vertex)?.xyz();
            lvert.push(vertex);
        }
        let edges: Vec<Shape> = node_edges.find(oldnode)?.iter().cloned().collect();
        for edge in edges {
            if medge.add(edge.clone()) {
                ledge.append(edge);
            }
        }
        node_edges.unbind(oldnode);
    }
    node_edges.bind(newnode.clone().into_shape(), ledge);
    let mean = coordinates / lvert.len() as f64;
    let center = Point3::new(mean.x(), mean.y(), mean.z());
    let mut toler: f64 = 0.0;
    for vertex in &lvert {
        let t = center.distance(&BRepTool::pnt(vertex)?) + BRepTool::vertex_tolerance(vertex)?;
        if toler < t {
            toler = t;
        }
    }
    BRepBuilder::default().update_vertex_point(newnode, center, toler)
}
