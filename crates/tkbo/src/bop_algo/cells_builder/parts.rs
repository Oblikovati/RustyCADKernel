//! The parts of `BOPAlgo_CellsBuilder`: indexing them by origin, finding them, and taking
//! them into or out of the result.
use super::boundaries::{is_container, make_typed_containers, type_to_explore};
use super::{BOPAlgoCellsBuilder, PartOrigins, children};
use crate::bop_tools::{dimension, treat_compound};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;

impl BOPAlgoCellsBuilder {
    /// Indexes every split part (of the dimension of its argument) to the arguments it comes
    /// from and collects all parts; for arguments of several dimensions, the sub-shapes of
    /// the parts of lower dimension inherit the origins of the parts (`IndexParts`).
    pub(super) fn index_parts(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut all_parts = Compound::default();
        builder.make_compound(&mut all_parts);
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        let mut dimensions: Map<i32> = Map::new(1);
        let arguments: Vec<Shape> = self.builder.arguments.iter().cloned().collect();
        for argument in &arguments {
            let mut leaves = Vec::new();
            treat_compound(argument, &mut leaves, None)?;
            for leaf in leaves {
                let leaf_dimension = dimension(&leaf)?;
                dimensions.add(leaf_dimension);
                for typed in sub_shapes(&leaf, type_to_explore(leaf_dimension))? {
                    for part in self.parts_of(&typed) {
                        index_origin(&mut self.index, &part, argument);
                        if fence.add(part.clone()) {
                            builder.add(&mut all_parts, &part)?;
                        }
                    }
                }
            }
        }
        self.all_parts = all_parts.into_shape();
        if dimensions.extent() == 1 {
            return Ok(());
        }
        self.index_lower_dimensions(&dimensions)
    }

    /// The split parts of a sub-shape of an argument, or the sub-shape itself.
    fn parts_of(&self, shape: &Shape) -> Vec<Shape> {
        match self.builder.images.seek(shape) {
            Some(images) => images.iter().cloned().collect(),
            None => vec![shape.clone()],
        }
    }

    /// The sub-shapes, of each lower dimension of the arguments, of every indexed part get
    /// the origins of the part, each origin once.
    fn index_lower_dimensions(&mut self, dimensions: &Map<i32>) -> Result<(), Failure> {
        for position in 1..=self.index.extent() {
            let part = self.index.find_key(position)?.clone();
            let origins = self.index.find_from_index(position)?.clone();
            let part_dimension = dimension(&part)?;
            for lower in dimensions
                .iter()
                .copied()
                .filter(|lower| *lower < part_dimension)
            {
                for sub_shape in sub_shapes(&part, type_to_explore(lower))? {
                    inherit_origins(&mut self.index, &sub_shape, &origins);
                }
            }
        }
        Ok(())
    }

    /// The parts of the shape of minimal dimension among `take` that every shape of `take`
    /// and no shape of `avoid` contains (`FindParts`).
    pub(super) fn find_parts(
        &self,
        take: &List<Shape>,
        avoid: &List<Shape>,
    ) -> Result<Vec<Shape>, Failure> {
        let mut parts = Vec::new();
        if take.is_empty() {
            return Ok(parts);
        }
        let avoided: Map<Shape, ShapeMapHasher> = shape_map(avoid);
        let taken_count = shape_map(take).extent();
        let (minimal, minimal_dimension) = minimal_dimension_shape(take)?;
        for typed in sub_shapes(&minimal, type_to_explore(minimal_dimension))? {
            for part in self.parts_of(&typed) {
                let Some(origins) = self.index.seek(&part) else {
                    continue;
                };
                if origins.extent() as usize >= taken_count && is_wanted(origins, take, &avoided) {
                    parts.push(part);
                }
            }
        }
        Ok(parts)
    }

