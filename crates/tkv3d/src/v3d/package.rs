//! The package class `V3d`: projection axes, the arrow and circle helpers of the grid and
//! light presentations, and the orientation names.
use super::{V3dTypeOfOrientation, V3dView};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, dx, dy, dz};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPolylines, Graphic3dGroup, Graphic3dGroupMembers as _,
};
use std::f64::consts::PI;

/// The package class `V3d`.
pub struct V3d;

/// `V3d_Table_PrintTypeOfOrientation`. Native quirk: the table does not follow the enumeration
/// order from the seventh entry on (and holds "XPOSZPOS" twice), so names and values of
/// `TypeOfOrientationToString` and `TypeOfOrientationFromString` do not round-trip.
const PRINT_TYPE_OF_ORIENTATION: [&str; 26] = [
    "XPOS",
    "YPOS",
    "ZPOS",
    "XNEG",
    "YNEG",
    "ZNEG",
    "XPOSYPOS",
    "XPOSZPOS",
    "XPOSZPOS",
    "XNEGYNEG",
    "XNEGYPOS",
    "XNEGZNEG",
    "XNEGZPOS",
    "YNEGZNEG",
    "YNEGZPOS",
    "XPOSYNEG",
    "XPOSZNEG",
    "YPOSZNEG",
    "XPOSYPOSZPOS",
    "XPOSYNEGZPOS",
    "XPOSYPOSZNEG",
    "XNEGYPOSZPOS",
    "XPOSYNEGZNEG",
    "XNEGYPOSZNEG",
    "XNEGYNEGZPOS",
    "XNEGYNEGZNEG",
];

/// A normalized direction of non-null constant coordinates (the native `gp_Dir(x, y, z)`).
fn unit(x: f64, y: f64, z: f64) -> Direction3 {
    // Every caller passes a non-null vector, so the normalization cannot fail.
    Direction3::new(x, y, z).unwrap_or_else(|_| dz())
}

/// The table of `GetProjAxis`, by orientation.
fn proj_axis_coords(orientation: V3dTypeOfOrientation) -> [f64; 3] {
    use V3dTypeOfOrientation as O;
    match orientation {
        O::Xpos => [1.0, 0.0, 0.0],
        O::Ypos => [0.0, 1.0, 0.0],
        O::Zpos => [0.0, 0.0, 1.0],
        O::Xneg => [-1.0, 0.0, 0.0],
        O::Yneg => [0.0, -1.0, 0.0],
        O::Zneg => [0.0, 0.0, -1.0],
        O::XposYposZpos => [1.0, 1.0, 1.0],
        O::XposYposZneg => [1.0, 1.0, -1.0],
        O::XposYnegZpos => [1.0, -1.0, 1.0],
        O::XposYnegZneg => [1.0, -1.0, -1.0],
        O::XnegYposZpos => [-1.0, 1.0, 1.0],
        O::XnegYposZneg => [-1.0, 1.0, -1.0],
        O::XnegYnegZpos => [-1.0, -1.0, 1.0],
        O::XnegYnegZneg => [-1.0, -1.0, -1.0],
        O::XposYpos => [1.0, 1.0, 0.0],
        O::XposYneg => [1.0, -1.0, 0.0],
        O::XnegYpos => [-1.0, 1.0, 0.0],
        O::XnegYneg => [-1.0, -1.0, 0.0],
        O::XposZpos => [1.0, 0.0, 1.0],
        O::XposZneg => [1.0, 0.0, -1.0],
        O::XnegZpos => [-1.0, 0.0, 1.0],
        O::XnegZneg => [-1.0, 0.0, -1.0],
        O::YposZpos => [0.0, 1.0, 1.0],
        O::YposZneg => [0.0, 1.0, -1.0],
        O::YnegZpos => [0.0, -1.0, 1.0],
        O::YnegZneg => [0.0, -1.0, -1.0],
    }
}

