//! Borrow the actual native AIS_Shape topology field across assignments.
use super::{AisShapeFields, AisShapeSource};
use crate::{prs_mgr::PrsMgrPresentableObjectFields, prs3d::Prs3dDrawer};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Handle, HandleFieldRef, SharedState};
/// Non-owning live `Shape()` reference; `let alias=object.shape_ref();`.
#[derive(Clone, Copy)]
pub struct AisShapeRef<'a> {
    fields: &'a SharedState<AisShapeFields>,
}
impl AisShapeRef<'_> {
    /// Observe current topology identity/orientation/location; `alias.is_equal(&shape)`.
    pub fn is_equal(&self, shape: &Shape) -> bool {
        let fields = self.fields.read();
        fields.shape.is_equal(shape)
    }
    /// Compare actual member slots without retaining topology; `alias.is_same_reference(&other)`.
    pub fn is_same_reference(&self, other: &Self) -> bool {
        // The shared field storage remains stable throughout its owning parent's lifetime.
        std::ptr::eq(self.fields, other.fields)
    }
    /// Explicitly retain an owning snapshot; `let copied=alias.clone_shape();`.
    pub fn clone_shape(&self) -> Shape {
        let fields = self.fields.read();
        fields.shape.clone()
    }
}
/// Nonvirtual parent topology methods; `object.set_shape(&replacement)`.
pub trait AisShapeMembers: AisShapeSource {
    /// Borrow the native const topology member rather than return a copied key; `object.shape_ref()`.
    fn shape_ref(&self) -> AisShapeRef<'_> {
        // Retained aliases observe later SetShape assignments without holding a lock.
        AisShapeRef {
            fields: &self.shape_base().fields,
        }
    }
    /// Borrow the actual inherited drawer member; `let alias=object.attributes_ref();`.
    fn attributes_ref(&self) -> HandleFieldRef<'_, PrsMgrPresentableObjectFields, Prs3dDrawer> {
        // Like native Attributes(), this alias retains neither current nor replaced target.
        HandleFieldRef::new(
            &self.shape_base().interactive.presentable.fields,
            shape_drawer_slot,
        )
    }
    /// Assign before enabling lazy bounds recomputation; `object.set_shape(&shape)`.
    fn set_shape(&self, shape: &Shape) {
        let mut fields = self.shape_base().fields.write();
        fields.shape = shape.clone();
        fields.recompute_bounds = true;
    }
}
impl<S: AisShapeSource + ?Sized> AisShapeMembers for S {}

fn shape_drawer_slot(fields: &PrsMgrPresentableObjectFields) -> &Handle<Prs3dDrawer> {
    // Select the actual native parent member rather than create a temporary owning getter.
    &fields.drawer
}
