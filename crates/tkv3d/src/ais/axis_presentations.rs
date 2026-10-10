//! Native axis presentation, all-mode segment selection and datum overrides.
use super::AisAxis;
use crate::{
    dsg_prs::DsgPrsXYZAxisPresentation, prs_mgr::*, select_mgr::*,
    select3d::Select3DSensitiveSegment, std_prs::StdPrsCurve,
};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkservice::graphic3d::*;
impl AisAxis {
    fn compute_axis(&self, prs: &Handle<dyn Graphic3dStructureSource>) -> Result<(), Failure> {
        prs.set_infinite_state(self.is_infinite());
        let g = self.geometry.read().clone();
        if !g.xyz {
            let curve = GeomAdaptorCurve::with_curve(&g.component.cast())?;
            return StdPrsCurve::add(prs, &curve, &self.attributes(), true);
        }
        let label = if self.attributes().datum_aspect().to_draw_labels() {
            g.label
        } else {
            ""
        };
        DsgPrsXYZAxisPresentation::add(
            prs,
            &g.line,
            &g.direction,
            g.length,
            label,
            &g.first,
            &g.last,
        )
    }
}
impl PrsMgrPresentableObject for AisAxis {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        self.compute_axis(prs)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.set_axis_color(color);
        Ok(())
    }
    fn unset_color(&self) -> Result<(), Failure> {
        self.unset_axis_color();
        Ok(())
    }
    fn set_width(&self, width: f64) -> Result<(), Failure> {
        self.set_axis_width(width)
    }
    fn unset_width(&self) -> Result<(), Failure> {
        self.unset_axis_width()
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisAxis {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let g = self.geometry.read();
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 3);
        selection.add(
            &Handle::new(Select3DSensitiveSegment::new(
                &owner.cast(),
                g.first,
                g.last,
            ))
            .cast(),
        )
    }
}
