//! Shared native fixture policy and workload names keep captures and timings paired.
use martensite_vulkan::{RenderMeshEdgePipeline, RenderTextureCoordinatePipeline};
/// Select the native fixture's explicit policies; `cap_fixture_pipelines("cap-generated-polygon")`.
pub fn cap_fixture_pipelines(
    mode: &str,
) -> (RenderTextureCoordinatePipeline, RenderMeshEdgePipeline) {
    let texture = if mode.contains("generated") || mode.contains("fixed") {
        RenderTextureCoordinatePipeline::GeneratedCompatibility
    } else {
        RenderTextureCoordinatePipeline::Programmable
    };
    let edges = if mode.contains("polygon") {
        RenderMeshEdgePipeline::PolygonCompatibility
    } else {
        RenderMeshEdgePipeline::Programmable
    };
    (texture, edges)
}
pub const CAP_TEXTURE_EDGE_MODES: &[&str] = &[
    "cap-section-edges-oblique-generated-polygon-thin-sampler-updated",
    "cap-section-edges-generated-polygon-thin-fixed-bilinear-fixed-center-edges-disabled",
    "cap-section-edges-generated-polygon-thin-fixed-trilinear-fixed-minified-edges-disabled",
    "cap-section-edges-generated-polygon-thin-fixed-trilinear-fixed-minified-no-modulate-edges-disabled",
    "cap-section-edges-generated-polygon-thin-fixed-trilinear-fixed-minified-image-refreshed-edges-disabled",
    "cap-section-edges-generated-polygon-thin-blend-edges-disabled",
    "cap-section-edges-generated-polygon-thin-mask-edges-disabled",
    "cap-section-edges-generated-polygon-thin-mask-blend-edges-disabled",
    "cap-section-edges-generated-polygon-thin-opaque-alpha-edges-disabled",
    "cap-section-edges-oblique-generated-polygon-thin-bc1-rgb",
    "cap-section-edges-oblique-generated-polygon-thin-bc1-rgba",
    "cap-section-edges-oblique-generated-polygon-thin-bc2",
    "cap-section-edges-oblique-generated-polygon-thin-bc3",
    "cap-section-edges",
    "cap-section-edges-oblique",
    "cap-section-edges-oblique-dashed",
    "cap-section-edges-oblique-skip-first",
    "cap-section-edges-oblique-edges-disabled",
    "cap-section-edges-oblique-edge-alpha",
    "cap-section-edges-oblique-negative-alpha",
    "cap-section-edges-oblique-edge-background",
    "cap-section-edges-oblique-mask",
    "cap-section-edges-oblique-mask-blend",
    "cap-section-edges-oblique-blend",
    "cap-section-edges-oblique-phong",
    "cap-section-edges-oblique-gouraud",
    "cap-section-edges-oblique-facet",
    "cap-section-edges-oblique-object-material",
    "cap-section-edges-oblique-object-texture",
    "cap-section-edges-oblique-no-modulate",
    "cap-section-edges-oblique-polygon",
    "cap-section-edges-oblique-polygon-fill-offset",
    "cap-section-edges-oblique-dashed-fill-offset",
    "cap-section-edges-oblique-dashed-line-offset",
    "cap-section-edges-oblique-dashed-both-offset",
    "cap-section-edges-oblique-generated",
    "cap-section-edges-oblique-generated-eye",
    "cap-section-edges-oblique-generated-polygon",
    "cap-section-edges-oblique-generated-eye-polygon",
    "cap-section-edges-oblique-image-updated",
    "cap-section-edges-oblique-image-refreshed",
    "cap-section-edges-oblique-sampler-updated",
    "cap-section-edges-oblique-plane-updated",
    "cap-section-edges-oblique-alpha-updated",
    "cap-section-edges-oblique-bc1-rgba",
    "cap-section-edges-oblique-bc3",
    "cap-section-edges-oblique-source-mask",
    "cap-section-edges-oblique-source-edges",
    "cap-section-edges-oblique-generated-polygon-thin",
    "cap-section-edges-oblique-generated-eye-polygon-thin-alpha-updated",
    "cap-section-edges-oblique-shader-eye-polygon-thin-alpha-updated",
    "cap-section-edges-oblique-generated-polygon-thin-format-alpha-f",
    "cap-section-edges-oblique-generated-polygon-thin-format-alpha-f-no-modulate",
    "cap-section-edges-oblique-generated-polygon-thin-format-alpha",
    "cap-section-edges-oblique-generated-polygon-thin-format-alpha-no-modulate",
    "cap-section-edges-oblique-generated-polygon-thin-format-gray",
    "cap-section-edges-oblique-generated-polygon-thin-format-gray-no-modulate",
    "cap-section-edges-oblique-generated-polygon-thin-format-rgb",
    "cap-section-edges-oblique-generated-polygon-thin-format-rgb-no-modulate",
    "cap-section-edges-oblique-generated-polygon-thin-format-alpha-f-edges-disabled",
    "cap-section-edges-oblique-generated-polygon-thin-format-alpha-edges-disabled",
    "cap-section-edges-oblique-generated-polygon-thin-format-gray-edges-disabled",
    "cap-section-edges-oblique-generated-polygon-thin-format-rgb-edges-disabled",
    "cap-section-edges-oblique-generated-polygon-thin-no-modulate",
];
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_policy_and_workload_names_stay_unique() {
        let names: std::collections::HashSet<_> = CAP_TEXTURE_EDGE_MODES.iter().collect();
        assert_eq!(names.len(), CAP_TEXTURE_EDGE_MODES.len());
        assert_eq!(
            cap_fixture_pipelines("generated-polygon"),
            (
                RenderTextureCoordinatePipeline::GeneratedCompatibility,
                RenderMeshEdgePipeline::PolygonCompatibility
            )
        );
        assert_eq!(
            cap_fixture_pipelines("shader-eye-polygon"),
            (
                RenderTextureCoordinatePipeline::Programmable,
                RenderMeshEdgePipeline::PolygonCompatibility
            )
        );
        assert_eq!(
            cap_fixture_pipelines("ordinary"),
            (
                RenderTextureCoordinatePipeline::Programmable,
                RenderMeshEdgePipeline::Programmable
            )
        );
    }
}
