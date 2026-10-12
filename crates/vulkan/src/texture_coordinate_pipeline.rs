//! Inject coordinate behavior independently from shader lighting or the Vulkan device.

/// Select the legacy texture-coordinate path; `RenderTextureCoordinatePipeline::default()`
/// preserves the ordinary programmable path, including its supplied homogeneous UVs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderTextureCoordinatePipeline {
    /// Built-in programmable shaders consume the UV attribute independently of GenMode.
    #[default]
    Programmable,
    /// Emulate fixed-function texture-coordinate generation and its retained binding state.
    /// This selects coordinate behavior only; it does not select fixed-function lighting.
    GeneratedCompatibility,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_coordinate_pipeline_preserves_programmable_behavior() {
        assert_eq!(RenderTextureCoordinatePipeline::default(), RenderTextureCoordinatePipeline::Programmable);
        assert_ne!(RenderTextureCoordinatePipeline::GeneratedCompatibility, RenderTextureCoordinatePipeline::Programmable);
    }
}
