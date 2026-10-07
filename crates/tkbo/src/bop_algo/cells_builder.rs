//! `BOPAlgo_CellsBuilder`: the split parts (cells) of the arguments, from which any
//! combination is taken into the result, optionally with materials whose internal
//! boundaries are removed.
#[path = "cells_builder/boundaries.rs"]
mod boundaries;
#[path = "cells_builder/parts.rs"]
mod parts;
use super::builder::history::HistorySources;
use super::builder::perform::{no_structure, perform_internal1, perform_on, perform_with};
use super::{BOPAlgoBuilder, BOPAlgoPaveFiller, BuilderHooks};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, IndexedDataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use std::ops::{Deref, DerefMut};

/// The split parts of the shapes and the argument shapes each part comes from (`myIndex`).
type PartOrigins = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The cells builder: the General Fuse of the arguments, whose split parts are added to or
/// removed from the result by the arguments they belong to and do not belong to.
///
/// ```
/// use rustycad_tkbo::bop_algo::BOPAlgoCellsBuilder;
/// let cells = BOPAlgoCellsBuilder::new();
/// assert!(cells.all_parts().is_null());
/// ```
pub struct BOPAlgoCellsBuilder {
    builder: BOPAlgoBuilder,
    all_parts: Shape,
    index: PartOrigins,
    materials: DataMap<i32, List<Shape>>,
    shape_material: DataMap<Shape, i32, ShapeMapHasher>,
    map_modified: DataMap<Shape, Shape, ShapeMapHasher>,
    /// The source shapes of the General Fuse for the history of every later result.
    history_sources: HistorySources,
}

impl Default for BOPAlgoCellsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BOPAlgoCellsBuilder {
    type Target = BOPAlgoBuilder;
    fn deref(&self) -> &BOPAlgoBuilder {
        &self.builder
    }
}

impl DerefMut for BOPAlgoCellsBuilder {
    fn deref_mut(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.builder
    }
}

impl BOPAlgoCellsBuilder {
    /// A builder without arguments (`BOPAlgo_CellsBuilder()`; the allocator overload maps here
    /// too); `BOPAlgoCellsBuilder::new()`.
    pub fn new() -> Self {
        Self {
            builder: BOPAlgoBuilder::new(),
            all_parts: Shape::default(),
            index: IndexedDataMap::with_hasher(ShapeMapHasher, 100),
            materials: DataMap::with_hasher(DefaultHashPolicy, 100),
            shape_material: DataMap::with_hasher(ShapeMapHasher, 100),
            map_modified: DataMap::with_hasher(ShapeMapHasher, 100),
            history_sources: Vec::new(),
        }
    }

    /// Forgets the arguments, the parts and the materials (`Clear`); `cells.clear()`.
    pub fn clear(&mut self) {
        self.builder.clear();
        self.index.clear(false);
        self.materials.clear(false);
        self.shape_material.clear(false);
        self.map_modified.clear(false);
    }

    /// The compound of all split parts (`GetAllParts`); `cells.all_parts()`.
    pub fn all_parts(&self) -> &Shape {
        &self.all_parts
    }

    /// Splits the arguments with an own pave filler and indexes the parts; the result starts
    /// empty (`Perform`); `cells.perform()`.
    pub fn perform(&mut self) {
        let shapes = self.builder.arguments.clone();
        perform_on(self, shapes);
    }

    /// Splits the arguments with the results of `filler` (`PerformWithFiller`);
    /// `cells.perform_with_filler(&mut filler)`.
    pub fn perform_with_filler(&mut self, filler: &mut BOPAlgoPaveFiller) {
        perform_with(self, filler);
    }

    /// Empties the result and forgets the materials (`RemoveAllFromResult`);
    /// `cells.remove_all_from_result()?`.
    pub fn remove_all_from_result(&mut self) -> Result<(), Failure> {
        let mut compound = Compound::default();
        BRepBuilder::default().make_compound(&mut compound);
        self.builder.shape = compound.into_shape();
        self.materials.clear(false);
        self.shape_material.clear(false);
        self.map_modified.clear(false);
        self.prepare_history()
    }

    /// The General Fuse with the history off, then the index of the parts and an empty
    /// result; the history setting is restored only after a successful split
    /// (`PerformInternal1`).
    fn run(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        let fill_history = self.builder.has_history();
        self.builder.set_to_fill_history(false);
        perform_internal1(&mut self.builder, filler)?;
        if self.builder.options().has_errors() {
            return Ok(());
        }
        let structure = filler.data_structure().ok_or_else(no_structure)?;
        self.history_sources = self.builder.history_sources(structure)?;
        self.index_parts()?;
        self.remove_all_from_result()?;
        self.builder.set_to_fill_history(fill_history);
        Ok(())
    }

    /// The history of the current result, with the unified shapes (`PrepareHistory`).
    fn prepare_history(&mut self) -> Result<(), Failure> {
        let map_modified = &self.map_modified;
        let loc_modified =
            |builder: &BOPAlgoBuilder, shape: &Shape| loc_modified(builder, map_modified, shape);
        self.builder
            .prepare_history_from(&self.history_sources, &loc_modified)
    }
}

impl BuilderHooks for BOPAlgoCellsBuilder {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.builder
    }

    fn perform_internal1(&mut self, filler: &mut BOPAlgoPaveFiller) -> Result<(), Failure> {
        self.run(filler)
    }
}

/// The General Fuse splits of `shape` with each split replaced by the shape it was unified
/// into, each once; without splits, the unified shape only (`LocModified`).
fn loc_modified(
    builder: &BOPAlgoBuilder,
    map_modified: &DataMap<Shape, Shape, ShapeMapHasher>,
    shape: &Shape,
) -> Option<List<Shape>> {
    let splits = builder.loc_modified(shape);
    if map_modified.is_empty() {
        return splits.cloned();
    }
    let Some(splits) = splits else {
        let unified = map_modified.seek(shape)?;
        return Some([unified.clone()].into_iter().collect());
    };
    let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    let mut shapes = List::default();
    for split in splits.iter() {
        let split = map_modified.seek(split).unwrap_or(split);
        if fence.add(split.clone()) {
            shapes.append(split.clone());
        }
    }
    Some(shapes)
}

/// The direct children of a shape.
fn children(shape: &Shape) -> Vec<Shape> {
    ShapeIterator::new(shape, true, true).collect()
}
