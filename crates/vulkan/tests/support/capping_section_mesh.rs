//! Genuine capping aspects retain their own edge state independently of source geometry.
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::{
    aspect::AspectTypeOfLine,
    graphic3d::{
        Graphic3dAspectsEdgeMembers, Graphic3dAspectsLineMembers, Graphic3dAspectsRenderMembers,
        Graphic3dAspectsSource,
    },
};
pub fn configure_section_mesh_edges(aspect: &dyn Graphic3dAspectsSource, mode: &str) {
    if !mode.contains("section-edges") {
        return;
    }
    aspect.set_draw_edges(!mode.contains("edges-disabled"));
    aspect.set_edge_color_rgba(ColorRgba::new(0., 1., 1., section_edge_alpha(mode)).unwrap());
    aspect
        .set_line_width(if mode.contains("thin") { 1. } else { 3. })
        .unwrap();
    aspect.set_skip_first_edge(mode.contains("skip-first"));
    if mode.contains("dashed") {
        aspect.set_line_type(AspectTypeOfLine::Dash);
    }
    configure_section_mesh_offset(aspect, mode);
}
fn section_edge_alpha(mode: &str) -> f32 {
    if mode.contains("negative-alpha") {
        return -0.25;
    }
    if mode.contains("edge-alpha") {
        return 0.25;
    }
    if mode.contains("edge-background") {
        return 0.05;
    }
    1.
}
fn configure_section_mesh_offset(aspect: &dyn Graphic3dAspectsSource, mode: &str) {
    let mask = if mode.contains("both-offset") {
        3
    } else if mode.contains("line-offset") {
        2
    } else {
        1
    };
    if mode.contains("offset") {
        aspect.set_polygon_offsets(mask, if mode.contains("factor-zero") { 0. } else { 2. }, 3.);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkservice::graphic3d::*;
    #[test]
    fn cap_edges_preserve_selected_alpha_and_skip_policy() {
        let aspect = Graphic3dAspects::default();
        aspect.set_alpha_mode(Graphic3dAlphaMode::MaskBlend, 0.5);
        configure_section_mesh_edges(&aspect, "section-edges-dashed-skip-first");
        assert!(aspect.to_draw_edges());
        assert_eq!(aspect.line_width(), 3.);
        assert!(aspect.to_skip_first_edge());
        assert_eq!(aspect.alpha_mode(), Graphic3dAlphaMode::MaskBlend);
        assert_eq!(aspect.line_type(), AspectTypeOfLine::Dash);
    }
    #[test]
    fn actual_edge_alpha_preserves_hollow_and_background_sentinels() {
        assert_eq!(section_edge_alpha("negative-alpha"), -0.25);
        assert_eq!(section_edge_alpha("edge-alpha"), 0.25);
        assert_eq!(section_edge_alpha("edge-background"), 0.05);
        assert_eq!(section_edge_alpha("opaque"), 1.);
    }
    #[test]
    fn section_line_and_fill_offset_masks_remain_independent() {
        for (mode, mask) in [("line-offset", 2), ("fill-offset", 1), ("both-offset", 3)] {
            let aspect = Graphic3dAspects::default();
            configure_section_mesh_offset(&aspect, mode);
            assert_eq!(aspect.polygon_offsets(), (mask, 2., 3.));
        }
    }
}
