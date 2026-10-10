//! The display state members of `Graphic3d_Structure`: the virtual `Clear`, `Display`,
//! `Erase` and `SetVisual` bodies, and the priority, highlight and visibility members.
use super::exceptions::{priority_definition_error, structure_definition_error};
use super::{
    Graphic3dCStructureMembers as _, Graphic3dDisplayPriority, Graphic3dGroupMembers as _,
    Graphic3dPresentationAttributesSource, Graphic3dStructureMembers as _,
    Graphic3dStructureSource, Graphic3dTypeOfStructure,
};
use rustycad_tkernel::standard::{Failure, Handle};

/// The native protected `clear()`, for the subclasses that call it: clears the groups,
/// resets the group flags, clears the computed structures of the views and updates the
/// structure layer. Example: `graphic3d_structure_clear(&*prs, true)`.
pub fn graphic3d_structure_clear<S: Graphic3dStructureSource + ?Sized>(
    structure: &S,
    with_destruction: bool,
) {
    if structure.is_deleted() {
        return;
    }
    structure.graphic_clear(with_destruction);
    if let Some(cstructure) = structure.cstructure().get() {
        cstructure.set_group_transform_persistence(false);
        cstructure.set_group_flipping(false);
    }
    let this = structure.structure_base().this();
    if let Some(manager) = structure.structure_manager().get() {
        manager.clear(&this, with_destruction);
    }
    structure.update(true);
}

/// The native `Graphic3d_Structure::Display()` body, for the subclasses that override it.
/// Example: `graphic3d_structure_display(&*prs)`.
pub fn graphic3d_structure_display<S: Graphic3dStructureSource + ?Sized>(structure: &S) {
    let cstructure = structure.cstructure();
    let Some(cstructure) = cstructure.get() else {
        return;
    };
    if !cstructure.cstructure_base().fields.read().stick {
        cstructure.cstructure_base().fields.write().stick = true;
        if let Some(manager) = structure.structure_manager().get() {
            manager.display(&structure.structure_base().this());
        }
    }
    if !cstructure.is_visible() {
        cstructure.cstructure_base().fields.write().visible = true;
        cstructure.on_visibility_changed();
    }
}

/// The native protected `erase()`, for the subclasses that call it.
/// Example: `graphic3d_structure_erase(&*prs)`.
pub fn graphic3d_structure_erase<S: Graphic3dStructureSource + ?Sized>(structure: &S) {
    let cstructure = structure.cstructure();
    let Some(cstructure) = cstructure.get() else {
        return;
    };
    if cstructure.cstructure_base().fields.read().stick {
        cstructure.cstructure_base().fields.write().stick = false;
        if let Some(manager) = structure.structure_manager().get() {
            manager.erase_structure(&structure.structure_base().this());
        }
    }
}

/// The native `SetVisual`: a displayed structure is erased and displayed again.
pub(crate) fn structure_set_visual<S: Graphic3dStructureSource + ?Sized>(
    structure: &S,
    visual: Graphic3dTypeOfStructure,
) {
    if structure.is_deleted() || structure.visual() == visual {
        return;
    }
    let stuck = structure
        .cstructure()
        .get()
        .is_some_and(|c| c.cstructure_base().fields.read().stick);
    if stuck {
        graphic3d_structure_erase(structure);
    }
    structure.structure_base().fields.write().visual = visual;
    structure.set_compute_visual(visual);
    if stuck {
        structure.display();
    }
}

