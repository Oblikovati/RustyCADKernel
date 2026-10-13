//! `BOPAlgo_CheckerSI::CheckFaceSelfIntersection` and its `BOPAlgo_FaceSelfIntersect` task.
use super::BOPAlgoCheckerSI;
use super::completion::{
    SelfCheckCompletion, SelfCheckPhase, enter_face, enter_phase, record_policy,
    record_termination, schedule_face_phase,
};
use crate::bop_ds::{DataStructure, Pair};
use crate::int_tools::FaceFace;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;

/// The first level of check that includes face/face interferences.
const FACE_FACE_LEVEL: i32 = 5;

impl BOPAlgoCheckerSI {
    /// Replaces the interference table by the faces that intersect themselves, from level
    /// 5 (F/F) on; planes, cylinders, cones, spheres and ring tori never do, so only the other
    /// faces are intersected with themselves (`CheckFaceSelfIntersection`).
    pub(super) fn check_face_self_intersection(
        &mut self,
        mut trace: Option<&mut SelfCheckCompletion>,
    ) -> Result<(), Failure> {
        if self.level_of_check < FACE_FACE_LEVEL {
            return Ok(());
        }
        schedule_face_phase(trace.as_deref_mut());
        let run_parallel = self.filler.options().run_parallel();
        let structure = self.structure_mut()?;
        structure.interference_table_mut().clear(false);
        let candidates = self_intersection_candidates(structure, trace.as_deref_mut())?;
        perform_candidates(structure, candidates, run_parallel, trace)
    }
}

fn perform_candidates(
    structure: &mut DataStructure,
    candidates: Vec<(i32, Face)>,
    run_parallel: bool,
    mut trace: Option<&mut SelfCheckCompletion>,
) -> Result<(), Failure> {
    enter_phase(trace.as_deref_mut(), SelfCheckPhase::FaceSelf);
    for (index, face) in candidates {
        enter_face(trace.as_deref_mut(), index);
        if is_self_intersecting(&face, run_parallel, index, trace.as_deref_mut())? {
            add_self_pair(structure, index);
        }
    }
    Ok(())
}

/// The source faces whose surface type may intersect itself, with their indices.
fn self_intersection_candidates(
    structure: &DataStructure,
    mut trace: Option<&mut SelfCheckCompletion>,
) -> Result<Vec<(i32, Face)>, Failure> {
    let mut candidates = Vec::new();
    for index in 0..structure.nb_source_shapes() {
        let info = structure.shape_info(index)?;
        if info.shape_type() != ShapeEnum::Face {
            continue;
        }
        enter_face(trace.as_deref_mut(), index);
        let face = topods::face(info.shape())?;
        if may_self_intersect(&face, index, trace.as_deref_mut())? {
            candidates.push((index, face));
        }
    }
    Ok(candidates)
}

/// False for the analytic surfaces that cannot self-intersect, a torus included when its
/// major radius exceeds the minor one by more than the confusion tolerance.
fn may_self_intersect(
    face: &Face,
    index: i32,
    trace: Option<&mut SelfCheckCompletion>,
) -> Result<bool, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let kind = surface.get_type()?;
    let candidate = match kind {
        SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere => {
            false
        }
        SurfaceType::Torus => {
            let torus = surface.torus()?;
            torus.major_radius() <= torus.minor_radius() + precision::confusion()
        }
        _ => true,
    };
    record_policy(trace, index, kind, candidate);
    Ok(candidate)
}

/// Whether intersecting the face with itself gives a curve or a point
/// (`BOPAlgo_FaceSelfIntersect::Perform`).
fn is_self_intersecting(
    face: &Face,
    run_parallel: bool,
    index: i32,
    trace: Option<&mut SelfCheckCompletion>,
) -> Result<bool, Failure> {
    let mut intersector = FaceFace::default();
    intersector.perform(face, face, run_parallel)?;
    if !intersector.is_done() {
        record_termination(trace, index, false, false);
        return Ok(false);
    }
    let contacts = !intersector.lines()?.is_empty() || !intersector.points().is_empty();
    record_termination(trace, index, true, contacts);
    Ok(contacts)
}

fn add_self_pair(structure: &mut DataStructure, index: i32) {
    let pair = Pair::new(index, index);
    structure.interference_table_mut().add(pair);
}
