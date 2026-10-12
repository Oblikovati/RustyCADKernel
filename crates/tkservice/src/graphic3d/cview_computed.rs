//! The computed mode of `Graphic3d_CView` (`SetComputedMode`, `ReCompute`, `Compute`) and
//! the bounds members `MinMaxValues(set)` and `ConsiderZoomPersistenceObjects`.
use super::cview_structures::{computed_at, computed_visuals, is_cullable};
use super::{
    Graphic3dCView, Graphic3dCViewMembers as _, Graphic3dCViewStructures as _,
    Graphic3dStructureBounds as _, Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _,
    Graphic3dStructureSource, Graphic3dTransformPersApply as _, Graphic3dTransformPersMembers as _,
    Graphic3dTypeOfAnswer as Answer, Graphic3dTypeOfStructure as Visual,
};
use crate::graphic3d::frame_stats_data::native_max;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;

/// The native `ReCompute` body.
pub(crate) fn re_compute<V: Graphic3dCView + ?Sized>(
    view: &V,
    structure: &Handle<dyn Graphic3dStructureSource>,
) {
    let Some(value) = structure.get() else { return };
    value.calculate_bound_box();
    if is_cullable(value) {
        view.invalidate_bvh_data(value.get_z_layer());
    }
    if !view.computed_mode() || !view.is_active() || !value.is_displayed() {
        return;
    }
    value.recompute_transformation(&view.camera());
    let index = view.computed_index(structure);
    if view.accept_display(value.visual()) != Answer::Compute || index == 0 {
        return;
    }
    let old = computed_at(view, index);
    let mut computed = old.clone();
    if let Some(computed) = computed.get() {
        computed.set_transformation(&Handle::default());
    }
    value.compute_hlr(&view.camera(), &mut computed);
    let Some(computed_ref) = computed.get() else {
        return;
    };
    computed_ref.set_hlr_validation(true);
    computed_ref.calculate_bound_box();
    match computed_visuals(view, value) {
        [true, _] => computed_ref.set_visual(Visual::Wireframe),
        [false, true] => computed_ref.set_visual(Visual::Shading),
        _ => {}
    }
    if value.is_highlighted() {
        computed_ref.highlight(&value.highlight_style(), false);
    }
    if let Some(old) = old.get() {
        view.erase_cstructure(&old.cstructure());
    }
    view.display_cstructure(&computed_ref.cstructure(), value.display_priority());
    let mut fields = view.cview_base().fields.write();
    fields.structs_to_compute.push(structure.clone());
    fields.structs_computed.push(computed.clone());
    fields.structs_to_compute.remove(index - 1);
    fields.structs_computed.remove(index - 1);
}