/// The priority, highlight and visibility members of every structure.
pub trait Graphic3dStructureDisplay: Graphic3dStructureSource {
    /// `SetDisplayPriority`: remembers the previous priority and moves a displayed structure;
    /// a priority outside Bottom..Topmost raises `Graphic3d_PriorityDefinitionError`.
    /// Example: `structure.set_display_priority(Graphic3dDisplayPriority::Above)?`.
    fn set_display_priority(&self, priority: Graphic3dDisplayPriority) -> Result<(), Failure> {
        let cstructure = self.cstructure();
        let Some(cs) = cstructure.get().filter(|c| c.priority() != priority) else {
            return Ok(());
        };
        if priority == Graphic3dDisplayPriority::Invalid {
            return Err(priority_definition_error(&format!(
                "Bad value for StructurePriority: {priority:?}; expected Bottom..Topmost"
            )));
        }
        cs.set_previous_priority(cs.priority());
        cs.set_priority(priority);
        if cs.priority() != cs.previous_priority()
            && cs.cstructure_base().fields.read().stick
            && let Some(manager) = self.structure_manager().get()
        {
            manager.change_display_priority(
                &self.structure_base().this(),
                cs.previous_priority(),
                cs.priority(),
            );
        }
        Ok(())
    }
    /// `ResetDisplayPriority`: back to the previous priority; `structure.reset_display_priority()`.
    fn reset_display_priority(&self) {
        let cstructure = self.cstructure();
        let Some(cs) = cstructure
            .get()
            .filter(|c| c.priority() != c.previous_priority())
        else {
            return;
        };
        let priority = cs.priority();
        cs.set_priority(cs.previous_priority());
        if cs.cstructure_base().fields.read().stick
            && let Some(manager) = self.structure_manager().get()
        {
            manager.change_display_priority(&self.structure_base().this(), priority, cs.priority());
        }
    }
    /// `Highlight`: the highlight priority and the driver highlight, then (when asked) the
    /// manager highlight of a displayed structure and an update; `structure.highlight(&style, true)`.
    fn highlight(
        &self,
        style: &Handle<dyn Graphic3dPresentationAttributesSource>,
        to_update_mgr: bool,
    ) {
        let cstructure = self.cstructure();
        let Some(cs) = cstructure.get() else { return };
        // The highlight priority is in range, so the priority check cannot fail.
        let _ = self.set_display_priority(Graphic3dDisplayPriority::Highlight);
        cs.graphic_highlight(style);
        if !to_update_mgr {
            return;
        }
        if cs.cstructure_base().fields.read().stick
            && let Some(manager) = self.structure_manager().get()
        {
            manager.highlight(&self.structure_base().this());
        }
        self.update(false);
    }
    /// `SetVisible`: changes the visibility flag and updates the layer; `structure.set_visible(false)`.
    fn set_visible(&self, value: bool) {
        let cstructure = self.cstructure();
        let Some(cs) = cstructure.get().filter(|c| c.is_visible() != value) else {
            return;
        };
        cs.cstructure_base().fields.write().visible = value;
        cs.on_visibility_changed();
        self.update(true);
    }
    /// `UnHighlight`: the driver and manager unhighlight, the previous priority and an update;
    /// `structure.unhighlight()`.
    fn unhighlight(&self) {
        let cstructure = self.cstructure();
        let Some(cs) = cstructure
            .get()
            .filter(|c| c.cstructure_base().fields.read().highlight)
        else {
            return;
        };
        cs.cstructure_base().fields.write().highlight = false;
        cs.graphic_unhighlight();
        if let Some(manager) = self.structure_manager().get() {
            manager.unhighlight_structure(&self.structure_base().this());
        }
        self.reset_display_priority();
        self.update(false);
    }
    /// `GraphicClear`: clears the groups and, with destruction, removes them and clears the
    /// driver structure; `structure.graphic_clear(true)`.
    fn graphic_clear(&self, with_destruction: bool) {
        let cstructure = self.cstructure();
        let Some(cs) = cstructure.get() else { return };
        for group in cs.groups().iter().filter_map(Handle::get) {
            group.clear(true);
        }
        if !with_destruction {
            return;
        }
        while let Some(group) = cs.groups().first().cloned() {
            let Some(group) = group.get() else { break };
            group.remove();
        }
        cs.clear();
    }
    /// `SetZoomLimit`: only validates the limits (positive, sup not below inf);
    /// `structure.set_zoom_limit(1.0, 2.0)?`.
    fn set_zoom_limit(&self, limit_inf: f64, limit_sup: f64) -> Result<(), Failure> {
        let message = if limit_inf <= 0.0 {
            "Bad value for ZoomLimit inf"
        } else if limit_sup <= 0.0 {
            "Bad value for ZoomLimit sup"
        } else if limit_sup < limit_inf {
            "ZoomLimit sup < ZoomLimit inf"
        } else {
            return Ok(());
        };
        Err(structure_definition_error(&format!(
            "{message}: inf {limit_inf}, sup {limit_sup}; expected 0 < inf <= sup"
        )))
    }
}
impl<T: Graphic3dStructureSource + ?Sized> Graphic3dStructureDisplay for T {}
