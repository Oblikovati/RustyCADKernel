//! Resolve OCCT's opaque automatic culling without exposing Vulkan state to scene extraction.
use crate::PrimitiveDraw;
use rustycad_tkservice::graphic3d::Graphic3dTypeOfBackfacingModel;
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
}
