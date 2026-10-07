//! `BOPAlgo_CellsBuilder::RemoveInternalBoundaries`: the parts of one material merged into
//! one shape, and the typed containers of the result.
use super::parts::{compound_of, sub_shapes};
use super::unify::UnifyQueue;
use super::{BOPAlgoCellsBuilder, children};
use crate::bop_algo::{
    AlertRemovalOfIBForMDimShapes, AlertRemovalOfIBForSolidsFailed, BuilderSolid,
};
use crate::bop_tools::{dimension, make_connexity_blocks, make_container, orient_faces_on_shell};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::sync::Arc;

impl BOPAlgoCellsBuilder {
    /// Merges the parts of each material: the solids of a connected block into one solid
    /// without their shared faces, then the edges and the faces of all materials, each
    /// type unified at once keeping the material boundaries; parts of several dimensions
    /// only warn. The parts without material keep their place, and the history is refilled
    /// when a merge happened (`RemoveInternalBoundaries`);
    /// `cells.remove_internal_boundaries()?`.
    pub fn remove_internal_boundaries(&mut self) -> Result<(), Failure> {
        if self.materials.is_empty() {
            return Ok(());
        }
        let mut result = new_compound();
        let mut changed = false;
        let mut unify = UnifyQueue::default();
        let materials: Vec<i32> = self
            .materials
            .items()
            .map(|(material, _)| *material)
            .collect();
        for material in materials {
            changed |= self.merge_material(material, &mut result, &mut unify)?;
        }
        changed |= self.unify_queued(unify, &mut result)?;
        if !changed {
            return Ok(());
        }
        self.add_parts_without_material(&mut result)?;
        self.builder.shape = result;
        self.prepare_history()
    }

    /// Adds the parts of one material to the result, merged when they are solids, or
    /// queues them when they are several edges or faces; true when solids were merged.
    fn merge_material(
        &mut self,
        material: i32,
        result: &mut Shape,
        unify: &mut UnifyQueue,
    ) -> Result<bool, Failure> {
        let parts: Vec<Shape> = match self.materials.seek(&material) {
            Some(parts) if !parts.is_empty() => parts.iter().cloned().collect(),
            _ => return Ok(false),
        };
        let builder = BRepBuilder::default();
        if parts.len() == 1 && !is_container(&parts[0])? {
            builder.add(result, &parts[0])?;
            return Ok(false);
        }
        let kind = parts[0].shape_type()?;
        let mut changed = false;
        let mut merged = parts.clone();
        if let Some(other) = parts
            .iter()
            .find(|part| part.shape_type().is_ok_and(|other| other != kind))
        {
            let shapes = compound_of(&[parts[0].clone(), other.clone()])?;
            let alert: AlertHandle = Arc::new(AlertRemovalOfIBForMDimShapes::new(&shapes));
            self.builder.options().add_warning(&alert);
        } else if matches!(kind, ShapeEnum::Edge | ShapeEnum::Face) {
            // Unified later with the edges or faces of the other materials.
            unify.queue(kind, &parts)?;
            return Ok(false);
        } else if kind == ShapeEnum::Solid {
            let (solids, removed) = self.remove_internal_faces(&parts)?;
            if removed {
                changed = true;
                for solid in &solids {
                    self.shape_material.bind(solid.clone(), material);
                }
                if let Some(list) = self.materials.change_seek(&material) {
                    *list = solids.iter().cloned().collect();
                }
                merged = solids;
            }
        }
        for part in &merged {
            builder.add(result, part)?;
        }
        Ok(changed)
    }

    /// Each connected block of solids made one solid of its unshared faces, holding the
    /// internal parts of the block (`RemoveInternals` for solids); also whether a block
    /// was merged.
    fn remove_internal_faces(&mut self, solids: &[Shape]) -> Result<(Vec<Shape>, bool), Failure> {
        if solids.len() < 2 {
            return Ok((solids.to_vec(), false));
        }
        let mut blocks = Vec::new();
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        make_connexity_blocks(
            &compound_of(solids)?,
            ShapeEnum::Face,
            ShapeEnum::Solid,
            &mut blocks,
            &mut ancestors,
        )?;
        let mut merged = Vec::new();
        let mut removed = false;
        for block in blocks {
            match self.merge_block(&block)? {
                Some(solid) => {
                    for old in &block {
                        self.map_modified.bind(old.clone(), solid.clone());
                    }
                    merged.push(solid);
                    removed = true;
                }
                None => merged.extend(block),
            }
        }
        Ok((merged, removed))
    }

