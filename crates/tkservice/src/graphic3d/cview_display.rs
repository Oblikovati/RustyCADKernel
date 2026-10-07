//! `Graphic3d_CView::Display`: a structure is displayed as is, or replaced by its computed
//! (hidden line) structure in computed mode, reusing a valid computed structure.
use super::cview_structures::{add_displayed, computed_at, computed_visuals, remove_computed};
use super::{
    Graphic3dCView, Graphic3dCViewMembers as _, Graphic3dCViewStructures as _,
    Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
    Graphic3dTypeOfAnswer as Answer, Graphic3dTypeOfStructure as Visual,
};
use rustycad_tkernel::standard::Handle;

/// Shows the driver structure of `shown` at the priority of `structure` and updates its layer.
fn show<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &dyn Graphic3dStructureSource,
    shown: &dyn Graphic3dStructureSource,
) {
    view.display_cstructure(&shown.cstructure(), structure.display_priority());
    view.update(shown.get_z_layer());
}

/// The native `Display` body.
pub(crate) fn display<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &Handle<dyn Graphic3dStructureSource>,
) {
    let Some(value) = structure.get().filter(|_| view.is_active()) else {
        return;
    };
    let mut index = view.computed_index(structure);
    if index != 0 && value.visual() != Visual::Computed {
        remove_computed(view, index);
        index = 0;
    }
    value.recompute_transformation(&view.camera());
    let mut answer = view.accept_display(value.visual());
    if answer == Answer::No {
        return;
    }
    if !view.computed_mode() {
        answer = Answer::Yes;
    }
    if answer == Answer::Yes {
        if add_displayed(view, structure) {
            value.calculate_bound_box();
            show(view, value, value);
        }
        return;
    }
    if answer != Answer::Compute || (index != 0 && reuse_computed(view, structure, index)) {
        return;
    }
    display_new_computed(view, structure, index);
}

/// Displays the existing computed structure at `index` when it is still valid (or another
/// valid one of the same owner); true when done. Otherwise erases the stale one.
fn reuse_computed<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &Handle<dyn Graphic3dStructureSource>,
    index: usize,
) -> bool {
    let Some(value) = structure.get() else {
        return true;
    };
    let old = computed_at(view, index);
    let Some(old_ref) = old.get() else {
        return false;
    };
    if old_ref.hlr_validation() {
        if add_displayed(view, structure) {
            show(view, value, old_ref);
        }
        return true;
    }
    let new_index = view.have_the_same_owner(structure);
    if new_index != 0 {
        if add_displayed(view, structure) {
            let other = computed_at(view, new_index);
            view.cview_base().fields.write().structs_computed[index - 1] = other.clone();
            if let Some(other) = other.get() {
                show(view, value, other);
            }
        }
        return true;
    }
    if view.is_displayed(structure) {
        view.erase_cstructure(&old_ref.cstructure());
    }
    false
}

/// Computes a new hidden line structure (replacing the one at `index` when not 0),
/// sets its visual and highlight, and displays it.
fn display_new_computed<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &Handle<dyn Graphic3dStructureSource>,
    index: usize,
) {
    let Some(value) = structure.get() else { return };
    let mut computed = Handle::default();
    if index != 0 {
        computed = computed_at(view, index);
        if let Some(previous) = computed.get() {
            previous.set_transformation(&Handle::default());
        }
    }
    value.compute_hlr(&view.camera(), &mut computed);
    let Some(computed_ref) = computed.get() else {
        return;
    };
    computed_ref.set_hlr_validation(true);
    {
        let mut fields = view.cview_base().fields.write();
        fields.structs_to_compute.push(structure.clone());
        fields.structs_computed.push(computed.clone());
    }
    if index != 0 {
        remove_computed(view, index);
    }
    let [wireframe, shading] = computed_visuals(view, value);
    let mut answer = Answer::No;
    if wireframe || shading {
        computed_ref.set_visual(if wireframe {
            Visual::Wireframe
        } else {
            Visual::Shading
        });
        answer = view.accept_display(computed_ref.visual());
    }
    if value.is_highlighted() {
        computed_ref.highlight(&value.highlight_style(), false);
    }
    if answer == Answer::No {
        return;
    }
    add_displayed(view, structure);
    show(view, value, computed_ref);
}
