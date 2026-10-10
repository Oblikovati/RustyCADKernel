//! Adaptive presentations and snapshot sensitivity; native circle callbacks ignore display/selection mode.
use super::{AisCircle, line_appearance::LineDatumAppearance};
use crate::{
    prs_mgr::*,
    select_mgr::*,
    select3d::{Select3DSensitiveCircle, Select3DSensitivePoly},
    std_prs::StdPrsDeflectionCurve,
};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
impl AisCircle {
    fn compute_circle(&self, prs: &Handle<dyn Graphic3dStructureSource>) -> Result<(), Failure> {
        let geometry = self.geometry.read().clone();
        let curve = if geometry.arc {
            GeomAdaptorCurve::with_range(
                &geometry.component.cast(),
                geometry.bounds[0],
                geometry.bounds[1],
            )?
        } else {
            GeomAdaptorCurve::with_curve(&geometry.component.cast())?
        };
        let drawer = self.attributes();
        let previous = drawer.deviation_coefficient();
        drawer.set_deviation_coefficient(1.0e-5);
        // OCCT restores only after successful computation; failure leaves the tighter coefficient.
        StdPrsDeflectionCurve::add(prs, &curve, &drawer, true)?;
        drawer.set_deviation_coefficient(previous);
        Ok(())
    }
}
impl PrsMgrPresentableObject for AisCircle {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        self.compute_circle(prs)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        self.set_line_color(color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        self.unset_line_color()
    }
    fn set_width(&self, width: f64) -> Result<(), Failure> {
        self.set_line_width(width)
    }
    fn unset_width(&self) -> Result<(), Failure> {
        self.unset_line_width(false)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisCircle {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let geometry = self.geometry.read().clone();
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 0);
        let circle = geometry.component.circ()?;
        if geometry.arc {
            return selection.add(
                &Handle::new(Select3DSensitivePoly::circle(
                    &owner.cast(),
                    &circle,
                    geometry.bounds[0],
                    geometry.bounds[1],
                    geometry.filled,
                    12,
                )?)
                .cast(),
            );
        }
        selection.add(
            &Handle::new(Select3DSensitiveCircle::new(
                &owner.cast(),
                &circle,
                geometry.filled,
            ))
            .cast(),
        )
    }
}
