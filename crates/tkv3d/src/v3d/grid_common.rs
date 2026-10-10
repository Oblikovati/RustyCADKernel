//! The code that `V3d_RectangularGrid` and `V3d_CircularGrid` repeat natively: the placement
//! of the grid structure in the privileged plane and the end of a group rebuild.
use super::V3dViewer;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis3, Transform3};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::graphic3d::{
    Graphic3dGroup, Graphic3dGroupMembers as _, Graphic3dStructureMembers as _,
    Graphic3dStructureSource,
};

/// The placement a grid structure was last transformed with (`myCurViewPlane`, `myCurXo`,
/// `myCurYo`, `myCurAngle`).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct V3dGridPlacement {
    pub(crate) view_plane: Axis3,
    pub(crate) x_origin: f64,
    pub(crate) y_origin: f64,
    pub(crate) angle: f64,
}

/// The twelve coordinates of a frame, compared field by field natively.
fn frame_values(plane: &Axis3) -> [f64; 12] {
    let (l, x, y, d) = (
        plane.location(),
        plane.x_direction(),
        plane.y_direction(),
        plane.direction(),
    );
    [
        l.x(),
        l.y(),
        l.z(),
        x.x(),
        x.y(),
        x.z(),
        y.x(),
        y.y(),
        y.z(),
        d.x(),
        d.y(),
        d.z(),
    ]
}

impl V3dGridPlacement {
    /// Whether the structure needs a new transformation (`MakeTransform`).
    pub(crate) fn needs_transform(&self, defined: bool, plane: &Axis3, wanted: &Self) -> bool {
        if !defined {
            return true;
        }
        if wanted.angle != self.angle
            || wanted.x_origin != self.x_origin
            || wanted.y_origin != self.y_origin
        {
            return true;
        }
        frame_values(plane) != frame_values(&self.view_plane)
    }
    /// The privileged plane frame composed with the grid rotation and origin.
    pub(crate) fn transformation(&self) -> Result<Transform3, Failure> {
        let [xl, yl, zl, xdx, xdy, xdz, ydx, ydy, ydz, dx, dy, dz] = frame_values(&self.view_plane);
        let (cos_alpha, sin_alpha) = (self.angle.cos(), self.angle.sin());
        let mut trsf = Transform3::default();
        trsf.set_values([[xdx, ydx, dx, xl], [xdy, ydy, dy, yl], [xdz, ydz, dz, zl]])?;
        let mut trsf2 = Transform3::default();
        trsf2.set_values([
            [cos_alpha, sin_alpha, 0.0, -self.x_origin],
            [-sin_alpha, cos_alpha, 0.0, -self.y_origin],
            [0.0, 0.0, 1.0, 0.0],
        ])?;
        trsf.multiply(trsf2);
        Ok(trsf)
    }
    /// Applies the transformation to the grid structure.
    pub(crate) fn apply(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let trsf = self.transformation()?;
        if let Some(value) = structure.get() {
            value.set_transformation(&Handle::new(Datum3::new(trsf)));
        }
        Ok(())
    }
}

/// The end of a group rebuild: the group bounds, the structure box and a manager update.
pub(crate) fn finish_grid_group(
    group: &Handle<dyn Graphic3dGroup>,
    structure: &Handle<dyn Graphic3dStructureSource>,
    viewer: &Handle<V3dViewer>,
    min_max: [f64; 6],
) {
    if let Some(group) = group.get() {
        group.set_min_max_values(min_max);
    }
    let Some(value) = structure.get() else {
        return;
    };
    value.calculate_bound_box();
    if let Some(manager) = viewer.get().map(V3dViewer::structure_manager)
        && let Some(manager) = manager.get()
    {
        manager.update(value.get_z_layer());
    }
}

/// Whether the grid geometry must be rebuilt: `toUpdate`, `myToComputePrs` and the display
/// state; marks a pending rebuild when the structure is not displayed. Returns true to rebuild.
pub(crate) fn should_rebuild(
    to_update: bool,
    to_compute_prs: &mut bool,
    structure: &Handle<dyn Graphic3dStructureSource>,
) -> bool {
    if !to_update && !*to_compute_prs {
        return false;
    }
    if !structure.get().is_some_and(|s| s.is_displayed()) {
        *to_compute_prs = true;
        return false;
    }
    *to_compute_prs = false;
    true
}
