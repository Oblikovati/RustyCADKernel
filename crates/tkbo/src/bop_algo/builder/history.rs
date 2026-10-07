//! `BOPAlgo_Builder::PrepareHistory`, `LocModified` and `LocGenerated`
//! (BOPAlgo_Builder_4.cxx).
use super::{BOPAlgoBuilder, required_context};
use crate::bop_ds::DataStructure;
use crate::bop_tools::is_split_shape_to_reverse;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The source shapes of the data structure that a history may hold, each with the shapes
/// generated from it whatever the result (`LocGenerated` before its result filter). Taken
/// once after the General Fuse, it lets a derived algorithm refill the history of a changed
/// result without the data structure.
pub(crate) type HistorySources = Vec<(Shape, Vec<Shape>)>;

/// `LocModified` of the algorithm filling the history: the splits of a shape.
pub(crate) type LocModified<'a> = &'a dyn Fn(&BOPAlgoBuilder, &Shape) -> Option<List<Shape>>;

impl BOPAlgoBuilder {
    /// Fills the history: the splits kept in the result as Modified, the new vertices and
    /// section edges kept in it as Generated, and the shapes without trace as Removed
    /// (`PrepareHistory`); `builder.prepare_history(&ds)?`.
    pub fn prepare_history(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        if !self.has_history() {
            return Ok(());
        }
        let sources = self.history_sources(structure)?;
        self.prepare_history_from(&sources, &|builder, shape| {
            builder.loc_modified(shape).cloned()
        })
    }

    /// The source shapes a history may hold with their generated shapes.
    pub(crate) fn history_sources(
        &self,
        structure: &DataStructure,
    ) -> Result<HistorySources, Failure> {
        let mut sources = Vec::new();
        for index in 0..structure.nb_source_shapes() {
            let shape = structure.shape(index)?.clone();
            if History::is_supported_type(&shape)? {
                let generated = self.generated_candidates(structure, &shape)?;
                sources.push((shape, generated));
            }
        }
        Ok(sources)
    }

    /// `PrepareHistory` on the source shapes of the General Fuse, with the `LocModified` of
    /// the algorithm.
    pub(crate) fn prepare_history_from(
        &mut self,
        sources: &HistorySources,
        loc_modified: LocModified<'_>,
    ) -> Result<(), Failure> {
        if !self.has_history() {
            return Ok(());
        }
        let history = Handle::new(History::new());
        self.history = history.clone();
        let history = history.get().expect("created history");
        self.map_shape.clear(false);
        let mut result_shapes = rustycad_tkbrep::toptools::MapOfShape::default();
        topexp::map_shapes_into_map(&self.shape, &mut result_shapes, true, true);
        self.map_shape = result_shapes;
        for (shape, generated) in sources {
            let modified = self.add_modified(shape, loc_modified(self, shape), history)?;
            for generated in generated
                .iter()
                .filter(|generated| self.map_shape.contains(generated))
            {
                history.add_generated(shape, generated)?;
            }
            if !modified && !self.map_shape.contains(shape) {
                history.remove(shape)?;
            }
        }
        Ok(())
    }

    /// Adds the splits of `shape` kept in the result as Modified, with the orientation of
    /// `shape`; true when one was added.
    fn add_modified(
        &self,
        shape: &Shape,
        splits: Option<List<Shape>>,
        history: &History,
    ) -> Result<bool, Failure> {
        let Some(splits) = splits else {
            return Ok(false);
        };
        let mut modified = false;
        for split in splits.iter() {
            if !self.map_shape.contains(split) {
                continue;
            }
            let mut split = split.clone();
            if matches!(split.shape_type()?, ShapeEnum::Vertex | ShapeEnum::Solid) {
                split.set_orientation(shape.orientation());
            } else {
                let mut error = 0;
                let context = required_context(&self.context)?;
                if is_split_shape_to_reverse(&split, shape, context, &mut error)? {
                    split.reverse();
                }
            }
            history.add_modified(shape, &split)?;
            modified = true;
        }
        Ok(modified)
    }

    /// The splits of `shape` (`LocModified`); `builder.loc_modified(&edge)`.
    pub fn loc_modified(&self, shape: &Shape) -> Option<&List<Shape>> {
        self.images.seek(shape)
    }

    /// The new vertices of the Edge/Edge and Edge/Face interferences of an edge or face, and
    /// the section edges and vertices of a face, kept in the result (`LocGenerated`).
    /// `builder.loc_generated(&ds, &face)?`.
    pub fn loc_generated(
        &self,
        structure: &DataStructure,
        shape: &Shape,
    ) -> Result<Vec<Shape>, Failure> {
        Ok(self
            .generated_candidates(structure, shape)?
            .into_iter()
            .filter(|generated| self.map_shape.contains(generated))
            .collect())
    }

    /// The shapes `LocGenerated` gives for `shape` before keeping the ones of the result.
    fn generated_candidates(
        &self,
        structure: &DataStructure,
        shape: &Shape,
    ) -> Result<Vec<Shape>, Failure> {
        let mut generated = Vec::new();
        if shape.is_null() {
            return Ok(generated);
        }
        let kind = shape.shape_type()?;
        if kind != ShapeEnum::Edge && kind != ShapeEnum::Face {
            return Ok(generated);
        }
        let index = structure.index(shape);
        if index < 0 || !structure.shape_info(index)?.has_reference() {
            // Unknown or untouched shape.
            return Ok(generated);
        }
        let is_face = kind == ShapeEnum::Face;
        new_vertices(structure, index, is_face, &mut generated)?;
        if is_face {
            face_section_shapes(structure, index, &mut generated)?;
        }
        Ok(generated)
    }
}

/// The new vertices (or their same-domain vertex) of the Edge/Edge (for an edge) and
/// Edge/Face interferences of the shape `index`, each once.
fn new_vertices(
    structure: &DataStructure,
    index: i32,
    is_face: bool,
    generated: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let mut fence: Map<i32> = Map::new(1);
    let new_vertices = structure
        .interf_ee_records()
        .iter()
        .filter(|_| !is_face)
        .map(|record| {
            (
                record.has_index_new(),
                record.contains(index),
                record.index_new(),
            )
        })
        .chain(structure.interf_ef_records().iter().map(|record| {
            (
                record.has_index_new(),
                record.contains(index),
                record.index_new(),
            )
        }));
    for (has_new, contains, mut vertex) in new_vertices {
        if !has_new || !contains {
            continue;
        }
        structure.has_shape_sd_into(vertex, &mut vertex);
        if fence.add(vertex) {
            generated.push(structure.shape(vertex)?.clone());
        }
    }
    Ok(())
}

/// The section edges and vertices of the face `index`.
fn face_section_shapes(
    structure: &DataStructure,
    index: i32,
    generated: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let info = structure.face_info(index)?;
    for block in info.pave_blocks_section().iter() {
        let edge = structure.shape(block.lock().expect("pave-block lock").edge())?;
        generated.push(edge.clone());
    }
    for vertex in info.vertices_section().iter() {
        generated.push(structure.shape(*vertex)?.clone());
    }
    Ok(())
}
