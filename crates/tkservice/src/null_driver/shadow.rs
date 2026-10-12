//! OpenGl shadow groups belong to the retained parent, never the removable shadow.
use super::NullCStructure;
use crate::graphic3d::{Graphic3dCStructure, Graphic3dStructureManager};
use rustycad_tkernel::standard::{Failure, Handle};
pub(super) fn null_structure_shadow(
    source: &NullCStructure,
    manager: &Handle<Graphic3dStructureManager>,
) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
    let parent = if source.shadow_parent.read().is_null() {
        source.this.read().upgrade()
    } else {
        source.shadow_parent.read().clone()
    };
    let shadow = NullCStructure::new(manager)?;
    inherit_shadow_camera_fields(&shadow, &parent);
    *shadow.shadow_parent.write() = parent;
    Ok(shadow.cast())
}
fn inherit_shadow_camera_fields(shadow: &NullCStructure, parent: &NullCStructure) {
    let parent = parent.base.fields.read();
    let mut fields = shadow.base.fields.write();
    // OCCT3d097a0 OpenGl_StructureShadow copies bounds and placement, with empty own groups.
    fields.is_infinite = parent.is_infinite;
    fields.bnd_box = parent.bnd_box;
    fields.trsf = parent.trsf.clone();
    fields.trsf_pers = parent.trsf_pers.clone();
}
