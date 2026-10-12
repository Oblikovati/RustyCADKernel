//! Nonvirtual ColoredShape map aliases and ordered custom-drawer removal.
use super::{AisColoredDrawer, AisColoredShapeSource, AisShapeDrawerMapMut, AisShapeDrawerMapRef};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
/// Native nonvirtual lifecycle members; `object.unset_custom_aspects(&shape,false)?`.
pub trait AisColoredShapeMembers: AisColoredShapeSource {
    /// Borrow the actual const container, preserving subsequent changes; `let map=object.custom_aspects_map();`.
    fn custom_aspects_map(&self) -> AisShapeDrawerMapRef<'_> {
        // The reference neither copies nodes nor retains drawer/key ownership.
        self.colored_shape_base().shape_colors.as_const()
    }
    /// Borrow the actual mutable container without automatic invalidation; `object.change_custom_aspects_map().clear(false)`.
    fn change_custom_aspects_map(&self) -> AisShapeDrawerMapMut<'_> {
        // Raw native map mutation intentionally bypasses SetToUpdate.
        self.colored_shape_base().shape_colors.as_mut()
    }
    /// Invalidate before replacement/removal, leaving missing keys untouched; `object.unset_custom_aspects(&shape,true)?`.
    fn unset_custom_aspects(&self, shape: &Shape, unregister: bool) -> Result<(), Failure> {
        let map = &self.colored_shape_base().shape_colors;
        if !map.is_bound(shape) {
            return Ok(());
        }
        // Native ordering is observable when removal destroys a custom drawer.
        self.set_to_update_all();
        if unregister {
            map.unbind(shape);
            return Ok(());
        }
        let drawer = colored_shape_replacement_drawer(self);
        map.bind(shape, &drawer)?;
        Ok(())
    }
}
impl<S: AisColoredShapeSource + ?Sized> AisColoredShapeMembers for S {}

fn colored_shape_replacement_drawer<S: AisColoredShapeSource + ?Sized>(
    object: &S,
) -> Handle<AisColoredDrawer> {
    // Drop the temporary owning Attributes convenience before old-value destruction.
    // Native passes its actual const handle member and has no extra transient refcount.
    let link = object.attributes();
    Handle::new(AisColoredDrawer::new(&link))
}
