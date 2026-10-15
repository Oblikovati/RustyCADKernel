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
    let mut primitives = SetPrimitives {
        set,
        content: &mut content,
    };
    primitives.update()?;
    let bounds = primitives.cached_box()?;
    let tree = primitives.take_primitive_bvh()?;
    let result = body(&tree, &bounds);
    primitives.restore_primitive_bvh(tree);
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
    let mut state = ElementMatchState::new(pick_result, matches_nb);
    for idx in range.first..=range.last {
        if !process_range_element(set, manager, range, idx, to_check_all_inside, &mut state)? {
            return Ok(false);
        }
    }
    Ok(true)
}

struct ElementMatchState<'a> {
    scratch: SelectBasicsPickResult,
    nearest: &'a mut SelectBasicsPickResult,
    matches: &'a mut i32,
}
impl<'a> ElementMatchState<'a> {
    fn new(nearest: &'a mut SelectBasicsPickResult, matches: &'a mut i32) -> Self {
        Self {
            scratch: SelectBasicsPickResult::default(),
            nearest,
            matches,
        }
    }
    fn retain_nearest<S: Select3DSensitiveSet + ?Sized>(&mut self, set: &S, idx: i32) {
        // Keep the native strict comparison: unordered NaN depths must not replace a pick.
        if self.nearest.depth() > self.scratch.depth() {
            *self.nearest = self.scratch;
            set.set_base().set_detected_idx(idx);
        }
    }
}
fn process_range_element<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    range: &ElementRange,
    idx: i32,
    check_all: bool,
    state: &mut ElementMatchState<'_>,
) -> Result<bool, Failure> {
    // Callbacks may change volume policy; native OCCT queries it for every element.
    let overlap = manager.is_overlap_allowed();
    if !match_range_element(set, manager, range, idx, overlap, state)? {
        return Ok(overlap || check_all);
    }
    *state.matches += 1;
    Ok(true)
}
fn match_range_element<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    range: &ElementRange,
    idx: i32,
    overlap: bool,
    state: &mut ElementMatchState<'_>,
) -> Result<bool, Failure> {
    if !overlap {
        return set.element_is_inside(manager, idx, range.is_full_inside);
    }
    if !set.overlaps_element(&mut state.scratch, manager, idx, range.is_full_inside)? {
        return Ok(false);
    }
    state.retain_nearest(set, idx);
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
    let mut left = child_stack_node(data.0, full_inside_check);
    let mut right = child_stack_node(data.1, full_inside_check);
    let (mut to_left, mut to_right) = (true, true);
    if !node.is_full_inside {
        to_left = overlaps_node(manager, tree, &mut left, full_inside_check)?;
        to_right = overlaps_node(manager, tree, &mut right, full_inside_check)?;
    }
    let visits = ((left, to_left), (right, to_right));
    filter_child_visits(manager, to_check_all_inside, visits)
}
fn child_stack_node(id: i32, is_full_inside: bool) -> NodeInStack {
    // Containment traversal inherits this flag before volume callbacks refine it.
    NodeInStack { id, is_full_inside }
}
fn filter_child_visits(
    manager: &dyn SelectBasicsSelectingVolumeManager,
    check_all: bool,
    mut visits: ChildVisits,
) -> Result<Option<ChildVisits>, Failure> {
    if !manager.is_overlap_allowed() && !check_all {
        if !visits.0.1 || !visits.1.1 {
            return Ok(None);
        }
        visits.0.1 &= !visits.0.0.is_full_inside;
        visits.1.1 &= !visits.1.0.is_full_inside;
    }
    Ok(Some(visits))
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
    let mut inputs = TraversalInputs {
        set,
        manager,
        tree,
        full_inside_check,
        to_check_all_inside,
        pick_result,
        matches_nb,
    };
    run_traversal(&mut inputs)
}
struct TraversalInputs<'a, S: Select3DSensitiveSet + ?Sized> {
    set: &'a S,
    manager: &'a dyn SelectBasicsSelectingVolumeManager,
    tree: &'a BvhTree<BvhVec3d>,
    full_inside_check: bool,
    to_check_all_inside: bool,
    pick_result: &'a mut SelectBasicsPickResult,
    matches_nb: &'a mut i32,
}
enum TraversalStep {
    Next(NodeInStack),
    Finished(bool),
}
fn run_traversal<S: Select3DSensitiveSet + ?Sized>(
    inputs: &mut TraversalInputs<'_, S>,
) -> Result<bool, Failure> {
    let mut stack = Vec::with_capacity(MAX_TREE_DEPTH as usize);
    let mut node = NodeInStack::default();
    loop {
        match visit_traversal_node(inputs, node, &mut stack)? {
            TraversalStep::Next(next) => node = next,
            TraversalStep::Finished(result) => return Ok(result),
        }
    }
}
fn visit_traversal_node<S: Select3DSensitiveSet + ?Sized>(
    inputs: &mut TraversalInputs<'_, S>,
    node: NodeInStack,
    stack: &mut Vec<NodeInStack>,
) -> Result<TraversalStep, Failure> {
    let info = inputs.tree.node_info_buffer()[node.id as usize];
    let indices = (info.y(), info.z());
    if info.x() == 0 {
        return visit_inner_node(inputs, node, indices, stack);
    }
    if !visit_leaf_node(inputs, node, indices)? {
        return Ok(TraversalStep::Finished(false));
    }
    Ok(pop_traversal_node(stack))
}
fn visit_leaf_node<S: Select3DSensitiveSet + ?Sized>(
    inputs: &mut TraversalInputs<'_, S>,
    node: NodeInStack,
    indices: (i32, i32),
) -> Result<bool, Failure> {
    let range = ElementRange {
        first: indices.0,
        last: indices.1,
        is_full_inside: node.is_full_inside,
    };
    process_elements(
        inputs.set,
        inputs.manager,
        &range,
        inputs.to_check_all_inside,
        inputs.pick_result,
        inputs.matches_nb,
    )
}
fn traversal_child_visits<S: Select3DSensitiveSet + ?Sized>(
    inputs: &TraversalInputs<'_, S>,
    node: NodeInStack,
    indices: (i32, i32),
) -> Result<Option<ChildVisits>, Failure> {
    visit_children(
        inputs.manager,
        inputs.tree,
        node,
        indices,
        inputs.full_inside_check,
        inputs.to_check_all_inside,
    )
}
fn visit_inner_node<S: Select3DSensitiveSet + ?Sized>(
    inputs: &TraversalInputs<'_, S>,
    node: NodeInStack,
    indices: (i32, i32),
    stack: &mut Vec<NodeInStack>,
) -> Result<TraversalStep, Failure> {
    let Some(visits) = traversal_child_visits(inputs, node, indices)? else {
        return Ok(TraversalStep::Finished(false));
    };
    if let Some(next) = descend_child_node(visits, stack) {
        return Ok(TraversalStep::Next(next));
    }
    Ok(pop_traversal_node(stack))
}
fn descend_child_node(visits: ChildVisits, stack: &mut Vec<NodeInStack>) -> Option<NodeInStack> {
    let ((left, to_left), (right, to_right)) = visits;
    if to_left && to_right {
        stack.push(right);
    }
    if to_left {
        return Some(left);
    }
    to_right.then_some(right)
}
fn pop_traversal_node(stack: &mut Vec<NodeInStack>) -> TraversalStep {
    stack
        .pop()
        .map_or(TraversalStep::Finished(true), TraversalStep::Next)
}

#[cfg(test)]
#[path = "sensitive_set_traversal_tests.rs"]
mod tests;

/// `Select3D_SensitiveSet::processElements` (protected): the inclusion or overlap test of the
/// elements `first..=last`, keeping the nearest overlap in the pick result; false when an
/// inclusion fails. Counts the matches in `matches_nb`.
/// Example: `select3d_set_process_elements(&*set, &manager, 0, 3, false, false, &mut pick, &mut count)?`.
#[allow(clippy::too_many_arguments)]
pub fn select3d_set_process_elements<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    first_elem: i32,
    last_elem: i32,
    is_full_inside: bool,
    to_check_all_inside: bool,
    pick_result: &mut SelectBasicsPickResult,
    matches_nb: &mut i32,
) -> Result<bool, Failure> {
    let range = ElementRange {
        first: first_elem,
        last: last_elem,
        is_full_inside,
    };
    process_elements(
        set,
        manager,
        &range,
        to_check_all_inside,
        pick_result,
        matches_nb,
    )
}

/// `Select3D_SensitiveSet::Clear`: does nothing natively (the owner is kept).
/// Example: `select3d_set_clear(&*set)`.
pub fn select3d_set_clear<S: Select3DSensitiveSet + ?Sized>(_set: &S) {}
