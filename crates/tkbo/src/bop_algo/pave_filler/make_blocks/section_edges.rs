//! `BOPAlgo_PaveFiller_6.cxx`: the section edges of one curve (the "Make section edges"
//! loop of MakeBlocks) and `RemoveMicroSectionEdges`.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::existing_blocks::add_block_face;
use super::{BlockTree, CurveRef, MakeBlocksState, SectionShapes};
use crate::bop_ds::{
    CoupleOfPaveBlocks, PaveBlock, PaveBlockHandle, PaveBlockIndexMap, pave_block_handle,
};
use crate::bop_tools::{is_micro_edge, make_edge, make_pcurve};
use crate::int_tools::Curve;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_lib::{RangeEnd, find_valid_range};
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Makes the section edges of one curve from the splits of its first pave block;
    /// returns true when the curve had at least one split (clears the recheck flag).
    pub(super) fn make_section_edges(
        &mut self,
        curve: CurveRef,
        shapes: &[Face; 2],
        faces: (i32, i32),
        tree: &mut BlockTree,
        state: &mut MakeBlocksState,
    ) -> Result<bool, Failure> {
        let (section, tolerance, first) = {
            let stored = self.ff_curve_mut(curve)?;
            let tolerance = stored.tolerance().max(stored.tangential_tolerance());
            (
                stored.curve().clone(),
                tolerance,
                Arc::clone(stored.change_pave_block1()?),
            )
        };
        let mut splits: Vec<PaveBlock> = Vec::new();
        first
            .lock()
            .expect("pave-block lock")
            .update(&mut splits, false);
        let had_splits = !splits.is_empty();
        for block in splits.into_iter().map(pave_block_handle) {
            self.make_section_edge(
                curve, &block, &section, tolerance, shapes, faces, tree, state,
            )?;
        }
        let blocks = self.ff_curve_mut(curve)?.change_pave_blocks();
        if !blocks.is_empty() {
            blocks.remove(0);
        }
        Ok(had_splits)
    }

    #[allow(clippy::too_many_arguments)]
    fn make_section_edge(
        &mut self,
        curve: CurveRef,
        block: &PaveBlockHandle,
        section: &Curve,
        tolerance: f64,
        shapes: &[Face; 2],
        faces: (i32, i32),
        tree: &mut BlockTree,
        state: &mut MakeBlocksState,
    ) -> Result<(), Failure> {
        let (bounds, range) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.indices(), locked.range())
        };
        if (range.0 - range.1).abs() < precision::p_confusion() {
            return Ok(());
        }
        let context = self.context_ref()?;
        if !context.is_valid_block_for_faces(
            range.0, range.1, section, &shapes[0], &shapes[1], tolerance,
        )? {
            return Ok(());
        }
        if let Some((edge, new_tolerance)) =
            self.existing_shared_edge(block, section, &state.shared_edges)?
        {
            self.update_edge_tolerance(edge, new_tolerance)?;
            self.update_saved_tolerance(edge, new_tolerance, &mut state.tolerances)?;
            return Ok(());
        }
        let vertices = [self.vertex_shape(bounds.0)?, self.vertex_shape(bounds.1)?];
        if !self.has_valid_range(section, tolerance, range, &vertices)? {
            // Covered by the vertex spheres: the vertices are fused in the post treatment.
            if !state.vertices_bounds.contains(&bounds.0)
                && !state.vertices_bounds.contains(&bounds.1)
            {
                state.micro_blocks.add(Arc::clone(block));
                state
                    .sections
                    .vertex_indices
                    .bind(vertices[0].clone().into_shape(), bounds.0);
                state
                    .sections
                    .vertex_indices
                    .bind(vertices[1].clone().into_shape(), bounds.1);
            }
            return Ok(());
        }
        let existing = self.existing_on_in_block(
            block,
            section,
            tolerance,
            &state.shared.pave_blocks_on_in,
            tree,
            &state.shared.common_pave_blocks,
        )?;
        if let Some((existing, new_tolerance)) = existing {
            return self.reuse_existing_block(
                curve,
                faces,
                bounds,
                &vertices,
                &existing,
                new_tolerance,
                state,
            );
        }
        self.append_new_section_edge(
            curve, block, section, tolerance, shapes, faces, tree, state, bounds, &vertices,
        )
    }

    /// `BRepLib::FindValidRange` of the block on the section, with the vertex tolerances.
    fn has_valid_range(
        &self,
        section: &Curve,
        tolerance: f64,
        range: (f64, f64),
        vertices: &[Vertex; 2],
    ) -> Result<bool, Failure> {
        let adaptor = GeomAdaptorCurve::with_curve(section.curve())?;
        let ends = [
            RangeEnd {
                parameter: range.0,
                point: BRepTool::pnt(&vertices[0])?,
                tolerance: tolerance.max(BRepTool::vertex_tolerance(&vertices[0])?),
            },
            RangeEnd {
                parameter: range.1,
                point: BRepTool::pnt(&vertices[1])?,
                tolerance: tolerance.max(BRepTool::vertex_tolerance(&vertices[1])?),
            },
        ];
        Ok(find_valid_range(&adaptor, tolerance, ends[0], ends[1])?.is_valid)
    }

    /// The section block coincides with an existing On/In block: the existing edge is made to
    /// touch the face missing it and joins the post treatment.
    #[allow(clippy::too_many_arguments)]
    fn reuse_existing_block(
        &mut self,
        curve: CurveRef,
        faces: (i32, i32),
        bounds: (i32, i32),
        vertices: &[Vertex; 2],
        existing: &PaveBlockHandle,
        new_tolerance: f64,
        state: &mut MakeBlocksState,
    ) -> Result<(), Failure> {
        let (in_first, in_second) = self.block_in_faces(existing, faces)?;
        if in_first && in_second {
            return Ok(());
        }
        let (edge, existing_bounds) = {
            let locked = existing.lock().expect("pave-block lock");
            (locked.edge(), locked.indices())
        };
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let edge_tolerance =
            BRepTool::edge_tolerance(&Edge::unchecked(structure.shape(edge)?.clone()))?;
        let curve_tolerance = self.ff_curve(curve)?.tolerance();
        // The real tolerance of the intersection is the minimum.
        let tolerance = if new_tolerance < curve_tolerance {
            curve_tolerance
        } else {
            new_tolerance
        };
        if tolerance > edge_tolerance {
            self.update_edge_tolerance(edge, tolerance)?;
            self.update_saved_tolerance(edge, tolerance, &mut state.tolerances)?;
        }
        let face = if in_first { faces.1 } else { faces.0 };
        add_block_face(&mut state.block_faces, existing, face);
        // Section vertices off the existing block are fused with it later (technological
        // bound vertices are removed instead).
        for (position, vertex) in [bounds.0, bounds.1].into_iter().enumerate() {
            if vertex != existing_bounds.0
                && vertex != existing_bounds.1
                && !state.vertices_bounds.contains(&vertex)
            {
                state
                    .vertices_on_rejected
                    .add(vertices[position].clone().into_shape());
            }
        }
        if state.added.add(Arc::clone(existing)) {
            self.prepare_post_treat_ff(curve, existing, &mut state.sections)?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn append_new_section_edge(
        &mut self,
        curve: CurveRef,
        block: &PaveBlockHandle,
        section: &Curve,
        tolerance: f64,
        shapes: &[Face; 2],
        faces: (i32, i32),
        tree: &mut BlockTree,
        state: &mut MakeBlocksState,
        bounds: (i32, i32),
        vertices: &[Vertex; 2],
    ) -> Result<(), Failure> {
        let range = block.lock().expect("pave-block lock").range();
        let mut edge = Edge::default();
        make_edge(
            section,
            &vertices[0],
            range.0,
            &vertices[1],
            range.1,
            tolerance,
            &mut edge,
        )?;
        let attribute = self.section_attribute;
        let context = self.context.clone();
        make_pcurve(
            &edge,
            &shapes[0],
            &shapes[1],
            section,
            attribute.pcurve_on_s1(),
            attribute.pcurve_on_s2(),
            context.get(),
        )?;
        self.ff_curve_mut(curve)?
            .change_pave_blocks()
            .push(Arc::clone(block));
        let mut couple = CoupleOfPaveBlocks::new();
        couple.set_interference_index(curve.0 as i32);
        couple.set_index(curve.1 as i32);
        couple.set_pave_block1(Some(Arc::clone(block)));
        state
            .sections
            .couples
            .add(edge.clone().into_shape(), couple);
        state
            .sections
            .vertex_indices
            .bind(vertices[0].clone().into_shape(), bounds.0);
        state
            .sections
            .vertex_indices
            .bind(vertices[1].clone().into_shape(), bounds.1);
        state.tolerances.unbind(&bounds.0);
        state.tolerances.unbind(&bounds.1);
        self.process_existing_blocks_near_edge(
            curve,
            faces,
            &edge,
            &state.shared.pave_blocks_on_in,
            tree,
            &mut state.sections,
            &mut state.block_faces,
            &mut state.added,
        )
    }

    /// Drops the micro section edges from the post treatment and their curve, keeping their
    /// blocks for the fusion of their vertices (`RemoveMicroSectionEdges`).
    pub(super) fn remove_micro_section_edges(
        &mut self,
        sections: &mut SectionShapes,
        micro_blocks: &mut PaveBlockIndexMap,
    ) -> Result<(), Failure> {
        if sections.couples.is_empty() {
            return Ok(());
        }
        let mut kept: IndexedDataMap<_, CoupleOfPaveBlocks, _> =
            IndexedDataMap::with_hasher(rustycad_tkbrep::toptools::ShapeMapHasher, 1);
        for index in 1..=sections.couples.extent() {
            let shape = sections.couples.find_key(index)?.clone();
            let couple = sections.couples.find_from_index(index)?.clone();
            if !self.is_micro_section(&shape, &couple)? {
                kept.add(shape, couple);
                continue;
            }
            let block = couple
                .pave_block1()
                .cloned()
                .expect("section edges carry a pave block");
            let curve = (
                couple.interference_index() as usize,
                couple.index() as usize,
            );
            let blocks = self.ff_curve_mut(curve)?.change_pave_blocks();
            if let Some(position) = blocks.iter().position(|item| Arc::ptr_eq(item, &block)) {
                blocks.remove(position);
            }
            micro_blocks.add(block);
        }
        if kept.extent() != sections.couples.extent() {
            sections.couples = kept;
        }
        Ok(())
    }

    fn is_micro_section(
        &self,
        shape: &rustycad_tkbrep::topods::Shape,
        couple: &CoupleOfPaveBlocks,
    ) -> Result<bool, Failure> {
        if shape.shape_type()? != ShapeEnum::Edge {
            return Ok(false);
        }
        let Some(block) = couple.pave_block1() else {
            return Ok(false);
        };
        if block.lock().expect("pave-block lock").has_edge() {
            return Ok(false);
        }
        is_micro_edge(&Edge::unchecked(shape.clone()), &self.context, false)
    }
}
