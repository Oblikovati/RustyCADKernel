//! Sensitive-set traversal defers BVH construction until partial-volume traversal needs it.
use super::Select3DBndBox3d;
use super::sensitive_set::{Select3DSensitiveSet, SetPrimitives};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::constants::MAX_TREE_DEPTH;
use rustycad_tkmath::bvh::{BvhPrimitiveSetAccess, BvhTree, BvhVec3d};
/// Runs `body` on the built tree and the box of the content, holding the content lock; the
/// tree is taken out of the primitive set for the traversal and put back afterwards.
fn with_built_tree<S: Select3DSensitiveSet + ?Sized, R>(
    set: &S,
    body: impl FnOnce(&BvhTree<BvhVec3d>, &Select3DBndBox3d) -> Result<R, Failure>,
) -> Result<R, Failure> {
    let mut content = set.set_base().lock_content();
    let (tree, bounds) = {
        let mut primitives = SetPrimitives {
            set,
            content: &mut content,
        };
        primitives.update()?;
        let bounds = primitives.cached_box()?;
        (primitives.take_primitive_bvh()?, bounds)
    };
    let result = body(&tree, &bounds);
    SetPrimitives {
        set,
        content: &mut content,
    }
    .restore_primitive_bvh(tree);
    result
}

/// The arguments of `processElements`.
struct ElementRange {
    first: i32,
    last: i32,
    is_full_inside: bool,
}

/// `processElements` (protected): the inclusion or overlap test of the elements of a range,
/// keeping the nearest overlap; false when an inclusion fails. Counts the matches.
fn process_elements<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    range: &ElementRange,
    to_check_all_inside: bool,
    pick_result: &mut SelectBasicsPickResult,
    matches_nb: &mut i32,
) -> Result<bool, Failure> {
    let mut pick = SelectBasicsPickResult::default();
    for idx in range.first..=range.last {
        if !manager.is_overlap_allowed() {
            if !set.element_is_inside(manager, idx, range.is_full_inside)? {
                if to_check_all_inside {
                    continue;
                }
                return Ok(false);
            }
        } else {
            if !set.overlaps_element(&mut pick, manager, idx, range.is_full_inside)? {
                continue;
            }
            if pick_result.depth() > pick.depth() {
                *pick_result = pick;
                set.set_base().set_detected_idx(idx);
            }
        }
        *matches_nb += 1;
    }
    Ok(true)
}

/// A node of the traversal stack (`NodeInStack`).
#[derive(Clone, Copy, Default)]
struct NodeInStack {
    id: i32,
    is_full_inside: bool,
}

/// `Select3D_SensitiveSet::matches` (protected): the traversal of the tree with the overlap
/// or inclusion test; the distance to the center of geometry is set on a match.
/// Example: `select3d_set_matches(&*set, &manager, &mut pick, false)?`.
pub fn select3d_set_matches<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    pick_result: &mut SelectBasicsPickResult,
    to_check_all_inside: bool,
) -> Result<bool, Failure> {
    set.set_base().set_detected_idx(-1);
    if set.size() < 1 {
        return Ok(false);
    }
    match_cached_volume(set, manager, pick_result, to_check_all_inside)
}

fn cached_sensitive_bounds<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
) -> Result<Select3DBndBox3d, Failure> {
    let mut content = set.set_base().lock_content();
    SetPrimitives {
        set,
        content: &mut content,
    }
    .cached_box()
}
fn match_cached_volume<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
    check_all: bool,
) -> Result<bool, Failure> {
    let Some(full_inside) = cached_volume_inside(set, manager)? else {
        return Ok(false);
    };
    let mut matches = -1;
    if !match_sensitive_elements(set, manager, pick, check_all, full_inside, &mut matches)? {
        return Ok(false);
    }
    finish_sensitive_match(set, manager, pick, check_all, matches)
}
fn cached_volume_inside<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
) -> Result<Option<bool>, Failure> {
    let bounds = cached_sensitive_bounds(set)?;
    let mut inside = true;
    // OCCT 3d097a0 rejects global misses and handles full inclusion before constructing BVH.
    if !manager.overlaps_box_inside(bounds.corner_min(), bounds.corner_max(), Some(&mut inside))? {
        return Ok(None);
    }
    Ok(Some(
        inside && manager.get_active_selection_type() != SelectMgrSelectionType::Point,
    ))
}
fn finish_sensitive_match<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
    check_all: bool,
    matches: i32,
) -> Result<bool, Failure> {
    if matches != -1 {
        pick.set_dist_to_geom_center(set.distance_to_cog(manager)?);
    }
    Ok(matches != -1 || (!check_all && !manager.is_overlap_allowed()))
}

