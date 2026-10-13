// Pipeline regressions remain beside their private implementation.
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_marker_cutoff_extends_only_its_point_constant_range() {
        assert_eq!(fixed_marker_push_range(true, false, true).size, 84);
        assert_eq!(fixed_marker_push_range(true, false, false).size, 80);
        assert_eq!(fixed_marker_push_range(false, false, false).size, 64);
        assert_eq!(
            fixed_marker_push_range(true, false, true).stage_flags,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT
        );
    }
    #[test]
    fn triangle_constants_do_not_invalidate_fragment_state() {
        assert_eq!(
            push_range(false, false).stage_flags,
            vk::ShaderStageFlags::VERTEX
        );
        assert_eq!(push_range(false, false).size, 64);
        assert_eq!(push_range(true, false).size, 80);
        assert!(
            push_range(true, false)
                .stage_flags
                .contains(vk::ShaderStageFlags::FRAGMENT)
        );
    }
    #[test]
    fn native_mesh_constants_are_visible_to_all_three_shader_stages() {
        let range = push_range(false, true);
        assert_eq!(range.size, 128);
        assert_eq!(range.offset, 0);
        assert_eq!(
            range.stage_flags,
            vk::ShaderStageFlags::VERTEX
                | vk::ShaderStageFlags::GEOMETRY
                | vk::ShaderStageFlags::FRAGMENT
        );
    }
}

#[cfg(test)]
mod marker_descriptor_tests {
    use super::*;
    use ash::vk::Handle;
    #[test]
    fn lit_marker_keeps_lighting_clipping_and_sprite_in_distinct_slots() {
        let [sprite, lighting, clipping] = [1, 2, 3].map(vk::DescriptorSetLayout::from_raw);
        assert_eq!(
            descriptor_layout_handles(Some(sprite), Some(lighting), Some(clipping)),
            vec![lighting, clipping, sprite]
        );
        assert_eq!(
            descriptor_layout_handles(Some(sprite), Some(lighting), None),
            vec![lighting, lighting, sprite]
        );
        assert_eq!(
            descriptor_layout_handles(Some(sprite), None, Some(clipping)),
            vec![sprite, clipping]
        );
        assert_eq!(
            descriptor_layout_handles(None, Some(lighting), Some(clipping)),
            vec![lighting, clipping]
        );
        assert_eq!(
            descriptor_layout_handles(None, None, Some(clipping)),
            vec![clipping, clipping]
        );
        assert!(descriptor_layout_handles(None, None, None).is_empty());
    }
}
