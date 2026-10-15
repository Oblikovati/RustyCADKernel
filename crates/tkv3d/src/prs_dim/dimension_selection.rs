//! `PrsDim_Dimension::ComputeSelection`: the dimension line, arrows and label sensitives of
//! the selection modes, built from the recorded selection geometry.
use super::dimension::{PrsDimDimensionMembers, PrsDimDimensionSource};
use super::dimension_draw::points_for_arrow;
use super::{PrsDimDimensionOwner, PrsDimDimensionSelectionMode};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::*;
use crate::select3d::*;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{self, Axis2, Axis3, Circle3, Point3, Transform3};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;

/// `THE_NEUTRAL_SEL_PRIORITY`.
const NEUTRAL_PRIORITY: i32 = 5;
/// `THE_LOCAL_SEL_PRIORITY`.
const LOCAL_PRIORITY: i32 = 6;

fn null_aspect() -> Failure {
    Failure::with_message("Standard_NullObject", "PrsDim_Dimension: null aspect")
}

/// The native `PrsDim_Dimension::ComputeSelection` body.
/// Example: `prs_dim_dimension_compute_selection(&*dimension, &selection, 0)?`.
pub fn prs_dim_dimension_compute_selection<D: PrsDimDimensionSource + ?Sized>(
    dimension: &D,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let geometry = dimension
        .dimension_base()
        .fields
        .read()
        .selection_geom
        .clone();
    if !geometry.is_computed {
        return Ok(());
    }
    let Some(mode) = PrsDimDimensionSelectionMode::from_i32(mode) else {
        // An unknown mode leaves the owner null natively; nothing is added.
        return Ok(());
    };
    let this: Handle<dyn SelectMgrSelectableObject> = dimension.presentable_base().this().cast();
    let owner: Handle<dyn SelectMgrEntityOwnerSource> = match mode {
        PrsDimDimensionSelectionMode::All => {
            SelectMgrEntityOwner::with_selectable(&this, NEUTRAL_PRIORITY).cast()
        }
        _ => PrsDimDimensionOwner::new(&this, mode, LOCAL_PRIORITY).cast(),
    };
    let target = selection.get().ok_or_else(null_aspect)?;
    let normal = dimension.get_plane().axis().direction();
    let aspect = dimension.attributes().dimension_aspect();
    let aspect = aspect.get().ok_or_else(null_aspect)?;
    if mode != PrsDimDimensionSelectionMode::Text {
        let group = Select3DSensitiveGroup::new(&owner, true);
        for curve in &geometry.dimension_line {
            group.add(&Handle::new(Select3DSensitiveCurve::from_points(&owner, curve)?).cast());
        }
        let arrow = aspect.arrow_aspect();
        let arrow = arrow.get().ok_or_else(null_aspect)?;
        let (length, angle) = (arrow.length(), arrow.angle());
        for recorded in &geometry.arrows {
            let peak = recorded.position;
            let (side1, side2) =
                points_for_arrow(&peak, &recorded.direction, &normal, length, angle)?;
            group.add(&triangle(&owner, peak, side1, side2));
            if !aspect.is_arrows_3d() {
                continue;
            }
            let ortho = recorded.direction.crossed(normal)?;
            let (side1, side2) =
                points_for_arrow(&peak, &recorded.direction, &ortho, length, angle)?;
            group.add(&triangle(&owner, peak, side1, side2));
        }
        target.add(&Handle::new(group).cast())?;
    }
    if mode != PrsDimDimensionSelectionMode::Line {
        let axes = Axis2::with_x_direction(geometry.text_pos, normal, geometry.text_dir)?;
        let text: Handle<dyn Select3DSensitiveEntity> = if aspect.is_text_3d() {
            let (dx, dy) = (geometry.text_width * 0.5, geometry.text_height * 0.5);
            let mut plane = Transform3::default();
            plane.set_transformation_between(Axis3::from_axis2(axes), Axis3::from_axis2(gp::xoy()));
            let corners = [(-dx, -dy), (-dx, dy), (dx, dy), (dx, -dy)]
                .map(|(x, y)| Point3::new(x, y, 0.0).transformed(plane));
            let triangles = [Triangle::new(1, 2, 3), Triangle::new(1, 3, 4)];
            let polygon = Handle::new(Triangulation::from_nodes(
                &Array1::from_shared(&corners, 1),
                &Array1::from_shared(&triangles, 1),
            )?);
            Handle::new(Select3DSensitiveTriangulation::new(
                &owner,
                &polygon,
                &Location::default(),
                true,
            )?)
            .cast()
        } else {
            let tolerance = dimension.sel_tolerance_for_text_2d();
            let radius = if tolerance != 0.0 { tolerance } else { 1.0 };
            Handle::new(Select3DSensitiveCircle::new(
                &owner,
                &Circle3::new(axes, radius)?,
                true,
            ))
            .cast()
        };
        target.add(&text)?;
    }
    if mode == PrsDimDimensionSelectionMode::All {
        dimension.compute_flyout_selection(selection, &owner)?;
    }
    Ok(())
}

fn triangle(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    a: Point3,
    b: Point3,
    c: Point3,
) -> Handle<dyn Select3DSensitiveEntity> {
    Handle::new(Select3DSensitiveTriangle::new(
        owner,
        a,
        b,
        c,
        Select3DTypeOfSensitivity::Interior,
    ))
    .cast()
}
