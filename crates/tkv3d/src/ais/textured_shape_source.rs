//! Virtual native texture-source dispatch and genuine const pixmap handle-reference storage.
use super::AisInteractiveObjectMembers;
use super::{shape::AisShapeSource, textured_shape::AisTexturedShapeBase};
use rustycad_tkernel::{
    standard::{Failure, HandleReference},
    tcollection::AsciiString,
};
use rustycad_tkservice::image::ImagePixMap;

/// Native subclassable texture-source operations; `object.set_texture_file_name(&file)?`.
pub trait AisTexturedShapeSource: AisShapeSource {
    /// Native protected fields; `object.textured_shape_base().fields.read().map_texture`.
    fn textured_shape_base(&self) -> &AisTexturedShapeBase;
    /// Replace the filename and release previous pixmap first; `object.set_texture_file_name(&"7".into())?`.
    fn set_texture_file_name(&self, file: &AsciiString) -> Result<(), Failure> {
        // Keep the virtual hook separate from its native failure-ordered base body.
        super::textured_shape_sources::ais_textured_shape_set_texture_file_name(self, file)
    }
    /// Retain the exact pixmap without changing its row-order flag; `object.set_texture_pix_map(&pixmap)`.
    fn set_texture_pix_map(&self, pixmap: &dyn HandleReference<ImagePixMap>) {
        // Forward the exact handle rather than copying image contents.
        super::textured_shape_sources::ais_textured_shape_set_texture_pix_map(self, pixmap);
    }
}
// OCCT 3d097a0 defines one concrete descriptor; the source view must not add an ancestor.
impl rustycad_tkernel::standard::RuntimeTyped for dyn AisTexturedShapeSource {
    const TYPE_NAME: &'static str = "AIS_TexturedShape";
    fn static_type() -> &'static rustycad_tkernel::standard::RuntimeType {
        // Genuine source casts share the concrete class descriptor and its AIS_Shape parent.
        <super::AisTexturedShape as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn project_from(object: &dyn rustycad_tkernel::standard::Transient) -> Option<&Self> {
        // Retain application subclass overrides through the actual interface projection.
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}

/// Native nonvirtual getters/operations shared by concrete classes and application subclasses.
/// Example: `object.texture_pix_map().read().object_address()` inspects the live handle field.
pub trait AisTexturedShapeMembers: AisTexturedShapeSource {
    /// Refresh the native currently selected presentation; `object.update_attributes()?` requires an existing context presentation.
    fn update_attributes(&self) -> Result<(), Failure> {
        let presentation = self.presentation()?;
        super::textured_shape_aspects::ais_textured_shape_update_attributes(self, &presentation)
    }
    /// Read mapping enable state; `object.texture_map_state()`.
    fn texture_map_state(&self) -> bool {
        let fields = self.textured_shape_base().fields.read();
        fields.map_texture
    }
    /// Enable mapping without recomputing presentations; `object.set_texture_map_on()`.
    fn set_texture_map_on(&self) {
        let mut fields = self.textured_shape_base().fields.write();
        fields.map_texture = true;
    }
    /// Disable mapping without dropping the old retained texture; `object.set_texture_map_off()`.
    fn set_texture_map_off(&self) {
        let mut fields = self.textured_shape_base().fields.write();
        fields.map_texture = false;
    }
    /// Borrow the actual current filename allocation; `object.texture_file().read()?.as_bytes()`.
    fn texture_file(&self) -> super::AisTexturedShapeFileRef {
        // Capture once, so later allocation replacement cannot silently retarget an old alias.
        super::AisTexturedShapeFileRef::new(&self.textured_shape_base().fields)
    }
    /// Borrow genuine native handle storage across assignments; `let source=object.texture_pix_map(); object.set_texture_pix_map(&next);`.
    fn texture_pix_map(&self) -> super::textured_shape_pixmap_ref::AisTexturedShapePixMapRef<'_> {
        super::textured_shape_pixmap_ref::AisTexturedShapePixMapRef::new(
            &self.textured_shape_base().texture_pix_map,
        )
    }
    /// Enable modulation without recomputing; `object.enable_texture_modulate()`.
    fn enable_texture_modulate(&self) {
        let mut fields = self.textured_shape_base().fields.write();
        fields.modulate = true;
    }
    /// Disable modulation without recomputing; `object.disable_texture_modulate()`.
    fn disable_texture_modulate(&self) {
        let mut fields = self.textured_shape_base().fields.write();
        fields.modulate = false;
    }
    /// Read native modulation state; `object.texture_modulate()`.
    fn texture_modulate(&self) -> bool {
        let fields = self.textured_shape_base().fields.read();
        fields.modulate
    }
    /// Read native triangle-edge request; `object.show_triangles()`.
    fn show_triangles(&self) -> bool {
        let fields = self.textured_shape_base().fields.read();
        fields.show_triangles
    }
    /// Store the triangle-edge request without recomputing; `object.set_show_triangles(true)`.
    fn set_show_triangles(&self, show: bool) {
        let mut fields = self.textured_shape_base().fields.write();
        fields.show_triangles = show;
    }
}
impl<T: AisTexturedShapeSource + ?Sized> AisTexturedShapeMembers for T {}

// Native TextureFile and the file constructor both consume myTextureFile.ToCString().
pub(super) fn native_textured_cstring_prefix(
    file: &rustycad_tkernel::tcollection::AsciiString,
) -> &[u8] {
    let bytes = file.as_bytes();
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..end]
}
