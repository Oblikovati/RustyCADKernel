//! Native face selection prioritizes analytic closed quadrics, then mesh, planar bounds and ordered wire contours.
use super::StdSelectBrepSelectionTool;
use crate::{select_mgr::SelectMgrEntityOwnerSource, select3d::*};
use rustycad_tkbrep::{brep::BRepTool, brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::{geom_abs::SurfaceType, poly::mesh_purpose};
pub(super) struct FaceSelectionCriteria {
    pub edge_points: i32,
    pub max_parameter: f64,
    pub interior: bool,
}
impl StdSelectBrepSelectionTool {
    /// Append native face entities while preserving existing entries and partial failures; `Self::get_sensitive_for_face(&face,&owner,&mut entities,false,9,500.,true)?`.
    #[allow(clippy::too_many_arguments)] // Pinned overload exposes append list and independent legacy tessellation flag.
    pub fn get_sensitive_for_face(
        face: &Face,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        entities: &mut Vec<Handle<dyn Select3DSensitiveEntity>>,
        _auto_triangulation: bool,
        edge_points: i32,
        max_parameter: f64,
        interior: bool,
    ) -> Result<bool, Failure> {
        let criteria = FaceSelectionCriteria {
            edge_points,
            max_parameter,
            interior,
        };
        catch_native_failure(|| select_brep_face(face, owner, entities, &criteria))
    }
}
fn select_brep_face(
    face: &Face,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    entities: &mut Vec<Handle<dyn Select3DSensitiveEntity>>,
    criteria: &FaceSelectionCriteria,
) -> Result<bool, Failure> {
    if let Some(analytic) =
        super::brep_face_quadric::brep_face_analytic(face, owner, criteria.interior)?
    {
        entities.push(analytic);
        return Ok(true);
    }
    let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    if !mesh.is_null() {
        entities.push(
            Handle::new(Select3DSensitiveTriangulation::new(
                owner,
                &mesh,
                &location,
                criteria.interior,
            )?)
            .cast(),
        );
        return Ok(true);
    }
    let surface = BRepAdaptorSurface::with_face(face, true)?;
    if surface.get_type()? == SurfaceType::Plane {
        entities.push(super::brep_face_plane::brep_face_plane_sensitive(
            &surface, owner, criteria,
        )?);
        return Ok(true);
    }
    super::brep_face_contour::brep_face_contour_sensitive(face, &surface, owner, entities, criteria)
}
