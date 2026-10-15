//! `DsgPrs_ArrowSide` and `DsgPrs::ComputeSymbol`: the arrow and point end symbols of a
//! dimension line.
use super::support::{DsgPrsPresentation, add_array, new_group, points, ring_marker};
use crate::prs3d::{Prs3dArrow, Prs3dDimensionAspect};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dGroup, Graphic3dStructureMembers as _};

/// `DsgPrs_ArrowSide`: which ends of a dimension get an arrow or a point.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DsgPrsArrowSide {
    /// `DsgPrs_AS_NONE`.
    #[default]
    None = 0,
    /// `DsgPrs_AS_FIRSTAR`.
    FirstAr = 1,
    /// `DsgPrs_AS_LASTAR`.
    LastAr = 2,
    /// `DsgPrs_AS_BOTHAR`.
    BothAr = 3,
    /// `DsgPrs_AS_FIRSTPT`.
    FirstPt = 4,
    /// `DsgPrs_AS_LASTPT`.
    LastPt = 5,
    /// `DsgPrs_AS_BOTHPT`.
    BothPt = 6,
    /// `DsgPrs_AS_FIRSTAR_LASTPT`.
    FirstArLastPt = 7,
    /// `DsgPrs_AS_FIRSTPT_LASTAR`.
    FirstPtLastAr = 8,
}

/// The two ends of a symbol: points and arrow directions.
#[derive(Clone, Copy, Debug)]
pub struct DsgPrsSymbolEnds {
    /// `pt1`.
    pub first: Point3,
    /// `pt2`.
    pub last: Point3,
    /// `dir1`.
    pub first_direction: Direction3,
    /// `dir2`.
    pub last_direction: Direction3,
}

/// `DsgPrs::ComputeSymbol(prs, LA, pt1, pt2, dir1, dir2, ArrowSide, drawFromCenter)`:
/// a new group with the dimension line aspect and the requested arrows and points.
/// Example: `dsg_prs_compute_symbol(&prs, &aspect, &ends, DsgPrsArrowSide::BothAr, true)?`.
pub fn dsg_prs_compute_symbol(
    prs: &DsgPrsPresentation,
    aspect: &Handle<Prs3dDimensionAspect>,
    ends: &DsgPrsSymbolEnds,
    side: DsgPrsArrowSide,
    draw_from_center: bool,
) -> Result<(), Failure> {
    let Some(la) = aspect.get() else {
        return Err(super::support::null_object(
            "DsgPrs::ComputeSymbol: the aspect is null",
        ));
    };
    let group = new_group(prs)?;
    let marker = ring_marker(la)?;
    if let (Some(value), Some(line)) = (group.get(), la.line_aspect().get()) {
        value.set_group_primitives_aspect(&line.aspect().cast());
    }
    let arrow = |point: &Point3, direction: &Direction3| -> Result<(), Failure> {
        let (angle, length) = super::support::arrow_angle_length(la);
        Prs3dArrow::draw(&group, point, direction, angle, length)
    };
    match side {
        DsgPrsArrowSide::None => {}
        DsgPrsArrowSide::FirstAr => arrow(&ends.first, &ends.first_direction)?,
        DsgPrsArrowSide::LastAr => arrow(&ends.last, &ends.last_direction)?,
        DsgPrsArrowSide::BothAr => {
            arrow(&ends.first, &ends.first_direction)?;
            arrow(&ends.last, &ends.last_direction)?;
        }
        DsgPrsArrowSide::FirstPt if draw_from_center => current_point(prs, &ends.first)?,
        DsgPrsArrowSide::FirstPt => {}
        DsgPrsArrowSide::LastPt => current_point(prs, &ends.last)?,
        DsgPrsArrowSide::BothPt if draw_from_center => {
            set_group_aspect(&group, &marker.clone().cast());
            add_array(&group, &points(&[ends.first, ends.last])?);
        }
        DsgPrsArrowSide::BothPt => {}
        DsgPrsArrowSide::FirstArLastPt => {
            arrow(&ends.first, &ends.first_direction)?;
            marked_point(&group, &marker, &ends.last)?;
        }
        DsgPrsArrowSide::FirstPtLastAr => {
            arrow(&ends.last, &ends.last_direction)?;
            if draw_from_center {
                marked_point(&group, &marker, &ends.first)?;
            }
        }
    }
    Ok(())
}

fn set_group_aspect(
    group: &Handle<dyn Graphic3dGroup>,
    aspect: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dAspectsSource>,
) {
    if let Some(value) = group.get() {
        value.set_group_primitives_aspect(aspect);
    }
}

// `SetPrimitivesAspect(aMarkerAsp)` then one point.
fn marked_point(
    group: &Handle<dyn Graphic3dGroup>,
    marker: &Handle<rustycad_tkservice::graphic3d::Graphic3dAspectMarker3d>,
    point: &Point3,
) -> Result<(), Failure> {
    if let Some(value) = group.get() {
        value.set_primitives_aspect(&marker.clone().cast());
    }
    add_array(group, &points(&[*point])?);
    Ok(())
}

// One point on `aPresentation->CurrentGroup()` (the group just created).
fn current_point(prs: &DsgPrsPresentation, point: &Point3) -> Result<(), Failure> {
    let group = super::support::structure(prs)?.current_group()?;
    add_array(&group, &points(&[*point])?);
    Ok(())
}
