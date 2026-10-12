//! Native mode-zero label presentation, persistence transitions and rectangle selection.
use super::AisTextLabel;
use crate::{prs_mgr::*, prs3d::Prs3dText, select_mgr::*, select3d::*};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    ncollection::Vec4,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Axis2, Point3, Transform3};
use rustycad_tkservice::graphic3d::*;
type LabelMeasure = Option<(Point3, f64, f64)>;
impl AisTextLabel {
    fn presentation_position(&self) -> Result<Point3, Failure> {
        let position = self.position();
        let zoomable = self.label_aspect().aspect().is_text_zoomable();
        let pers = self.transform_persistence();
        if self.has_orientation_3d() {
            self.label_aspect()
                .aspect()
                .set_text_zoomable(self.has_flipping());
            return self.replace_label_persistence(Graphic3dTransModeFlags::ZOOM_PERS, position);
        }
        if zoomable || pers.is_null() || pers.mode() != Graphic3dTransModeFlags::TMF_2D {
            let mode = label_screen_persistence(zoomable);
            return self.replace_label_persistence(mode, position);
        }
        Ok(position)
    }
    fn replace_label_persistence(
        &self,
        mode: Graphic3dTransModeFlags,
        position: Point3,
    ) -> Result<Point3, Failure> {
        self.set_transform_persistence(
            &Handle::new(Graphic3dTransformPers::with_point(mode, &position)?).cast(),
        );
        Ok(Point3::default())
    }
    pub(super) fn compute_label(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let position = self.presentation_position()?;
        let measured = self.calculate_label_params(position)?;
        let group = prs.current_group()?;
        self.submit_label_text(&group, position, measured)?;
        self.apply_label_bounds(&group, position, measured)?;
        Ok(())
    }
    fn apply_label_bounds(
        &self,
        group: &Handle<dyn Graphic3dGroup>,
        position: Point3,
        measured: LabelMeasure,
    ) -> Result<(), Failure> {
        let Some((mut center, width, height)) = measured else {
            return Ok(());
        };
        let plane = self.calculate_label_trsf(position, &mut center)?;
        let mut bounds = group.bounding_box();
        for point in [
            Point3::new(-width * 0.5, -height * 0.5, 0.),
            Point3::new(width * 0.5, height * 0.5, 0.),
        ] {
            let point = point.transformed(plane);
            bounds.add(&Vec4::new(
                point.x() as f32,
                point.y() as f32,
                point.z() as f32,
                1.,
            ));
        }
        group.set_bounding_box(bounds);
        Ok(())
    }
    fn submit_label_text(
        &self,
        group: &Handle<dyn Graphic3dGroup>,
        position: Point3,
        measured: LabelMeasure,
    ) -> Result<(), Failure> {
        if !self.has_orientation_3d() {
            let text = Prs3dText::draw(group, &self.label_aspect(), &self.text(), &position);
            text.set_text_formatter(self.text_formatter());
            return Ok(());
        }
        self.submit_oriented_label(group, position, measured)
    }
    fn submit_oriented_label(
        &self,
        group: &Handle<dyn Graphic3dGroup>,
        position: Point3,
        measured: LabelMeasure,
    ) -> Result<(), Failure> {
        let mut axes = self.orientation_3d();
        self.begin_label_flipping(group, axes, measured)?;
        axes.set_location(position);
        let own_anchor = self.has_own_anchor_point() && !self.has_flipping();
        let text =
            Prs3dText::draw_oriented(group, &self.label_aspect(), &self.text(), &axes, own_anchor);
        text.set_text_formatter(self.text_formatter());
        if self.has_flipping() && measured.is_some() {
            group.set_flipping_options(false, &Axis2::default());
        }
        Ok(())
    }
    fn begin_label_flipping(
        &self,
        group: &Handle<dyn Graphic3dGroup>,
        axes: Axis2,
        measured: LabelMeasure,
    ) -> Result<(), Failure> {
        let center = measured.map_or(Point3::default(), |row| row.0);
        if self.has_flipping() {
            let flip = Axis2::with_x_direction(center, axes.direction(), axes.x_direction())?;
            // Native 3d097a0 closes this bracket only when font measurement succeeded.
            group.set_flipping_options(true, &flip);
        }
        Ok(())
    }
    fn label_rectangle(
        &self,
        position: Point3,
        mut center: Point3,
        width: f64,
        height: f64,
    ) -> Result<Vec<Point3>, Failure> {
        let plane: Transform3 = self.calculate_label_trsf(position, &mut center)?;
        Ok([(-1., -1.), (-1., 1.), (1., 1.), (1., -1.), (-1., -1.)]
            .map(|(x, y)| Point3::new(x * width * 0.5, y * height * 0.5, 0.).transformed(plane))
            .to_vec())
    }
}
impl SelectMgrSelectableObject for AisTextLabel {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != 0 {
            return Ok(());
        }
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 10);
        let position = text_label_selection_position(self);
        let Some((center, width, height)) = self.calculate_label_params(position)? else {
            let point = Select3DSensitivePoint::new(&owner.cast(), position);
            return selection.observe().add(&Handle::new(point).cast());
        };
        let points = self.label_rectangle(position, center, width, height)?;
        let kind = Select3DTypeOfSensitivity::Interior;
        let face = Select3DSensitiveFace::new(&owner.cast(), &points, kind)?;
        selection.observe().add(&Handle::new(face).cast())
    }
}

fn label_screen_persistence(zoomable: bool) -> Graphic3dTransModeFlags {
    if zoomable {
        return Graphic3dTransModeFlags::ROTATE_PERS;
    }
    Graphic3dTransModeFlags::ZOOM_ROTATE_PERS
}

fn text_label_selection_position(label: &AisTextLabel) -> Point3 {
    let persistence = label.transform_persistence();
    if !persistence.is_null() && persistence.mode() != Graphic3dTransModeFlags::TMF_2D {
        return Point3::default();
    }
    label.position()
}
