// Rust translation of BRepMesh_MeshTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMesh_MeshTool`: edits of a BRepMesh_DataStructureOfDelaun (triangle insertion
//! with legalization, cleanup of frontier and free links).
use super::{
    BRepMeshCircleTool, BRepMeshDataStructureOfDelaun, BRepMeshEdge,
    BRepMeshSelectorOfDataStructureOfDelaun, BRepMeshTriangle, DegreeOfFreedom,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap, DefaultHashPolicy, PackedMap};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Direction2, Vector2, Xy};

#[path = "mesh_tool/cleanup.rs"]
mod cleanup;

/// `IMeshData::MapOfIntegerInteger`: link index to its orientation in the erased
/// triangle (an NCollection_DataMap on an incremental allocator).
pub type MapOfIntegerInteger = DataMap<i32, bool>;

/// A new `MapOfIntegerInteger(1, new NCollection_IncAllocator)`.
pub(crate) fn new_loop_edges() -> MapOfIntegerInteger {
    DataMap::with_allocator(DefaultHashPolicy, 1, BaseAllocator::incremental(0))
}

/// `BRepMesh_MeshTool::NodeClassifier`: whether a node lies strictly on the upper side
/// of a constraint link (orientation normalized so that the link points to +X).
pub struct NodeClassifier {
    structure: Handle<BRepMeshDataStructureOfDelaun>,
    location: Xy,
    direction: Direction2,
    sign: bool,
}

impl NodeClassifier {
    /// `NodeClassifier(theConstraint, theStructure)`: the line through the link nodes;
    /// gp_Dir2d's Standard_ConstructionError for a link of coincident nodes;
    /// `NodeClassifier::new(&link, &structure)?`.
    pub fn new(
        constraint: &BRepMeshEdge,
        structure: &Handle<BRepMeshDataStructureOfDelaun>,
    ) -> Result<Self, Failure> {
        let first = *structure.get_node(constraint.first_node())?.coord();
        let last = *structure.get_node(constraint.last_node())?.coord();
        let direction = Direction2::from_xy(last - first)?;
        Ok(Self {
            structure: structure.clone(),
            location: first,
            sign: direction.x() > 0.0,
            direction,
        })
    }

    /// `IsAbove(theNodeIndex)`: false for a node on the line (cross product or
    /// distance not above gp::Resolution()); `classifier.is_above(3)?`.
    pub fn is_above(&self, node_index: i32) -> Result<bool, Failure> {
        let vertex = *self.structure.get_node(node_index)?.coord();
        let node_vec = Vector2::from(vertex - self.location);
        if node_vec.square_magnitude() <= rustycad_tkmath::gp::resolution() {
            return Ok(false);
        }
        let cross = node_vec.crossed(Vector2::from(self.direction.xy()));
        if cross.abs() <= rustycad_tkmath::gp::resolution() {
            return Ok(false);
        }
        Ok(if self.sign { cross < 0.0 } else { cross > 0.0 })
    }
}

/// `BRepMesh_MeshTool` over a shared mesh structure.
///
/// ```
/// use rustycad_tkernel::ncollection::IncrementalAllocatorObject;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::Xy;
/// use rustycad_tkmesh::brep_mesh::{BRepMeshDataStructureOfDelaun, BRepMeshMeshTool, BRepMeshVertex, DegreeOfFreedom};
/// let allocator = Handle::new(IncrementalAllocatorObject::new(0));
/// let mesh = Handle::new(BRepMeshDataStructureOfDelaun::new(&allocator, 3));
/// for (x, y) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)] {
///     mesh.add_node(&BRepMeshVertex::new(Xy::new(x, y), 0, DegreeOfFreedom::Free), false)?;
/// }
/// let tool = BRepMeshMeshTool::new(&mesh);
/// tool.add_triangle(1, 2, 3)?;
/// assert_eq!(mesh.elements_of_domain().extent(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepMeshMeshTool {
    transient: StandardTransient,
    structure: Handle<BRepMeshDataStructureOfDelaun>,
}
impl_transient!(
    BRepMeshMeshTool,
    "BRepMesh_MeshTool",
    StandardTransient,
    transient
);