/// The computed mode and bounds members of every view.
pub trait Graphic3dCViewComputed: Graphic3dCView {
    /// `SetComputedMode`: switches the computed structures in (computing the missing ones)
    /// or out; `view.set_computed_mode(true)`.
    fn set_computed_mode(&self, mode: bool) {
        if mode == self.computed_mode() {
            return;
        }
        self.cview_base().fields.write().is_in_computed_mode = mode;
        if !mode {
            self.leave_computed_mode();
            return;
        }
        for structure in self.displayed_structures() {
            let Some(value) = structure
                .get()
                .filter(|s| self.accept_display(s.visual()) == Answer::Compute)
            else {
                continue;
            };
            let index = self.computed_index(&structure);
            if index != 0 {
                self.show_existing_computed(&structure, value, index);
            } else {
                self.compute_and_show(&structure, value);
            }
        }
        self.update(super::GRAPHIC3D_ZLAYER_ID_UNKNOWN);
    }
    /// Displays again the structures instead of their computed structures.
    fn leave_computed_mode(&self) {
        for structure in self.displayed_structures() {
            let Some(value) = structure
                .get()
                .filter(|s| self.accept_display(s.visual()) == Answer::Compute)
            else {
                continue;
            };
            let index = self.computed_index(&structure);
            if let Some(computed) = computed_at(self, index).get() {
                self.erase_cstructure(&computed.cstructure());
                self.display_cstructure(&value.cstructure(), value.display_priority());
                self.update(value.get_z_layer());
            }
        }
    }
    /// Shows the existing computed structure of a structure entering the computed mode.
    fn show_existing_computed(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        value: &dyn Graphic3dStructureSource,
        index: usize,
    ) {
        let computed = computed_at(self, index);
        self.erase_cstructure(&value.cstructure());
        if let Some(computed) = computed.get() {
            self.display_cstructure(&computed.cstructure(), value.display_priority());
        }
        self.display(structure);
        let Some(computed) = computed.get() else {
            return;
        };
        if value.is_highlighted() && !computed.is_highlighted() {
            computed.highlight(&value.highlight_style(), false);
        }
    }
    /// Computes and shows the computed structure of a structure entering the computed mode.
    fn compute_and_show(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        value: &dyn Graphic3dStructureSource,
    ) {
        let mut computed = Handle::default();
        value.compute_hlr(&self.camera(), &mut computed);
        let Some(computed_ref) = computed.get() else {
            return;
        };
        computed_ref.set_hlr_validation(true);
        let [wireframe, shading] = computed_visuals(self, value);
        if wireframe {
            computed_ref.set_visual(Visual::Wireframe);
        }
        if shading {
            computed_ref.set_visual(Visual::Shading);
        }
        if value.is_highlighted() {
            computed_ref.highlight(&value.highlight_style(), false);
        }
        self.store_computed(structure, value.identification(), &computed);
        computed_ref.calculate_bound_box();
        self.erase_cstructure(&value.cstructure());
        self.display_cstructure(&computed_ref.cstructure(), value.display_priority());
    }
    /// Replaces the computed structure of an identification already listed, or appends it.
    fn store_computed(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        id: i32,
        computed: &Handle<dyn Graphic3dStructureSource>,
    ) {
        let mut fields = self.cview_base().fields.write();
        let found = fields
            .structs_to_compute
            .iter()
            .position(|s| s.get().is_some_and(|s| s.identification() == id));
        match found {
            Some(index) => fields.structs_computed[index] = computed.clone(),
            None => {
                fields.structs_to_compute.push(structure.clone());
                fields.structs_computed.push(computed.clone());
            }
        }
    }
    /// `Compute`: invalidates the computed structures, recomputes the transformations and,
    /// in computed mode, displays the computed structures again; `view.compute()`.
    fn compute(&self) {
        let computed = self.cview_base().fields.read().structs_computed.clone();
        for structure in computed.iter().filter_map(Handle::get) {
            structure.set_hlr_validation(false);
        }
        let displayed = self.displayed_structures();
        for structure in displayed.iter().filter_map(Handle::get) {
            structure.recompute_transformation(&self.camera());
        }
        if !self.computed_mode() {
            return;
        }
        let to_display: Vec<_> = displayed
            .iter()
            .filter(|s| {
                s.get()
                    .is_some_and(|s| self.accept_display(s.visual()) == Answer::Compute)
            })
            .cloned()
            .collect();
        for structure in &to_display {
            self.display(structure);
        }
    }
    /// `MinMaxValues(set, toIgnoreInfiniteFlag)`: the union of the boxes of the visible
    /// non-empty structures of the set, moved by their transform persistence;
    /// `view.min_max_values_of(&structures, false)?`.
    fn min_max_values_of(
        &self,
        structures: &[Handle<dyn Graphic3dStructureSource>],
        ignore_infinite: bool,
    ) -> Result<BoundingBox3, Failure> {
        let mut result = BoundingBox3::default();
        let camera = self.camera();
        let size = if self.is_defined() {
            self.window().get().map_or([0, 0], |w| {
                let (width, height) = w.size();
                [width, height]
            })
        } else {
            [0, 0]
        };
        let Some(cam) = camera.get() else {
            return Ok(result);
        };
        let (projection, world_view) = (cam.projection_matrix(), cam.orientation_matrix());
        let view_id = self.identification();
        for structure in structures.iter().filter_map(Handle::get) {
            let visible = structure
                .cstructure()
                .get()
                .is_some_and(|c| super::Graphic3dCStructureMembers::is_visible_in_view(c, view_id));
            if structure.is_empty() || !visible {
                continue;
            }
            let pers = structure.transform_persistence();
            if pers
                .get()
                .is_some_and(|p| !ignore_infinite || p.is_trihedron_or_2d())
            {
                continue;
            }
            let mut bounds = structure.min_max_values(ignore_infinite);
            if bounds.is_whole() || bounds.is_void() {
                continue;
            }
            if let Some(pers) = pers.get() {
                pers.apply_bnd(&camera, &projection, &world_view, size, &mut bounds)?;
            }
            if !beyond_short_real(&bounds) {
                result.add_box(&bounds);
            }
        }
        Ok(result)
    }
    /// `ConsiderZoomPersistenceObjects`: the largest layer coefficient (at least 1; 1 for an
    /// undefined view); `view.consider_zoom_persistence_objects()?`.
    fn consider_zoom_persistence_objects(&self) -> Result<f64, Failure> {
        rustycad_tkernel::standard::catch_native_failure(|| {
            if !self.is_defined() {
                return Ok(1.0);
            }
            // OCCT 3d097a0 keeps the virtual getter's selected field across Size callbacks.
            let camera = self.camera_ref();
            let size = self.window().get().map_or([0, 0], |w| {
                let (width, height) = w.size();
                [width, height]
            });
            maximum_zoom_persistence_coefficient(self, camera.as_ref(), size)
        })
    }
}
impl<T: Graphic3dCView + ?Sized> Graphic3dCViewComputed for T {}

fn maximum_zoom_persistence_coefficient<V: Graphic3dCView + ?Sized>(
    view: &V,
    camera: &dyn rustycad_tkernel::standard::HandleReference<super::Graphic3dCamera>,
    size: [i32; 2],
) -> Result<f64, Failure> {
    let mut max_coef = 1.0;
    for layer in view.layers().iter().filter_map(Handle::get) {
        max_coef = native_max(
            max_coef,
            layer.consider_zoom_persistence_objects(
                view.identification(),
                &camera.observe(),
                size,
            )?,
        );
    }
    Ok(max_coef)
}

/// Whether a corner coordinate is at or beyond the short-real range.
fn beyond_short_real(bounds: &BoundingBox3) -> bool {
    let limit = f64::from(f32::MAX);
    let (Ok(min), Ok(max)) = (bounds.corner_min(), bounds.corner_max()) else {
        return true;
    };
    [max.x(), max.y(), max.z(), min.x(), min.y(), min.z()]
        .iter()
        .any(|v| v.abs() >= limit)
}