    /// One solid from the faces of the block that belong to one solid only, with the
    /// internal parts; None when no face is shared or the solid builder fails (the failure
    /// warns).
    fn merge_block(&mut self, block: &[Shape]) -> Result<Option<Shape>, Failure> {
        let (faces, internals) = block_faces(block)?;
        let unique: Vec<Shape> = (1..=faces.extent())
            .filter(|&position| {
                faces
                    .find_from_index(position)
                    .is_ok_and(|solids| solids.extent() == 1)
            })
            .map(|position| faces.find_key(position).cloned())
            .collect::<Result<_, _>>()?;
        if unique.len() == faces.extent() {
            return Ok(None);
        }
        let mut builder = BuilderSolid::new();
        builder.set_shapes(&unique.iter().cloned().collect());
        builder.perform()?;
        if builder.options().has_errors() || builder.areas().extent() != 1 {
            let alert: AlertHandle =
                Arc::new(AlertRemovalOfIBForSolidsFailed::new(&compound_of(&unique)?));
            self.builder.options().add_warning(&alert);
            return Ok(None);
        }
        self.builder
            .options()
            .report()
            .merge(builder.options().report());
        let mut solid = builder.areas().first().cloned().ok_or_else(missing_area)?;
        solid.set_free(true)?;
        for internal in &internals {
            BRepBuilder::default().add(&mut solid, internal)?;
        }
        solid.set_free(false)?;
        Ok(Some(solid))
    }

    /// Adds the parts of the result without material; a container keeps its parts without
    /// material, rebuilt as typed containers when some had one.
    fn add_parts_without_material(&self, result: &mut Shape) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for part in children(&self.builder.shape) {
            if self.shape_material.is_bound(&part) {
                continue;
            }
            if !is_container(&part)? {
                builder.add(result, &part)?;
                continue;
            }
            let inner = children(&part);
            let kept: Vec<Shape> = inner
                .iter()
                .filter(|shape| !self.shape_material.is_bound(shape))
                .cloned()
                .collect();
            if kept.is_empty() {
                continue;
            }
            if kept.len() == inner.len() {
                builder.add(result, &part)?;
            } else {
                make_typed_containers(&compound_of(&kept)?, result)?;
            }
        }
        Ok(())
    }
}

/// The faces of a block of solids with the solids holding each.
type FaceSolids = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The faces of the solids of a block with the solids holding each, and the internal parts
/// of the solids.
fn block_faces(block: &[Shape]) -> Result<(FaceSolids, Vec<Shape>), Failure> {
    let mut faces: FaceSolids = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
    let mut internals = Vec::new();
    for solid in block {
        for part in children(solid) {
            if part.orientation() == Orientation::Internal {
                internals.push(part);
                continue;
            }
            for face in children(&part) {
                match faces.change_seek(&face) {
                    Some(solids) => {
                        solids.append(solid.clone());
                    }
                    None => {
                        faces.add(face, [solid.clone()].into_iter().collect());
                    }
                }
            }
        }
    }
    Ok((faces, internals))
}

/// The failure of a solid builder that reported one area without giving it.
fn missing_area() -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        "BOPAlgo_CellsBuilder: the solid builder has no area; expected one",
        "",
    )
}

/// A new empty compound.
fn new_compound() -> Shape {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    compound.into_shape()
}

/// Whether a shape is a wire, shell or compsolid.
pub(super) fn is_container(shape: &Shape) -> Result<bool, Failure> {
    Ok(matches!(
        shape.shape_type()?,
        ShapeEnum::Wire | ShapeEnum::Shell | ShapeEnum::CompSolid
    ))
}

/// The type of the parts of dimension `dimension` (`TypeToExplore`).
pub(super) fn type_to_explore(dimension: i32) -> ShapeEnum {
    match dimension {
        0 => ShapeEnum::Vertex,
        1 => ShapeEnum::Edge,
        2 => ShapeEnum::Face,
        3 => ShapeEnum::Solid,
        _ => ShapeEnum::Shape,
    }
}

/// Adds to `result` one wire, shell or compsolid per connected block of the edges, faces or
/// solids of `parts`, the shells with consistently oriented faces (`MakeTypedContainers`).
pub(super) fn make_typed_containers(parts: &Shape, result: &mut Shape) -> Result<(), Failure> {
    let part_type = type_to_explore(dimension(parts)?);
    let (container_type, connexity_type) = match part_type {
        ShapeEnum::Edge => (ShapeEnum::Wire, ShapeEnum::Vertex),
        ShapeEnum::Face => (ShapeEnum::Shell, ShapeEnum::Edge),
        ShapeEnum::Solid => (ShapeEnum::CompSolid, ShapeEnum::Face),
        _ => return Ok(()),
    };
    let mut blocks = Vec::new();
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    make_connexity_blocks(
        parts,
        connexity_type,
        part_type,
        &mut blocks,
        &mut ancestors,
    )?;
    let builder = BRepBuilder::default();
    for block in blocks {
        let mut container = Shape::default();
        make_container(container_type, &mut container);
        for element in &block {
            for part in sub_shapes(element, part_type)? {
                builder.add(&mut container, &part)?;
            }
        }
        if container_type == ShapeEnum::Shell {
            orient_faces_on_shell(&mut container)?;
        }
        builder.add(result, &container)?;
    }
    Ok(())
}
