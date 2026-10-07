//! `BOPAlgo_PaveFiller.cxx`: the intersection pipeline (`Perform`, `PerformInternal`) and
//! `RepeatIntersection`.
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::AlertIntersectionFailed;
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

/// One stage of the pipeline; it stops at the first stage that reports an error.
type Stage = fn(&mut BOPAlgoPaveFiller) -> Result<(), Failure>;

impl BOPAlgoPaveFiller {
    /// Runs the whole intersection; any failure becomes an `AlertIntersectionFailed` error, as
    /// the native catch of Standard_Failure does (`Perform`); `filler.perform();`.
    pub fn perform(&mut self) {
        if self.perform_internal().is_err() {
            let alert: AlertHandle = Arc::new(AlertIntersectionFailed);
            self.options.add_error(&alert);
        }
    }

    /// Initialization and all interference stages in native order (`PerformInternal`).
    /// `filler.perform_internal()?`.
    pub fn perform_internal(&mut self) -> Result<(), Failure> {
        self.initialize()?;
        if self.options.has_errors() {
            return Ok(());
        }
        let stages: [Stage; 21] = [
            Self::prepare,
            Self::perform_vv,
            Self::perform_ve,
            Self::update_pave_blocks_with_sd_vertices,
            Self::perform_ee,
            Self::update_pave_blocks_with_sd_vertices,
            Self::perform_vf,
            Self::update_pave_blocks_with_sd_vertices,
            Self::perform_ef,
            Self::update_after_ef,
            Self::repeat_intersection,
            Self::force_interf_ee,
            Self::force_interf_ef,
            Self::perform_ff,
            Self::update_after_ff,
            Self::make_split_edges,
            Self::update_pave_blocks_with_sd_vertices,
            Self::make_blocks,
            Self::finish_blocks,
            Self::make_pcurves,
            Self::process_de,
        ];
        for stage in stages {
            stage(self)?;
            if self.options.has_errors() {
                return Ok(());
            }
        }
        Ok(())
    }

    fn update_after_ef(&mut self) -> Result<(), Failure> {
        self.update_pave_blocks_with_sd_vertices()?;
        self.update_interfs_with_sd_vertices()
    }

    fn update_after_ff(&mut self) -> Result<(), Failure> {
        self.update_blocks_with_shared_vertices()?;
        self.structure_mut()?.refine_face_info_in()
    }

    /// The native steps between MakeBlocks and MakePCurves, which cannot fail by error.
    fn finish_blocks(&mut self) -> Result<(), Failure> {
        self.check_self_interference()?;
        self.update_interfs_with_sd_vertices()?;
        let structure = self.structure_mut()?;
        structure.release_pave_blocks()?;
        structure.refine_face_info_on()?;
        self.remove_micro_edges()
    }

    /// Repeats the vertex intersections for the vertices whose tolerance grew, or whose new
    /// same-domain vertex grew, during the operation (`RepeatIntersection`).
    pub fn repeat_intersection(&mut self) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut extra: Map<i32> = Map::new(1);
        for index in 0..structure.nb_source_shapes() {
            if structure.shape_info(index)?.shape_type() != ShapeEnum::Vertex {
                continue;
            }
            if self.increased_ss.contains(&index) {
                extra.add(index);
                continue;
            }
            let mut representative = index;
            if structure.has_shape_sd_into(index, &mut representative)
                && self.increased_ss.contains(&representative)
            {
                extra.add(index);
            }
        }
        if extra.extent() == 0 {
            return Ok(());
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let iterator = self.iterator.as_mut().ok_or_else(missing_structure)?;
        iterator.intersect_extended(structure, &extra)?;
        let stages: [Stage; 6] = [
            Self::perform_vv,
            Self::update_pave_blocks_with_sd_vertices,
            Self::perform_ve,
            Self::update_pave_blocks_with_sd_vertices,
            Self::perform_vf,
            Self::update_after_ef,
        ];
        for stage in stages {
            stage(self)?;
            if self.options.has_errors() {
                return Ok(());
            }
        }
        Ok(())
    }
}
