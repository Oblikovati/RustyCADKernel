//! Abstract AIS object state, native context links and virtual object behavior.
use super::{AisDragAction, AisInteractiveContext, AisKindOfInteractive};
use crate::v3d::V3dViewSource;
use crate::{
    prs_mgr::{
        PrsMgrPresentableObjectBase, PrsMgrPresentableObjectMembers, PrsMgrTypeOfPresentation3d,
    },
    select_mgr::{
        SelectMgrEntityOwnerSource, SelectMgrSelectableObject, SelectMgrSelectableObjectBase,
    },
};
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Handle, SharedState, StandardTransient, WeakHandle},
};
/// Native protected object fields; `object.interactive_base().fields.read().context.is_null()`.
#[derive(Default)]
pub struct AisInteractiveObjectFields {
    /// `myCTXPtr`: never owns the context.
    pub context: WeakHandle<AisInteractiveContext>,
    /// `myOwner`: application-specific retained owner.
    pub owner: Handle<StandardTransient>,
}
/// Base state embedded by concrete interactive objects; `AisInteractiveObjectBase::new(kind)`.
pub struct AisInteractiveObjectBase {
    /// Inherited presentable state.
    pub presentable: PrsMgrPresentableObjectBase,
    /// Inherited selection state.
    pub selectable: SelectMgrSelectableObjectBase,
    /// Context and application owner.
    pub fields: SharedState<AisInteractiveObjectFields>,
}
impl AisInteractiveObjectBase {
    /// Construct inherited states with the requested presentation kind; `AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView)`.
    pub fn new(kind: PrsMgrTypeOfPresentation3d) -> Self {
        Self {
            presentable: PrsMgrPresentableObjectBase::new(kind),
            selectable: SelectMgrSelectableObjectBase::default(),
            fields: SharedState::default(),
        }
    }
}
/// Native abstract interactive object; rendering and selection remain subclass responsibilities.
pub trait AisInteractiveObject: SelectMgrSelectableObject {
    /// Embedded context/owner state; `object.interactive_base()`.
    fn interactive_base(&self) -> &AisInteractiveObjectBase;
    /// The default unknown category; `object.kind_of_interactive()`.
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::None
    }
    /// The default unspecified signature; `object.signature()`.
    fn signature(&self) -> i32 {
        -1
    }
    /// Assign before linking the drawer; clearing retains the existing link.
    /// Example: `object.set_context(&context)`.
    fn set_context(&self, context: &Handle<AisInteractiveContext>) {
        ais_interactive_object_set_context(self, context);
    }
    /// Default objects decline all drag actions; `object.process_dragging(&ctx,&view,&owner,from,to,AisDragAction::Start)`.
    fn process_dragging(
        &self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        from: Vec2<i32>,
        to: Vec2<i32>,
        action: AisDragAction,
    ) -> bool {
        let _ = (context, view, owner, from, to, action);
        false
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisInteractiveObject,
    "AIS_InteractiveObject",
    dyn SelectMgrSelectableObject
);

/// The native `AIS_InteractiveObject::SetContext` body, for the overrides that extend it:
/// the context (unchanged when it is the same), then the drawer linked to its default drawer.
/// Example: `ais_interactive_object_set_context(&*object, &context)`.
pub fn ais_interactive_object_set_context<O: AisInteractiveObject + ?Sized>(
    object: &O,
    context: &Handle<AisInteractiveContext>,
) {
    let previous = object
        .interactive_base()
        .fields
        .read()
        .context
        .object_address();
    if previous == context.object_address() {
        return;
    }
    object.interactive_base().fields.write().context = context.downgrade();
    if let Some(context) = context.get() {
        object
            .attributes()
            .link_to(&context.fields.read().default_drawer);
    }
}
