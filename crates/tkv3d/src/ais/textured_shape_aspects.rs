//! Native updateAttributes ordering: fresh aspect, optional fresh texture, then every existing group.
use super::{
    textured_shape_factory::AisTexturedTextureSource, textured_shape_source::AisTexturedShapeSource,
};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::{
    message::Gravity,
    standard::{Failure, Handle, exceptions::null_object},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::*;

/// Native protected update body; `ais_textured_shape_update_attributes(&object,&presentation)?` changes existing groups only.
pub fn ais_textured_shape_update_attributes<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    presentation: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    fresh_textured_fill_aspect(object)?;
    let has_texture = refresh_textured_source(object)?;
    apply_textured_fill_flags(object, has_texture);
    let presentation = presentation.get().ok_or_else(|| null_object("AIS_TexturedShape::updateAttributes: null presentation; expected an existing presentation"))?;
    update_live_textured_groups(object, presentation);
    Ok(())
}
fn update_live_textured_groups<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    presentation: &dyn Graphic3dStructureSource,
) {
    // Native Sequence iteration follows groups appended by virtual aspect callbacks.
    let mut index = 0;
    while let Some(group) = live_textured_group_at(presentation, index) {
        let aspect = object.textured_shape_base().fields.read().aspect.clone();
        group.set_group_primitives_aspect(&aspect.cast());
        index += 1;
    }
}
fn live_textured_group_at(
    presentation: &dyn Graphic3dStructureSource,
    index: usize,
) -> Option<Handle<dyn Graphic3dGroup>> {
    let structure = presentation.cstructure();
    let structure = structure.get()?;
    // Clone only the current handle; cloning Groups() each iteration would make updates quadratic.
    structure
        .cstructure_base()
        .fields
        .read()
        .groups
        .get(index)
        .cloned()
}
fn fresh_textured_fill_aspect<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
) -> Result<Handle<Graphic3dAspectFillArea3d>, Failure> {
    let shading = object.attributes().shading_aspect();
    let shading = shading.get().ok_or_else(|| null_object("AIS_TexturedShape::updateAttributes: null shading aspect; expected a context-linked or initialized drawer (native null dereference)"))?;
    let aspect = Handle::new(Graphic3dAspectFillArea3d::default());
    let source = shading.aspect();
    *aspect.aspects_base().fields.write() = source.aspects_base().fields.snapshot();
    object.textured_shape_base().fields.write().aspect = aspect.clone();
    if let Some((mode, factor, units)) = object.polygon_offsets() {
        aspect.set_polygon_offsets(mode, factor, units);
    }
    Ok(aspect)
}
fn refresh_textured_source<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
) -> Result<bool, Failure> {
    let base = object.textured_shape_base();
    if !base.fields.read().map_texture {
        return Ok(false);
    }
    let (source, description) = selected_textured_source(object);
    let texture = base.texture_factory.create(source)?;
    base.fields.write().texture = texture.clone();
    let modulate = base.fields.read().modulate;
    texture.texture_root_base().params.set_modulate(modulate);
    let description = completed_textured_description(&texture, description);
    let done = base.texture_factory.is_done(&texture);
    report_textured_source_failure(object, &description, done);
    Ok(done)
}
fn selected_textured_source<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
) -> (AisTexturedTextureSource, AsciiString) {
    let base = object.textured_shape_base();
    let pixels = base.texture_pix_map.read().clone();
    if !pixels.is_null() {
        return (
            AisTexturedTextureSource::PixMap(pixels),
            AsciiString::new(" (custom image)"),
        );
    }
    selected_textured_path(&base.fields.read())
}

fn apply_textured_fill_flags<S: AisTexturedShapeSource + ?Sized>(object: &S, has_texture: bool) {
    let fields = object.textured_shape_base().fields.read();
    fields.aspect.set_texture_map(fields.texture.clone().cast());
    fields.aspect.set_texture_map_on(has_texture);
    fields.aspect.set_draw_edges(fields.show_triangles);
}

fn completed_textured_description(
    texture: &Handle<dyn Graphic3dTexture2DSource>,
    description: AsciiString,
) -> AsciiString {
    if !description.as_bytes().is_empty() {
        return description;
    }
    let id = texture.get_id();
    AsciiString::from_bytes(
        [
            b" (predefined texture ".as_slice(),
            id.as_bytes(),
            b")".as_slice(),
        ]
        .concat(),
    )
}

fn report_textured_source_failure<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    description: &AsciiString,
    done: bool,
) {
    let base = object.textured_shape_base();
    if !done {
        let bytes = [
            b"Error: texture can not be loaded ".as_slice(),
            description.as_bytes(),
        ]
        .concat();
        base.messenger
            .send_ascii(&AsciiString::from_bytes(&bytes), Gravity::Fail);
    }
}

fn selected_textured_path(
    fields: &super::textured_shape::AisTexturedShapeFields,
) -> (AisTexturedTextureSource, AsciiString) {
    if fields.predefined_texture != Graphic3dNameOfTexture2D::Unknown {
        return (
            AisTexturedTextureSource::Predefined(fields.predefined_texture),
            AsciiString::default(),
        );
    }
    (
        AisTexturedTextureSource::File(AsciiString::from_bytes(
            super::textured_shape_source::native_textured_cstring_prefix(&fields.texture_file),
        )),
        textured_file_description(&fields.texture_file),
    )
}
fn textured_file_description(file: &AsciiString) -> AsciiString {
    let bytes = [b" (".as_slice(), file.as_bytes(), b")".as_slice()].concat();
    AsciiString::from_bytes(&bytes)
}
