//! Native orientation classification counts nonzero projection-axis components.
use crate::v3d::{V3d, V3dTypeOfOrientation};
fn cube_direction_components(orientation: V3dTypeOfOrientation) -> usize {
    let direction = V3d::get_proj_axis(orientation);
    direction
        .xyz()
        .coordinates()
        .iter()
        .filter(|value| **value != 0.0)
        .count()
}
/// True for the six box faces; `ais_view_cube_is_box_side(V3dTypeOfOrientation::Xpos)`.
pub fn ais_view_cube_is_box_side(orientation: V3dTypeOfOrientation) -> bool {
    cube_direction_components(orientation) == 1
}
/// True for the twelve box edges; `ais_view_cube_is_box_edge(V3dTypeOfOrientation::XposYpos)`.
pub fn ais_view_cube_is_box_edge(orientation: V3dTypeOfOrientation) -> bool {
    cube_direction_components(orientation) == 2
}
/// True for the eight box corners; `ais_view_cube_is_box_corner(V3dTypeOfOrientation::XposYposZpos)`.
pub fn ais_view_cube_is_box_corner(orientation: V3dTypeOfOrientation) -> bool {
    cube_direction_components(orientation) == 3
}
