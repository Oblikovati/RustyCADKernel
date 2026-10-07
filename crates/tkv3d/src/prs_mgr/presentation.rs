//! `PrsMgr_Presentation`: the structure of one display mode of a presentable object, made by a
//! presentation manager. It refers to its object weakly (a raw pointer natively).
use super::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectMembers as _, PrsMgrPresentationManager,
    PrsMgrTypeOfPresentation3d,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::message::{Gravity, Message};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dStructure, Graphic3dStructureBase, Graphic3dStructureDisplay as _,
    Graphic3dStructureMembers as _, Graphic3dStructureNetwork as _, Graphic3dStructureSource,
    Graphic3dTypeOfConnection, Graphic3dTypeOfStructure, graphic3d_structure_bind,
    graphic3d_structure_clear, graphic3d_structure_display, graphic3d_structure_dump_json,
    graphic3d_structure_erase,
};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_integer, field_pointer};
use std::io;

/// The native `BeforeHighlightState`: what `Unhighlight` restores.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BeforeHighlightState {
    Empty = 0,
    Hidden = 1,
    Visible = 2,
}

/// The mutable fields of a presentation.
#[derive(Clone, Copy)]
struct PresentationState {
    before_highlight_state: BeforeHighlightState,
    must_be_updated: bool,
}

/// The structure of one display mode of a presentable object.
///
/// Example: `let prs = manager.presentation(&object, 0, true, &Handle::default())?;`.
pub struct PrsMgrPresentation {
    transient: StandardTransient,
    base: Graphic3dStructureBase,
    presentation_manager: Handle<PrsMgrPresentationManager>,
    presentable_object: SharedState<WeakHandle<dyn PrsMgrPresentableObject>>,
    mode: i32,
    state: SharedState<PresentationState>,
}
rustycad_tkernel::impl_transient!(PrsMgrPresentation, "PrsMgr_Presentation", dyn Graphic3dStructureSource, transient,
    interfaces: [dyn Graphic3dStructureSource]);

impl PrsMgrPresentation {
    /// The presentation of `object` in `mode` (protected natively; the manager makes it): a
    /// computed visual for a projector-dependent object, the object as owner and its mutable
    /// flag. Example: `PrsMgrPresentation::new(&manager, &object, 0)?`.
    pub fn new(
        manager: &Handle<PrsMgrPresentationManager>,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
    ) -> Result<Handle<Self>, Failure> {
        let structure_manager = manager
            .get()
            .map(PrsMgrPresentationManager::structure_manager)
            .unwrap_or_default();
        let prs = Handle::new(Self {
            transient: StandardTransient::default(),
            base: Graphic3dStructureBase::new(&structure_manager, &Handle::default())?,
            presentation_manager: manager.clone(),
            presentable_object: SharedState::new(object.downgrade()),
            mode,
            state: SharedState::new(PresentationState {
                before_highlight_state: BeforeHighlightState::Empty,
                must_be_updated: false,
            }),
        });
        graphic3d_structure_bind(&prs.clone().cast());
        if let Some(value) = object.get() {
            if value.type_of_presentation3d() == PrsMgrTypeOfPresentation3d::ProjectorDependent {
                prs.set_visual(Graphic3dTypeOfStructure::Computed);
            }
            prs.set_owner(&object.clone().cast());
            prs.set_mutable(value.is_mutable());
        }
        Ok(prs)
    }
    /// `Presentation` (deprecated): the presentation itself; `prs.presentation()`.
    pub fn presentation(&self) -> &Self {
        self
    }
    /// `PresentationManager`; `prs.presentation_manager()`.
    pub fn presentation_manager(&self) -> Handle<PrsMgrPresentationManager> {
        self.presentation_manager.clone()
    }
    /// `SetUpdateStatus`; `prs.set_update_status(true)`.
    pub fn set_update_status(&self, update_status: bool) {
        self.state.write().must_be_updated = update_status;
    }
    /// `MustBeUpdated`; `prs.must_be_updated()`.
    pub fn must_be_updated(&self) -> bool {
        self.state.read().must_be_updated
    }
    /// `Mode`; `prs.mode()`.
    pub fn mode(&self) -> i32 {
        self.mode
    }
    /// The presentable object, null once it is gone; `prs.presentable_object()`.
    pub fn presentable_object(&self) -> Handle<dyn PrsMgrPresentableObject> {
        self.presentable_object.read().upgrade()
    }
    /// Forgets the object (its destructor clears `myPresentableObject`).
    pub(crate) fn forget_presentable_object(&self) {
        *self.presentable_object.write() = WeakHandle::default();
    }
    /// Whether the manager shows `object` in the mode of this presentation (displayed or
    /// highlighted), the test of `ToBeUpdated` and `UpdatePresentations`.
    pub(crate) fn is_shown_by_manager(&self, object: &Handle<dyn PrsMgrPresentableObject>) -> bool {
        self.presentation_manager.get().is_some_and(|m| {
            m.is_displayed(object, self.mode) || m.is_highlighted(object, self.mode)
        })
    }
    /// The native `Graphic3d_Structure::IsDisplayed`: stuck in the scene graph.
    fn is_stuck(&self) -> bool {
        self.cstructure()
            .get()
            .is_some_and(|c| c.cstructure_base().fields.read().stick)
    }
    /// `display` (protected): displays a structure not yet stuck (as a highlight structure
    /// when asked), or makes a hidden one visible. Example: `prs.display_for(false)`.
    pub fn display_for(&self, is_highlight: bool) {
        if !self.is_stuck() {
            self.set_is_for_highlight(is_highlight);
            graphic3d_structure_display(self);
        } else if !self.is_visible() {
            self.set_visible(true);
        }
    }
    /// The display state before a highlight.
    fn current_state(&self) -> BeforeHighlightState {
        if !self.is_stuck() {
            BeforeHighlightState::Empty
        } else if !self.is_visible() {
            BeforeHighlightState::Hidden
        } else {
            BeforeHighlightState::Visible
        }
    }
    /// `Highlight`: remembers the display state, displays and highlights with the style.
    /// Example: `prs.highlight_with(&style)`.
    pub fn highlight_with(&self, style: &Handle<Prs3dDrawer>) {
        if !self.is_highlighted() {
            self.state.write().before_highlight_state = self.current_state();
        }
        self.display_for(true);
        rustycad_tkservice::graphic3d::Graphic3dStructureDisplay::highlight(
            self,
            &style.clone().cast(),
            true,
        );
    }
    /// `Unhighlight`: unhighlights and restores the display state before the highlight.
    /// Example: `prs.unhighlight_restore()`.
    pub fn unhighlight_restore(&self) {
        rustycad_tkservice::graphic3d::Graphic3dStructureDisplay::unhighlight(self);
        match self.state.read().before_highlight_state {
            BeforeHighlightState::Visible => {}
            BeforeHighlightState::Hidden => self.set_visible(false),
            BeforeHighlightState::Empty => graphic3d_structure_erase(self),
        }
    }
    /// The display mode `Compute` passes: this presentation's mode when the object lists it, else 0.
    fn listed_mode(&self, object: &dyn PrsMgrPresentableObject) -> i32 {
        let listed = object.presentations();
        let this = self.structure_base().this();
        listed
            .iter()
            .find(|p| p.object_address() == this.object_address())
            .map_or(0, |_| self.mode)
    }
}

