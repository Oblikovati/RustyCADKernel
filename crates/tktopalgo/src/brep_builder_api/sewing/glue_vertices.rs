// Rust translation of the static GlueVertices and of
// BRepBuilderAPI_Sewing::VerticesAssembling (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gluing of the nodes of the free boundaries: each node is linked to its nearest
//! node within the tolerance (unless the two may not be merged), and the chains of
//! linked nodes become new nodes, until no node is linked.
use super::merge_nodes::{create_new_nodes, is_merged_vertices};
use super::{BRepBuilderAPISewing, SewingShapeIndexedMap, SewingShapeListMap};
use crate::brep_builder_api::BRepBuilderAPIVertexInspector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{CellFilter, List, Sequence};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// The maps GlueVertices works on: vertex -> node, node -> boundaries, boundary ->
/// faces.
struct GlueMaps<'m> {
    vertex_node: &'m mut SewingShapeIndexedMap,
    node_edges: &'m mut SewingShapeListMap,
    bound_faces: &'m IndexedDataMapOfShapeListOfShape,
}

/// The vertex of `edge` whose node is `node`, or the node itself (the last end wins).
fn vertex_of_node(
    edge: &Shape,
    node: &Vertex,
    vertex_node: &SewingShapeIndexedMap,
) -> Result<Shape, Failure> {
    let mut v = node.clone().into_shape();
    let (ov1, ov2) = topexp::vertices(&topods::edge(edge)?, false);
    for ov in [ov1, ov2] {
        if let Some(n) = vertex_node.seek(&ov)
            && node.is_same(n)
        {
            v = ov.into_shape();
        }
    }
    Ok(v)
}

/// The non-null faces of a boundary; Standard_NoSuchObject for an edge that is no
/// boundary.
fn faces_of(
    bound: &Shape,
    bound_faces: &IndexedDataMapOfShapeListOfShape,
) -> Result<MapOfShape, Failure> {
    let mut faces = MapOfShape::default();
    for f in bound_faces.find_from_key(bound)?.iter() {
        if !f.is_null() {
            faces.add(f.clone());
        }
    }
    Ok(faces)
}

/// The merging condition of two nodes over their boundaries: the first nonzero status
/// of `IsMergedVertices` on a face they share (or on a common floating edge), and
/// whether they bound a short closed chain (status 1).
fn merge_status(
    node1: &Vertex,
    node2: &Vertex,
    maps: &GlueMaps<'_>,
) -> Result<(i32, bool), Failure> {
    let ledges1: Vec<Shape> = maps.node_edges.find(node1)?.iter().cloned().collect();
    let ledges2: Vec<Shape> = maps.node_edges.find(node2)?.iter().cloned().collect();
    let (mut status, mut is_same_edge) = (0, false);
    for e1 in &ledges1 {
        if status != 0 || is_same_edge {
            break;
        }
        let v1 = vertex_of_node(e1, node1, maps.vertex_node)?;
        let faces1 = faces_of(e1, maps.bound_faces)?;
        for e2 in &ledges2 {
            if status != 0 || is_same_edge {
                break;
            }
            let v2 = vertex_of_node(e2, node2, maps.vertex_node)?;
            let lfac2 = maps.bound_faces.find_from_key(e2)?;
            let stat = if !lfac2.is_empty() {
                let mut stat = None;
                for f in lfac2.iter() {
                    if stat.is_some_and(|s| s != 0) {
                        break;
                    }
                    if faces1.contains(f) {
                        stat = Some(is_merged_vertices(f, e1, e2, &v1, &v2)?);
                        if stat == Some(1) {
                            break;
                        }
                    }
                }
                stat
            } else if faces1.is_empty() && e1 == e2 {
                Some(is_merged_vertices(&Shape::default(), e1, e1, &v1, &v2)?)
            } else {
                None
            };
            match stat {
                Some(1) => is_same_edge = true,
                Some(s) => status = s,
                None => {}
            }
            if lfac2.is_empty() && faces1.is_empty() && e1 == e2 {
                break;
            }
        }
    }
    Ok((status, is_same_edge))
}

