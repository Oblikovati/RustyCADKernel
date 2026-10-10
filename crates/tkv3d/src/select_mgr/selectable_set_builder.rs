//! Builder adapter keeps persistent bounds aligned with native indexed-map swaps.
use super::selectable_object_set::{SelectableMap, SelectableSubsetState};
use super::selectable_set_bounds::{
    SelectableBoundsView, selectable_persistent_bounds, selectable_regular_bounds,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{BvhBox, BvhSet, BvhVec3d, combined_element_boxes};
use std::cell::Cell;

struct SelectableBuilderAdapter<'a> {
    objects: &'a mut SelectableMap,
    persistent: Option<Vec<BvhBox<BvhVec3d>>>,
    cached: Cell<BvhBox<BvhVec3d>>,
}
impl BvhSet<BvhVec3d> for SelectableBuilderAdapter<'_> {
    fn size(&self) -> i32 {
        self.objects.len() as i32
    }
    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        let object = self.objects.find_key_int(index + 1)?;
        if let Some(bounds) = &self.persistent {
            return Ok(bounds[index as usize]);
        }
        selectable_regular_bounds(&**object)
    }
    fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        if !self.cached.get().is_valid() {
            self.cached.set(combined_element_boxes(self)?);
        }
        Ok(self.cached.get())
    }
    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        Ok(self.element_box(index)?.center_axis(axis))
    }
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.objects.swap_int(first + 1, second + 1)?;
        if let Some(bounds) = &mut self.persistent {
            bounds.swap(first as usize, second as usize);
        }
        Ok(())
    }
}
pub(super) fn build_selectable_subset(
    state: &mut SelectableSubsetState,
    view: Option<&SelectableBoundsView>,
) -> Result<(), Failure> {
    let persistent = view
        .map(|view| {
            state
                .objects
                .iter()
                .map(|object| selectable_persistent_bounds(&**object, view))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let mut adapter = SelectableBuilderAdapter {
        objects: &mut state.objects,
        persistent,
        cached: Cell::new(BvhBox::default()),
    };
    let bounds = adapter.bounding_box()?;
    state
        .builder
        .build(&mut adapter, &mut state.tree.write(), &bounds)
}
