//! Resolve OCCT's opaque automatic culling without exposing Vulkan state to scene extraction.
use crate::PrimitiveDraw;
use rustycad_tkservice::graphic3d::{Graphic3dAspectsMembers, Graphic3dTypeOfBackfacingModel};
/// Face rejection after vertex transformation; `RenderFaceCulling::Back`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderFaceCulling {
    /// Draw both orientations.
    #[default]
    None,
    /// Reject back faces, as for native closed opaque groups.
    Back,
    /// Reject front faces.
    Front,
}
impl PrimitiveDraw {
    /// Set face rejection; `draw.with_face_culling(RenderFaceCulling::Back)`.
    pub fn with_face_culling(mut self, culling: RenderFaceCulling) -> Self {
        self.face_culling = culling;
        self
    }
    /// Read face rejection; `draw.face_culling()`.
    pub fn face_culling(&self) -> RenderFaceCulling {
        self.face_culling
    }
}
pub(crate) fn resolve_culling(
    mode: Graphic3dTypeOfBackfacingModel,
    closed: bool,
) -> RenderFaceCulling {
    use Graphic3dTypeOfBackfacingModel::*;
    match mode {
        BackCulled => RenderFaceCulling::Back,
        FrontCulled => RenderFaceCulling::Front,
        Auto if closed => RenderFaceCulling::Back,
        _ => RenderFaceCulling::None,
    }
}
pub(crate) fn allow_surface_auto_culling(
    aspect: &dyn rustycad_tkservice::graphic3d::Graphic3dAspectsSource,
) -> bool {
    use rustycad_tkservice::graphic3d::Graphic3dAlphaMode::*;
    !matches!(aspect.alpha_mode(), Blend | Mask | MaskBlend)
        && (aspect.alpha_mode() != BlendAuto || aspect.front_material().transparency() == 0.)
        && aspect.interior_style() != rustycad_tkservice::aspect::AspectInteriorStyle::Hatch
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn automatic_culling_is_only_for_closed_opaque_geometry() {
        use Graphic3dTypeOfBackfacingModel::*;
        assert_eq!(resolve_culling(Auto, true), RenderFaceCulling::Back);
        assert_eq!(resolve_culling(Auto, false), RenderFaceCulling::None);
        assert_eq!(resolve_culling(DoubleSided, true), RenderFaceCulling::None);
        assert_eq!(
            resolve_culling(FrontCulled, false),
            RenderFaceCulling::Front
        );
        assert_eq!(resolve_culling(BackCulled, false), RenderFaceCulling::Back);
    }
    #[test]
    fn native_auto_culling_excludes_all_masked_and_blended_modes() {
        use rustycad_tkservice::graphic3d::*;
        let aspect = Graphic3dAspects::default();
        for (mode, allowed) in [
            (Graphic3dAlphaMode::Opaque, true),
            (Graphic3dAlphaMode::BlendAuto, true),
            (Graphic3dAlphaMode::Blend, false),
            (Graphic3dAlphaMode::Mask, false),
            (Graphic3dAlphaMode::MaskBlend, false),
        ] {
            aspect.set_alpha_mode(mode, 0.5);
            assert_eq!(allow_surface_auto_culling(&aspect), allowed, "{mode:?}");
        }
        aspect.set_alpha_mode(Graphic3dAlphaMode::BlendAuto, 0.5);
        let mut material = aspect.front_material();
        material.set_transparency(0.25).unwrap();
        aspect.set_front_material(material);
        assert!(!allow_surface_auto_culling(&aspect));
        aspect.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
        aspect.set_interior_style(rustycad_tkservice::aspect::AspectInteriorStyle::Hatch);
        assert!(!allow_surface_auto_culling(&aspect));
    }
}
