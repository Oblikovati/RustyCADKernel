//! Genuine BRep selection owners retain shape identity and the inherited selectable-owner state.
use super::{StdSelectShapeFields, StdSelectShapeRef, StdSelectShapeSource};
use crate::select_mgr::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use std::sync::atomic::{AtomicI32, Ordering};
/// Actual derived fields; `StdSelectBRepOwnerBase::new(&shape,&object,7,true)`.
pub struct StdSelectBRepOwnerBase {
    /// Highlight child is released before derived geometry and the base owner.
    pub presentation: SharedState<Handle<dyn StdSelectShapeSource>>,
    /// Native owned shape member, independent of the selectable pointer.
    pub shape: SharedState<StdSelectShapeFields>,
    /// Native highlight mode initialized to zero.
    pub highlight_mode: AtomicI32,
    /// The actual inherited owner subobject.
    pub owner: SelectMgrEntityOwnerBase,
}
impl StdSelectBRepOwnerBase {
    /// Construct the inherited owner first, then copy topology; `Self::new(&shape,&object,7,true)`.
    pub fn new(
        shape: &Shape,
        selectable: &Handle<dyn SelectMgrSelectableObject>,
        priority: i32,
        decomposed: bool,
    ) -> Self {
        let owner = SelectMgrEntityOwnerBase::new(selectable, priority);
        let shape = SharedState::new(StdSelectShapeFields {
            shape: shape.clone(),
        });
        owner.fields.write().from_decomposition = decomposed;
        Self {
            presentation: SharedState::default(),
            shape,
            highlight_mode: AtomicI32::new(0),
            owner,
        }
    }
}
/// Genuine derived runtime interface; `source.brep_owner_base()`.
pub trait StdSelectBRepOwnerSource: SelectMgrEntityOwnerSource {
    /// Borrow actual protected fields; `source.brep_owner_base().highlight_mode.load(Ordering::Relaxed)`.
    fn brep_owner_base(&self) -> &StdSelectBRepOwnerBase;
}
impl rustycad_tkernel::standard::RuntimeTyped for dyn StdSelectBRepOwnerSource {
    const TYPE_NAME: &'static str =
        <StdSelectBRepOwner as rustycad_tkernel::standard::RuntimeTyped>::TYPE_NAME;
    fn static_type() -> &'static rustycad_tkernel::standard::RuntimeType {
        // The application dispatch protocol shares the one genuine native class descriptor.
        <StdSelectBRepOwner as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn project_from(object: &dyn rustycad_tkernel::standard::Transient) -> Option<&Self> {
        // Registered casters project the actual subclass base without allocating a facade.
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
/// Nonvirtual BRep owner operations preserve the pinned inverted HasHilightMode predicate; `owner.reset_hilight_mode()`.
pub trait StdSelectBRepOwnerMembers: StdSelectBRepOwnerSource {
    /// Borrow the native geometry field without retaining topology; `let reference=owner.shape_ref();`.
    fn shape_ref(&self) -> StdSelectShapeRef<'_> {
        let fields = &self.brep_owner_base().shape;
        StdSelectShapeRef { fields }
    }
    /// Check whether native geometry is set; `owner.has_shape()`.
    fn has_shape(&self) -> bool {
        let reference = self.shape_ref();
        !reference.is_null()
    }
    /// Preserve OCCT 3d097a0's exact `myCurMode == -1`; `owner.has_hilight_mode()`.
    fn has_hilight_mode(&self) -> bool {
        let mode = self.hilight_mode();
        mode == -1
    }
    /// Store the native integer without validation; `owner.set_hilight_mode(17)`.
    fn set_hilight_mode(&self, mode: i32) {
        let field = &self.brep_owner_base().highlight_mode;
        field.store(mode, Ordering::Relaxed);
    }
    /// Reset the mode to native minus one; `owner.reset_hilight_mode()`.
    fn reset_hilight_mode(&self) {
        let mode = -1;
        self.set_hilight_mode(mode);
    }
    /// Return current mode, including negative values; `owner.hilight_mode()`.
    fn hilight_mode(&self) -> i32 {
        let field = &self.brep_owner_base().highlight_mode;
        field.load(Ordering::Relaxed)
    }
}
impl<S: StdSelectBRepOwnerSource + ?Sized> StdSelectBRepOwnerMembers for S {}
/// Concrete native BRep owner; `let owner=StdSelectBRepOwner::with_shape(&shape,7,true);`.
pub struct StdSelectBRepOwner {
    transient: StandardTransient,
    base: StdSelectBRepOwnerBase,
}
rustycad_tkernel::impl_transient!(StdSelectBRepOwner,"StdSelect_BRepOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn StdSelectBRepOwnerSource,dyn SelectMgrEntityOwnerSource]);
impl StdSelectBRepOwner {
    /// Construct a null-shape priority owner; `Self::new(5)`.
    pub fn new(priority: i32) -> Handle<Self> {
        let shape = Shape::default();
        Self::with_shape(&shape, priority, false)
    }
    /// Retain geometry with native decomposition state; `Self::with_shape(&shape,7,true)`.
    pub fn with_shape(shape: &Shape, priority: i32, decomposed: bool) -> Handle<Self> {
        let selectable = Handle::default();
        Self::with_selectable(shape, &selectable, priority, decomposed)
    }
    /// Retain geometry while borrowing the selectable identity; `Self::with_selectable(&shape,&object,7,true)`.
    pub fn with_selectable(
        shape: &Shape,
        selectable: &Handle<dyn SelectMgrSelectableObject>,
        priority: i32,
        decomposed: bool,
    ) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: StdSelectBRepOwnerBase::new(shape, selectable, priority, decomposed),
        });
        select_mgr_entity_owner_bind(&object.cast());
        object
    }
}
impl StdSelectBRepOwnerSource for StdSelectBRepOwner {
    fn brep_owner_base(&self) -> &StdSelectBRepOwnerBase {
        // Runtime projections borrow the real owned subclass state.
        &self.base
    }
}
