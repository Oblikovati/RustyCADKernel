//! `BOPAlgo_Tools::EdgesToWires`: the planar wires made of a set of edges.
use super::planes::{EdgeTangents, curve_plane, curve_tangent, wire_plane};
use crate::bop_algo::BOPAlgoBuilder;
use crate::bop_tools::make_connexity_blocks;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape, Wire};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Direction3, Plane3};

/// The plane of each edge that has one (`BOPAlgo_IndexedDataMapOfShapePln`).
type EdgePlanes = IndexedDataMap<Shape, Plane3, ShapeMapHasher>;

/// The planes and tangents of the edges, with the edges without a plane of their own.
struct EdgeGeometry {
    planes: EdgePlanes,
    tangents: EdgeTangents,
    no_unique_plane: MapOfShape,
    /// The edges without a plane, as one compound (`aLEdges`).
    lines: Shape,
}

/// Makes wires of edges lying in common planes: the non-degenerated geometric edges of
/// `edges` are intersected unless `shared`, grouped by the planes of the curves (normals
/// parallel within `angular_tolerance`) and of connected straight edges, and the edges left
/// over make their own wires. Returns 0, 1 for no edge, 2 when the intersection failed
/// (`EdgesToWires`); `edges_to_wires(&edges, &mut wires, false, 1e-8)?`.
pub fn edges_to_wires(
    edges: &Shape,
    wires: &mut Shape,
    shared: bool,
    angular_tolerance: f64,
) -> Result<i32, Failure> {
    let input = geometric_edges(edges)?;
    if input.is_empty() {
        return Ok(1);
    }
    let mut result = new_compound();
    if input.len() == 1 {
        add_wire(&mut result, &input)?;
        *wires = result;
        return Ok(0);
    }
    let Some(shared_edges) = shared_edges(&input, shared)? else {
        return Ok(2);
    };
    let mut geometry = edge_geometry(&shared_edges)?;
    let mut fence: Vec<Direction3> = Vec::new();
    wires_of_curve_planes(&mut geometry, &mut fence, &mut result, angular_tolerance)?;
    wires_of_line_pairs(&mut geometry, &mut fence, &mut result, angular_tolerance)?;
    wires_of_unused_edges(&mut geometry, &mut result)?;
    *wires = result;
    Ok(0)
}

/// The non-degenerated geometric edges of a shape.
fn geometric_edges(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut found = Vec::new();
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let edge = topods::edge(explorer.current())?;
        if !BRepTool::degenerated(&edge)? && BRepTool::is_geometric_edge(&edge)? {
            found.push(explorer.current().clone());
        }
        explorer.next()?;
    }
    Ok(found)
}

/// The edges sharing their vertices: the General Fuse of the edges unless `shared`; None
/// when the General Fuse failed.
fn shared_edges(edges: &[Shape], shared: bool) -> Result<Option<Shape>, Failure> {
    if shared {
        return compound_of(edges).map(Some);
    }
    let mut builder = BOPAlgoBuilder::new();
    builder.set_arguments(&edges.iter().cloned().collect::<List<Shape>>());
    builder.perform();
    if builder.options().has_errors() {
        return Ok(None);
    }
    Ok(Some(builder.shape().clone()))
}

/// The plane of each curved edge, and the tangent of each straight edge.
fn edge_geometry(edges: &Shape) -> Result<EdgeGeometry, Failure> {
    let mut geometry = EdgeGeometry {
        planes: IndexedDataMap::with_hasher(ShapeMapHasher, 1),
        tangents: IndexedDataMap::with_hasher(ShapeMapHasher, 1),
        no_unique_plane: MapOfShape::default(),
        lines: new_compound(),
    };
    let builder = BRepBuilder::default();
    let mut explorer = Explorer::new(edges, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let edge = explorer.current().clone();
        explorer.next()?;
        let curve = BRepAdaptorCurve::with_edge(&topods::edge(&edge)?)?;
        if let Some(plane) = curve_plane(&curve)? {
            geometry.planes.add(edge, plane);
        } else if let Some(tangent) = curve_tangent(&curve)? {
            geometry
                .tangents
                .add(edge.clone(), Direction3::from_vector(tangent)?);
            builder.add(&mut geometry.lines, &edge)?;
            geometry.no_unique_plane.add(edge);
        }
    }
    Ok(geometry)
}

