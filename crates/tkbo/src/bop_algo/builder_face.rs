//! `BOPAlgo_BuilderFace`: splits a face by a set of edges lying on it.
use super::alerts::alert;
use super::{AlertNullInputShapes, BuilderArea, WireEdgeSet, WireSplitter};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::ops::{Deref, DerefMut};

#[path = "builder_face/areas.rs"]
mod areas;
#[path = "builder_face/internal.rs"]
mod internal;
use internal::is_inside;

/// Builds the faces bounded by split edges on one face: wires from the edges, faces from
/// the outer wires with their holes, then the internal edges put into the faces.
///
/// ```
/// use rustycad_tkbo::bop_algo::BuilderFace;
/// let mut builder = BuilderFace::new();
/// builder.perform().unwrap();
/// assert!(builder.options().has_errors()); // no face was given
/// ```
pub struct BuilderFace {
    area: BuilderArea,
    face: Face,
    orientation: Orientation,
}

impl Default for BuilderFace {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BuilderFace {
    type Target = BuilderArea;
    fn deref(&self) -> &BuilderArea {
        &self.area
    }
}

impl DerefMut for BuilderFace {
    fn deref_mut(&mut self) -> &mut BuilderArea {
        &mut self.area
    }
}

impl BuilderFace {
    /// A builder with no face and an EXTERNAL orientation (`BOPAlgo_BuilderFace()`).
    /// `BuilderFace::new()`.
    pub fn new() -> Self {
        Self {
            area: BuilderArea::new(),
            face: Face::default(),
            orientation: Orientation::External,
        }
    }

    /// Sets the face to split, kept FORWARD with its orientation saved (`SetFace`).
    /// `builder.set_face(&face)`.
    pub fn set_face(&mut self, face: &Face) {
        self.orientation = face.orientation();
        self.face = Face::unchecked(face.oriented(Orientation::Forward));
    }

    /// The FORWARD face to split (`Face`); `builder.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }

    /// The orientation of the face given to `set_face` (`Orientation`); `builder.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Builds the split faces (`Perform`); `builder.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.area.options.clear();
        self.check_data();
        if self.area.options.has_errors() {
            return Ok(());
        }
        self.perform_shapes_to_avoid()?;
        if self.area.options.has_errors() {
            return Ok(());
        }
        self.perform_loops()?;
        if self.area.options.has_errors() {
            return Ok(());
        }
        self.perform_areas()?;
        if self.area.options.has_errors() {
            return Ok(());
        }
        self.perform_internal_shapes()
    }

    /// Reports a missing face and creates the context (`CheckData`).
    fn check_data(&mut self) {
        if self.face.is_null() {
            self.area.options.add_error(&alert(AlertNullInputShapes));
            return;
        }
        if self.area.context.is_null() {
            self.area.context = Handle::new(Context::default());
        }
    }

    /// Collects the edges to avoid (`PerformShapesToAvoid`): edges with a free vertex and
    /// edges met twice at a vertex, repeated until no more are found.
    fn perform_shapes_to_avoid(&mut self) -> Result<(), Failure> {
        self.area.shapes_to_avoid.clear(false);
        loop {
            let mut vertex_edges = IndexedDataMapOfShapeListOfShape::default();
            for edge in self.area.shapes.iter() {
                if !self.area.shapes_to_avoid.contains(edge) {
                    topexp::map_shapes_and_ancestors(
                        edge,
                        ShapeEnum::Vertex,
                        ShapeEnum::Edge,
                        &mut vertex_edges,
                    )?;
                }
            }
            if !self.avoid_free_edges(&vertex_edges)? {
                return Ok(());
            }
        }
    }

