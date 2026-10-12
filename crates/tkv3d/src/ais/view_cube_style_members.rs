//! Native front/back materials and live group aspect synchronization preserve drawer ownership flags.
use super::{AisViewCubeColorRef, AisViewCubePresentationSource, AisViewCubeShadingRef};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dShadingAspect};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel as Facing, graphic3d::*};
/// Real native style operations; `cube.set_box_transparency(0.3)?`.
pub trait AisViewCubeStyleMembers: AisViewCubePresentationSource {
    /// Borrow the actual drawer shading slot selected once; `cube.box_side_style_ref()`.
    fn box_side_style_ref(&self) -> AisViewCubeShadingRef<'_> {
        AisViewCubeShadingRef::drawer(&self.attributes(), self.view_cube_presentation_base())
    }
    /// Borrow the protected live edge slot; `cube.box_edge_style_ref()`.
    fn box_edge_style_ref(&self) -> AisViewCubeShadingRef<'_> {
        AisViewCubeShadingRef::local(&self.view_cube_presentation_base().edge_aspect)
    }
    /// Borrow the protected live corner slot; `cube.box_corner_style_ref()`.
    fn box_corner_style_ref(&self) -> AisViewCubeShadingRef<'_> {
        AisViewCubeShadingRef::local(&self.view_cube_presentation_base().corner_aspect)
    }
    /// Bind the native front color field once; `cube.box_color_ref().value()`.
    fn box_color_ref(&self) -> AisViewCubeColorRef<'_> {
        AisViewCubeColorRef::shading(
            &self.attributes().shading_aspect(),
            Facing::FrontSide,
            self.view_cube_presentation_base(),
        )
    }
    /// Update all three aspects only when any native color differs; `cube.set_box_color(red)?`.
    fn set_box_color(&self, color: Color) -> Result<(), Failure> {
        let aspects = cube_style_handles(self);
        if aspects
            .iter()
            .all(|style| style.color(Facing::FrontSide).is_equal(color))
        {
            return Ok(());
        }
        for style in &aspects {
            style.set_color(color, Facing::BothSide)?;
        }
        self.synchronize_aspects();
        Ok(())
    }
    /// Read side-front transparency; `cube.box_transparency()`.
    fn box_transparency(&self) -> f64 {
        self.attributes()
            .shading_aspect()
            .transparency(Facing::FrontSide)
    }
    /// Update all three aspects only beyond native Confusion; `cube.set_box_transparency(0.3)?`.
    fn set_box_transparency(&self, value: f64) -> Result<(), Failure> {
        let aspects = cube_style_handles(self);
        let changed = aspects.iter().any(|style| {
            (style.transparency(Facing::FrontSide) - value).abs()
                > rustycad_tkernel::precision::confusion()
        });
        if !changed {
            return Ok(());
        }
        for style in &aspects {
            style.set_transparency(value, Facing::BothSide)?;
        }
        self.synchronize_aspects();
        Ok(())
    }
    /// Bind the native back color field once; `cube.inner_color_ref().value()`.
    fn inner_color_ref(&self) -> AisViewCubeColorRef<'_> {
        AisViewCubeColorRef::shading(
            &self.attributes().shading_aspect(),
            Facing::BackSide,
            self.view_cube_presentation_base(),
        )
    }
    /// Change only side-back color and always synchronize; `cube.set_inner_color(red)?`.
    fn set_inner_color(&self, color: Color) -> Result<(), Failure> {
        self.attributes()
            .shading_aspect()
            .set_color(color, Facing::BackSide)?;
        self.synchronize_aspects();
        Ok(())
    }
}
impl<T: AisViewCubePresentationSource + ?Sized> AisViewCubeStyleMembers for T {}
pub(super) fn cube_style_handles<C: AisViewCubePresentationSource + ?Sized>(
    cube: &C,
) -> [Handle<Prs3dShadingAspect>; 3] {
    let base = cube.view_cube_presentation_base();
    [
        cube.attributes().shading_aspect(),
        base.edge_aspect.read().clone(),
        base.corner_aspect.read().clone(),
    ]
}
pub(super) fn cube_unset_color<C: AisViewCubePresentationSource + ?Sized>(
    cube: &C,
) -> Result<(), Failure> {
    for (style, color) in cube_style_handles(cube).iter().zip([
        NamedColor::WHITE,
        NamedColor::GRAY30,
        NamedColor::GRAY30,
    ]) {
        style.set_color(Color::named(color), Facing::BothSide)?;
    }
    cube.synchronize_aspects();
    Ok(())
}
pub(super) fn cube_set_material<C: AisViewCubePresentationSource + ?Sized>(
    cube: &C,
    material: &Graphic3dMaterialAspect,
) {
    for style in &cube_style_handles(cube) {
        style.set_material(material, Facing::BothSide);
    }
    cube.synchronize_aspects();
}
pub(super) fn cube_unset_material<C: AisViewCubePresentationSource + ?Sized>(
    cube: &C,
) -> Result<(), Failure> {
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::UserDefined);
    material.set_color(Color::named(NamedColor::WHITE))?;
    material.set_ambient_color(Color::named(NamedColor::GRAY60));
    let styles = cube_style_handles(cube);
    for (index, style) in styles.iter().enumerate() {
        style.set_material(&material, Facing::BothSide);
        if index > 0 {
            style.set_color(Color::named(NamedColor::GRAY30), Facing::BothSide)?;
        }
    }
    cube.synchronize_aspects();
    Ok(())
}
