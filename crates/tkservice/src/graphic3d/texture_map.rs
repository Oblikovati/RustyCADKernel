//! `Graphic3d_TextureMap` (a texture mapped on a primitive) and the shared helpers of the
//! predefined texture tables (`TextureName`, named constructors).
use super::texture_root::graphic3d_textures_folder;
use super::{
    Graphic3dLevelOfTextureAnisotropy, Graphic3dTextureRoot, Graphic3dTextureRootBase,
    Graphic3dTypeOfTexture, Graphic3dTypeOfTextureFilter,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::tcollection::AsciiString;

/// A texture mapped on primitives (`Graphic3d_TextureMap`): a marker over the root trait.
pub trait Graphic3dTextureMap: Graphic3dTextureRoot {}

rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dTextureMap,
    "Graphic3d_TextureMap",
    dyn Graphic3dTextureRoot
);

/// The members of `Graphic3d_TextureMap`, all on the texture parameters.
pub trait Graphic3dTextureMapMembers: Graphic3dTextureMap {
    /// Trilinear filtering; `texture.enable_smooth()`.
    fn enable_smooth(&self) {
        self.texture_root_base()
            .params
            .set_filter(Graphic3dTypeOfTextureFilter::Trilinear);
    }
    /// Whether the filter is not nearest; `texture.is_smoothed()`.
    fn is_smoothed(&self) -> bool {
        self.texture_root_base().params.filter() != Graphic3dTypeOfTextureFilter::Nearest
    }
    /// Nearest filtering; `texture.disable_smooth()`.
    fn disable_smooth(&self) {
        self.texture_root_base()
            .params
            .set_filter(Graphic3dTypeOfTextureFilter::Nearest);
    }
    /// Modulates the material color; `texture.enable_modulate()`.
    fn enable_modulate(&self) {
        self.texture_root_base().params.set_modulate(true);
    }
    /// Stops modulating the material color; `texture.disable_modulate()`.
    fn disable_modulate(&self) {
        self.texture_root_base().params.set_modulate(false);
    }
    /// Whether the material color is modulated; `texture.is_modulate()`.
    fn is_modulate(&self) -> bool {
        self.texture_root_base().params.is_modulate()
    }
    /// Repeats the coordinates; `texture.enable_repeat()`.
    fn enable_repeat(&self) {
        self.texture_root_base().params.set_repeat(true);
    }
    /// Clamps the coordinates; `texture.disable_repeat()`.
    fn disable_repeat(&self) {
        self.texture_root_base().params.set_repeat(false);
    }
    /// Whether the coordinates repeat; `texture.is_repeat()`.
    fn is_repeat(&self) -> bool {
        self.texture_root_base().params.is_repeat()
    }
    /// The anisotropy level; `texture.aniso_filter()`.
    fn aniso_filter(&self) -> Graphic3dLevelOfTextureAnisotropy {
        self.texture_root_base().params.aniso_filter()
    }
    /// Sets the anisotropy level; `texture.set_aniso_filter(Graphic3dLevelOfTextureAnisotropy::Fast)`.
    fn set_aniso_filter(&self, level: Graphic3dLevelOfTextureAnisotropy) {
        self.texture_root_base().params.set_aniso_filter(level);
    }
}
impl<T: Graphic3dTextureMap + ?Sized> Graphic3dTextureMapMembers for T {}

/// The file of a predefined texture; a name outside the table (the `UNKNOWN` value) reads
/// past the native array, here it raises `Standard_OutOfRange`.
pub(crate) fn predefined_file(
    table: &[&'static str],
    index: i32,
    class: &str,
) -> Result<&'static str, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|slot| table.get(slot).copied())
        .ok_or_else(|| {
            out_of_range(&format!(
                "{class}: no predefined texture file; got name {index}, expected [0, {}]",
                table.len() - 1
            ))
        })
}

/// `TextureName(theRank)`: the file name between the prefix of `prefix_length` bytes and the
/// extension; `Standard_OutOfRange` "BAD index of texture" outside `1..=table.len()`.
pub(crate) fn predefined_texture_name(
    table: &[&str],
    rank: i32,
    prefix_length: i32,
) -> Result<AsciiString, Failure> {
    let count = table.len() as i32;
    if rank < 1 || rank > count {
        return Err(out_of_range(&format!(
            "BAD index of texture; got rank {rank}, expected [1, {count}]"
        )));
    }
    let file_name = AsciiString::from(table[(rank - 1) as usize]);
    let dot = file_name.search_from_end(".");
    file_name.sub_string(prefix_length + 1, dot - 1)
}

/// The base of a texture built from a predefined name: the file in the textures folder and
/// the id `<class>_<file>`. Native order: the root constructor draws a counter id first.
pub(crate) fn predefined_base(
    file: &str,
    class: &str,
    kind: Graphic3dTypeOfTexture,
) -> Result<Graphic3dTextureRootBase, Failure> {
    let base = Graphic3dTextureRootBase::from_file(&AsciiString::from(file), kind);
    let folder = graphic3d_textures_folder()?;
    let mut fields = base.fields.write();
    fields.path.set_trek(folder.as_bytes());
    fields.tex_id = AsciiString::from(format!("{class}_{file}").as_str());
    drop(fields);
    Ok(base)
}
