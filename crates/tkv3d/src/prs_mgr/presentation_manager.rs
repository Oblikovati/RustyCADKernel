//! `PrsMgr_PresentationManager`: displays, erases, highlights and updates the presentations of
//! presentable objects in a structure manager. The members that act on an object propagate to
//! its children when the object propagates its visual state, as natively. The immediate mode
//! members are in presentation_immediate.rs.
use super::{
    PrsMgrDisplayStatus, PrsMgrPresentableObject, PrsMgrPresentableObjectMembers as _,
    PrsMgrPresentation,
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::graphic3d::{
    Graphic3dDisplayPriority, Graphic3dStructureDisplay as _, Graphic3dStructureManager,
    Graphic3dStructureMembers as _, Graphic3dStructureNetwork as _, Graphic3dStructureSource,
    Graphic3dTypeOfConnection, Graphic3dZLayerId,
};

/// The immediate mode state of a manager (`myImmediateModeOn` and the immediate lists).
#[derive(Clone, Default)]
pub(crate) struct ImmediateState {
    pub(crate) mode_on: i32,
    pub(crate) list: Vec<Handle<dyn Graphic3dStructureSource>>,
    pub(crate) view_dependent_list: Vec<Handle<dyn Graphic3dStructureSource>>,
}

/// The manager of the presentations of a structure manager.
///
/// Example: `let manager = PrsMgrPresentationManager::new(&viewer.structure_manager()); manager.display(&object, 0)?;`.
pub struct PrsMgrPresentationManager {
    transient: StandardTransient,
    structure_manager: Handle<Graphic3dStructureManager>,
    this: SharedState<WeakHandle<PrsMgrPresentationManager>>,
    pub(crate) immediate: SharedState<ImmediateState>,
}
rustycad_tkernel::impl_transient!(
    PrsMgrPresentationManager,
    "PrsMgr_PresentationManager",
    StandardTransient,
    transient
);

/// The children of an object when it propagates its visual state, else none.
fn propagated_children(
    object: &dyn PrsMgrPresentableObject,
) -> Vec<Handle<dyn PrsMgrPresentableObject>> {
    if object.to_propagate_visual_state() {
        object.children()
    } else {
        Vec::new()
    }
}

impl PrsMgrPresentationManager {
    /// A manager of the presentations of a structure manager.
    /// Example: `PrsMgrPresentationManager::new(&viewer.structure_manager())`.
    pub fn new(structure_manager: &Handle<Graphic3dStructureManager>) -> Handle<Self> {
        let manager = Handle::new(Self {
            transient: StandardTransient::default(),
            structure_manager: structure_manager.clone(),
            this: SharedState::new(WeakHandle::default()),
            immediate: SharedState::new(ImmediateState::default()),
        });
        *manager.this.write() = manager.downgrade();
        manager
    }
    /// The handle of the manager; `manager.this()`.
    pub fn this(&self) -> Handle<PrsMgrPresentationManager> {
        self.this.read().upgrade()
    }
    /// `StructureManager`; `manager.structure_manager()`.
    pub fn structure_manager(&self) -> Handle<Graphic3dStructureManager> {
        self.structure_manager.clone()
    }
    /// Whether a presentation belongs to this manager.
    fn owns(&self, prs: &PrsMgrPresentation) -> bool {
        prs.presentation_manager().object_address() == (self as *const Self).cast()
    }
    /// `Display`: the presentation of the mode (created and updated on demand) is displayed or,
    /// in immediate mode, queued; an object without own presentations computes itself;
    /// non-erased children follow. Example: `manager.display(&object, 0)?`.
    pub fn display(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        if value.has_own_presentations() {
            let prs = self.presentation(object, mode, true, &Handle::default())?;
            if prs.get().is_some_and(PrsMgrPresentation::must_be_updated) {
                self.update(object, mode)?;
            }
            if self.immediate.read().mode_on > 0 {
                self.add_to_immediate_list(&prs.cast());
            } else if let Some(prs) = prs.get() {
                prs.display();
            }
        } else {
            value.compute(&self.this(), &Handle::default(), mode)?;
        }
        for child in propagated_children(value) {
            if child
                .get()
                .is_some_and(|c| c.display_status() != PrsMgrDisplayStatus::Erased)
            {
                self.display(&child, mode)?;
            }
        }
        Ok(())
    }
    /// `Erase`: the children first, then the presentation of the mode of this manager (all
    /// of them for mode -1) is erased and forgotten. Example: `manager.erase(&object, 0)`.
    pub fn erase(&self, object: &Handle<dyn PrsMgrPresentableObject>, mode: i32) {
        let Some(value) = object.get() else {
            return;
        };
        propagated_children(value)
            .iter()
            .for_each(|child| self.erase(child, mode));
        let mut kept = Vec::new();
        let mut done = false;
        for handle in value.presentations() {
            let matches = handle
                .get()
                .is_some_and(|p| (mode == p.mode() || mode == -1) && self.owns(p));
            if done || !matches {
                kept.push(handle);
                continue;
            }
            if let Some(prs) = handle.get() {
                prs.erase();
            }
            done = mode != -1;
        }
        value.presentable_base().fields.write().presentations = kept;
    }
    /// `Clear`: clears the presentation of the mode; `manager.clear(&object, 0)`.
    pub fn clear(&self, object: &Handle<dyn PrsMgrPresentableObject>, mode: i32) {
        let Some(value) = object.get() else {
            return;
        };
        propagated_children(value)
            .iter()
            .for_each(|child| self.clear(child, mode));
        if let Some(prs) = self.existing(object, mode).get() {
            prs.clear(true);
        }
    }
    /// `SetVisibility`: shows or hides the presentation of the mode (hiding every child,
    /// showing the non-erased ones). Example: `manager.set_visibility(&object, 0, false)`.
    pub fn set_visibility(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
        value: bool,
    ) {
        let Some(own) = object.get() else {
            return;
        };
        for child in propagated_children(own) {
            if !value
                || child
                    .get()
                    .is_some_and(|c| c.display_status() != PrsMgrDisplayStatus::Erased)
            {
                self.set_visibility(&child, mode, value);
            }
        }
        if !own.has_own_presentations() {
            return;
        }
        if let Some(prs) = self.existing(object, mode).get() {
            prs.set_visible(value);
        }
    }
    /// `Unhighlight`: unhighlights the highlighted presentations of this manager.
    /// Example: `manager.unhighlight(&object)`.
    pub fn unhighlight(&self, object: &Handle<dyn PrsMgrPresentableObject>) {
        let Some(value) = object.get() else {
            return;
        };
        propagated_children(value)
            .iter()
            .for_each(|child| self.unhighlight(child));
        for prs in value.presentations().iter().filter_map(Handle::get) {
            if self.owns(prs) && prs.is_highlighted() {
                prs.unhighlight_restore();
            }
        }
    }
    /// `SetDisplayPriority`; `manager.set_display_priority(&object, 0, Graphic3dDisplayPriority::Above)?`.
    pub fn set_display_priority(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
        priority: Graphic3dDisplayPriority,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        for child in propagated_children(value) {
            self.set_display_priority(&child, mode, priority)?;
        }
        match self.existing(object, mode).get() {
            Some(prs) => prs.set_display_priority(priority),
            None => Ok(()),
        }
    }
    /// `DisplayPriority`: the first valid priority of the children, else the presentation's,
    /// else `INVALID`. Example: `manager.display_priority(&object, 0)`.
    pub fn display_priority(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
    ) -> Graphic3dDisplayPriority {
        let Some(value) = object.get() else {
            return Graphic3dDisplayPriority::Invalid;
        };
        for child in propagated_children(value) {
            let priority = self.display_priority(&child, mode);
            if priority != Graphic3dDisplayPriority::Invalid {
                return priority;
            }
        }
        self.existing(object, mode)
            .get()
            .map_or(Graphic3dDisplayPriority::Invalid, |p| p.display_priority())
    }
    /// `SetZLayer`: the layer of the object (and of its children); `manager.set_z_layer(&object, layer)`.
    pub fn set_z_layer(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        layer_id: Graphic3dZLayerId,
    ) {
        let Some(value) = object.get() else {
            return;
        };
        propagated_children(value)
            .iter()
            .for_each(|child| self.set_z_layer(child, layer_id));
        if value.has_own_presentations() {
            value.set_z_layer(layer_id);
        }
    }
    /// `GetZLayer`: the object layer; `manager.get_z_layer(&object)`.
    pub fn get_z_layer(&self, object: &Handle<dyn PrsMgrPresentableObject>) -> Graphic3dZLayerId {
        object.get().map_or(
            rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_UNKNOWN,
            |o| o.z_layer(),
        )
    }
    /// `IsDisplayed`: a displayed child or presentation of the mode; `manager.is_displayed(&object, 0)`.
    pub fn is_displayed(&self, object: &Handle<dyn PrsMgrPresentableObject>, mode: i32) -> bool {
        let Some(value) = object.get() else {
            return false;
        };
        propagated_children(value)
            .iter()
            .any(|child| self.is_displayed(child, mode))
            || self
                .existing(object, mode)
                .get()
                .is_some_and(|p| p.is_displayed())
    }
    /// `IsHighlighted`: a highlighted child or presentation of the mode; `manager.is_highlighted(&object, 0)`.
    pub fn is_highlighted(&self, object: &Handle<dyn PrsMgrPresentableObject>, mode: i32) -> bool {
        let Some(value) = object.get() else {
            return false;
        };
        propagated_children(value)
            .iter()
            .any(|child| self.is_highlighted(child, mode))
            || self
                .existing(object, mode)
                .get()
                .is_some_and(|p| p.is_highlighted())
    }
    /// `Update`: the children, then the presentation of the mode is cleared and filled again.
    /// Example: `manager.update(&object, 0)?`.
    pub fn update(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        for child in value.children() {
            self.update(&child, mode)?;
        }
        let prs = self.existing(object, mode);
        if let Some(structure) = prs.get() {
            structure.clear(true);
            value.fill(&self.this(), &prs, mode)?;
            structure.set_update_status(false);
        }
        Ok(())
    }
    /// `HasPresentation`: an own presentation of the mode in this manager.
    /// Example: `manager.has_presentation(&object, 0)`.
    pub fn has_presentation(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
    ) -> bool {
        object.get().is_some_and(|o| o.has_own_presentations())
            && !self.existing(object, mode).is_null()
    }
    /// The presentation of the mode in this manager, null without.
    fn existing(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
    ) -> Handle<PrsMgrPresentation> {
        let Some(value) = object.get() else {
            return Handle::default();
        };
        value
            .presentations()
            .into_iter()
            .find(|p| p.get().is_some_and(|v| v.mode() == mode && self.owns(v)))
            .unwrap_or_default()
    }
    /// `Presentation`: the presentation of the mode, or a new one (in the object's layer, with
    /// the view affinity of `sel_obj` or the object, filled) when asked.
    /// Example: `manager.presentation(&object, 0, true, &Handle::default())?`.
    pub fn presentation(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
        to_create: bool,
        sel_obj: &Handle<dyn PrsMgrPresentableObject>,
    ) -> Result<Handle<PrsMgrPresentation>, Failure> {
        let found = self.existing(object, mode);
        let Some(value) = object.get().filter(|_| found.is_null() && to_create) else {
            return Ok(found);
        };
        let prs = PrsMgrPresentation::new(&self.this(), object, mode)?;
        prs.set_z_layer(value.z_layer());
        let affinity = sel_obj
            .get()
            .map_or_else(|| value.view_affinity(), |s| s.view_affinity());
        if let Some(cstructure) = prs.cstructure().get() {
            cstructure.cstructure_base().fields.write().view_affinity = affinity;
        }
        value
            .presentable_base()
            .fields
            .write()
            .presentations
            .push(prs.clone());
        value.fill(&self.this(), &prs, mode)?;
        prs.set_update_status(false);
        Ok(prs)
    }
    /// `RemovePresentation` (protected): forgets the presentation of the mode; true when found.
    /// Example: `manager.remove_presentation(&object, 0)`.
    pub fn remove_presentation(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
    ) -> bool {
        let Some(value) = object.get() else {
            return false;
        };
        let presentations = value.presentations();
        let Some(index) = presentations
            .iter()
            .position(|p| p.get().is_some_and(|v| v.mode() == mode && self.owns(v)))
        else {
            return false;
        };
        value
            .presentable_base()
            .fields
            .write()
            .presentations
            .remove(index);
        true
    }
    /// `Connect`: connects the presentations of two objects (created on demand).
    /// Example: `manager.connect(&object, &other, 0, 0)?`.
    pub fn connect(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        other: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
        other_mode: i32,
    ) -> Result<(), Failure> {
        let prs = self.presentation(object, mode, true, &Handle::default())?;
        let other_prs = self.presentation(other, other_mode, true, &Handle::default())?;
        if let Some(prs) = prs.get() {
            prs.connect(
                &other_prs.cast(),
                Graphic3dTypeOfConnection::Descendant,
                false,
            );
        }
        Ok(())
    }
    /// `Transform`: the transformation of the presentation of the mode; a missing presentation
    /// raises `Standard_NullObject` (a null dereference natively). Example: `manager.transform(&object, &trsf, 0)?`.
    pub fn transform(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        trsf: &Handle<Datum3>,
        mode: i32,
    ) -> Result<(), Failure> {
        let prs = self.existing(object, mode);
        let Some(prs) = prs.get() else {
            return Err(rustycad_tkernel::standard::exceptions::null_object(
                &format!(
                    "PrsMgr_PresentationManager::Transform: no presentation of mode {mode}; expected a displayed mode"
                ),
            ));
        };
        prs.set_transformation(trsf);
        Ok(())
    }
}