impl BRepMeshMeshTool {
    /// `BRepMesh_MeshTool(theStructure)`; `BRepMeshMeshTool::new(&mesh)`.
    pub fn new(structure: &Handle<BRepMeshDataStructureOfDelaun>) -> Self {
        Self {
            transient: StandardTransient::default(),
            structure: structure.clone(),
        }
    }

    /// `GetStructure()`; `tool.get_structure()`.
    pub fn get_structure(&self) -> &Handle<BRepMeshDataStructureOfDelaun> {
        &self.structure
    }

    /// `AddAndLegalizeTriangle(p1, p2, p3)`: adds the triangle, then legalizes its
    /// three links in order; `tool.add_and_legalize_triangle(1, 2, 3)?`.
    pub fn add_and_legalize_triangle(&self, p1: i32, p2: i32, p3: i32) -> Result<(), Failure> {
        let edges = self.add_triangle(p1, p2, p3)?;
        for edge in edges {
            self.legalize(edge)?;
        }
        Ok(())
    }

    /// `AddTriangle(p1, p2, p3, theEdges)`: the three free links and a free triangle;
    /// returns the link indices; `tool.add_triangle(1, 2, 3)?`.
    pub fn add_triangle(&self, p1: i32, p2: i32, p3: i32) -> Result<[i32; 3], Failure> {
        let (e0, o0) = self.add_link(p1, p2)?;
        let (e1, o1) = self.add_link(p2, p3)?;
        let (e2, o2) = self.add_link(p3, p1)?;
        let triangle = BRepMeshTriangle::new([e0, e1, e2], [o0, o1, o2], DegreeOfFreedom::Free);
        self.structure.add_element(&triangle)?;
        Ok([e0, e1, e2])
    }

    /// `AddLink(theFirstNode, theLastNode, theLinkIndex, theLinkOri)`: the link index
    /// and whether it is stored in this direction; `tool.add_link(1, 2)?`.
    pub fn add_link(&self, first: i32, last: i32) -> Result<(i32, bool), Failure> {
        let link =
            self.structure
                .add_link(&BRepMeshEdge::new(first, last, DegreeOfFreedom::Free))?;
        Ok((link.abs(), link > 0))
    }

    /// `Legalize(theLinkIndex)`: flips the non-frontier links whose two triangles
    /// violate the circumcircle criterion, depth first from the given link;
    /// `tool.legalize(1)?`.
    pub fn legalize(&self, link_index: i32) -> Result<(), Failure> {
        let mut stack = vec![link_index];
        let mut used_links = PackedMap::<i32>::default();
        while let Some(link) = stack.pop() {
            used_links.add(link);
            self.legalize_link(link, &used_links, &mut stack)?;
        }
        Ok(())
    }

    /// One step of Legalize: the flip of `link` when it is needed.
    fn legalize_link(
        &self,
        link_index: i32,
        used_links: &PackedMap<i32>,
        stack: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let link = self.structure.get_link(link_index)?;
        let pair = self.structure.elements_connected_to(link_index)?;
        if link.movability() == DegreeOfFreedom::Frontier || pair.extent() != 2 {
            return Ok(());
        }
        let nodes = [
            self.element_nodes(pair.first_index())?,
            self.element_nodes(pair.last_index())?,
        ];
        let apex = [
            find_apex_index(&nodes[0], &link)?,
            find_apex_index(&nodes[1], &link)?,
        ];
        let flip = self.check_circle(&nodes[0], nodes[1][apex[1]])?
            || self.check_circle(&nodes[1], nodes[0][apex[0]])?;
        if !flip {
            return Ok(());
        }
        self.structure.remove_element(pair.first_index())?;
        self.structure.remove_element(pair.last_index())?;
        self.structure.remove_link(link_index, false)?;
        let (a, b) = (nodes[0], nodes[1]);
        let (i, j) = (apex[0], apex[1]);
        self.add_triangle_and_update_stack([a[i], a[(i + 1) % 3], b[j]], used_links, stack)?;
        self.add_triangle_and_update_stack([b[j], b[(j + 1) % 3], a[i]], used_links, stack)
    }