/// One group per new plane of a curved edge: the curved edges of parallel planes, and the
/// connected blocks of straight edges in that plane that touch them or form a planar wire.
fn wires_of_curve_planes(
    geometry: &mut EdgeGeometry,
    fence: &mut Vec<Direction3>,
    result: &mut Shape,
    angular_tolerance: f64,
) -> Result<(), Failure> {
    let mut used = MapOfShape::default();
    for position in 1..=geometry.planes.extent() {
        let edge = geometry.planes.find_key(position)?.clone();
        if !used.add(edge.clone()) {
            continue;
        }
        let normal = geometry
            .planes
            .find_from_index(position)?
            .position()
            .direction();
        fence.push(normal);
        let mut group = IndexedMapOfShape::default();
        group.add(edge.clone());
        let mut vertices = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&edge, ShapeEnum::Vertex, &mut vertices)?;
        add_parallel_planes(
            geometry,
            position,
            normal,
            angular_tolerance,
            (&mut group, &mut vertices, &mut used),
        )?;
        add_lines_in_plane(geometry, normal, angular_tolerance, &vertices, &mut group)?;
        make_wires(&group, result, false, geometry)?;
    }
    Ok(())
}

/// The later curved edges whose plane is parallel to `normal`, with their vertices.
fn add_parallel_planes(
    geometry: &EdgeGeometry,
    position: usize,
    normal: Direction3,
    angular_tolerance: f64,
    (group, vertices, used): (
        &mut IndexedMapOfShape,
        &mut IndexedMapOfShape,
        &mut MapOfShape,
    ),
) -> Result<(), Failure> {
    for later in position + 1..=geometry.planes.extent() {
        let plane = geometry.planes.find_from_index(later)?;
        if normal.is_parallel(plane.position().direction(), angular_tolerance) {
            let edge = geometry.planes.find_key(later)?.clone();
            group.add(edge.clone());
            used.add(edge.clone());
            topexp::map_shapes_of_type(&edge, ShapeEnum::Vertex, vertices)?;
        }
    }
    Ok(())
}

/// The connected blocks of straight edges normal to `normal` that share a vertex with the
/// group or form a planar wire on their own.
fn add_lines_in_plane(
    geometry: &mut EdgeGeometry,
    normal: Direction3,
    angular_tolerance: f64,
    vertices: &IndexedMapOfShape,
    group: &mut IndexedMapOfShape,
) -> Result<(), Failure> {
    let mut in_plane = Vec::new();
    for position in 1..=geometry.tangents.extent() {
        if normal.is_normal(
            *geometry.tangents.find_from_index(position)?,
            angular_tolerance,
        ) {
            in_plane.push(geometry.tangents.find_key(position)?.clone());
        }
    }
    for block in connexity_blocks(&compound_of(&in_plane)?)? {
        let block_shape = compound_of(&block)?;
        let touches = sub_shapes(&block_shape, ShapeEnum::Vertex)?
            .iter()
            .any(|vertex| vertices.contains(vertex));
        if touches
            || wire_plane(
                &block_shape,
                &mut geometry.tangents,
                &mut geometry.no_unique_plane,
            )?
            .is_some()
        {
            for edge in block {
                group.add(edge);
            }
        }
    }
    Ok(())
}

