//! `PrsDim_Dimension` linear layout: the flyout line, the label and arrow placement, the
//! linear dimension presentation and its flyout selection.
use super::dimension::{PrsDimDimensionMembers, PrsDimDimensionSource, label_position as lp};
use super::dimension_draw::{PrsDimDimensionDraw, PrsDimLabel, THE_3D_TEXT_MARGIN};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::prs3d::{
    Prs3dDimensionArrowOrientation, Prs3dDimensionTextHorizontalPosition,
    Prs3dDimensionTextVerticalPosition,
};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection};
use crate::select3d::{Select3DSensitiveGroup, Select3DSensitiveSegment};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, exceptions::program_error};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkgeombase::gce;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point3, Vector3};
use rustycad_tkservice::graphic3d::*;

fn make_dir(from: &Point3, to: &Point3) -> Result<Direction3, Failure> {
    Direction3::from_xyz(to.xyz() - from.xyz())
        .map_err(|_| Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result"))
}

fn null_aspect() -> Failure {
    Failure::with_message("Standard_NullObject", "PrsDim_Dimension: null aspect")
}

/// The flyout line ends of two points along the flyout direction.
fn flyout_ends(
    normal: Direction3,
    first: &Point3,
    second: &Point3,
    flyout: f64,
) -> Result<(Point3, Point3), Failure> {
    let target = make_dir(first, second)?;
    let flyout_dir = normal.crossed(target)?;
    let line1 = Line3::new(*first, flyout_dir);
    let line2 = Line3::new(*second, flyout_dir);
    Ok((
        elclib::value(elclib::parameter(&line1, *first)? + flyout, &line1),
        elclib::value(elclib::parameter(&line2, *second)? + flyout, &line2),
    ))
}

/// The native base body of `ComputeFlyoutLinePoints`.
pub fn prs_dim_dimension_flyout_line_points<D: PrsDimDimensionSource + ?Sized>(
    dimension: &D,
    first: &Point3,
    second: &Point3,
) -> Result<(Point3, Point3), Failure> {
    flyout_ends(
        dimension.get_plane().axis().direction(),
        first,
        second,
        dimension.get_flyout(),
    )
}

/// The placement of a linear label: its position flags and whether the arrows are outside.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrsDimLinearFit {
    /// The label position flags.
    pub label_position: i32,
    /// Whether the arrows point inward from outside.
    pub arrows_external: bool,
}

