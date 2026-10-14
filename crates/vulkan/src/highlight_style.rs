//! OpenGL color highlighting overrides vertex/bound color without replacing line width/pattern.
use crate::{PrimitiveDraw, RenderError, error::rejected};
use rustycad_tkservice::{aspect::AspectTypeOfHighlightMethod, graphic3d::*};
pub(crate) fn validate_structure(structure: &Graphic3dCStructureFields) -> Result<(), RenderError> {
    let style = &structure.highlight_style;
    if structure.highlight
        && !style.is_null()
        && style.method() != AspectTypeOfHighlightMethod::Color
    {
        return Err(rejected(
            style.method(),
            "color highlighting until bounding-box highlighting is implemented",
        ));
    }
    Ok(())
}
pub(crate) fn apply_highlight(
    draw: &mut PrimitiveDraw,
    structure: &Graphic3dCStructureFields,
) -> Result<(), RenderError> {
    if !structure.highlight || structure.highlight_style.is_null() {
        return Ok(());
    }
    let color = structure.highlight_style.color_rgba();
    let rgba = [
        color.rgb().red() as f32,
        color.rgb().green() as f32,
        color.rgb().blue() as f32,
        color.alpha(),
    ];
    let vertices = draw
        .vertices
        .iter()
        .map(|v| crate::RenderVertex { color: rgba, ..*v })
        .collect::<Vec<_>>();
    crate::scene::validate_components(&vertices, &draw.clip_transform)?;
    crate::scene::validate_annotation_opacity(&vertices)?;
    draw.vertices = vertices;
    draw.unlit_back_color = draw.unlit_back_color.map(|_| rgba);
    if let Some(lighting) = &mut draw.lighting {
        lighting
            .material
            .set_color(rgba[..3].try_into().expect("RGB components"));
        lighting.vertex_colors = false;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
    #[test]
    fn color_highlight_preserves_homogeneous_position_weight() {
        let manager = Handle::new(
            Graphic3dStructureManager::new(
                &rustycad_tkservice::null_driver::NullGraphicDriver::new_handle(),
            )
            .unwrap(),
        );
        let mut fields = Graphic3dCStructureBase::new(&manager)
            .unwrap()
            .fields
            .read()
            .clone();
        fields.highlight = true;
        fields.highlight_style = Handle::new(Graphic3dPresentationAttributes::default()).cast();
        fields
            .highlight_style
            .set_color(*ColorRgba::new(0., 1., 0., 1.).unwrap().rgb());
        let vertices = vec![crate::RenderVertex::homogeneous([2., 3., 4., 2.], [1.; 4]); 3];
        let mut draw = PrimitiveDraw::new(vertices, vec![0, 1, 2], crate::identity_clip()).unwrap();
        draw.unlit_back_color = Some([1., 0., 0., 0.25]);
        apply_highlight(&mut draw, &fields).unwrap();
        assert_eq!(draw.unlit_back_color, Some([0., 1., 0., 1.]));
        assert_eq!(draw.vertices[0].weight, 2.);
        assert_eq!(draw.vertices[0].color, [0., 1., 0., 1.]);
        fields.highlight_style.set_transparency(0.5);
        assert!(apply_highlight(&mut draw, &fields).is_err());
    }
}