    /// Adds the parts of `take` and not of `avoid` to the result, each once and only when it
    /// has no material yet; with a non-zero `material` the parts without one get it. The
    /// history is refilled when the result changed, unless `update` removes the internal
    /// boundaries instead (`AddToResult`); `cells.add_to_result(&take, &avoid, 1, false)?`.
    pub fn add_to_result(
        &mut self,
        take: &List<Shape>,
        avoid: &List<Shape>,
        material: i32,
        update: bool,
    ) -> Result<(), Failure> {
        let parts = self.find_parts(take, avoid)?;
        if parts.is_empty() {
            return Ok(());
        }
        let mut in_result: Map<Shape, ShapeMapHasher> =
            shape_map(&children(&self.builder.shape).into_iter().collect());
        let mut changed = false;
        for part in &parts {
            if in_result.add(part.clone()) && !self.shape_material.is_bound(part) {
                BRepBuilder::default().add(&mut self.builder.shape, part)?;
                changed = true;
            }
        }
        if material != 0 {
            self.assign_material(&parts, material);
        }
        if update {
            return self.remove_internal_boundaries();
        }
        if changed {
            return self.prepare_history();
        }
        Ok(())
    }

    /// Gives `material` to the parts without one and lists them under it.
    fn assign_material(&mut self, parts: &[Shape], material: i32) {
        let mut assigned = List::default();
        for part in parts {
            if !self.shape_material.is_bound(part) {
                self.shape_material.bind(part.clone(), material);
                assigned.append(part.clone());
            }
        }
        if assigned.is_empty() {
            return;
        }
        let list = match self.materials.change_seek(&material) {
            Some(list) => list,
            None => self.materials.bound(material, List::default()),
        };
        for part in assigned.iter() {
            list.append(part.clone());
        }
    }

    /// Makes all parts the result, all of `material` when non-zero; the history is refilled
    /// unless `update` removes the internal boundaries instead (`AddAllToResult`);
    /// `cells.add_all_to_result(0, false)?`.
    pub fn add_all_to_result(&mut self, material: i32, update: bool) -> Result<(), Failure> {
        self.shape_material.clear(false);
        self.materials.clear(false);
        self.map_modified.clear(false);
        self.builder.shape = self.all_parts.clone();
        if material != 0 {
            let parts = children(&self.all_parts);
            let list = self.materials.bound(material, List::default());
            for part in &parts {
                list.append(part.clone());
            }
            for part in parts {
                self.shape_material.bind(part, material);
            }
        }
        if update {
            return self.remove_internal_boundaries();
        }
        self.prepare_history()
    }

    /// Removes the parts of `take` and not of `avoid` from the result and from their
    /// material; the containers of the result keep their other parts (`RemoveFromResult`);
    /// `cells.remove_from_result(&take, &avoid)?`.
    pub fn remove_from_result(
        &mut self,
        take: &List<Shape>,
        avoid: &List<Shape>,
    ) -> Result<(), Failure> {
        let parts = self.find_parts(take, avoid)?;
        if parts.is_empty() {
            return Ok(());
        }
        let mut removed: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        for part in &parts {
            removed.add(part.clone());
            self.forget_material(part);
        }
        let (result, changed) = without_parts(&self.builder.shape, &removed)?;
        if changed {
            self.builder.shape = result;
            return self.prepare_history();
        }
        Ok(())
    }

    /// Removes a part from its material list and forgets its material.
    fn forget_material(&mut self, part: &Shape) {
        let Some(material) = self.shape_material.seek(part).copied() else {
            return;
        };
        if let Some(list) = self.materials.change_seek(&material)
            && let Some(position) = list.iter().position(|shape| shape.is_same(part))
        {
            *list = list
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != position)
                .map(|(_, s)| s.clone())
                .collect();
        }
        self.shape_material.unbind(part);
    }

    /// Groups the edges, faces and solids of the result into wires, shells and compsolids
    /// of connected parts (`MakeContainers`); `cells.make_containers()?`.
    pub fn make_containers(&mut self) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut result = Compound::default();
        builder.make_compound(&mut result);
        let mut result = result.into_shape();
        let mut by_dimension: [Vec<Shape>; 3] = Default::default();
        for part in children(&self.builder.shape) {
            let part_dimension = dimension(&part)?;
            if part_dimension <= 0 {
                builder.add(&mut result, &part)?;
            } else {
                by_dimension[part_dimension as usize - 1].push(part);
            }
        }
        for parts in by_dimension.iter().filter(|parts| !parts.is_empty()) {
            make_typed_containers(&compound_of(parts)?, &mut result)?;
        }
        self.builder.shape = result;
        Ok(())
    }
}

