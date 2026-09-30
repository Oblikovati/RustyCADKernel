// Rust translation of BRepLib_MakeWire::CollectCoincidentVertices and
// BRepLib_BndBoxVertexSelector (BRepLib_MakeWire_1.cxx, BRepLib_MakeWire.hxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The groups of coincident vertices of `BRepLib_MakeWire::Add(L)`: vertices whose
//! tolerance spheres touch, found through a bounding-box tree and chained into groups.
use crate::brep_bnd_lib::BRepBndLib;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::{DataMap, List, Map, TreeSelector, UBTree, UBTreeFiller};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// `NCollection_Map<TopoDS_Vertex>`: vertices compared with `IsEqual`.
type VertexSet = Map<Vertex>;

/// `BRepLib_BndBoxVertexSelector`: the vertices whose tolerance sphere touches the
/// sphere of the current vertex. Its box grows with every current vertex (it is never
/// reset natively).
struct BndBoxVertexSelector<'m> {
    map_of_shape: &'m IndexedMapOfShape,
    p: Point3,
    tol_p: f64,
    v_ind: i32,
    v_box: BoundingBox3,
    result_ind: Vec<i32>,
    failure: Option<Failure>,
}

impl<'m> BndBoxVertexSelector<'m> {
    fn new(map_of_shape: &'m IndexedMapOfShape) -> Self {
        Self {
            map_of_shape,
            p: Point3::default(),
            tol_p: 0.0,
            v_ind: 0,
            v_box: BoundingBox3::default(),
            result_ind: Vec::new(),
            failure: None,
        }
    }

    /// `SetCurrentVertex(theP, theTol, theVInd)`.
    fn set_current_vertex(&mut self, p: Point3, tol: f64, v_ind: i32) {
        self.p = p;
        self.v_box.add_point(p);
        self.v_box.enlarge(tol);
        self.tol_p = tol;
        self.v_ind = v_ind;
    }

    /// The test of `Accept`: the spheres of the current vertex and of `obj` touch.
    fn touches(&self, obj: i32) -> Result<bool, Failure> {
        let vertex = Vertex::unchecked(self.map_of_shape.find_key(obj as usize)?.clone());
        let (point, tol) = (
            BRepTool::pnt(&vertex)?,
            BRepTool::vertex_tolerance(&vertex)?,
        );
        let distance2: f64 = self.p.square_distance(&point);
        let mut sum_tol: f64 = tol + self.tol_p;
        sum_tol *= sum_tol;
        Ok(distance2 <= sum_tol)
    }
}

impl TreeSelector<i32, BoundingBox3> for BndBoxVertexSelector<'_> {
    fn reject(&self, bounds: &BoundingBox3) -> bool {
        bounds.is_out_box(&self.v_box)
    }

    fn accept(&mut self, obj: &i32) -> bool {
        if *obj as usize > self.map_of_shape.extent() || *obj == self.v_ind {
            return false;
        }
        match self.touches(*obj) {
            Ok(true) => {
                self.result_ind.push(*obj);
                true
            }
            Ok(false) => false,
            Err(error) => {
                self.failure.get_or_insert(error);
                false
            }
        }
    }

    fn stop(&self) -> bool {
        false
    }
}

/// `CollectCoincidentVertices(theL, theGrVL)`: the vertices of the wire and of the edges
/// in a box tree; each vertex outside the wire records the vertices touching it (and a
/// wire vertex touching it records it back); the vertices linked this way form groups.
pub(super) fn collect_coincident_vertices(
    wire: &Shape,
    wire_vertices: &IndexedMapOfShape,
    edges: &List<Shape>,
) -> Result<Vec<Vec<Vertex>>, Failure> {
    let mut all_v = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(wire, ShapeEnum::Vertex, &mut all_v)?;
    for edge in edges.iter() {
        topexp::map_shapes_of_type(edge, ShapeEnum::Vertex, &mut all_v)?;
    }
    let tree = vertex_tree(&all_v)?;
    let (v2cv, non_grouped) = coincidences(&all_v, wire_vertices, &tree)?;
    group_vertices(&v2cv, non_grouped)
}

