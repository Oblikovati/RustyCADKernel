//! `BOPAlgo_WireSplitter::SplitBlock` (BOPAlgo_WireSplitter_1.cxx): the edge/vertex map
//! of one irregular connexity block and the walk over it.
use super::angles::{angle_2d, refine_angles};
use super::path::{PathWalk, path};
use super::{EdgeInfo, WireSplitter};
use crate::bop_tools::ConnexityBlock;
use crate::bop_tools::algo_tools2d::has_curve_on_surface_simple;
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;

/// The edges at each vertex of the block (`mySmartMap`).
pub(super) type SmartMap = IndexedDataMap<Shape, Vec<EdgeInfo>, ShapeMapHasher>;
/// Whether a vertex closes an edge on itself (`MyDataMapOfShapeBoolean aVertMap`).
pub(super) type ClosedVertices = DataMap<Shape, bool, ShapeMapHasher>;

impl WireSplitter {
    /// Splits one irregular connexity block of edges of `face` into loops (`SplitBlock`).
    /// `WireSplitter::split_block(&face, &mut block, &context)?`.
    pub fn split_block(
        face: &Face,
        block: &mut ConnexityBlock,
        context: &Handle<Context>,
    ) -> Result<(), Failure> {
        let mut smart: SmartMap = IndexedDataMap::with_hasher(ShapeMapHasher, 100);
        let mut closed: ClosedVertices = DataMap::with_hasher(ShapeMapHasher, 1);
        let mut boundary: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        fill_smart_map(face, block.shapes(), &mut smart, &mut closed, &mut boundary)?;
        if each_vertex_simple(&smart) && edges_distinct(block.shapes()) {
            let wire = Self::make_wire(block.shapes())?;
            block.change_loops().append(wire.into_shape());
            return Ok(());
        }
        let context = context.get().ok_or_else(missing_context)?;
        let adaptor = context.surface_adaptor(face)?;
        let surface = adaptor.read();
        set_angles(face, &surface, &mut smart, &boundary, context)?;
        refine_angles(face, &mut smart, context)?;
        let walk = PathWalk {
            surface: &surface,
            face,
            closed: &closed,
        };
        for index in 1..=smart.extent() {
            let vertex = Vertex::unchecked(smart.find_key(index)?.clone());
            for position in 0..smart.find_from_index(index)?.len() {
                let info = &smart.find_from_index(index)?[position];
                if !info.is_in() && !info.passed() {
                    let edge = info.edge().clone();
                    path(&walk, &vertex, &edge, (index, position), block, &mut smart)?;
                }
            }
        }
        Ok(())
    }
}

/// Fills the vertex/edge map, the closed-vertex flags and the boundary edges (`aMS`: the
/// edges met once, or closed on the face).
fn fill_smart_map(
    face: &Face,
    edges: &List<Shape>,
    smart: &mut SmartMap,
    closed: &mut ClosedVertices,
    boundary: &mut Map<Shape, ShapeMapHasher>,
) -> Result<(), Failure> {
    for shape in edges.iter() {
        let edge = Edge::unchecked(shape.clone());
        if !has_curve_on_surface_simple(&edge, face)? {
            continue;
        }
        let mut is_closed =
            BRepTool::degenerated(&edge)? || BRepTool::is_closed_on_face(&edge, face)?;
        if !boundary.add(shape.clone()) && !is_closed {
            boundary.remove(shape);
        }
        let mut first = Shape::default();
        for (position, vertex) in ShapeIterator::new(shape, true, true).enumerate() {
            add_edge_info(smart, &vertex, &edge);
            if position == 0 {
                first = vertex.clone();
            } else {
                is_closed = is_closed || first.is_same(&vertex);
            }
            update_closed(closed, &vertex, is_closed);
        }
    }
    Ok(())
}

fn add_edge_info(smart: &mut SmartMap, vertex: &Shape, edge: &Edge) {
    let mut info = EdgeInfo::new();
    info.set_edge(edge);
    info.set_in_flag(vertex.orientation() == Orientation::Reversed);
    if let Some(infos) = smart.change_seek(vertex) {
        infos.push(info);
        return;
    }
    smart.add(vertex.clone(), vec![info]);
}

fn update_closed(closed: &mut ClosedVertices, vertex: &Shape, is_closed: bool) {
    if let Ok(flag) = closed.change_find(vertex) {
        if is_closed {
            *flag = is_closed;
        }
        return;
    }
    closed.bind(vertex.clone(), is_closed);
}

/// Whether every vertex has exactly one entering and one leaving edge.
fn each_vertex_simple(smart: &SmartMap) -> bool {
    smart.iter().all(|infos| {
        let entering = infos.iter().filter(|info| info.is_in()).count();
        entering == 1 && infos.len() - entering == 1
    })
}

/// Whether no edge is repeated with the same orientation or more than twice (the check on
/// TShape coincidence of the native "nothing to do" test).
fn edges_distinct(edges: &List<Shape>) -> bool {
    let mut occurrences: IndexedDataMap<Shape, Vec<Shape>, ShapeMapHasher> =
        IndexedDataMap::with_hasher(ShapeMapHasher, 100);
    for edge in edges.iter() {
        if let Some(found) = occurrences.change_seek(edge) {
            found.push(edge.clone());
            continue;
        }
        occurrences.add(edge.clone(), vec![edge.clone()]);
    }
    occurrences.iter().all(|found| match found.len() {
        1 => true,
        2 => !found[0].is_same(&found[1]),
        _ => false,
    })
}

/// Sets the inside flag and the 2D angle of every edge at every vertex.
fn set_angles(
    face: &Face,
    surface: &rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface,
    smart: &mut SmartMap,
    boundary: &Map<Shape, ShapeMapHasher>,
    context: &Context,
) -> Result<(), Failure> {
    for index in 1..=smart.extent() {
        let vertex = smart.find_key(index)?.clone();
        for info in smart.change_from_index(index)?.iter_mut() {
            info.set_is_inside(!boundary.contains(&info.edge().clone().into_shape()));
            let is_in = info.is_in();
            let oriented = if is_in {
                Orientation::Reversed
            } else {
                Orientation::Forward
            };
            let vertex = Vertex::unchecked(vertex.oriented(oriented));
            let angle = angle_2d(&vertex, info.edge(), face, surface, is_in, context)?;
            info.set_angle(angle);
        }
    }
    Ok(())
}

fn missing_context() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BOPAlgo_WireSplitter::SplitBlock: context is null; expected a created IntTools_Context",
        "",
    )
}