/// The non-virtual linear layout members of every dimension.
pub trait PrsDimDimensionLinear: PrsDimDimensionSource {
    /// `FitTextAlignmentForLinear`: the label position and arrow orientation fitting the
    /// label and arrows on the dimension line.
    /// Example: `dimension.fit_text_alignment_for_linear(&a, &b, false, h, width)?`.
    fn fit_text_alignment_for_linear(
        &self,
        first: &Point3,
        second: &Point3,
        one_side: bool,
        horizontal: Prs3dDimensionTextHorizontalPosition,
        label_width: f64,
    ) -> Result<PrsDimLinearFit, Failure> {
        let (begin, end) = flyout_ends(
            self.get_plane().axis().direction(),
            first,
            second,
            self.get_flyout(),
        )?;
        let aspect = self.attributes().dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let arrow_length = aspect
            .arrow_aspect()
            .get()
            .ok_or_else(null_aspect)?
            .length();
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
                let width = begin.distance(&end);
                let arrows = if one_side {
                    arrow_length + margin
                } else {
                    (arrow_length + margin) * 2.0
                };
                width < label_width + arrows
            }
        };
        let mut fit = PrsDimLinearFit {
            label_position: lp::NONE,
            arrows_external,
        };
        fit.label_position |= match horizontal {
            Prs3dDimensionTextHorizontalPosition::Left => lp::LEFT,
            Prs3dDimensionTextHorizontalPosition::Right => lp::RIGHT,
            Prs3dDimensionTextHorizontalPosition::Center => lp::H_CENTER,
            Prs3dDimensionTextHorizontalPosition::Fit => {
                let width = begin.distance(&end);
                let arrows = if one_side {
                    arrow_length
                } else {
                    2.0 * arrow_length
                };
                let content = if fit.arrows_external {
                    label_width
                } else {
                    label_width + arrows
                };
                if width < content {
                    lp::LEFT
                } else {
                    lp::H_CENTER
                }
            }
        };
        fit.label_position |= match aspect.text_vertical_position() {
            Prs3dDimensionTextVerticalPosition::Above => lp::ABOVE,
            Prs3dDimensionTextVerticalPosition::Below => lp::BELOW,
            Prs3dDimensionTextVerticalPosition::Center => lp::V_CENTER,
        };
        Ok(fit)
    }

    /// `GetTextPositionForLinear`: the label position of a linear dimension (the origin when
    /// invalid); `dimension.get_text_position_for_linear(&a, &b, false)?`.
    fn get_text_position_for_linear(
        &self,
        first: &Point3,
        second: &Point3,
        one_side: bool,
    ) -> Result<Point3, Failure> {
        if !self.is_valid() {
            return Ok(Point3::default());
        }
        let aspect = self.attributes().dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let mut label = self.get_value_string()?;
        if aspect.is_text_3d() {
            label.width += aspect.text_aspect().get().ok_or_else(null_aspect)?.height()
                * THE_3D_TEXT_MARGIN
                * 2.0;
        }
        let fit = self.fit_text_alignment_for_linear(
            first,
            second,
            one_side,
            aspect.text_horizontal_position(),
            label.width,
        )?;
        let normal = self.get_plane().axis().direction();
        let flyout_dir =
            normal.crossed(Direction3::from_vector(Vector3::between(*first, *second))?)?;
        let line1 = Line3::new(*first, flyout_dir);
        let line2 = Line3::new(*second, flyout_dir);
        let flyout = self.get_flyout();
        let begin = elclib::value(elclib::parameter(&line1, *first)? + flyout, &line1);
        let end = elclib::value(elclib::parameter(&line2, *second)? + flyout, &line2);
        let arrow_length = aspect
            .arrow_aspect()
            .get()
            .ok_or_else(null_aspect)?
            .length();
        let offset = if fit.arrows_external {
            aspect.extension_size() + arrow_length
        } else {
            aspect.extension_size()
        };
        Ok(match fit.label_position & lp::H_MASK {
            lp::LEFT => end.translated(Vector3::from(make_dir(first, second)?) * -offset),
            lp::RIGHT => begin.translated(Vector3::from(make_dir(first, second)?) * offset),
            lp::H_CENTER => Point3::from((begin.xyz() + end.xyz()) * 0.5),
            _ => Point3::default(),
        })
    }

    /// `AdjustParametersForLinear`: the extension size, alignment, flyout and plane placing
    /// the label at a custom position; false when the plane normal is along the dimension.
    /// Example: `dimension.adjust_parameters_for_linear(&text, &a, &b, &mut adjust)?`.
    fn adjust_parameters_for_linear(
        &self,
        text_pos: &Point3,
        first: &Point3,
        second: &Point3,
        adjust: &mut PrsDimLinearAdjust,
    ) -> Result<bool, Failure> {
        let aspect = self.attributes().dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_aspect)?;
        let arrow_length = aspect
            .arrow_aspect()
            .get()
            .ok_or_else(null_aspect)?
            .length();
        let target_dir = make_dir(first, second)?;
        let target_line = Line3::new(*first, target_dir);
        if !target_line.contains(*text_pos, precision::confusion()) {
            adjust.plane = gce::MakePln::through_points(*text_pos, *first, *second)?.value()?;
            adjust.is_plane_old = false;
        }
        // `GetPlane()` reads the dimension plane the caller passes as `thePlane`, just updated.
        let normal = adjust.plane.axis().direction();
        // The cross product raises before the parallel check, as natively.
        let positive = normal.crossed(target_dir)?;
        if normal.is_parallel(target_dir, precision::angular()) {
            return Ok(false);
        }
        let to_text = Vector3::between(*first, *text_pos);
        let cos = to_text.normalized()?.dot(Vector3::from(target_dir));
        let projection = first.translated(Vector3::from(target_dir) * to_text.magnitude() * cos);
        let flyout_vector = Vector3::between(projection, *text_pos);
        adjust.flyout = 0.0;
        if flyout_vector.magnitude() > precision::confusion() {
            let opposite =
                Direction3::from_vector(flyout_vector)?.is_opposite(positive, precision::angular());
            adjust.flyout = if opposite {
                -flyout_vector.magnitude()
            } else {
                flyout_vector.magnitude()
            };
        }
        let first_attach = first.translated(flyout_vector);
        let second_attach = second.translated(flyout_vector);
        if cos < 0.0 {
            adjust.alignment = Prs3dDimensionTextHorizontalPosition::Left;
            let size = text_pos.distance(&first_attach) - arrow_length;
            adjust.extension_size = if size < 0.0 { 0.0 } else { size };
        } else if projection.distance(first) > first.distance(second) {
            adjust.alignment = Prs3dDimensionTextHorizontalPosition::Right;
            let size = text_pos.distance(&second_attach) - arrow_length;
            adjust.extension_size = if size < 0.0 { 0.0 } else { size };
        } else {
            adjust.alignment = Prs3dDimensionTextHorizontalPosition::Center;
        }
        Ok(true)
    }

    /// `ComputeLinearFlyouts`: the two flyout segments of a linear dimension as one sensitive
    /// group; `dimension.compute_linear_flyouts(&selection, &owner, &a, &b)?`.
    fn compute_linear_flyouts(
        &self,
        selection: &Handle<SelectMgrSelection>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        first: &Point3,
        second: &Point3,
    ) -> Result<(), Failure> {
        let (end1, end2) = flyout_ends(
            self.get_plane().axis().direction(),
            first,
            second,
            self.get_flyout(),
        )?;
        let group = Select3DSensitiveGroup::new(owner, true);
        group.add(&Handle::new(Select3DSensitiveSegment::new(owner, *first, end1)).cast());
        group.add(&Handle::new(Select3DSensitiveSegment::new(owner, *second, end2)).cast());
        selection
            .get()
            .ok_or_else(null_aspect)?
            .add(&Handle::new(group).cast())
    }

    /// `DrawLinearDimension`: the label, dimension line, arrows, extensions and flyout lines
    /// between two points (equal points are a `Standard_ProgramError`).
    /// Example: `dimension.draw_linear_dimension(&prs, mode, &a, &b, false)?`.
    fn draw_linear_dimension(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
        first: &Point3,
        second: &Point3,
        one_side: bool,
    ) -> Result<(), Failure> {
        super::dimension_linear_draw::draw_linear(self, prs, mode, first, second, one_side)
    }
}
impl<T: PrsDimDimensionSource + ?Sized> PrsDimDimensionLinear for T {}