/// The box tree of the vertices, filled in the random order of UBTreeFiller.
fn vertex_tree(all_v: &IndexedMapOfShape) -> Result<UBTree<i32, BoundingBox3>, Failure> {
    let mut tree: UBTree<i32, BoundingBox3> = UBTree::default();
    let mut filler = UBTreeFiller::new(&mut tree, None, true);
    for index in 1..=all_v.extent() {
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(all_v.find_key(index)?, &mut bounds, true)?;
        filler.add(index as i32, bounds);
    }
    filler.fill();
    drop(filler);
    Ok(tree)
}

/// `aV2CV` (vertex to its coincident vertices) and the vertices still to group.
fn coincidences(
    all_v: &IndexedMapOfShape,
    wire_vertices: &IndexedMapOfShape,
    tree: &UBTree<i32, BoundingBox3>,
) -> Result<(DataMap<Vertex, VertexSet>, VertexSet), Failure> {
    let mut v2cv: DataMap<Vertex, VertexSet> = DataMap::default();
    let mut non_grouped = VertexSet::default();
    let mut selector = BndBoxVertexSelector::new(all_v);
    for index in 1..=all_v.extent() {
        let vertex = Vertex::unchecked(all_v.find_key(index)?.clone());
        if wire_vertices.contains(&vertex) {
            continue;
        }
        selector.set_current_vertex(
            BRepTool::pnt(&vertex)?,
            BRepTool::vertex_tolerance(&vertex)?,
            index as i32,
        );
        let nb_coll: i32 = tree.select(&mut selector);
        if let Some(error) = selector.failure.take() {
            return Err(error);
        }
        if nb_coll > 0 {
            record(
                &vertex,
                &selector.result_ind,
                all_v,
                wire_vertices,
                &mut v2cv,
                &mut non_grouped,
            )?;
        }
        selector.result_ind.clear();
    }
    Ok((v2cv, non_grouped))
}

/// The coincidences of one vertex: its set, and the vertex in the set of each wire
/// vertex it touches.
fn record(
    vertex: &Vertex,
    result_ind: &[i32],
    all_v: &IndexedMapOfShape,
    wire_vertices: &IndexedMapOfShape,
    v2cv: &mut DataMap<Vertex, VertexSet>,
    non_grouped: &mut VertexSet,
) -> Result<(), Failure> {
    v2cv.bound(vertex.clone(), VertexSet::default());
    for &index in result_ind {
        let cv = Vertex::unchecked(all_v.find_key(index as usize)?.clone());
        v2cv.change_find(vertex)?.add(cv.clone());
        if !wire_vertices.contains(&cv) {
            continue;
        }
        if v2cv.is_bound(&cv) {
            v2cv.change_find(&cv)?.add(vertex.clone());
        } else {
            v2cv.bound(cv.clone(), VertexSet::default())
                .add(vertex.clone());
            non_grouped.add(cv);
        }
    }
    if !v2cv.find(vertex)?.is_empty() {
        non_grouped.add(vertex.clone()); // vertexes to be grouped; store only coincident vertices
    }
    Ok(())
}

/// "group the coincident vertices": starting from the first ungrouped vertex (map
/// order), the vertices reachable through the coincidence sets form one group.
fn group_vertices(
    v2cv: &DataMap<Vertex, VertexSet>,
    mut non_grouped: VertexSet,
) -> Result<Vec<Vec<Vertex>>, Failure> {
    let mut groups: Vec<Vec<Vertex>> = Vec::new();
    let mut start: Vec<Vertex> = Vec::new();
    let mut one_group: Vec<Vertex> = Vec::new();
    let mut is_start_new_group: bool = true;
    // exit only if there are no nongrouped vertices and the last group is fully constructed
    while !non_grouped.is_empty() || !is_start_new_group {
        if is_start_new_group {
            let current: Vertex = non_grouped.iter().next().expect("a non-empty map").clone();
            non_grouped.remove(&current);
            start.push(current);
        }
        let mut current: Vec<Vertex> = Vec::new();
        for sv in &start {
            one_group.push(sv.clone());
            for cv in v2cv.find(sv)?.iter() {
                if non_grouped.contains(cv) {
                    current.push(cv.clone());
                    non_grouped.remove(cv);
                }
            }
        }
        start = current;
        is_start_new_group = start.is_empty();
        if is_start_new_group && !one_group.is_empty() {
            groups.push(std::mem::take(&mut one_group));
        }
    }
    Ok(groups)
}
