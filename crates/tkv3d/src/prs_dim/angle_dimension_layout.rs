//! `PrsDim_AngleDimension` label layout: the text alignment fitting, the parameters adjusted
//! to a custom label position, and the automatic label position.
use super::angle_dimension::{PrsDimAngleDimension, attach_at};
use super::dimension::{PrsDimDimensionMembers, label_position as lp};
use super::dimension_draw::{PrsDimDimensionDraw, THE_3D_TEXT_MARGIN};
use super::{PrsDim, PrsDimLinearFit};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::prs3d::{
    Prs3dDimensionArrowOrientation, Prs3dDimensionAspect, Prs3dDimensionTextHorizontalPosition,
    Prs3dDimensionTextVerticalPosition,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::gce::{MakeCirc, MakeLin, Root};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Direction3, Point3, Vector3};
use std::f64::consts::PI;

/// The in/out parameters of `PrsDim_AngleDimension::AdjustParameters`.
#[derive(Clone, Copy, Debug)]
pub struct PrsDimAngleAdjust {
    /// `theExtensionSize`.
    pub extension_size: f64,
    /// `theAlignment`.
    pub alignment: Prs3dDimensionTextHorizontalPosition,
    /// `theFlyout`.
    pub flyout: f64,
}

pub(super) fn null_aspect() -> Failure {
    Failure::with_message("Standard_NullObject", "PrsDim_Dimension: null aspect")
}

/// The dimension aspect of the drawer.
pub(super) fn dimension_aspect_of(
    dimension: &PrsDimAngleDimension,
) -> Result<Handle<Prs3dDimensionAspect>, Failure> {
    let aspect = dimension.attributes().dimension_aspect();
    if aspect.is_null() {
        return Err(null_aspect());
    }
    Ok(aspect)
}

/// The label width with the 3D text margins.
pub(super) fn margin_width(aspect: &Prs3dDimensionAspect, width: f64) -> Result<f64, Failure> {
    if !aspect.is_text_3d() {
        return Ok(width);
    }
    let height = aspect.text_aspect().get().ok_or_else(null_aspect)?.height();
    Ok(width + height * THE_3D_TEXT_MARGIN * 2.0)
}

impl PrsDimAngleDimension {
    /// `FitTextAlignment`: the label position and arrow orientation fitting the label and
    /// arrows between the attach points at the flyout.
    /// Example: `dimension.fit_text_alignment(h, width)?`.
    pub fn fit_text_alignment(
        &self,
        horizontal: Prs3dDimensionTextHorizontalPosition,
        label_width: f64,
    ) -> Result<PrsDimLinearFit, Failure> {
        let aspect = dimension_aspect_of(self)?;
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let arrow_length = aspect
            .arrow_aspect()
            .get()
            .ok_or_else(null_aspect)?
            .length();
        let (first_attach, second_attach) = self.attach_points(self.get_flyout())?;
        let width = Vector3::between(first_attach, second_attach).magnitude();
        let arrows_external = match aspect.arrow_orientation() {
            Prs3dDimensionArrowOrientation::External => true,
            Prs3dDimensionArrowOrientation::Internal => false,
            Prs3dDimensionArrowOrientation::Fit => {
                let margin = if aspect.is_text_3d() {
                    aspect.text_aspect().get().ok_or_else(null_aspect)?.height()
                        * THE_3D_TEXT_MARGIN
                } else {
                    0.0
                };
                width < label_width + (arrow_length + margin) * 2.0
            }
        };
        let mut label_position = match horizontal {
            Prs3dDimensionTextHorizontalPosition::Left => lp::LEFT,
            Prs3dDimensionTextHorizontalPosition::Right => lp::RIGHT,
            Prs3dDimensionTextHorizontalPosition::Center => lp::H_CENTER,
            Prs3dDimensionTextHorizontalPosition::Fit => {
                let content = if arrows_external {
                    label_width
                } else {
                    label_width + arrow_length * 2.0
                };
                if width < content {
                    lp::LEFT
                } else {
                    lp::H_CENTER
                }
            }
        };
        label_position |= match aspect.text_vertical_position() {
            Prs3dDimensionTextVerticalPosition::Above => lp::ABOVE,
            Prs3dDimensionTextVerticalPosition::Below => lp::BELOW,
            Prs3dDimensionTextVerticalPosition::Center => lp::V_CENTER,
        };
        Ok(PrsDimLinearFit {
            label_position,
            arrows_external,
        })
    }