/// The nearest node of `node1` among the found ones: the candidates within the
/// tolerance by increasing distance, those nearer to a node of a short closed chain
/// than to `node1` removed.
fn nearest_node(
    node1: &Vertex,
    found: &[i32],
    node_vertices: &IndexedDataMapOfShapeListOfShape,
    maps: &GlueMaps<'_>,
    tolerance: f64,
) -> Result<Option<Shape>, Failure> {
    let pt1 = BRepTool::pnt(node1)?;
    let mut seq_nodes: Sequence<Shape> = Sequence::default();
    let mut same_edge_nodes: Vec<Vertex> = Vec::new();
    for &index in found {
        let node2 = topods::vertex(node_vertices.find_key(index as usize)?)?;
        if *node1 == node2 {
            continue;
        }
        let (status, is_same_edge) = merge_status(node1, &node2, maps)?;
        if status != 0 {
            continue;
        }
        if is_same_edge {
            same_edge_nodes.push(node2.clone());
        }
        let dist = pt1.distance(&BRepTool::pnt(&node2)?);
        if dist < tolerance {
            insert_by_distance(&mut seq_nodes, node2, (pt1, dist))?;
        }
    }
    if seq_nodes.is_empty() {
        return Ok(None);
    }
    for n2 in &same_edge_nodes {
        let p2 = BRepTool::pnt(n2)?;
        let mut k = 1;
        while k <= seq_nodes.len() {
            let n1 = topods::vertex(seq_nodes.value(k)?)?;
            if n1 != *n2 {
                let p1: Point3 = BRepTool::pnt(&n1)?;
                if p2.distance(&p1) >= pt1.distance(&p1) {
                    k += 1;
                    continue;
                }
            }
            seq_nodes.remove(k)?;
        }
    }
    Ok(seq_nodes.first().cloned())
}

/// Inserts a node before the first one farther from `pt1`.
fn insert_by_distance(
    seq_nodes: &mut Sequence<Shape>,
    node2: Vertex,
    (pt1, dist): (Point3, f64),
) -> Result<(), Failure> {
    for kk in 1..=seq_nodes.len() {
        let pt = BRepTool::pnt(&topods::vertex(seq_nodes.value(kk)?)?)?;
        if dist < pt1.distance(&pt) {
            seq_nodes.insert_before(kk, node2.into_shape())?;
            return Ok(());
        }
    }
    seq_nodes.append(node2.into_shape());
    Ok(())
}

/// The nodes and their vertices (a node is filed in the cell filter and the inspector
/// when first met, with its index).
struct NodeCells {
    node_vertices: IndexedDataMapOfShapeListOfShape,
    filter: CellFilter<BRepBuilderAPIVertexInspector>,
    inspector: BRepBuilderAPIVertexInspector,
}

/// The `map node -> vertices` of GlueVertices, with the cells of the nodes.
fn node_cells(vertex_node: &SewingShapeIndexedMap, tolerance: f64) -> Result<NodeCells, Failure> {
    let mut cells = NodeCells {
        node_vertices: IndexedDataMapOfShapeListOfShape::default(),
        filter: CellFilter::new(tolerance),
        inspector: BRepBuilderAPIVertexInspector::new(tolerance),
    };
    for i in 1..=vertex_node.extent() {
        let vertex = vertex_node.find_key(i)?.clone();
        let node = topods::vertex(vertex_node.find_from_index(i)?)?;
        if let Some(list) = cells.node_vertices.change_seek(&node) {
            list.append(vertex);
            continue;
        }
        let mut vlist = List::default();
        vlist.append(vertex);
        let index = cells.node_vertices.add(node.clone().into_shape(), vlist) as i32;
        let point = BRepTool::pnt(&node)?.xyz();
        cells.filter.add(&index, &point);
        cells.inspector.add(point);
    }
    Ok(cells)
}

