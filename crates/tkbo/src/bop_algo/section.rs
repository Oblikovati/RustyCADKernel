//! `BOPAlgo_Section`: the section of the arguments, made of the section edges and vertices
//! and the common edges of the arguments.
use super::alerts::alert;
use super::builder::perform::{no_structure, perform_on, perform_with};
use super::{AlertTooFewArguments, BOPAlgoBuilder, BOPAlgoPaveFiller, BuilderHooks};
use crate::bop_ds::DataStructure;
use crate::bop_tools::make_container;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::ops::{Deref, DerefMut};

/// The section algorithm: the General Fuse of vertices and edges, then the section.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoSection;
/// let section = BOPAlgoSection::new();
/// assert!(section.arguments().is_empty());
/// ```
#[derive(Default)]
pub struct BOPAlgoSection {
    builder: BOPAlgoBuilder,
}

impl Deref for BOPAlgoSection {
    type Target = BOPAlgoBuilder;
    fn deref(&self) -> &BOPAlgoBuilder {
        &self.builder
    }
}

impl DerefMut for BOPAlgoSection {
    fn deref_mut(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.builder
    }
}

impl BOPAlgoSection {
    /// An empty section (`BOPAlgo_Section()`); `BOPAlgoSection::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intersects the arguments with an own pave filler, then builds the section
    /// (`Perform`); `section.perform();`.
    pub fn perform(&mut self) {
        let shapes = self.builder.arguments.clone();
        perform_on(self, shapes);
    }

    /// Builds the section from the results of `filler` (`PerformWithFiller`).
    /// `section.perform_with_filler(&mut filler);`.
    pub fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        perform_with(self, filler);
    }

    /// The pipeline of the section (`PerformInternal1`).
    fn run(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        self.builder.context = filler.context_handle();
        self.check_data(filler);
        if self.builder.options().has_errors() {
            return Ok(());
        }
        self.builder.prepare();
        let Some(structure) = filler.data_structure_mut() else {
            return Err(no_structure());
        };
        self.builder.fill_images_vertices(structure)?;
        self.builder.build_result_default(ShapeEnum::Vertex)?;
        self.builder.fill_images_edges(structure)?;
        self.builder.build_result_default(ShapeEnum::Edge)?;
        self.build_section(structure)?;
        if self.builder.options().has_errors() {
            return Ok(());
        }
        self.builder.prepare_history(structure)?;
        if self.builder.options().has_errors() {
            return Ok(());
        }
        self.builder.post_treat(structure)
    }

    /// The section shape (`BuildSection`): the section vertices and edges of the faces,
    /// the edges of edge/face common blocks, and the vertices and edges shared by the
    /// boundaries of several arguments, as edges at their vertices or alone vertices.
    pub fn build_section(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        self.builder.options().clear();
        let mut sections = Shape::default();
        make_container(ShapeEnum::Compound, &mut sections);
        add_face_sections(structure, &mut sections)?;
        add_face_common_edges(structure, &mut sections)?;
        let counts = self.boundary_counts()?;
        let mut vertex_edges = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &sections,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut vertex_edges,
        )?;
        for position in 1..=counts.extent() {
            if *counts.find_from_index(position)? > 1 {
                topexp::map_shapes_and_ancestors(
                    counts.find_key(position)?,
                    ShapeEnum::Vertex,
                    ShapeEnum::Edge,
                    &mut vertex_edges,
                )?;
            }
        }
        let builder = BRepBuilder::default();
        let mut result = Shape::default();
        make_container(ShapeEnum::Compound, &mut result);
        let mut fence = MapOfShape::default();
        for position in 1..=vertex_edges.extent() {
            let edges = vertex_edges.find_from_index(position)?;
            if edges.is_empty() {
                // Alone vertex.
                let vertex = vertex_edges.find_key(position)?;
                if fence.add(vertex.clone()) {
                    builder.add(&mut result, vertex)?;
                }
                continue;
            }
            for edge in edges.iter() {
                if fence.add(edge.clone()) {
                    builder.add(&mut result, edge)?;
                }
            }
        }
        self.builder.shape = result;
        Ok(())
    }

    /// For each vertex and edge of the boundaries of the arguments (or of their splits), the
    /// number of arguments it bounds (`aMSI`).
    fn boundary_counts(&self) -> Result<IndexedDataMap<Shape, i32, ShapeMapHasher>, Failure> {
        let mut unique = MapOfShape::default();
        let mut counts: IndexedDataMap<Shape, i32, ShapeMapHasher> =
            IndexedDataMap::with_hasher(ShapeMapHasher, 100);
        for argument in self.builder.arguments.iter() {
            if !unique.add(argument.clone()) {
                continue;
            }
            for shape in self.boundary_splits(argument)?.iter() {
                match counts.change_seek(shape) {
                    Some(count) => *count += 1,
                    None => {
                        counts.add(shape.clone(), 1);
                    }
                }
            }
        }
        Ok(counts)
    }

    /// The vertices and edges of the edge and vertex images of the argument.
    fn boundary_splits(&self, argument: &Shape) -> Result<IndexedMapOfShape, Failure> {
        let mut fence = MapOfShape::default();
        let mut parts = Vec::new();
        for kind in [ShapeEnum::Edge, ShapeEnum::Vertex] {
            for part in Explorer::new(argument, kind, ShapeEnum::Shape)? {
                if fence.add(part.clone()) {
                    parts.push(part);
                }
            }
        }
        let mut splits = IndexedMapOfShape::default();
        for part in parts {
            let images: Vec<Shape> = match self.builder.images().seek(&part) {
                Some(images) => images.iter().cloned().collect(),
                None => vec![part],
            };
            for image in images {
                topexp::map_shapes_of_type(&image, ShapeEnum::Vertex, &mut splits)?;
                topexp::map_shapes_of_type(&image, ShapeEnum::Edge, &mut splits)?;
            }
        }
        Ok(splits)
    }
}

