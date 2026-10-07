//! `BOPAlgo_BuilderFace::PerformInternalShapes`: the internal edges put into the faces that
//! contain them, with `MakeInternalWires` and the `IsInside` classification.
use super::BuilderFace;
use crate::bop_algo::AlertFaceBuilderUnusedEdges;
use crate::bop_tools::{BoxSelector, BoxSet};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{add_uv_bounds, add_uv_bounds_of_edge};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{IndexedMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::bnd::{BoundingBox2, box2_to_bvh};
use rustycad_tkmath::bvh::{BvhSetTraverse, BvhVec2d};
use std::sync::Arc;

impl BuilderFace {
    /// Puts each internal edge into the face containing it, warning about the edges left
    /// over (`PerformInternalShapes`).
    pub(super) fn perform_internal_shapes(&mut self) -> Result<(), Failure> {
        if self.area.avoid_internal_shapes || self.area.loops_internal.is_empty() {
            // Internal edges are avoided by option, or none is left to classify.
            return Ok(());
        }
        let mut edges: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        let mut tree: BoxSet<BvhVec2d> = BoxSet::default();
        for wire in self.area.loops_internal.iter() {
            for edge in ShapeIterator::new(wire, true, true) {
                if edges.contains(&edge) {
                    continue;
                }
                let mut bounds = BoundingBox2::default();
                add_uv_bounds_of_edge(&self.face, &Edge::unchecked(edge.clone()), &mut bounds)?;
                // The index of the edge in the map is the index of its box in the tree.
                tree.add(edges.add(edge) as i32, box2_to_bvh(&bounds)?);
            }
        }
        tree.build()?;
        let handle = self.area.context.clone();
        let context = super::required_context(&handle)?;
        let mut done: Map<i32> = Map::new(1);
        for face in self.area.areas.iter() {
            let face = Face::unchecked(face.clone());
            if place_inside_edges(&face, &edges, &mut tree, &mut done, context)?
                && done.extent() == edges.extent()
            {
                // All edges are classified and added into the faces.
                return Ok(());
            }
        }
        self.warn_unused_edges(&edges, &done)
    }

    fn warn_unused_edges(
        &mut self,
        edges: &IndexedMap<Shape, ShapeMapHasher>,
        done: &Map<i32>,
    ) -> Result<(), Failure> {
        let mut unused: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        for index in 1..=edges.extent() {
            if !done.contains(&(index as i32)) {
                unused.add(edges.find_key(index)?.clone());
            }
        }
        let wires = make_internal_wires(&unused)?;
        let builder = BRepBuilder::default();
        let mut warning = compound()?;
        builder.add(&mut warning, &self.face)?;
        if wires.len() == 1 {
            builder.add(&mut warning, wires.iter().next().expect("one wire"))?;
        } else {
            let mut group = compound()?;
            for wire in wires.iter() {
                builder.add(&mut group, wire)?;
            }
            builder.add(&mut warning, &group)?;
        }
        let alert: AlertHandle = Arc::new(AlertFaceBuilderUnusedEdges::new(&warning));
        self.area.options.add_warning(&alert);
        Ok(())
    }
}

/// Adds to `face` the internal wires of the unclassified edges inside it; true when the
/// face's box met some edge box (the native `Select` returned a non-zero count).
fn place_inside_edges(
    face: &Face,
    edges: &IndexedMap<Shape, ShapeMapHasher>,
    tree: &mut BoxSet<BvhVec2d>,
    done: &mut Map<i32>,
    context: &Context,
) -> Result<bool, Failure> {
    let mut bounds = BoundingBox2::default();
    add_uv_bounds(face, &mut bounds)?;
    let mut selector = BoxSelector::default();
    selector.set_bvh_set(Some(tree));
    selector.set_box(box2_to_bvh(&bounds)?);
    if selector.select()? == 0 {
        return Ok(false);
    }
    let mut inside: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::with_hasher(ShapeMapHasher, 1);
    for index in selector.indices().iter().copied() {
        if done.contains(&index) {
            continue;
        }
        let edge = edges.find_key(index as usize)?;
        if is_inside(edge, face, context)? {
            inside.add(edge.clone());
            done.add(index);
        }
    }
    if inside.is_empty() {
        return Ok(false);
    }
    let mut shape = face.clone().into_shape();
    for wire in make_internal_wires(&inside)?.iter() {
        BRepBuilder::default().add(&mut shape, wire)?;
    }
    Ok(true)
}

fn compound() -> Result<Shape, Failure> {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    Ok(compound.into_shape())
}

/// Connected INTERNAL wires of the edges (`MakeInternalWires`).
pub(super) fn make_internal_wires(
    edges: &IndexedMap<Shape, ShapeMapHasher>,
) -> Result<List<Shape>, Failure> {
    let mut vertex_edges = IndexedDataMapOfShapeListOfShape::default();
    for edge in edges.iter() {
        topexp::map_shapes_and_ancestors(
            edge,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut vertex_edges,
        )?;
    }
    let mut added: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    let mut wires = List::default();
    for edge in edges.iter() {
        if !added.add(edge.clone()) {
            continue;
        }
        wires.append(internal_wire(edge, &vertex_edges, &mut added)?);
    }
    Ok(wires)
}

fn internal_wire(
    seed: &Shape,
    vertex_edges: &IndexedDataMapOfShapeListOfShape,
    added: &mut Map<Shape, ShapeMapHasher>,
) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let mut wire = wire.into_shape();
    builder.add(&mut wire, &seed.oriented(Orientation::Internal))?;
    let mut position = 0;
    while let Some(edge) = ShapeIterator::new(&wire, true, true).nth(position) {
        position += 1;
        for vertex in Explorer::new(&edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            for other in vertex_edges.find_from_key(&vertex)?.iter() {
                if added.add(other.clone()) {
                    builder.add(&mut wire, &other.oriented(Orientation::Internal))?;
                }
            }
        }
    }
    wire.set_closed(BRepTool::is_closed(&wire)?)?;
    Ok(wire)
}

/// Whether `shape` lies inside `face`: the middle of its first non-degenerated edge with a
/// p-curve is classified, and an edge of the face makes it outside (`IsInside`).
pub(super) fn is_inside(shape: &Shape, face: &Shape, context: &Context) -> Result<bool, Failure> {
    let mut face_edges: IndexedMap<Shape, ShapeMapHasher> =
        IndexedMap::with_hasher(ShapeMapHasher, 1);
    topexp::map_shapes_of_type(face, ShapeEnum::Edge, &mut face_edges)?;
    let face = Face::unchecked(face.clone());
    let classifier = context.fclass2d(&face)?;
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = Edge::unchecked(edge);
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        if face_edges.contains(&edge) {
            // The face contains the edge, so the shape cannot be inside it.
            return Ok(false);
        }
        let found = BRepTool::curve_on_face(&edge, &face)?;
        let Some(pcurve) = found.pcurve.get() else {
            continue;
        };
        let point = pcurve.value((found.first + found.last) / 2.)?;
        return Ok(classifier.read().perform(point, true)? == State::In);
    }
    Ok(false)
}
