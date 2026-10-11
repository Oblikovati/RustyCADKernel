//! Martensite controller picking uses the context selector without detection filters.
use super::{AisInteractiveContext, AisViewController};
use crate::{select_basics::SelectBasicsSelectingVolumeManager, select_mgr::*, v3d::V3dView};
use rustycad_tkernel::{ncollection::Vec2, precision::is_infinite, standard::Failure};
use rustycad_tkmath::gp::{Axis1, Point3};

impl AisViewController {
    /// Pick a world point, optionally projecting onto the cursor ray; `controller.pick_point(&mut point, &context, &view, &pixel, true)?`.
    pub fn pick_point(
        &mut self,
        point: &mut Point3,
        context: &AisInteractiveContext,
        view: &V3dView,
        cursor: &Vec2<i32>,
        stick_to_pick_ray: bool,
    ) -> Result<bool, Failure> {
        self.reset_previous_move_to();
        let selector = context.main_selector().clone();
        selector.pick_point(cursor.x, cursor.y, view)?;
        if selector.nb_picked() < 1 {
            return Ok(false);
        }
        *point = controller_picked_position(selector.get().unwrap(), stick_to_pick_ray)?;
        Ok(controller_point_is_defined(point))
    }

    /// Pick along a world axis, retaining the output on a miss; `controller.pick_axis(&mut point, &context, &view, axis)?`.
    pub fn pick_axis(
        &mut self,
        point: &mut Point3,
        context: &AisInteractiveContext,
        view: &V3dView,
        axis: Axis1,
    ) -> Result<bool, Failure> {
        self.reset_previous_move_to();
        let selector = context.main_selector().clone();
        selector.pick_axis(axis, view)?;
        if selector.nb_picked() < 1 {
            return Ok(false);
        }
        *point = selector.picked_data(1)?.point;
        Ok(controller_point_is_defined(point))
    }
}

fn controller_picked_position(
    selector: &dyn SelectMgrViewerSelectorSource,
    stick_to_pick_ray: bool,
) -> Result<Point3, Failure> {
    let depth = selector.picked_data(1)?.depth;
    if stick_to_pick_ray && !is_infinite(depth) {
        return selector.get_manager().detected_point(depth);
    }
    selector.picked_point(1)
}

fn controller_point_is_defined(point: &Point3) -> bool {
    // OCCT 3d097a0 tests Precision::IsInfinite, which intentionally does not reject NaN.
    !is_infinite(point.x()) && !is_infinite(point.y()) && !is_infinite(point.z())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picking_uses_native_infinite_threshold_on_every_coordinate() {
        assert!(controller_point_is_defined(&Point3::new(f64::NAN, 0., 0.)));
        for axis in 0..3 {
            let mut coordinates = [0.; 3];
            coordinates[axis] = 1e100;
            let point = Point3::new(coordinates[0], coordinates[1], coordinates[2]);
            assert!(!controller_point_is_defined(&point));
        }
    }
}
