//! The structure members of `Graphic3d_CView` that the structure manager calls (friend
//! members natively): display, erase, highlight, transformation, priority and layer
//! changes, and the bookkeeping of the computed structures.
use super::{
    Graphic3dCStructure, Graphic3dCView, Graphic3dCViewMembers as _, Graphic3dDisplayPriority,
    Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
    Graphic3dTypeOfAnswer as Answer, Graphic3dTypeOfStructure as Visual,
    Graphic3dTypeOfVisualization, Graphic3dZLayerId,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::TransformForm;
use rustycad_tkmath::toploc::Datum3;

/// The computed structure at a one-based index (null outside the list).
pub(crate) fn computed_at<V: Graphic3dCView + ?Sized>(
    view: &V,
    index: usize,
) -> Handle<dyn Graphic3dStructureSource> {
    let fields = view.cview_base().fields.read();
    index
        .checked_sub(1)
        .and_then(|i| fields.structs_computed.get(i))
        .cloned()
        .unwrap_or_default()
}

/// `myStructsToCompute.Remove(i)` and `myStructsComputed.Remove(i)` (one-based).
pub(crate) fn remove_computed<V: Graphic3dCView + ?Sized>(view: &V, index: usize) {
    let mut fields = view.cview_base().fields.write();
    if index >= 1
        && index <= fields.structs_to_compute.len()
        && index <= fields.structs_computed.len()
    {
        fields.structs_to_compute.remove(index - 1);
        fields.structs_computed.remove(index - 1);
    }
}

/// `myStructsDisplayed.Add`: true when the structure was not displayed.
pub(crate) fn add_displayed<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &Handle<dyn Graphic3dStructureSource>,
) -> bool {
    let mut fields = view.cview_base().fields.write();
    let before = fields.structs_displayed.extent();
    fields.structs_displayed.add(structure.clone());
    fields.structs_displayed.extent() > before
}

/// Whether a structure is neither mutable, for highlight nor infinite (its moves invalidate
/// the layer BVH).
pub(crate) fn is_cullable(structure: &dyn Graphic3dStructureSource) -> bool {
    let cstructure = structure.cstructure();
    !structure.is_mutable()
        && cstructure.get().is_some_and(|c| {
            let fields = c.cstructure_base().fields.read();
            !fields.is_for_highlight && !fields.is_infinite
        })
}

/// The wireframe and shading visuals a computed structure gets: `[wireframe, shading]`.
pub(crate) fn computed_visuals<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &dyn Graphic3dStructureSource,
) -> [bool; 2] {
    let visualization = view.visualization_type();
    let compute_visual = structure.compute_visual();
    [
        visualization == Graphic3dTypeOfVisualization::Wireframe
            && compute_visual != Visual::Shading,
        visualization == Graphic3dTypeOfVisualization::Shading
            && compute_visual != Visual::Wireframe,
    ]
}

