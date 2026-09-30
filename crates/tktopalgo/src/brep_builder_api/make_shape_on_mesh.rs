// Rust translation of BRepBuilderAPI_MakeShapeOnMesh.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeShapeOnMesh`: a compound of planar triangular faces that share
//! their vertices and edges, built from a `Poly_Triangulation`.
use super::make_edge::BRepBuilderAPIMakeEdge;
use super::make_face::BRepBuilderAPIMakeFace;
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::make_vertex::BRepBuilderAPIMakeVertex;
use super::make_wire::BRepBuilderAPIMakeWire;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Compound, Edge, Vertex};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Direction3, Plane3, Xyz};
use rustycad_tkmath::poly::Triangulation;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/// `BRepBuilderAPI_MakeShapeOnMesh` (docs/topalgo-design.md sections 11 and 14): the
/// mesh to rebuild; `Build` makes one vertex per node, one edge per mesh edge (shared by
/// its triangles) and one planar face per triangle, in a compound. It dereferences to
/// the MakeShape base, as the other makers of the package.
///
/// ```
/// use rustycad_tkernel::message::ProgressRange;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::poly::Triangulation;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeOnMesh};
/// let mut maker = BRepBuilderAPIMakeShapeOnMesh::new(&Handle::<Triangulation>::default());
/// maker.build(&ProgressRange::default())?;
/// assert!(!maker.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeShapeOnMesh {
    base: BRepBuilderAPIMakeShapeBase,
    my_mesh: Handle<Triangulation>,
}

impl Deref for BRepBuilderAPIMakeShapeOnMesh {
    type Target = BRepBuilderAPIMakeShapeBase;
    fn deref(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }
}

impl DerefMut for BRepBuilderAPIMakeShapeOnMesh {
    fn deref_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }
}

