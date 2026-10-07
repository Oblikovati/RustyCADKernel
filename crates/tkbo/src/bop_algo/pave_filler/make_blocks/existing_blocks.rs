//! `BOPAlgo_PaveFiller_6.cxx`: matching section pave blocks with existing edges.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::{BlockFaces, BlockTree, CurveRef, SectionShapes};
use crate::bop_ds::{CoupleOfPaveBlocks, PaveBlockHandle, PaveBlockIndexMap};
use crate::bop_tools::BoxSelector;
use crate::int_tools::{Curve, Tools};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};
use rustycad_tkmath::bvh::BvhSetTraverse;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Point3};
use std::sync::Arc;

/// The section pave block's points and tolerances used against each candidate.
pub(super) struct SectionProbe {
    bounds: (i32, i32),
    first: Point3,
    middle: Point3,
    last: Point3,
    tangent: Option<rustycad_tkmath::gp::Vector3>,
    last_box: BoundingBox3,
    vertex_tolerance: f64,
    check: f64,
    max_additional: f64,
    is_line: bool,
}

impl BOPAlgoPaveFiller {
    /// Finds a shared edge of both faces passing through the middle of the pave block
    /// (`IsExistingPaveBlock` with the shared edges); returns its index and distance.
    pub(super) fn existing_shared_edge(
        &self,
        block: &PaveBlockHandle,
        section: &Curve,
        shared_edges: &[i32],
    ) -> Result<Option<(i32, f64)>, Failure> {
        if shared_edges.is_empty() {
            return Ok(None);
        }
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let (range, bounds) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.range(), locked.indices())
        };
        let tolerance = BRepTool::vertex_tolerance(&self.vertex_shape(bounds.0)?)?
            .max(BRepTool::vertex_tolerance(&self.vertex_shape(bounds.1)?)?);
        let mut middle = Point3::default();
        section.d0(Tools::intermediate_point(range.0, range.1), &mut middle)?;
        let mut middle_box = BoundingBox3::default();
        middle_box.add_point(middle);
        middle_box.enlarge(tolerance);
        for edge in shared_edges.iter().copied() {
            if edge < 0 {
                continue;
            }
            let info = structure.shape_info(edge)?;
            if info.box_bounds().is_out_box(&middle_box) {
                continue;
            }
            let shape = Edge::unchecked(info.shape().clone());
            let check =
                BRepTool::edge_tolerance(&shape)?.max(tolerance) + self.options.fuzzy_value();
            let (mut parameter, mut distance) = (0., 0.);
            if self.context_ref()?.compute_pe(
                middle,
                check,
                &shape,
                &mut parameter,
                &mut distance,
            )? == 0
            {
                return Ok(Some((edge, distance)));
            }
        }
        Ok(None)
    }

    /// Finds the existing On/In pave block closest to the section block, also accepting a
    /// thin coincident edge (`IsExistingPaveBlock` with the box tree); returns it and its
    /// distance.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn existing_on_in_block(
        &self,
        block: &PaveBlockHandle,
        section: &Curve,
        tolerance: f64,
        on_in: &PaveBlockIndexMap,
        tree: &mut BlockTree,
        common: &PaveBlockIndexMap,
    ) -> Result<Option<(PaveBlockHandle, f64)>, Failure> {
        let probe = self.section_probe(block, section, tolerance)?;
        let mut first_box = BoundingBox3::default();
        first_box.add_point(probe.first);
        first_box.enlarge(BRepTool::vertex_tolerance(
            &self.vertex_shape(probe.bounds.0)?,
        )?);
        let mut selector = BoxSelector::default();
        selector.set_box(box3_to_bvh(&first_box)?);
        selector.set_bvh_set(Some(tree));
        if selector.select()? == 0 {
            return Ok(None);
        }
        let candidates: Vec<i32> = selector.indices().iter().copied().collect();
        let mut best: Option<(PaveBlockHandle, f64)> = None;
        let mut best_distance = f64::MAX;
        for index in candidates {
            let candidate = Arc::clone(on_in.find_key(index as usize)?);
            if let Some((distance, coefficient)) =
                self.candidate_distance(&probe, &candidate, common)?
                && distance < best_distance
            {
                best_distance = coefficient * distance;
                best = Some((candidate, best_distance));
            }
        }
        Ok(best)
    }

    pub(super) fn section_probe(
        &self,
        block: &PaveBlockHandle,
        section: &Curve,
        tolerance: f64,
    ) -> Result<SectionProbe, Failure> {
        let (range, bounds) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.range(), locked.indices())
        };
        let (mut first, mut last) = (Point3::default(), Point3::default());
        section.d0(range.0, &mut first)?;
        section.d0(range.1, &mut last)?;
        let derivative = section
            .curve()
            .eval_d1(Tools::intermediate_point(range.0, range.1))?;
        let (middle, mut tangent) = (derivative.point, derivative.d1);
        let tangent = if tangent.square_magnitude() > gp::resolution() {
            tangent.normalize()?;
            Some(tangent)
        } else {
            None
        };
        let first_tolerance = BRepTool::vertex_tolerance(&self.vertex_shape(bounds.0)?)?;
        let last_tolerance = BRepTool::vertex_tolerance(&self.vertex_shape(bounds.1)?)?;
        let mut last_box = BoundingBox3::default();
        last_box.add_point(last);
        last_box.enlarge(last_tolerance);
        let fuzzy = self.options.fuzzy_value();
        let check = tolerance + fuzzy;
        // Limits defining a "thin" face when both ends are shared and no common block exists.
        let max_additional = 0.001_f64.min(10. * check);
        Ok(SectionProbe {
            bounds,
            first,
            middle,
            last,
            tangent,
            last_box,
            vertex_tolerance: first_tolerance.max(last_tolerance) + fuzzy,
            check,
            max_additional,
            is_line: section.curve_type()? == CurveType::Line,
        })
    }

    /// The distance of a candidate block to the section block and the coefficient applied to
    /// it, or `None` when the candidate does not match.
    pub(super) fn candidate_distance(
        &self,
        probe: &SectionProbe,
        candidate: &PaveBlockHandle,
        common: &PaveBlockIndexMap,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let context = self.context_ref()?;
        let ((v21, v22), edge) = {
            let locked = candidate.lock().expect("pave-block lock");
            (locked.indices(), locked.edge())
        };
        let candidate_tolerance = BRepTool::vertex_tolerance(&self.vertex_shape(v21)?)?
            .max(BRepTool::vertex_tolerance(&self.vertex_shape(v22)?)?)
            + self.options.fuzzy_value();
        let info = structure.shape_info(edge)?;
        let shape = Edge::unchecked(info.shape().clone());
        let (v11, v12) = probe.bounds;
        let mut flag1 = if v11 == v21 || v11 == v22 { 2 } else { 1 };
        let mut flag2 = if v12 == v21 || v12 == v22 {
            2
        } else if !info.box_bounds().is_out_box(&probe.last_box) {
            1
        } else {
            0
        };
        if flag2 == 0 {
            return Ok(None);
        }
        let mut coefficient = 1.;
        let mut middle_distance = 0.;
        let mut status = 1;
        let mut real_tolerance = probe.check;
        if structure.is_common_block(candidate) {
            real_tolerance = real_tolerance.max(probe.vertex_tolerance.max(candidate_tolerance));
            if common.contains(candidate) {
                // An edge in a common block with a face gets more chance to match.
                real_tolerance *= 2.;
            }
        } else if flag1 == 2 && flag2 == 2 {
            let skip = (v11 == v12 && v21 != v22) || (v11 != v12 && v21 == v22);
            if !skip && let Some(tangent) = probe.tangent {
                let adaptor = BRepAdaptorCurve::with_edge(&shape)?;
                if !probe.is_line || adaptor.get_type()? != CurveType::Line {
                    let additional = 2.
                        * probe.max_additional.min(
                            real_tolerance.max(probe.vertex_tolerance.max(candidate_tolerance)),
                        );
                    let mut on_edge = 0.;
                    status = context.compute_pe(
                        probe.middle,
                        additional,
                        &shape,
                        &mut on_edge,
                        &mut middle_distance,
                    )?;
                    if status == 0 {
                        let (_, other) = adaptor.d1(on_edge)?;
                        if other.square_magnitude() > gp::resolution()
                            && tangent.dot(other.normalized()?).abs() >= 0.9063
                        {
                            real_tolerance = additional;
                            coefficient = 2.;
                        }
                    }
                }
            }
        }
        let mut probe_box = BoundingBox3::default();
        probe_box.add_point(probe.middle);
        probe_box.enlarge(real_tolerance);
        if info.box_bounds().is_out_box(&probe_box) || status < 0 {
            return Ok(None);
        }
        let mut distance = 0.;
        let mut parameter = 0.;
        if status == 0 {
            distance = middle_distance;
        } else if context.compute_pe(
            probe.middle,
            real_tolerance,
            &shape,
            &mut parameter,
            &mut distance,
        )? < 0
        {
            return Ok(None);
        }
        let mut end_distance = 0.;
        if flag1 == 1 {
            flag1 = i32::from(
                context.compute_pe(
                    probe.first,
                    real_tolerance,
                    &shape,
                    &mut parameter,
                    &mut end_distance,
                )? == 0,
            );
            if flag1 != 0 && distance < end_distance {
                distance = end_distance;
            }
        }
        if flag2 == 1 {
            flag2 = i32::from(
                context.compute_pe(
                    probe.last,
                    real_tolerance,
                    &shape,
                    &mut parameter,
                    &mut end_distance,
                )? == 0,
            );
            if flag2 != 0 && distance < end_distance {
                distance = end_distance;
            }
        }
        if flag1 == 0 || flag2 == 0 {
            return Ok(None);
        }
        Ok(Some((distance, coefficient)))
    }

    /// Whether the block is On/In the first and the second face.
    pub(super) fn block_in_faces(
        &self,
        block: &PaveBlockHandle,
        faces: (i32, i32),
    ) -> Result<(bool, bool), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let first = structure.face_info(faces.0)?;
        let second = structure.face_info(faces.1)?;
        Ok((
            first.pave_blocks_on().contains(block) || first.pave_blocks_in().contains(block),
            second.pave_blocks_on().contains(block) || second.pave_blocks_in().contains(block),
        ))
    }

    /// Appends an existing pave block to the curve's blocks and keeps its edge and vertices
    /// for the post treatment (`PreparePostTreatFF`).
    pub(super) fn prepare_post_treat_ff(
        &mut self,
        curve: CurveRef,
        block: &PaveBlockHandle,
        sections: &mut SectionShapes,
    ) -> Result<(), Failure> {
        self.ff_curve_mut(curve)?
            .change_pave_blocks()
            .push(Arc::clone(block));
        let (bounds, edge) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.indices(), locked.edge())
        };
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut couple = CoupleOfPaveBlocks::new();
        couple.set_interference_index(curve.0 as i32);
        couple.set_index(curve.1 as i32);
        couple.set_pave_block1(Some(Arc::clone(block)));
        sections.couples.add(structure.shape(edge)?.clone(), couple);
        sections
            .vertex_indices
            .bind(structure.shape(bounds.0)?.clone(), bounds.0);
        sections
            .vertex_indices
            .bind(structure.shape(bounds.1)?.clone(), bounds.1);
        Ok(())
    }
}

/// Appends a face to a block's face list unless already present.
pub(super) fn add_block_face(block_faces: &mut BlockFaces, block: &PaveBlockHandle, face: i32) {
    if let Some(list) = block_faces.change_seek(block) {
        if !list.contains(&face) {
            list.push(face);
        }
        return;
    }
    block_faces.bind(Arc::clone(block), vec![face]);
}