/// The section vertices, the new or interfered In vertices and the section edges of the
/// faces.
fn add_face_sections(structure: &DataStructure, sections: &mut Shape) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for index in 0..structure.nb_source_shapes() {
        if structure.shape_info(index)?.shape_type() != ShapeEnum::Face {
            continue;
        }
        let info = structure.face_info(index)?;
        for vertex in info.vertices_section().iter() {
            builder.add(sections, structure.shape(*vertex)?)?;
        }
        for &vertex in info.vertices_in().iter() {
            if vertex >= 0
                && (structure.is_new_shape(vertex) || structure.has_interference_for(vertex))
            {
                builder.add(sections, structure.shape(vertex)?)?;
            }
        }
        for block in info.pave_blocks_section().iter() {
            let edge = block.lock().expect("pave-block lock").edge();
            builder.add(sections, structure.shape(edge)?)?;
        }
    }
    Ok(())
}

/// The edges of the common blocks lying on faces.
fn add_face_common_edges(structure: &DataStructure, sections: &mut Shape) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for blocks in structure.pave_blocks_pool() {
        for block in blocks {
            let Some(common) = structure.common_block(block) else {
                continue;
            };
            let first = {
                let common = common.lock().expect("common-block lock");
                if common.faces().is_empty() {
                    continue;
                }
                std::sync::Arc::clone(common.pave_block1()?)
            };
            let edge = first.lock().expect("pave-block lock").edge();
            builder.add(sections, structure.shape(edge)?)?;
        }
    }
    Ok(())
}

impl BuilderHooks for BOPAlgoSection {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.builder
    }

    /// At least one argument, then the filler report merged (`CheckData`).
    fn check_data(&mut self, filler: &BOPAlgoPaveFiller) {
        if self.builder.arguments.is_empty() {
            self.builder
                .options()
                .add_error(&alert(AlertTooFewArguments));
            return;
        }
        self.builder.check_filler(filler);
    }

    fn perform_internal1(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        self.run(filler)
    }
}