/// The indices of the nodes within the tolerance box of `node1` (itself included).
fn near_nodes(cells: &mut NodeCells, node1: &Vertex, tolerance: f64) -> Result<Vec<i32>, Failure> {
    let pt1 = BRepTool::pnt(node1)?.xyz();
    cells.inspector.set_current(pt1);
    let pmin = BRepBuilderAPIVertexInspector::shift(&pt1, -tolerance);
    let pmax = BRepBuilderAPIVertexInspector::shift(&pt1, tolerance);
    cells
        .filter
        .inspect_range(&pmin, &pmax, &mut cells.inspector);
    Ok(cells.inspector.res_ind().iter().copied().collect())
}

/// The static `GlueVertices(aVertexNode, aNodeEdges, aBoundFaces, Tolerance,
/// theProgress)`: the nodes are filed in a cell filter; each node is linked to its
/// nearest node, then the chains are merged (`CreateNewNodes`). False when nothing is
/// linked or merged.
fn glue_vertices(
    maps: &mut GlueMaps<'_>,
    tolerance: f64,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let mut cells = node_cells(maps.vertex_node, tolerance)?;
    let nb_nodes = cells.node_vertices.extent();
    let mut node_nearest_node = SewingShapeIndexedMap::default();
    let scope = ProgressScope::with_literal(progress, "Glueing nodes", nb_nodes as f64, true);
    let mut i = 1;
    while i <= nb_nodes && scope.more() {
        let node1 = topods::vertex(cells.node_vertices.find_key(i)?)?;
        let found = near_nodes(&mut cells, &node1, tolerance)?;
        if !found.is_empty() {
            let nearest = nearest_node(&node1, &found, &cells.node_vertices, maps, tolerance)?;
            if let Some(nearest) = nearest {
                node_nearest_node.add(node1.into_shape(), nearest);
            }
            cells.inspector.clear_res_list();
        }
        i += 1;
        scope.next(1.0);
    }
    if node_nearest_node.is_empty() {
        return Ok(false);
    }
    create_new_nodes(
        &node_nearest_node,
        &cells.node_vertices,
        maps.vertex_node,
        maps.node_edges,
    )
}

impl BRepBuilderAPISewing {
    /// `VerticesAssembling(theProgress)`: the boundaries of each node are collected
    /// (`myNodeSections`), then the nodes of the boundaries of faces, and those of the
    /// floating edges, are glued until stable.
    pub(crate) fn vertices_assembling(
        &mut self,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let nb_vert = self.my_vertex_node.extent();
        let nb_vert_free = self.my_vertex_node_free.extent();
        let scope = ProgressScope::with_literal(progress, "Vertices assembling", 2.0, false);
        if nb_vert == 0 && nb_vert_free == 0 {
            return Ok(());
        }
        self.collect_node_sections()?;
        let tolerance = self.my_tolerance;
        if nb_vert != 0 {
            let mut maps = GlueMaps {
                vertex_node: &mut self.my_vertex_node,
                node_edges: &mut self.my_node_sections,
                bound_faces: &self.my_bound_faces,
            };
            while glue_vertices(&mut maps, tolerance, &scope.next(1.0))? {}
        }
        if !scope.more() {
            return Ok(());
        }
        if nb_vert_free != 0 {
            let mut maps = GlueMaps {
                vertex_node: &mut self.my_vertex_node_free,
                node_edges: &mut self.my_node_sections,
                bound_faces: &self.my_bound_faces,
            };
            while glue_vertices(&mut maps, tolerance, &scope.next(1.0))? {}
        }
        Ok(())
    }

    /// Each vertex of each boundary is bound to the boundaries it ends.
    fn collect_node_sections(&mut self) -> Result<(), Failure> {
        for i in 1..=self.my_bound_faces.extent() {
            let bound = self.my_bound_faces.find_key(i)?.clone();
            let mut it = ShapeIterator::new(&bound, false, true);
            while let Ok(node) = it.value() {
                match self.my_node_sections.change_find(node) {
                    Ok(list) => {
                        list.append(bound.clone());
                    }
                    Err(_) => {
                        let mut list = List::default();
                        list.append(bound.clone());
                        self.my_node_sections.bind(node.clone(), list);
                    }
                }
                it.next();
            }
        }
        Ok(())
    }
}
