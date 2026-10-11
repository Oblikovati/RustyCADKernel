//! Martensite camera-frustum geometry and ownership, from OCCT 3d097a0.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkservice::{
    aspect::{AspectTypeOfFacingModel, AspectTypeOfLine},
    graphic3d::*,
};
/// Native frustum sensitivity modes; `AisCameraFrustumSelectionMode::Edges`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AisCameraFrustumSelectionMode {
    /// Detect the duplicated face-border segments.
    Edges = 0,
    /// Detect the twelve face triangles.
    Volume = 1,
}
/// Retained primitive arrays; vertices update in place without replacing buffer identities.
#[derive(Default)]
pub struct AisCameraFrustumFields {
    /// Native points use double precision; the spare ninth point is never used.
    pub points: [[f64; 3]; 9],
    /// Lazy triangulation.
    pub triangles: Handle<Graphic3dArrayOfTriangles>,
    /// Lazy face borders, including shared edges twice.
    pub borders: Handle<Graphic3dArrayOfSegments>,
}
/// Camera visualization with live presentation aspects; `AisCameraFrustum::new()?`.
pub struct AisCameraFrustum {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    /// Native cached geometry and aliased primitive arrays.
    pub fields: SharedState<AisCameraFrustumFields>,
}
rustycad_tkernel::impl_transient!(AisCameraFrustum,"AIS_CameraFrustum",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisCameraFrustum {
    /// Create a white, plastified, blended frustum with transparency 0.7; `AisCameraFrustum::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::default(),
        });
        object.initialize_frustum_aspects()?;
        object.set_display_mode(1);
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
    fn initialize_frustum_aspects(&self) -> Result<(), Failure> {
        let white = Color::named(NamedColor::WHITE);
        let drawer = self.attributes();
        drawer.set_line_aspect(Handle::new(Prs3dLineAspect::new(
            white,
            AspectTypeOfLine::Solid,
            1.0,
        )?));
        let shading = default_frustum_shading(white)?;
        drawer.set_shading_aspect(shading);
        drawer.set_transparency(0.7);
        Ok(())
    }
    /// Snapshot camera corners, update shared arrays and invalidate presentations; null is unchanged.
    /// Example: `object.set_camera_frustum(&camera)?`.
    pub fn set_camera_frustum(&self, camera: &Handle<Graphic3dCamera>) -> Result<(), Failure> {
        let Some(camera) = camera.get() else {
            return Ok(());
        };
        let corners = camera.frustum_points(&rustycad_tkernel::ncollection::Mat4::identity());
        for (index, point) in corners.into_iter().enumerate() {
            self.fields.write().points[index] = [point.x, point.y, point.z];
        }
        self.fill_triangles()?;
        self.fill_borders()?;
        self.set_to_update_all();
        Ok(())
    }
}
impl AisInteractiveObject for AisCameraFrustum {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}

fn default_frustum_shading(white: Color) -> Result<Handle<Prs3dShadingAspect>, Failure> {
    let shading = Handle::new(Prs3dShadingAspect::new()?);
    shading.set_material(
        &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified),
        AspectTypeOfFacingModel::BothSide,
    );
    shading
        .aspect()
        .set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
    shading.set_transparency(0.7, AspectTypeOfFacingModel::BothSide)?;
    shading.set_color(white, AspectTypeOfFacingModel::BothSide)?;
    Ok(shading)
}
