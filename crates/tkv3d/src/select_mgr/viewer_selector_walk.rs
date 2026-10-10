//! Left-first native BVH traversal, releasing tree guards before client callbacks.
use super::SelectMgrSelectingVolumeManager;
use crate::select_basics::SelectBasicsSelectingVolumeManager;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{BvhTreeHandle, BvhVec3d};
use rustycad_tkservice::graphic3d::Graphic3dBndBox3d;
pub(super) fn selector_node_bounds(
    tree: &BvhTreeHandle<BvhVec3d>,
    node: i32,
) -> Result<Graphic3dBndBox3d, Failure> {
    let tree = tree.read();
    Ok(Graphic3dBndBox3d::from_corners(
        *tree.min_point(node)?,
        *tree.max_point(node)?,
    ))
}
pub(super) fn selector_node_overlaps(
    tree: &BvhTreeHandle<BvhVec3d>,
    node: i32,
    volume: &SelectMgrSelectingVolumeManager,
) -> Result<bool, Failure> {
    let bounds = selector_node_bounds(tree, node)?;
    volume.overlaps_box_inside(bounds.corner_min(), bounds.corner_max(), None)
}
pub(super) fn next_selector_leaf(
    tree: &BvhTreeHandle<BvhVec3d>,
    stack: &mut Vec<i32>,
    mut intersects: impl FnMut(i32) -> Result<bool, Failure>,
) -> Result<Option<i32>, Failure> {
    while let Some(node) = stack.pop() {
        let current = tree.read();
        if current.is_outer(node)? {
            return Ok(Some(node));
        }
        let left = current.child::<0>(node)?;
        let right = current.child::<1>(node)?;
        drop(current);
        let left_inside = intersects(left)?;
        let right_inside = intersects(right)?;
        if right_inside {
            stack.push(right);
        }
        if left_inside {
            stack.push(left);
        }
    }
    Ok(None)
}