/// One group per new plane of two connected non-parallel straight edges: all straight edges
/// in that plane, keeping only the wires that are planar.
fn wires_of_line_pairs(
    geometry: &mut EdgeGeometry,
    fence: &mut Vec<Direction3>,
    result: &mut Shape,
    angular_tolerance: f64,
) -> Result<(), Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(
        &geometry.lines,
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut ancestors,
    )?;
    for position in 1..=ancestors.extent() {
        let edges: Vec<Shape> = ancestors
            .find_from_index(position)?
            .iter()
            .cloned()
            .collect();
        if edges.len() < 2 {
            continue;
        }
        for first in &edges {
            for second in edges.iter().filter(|second| !second.is_same(first)) {
                let Some(normal) =
                    new_pair_normal(geometry, first, second, fence, angular_tolerance)?
                else {
                    continue;
                };
                let mut group = IndexedMapOfShape::default();
                group.add(first.clone());
                group.add(second.clone());
                for index in 1..=geometry.tangents.extent() {
                    if normal.is_normal(
                        *geometry.tangents.find_from_index(index)?,
                        angular_tolerance,
                    ) {
                        group.add(geometry.tangents.find_key(index)?.clone());
                    }
                }
                make_wires(&group, result, true, geometry)?;
            }
        }
    }
    Ok(())
}

/// The normal of two non-parallel tangents when no fenced normal is parallel to it; it is
/// fenced then.
fn new_pair_normal(
    geometry: &EdgeGeometry,
    first: &Shape,
    second: &Shape,
    fence: &mut Vec<Direction3>,
    angular_tolerance: f64,
) -> Result<Option<Direction3>, Failure> {
    let first_tangent = *geometry.tangents.find_from_key(first)?;
    let second_tangent = *geometry.tangents.find_from_key(second)?;
    if first_tangent.is_parallel(second_tangent, angular_tolerance) {
        return Ok(None);
    }
    let normal = first_tangent.crossed(second_tangent)?;
    if fence
        .iter()
        .any(|known| normal.is_parallel(*known, angular_tolerance))
    {
        return Ok(None);
    }
    fence.push(normal);
    Ok(Some(normal))
}

/// The straight edges in no wire yet, made into wires.
fn wires_of_unused_edges(geometry: &mut EdgeGeometry, result: &mut Shape) -> Result<(), Failure> {
    let mut used = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(result, ShapeEnum::Edge, &mut used)?;
    let mut alone = IndexedMapOfShape::default();
    for position in 1..=geometry.tangents.extent() {
        let edge = geometry.tangents.find_key(position)?;
        if !used.contains(edge) {
            alone.add(edge.clone());
        }
    }
    make_wires(&alone, result, false, geometry)
}

/// One wire per connected block of the edges, only the planar ones when `check_plane`
/// (`MakeWires`).
fn make_wires(
    edges: &IndexedMapOfShape,
    result: &mut Shape,
    check_plane: bool,
    geometry: &mut EdgeGeometry,
) -> Result<(), Failure> {
    let all: Vec<Shape> = edges.iter().cloned().collect();
    for block in connexity_blocks(&compound_of(&all)?)? {
        let block_shape = compound_of(&block)?;
        if check_plane
            && wire_plane(
                &block_shape,
                &mut geometry.tangents,
                &mut geometry.no_unique_plane,
            )?
            .is_none()
        {
            continue;
        }
        add_wire(result, &block)?;
    }
    Ok(())
}

/// Adds one wire of the edges to the result.
fn add_wire(result: &mut Shape, edges: &[Shape]) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, edge)?;
    }
    builder.add(result, &wire)
}

/// The vertex-connected blocks of the edges of a compound.
fn connexity_blocks(edges: &Shape) -> Result<Vec<Vec<Shape>>, Failure> {
    let mut blocks = Vec::new();
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    make_connexity_blocks(
        edges,
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut blocks,
        &mut ancestors,
    )?;
    Ok(blocks)
}

/// The sub-shapes of `kind` in explorer order.
fn sub_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}

/// A compound of the shapes.
pub(super) fn compound_of(shapes: &[Shape]) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = new_compound();
    for shape in shapes {
        builder.add(&mut compound, shape)?;
    }
    Ok(compound)
}

/// A new empty compound.
pub(super) fn new_compound() -> Shape {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    compound.into_shape()
}
