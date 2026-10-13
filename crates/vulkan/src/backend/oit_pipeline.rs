//! OIT targets vary attachment layouts, never the retained geometry or material policy.
use super::pipeline_variants::PipelineVariant;
use crate::{RenderError, error::rejected};
use ash::vk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OitGeometryPhase {
    Weighted,
    Initial,
    Peeling,
}
impl OitGeometryPhase {
    pub fn name(self) -> &'static str {
        match self {
            Self::Weighted => "weighted",
            Self::Initial => "initial",
            Self::Peeling => "peeling",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct OitPipelineTarget {
    pub phase: OitGeometryPhase,
    pub formats: [vk::Format; 3],
    pub count: usize,
    pub samples: vk::SampleCountFlags,
    pub fixed_output: bool,
}
pub(super) fn oit_fragment_bytes(
    kind: PipelineVariant,
    target: OitPipelineTarget,
) -> Result<&'static [u8], RenderError> {
    if kind.point || kind.text || (kind.line && !kind.polygon_edges) {
        return Err(rejected(
            (kind.line, kind.point, kind.text),
            "a native fill primitive for an OIT geometry pass",
        ));
    }
    let name = oit_fragment_name(kind, target);
    super::oit_shader_table::OIT_FRAGMENTS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, bytes)| *bytes)
        .ok_or_else(|| rejected(name, "a compiled OIT fill shader interface"))
}
fn oit_fragment_name(kind: PipelineVariant, target: OitPipelineTarget) -> String {
    let suffix = if target.fixed_output {
        ".fixed-output"
    } else {
        ""
    };
    format!(
        "{}.oit.{}{suffix}",
        oit_fill_stem(kind),
        target.phase.name()
    )
}
fn oit_fill_stem(kind: PipelineVariant) -> String {
    let surface = kind.surface_texture || kind.surface_alpha || kind.mesh_edges;
    let family = oit_fill_family(kind, surface);
    let names = [
        (".edges", kind.mesh_edges),
        (".fixed", kind.fixed_lighting),
        (".lit", surface && kind.lighting && !kind.fixed_lighting),
        (".clipped", kind.clipped),
        (".image", kind.surface_texture),
    ];
    family.to_owned()
        + &names
            .into_iter()
            .filter_map(|(name, enabled)| enabled.then_some(name))
            .collect::<String>()
}
fn oit_fill_family(kind: PipelineVariant, surface: bool) -> &'static str {
    if surface {
        return "surface";
    }
    if kind.lighting {
        return "lighting";
    }
    "unlit"
}
pub(super) fn geometry_blend_attachments(
    target: OitPipelineTarget,
    kind: PipelineVariant,
) -> Vec<vk::PipelineColorBlendAttachmentState> {
    let mut blend = if target.phase == OitGeometryPhase::Weighted {
        super::oit_blend::weighted_blend()
    } else {
        super::oit_blend::peeling_blend()
    };
    if kind.capping.is_mask() {
        blend.color_write_mask = vk::ColorComponentFlags::empty();
    }
    vec![blend; target.count]
}
#[cfg(test)]
mod tests {
    use super::*;
    fn unlit_kind() -> PipelineVariant {
        super::super::pipeline_variants::PipelineVariant::from_index(0, false)
    }
    fn weighted_target() -> OitPipelineTarget {
        OitPipelineTarget {
            phase: OitGeometryPhase::Weighted,
            formats: [vk::Format::R16G16B16A16_SFLOAT; 3],
            count: 2,
            samples: vk::SampleCountFlags::TYPE_1,
            fixed_output: false,
        }
    }
    #[test]
    fn phase_and_fixed_broadcast_have_actual_distinct_spirv_interfaces() {
        let kind = unlit_kind();
        let mut target = weighted_target();
        let weighted = oit_fragment_bytes(kind, target).unwrap();
        target.phase = OitGeometryPhase::Initial;
        let initial = oit_fragment_bytes(kind, target).unwrap();
        target.phase = OitGeometryPhase::Peeling;
        assert_ne!(weighted, initial);
        assert_ne!(initial, oit_fragment_bytes(kind, target).unwrap());
        target.fixed_output = true;
        assert!(oit_fragment_bytes(kind, target).is_ok());
    }
    #[test]
    fn sampled_fixed_edge_family_keeps_its_material_and_clipping_interface() {
        let mut kind = unlit_kind();
        kind.mesh_edges = true;
        kind.surface_texture = true;
        kind.lighting = true;
        kind.fixed_lighting = true;
        kind.clipped = true;
        assert_eq!(oit_fill_stem(kind), "surface.edges.fixed.clipped.image");
        assert!(oit_fragment_bytes(kind, weighted_target()).is_ok());
        kind.point = true;
        assert!(
            oit_fragment_bytes(kind, weighted_target())
                .unwrap_err()
                .to_string()
                .contains("fill primitive")
        );
    }
}
