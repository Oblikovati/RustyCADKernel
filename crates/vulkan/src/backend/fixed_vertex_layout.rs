//! Fixed vertices carry geometric and logical provoking inputs independently.
use ash::vk;
pub(super) fn fixed_vertex_layout(
    surface: bool,
) -> vk::PipelineVertexInputStateCreateInfo<'static> {
    const BASIC: [vk::VertexInputBindingDescription; 1] = [binding(88)];
    const SURFACE: [vk::VertexInputBindingDescription; 1] = [binding(104)];
    const BASIC_ATTRS: [vk::VertexInputAttributeDescription; 6] = [
        attr(0, 0, 4),
        attr(1, 16, 4),
        attr(2, 32, 3),
        attr(4, 44, 4),
        attr(5, 60, 3),
        attr(6, 72, 4),
    ];
    const SURFACE_ATTRS: [vk::VertexInputAttributeDescription; 7] = [
        attr(0, 0, 4),
        attr(1, 16, 4),
        attr(2, 32, 4),
        attr(3, 48, 3),
        attr(4, 60, 4),
        attr(5, 76, 3),
        attr(6, 88, 4),
    ];
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(if surface { &SURFACE } else { &BASIC })
        .vertex_attribute_descriptions(if surface {
            &SURFACE_ATTRS
        } else {
            &BASIC_ATTRS
        })
}
const fn binding(stride: u32) -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription {
        binding: 0,
        stride,
        input_rate: vk::VertexInputRate::VERTEX,
    }
}
const fn attr(location: u32, offset: u32, components: u32) -> vk::VertexInputAttributeDescription {
    let format = if components == 4 {
        vk::Format::R32G32B32A32_SFLOAT
    } else {
        vk::Format::R32G32B32_SFLOAT
    };
    vk::VertexInputAttributeDescription {
        location,
        binding: 0,
        format,
        offset,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_geometry_and_provoking_layouts_have_independent_offsets() {
        let basic = fixed_vertex_layout(false);
        let surface = fixed_vertex_layout(true);
        unsafe {
            assert_eq!((*basic.p_vertex_binding_descriptions).stride, 88);
            assert_eq!((*surface.p_vertex_binding_descriptions).stride, 104);
            assert_eq!((*basic.p_vertex_attribute_descriptions.add(3)).offset, 44);
            assert_eq!((*surface.p_vertex_attribute_descriptions.add(4)).offset, 60);
        }
        assert_eq!(binding(88).binding, 0);
        assert_eq!(attr(1, 16, 4).format, vk::Format::R32G32B32A32_SFLOAT);
    }
}