    /// The nodes of an element; Standard_OutOfRange natively for a bad index.
    fn element_nodes(&self, element: i32) -> Result<[i32; 3], Failure> {
        let triangle = self.structure.get_element(element)?;
        self.structure.element_nodes(&triangle)
    }

    /// `checkCircle`: whether `point` lies in the circumcircle of the nodes (squared
    /// distance less the squared radius below Precision::SquareConfusion()).
    fn check_circle(&self, nodes: &[i32; 3], point: i32) -> Result<bool, Failure> {
        let coords = [
            *self.structure.get_node(nodes[0])?.coord(),
            *self.structure.get_node(nodes[1])?.coord(),
            *self.structure.get_node(nodes[2])?.coord(),
        ];
        let (mut location, mut radius) = (Xy::default(), 0.0);
        if !BRepMeshCircleTool::make_circle(&coords, &mut location, &mut radius) {
            return Ok(false);
        }
        let vertex = *self.structure.get_node(point)?.coord();
        let dist = (vertex - location).square_modulus() - (radius * radius);
        Ok(dist < precision::square_confusion())
    }

    /// `addTriangleAndUpdateStack`: the new links not yet used go on the stack.
    fn add_triangle_and_update_stack(
        &self,
        nodes: [i32; 3],
        used_links: &PackedMap<i32>,
        stack: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let edges = self.add_triangle(nodes[0], nodes[1], nodes[2])?;
        stack.extend(edges.into_iter().filter(|edge| !used_links.contains(*edge)));
        Ok(())
    }

    /// `EraseItemsConnectedTo(theNodeIndex)`: the triangles around the node, the links
    /// they free, then the node itself; `tool.erase_items_connected_to(4)?`.
    pub fn erase_items_connected_to(&self, node_index: i32) -> Result<(), Failure> {
        let selector = BRepMeshSelectorOfDataStructureOfDelaun::new(&self.structure);
        selector.neighbours_of_node(node_index)?;
        let mut loop_edges = new_loop_edges();
        self.erase_triangles(&selector.elements(), &mut loop_edges)?;
        self.erase_free_links_of(&loop_edges)?;
        self.structure.remove_node(node_index, false)
    }

    /// `GetEdgesByType(theEdgeType)`: the domain links of that movability, in the
    /// packed-map order of the domain; `tool.get_edges_by_type(DegreeOfFreedom::Frontier)?`.
    pub fn get_edges_by_type(&self, edge_type: DegreeOfFreedom) -> Result<PackedMap<i32>, Failure> {
        let mut result = PackedMap::<i32>::default();
        for index in self.structure.links_of_domain().iter() {
            if self.structure.get_link(index)?.movability() == edge_type {
                result.add(index);
            }
        }
        Ok(result)
    }
}

/// `findApexIndex`: the position of the triangle node that is not on the link. When
/// every node is on it (a degenerate triangle) the native index 3 then reads past the
/// node array; Rust reports that as Standard_OutOfRange.
fn find_apex_index(nodes: &[i32; 3], link: &BRepMeshEdge) -> Result<usize, Failure> {
    nodes
        .iter()
        .position(|node| *node != link.first_node() && *node != link.last_node())
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "BRepMesh_MeshTool::Legalize: triangle nodes {nodes:?} all lie on link ({}, {}); expected an apex node",
                    link.first_node(),
                    link.last_node()
                ),
            )
        })
}