impl BRepBuilderAPIMakeShapeOnMesh {
    /// `BRepBuilderAPI_MakeShapeOnMesh(theMesh)`: stores the mesh; nothing is built yet;
    /// `BRepBuilderAPIMakeShapeOnMesh::new(&mesh)`.
    pub fn new(mesh: &Handle<Triangulation>) -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_mesh: mesh.clone(),
        }
    }
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeShapeOnMesh {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build(theRange)`: nothing for a null mesh or one without nodes or triangles;
    /// else the shared vertices, the shared edges of the valid triangles and the faces,
    /// one progress step per node and two per triangle (a user break returns before
    /// the maker is done). The makers' exceptions (an edge between two nodes closer
    /// than `Precision::Confusion()`) are returned; `maker.build(&range)?`.
    fn build(&mut self, range: &ProgressRange<'_>) -> Result<(), Failure> {
        let Some(mesh) = self
            .my_mesh
            .get()
            .filter(|m| m.nb_nodes() != 0 && m.nb_triangles() != 0)
        else {
            return Ok(());
        };
        let total: f64 = f64::from(mesh.nb_nodes() + 2 * mesh.nb_triangles());
        let scope =
            ProgressScope::with_literal(range, "Per-facet shape construction", total, false);
        let Some(vertices) = node_vertices(mesh, &scope)? else {
            return Ok(());
        };
        let Some(edges) = mesh_edges(mesh, &vertices, &scope)? else {
            return Ok(());
        };
        let Some(compound) = triangle_faces(mesh, &edges, &scope)? else {
            return Ok(());
        };
        self.base.set_done();
        self.base.my_shape = compound.into();
        Ok(())
    }
}

/// `Edge` of the native anonymous namespace: the node indices of a mesh edge, the
/// smaller first.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MeshEdge {
    idx1: i32,
    idx2: i32,
}

impl MeshEdge {
    fn new(first: i32, second: i32) -> Self {
        Self {
            idx1: first.min(second),
            idx2: first.max(second),
        }
    }
}

/// The shared edges by mesh edge. The native `NCollection_IndexedDataMap` keeps the
/// first edge added for a key and is only searched by key, so a hash map with the same
/// "first wins" insertion gives the same edges (its order is never read).
type MeshEdgeMap = HashMap<MeshEdge, Edge>;

/// One `aPS.Next()`, then whether the user asked to stop.
fn step_breaks(scope: &ProgressScope<'_>) -> bool {
    drop(scope.next(1.0));
    scope.user_break()
}

/// The vertex of each node (`aPnt2VertexMap`, node `i` at index `i - 1`); `None` on a
/// user break.
fn node_vertices(
    mesh: &Triangulation,
    scope: &ProgressScope<'_>,
) -> Result<Option<Vec<Vertex>>, Failure> {
    let mut vertices = Vec::with_capacity(mesh.nb_nodes().max(0) as usize);
    for i in 1..=mesh.nb_nodes() {
        if step_breaks(scope) {
            return Ok(None);
        }
        vertices.push(BRepBuilderAPIMakeVertex::new(mesh.node(i)?)?.vertex()?);
    }
    Ok(Some(vertices))
}

/// `aPnt2VertexMap.FindFromKey(i)`: Standard_NoSuchObject for a node index outside the
/// map (the node reads before it raise first natively).
fn vertex_of(vertices: &[Vertex], index: i32) -> Result<&Vertex, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|i| vertices.get(i))
        .ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "NCollection_IndexedDataMap::FindFromKey: node {index}; expected 1..={}",
                    vertices.len()
                ),
                "",
            )
        })
}

/// The shared edges: for each triangle that is neither topologically nor geometrically
/// degenerated (a squared side below `gp::Resolution()`), the three edges V1V2, V2V3
/// and V3V1, each reversed when its mesh edge runs from the larger index, added when
/// their mesh edge is new; `None` on a user break.
fn mesh_edges(
    mesh: &Triangulation,
    vertices: &[Vertex],
    scope: &ProgressScope<'_>,
) -> Result<Option<MeshEdgeMap>, Failure> {
    let mut edges = MeshEdgeMap::new();
    for i in 1..=mesh.nb_triangles() {
        if step_breaks(scope) {
            return Ok(None);
        }
        let idx = triangle_indices(mesh, i)?;
        if idx[0] == idx[1] || idx[0] == idx[2] || idx[1] == idx[2] || is_flat(mesh, idx)? {
            continue;
        }
        let (v1, v2, v3) = (
            vertex_of(vertices, idx[0])?,
            vertex_of(vertices, idx[1])?,
            vertex_of(vertices, idx[2])?,
        );
        let made = [
            oriented_edge(v1, v2, idx[1] < idx[0])?,
            oriented_edge(v2, v3, idx[2] < idx[1])?,
            oriented_edge(v3, v1, idx[0] < idx[2])?,
        ];
        for (key, edge) in side_keys(idx).into_iter().zip(made) {
            edges.entry(key).or_insert(edge);
        }
    }
    Ok(Some(edges))
}

/// `aTriangle.Get(...)`.
fn triangle_indices(mesh: &Triangulation, index: i32) -> Result<[i32; 3], Failure> {
    let (a, b, c) = mesh.triangle(index)?.get();
    Ok([a, b, c])
}

/// Whether a squared side of the triangle is below `gp::Resolution()` (the three node
/// reads come first, as natively).
fn is_flat(mesh: &Triangulation, idx: [i32; 3]) -> Result<bool, Failure> {
    let (p1, p2, p3) = (mesh.node(idx[0])?, mesh.node(idx[1])?, mesh.node(idx[2])?);
    let (d1, d2, d3) = (
        p1.square_distance(&p2),
        p1.square_distance(&p3),
        p2.square_distance(&p3),
    );
    Ok(d1 < gp::resolution() || d2 < gp::resolution() || d3 < gp::resolution())
}

/// The mesh edges of the three sides, in the order (0, 1), (1, 2), (2, 0).
fn side_keys(idx: [i32; 3]) -> [MeshEdge; 3] {
    [
        MeshEdge::new(idx[0], idx[1]),
        MeshEdge::new(idx[1], idx[2]),
        MeshEdge::new(idx[2], idx[0]),
    ]
}

/// `BRepBuilderAPI_MakeEdge(Va, Vb).Edge()`, reversed on request (StdFail_NotDone when
/// the maker fails).
fn oriented_edge(first: &Vertex, second: &Vertex, reverse: bool) -> Result<Edge, Failure> {
    let mut edge = BRepBuilderAPIMakeEdge::from_vertices(first, second)?.edge()?;
    if reverse {
        edge.reverse();
    }
    Ok(edge)
}

/// The compound of one planar face per triangle whose three mesh edges exist and whose
/// first two sides are not parallel; `None` on a user break.
fn triangle_faces(
    mesh: &Triangulation,
    edges: &MeshEdgeMap,
    scope: &ProgressScope<'_>,
) -> Result<Option<Compound>, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for i in 1..=mesh.nb_triangles() {
        if step_breaks(scope) {
            return Ok(None);
        }
        let idx = triangle_indices(mesh, i)?;
        let reversed = [idx[1] < idx[0], idx[2] < idx[1], idx[0] < idx[2]];
        let keys = side_keys(idx);
        let [Some(e1), Some(e2), Some(e3)] = keys.map(|key| edges.get(&key)) else {
            continue;
        };
        let mut sides: [Edge; 3] = [e1.clone(), e2.clone(), e3.clone()];
        for (edge, flip) in sides.iter_mut().zip(reversed) {
            if flip {
                edge.reverse();
            }
        }
        if let Some(face) = triangle_face(mesh, idx[0], &sides)? {
            builder.add(&mut compound, &face)?;
        }
    }
    Ok(Some(compound))
}

/// The face of one triangle: the wire of its three edges on the plane through its first
/// node, normal to the cross product of the first two edge lines (reversed once per
/// REVERSED edge among the first two); `None` when that product is below
/// `Precision::SquareConfusion()`.
fn triangle_face(
    mesh: &Triangulation,
    first_node: i32,
    sides: &[Edge],
) -> Result<Option<rustycad_tkbrep::topods::Face>, Failure> {
    let mut wire_maker = BRepBuilderAPIMakeWire::new();
    for edge in sides {
        wire_maker.add_edge(edge)?;
    }
    let wire = wire_maker.wire()?;
    let d1: Direction3 = BRepAdaptorCurve::with_edge(&sides[0])?.line()?.direction();
    let d2: Direction3 = BRepAdaptorCurve::with_edge(&sides[1])?.line()?.direction();
    let mut normal: Xyz = d1.xyz().crossed(d2.xyz());
    if normal.square_modulus() < precision::square_confusion() {
        return Ok(None);
    }
    for edge in &sides[..2] {
        if edge.orientation() == Orientation::Reversed {
            normal.reverse();
        }
    }
    let plane = Plane3::new(mesh.node(first_node)?, Direction3::from_xyz(normal)?)?;
    let face = BRepBuilderAPIMakeFace::from_plane_wire(plane, &wire, true)?.face()?;
    Ok(Some(face))
}
