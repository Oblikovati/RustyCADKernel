//! Native presentation and sensitive-segment construction; units are read for every selection.
use super::{AisLine, line_appearance::LineDatumAppearance};
use crate::{prs_mgr::*, select_mgr::*, select3d::Select3DSensitiveSegment, std_prs::StdPrsCurve};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::GeomLine, geom_adaptor::GeomAdaptorCurve};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers, Graphic3dStructureSource};
impl AisLine {
    fn compute_line(&self, prs: &Handle<dyn Graphic3dStructureSource>) -> Result<(), Failure> {
        if self.geometry.read().segment {
            return self.compute_segment(prs);
        }
        let curve = GeomAdaptorCurve::with_curve(&self.line().cast())?;
        StdPrsCurve::add(prs, &curve, &self.attributes(), true)?;
        prs.set_infinite_state(true);
        Ok(())
    }
    fn compute_segment(&self, prs: &Handle<dyn Graphic3dStructureSource>) -> Result<(), Failure> {
        let geometry = self.geometry.read().clone();
        let (first, last) = (geometry.start.pnt(), geometry.end.pnt());
        let line = Handle::new(GeomLine::new(Axis1::new(
            first,
            Direction3::from_xyz(last.xyz() - first.xyz())?,
        )));
        self.geometry.write().component = line.clone();
        let curve = GeomAdaptorCurve::with_range(&line.cast(), 0., first.distance(&last))?;
        StdPrsCurve::add(prs, &curve, &self.attributes(), true)
    }
    fn selection_ends(&self) -> Result<(Point3, Point3), Failure> {
        let geometry = self.geometry.read().clone();
        if geometry.segment {
            return Ok((geometry.start.pnt(), geometry.end.pnt()));
        }
        let axis = geometry.component.position();
        let length = self.units.write().any_to_ls(250000., b"mm")?;
        let offset = axis.direction().xyz() * length;
        Ok((
            Point3::from(axis.location().xyz() + offset),
            Point3::from(axis.location().xyz() - offset),
        ))
    }
}
impl PrsMgrPresentableObject for AisLine {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        self.compute_line(prs)
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
        self.unset_line_width(true)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisLine {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != 0 {
            return Ok(());
        }
        let (first, last) = self.selection_ends()?;
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 5);
        selection
            .add(&Handle::new(Select3DSensitiveSegment::new(&owner.cast(), first, last)).cast())
    }
}