/// Adds `origin` to the origins of `part` (`myIndex`).
fn index_origin(index: &mut PartOrigins, part: &Shape, origin: &Shape) {
    match index.change_seek(part) {
        Some(origins) => {
            origins.append(origin.clone());
        }
        None => {
            index.add(part.clone(), [origin.clone()].into_iter().collect());
        }
    }
}

/// Adds the origins of a part to its sub-shape, each origin once.
fn inherit_origins(index: &mut PartOrigins, sub_shape: &Shape, origins: &List<Shape>) {
    let Some(known) = index.change_seek(sub_shape) else {
        index.add(sub_shape.clone(), origins.clone());
        return;
    };
    for origin in origins.iter() {
        if !known.iter().any(|shape| origin.is_same(shape)) {
            known.append(origin.clone());
        }
    }
}

/// Whether no origin of a part is avoided and every shape to take is an origin.
fn is_wanted(
    origins: &List<Shape>,
    take: &List<Shape>,
    avoided: &Map<Shape, ShapeMapHasher>,
) -> bool {
    let mut seen: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    for origin in origins.iter() {
        seen.add(origin.clone());
        if avoided.contains(origin) {
            return false;
        }
    }
    take.iter().all(|shape| seen.contains(shape))
}

/// The first shape of minimal dimension among `shapes`, with its dimension.
fn minimal_dimension_shape(shapes: &List<Shape>) -> Result<(Shape, i32), Failure> {
    let mut minimal = (Shape::default(), 10);
    for shape in shapes.iter() {
        let shape_dimension = dimension(shape)?;
        if shape_dimension < minimal.1 {
            minimal = (shape.clone(), shape_dimension);
        }
    }
    Ok(minimal)
}

/// The result without the removed parts; a container keeps its other parts as typed
/// containers. Also whether a part was removed.
fn without_parts(
    shape: &Shape,
    removed: &Map<Shape, ShapeMapHasher>,
) -> Result<(Shape, bool), Failure> {
    let builder = BRepBuilder::default();
    let mut result = Compound::default();
    builder.make_compound(&mut result);
    let mut result = result.into_shape();
    let mut changed = false;
    for child in children(shape) {
        if !is_container(&child)? {
            if removed.contains(&child) {
                changed = true;
            } else {
                builder.add(&mut result, &child)?;
            }
            continue;
        }
        let kept: Vec<Shape> = children(&child)
            .into_iter()
            .filter(|part| !removed.contains(part))
            .collect();
        changed |= kept.len() != children(&child).len();
        if !kept.is_empty() {
            make_typed_containers(&compound_of(&kept)?, &mut result)?;
        }
    }
    Ok((result, changed))
}

/// A compound of the shapes.
pub(super) fn compound_of(shapes: &[Shape]) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape)?;
    }
    Ok(compound.into_shape())
}

/// The shapes as an identity map.
fn shape_map(shapes: &List<Shape>) -> Map<Shape, ShapeMapHasher> {
    let mut map = Map::with_hasher(ShapeMapHasher, 1);
    for shape in shapes.iter() {
        map.add(shape.clone());
    }
    map
}

/// The sub-shapes of `kind` in explorer order.
pub(super) fn sub_shapes(
    shape: &Shape,
    kind: rustycad_tkg3d::top_abs::ShapeEnum,
) -> Result<Vec<Shape>, Failure> {
    let mut explorer = Explorer::new(shape, kind, rustycad_tkg3d::top_abs::ShapeEnum::Shape)?;
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}
