//! Native point highlights use their own mode; selection accepts every requested mode.
use super::AisPoint;
use crate::{
    prs_mgr::*,
    prs3d::{Prs3dDrawer, Prs3dPointAspect},
    select_mgr::*,
    select3d::Select3DSensitivePoint,
    std_prs::StdPrsPoint,
};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
fn point_highlight_drawer(color: NamedColor, layer: i32) -> Result<Handle<Prs3dDrawer>, Failure> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.set_display_mode(-99);
    drawer.set_point_aspect(Handle::new(Prs3dPointAspect::new(
        AspectTypeOfMarker::Plus,
        Color::named(color),
        3.,
    )?));
    drawer.set_color(Color::named(color));
    drawer.set_z_layer(layer);
    Ok(drawer)
}
impl AisPoint {
    pub(super) fn initialize_point_highlights(&self) -> Result<(), Failure> {
        self.set_hilight_attributes(&point_highlight_drawer(
            NamedColor::GRAY80,
            GRAPHIC3D_ZLAYER_ID_UNKNOWN,
        )?);
        self.set_dynamic_hilight_attributes(&point_highlight_drawer(
            NamedColor::CYAN1,
            GRAPHIC3D_ZLAYER_ID_TOP,
        )?);
        Ok(())
    }
    fn compute_point(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        prs.set_infinite_state(self.is_infinite());
        if mode == 0 {
            return StdPrsPoint::add(prs, &self.component(), &self.attributes());
        }
        if mode != -99 {
            return Ok(());
        }
        // Native 3d097a0 always uses the static highlight drawer's marker in mode -99.
        let group = prs.current_group()?;
        group.set_primitives_aspect(&self.hilight_attributes().point_aspect().aspect().cast());
        let points = Handle::new(Graphic3dArrayOfPoints::new(1, 0)?);
        points.add_vertex_pnt(&self.component().pnt())?;
        group.add_primitive_array(&points.cast(), true);
        Ok(())
    }
}
impl PrsMgrPresentableObject for AisPoint {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0 || mode == -99
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.compute_point(prs, mode)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.set_point_color(color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        self.unset_point_color()
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisPoint {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 10);
        selection.add(
            &Handle::new(Select3DSensitivePoint::new(
                &owner.cast(),
                self.component().pnt(),
            ))
            .cast(),
        )
    }
}
