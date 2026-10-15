//! `BOPAlgo_BOP::BuildShape`, `CollectContainers`, `RemoveDuplicates` and
//! `NbCommonItemsInMap` (BOPAlgo_BOP.cxx): the result shape with rebuilt containers.
use super::BOPAlgoBOP;
use crate::bop_algo::Operation;
use crate::bop_ds::DataStructure;
use crate::bop_tools::{
    make_connexity_compounds, make_container, orient_edges_on_wire, orient_faces_on_shell,
    treat_compound,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::{Gravity, Report};
use rustycad_tkernel::ncollection::{IndexedDataMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

impl BOPAlgoBOP {
    /// Builds the result (`BuildShape`): open solids (`open_solids`, the answer of
    /// `CheckArgsForOpenSolid` for solid arguments) first try `BuildBOP`; then the splits of
    /// the result, solids for FUSE of solids, else the containers of the arguments rebuilt
    /// from their kept splits and the other kept splits; `bop.build_shape(&ds, false)?`.
    pub fn build_shape(
        &mut self,
        structure: &DataStructure,
        open_solids: bool,
    ) -> Result<(), Failure> {
        if open_solids {
            // Open solids cannot be split well by the solid builder: build the result from
            // the face splits (this loses the history of the solids).
            let report = Arc::new(Report::new());
            let (arguments, tools) = (
                self.provider.builder.arguments.clone(),
                self.provider.tools().clone(),
            );
            let operation = self.operation;
            let states = super::super::builder::operation_states(operation);
            self.provider.builder.build_bop_with(
                structure,
                &arguments,
                states.0,
                &tools,
                states.1,
                Some(&report),
            )?;
            if report.get_alerts(Gravity::Fail).is_empty() {
                self.builder().options().report().merge(&report);
                return Ok(());
            }
        }
        self.build_rc(structure)?;
        if self.operation == Operation::Fuse && self.dims[0] == 3 {
            return self.build_solid();
        }
        self.build_containers_result()
    }

    /// The containers of the arguments rebuilt from their kept splits, without duplicates,
    /// then the kept splits of the other shapes.
    fn build_containers_result(&mut self) -> Result<(), Failure> {
        let mut kept = MapOfShape::default();
        topexp::map_shapes_into_map(&self.rc, &mut kept, true, true);
        let groups = [
            self.provider.builder.arguments.clone(),
            self.provider.tools().clone(),
        ];
        let mut containers = Vec::new();
        for group in &groups {
            for shape in group.iter() {
                collect_containers(shape, &mut containers)?;
            }
        }
        let mut input_fence = MapOfShape::default();
        let mut rebuilt = Vec::new();
        for container in &containers {
            input_fence.add(container.clone());
            self.rebuild_container(container, &kept, &mut rebuilt)?;
        }
        remove_duplicates(&mut rebuilt)?;
        let builder = BRepBuilder::default();
        let mut result = Shape::default();
        make_container(ShapeEnum::Compound, &mut result);
        for container in &rebuilt {
            builder.add(&mut result, container)?;
        }
        let mut in_result = MapOfShape::default();
        topexp::map_shapes_into_map(&result, &mut in_result, true, true);
        let mut others = Vec::new();
        for group in &groups {
            for shape in group.iter() {
                treat_compound(shape, &mut others, Some(&mut input_fence))?;
            }
        }
        for shape in others {
            for kept_shape in self.kept_images(&shape, &kept) {
                if in_result.add(kept_shape.clone()) {
                    builder.add(&mut result, &kept_shape)?;
                }
            }
        }
        self.provider.builder.shape = result;
        Ok(())
    }

    /// The images of `shape` (or the shape) kept in the result splits.
    fn kept_images(&self, shape: &Shape, kept: &MapOfShape) -> Vec<Shape> {
        match self.builder().images().seek(shape) {
            Some(images) => images
                .iter()
                .filter(|image| kept.contains(image))
                .cloned()
                .collect(),
            None if kept.contains(shape) => vec![shape.clone()],
            None => Vec::new(),
        }
    }

    /// One container of the type of `container` per connected block of its kept splits,
    /// oriented as it.
    fn rebuild_container(
        &self,
        container: &Shape,
        kept: &MapOfShape,
        rebuilt: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut parts = Shape::default();
        make_container(ShapeEnum::Compound, &mut parts);
        for part in ShapeIterator::new(container, true, true) {
            for kept_part in self.kept_images(&part, kept) {
                builder.add(&mut parts, &kept_part)?;
            }
        }
        let kind = container.shape_type()?;
        let (connection, element) = match kind {
            ShapeEnum::Wire => (ShapeEnum::Vertex, ShapeEnum::Edge),
            ShapeEnum::Shell => (ShapeEnum::Edge, ShapeEnum::Face),
            _ => (ShapeEnum::Face, ShapeEnum::Solid),
        };
        let mut blocks = Vec::new();
        make_connexity_compounds(&parts, connection, element, &mut blocks)?;
        for block in blocks {
            let mut part_container = Shape::default();
            make_container(kind, &mut part_container);
            for part in ShapeIterator::new(&block, true, true) {
                builder.add(&mut part_container, &part)?;
            }
            match kind {
                ShapeEnum::Wire => orient_edges_on_wire(&mut part_container)?,
                ShapeEnum::Shell => orient_faces_on_shell(&mut part_container)?,
                _ => {}
            }
            part_container.set_orientation(container.orientation());
            rebuilt.push(part_container);
        }
        Ok(())
    }
}

/// The wires, shells and compsolids of a shape, through its compounds (`CollectContainers`).
pub(super) fn collect_containers(
    shape: &Shape,
    containers: &mut Vec<Shape>,
) -> Result<(), Failure> {
    match shape.shape_type()? {
        ShapeEnum::Wire | ShapeEnum::Shell | ShapeEnum::CompSolid => containers.push(shape.clone()),
        ShapeEnum::Compound => {
            for part in ShapeIterator::new(shape, true, true) {
                collect_containers(&part, containers)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Removes the wires, shells and compsolids whose contents are within another container
/// of the same type (`RemoveDuplicates`).
fn remove_duplicates(containers: &mut Vec<Shape>) -> Result<(), Failure> {
    for kind in [ShapeEnum::Wire, ShapeEnum::Shell, ShapeEnum::CompSolid] {
        remove_duplicates_of(containers, kind)?;
    }
    Ok(())
}

fn remove_duplicates_of(containers: &mut Vec<Shape>, kind: ShapeEnum) -> Result<(), Failure> {
    let mut contents: IndexedDataMap<Shape, MapOfShape> = IndexedDataMap::new(1);
    for container in containers.iter() {
        if container.shape_type()? != kind {
            continue;
        }
        let mut parts = MapOfShape::default();
        for part in ShapeIterator::new(container, true, true) {
            parts.add(part);
        }
        if !contents.contains(container) {
            contents.add(container.clone(), parts);
        }
    }
    if contents.is_empty() {
        return Ok(());
    }
    let duplicates = find_duplicates(&contents)?;
    if !duplicates.is_empty() {
        containers.retain(|container| !duplicates.contains(container));
    }
    Ok(())
}

/// The containers whose contents are all in a later (or earlier) container.
fn find_duplicates(
    contents: &IndexedDataMap<Shape, MapOfShape>,
) -> Result<Map<Shape, ShapeMapHasher>, Failure> {
    let mut duplicates: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    let count = contents.extent();
    for first in 1..=count {
        let first_container = contents.find_key(first)?;
        if duplicates.contains(first_container) {
            continue;
        }
        let first_parts = contents.find_from_index(first)?;
        for second in first + 1..=count {
            let second_container = contents.find_key(second)?;
            if duplicates.contains(second_container) {
                continue;
            }
            let second_parts = contents.find_from_index(second)?;
            let common = nb_common_items(first_parts, second_parts);
            if second_parts.extent() == common {
                duplicates.add(second_container.clone());
                continue;
            }
            if first_parts.extent() == common {
                duplicates.add(first_container.clone());
                break;
            }
        }
    }
    Ok(duplicates)
}

/// The number of items in both maps (`NbCommonItemsInMap`).
fn nb_common_items(first: &MapOfShape, second: &MapOfShape) -> usize {
    let (smaller, larger) = if second.extent() < first.extent() {
        (second, first)
    } else {
        (first, second)
    };
    smaller.iter().filter(|item| larger.contains(item)).count()
}
