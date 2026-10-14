//! Select legacy lighting independently of the default programmable renderer.
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;

/// Inject the native lighting algorithm; `RenderLightingPipeline::FixedFunction`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderLightingPipeline {
    /// Preserve the built-in Phong-family programmable behavior.
    #[default]
    Programmable,
    /// Match the legacy per-vertex GL lighting and material/color rules.
    FixedFunction,
}
pub(crate) fn structure_lighting_pipeline(
    structure: &rustycad_tkernel::standard::Handle<
        dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource,
    >,
) -> RenderLightingPipeline {
    structure_render_driver(structure)
        .get()
        .map_or_else(Default::default, |driver| driver.lighting_pipeline)
}
pub(crate) fn structure_unlit_face_policy(
    structure: &rustycad_tkernel::standard::Handle<
        dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource,
    >,
) -> crate::RenderUnlitFaceColorPolicy {
    structure_render_driver(structure)
        .get()
        .map_or_else(Default::default, |driver| driver.unlit_face_color_policy)
}
fn structure_render_driver(
    structure: &rustycad_tkernel::standard::Handle<
        dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource,
    >,
) -> rustycad_tkernel::standard::Handle<crate::Graphic3dRenderDriver> {
    let manager = structure.structure_manager();
    let Some(manager) = manager.get() else {
        return Default::default();
    };
    manager.graphic_driver().cast()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_driver_keeps_programmable_lighting() {
        assert_eq!(
            RenderLightingPipeline::default(),
            RenderLightingPipeline::Programmable
        );
        assert_ne!(
            RenderLightingPipeline::FixedFunction,
            RenderLightingPipeline::default()
        );
    }
}