/// The in/out parameters of `AdjustParametersForLinear`.
#[derive(Clone, Copy, Debug)]
pub struct PrsDimLinearAdjust {
    /// `theExtensionSize`.
    pub extension_size: f64,
    /// `theAlignment`.
    pub alignment: Prs3dDimensionTextHorizontalPosition,
    /// `theFlyout`.
    pub flyout: f64,
    /// `thePlane`.
    pub plane: Plane3,
    /// `theIsPlaneOld`.
    pub is_plane_old: bool,
}

/// The label drawn at the center of a dimension line: its group and line break.
pub(super) fn line_aspect_of<D: PrsDimDimensionSource + ?Sized>(
    dimension: &D,
) -> Result<Handle<dyn Graphic3dAspectsSource>, Failure> {
    let aspect = dimension.attributes().dimension_aspect();
    let aspect = aspect.get().ok_or_else(null_aspect)?;
    Ok(aspect.line_aspect().aspect().cast())
}

/// An empty label (`TCollection_ExtendedString::EmptyString()`).
pub(super) fn empty_label(font: &PrsDimLabel) -> PrsDimLabel {
    PrsDimLabel {
        text: ExtendedString::default(),
        width: 0.0,
        font: font.font.clone(),
        plan: font.plan.clone(),
    }
}

/// `throw Standard_ProgramError`.
pub(super) fn program(message: &str) -> Failure {
    program_error(message)
}