fn match_sensitive_elements<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
    check_all: bool,
    full_inside: bool,
    matches: &mut i32,
) -> Result<bool, Failure> {
    if full_inside {
        let range = ElementRange {
            first: 0,
            last: set.size() - 1,
            is_full_inside: true,
        };
        return process_elements(set, manager, &range, check_all, pick, matches);
    }
    match_partial_set(set, manager, pick, check_all, matches)
}
fn match_partial_set<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
    check_all: bool,
    matches: &mut i32,
) -> Result<bool, Failure> {
    let range_selection = manager.get_active_selection_type() != SelectMgrSelectionType::Point;
    with_built_tree(set, |tree, _| {
        traverse(
            set,
            manager,
            tree,
            range_selection,
            check_all,
            pick,
            matches,
        )
    })
}

/// The left and right children of an inner node with whether to visit them.
type ChildVisits = ((NodeInStack, bool), (NodeInStack, bool));

/// The children of an inner node and whether to visit them; None stops the traversal (an
/// inclusion failure).
fn visit_children(
    manager: &dyn SelectBasicsSelectingVolumeManager,
    tree: &BvhTree<BvhVec3d>,
    node: NodeInStack,
    data: (i32, i32),
    full_inside_check: bool,
    to_check_all_inside: bool,
) -> Result<Option<ChildVisits>, Failure> {
    let mut left = NodeInStack {
        id: data.0,
        is_full_inside: full_inside_check,
    };
    let mut right = NodeInStack {
        id: data.1,
        is_full_inside: full_inside_check,
    };
    let (mut to_left, mut to_right) = (true, true);
    if !node.is_full_inside {
        to_left = overlaps_node(manager, tree, &mut left, full_inside_check)?;
        to_right = overlaps_node(manager, tree, &mut right, full_inside_check)?;
    }
    if !manager.is_overlap_allowed() && !to_check_all_inside {
        if !to_left || !to_right {
            return Ok(None);
        }
        to_left &= !left.is_full_inside;
        to_right &= !right.is_full_inside;
    }
    Ok(Some(((left, to_left), (right, to_right))))
}

/// The overlap test of a child node box, clearing its full-inside flag when it misses.
fn overlaps_node(
    manager: &dyn SelectBasicsSelectingVolumeManager,
    tree: &BvhTree<BvhVec3d>,
    node: &mut NodeInStack,
    full_inside_check: bool,
) -> Result<bool, Failure> {
    let (min, max) = (tree.min_point(node.id)?, tree.max_point(node.id)?);
    let overlaps = manager.overlaps_box_inside(
        min,
        max,
        full_inside_check.then_some(&mut node.is_full_inside),
    )?;
    if !overlaps {
        node.is_full_inside = false;
    }
    Ok(overlaps)
}

/// The depth-first traversal of `matches` with the native fixed stack order.
fn traverse<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    tree: &BvhTree<BvhVec3d>,
    full_inside_check: bool,
    to_check_all_inside: bool,
    pick_result: &mut SelectBasicsPickResult,
    matches_nb: &mut i32,
) -> Result<bool, Failure> {
    let mut stack: Vec<NodeInStack> = Vec::with_capacity(MAX_TREE_DEPTH as usize);
    let mut node = NodeInStack::default();
    loop {
        let info = tree.node_info_buffer()[node.id as usize];
        if info.x() == 0 {
            let Some(((left, to_left), (right, to_right))) = visit_children(
                manager,
                tree,
                node,
                (info.y(), info.z()),
                full_inside_check,
                to_check_all_inside,
            )?
            else {
                return Ok(false);
            };
            if to_left || to_right {
                node = if to_left { left } else { right };
                if to_left && to_right {
                    stack.push(right);
                }
                continue;
            }
        } else {
            let range = ElementRange {
                first: info.y(),
                last: info.z(),
                is_full_inside: node.is_full_inside,
            };
            if !process_elements(
                set,
                manager,
                &range,
                to_check_all_inside,
                pick_result,
                matches_nb,
            )? {
                return Ok(false);
            }
        }
        let Some(next) = stack.pop() else {
            return Ok(true);
        };
        node = next;
    }
}