    /// One pass of `PerformShapesToAvoid` over the vertex/edge map; true when an edge was
    /// added to the shapes to avoid.
    fn avoid_free_edges(
        &mut self,
        vertex_edges: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<bool, Failure> {
        let mut found = false;
        for index in 1..=vertex_edges.extent() {
            let vertex = vertex_edges.find_key(index)?;
            let edges: Vec<&Shape> = vertex_edges.find_from_index(index)?.iter().collect();
            let Some(first) = edges.first().copied() else {
                continue;
            };
            if edges.len() == 1 {
                if BRepTool::degenerated(&Edge::unchecked(first.clone()))?
                    || vertex.orientation() == Orientation::Internal
                {
                    continue;
                }
                found = true;
                self.area.shapes_to_avoid.add(first.clone());
            } else if edges.len() == 2 && edges[1].is_same(first) {
                let (start, end) = topexp::vertices(&Edge::unchecked(first.clone()), false);
                if start.is_same(&end) {
                    continue;
                }
                found = true;
                self.area.shapes_to_avoid.add(first.clone());
                self.area.shapes_to_avoid.add(edges[1].clone());
            }
        }
        Ok(found)
    }

    /// Builds the boundary loops with the wire splitter and the internal loops from the
    /// edges left over (`PerformLoops`).
    fn perform_loops(&mut self) -> Result<(), Failure> {
        self.area.loops.clear();
        let mut wes = WireEdgeSet::new();
        wes.set_face(&self.face);
        for edge in self.area.shapes.iter() {
            if !self.area.shapes_to_avoid.contains(edge) {
                wes.add_start_element(edge);
            }
        }
        let mut splitter = WireSplitter::new();
        splitter.set_wes(wes);
        splitter
            .options_mut()
            .set_run_parallel(self.area.options.run_parallel());
        splitter.set_context(self.area.context.clone());
        splitter.perform()?;
        if splitter.options().has_errors() {
            return Ok(());
        }
        if let Some(wes) = splitter.wes() {
            for wire in wes.shapes().iter() {
                self.area.loops.append(wire.clone());
            }
        }
        self.avoid_unused_edges();
        self.make_internal_loops()
    }

    /// Adds to the shapes to avoid every edge in neither a loop nor the shapes to avoid.
    fn avoid_unused_edges(&mut self) {
        let mut used: Map<Shape> = Map::new(1);
        for wire in self.area.loops.iter() {
            for edge in ShapeIterator::new(wire, true, true) {
                used.add(edge);
            }
        }
        for edge in self.area.shapes_to_avoid.iter() {
            used.add(edge.clone());
        }
        for edge in self.area.shapes.iter() {
            if !used.contains(edge) {
                self.area.shapes_to_avoid.add(edge.clone());
            }
        }
    }

    /// Groups the edges to avoid into connected internal wires (`myLoopsInternal`).
    fn make_internal_loops(&mut self) -> Result<(), Failure> {
        self.area.loops_internal.clear();
        let mut vertex_edges = IndexedDataMapOfShapeListOfShape::default();
        let avoid: Vec<Shape> = self.area.shapes_to_avoid.iter().cloned().collect();
        for edge in &avoid {
            topexp::map_shapes_and_ancestors(
                edge,
                ShapeEnum::Vertex,
                ShapeEnum::Edge,
                &mut vertex_edges,
            )?;
        }
        let mut added: Map<Shape> = Map::new(1);
        let mut more = true;
        for edge in &avoid {
            if !more {
                break;
            }
            if !added.add(edge.clone()) {
                continue;
            }
            let wire = grow_internal_wire(edge, &vertex_edges, &mut added, avoid.len(), &mut more)?;
            self.area.loops_internal.append(wire.into_shape());
        }
        Ok(())
    }
}

/// A wire of `seed` and the edges connected to it, stopping when all `total` edges were
/// added (`more` turns false).
fn grow_internal_wire(
    seed: &Shape,
    vertex_edges: &IndexedDataMapOfShapeListOfShape,
    added: &mut Map<Shape>,
    total: usize,
    more: &mut bool,
) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let mut wire = wire.into_shape();
    builder.add(&mut wire, seed)?;
    let mut position = 0;
    while *more {
        let Some(edge) = ShapeIterator::new(&wire, true, true).nth(position) else {
            break;
        };
        position += 1;
        for vertex in ShapeIterator::new(&edge, true, true) {
            if !*more {
                break;
            }
            for other in vertex_edges.find_from_key(&vertex)?.iter() {
                if !*more {
                    break;
                }
                if added.add(other.clone()) {
                    builder.add(&mut wire, other)?;
                    if added.extent() == total {
                        *more = false;
                    }
                }
            }
        }
    }
    wire.set_closed(BRepTool::is_closed(&wire)?)?;
    Ok(Wire::unchecked(wire))
}

/// The context created by `CheckData`.
fn required_context(handle: &Handle<Context>) -> Result<&Context, Failure> {
    super::builder_area::required_context(handle, "BOPAlgo_BuilderFace")
}
