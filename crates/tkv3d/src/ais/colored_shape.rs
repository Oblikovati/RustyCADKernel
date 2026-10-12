//! Martensite's owned AIS_ColoredShape lifecycle; geometry computation remains partial.
use super::{AisColoredDrawer, AisShapeBase, AisShapeDrawerMap, AisShapeSource};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};

/// Genuine inherited shape and custom-node ownership; `AisColoredShapeBase::new(&shape)`.
pub struct AisColoredShapeBase {
    /// `myShapeColors`: aliases borrow this container rather than a copied map.
    /// Declared first because Rust drops fields forward; native destroys this derived map before its parent.
    pub shape_colors: AisShapeDrawerMap,
    /// The actual AIS_Shape parent subobject, alive throughout custom drawer destruction.
    pub shape: AisShapeBase,
}
impl AisColoredShapeBase {
    /// Initialize native parent defaults and dedicated-line alias policy; `AisColoredShapeBase::new(&shape)`.
    pub fn new(shape: &Shape) -> Self {
        let base = Self {
            shape: AisShapeBase::new(shape),
            shape_colors: AisShapeDrawerMap::default(),
        };
        colored_shape_link_line_aspects(&base);
        base
    }
}
fn colored_shape_link_line_aspects(base: &AisColoredShapeBase) {
    // OCCT 3d097a0 disables these dedicated aspects by sharing the actual line handle.
    let drawer = base
        .shape
        .interactive
        .presentable
        .fields
        .read()
        .drawer
        .clone();
    let line = drawer.line_aspect();
    drawer.set_free_boundary_aspect(line.clone());
    drawer.set_un_free_boundary_aspect(line.clone());
    drawer.set_seen_line_aspect(line.clone());
    drawer.set_face_boundary_aspect(line);
}
/// Native virtual lifecycle dispatch; `object.custom_aspects(&subshape)?`.
pub trait AisColoredShapeSource: AisShapeSource {
    /// Borrow owned native subclass fields; `object.colored_shape_base().shape_colors.len()`.
    fn colored_shape_base(&self) -> &AisColoredShapeBase;
    /// Reuse or allocate a linked custom drawer, then invalidate presentations; `object.custom_aspects(&shape)?`.
    fn custom_aspects(&self, shape: &Shape) -> Result<Handle<AisColoredDrawer>, Failure> {
        // A native subclass can override this virtual, while explicitly invoking the core body.
        ais_colored_shape_custom_aspects(self, shape)
    }
    /// Clear populated custom fields before invalidating; `object.clear_custom_aspects()?`.
    fn clear_custom_aspects(&self) -> Result<(), Failure> {
        // Empty clear must not change presentation status.
        ais_colored_shape_clear_custom_aspects(self);
        Ok(())
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisColoredShapeSource,
    "AIS_ColoredShape",
    dyn AisShapeSource
);
/// Invoke the native base CustomAspects body without virtual redispatch; `ais_colored_shape_custom_aspects(object,&shape)?`.
pub fn ais_colored_shape_custom_aspects<S: AisColoredShapeSource + ?Sized>(
    object: &S,
    shape: &Shape,
) -> Result<Handle<AisColoredDrawer>, Failure> {
    let map = &object.colored_shape_base().shape_colors;
    let (drawer, created) = map.custom_aspects(shape, &object.attributes())?;
    if created {
        object.set_to_update_all();
    }
    Ok(drawer)
}
/// Invoke the native base ClearCustomAspects body without virtual redispatch; `ais_colored_shape_clear_custom_aspects(object)`.
pub fn ais_colored_shape_clear_custom_aspects<S: AisColoredShapeSource + ?Sized>(object: &S) {
    let map = &object.colored_shape_base().shape_colors;
    if map.clear_custom_aspects() {
        // Unlike UnsetCustomAspects, native Clear releases node ownership before SetToUpdate.
        object.set_to_update_all();
    }
}
