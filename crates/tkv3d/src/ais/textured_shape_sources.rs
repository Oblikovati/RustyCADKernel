//! Native source mutation ordering and predefined-index fallback, with no codec promises.
use super::textured_shape_source::AisTexturedShapeSource;
use rustycad_tkernel::{
    message::{Gravity, Messenger},
    standard::{Failure, HandleReference},
    tcollection::AsciiString,
};
use rustycad_tkservice::{
    graphic3d::{Graphic3dNameOfTexture2D, Graphic3dTexture2DBase},
    image::ImagePixMap,
};

/// Native base body for virtual filename assignment; `ais_textured_shape_set_texture_file_name(&object,&file)?`.
pub fn ais_textured_shape_set_texture_file_name<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    file: &AsciiString,
) -> Result<(), Failure> {
    let base = object.textured_shape_base();
    base.texture_pix_map.clear()?;
    assign_textured_file_after_retirement(object, file)
}
pub(super) fn assign_textured_file_after_retirement<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    file: &AsciiString,
) -> Result<(), Failure> {
    let base = object.textured_shape_base();
    let name = textured_predefined_name(file, &base.messenger)?;
    let mut fields = base.fields.write();
    fields.predefined_texture = name;
    if name == Graphic3dNameOfTexture2D::Unknown {
        fields.texture_file.assign_from(file);
        return Ok(());
    }
    fields.texture_file.clear();
    Ok(())
}
/// Native base body retains the exact source handle; `ais_textured_shape_set_texture_pix_map(&object,&pixmap)`.
pub fn ais_textured_shape_set_texture_pix_map<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    pixmap: &dyn HandleReference<ImagePixMap>,
) {
    reset_textured_file_for_pixmap(object);
    rustycad_tkernel::standard::raise_native_failure(object.textured_shape_base().texture_pix_map.assign(pixmap));
}
pub(super) fn reset_textured_file_for_pixmap<S: AisTexturedShapeSource + ?Sized>(object: &S) {
    let base = object.textured_shape_base();
    let mut fields = base.fields.write();
    fields.texture_file = AsciiString::default();
    fields.predefined_texture = Graphic3dNameOfTexture2D::Unknown;
    // The caller performs native Assign only after releasing the source-field guard.
    drop(fields);
}
fn textured_predefined_name(
    file: &AsciiString,
    messenger: &Messenger,
) -> Result<Graphic3dNameOfTexture2D, Failure> {
    if !file.is_integer_value() {
        return Ok(Graphic3dNameOfTexture2D::Unknown);
    }
    let index = file.integer_value()?;
    if index >= 0 && index < Graphic3dTexture2DBase::number_of_textures() {
        return Ok(TEXTURED_PREDEFINED_NAMES[index as usize]);
    }
    let bytes = [
        b"Error: texture with ID ".as_slice(),
        file.as_bytes(),
        b" is undefined. Texture 0 will be used instead.".as_slice(),
    ]
    .concat();
    messenger.send_ascii(&AsciiString::from_bytes(&bytes), Gravity::Fail);
    Ok(Graphic3dNameOfTexture2D::Matra)
}
const TEXTURED_PREDEFINED_NAMES: [Graphic3dNameOfTexture2D; 22] = {
    use Graphic3dNameOfTexture2D::*;
    [
        Matra,
        Alienskin,
        BlueRock,
        BluewhitePaper,
        Brushed,
        Bubbles,
        Bump,
        Cast,
        Chipbd,
        Clouds,
        Flesh,
        Floor,
        Galvnisd,
        Grass,
        Aluminum,
        Rock,
        Knurl,
        Maple,
        Marble,
        Mottled,
        Rain,
        Chess,
    ]
};
