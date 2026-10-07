//! `BOPAlgo_BuilderSolid`: builds solids from a set of faces.
use super::tools::ShapeBoxMap;
use super::{AlertShellSplitterFailed, BuilderArea, ShellSplitter};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape, ShapeIterator, Shell};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{DataMap, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[path = "builder_solid/areas.rs"]
mod areas;
#[path = "builder_solid/internal.rs"]
mod internal;

/// Builds solids from faces: closed shells by the shell splitter, holes put into the
/// closest growth solid containing them, then the internal faces put into the solids.
///
/// ```
/// use rustycad_tkbo::bop_algo::BuilderSolid;
/// let mut builder = BuilderSolid::new();
/// builder.perform().unwrap();
/// assert!(builder.areas().is_empty());
/// ```
pub struct BuilderSolid {
    area: BuilderArea,
    boxes: ShapeBoxMap,
}

impl Default for BuilderSolid {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BuilderSolid {
    type Target = BuilderArea;
    fn deref(&self) -> &BuilderArea {
        &self.area
    }
}

impl DerefMut for BuilderSolid {
    fn deref_mut(&mut self) -> &mut BuilderArea {
        &mut self.area
    }
}

impl BuilderSolid {
    /// An empty builder (`BOPAlgo_BuilderSolid()`); `BuilderSolid::new()`.
    pub fn new() -> Self {
        Self {
            area: BuilderArea::new(),
            boxes: DataMap::with_hasher(rustycad_tkbrep::toptools::ShapeMapHasher, 1),
        }
    }

    /// The bounding boxes of the built solids (`GetBoxesMap`); `builder.boxes_map()`.
    pub fn boxes_map(&self) -> &ShapeBoxMap {
        &self.boxes
    }

    /// Builds the solids from the faces (`Perform`); `builder.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.area.options.clear();
        if self.area.shapes.is_empty() {
            return Ok(());
        }
        if self.area.context.is_null() {
            self.area.context = Handle::new(Context::default());
        }
        self.boxes.clear(false);
        // Native: a compound of the faces is made here and never used.
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

    /// Collects the faces to avoid (`PerformShapesToAvoid`): faces with a free edge and
    /// faces met twice on a non-seam edge, repeated until no more are found.
    fn perform_shapes_to_avoid(&mut self) -> Result<(), Failure> {
        self.area.shapes_to_avoid.clear(false);
        loop {
            let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
            for face in self.area.shapes.iter() {
                if !self.area.shapes_to_avoid.contains(face) {
                    topexp::map_shapes_and_ancestors(
                        face,
                        ShapeEnum::Edge,
                        ShapeEnum::Face,
                        &mut edge_faces,
                    )?;
                }
            }
            if !self.avoid_free_faces(&edge_faces)? {
                return Ok(());
            }
        }
    }

    /// One pass of `PerformShapesToAvoid`; true when a face was added.
    fn avoid_free_faces(
        &mut self,
        edge_faces: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<bool, Failure> {
        let mut found = false;
        for index in 1..=edge_faces.extent() {
            let edge = Edge::unchecked(edge_faces.find_key(index)?.clone());
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            let faces: Vec<&Shape> = edge_faces.find_from_index(index)?.iter().collect();
            let Some(first) = faces.first().copied() else {
                continue;
            };
            let internal = edge.orientation() == Orientation::Internal;
            if faces.len() == 1 {
                if internal {
                    continue;
                }
                found = true;
                self.area.shapes_to_avoid.add(first.clone());
            } else if faces.len() == 2 && faces[1].is_same(first) {
                if BRepTool::is_closed_on_face(&edge, &Face::unchecked(first.clone()))? || internal
                {
                    continue;
                }
                found = true;
                self.area.shapes_to_avoid.add(first.clone());
                self.area.shapes_to_avoid.add(faces[1].clone());
            }
        }
        Ok(found)
    }

    /// Builds the closed shells with the shell splitter and the internal shells from the
    /// faces left over (`PerformLoops`).
    fn perform_loops(&mut self) -> Result<(), Failure> {
        self.area.loops.clear();
        let context = self.context_handle();
        let context = internal::required_context(&context)?;
        let mut splitter = ShellSplitter::new();
        for face in self.area.shapes.iter() {
            if context.is_infinite_face(&Face::unchecked(face.clone()))? {
                self.area.loops.append(shell_of(face)?);
                continue;
            }
            if !self.area.shapes_to_avoid.contains(face) {
                splitter.add_start_element(face);
            }
        }
        splitter
            .options_mut()
            .set_run_parallel(self.area.options.run_parallel());
        splitter.perform()?;
        if splitter.options().has_errors() {
            self.warn_shell_splitter_failed(&splitter)?;
            return Ok(());
        }
        for shell in splitter.shells().iter() {
            self.area.loops.append(shell.clone());
        }
        self.avoid_unused_faces(context)?;
        self.make_internal_loops()
    }

    fn warn_shell_splitter_failed(&mut self, splitter: &ShellSplitter) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut faces = Compound::default();
        builder.make_compound(&mut faces);
        let mut faces = faces.into_shape();
        for face in splitter.start_elements().iter() {
            builder.add(&mut faces, face)?;
        }
        let warning: AlertHandle = Arc::new(AlertShellSplitterFailed::new(&faces));
        self.area.options.add_warning(&warning);
        Ok(())
    }

    /// Adds to the shapes to avoid every finite face in neither a shell nor the shapes to
    /// avoid.
    fn avoid_unused_faces(&mut self, context: &Context) -> Result<(), Failure> {
        let mut used: Map<Shape> = Map::new(1);
        for shell in self.area.loops.iter() {
            for face in ShapeIterator::new(shell, true, true) {
                used.add(face);
            }
        }
        for face in self.area.shapes_to_avoid.iter() {
            used.add(face.clone());
        }
        for face in self.area.shapes.iter() {
            if !context.is_infinite_face(&Face::unchecked(face.clone()))? && !used.contains(face) {
                self.area.shapes_to_avoid.add(face.clone());
            }
        }
        Ok(())
    }

    /// Groups the faces to avoid into connected internal shells (`myLoopsInternal`).
    fn make_internal_loops(&mut self) -> Result<(), Failure> {
        self.area.loops_internal.clear();
        let avoid: Vec<Shape> = self.area.shapes_to_avoid.iter().cloned().collect();
        let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
        for face in &avoid {
            topexp::map_shapes_and_ancestors(
                face,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut edge_faces,
            )?;
        }
        let mut added: Map<Shape> = Map::new(1);
        for face in &avoid {
            if !added.add(face.clone()) {
                continue;
            }
            let shell = grow_internal_shell(face, &edge_faces, &mut added)?;
            self.area.loops_internal.append(shell);
        }
        Ok(())
    }

    fn context_handle(&self) -> Handle<Context> {
        self.area.context.clone()
    }
}

/// A shell of one face (an infinite face kept as is).
fn shell_of(face: &Shape) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell = shell.into_shape();
    builder.add(&mut shell, face)?;
    Ok(shell)
}

/// A shell of `seed` and every face connected to it by an edge.
fn grow_internal_shell(
    seed: &Shape,
    edge_faces: &IndexedDataMapOfShapeListOfShape,
    added: &mut Map<Shape>,
) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut shell = shell_of(seed)?;
    let mut position = 0;
    while let Some(face) = ShapeIterator::new(&shell, true, true).nth(position) {
        position += 1;
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            for other in edge_faces.find_from_key(&edge)?.iter() {
                if added.add(other.clone()) {
                    builder.add(&mut shell, other)?;
                }
            }
        }
    }
    shell.set_closed(BRepTool::is_closed(&shell)?)?;
    Ok(shell)
}
