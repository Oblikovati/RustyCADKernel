//! Native textured appearance overrides update only already-existing mode3 presentations.
use super::{
    AisTexturedShapeSource, ais_shape_set_color, ais_shape_set_material, ais_shape_unset_color,
    ais_shape_unset_material, ais_textured_shape_update_attributes,
};
use crate::prs_mgr::PrsMgrPresentation;
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::Graphic3dMaterialAspect;

/// Apply inherited color then update existing textured modes; `ais_textured_shape_set_color(&object,color)?`.
pub fn ais_textured_shape_set_color<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    color: Color,
) -> Result<(), Failure> {
    ais_shape_set_color(object, color)?;
    refresh_textured_mode_presentations(object)
}
/// Remove inherited color without refreshing the native copied texture aspect; `ais_textured_shape_unset_color(&object)?`.
pub fn ais_textured_shape_unset_color<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
) -> Result<(), Failure> {
    // OCCT 3d097a0 intentionally delegates without the loop used by the other three overrides.
    ais_shape_unset_color(object)
}
/// Apply inherited material then refresh existing textured modes; `ais_textured_shape_set_material(&object,&material)?`.
pub fn ais_textured_shape_set_material<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    ais_shape_set_material(object, material)?;
    refresh_textured_mode_presentations(object)
}
/// Restore inherited material while preserving native parent ownership behavior; `ais_textured_shape_unset_material(&object)?`.
pub fn ais_textured_shape_unset_material<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
) -> Result<(), Failure> {
    ais_shape_unset_material(object)?;
    refresh_textured_mode_presentations(object)
}
fn refresh_textured_mode_presentations<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
) -> Result<(), Failure> {
    let mut index = 0;
    loop {
        // OCCT 3d097a0 iterates the live Sequence, including presentations appended by group hooks.
        let current = live_textured_presentation_at(object, index);
        let Some(presentation) = current else {
            return Ok(());
        };
        if presentation.mode() == 3 {
            ais_textured_shape_update_attributes(object, &presentation.cast())?;
        }
        index += 1;
    }
}

fn live_textured_presentation_at<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    index: usize,
) -> Option<Handle<PrsMgrPresentation>> {
    object
        .presentable_base()
        .read()
        .presentations
        .get(index)
        .cloned()
}