/// Reports a failure of a virtual member whose native signature returns nothing.
fn report_failure(member: &str, failure: &Failure) {
    let text = format!(
        "PrsMgr_Presentation::{member}: {} {}",
        failure.exception_type(),
        failure.message()
    );
    Message::send_text(&AsciiString::from(text.as_str()), Gravity::Fail);
}

impl Graphic3dStructureSource for PrsMgrPresentation {
    fn structure_base(&self) -> &Graphic3dStructureBase {
        &self.base
    }
    /// `Display`: displays the presentation, remembered as visible.
    fn display(&self) {
        self.display_for(false);
        self.state.write().before_highlight_state = BeforeHighlightState::Visible;
    }
    /// `Erase`: erases, clears, disconnects and removes the structure.
    fn erase(&self) {
        if self.is_deleted() {
            return;
        }
        graphic3d_structure_erase(self);
        graphic3d_structure_clear(self, true);
        self.disconnect_all(Graphic3dTypeOfConnection::Descendant);
        self.remove();
    }
    /// `Clear`: flags the update, clears the groups and disconnects the descendants.
    fn clear(&self, with_destruction: bool) {
        self.set_update_status(true);
        if self.is_deleted() {
            return;
        }
        graphic3d_structure_clear(self, with_destruction);
        self.disconnect_all(Graphic3dTypeOfConnection::Descendant);
    }
    /// `IsDisplayed`: stuck in the scene graph and visible.
    fn is_displayed(&self) -> bool {
        self.is_stuck() && self.is_visible()
    }
    /// `Compute`: the object computes this presentation; a failure is reported as a fail message.
    fn compute(&self) {
        let object = self.presentable_object();
        let Some(value) = object.get() else {
            return;
        };
        let this: Handle<dyn Graphic3dStructureSource> = self.structure_base().this();
        if let Err(failure) =
            value.compute(&self.presentation_manager, &this, self.listed_mode(value))
        {
            report_failure("Compute", &failure);
        }
    }
    /// `computeHLR`: a new structure when none is given, cleared, then the object's hidden
    /// line computation; a failure is reported as a fail message.
    fn compute_hlr(
        &self,
        projector: &Handle<Graphic3dCamera>,
        structure: &mut Handle<dyn Graphic3dStructureSource>,
    ) {
        if structure.is_null() {
            let manager = self
                .presentation_manager
                .get()
                .map(PrsMgrPresentationManager::structure_manager)
                .unwrap_or_default();
            match Graphic3dStructure::new(&manager, &Handle::default()) {
                Ok(created) => *structure = created.cast(),
                Err(failure) => return report_failure("computeHLR", &failure),
            }
        }
        if let Some(target) = structure.get() {
            target.clear(true);
        }
        let object = self.presentable_object();
        let Some(value) = object.get() else {
            return;
        };
        if let Err(failure) = value.compute_hlr(projector, &self.transformation(), structure) {
            report_failure("computeHLR", &failure);
        }
    }
    /// `RecomputeTransformation`: the object's; `prs.recompute_transformation(&camera)`.
    fn recompute_transformation(&self, projector: &Handle<Graphic3dCamera>) {
        if let Some(value) = self.presentable_object().get() {
            value.recompute_transformation(projector);
        }
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "PrsMgr_Presentation")?;
        base_class(stream, depth, &|s, d| {
            graphic3d_structure_dump_json(self, s, d)
        })?;
        field_pointer(
            stream,
            "myPresentableObject",
            self.presentable_object().object_address() as usize,
        )?;
        let state = *self.state.read();
        field_integer(
            stream,
            "myBeforeHighlightState",
            state.before_highlight_state as i32,
        )?;
        field_integer(stream, "myMode", self.mode)?;
        field_integer(stream, "myMustBeUpdated", i32::from(state.must_be_updated))
    }
}
