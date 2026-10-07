//! `BOPAlgo_CheckerSI::CheckFaceSelfIntersection` and its `BOPAlgo_FaceSelfIntersect` task.
use super::BOPAlgoCheckerSI;
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
    pub(super) fn check_face_self_intersection(&mut self) -> Result<(), Failure> {
        if self.level_of_check < FACE_FACE_LEVEL {
            return Ok(());
        }
        let run_parallel = self.filler.options().run_parallel();
        let structure = self.structure_mut()?;
        structure.interference_table_mut().clear(false);
        for (index, face) in self_intersection_candidates(structure)? {
            if is_self_intersecting(&face, run_parallel)? {
                structure
                    .interference_table_mut()
                    .add(Pair::new(index, index));
            }
        }
        Ok(())
    }
}

/// The source faces whose surface type may intersect itself, with their indices.
fn self_intersection_candidates(structure: &DataStructure) -> Result<Vec<(i32, Face)>, Failure> {
    let mut candidates = Vec::new();
    for index in 0..structure.nb_source_shapes() {
        let info = structure.shape_info(index)?;
        if info.shape_type() != ShapeEnum::Face {
            continue;
        }
        let face = topods::face(info.shape())?;
        if may_self_intersect(&face)? {
            candidates.push((index, face));
        }
    }
    Ok(candidates)
}

/// False for the analytic surfaces that cannot self-intersect, a torus included when its
/// major radius exceeds the minor one by more than the confusion tolerance.
fn may_self_intersect(face: &Face) -> Result<bool, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    Ok(match surface.get_type()? {
        SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere => {
            false
        }
        SurfaceType::Torus => {
            let torus = surface.torus()?;
            torus.major_radius() <= torus.minor_radius() + precision::confusion()
        }
        _ => true,
    })
}

/// Whether intersecting the face with itself gives a curve or a point
/// (`BOPAlgo_FaceSelfIntersect::Perform`).
fn is_self_intersecting(face: &Face, run_parallel: bool) -> Result<bool, Failure> {
    let mut intersector = FaceFace::default();
    intersector.perform(face, face, run_parallel)?;
    if !intersector.is_done() {
        return Ok(false);
    }
    Ok(!intersector.lines()?.is_empty() || !intersector.points().is_empty())
}
