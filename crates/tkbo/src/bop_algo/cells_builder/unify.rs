//! `BOPAlgo_CellsBuilder::RemoveInternals` for edges and faces: the edges or faces of all
//! materials unified by ShapeUpgrade_UnifySameDomain, keeping the material boundaries.
use super::BOPAlgoCellsBuilder;
use super::parts::sub_shapes;
use crate::bop_algo::{AlertRemovalOfIBForEdgesFailed, AlertRemovalOfIBForFacesFailed};
use crate::bop_tools::make_container;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use std::sync::Arc;

/// The edges (slot 0) and faces (slot 1) of the materials to unify after the loop over the
/// materials, with the boundaries of each material to keep (`aLSUnify`, `aKeepMap`).
#[derive(Default)]
pub(super) struct UnifyQueue {
    parts: [Vec<Shape>; 2],
    keep: [MapOfShape; 2],
}

impl UnifyQueue {
    /// Queues the edges or faces of one material and keeps its boundary vertices or edges
    /// (`CollectMaterialBoundaries`).
    pub(super) fn queue(&mut self, kind: ShapeEnum, parts: &[Shape]) -> Result<(), Failure> {
        let slot = usize::from(kind == ShapeEnum::Face);
        let boundary_kind = if slot == 1 {
            ShapeEnum::Edge
        } else {
            ShapeEnum::Vertex
        };
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        for part in parts {
            topexp::map_shapes_and_ancestors(part, boundary_kind, kind, &mut ancestors)?;
        }
        for position in 1..=ancestors.extent() {
            // A sub-shape of one part only lies on the boundary of the material.
            if ancestors.find_from_index(position)?.extent() == 1 {
                self.keep[slot].add(ancestors.find_key(position)?.clone());
            }
        }
        self.parts[slot].extend(parts.iter().cloned());
        Ok(())
    }
}

impl BOPAlgoCellsBuilder {
    /// Unifies the queued edges, then faces, adding the unified shapes to the result; true
    /// when a shape was unified.
    pub(super) fn unify_queued(
        &mut self,
        queue: UnifyQueue,
        result: &mut Shape,
    ) -> Result<bool, Failure> {
        let mut changed = false;
        let kinds = [ShapeEnum::Edge, ShapeEnum::Face];
        for ((parts, keep), kind) in queue.parts.into_iter().zip(queue.keep).zip(kinds) {
            if parts.is_empty() {
                continue;
            }
            let (unified, removed) = self.unify_parts(&parts, &keep, kind)?;
            changed |= removed;
            for shape in &unified {
                BRepBuilder::default().add(result, shape)?;
            }
        }
        Ok(changed)
    }

    /// The parts unified in a wire or shell keeping `keep`; the unified parts take the
    /// material of their origins. A unification without result warns and keeps the parts
    /// (`RemoveInternals` for edges and faces).
    fn unify_parts(
        &mut self,
        parts: &[Shape],
        keep: &MapOfShape,
        kind: ShapeEnum,
    ) -> Result<(Vec<Shape>, bool), Failure> {
        if parts.len() < 2 {
            return Ok((parts.to_vec(), false));
        }
        let faces = kind == ShapeEnum::Face;
        let mut container = Shape::default();
        make_container(
            if faces {
                ShapeEnum::Shell
            } else {
                ShapeEnum::Wire
            },
            &mut container,
        );
        for part in parts {
            BRepBuilder::default().add(&mut container, part)?;
        }
        let unifier = ShapeUpgradeUnifySameDomain::new(&container, !faces, faces, false);
        unifier.keep_shapes(keep)?;
        unifier.build()?;
        let unified = sub_shapes(&unifier.shape(), kind)?;
        if unified.is_empty() {
            self.warn_unification_failed(&container, faces);
            return Ok((parts.to_vec(), false));
        }
        let mut container_shapes = IndexedMapOfShape::default();
        for sub_kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
            topexp::map_shapes_of_type(&container, sub_kind, &mut container_shapes)?;
        }
        let removed = self.record_unification(&container_shapes, &unifier)?;
        Ok((unified, removed))
    }

    /// Each sub-shape of the container is modified into its unified images, which take its
    /// material; true when a sub-shape was modified.
    fn record_unification(
        &mut self,
        sub_shapes: &IndexedMapOfShape,
        unifier: &ShapeUpgradeUnifySameDomain,
    ) -> Result<bool, Failure> {
        let history = unifier.history();
        let Some(history) = history.get() else {
            return Ok(false);
        };
        let mut removed = false;
        for sub_shape in sub_shapes.iter() {
            let material = self.shape_material.seek(sub_shape).copied();
            for image in history.modified(sub_shape)? {
                self.map_modified.bind(sub_shape.clone(), image.clone());
                removed = true;
                if let Some(material) = material
                    && !self.shape_material.is_bound(&image)
                {
                    self.shape_material.bind(image, material);
                }
            }
        }
        Ok(removed)
    }

    /// `AlertRemovalOfIBForFacesFailed` or `AlertRemovalOfIBForEdgesFailed` with the
    /// container that could not be unified.
    fn warn_unification_failed(&self, container: &Shape, faces: bool) {
        let alert: AlertHandle = match faces {
            true => Arc::new(AlertRemovalOfIBForFacesFailed::new(container)),
            false => Arc::new(AlertRemovalOfIBForEdgesFailed::new(container)),
        };
        self.builder.options().add_warning(&alert);
    }
}