/// The structure members of every view.
pub trait Graphic3dCViewStructures: Graphic3dCView {
    /// `acceptDisplay`: whether a structure of a visual is displayed, or computed;
    /// `view.accept_display(Graphic3dTypeOfStructure::All)`.
    fn accept_display(&self, visual: Visual) -> Answer {
        let visualization = self.visualization_type();
        let matches = |wanted: Graphic3dTypeOfVisualization| {
            if visualization == wanted {
                Answer::Yes
            } else {
                Answer::No
            }
        };
        match visual {
            Visual::All => Answer::Yes,
            Visual::Shading => matches(Graphic3dTypeOfVisualization::Shading),
            Visual::Wireframe => matches(Graphic3dTypeOfVisualization::Wireframe),
            Visual::Computed => Answer::Compute,
        }
    }
    /// `IsComputed(const Graphic3d_Structure*)`: the one-based index of the structure in the
    /// computed list (by identification), or 0; `view.computed_index(&s)`.
    fn computed_index(&self, structure: &Handle<dyn Graphic3dStructureSource>) -> usize {
        let id = structure.get().map_or(-1, |s| s.identification());
        let list = self.cview_base().fields.read().structs_to_compute.clone();
        list.iter()
            .position(|s| s.get().is_some_and(|s| s.identification() == id))
            .map_or(0, |i| i + 1)
    }
    /// `IsComputed(theStructId, theComputedStruct)`: the computed structure of an
    /// identification in computed mode, or null; `view.is_computed(3)`.
    fn is_computed(&self, structure_id: i32) -> Handle<dyn Graphic3dStructureSource> {
        if !self.computed_mode() {
            return Handle::default();
        }
        let fields = self.cview_base().fields.read();
        let index = fields
            .structs_to_compute
            .iter()
            .position(|s| s.get().is_some_and(|s| s.identification() == structure_id));
        index
            .and_then(|i| fields.structs_computed.get(i))
            .cloned()
            .unwrap_or_default()
    }
    /// `IsDisplayed`; `view.is_displayed(&s)`.
    fn is_displayed(&self, structure: &Handle<dyn Graphic3dStructureSource>) -> bool {
        self.cview_base()
            .fields
            .read()
            .structs_displayed
            .contains(structure)
    }
    /// `DisplayedStructures`; `view.displayed_structures()`.
    fn displayed_structures(&self) -> Vec<Handle<dyn Graphic3dStructureSource>> {
        self.cview_base()
            .fields
            .read()
            .structs_displayed
            .iter()
            .cloned()
            .collect()
    }
    /// `HaveTheSameOwner`: the one-based index of another computed structure of the same
    /// owner whose result is still valid, or 0; `view.have_the_same_owner(&s)`.
    fn have_the_same_owner(&self, structure: &Handle<dyn Graphic3dStructureSource>) -> usize {
        let Some(value) = structure.get() else {
            return 0;
        };
        let owner = value.structure_base().fields.read().owner.object_address();
        let (to_compute, computed) = {
            let fields = self.cview_base().fields.read();
            (
                fields.structs_to_compute.clone(),
                fields.structs_computed.clone(),
            )
        };
        for (index, candidate) in to_compute.iter().enumerate() {
            let Some(candidate) = candidate.get() else {
                continue;
            };
            let same_owner = candidate
                .structure_base()
                .fields
                .read()
                .owner
                .object_address()
                == owner;
            if same_owner
                && candidate.identification() != value.identification()
                && computed[index].get().is_some_and(|c| c.hlr_validation())
            {
                return index + 1;
            }
        }
        0
    }
    /// `Update`: invalidates the box of a layer (or of every persistent layer);
    /// `view.update(GRAPHIC3D_ZLAYER_ID_UNKNOWN)`.
    fn update(&self, layer_id: Graphic3dZLayerId) {
        self.invalidate_z_layer_bounding_box(layer_id);
    }
    /// `Display`: displays a structure, or its computed structure in computed mode;
    /// `view.display(&s)`.
    fn display(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        super::cview_display::display(self, structure);
    }
    /// `Erase`: erases a structure and its computed structure; `view.erase(&s)`.
    fn erase(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        let Some(value) = structure.get().filter(|_| self.is_displayed(structure)) else {
            return;
        };
        let answer = if self.computed_mode() {
            self.accept_display(value.visual())
        } else {
            Answer::Yes
        };
        if answer != Answer::Compute {
            self.erase_cstructure(&value.cstructure());
        }
        let has_computed = !self
            .cview_base()
            .fields
            .read()
            .structs_to_compute
            .is_empty();
        let index = if has_computed {
            self.computed_index(structure)
        } else {
            0
        };
        if index != 0 {
            if answer == Answer::Compute
                && self.computed_mode()
                && let Some(computed) = computed_at(self, index).get()
            {
                self.erase_cstructure(&computed.cstructure());
            }
            remove_computed(self, index);
        }
        self.cview_base()
            .fields
            .write()
            .structs_displayed
            .remove_key(structure);
        self.update(value.get_z_layer());
    }
    /// `Highlight`: highlights the computed structure; `view.highlight(&s)`.
    fn highlight(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        let computed = computed_at(self, self.computed_index(structure));
        if let (Some(computed), Some(value)) = (computed.get(), structure.get()) {
            computed.highlight(&value.highlight_style(), false);
        }
    }
    /// `UnHighlight`: unhighlights the computed driver structure; `view.unhighlight(&s)`.
    fn unhighlight(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        let computed = computed_at(self, self.computed_index(structure));
        if let Some(cstructure) = computed.get().map(|c| c.cstructure())
            && let Some(cstructure) = cstructure.get()
        {
            cstructure.graphic_unhighlight();
        }
    }
    /// `SetTransform`: recomputes or moves the computed structure, then recomputes the box
    /// and invalidates the layer BVH of a cullable structure; `view.set_transform(&s, &datum)`.
    fn set_transform(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        trsf: &Handle<Datum3>,
    ) {
        let Some(value) = structure.get() else { return };
        let index = self.computed_index(structure);
        if index != 0 {
            let form = trsf.get().map(Datum3::form);
            if matches!(
                form,
                Some(TransformForm::Translation | TransformForm::Scale | TransformForm::Compound)
            ) {
                self.re_compute(structure);
            } else if let Some(computed) = computed_at(self, index).get() {
                computed.graphic_transform(trsf);
            }
        }
        value.calculate_bound_box();
        if is_cullable(value) {
            self.invalidate_bvh_data(value.get_z_layer());
        }
    }
    /// The driver structure displayed for a structure: the computed one in computed mode.
    fn displayed_cstructure(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Handle<dyn Graphic3dCStructure> {
        let own = structure.get().map(|s| s.cstructure()).unwrap_or_default();
        if !self.computed_mode() {
            return own;
        }
        let computed = computed_at(self, self.computed_index(structure));
        computed.get().map_or(own, |c| c.cstructure())
    }
    /// `ChangePriority`: moves a displayed structure (or its computed one) to a priority;
    /// `view.change_priority(&s, old, new)`.
    fn change_priority(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        old: Graphic3dDisplayPriority,
        new: Graphic3dDisplayPriority,
    ) {
        let _ = old;
        if self.is_active() && self.is_displayed(structure) {
            self.change_cstructure_priority(&self.displayed_cstructure(structure), new);
        }
    }
    /// `ChangeZLayer`: moves a displayed structure (or its computed one) to a layer;
    /// `view.change_z_layer(&s, 1)`.
    fn change_z_layer(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        layer_id: Graphic3dZLayerId,
    ) {
        if self.is_active() && self.is_displayed(structure) {
            self.change_cstructure_z_layer(&self.displayed_cstructure(structure), layer_id);
        }
    }
    /// `Clear`: clears the computed structure and invalidates it; `view.clear_computed(&s, true)`.
    fn clear_computed(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        with_destruction: bool,
    ) {
        let computed = computed_at(self, self.computed_index(structure));
        if let Some(computed) = computed.get() {
            computed.graphic_clear(with_destruction);
            computed.set_hlr_validation(false);
        }
    }
    /// `Connect`: connects the computed structures of both; `view.connect_computed(&mother, &daughter)`.
    fn connect_computed(
        &self,
        mother: &Handle<dyn Graphic3dStructureSource>,
        daughter: &Handle<dyn Graphic3dStructureSource>,
    ) {
        let (m, d) = (self.computed_index(mother), self.computed_index(daughter));
        if m != 0
            && d != 0
            && let Some(mother) = computed_at(self, m).get()
        {
            mother.graphic_connect(&computed_at(self, d));
        }
    }
    /// `Disconnect`: disconnects the computed structures of both; `view.disconnect_computed(&mother, &daughter)`.
    fn disconnect_computed(
        &self,
        mother: &Handle<dyn Graphic3dStructureSource>,
        daughter: &Handle<dyn Graphic3dStructureSource>,
    ) {
        let (m, d) = (self.computed_index(mother), self.computed_index(daughter));
        if m != 0
            && d != 0
            && let Some(mother) = computed_at(self, m).get()
        {
            mother.graphic_disconnect(&computed_at(self, d));
        }
    }
    /// `ReCompute`: recomputes the box (invalidating the layer BVH) and, in computed mode,
    /// the computed structure of an active displayed structure; `view.re_compute(&s)`.
    fn re_compute(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        super::cview_computed::re_compute(self, structure);
    }
}
impl<T: Graphic3dCView + ?Sized> Graphic3dCViewStructures for T {}
