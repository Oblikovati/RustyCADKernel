//! Cached screen-space border/fill presentation; native callbacks append rather than clear.
use super::{AisRubberBand, AisRubberBandSource};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::*;
/// Native presentation hook for subclass reuse; `ais_rubber_band_compute(&*band, &prs, 0)?`.
pub fn ais_rubber_band_compute<O: AisRubberBandSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    if mode != 0 {
        return Ok(());
    }
    if object.is_filling() && super::ais_rubber_band_fill_triangles(object)? {
        let group = prs.new_group()?;
        group.set_group_primitives_aspect(&object.attributes().shading_aspect().aspect().cast());
        group.add_primitive_array(
            &object.rubber_band_base().fields.triangles.read().cast(),
            true,
        );
    }
    rubber_band_update_border(object)?;
    let group = prs.new_group()?;
    group.set_group_primitives_aspect(&object.attributes().line_aspect().aspect().cast());
    group.add_primitive_array(
        &object.rubber_band_base().fields.borders.read().cast(),
        true,
    );
    Ok(())
}
fn rubber_band_update_border<O: AisRubberBandSource + ?Sized>(object: &O) -> Result<(), Failure> {
    let points = object.points().read();
    let closed = object.is_polygon_closed();
    let vertex_count = points.length() + i32::from(closed);
    let mut cache = object.rubber_band_base().fields.borders.write();
    let allocate = cache.is_null() || cache.vertex_number() != vertex_count;
    if allocate {
        *cache = Handle::new(Graphic3dArrayOfPolylines::new(vertex_count, 0, 0, 0)?);
    }
    for (index, point) in points.iter().enumerate() {
        rubber_band_border_vertex(&cache, *point, index as i32 + 1, allocate)?;
    }
    if closed {
        rubber_band_border_vertex(&cache, *points.value(1)?, vertex_count, allocate)?;
    }
    Ok(())
}
fn rubber_band_border_vertex(
    array: &Handle<Graphic3dArrayOfPolylines>,
    point: Vec2<i32>,
    index: i32,
    allocate: bool,
) -> Result<(), Failure> {
    if allocate {
        array.add_vertex_f64(f64::from(point.x), f64::from(point.y), 0.)?;
        return Ok(());
    }
    array.set_vertice(index, point.x as f32, point.y as f32, 0.)
}
impl PrsMgrPresentableObject for AisRubberBand {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.rubber_band_base().interactive.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_rubber_band_compute(self, &prs.observe(), mode)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisRubberBand {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.rubber_band_base().interactive.selectable
    }
    fn compute_selection(
        &self,
        _: &dyn HandleReference<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