    /// `AdjustParameters`: the flyout, alignment and extension size placing the label at a
    /// position (centered between the attach points or their reflections, else on the
    /// nearest extension); unchanged when the circle cannot be built.
    /// Example: `dimension.adjust_parameters(&text, &mut adjust)?`.
    pub fn adjust_parameters(
        &self,
        text_pos: &Point3,
        adjust: &mut PrsDimAngleAdjust,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect_of(self)?;
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let arrow_length = aspect
            .arrow_aspect()
            .get()
            .ok_or_else(null_aspect)?
            .length();
        let (first, center, second) = self.points();
        let radius = Vector3::between(center, *text_pos).magnitude();
        let (first_attach, second_attach) = self.attach_points(radius)?;
        let maker = MakeCirc::from_center_plane(center, self.get_plane(), radius)?;
        if !maker.is_done() {
            return Ok(());
        }
        let circle = maker.value()?;
        adjust.extension_size = arrow_length;
        adjust.alignment = Prs3dDimensionTextHorizontalPosition::Center;
        let mut begin = elclib::parameter(&circle, first_attach)?;
        let mut end = elclib::parameter(&circle, second_attach)?;
        if end < begin {
            std::mem::swap(&mut begin, &mut end);
        }
        let (begin, end) =
            elclib::adjust_periodic(0.0, PI * 2.0, precision::p_confusion(), begin, end);
        let text_par = elclib::parameter(&circle, *text_pos)?;
        if text_par > begin && text_par < end {
            adjust.flyout = radius;
            return Ok(());
        }
        let (begin, end) = elclib::adjust_periodic(
            0.0,
            PI * 2.0,
            precision::p_confusion(),
            begin + PI,
            end + PI,
        );
        if text_par > begin && text_par < end {
            adjust.flyout = -radius;
            return Ok(());
        }
        let first_line = MakeLin::from_points(center, first)?.value()?;
        let second_line = MakeLin::from_points(center, second)?.value()?;
        let first_proj = PrsDim::nearest_on_line(&first_line, text_pos)?;
        let second_proj = PrsDim::nearest_on_line(&second_line, text_pos)?;
        let first_dist = first_proj.distance(text_pos);
        let second_dist = second_proj.distance(text_pos);
        let (projection, distance, side, alignment) = if first_dist <= second_dist {
            (
                first_proj,
                first_dist,
                first,
                Prs3dDimensionTextHorizontalPosition::Left,
            )
        } else {
            (
                second_proj,
                second_dist,
                second,
                Prs3dDimensionTextHorizontalPosition::Right,
            )
        };
        let radius = center.distance(&projection);
        let extension = distance - arrow_length;
        adjust.extension_size = if extension < 0.0 { 0.0 } else { extension };
        adjust.alignment = alignment;
        let positive = attach_at(&center, &side, radius)?;
        adjust.flyout = if projection.distance(&positive) > precision::confusion() {
            -radius
        } else {
            radius
        };
        Ok(())
    }
}

/// `GetTextPosition` of a valid dimension without a fixed position: the middle of the arc,
/// or the end of the left or right extension.
pub(super) fn text_position(dimension: &PrsDimAngleDimension) -> Result<Point3, Failure> {
    let aspect = dimension_aspect_of(dimension)?;
    let aspect = aspect.get().ok_or_else(null_aspect)?;
    let label = dimension.get_value_string()?;
    let width = margin_width(aspect, label.width)?;
    let center = dimension.center_point();
    let (first_attach, second_attach) = dimension.attach_points(dimension.get_flyout())?;
    let fit = dimension.fit_text_alignment(aspect.text_horizontal_position(), width)?;
    let offset = || -> Result<f64, Failure> {
        let size = aspect.extension_size();
        Ok(if fit.arrows_external {
            size + aspect
                .arrow_aspect()
                .get()
                .ok_or_else(null_aspect)?
                .length()
        } else {
            size
        })
    };
    let plane_normal = || {
        Direction3::from_vector(
            Vector3::between(first_attach, second_attach)
                .crossed(Vector3::between(center, first_attach)),
        )
    };
    Ok(match fit.label_position & lp::H_MASK {
        lp::H_CENTER => dimension.get_center_on_arc(&first_attach, &second_attach, &center)?,
        lp::LEFT => {
            // `gp_Dir ^ gp_Vec` converts the vector to a direction first.
            let direction = plane_normal()?.crossed(Direction3::from_vector(Vector3::between(
                center,
                first_attach,
            ))?)?;
            first_attach.translated(Vector3::from(direction) * -offset()?)
        }
        lp::RIGHT => {
            // `gp_Dir ^ gp_Vec` converts the vector to a direction first.
            let direction = plane_normal()?.crossed(Direction3::from_vector(Vector3::between(
                center,
                second_attach,
            ))?)?;
            second_attach.translated(Vector3::from(direction) * offset()?)
        }
        _ => gp::origin(),
    })
}