/// The unit vectors `(i, j)` orthogonal to `(dx, dy, dz)`, built from the least aligned axis
/// with the native operation order.
fn orthogonal_pair(d: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let [vx, vy, vz] = d;
    let mut n = [0.0; 3];
    if vx.abs() <= vy.abs() && vx.abs() <= vz.abs() {
        n[0] = 1.0;
    } else if vy.abs() <= vz.abs() && vy.abs() <= vx.abs() {
        n[1] = 1.0;
    } else {
        n[2] = 1.0;
    }
    let mut i = [
        vy * n[2] - vz * n[1],
        vz * n[0] - vx * n[2],
        vx * n[1] - vy * n[0],
    ];
    let norme = (i[0] * i[0] + i[1] * i[1] + i[2] * i[2]).sqrt();
    i = [i[0] / norme, i[1] / norme, i[2] / norme];
    let j = [
        vy * i[2] - vz * i[1],
        vz * i[0] - vx * i[2],
        vx * i[1] - vy * i[0],
    ];
    (i, j)
}

impl V3d {
    /// `GetProjAxis`: the projection direction of an orientation.
    /// Example: `V3d::get_proj_axis(V3dTypeOfOrientation::Xneg)`.
    pub fn get_proj_axis(orientation: V3dTypeOfOrientation) -> Direction3 {
        match orientation {
            V3dTypeOfOrientation::Xpos => dx(),
            V3dTypeOfOrientation::Ypos => dy(),
            V3dTypeOfOrientation::Zpos => dz(),
            V3dTypeOfOrientation::Xneg => dx().reversed(),
            V3dTypeOfOrientation::Yneg => dy().reversed(),
            V3dTypeOfOrientation::Zneg => dz().reversed(),
            _ => {
                let [x, y, z] = proj_axis_coords(orientation);
                unit(x, y, z)
            }
        }
    }
    /// `ArrowOfRadius`: a ten-sided arrow cone of polylines from the tip `(x0, y0, z0)` back
    /// along `(dx, dy, dz)`, added to a group.
    /// Example: `V3d::arrow_of_radius(&group, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.3, 1.0)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn arrow_of_radius(
        group: &Handle<dyn Graphic3dGroup>,
        x0: f64,
        y0: f64,
        z0: f64,
        d_x: f64,
        d_y: f64,
        d_z: f64,
        alpha: f64,
        lng: f64,
    ) -> Result<(), Failure> {
        const NB_POINTS: i32 = 10;
        let center = [x0 - d_x * lng, y0 - d_y * lng, z0 - d_z * lng];
        let (i, j) = orthogonal_pair([d_x, d_y, d_z]);
        let prims = Graphic3dArrayOfPolylines::new(3 * NB_POINTS, NB_POINTS, 0, 0)?;
        let tg = alpha.tan();
        let mut first = [0.0; 3];
        for index in 1..=NB_POINTS {
            let angle = 2.0 * PI / f64::from(NB_POINTS) * f64::from(index - 1);
            let (cosinus, sinus) = (angle.cos(), angle.sin());
            let p: [f64; 3] =
                std::array::from_fn(|k| center[k] + (cosinus * i[k] + sinus * j[k]) * lng * tg);
            if index == 1 {
                first = p;
            } else {
                prims.add_vertex_f64(p[0], p[1], p[2])?;
            }
            prims.add_bound(3)?;
            prims.add_vertex_f64(x0, y0, z0)?;
            prims.add_vertex_f64(p[0], p[1], p[2])?;
        }
        prims.add_vertex_f64(first[0], first[1], first[2])?;
        if let Some(group) = group.get() {
            group.add_primitive_array(&Handle::new(prims).cast(), true);
        }
        Ok(())
    }
    /// `CircleInPlane`: a 30-sided circle of radius `rayon` in the plane of normal
    /// `(d_x, d_y, d_z)` through `(x0, y0, z0)`; nothing for a normal shorter than 1e-4.
    /// Example: `V3d::circle_in_plane(&group, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn circle_in_plane(
        group: &Handle<dyn Graphic3dGroup>,
        x0: f64,
        y0: f64,
        z0: f64,
        d_x: f64,
        d_y: f64,
        d_z: f64,
        rayon: f64,
    ) -> Result<(), Failure> {
        let norme = (d_x * d_x + d_y * d_y + d_z * d_z).sqrt();
        if norme < 0.0001 {
            return Ok(());
        }
        let (i, j) = orthogonal_pair([d_x / norme, d_y / norme, d_z / norme]);
        const NFACES: i32 = 30;
        let prims = Graphic3dArrayOfPolylines::new(NFACES + 1, 0, 0, 0)?;
        let origin = [x0, y0, z0];
        let dalpha = 2.0 * PI / f64::from(NFACES);
        let mut alpha: f64 = 0.0;
        for _ in 0..=NFACES {
            let (cosinus, sinus) = (alpha.cos(), alpha.sin());
            let p: [f64; 3] =
                std::array::from_fn(|k| origin[k] + (cosinus * i[k] + sinus * j[k]) * rayon);
            prims.add_vertex_f64(p[0], p[1], p[2])?;
            alpha += dalpha;
        }
        if let Some(group) = group.get() {
            group.add_primitive_array(&Handle::new(prims).cast(), true);
        }
        Ok(())
    }
    /// `SwitchViewsinWindow`: turns the previous view off and the next one on in the same
    /// window. Example: `V3d::switch_views_in_window(&previous, &next)?`.
    pub fn switch_views_in_window(
        previous_view: &Handle<V3dView>,
        next_view: &Handle<V3dView>,
    ) -> Result<(), Failure> {
        let (Some(previous), Some(next)) = (previous_view.get(), next_view.get()) else {
            return Ok(());
        };
        if let Some(viewer) = previous.viewer().get() {
            viewer.set_view_off_one(previous_view);
        }
        if !next.if_window() {
            next.set_window(&previous.window(), 0)?;
        }
        if let Some(viewer) = next.viewer().get() {
            viewer.set_view_on_one(next_view)?;
        }
        Ok(())
    }
    /// `TypeOfOrientationToString`; `V3d::type_of_orientation_to_string(V3dTypeOfOrientation::Xpos) == "XPOS"`.
    pub fn type_of_orientation_to_string(kind: V3dTypeOfOrientation) -> &'static str {
        PRINT_TYPE_OF_ORIENTATION[kind as usize]
    }
    /// `TypeOfOrientationFromString`: the orientation of an upper-cased name, or `None`.
    /// Example: `V3d::type_of_orientation_from_string_checked("ypos")`.
    pub fn type_of_orientation_from_string_checked(name: &str) -> Option<V3dTypeOfOrientation> {
        let upper = name.to_ascii_uppercase();
        PRINT_TYPE_OF_ORIENTATION
            .iter()
            .position(|entry| *entry == upper)
            .map(|index| V3dTypeOfOrientation::ALL[index])
    }
    /// `TypeOfOrientationFromString(name)`: the orientation of a name, `Xpos` when unknown.
    /// Example: `V3d::type_of_orientation_from_string("zneg")`.
    pub fn type_of_orientation_from_string(name: &str) -> V3dTypeOfOrientation {
        Self::type_of_orientation_from_string_checked(name).unwrap_or(V3dTypeOfOrientation::Xpos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proj_axes_are_unit_directions() {
        let axis = V3d::get_proj_axis(V3dTypeOfOrientation::XposYnegZpos);
        let third = 1.0 / 3.0f64.sqrt();
        assert!((axis.x() - third).abs() < 1e-15 && (axis.y() + third).abs() < 1e-15);
        assert_eq!(V3d::get_proj_axis(V3dTypeOfOrientation::Zneg).z(), -1.0);
    }

    #[test]
    fn orientation_names_keep_the_native_table() {
        assert_eq!(
            V3d::type_of_orientation_to_string(V3dTypeOfOrientation::YposZpos),
            "XPOSZPOS"
        );
        assert_eq!(
            V3d::type_of_orientation_from_string("xposzpos"),
            V3dTypeOfOrientation::XposZpos
        );
        assert_eq!(
            V3d::type_of_orientation_from_string("unknown"),
            V3dTypeOfOrientation::Xpos
        );
        assert_eq!(V3d::type_of_orientation_from_string_checked("bad"), None);
    }
}
