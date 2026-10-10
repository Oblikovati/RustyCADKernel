//! `Select3D_SensitiveSet`: a sensitive entity made of elements kept in a BVH tree, which the
//! selecting volume traverses (v3d-design.md section 6). The abstract class is the
//! [`Select3DSensitiveSet`] trait over [`Select3DSensitiveSetBase`]; the native members it
//! implements are the `select3d_set_*` functions, which every set type calls from its
//! `Select3DSensitiveEntity` implementation.
use super::{Select3DBndBox3d, Select3DSensitiveEntity, select3d_sensitive_entity_dump_json};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, SharedState};
use rustycad_tkmath::bvh::constants::{LEAF_NODE_SIZE_SMALL, MAX_TREE_DEPTH};
use rustycad_tkmath::bvh::{
    BvhBox, BvhBuilder, BvhLinearBuilder, BvhPrimitiveSet, BvhPrimitiveSetAccess, BvhSet, BvhTree,
    BvhVec3d,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

/// `Select3D_BVHBuilder3d`: the builder of the sensitive set trees.
pub type Select3DBvhBuilder3d = Arc<dyn BvhBuilder<BvhVec3d>>;

/// `THE_SENS_SET_BUILDER`: a linear builder (8 primitives per leaf) unless replaced.
static DEFAULT_BUILDER: RwLock<Option<Select3DBvhBuilder3d>> = RwLock::new(None);

/// `Select3D_SensitiveSet::DefaultBVHBuilder`; `select3d_default_bvh_builder()`.
pub fn select3d_default_bvh_builder() -> Select3DBvhBuilder3d {
    let current = DEFAULT_BUILDER
        .read()
        .map(|b| b.clone())
        .unwrap_or_default();
    current.unwrap_or_else(|| {
        Arc::new(BvhLinearBuilder::<BvhVec3d>::new(
            LEAF_NODE_SIZE_SMALL,
            MAX_TREE_DEPTH,
        ))
    })
}

/// `Select3D_SensitiveSet::SetDefaultBVHBuilder`: the builder of the sets created afterwards.
/// Example: `select3d_set_default_bvh_builder(builder)`.
pub fn select3d_set_default_bvh_builder(builder: Select3DBvhBuilder3d) {
    if let Ok(mut current) = DEFAULT_BUILDER.write() {
        *current = Some(builder);
    }
}

/// The state every sensitive set embeds: the BVH content and the detected element.
pub struct Select3DSensitiveSetBase {
    content: Mutex<BvhPrimitiveSet<BvhVec3d>>,
    detected_idx: SharedState<i32>,
}

impl Default for Select3DSensitiveSetBase {
    /// The constructor state: the default builder, a dirty tree, no detected element.
    fn default() -> Self {
        // The constructor marks the content dirty (SetSensitiveSet and MarkDirty): the tree is
        // built on the first traversal.
        let mut content = BvhPrimitiveSet::new(Some(select3d_default_bvh_builder()));
        content.object_transient_mut().is_dirty = true;
        Self {
            content: Mutex::new(content),
            detected_idx: SharedState::new(-1),
        }
    }
}

impl Select3DSensitiveSetBase {
    /// The locked BVH content (a poisoned lock keeps its data: the content stays consistent
    /// between the builder calls).
    fn lock_content(&self) -> MutexGuard<'_, BvhPrimitiveSet<BvhVec3d>> {
        self.content
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// `myDetectedIdx`: the detected element in the BVH order, -1 without; `base.detected_idx()`.
    pub fn detected_idx(&self) -> i32 {
        *self.detected_idx.read()
    }
    /// Sets `myDetectedIdx` (a subclass match outside the traversal); `base.set_detected_idx(0)`.
    pub fn set_detected_idx(&self, idx: i32) {
        *self.detected_idx.write() = idx;
    }
    /// `MarkDirty`: the tree is built again before the next traversal; `base.mark_dirty()`.
    pub fn mark_dirty(&self) {
        self.lock_content().object_transient_mut().is_dirty = true;
    }
    /// `SetBuilder`: the builder of the next build; `base.set_builder(builder)`.
    pub fn set_builder(&self, builder: Select3DBvhBuilder3d) {
        let mut content = self.lock_content();
        ContentOnly {
            content: &mut content,
        }
        .set_builder(Some(builder));
    }
    /// `GetLeafNodeSize`: the leaf size of the builder; `base.leaf_node_size()`.
    pub fn leaf_node_size(&self) -> i32 {
        let mut content = self.lock_content();
        let adapter = ContentOnly {
            content: &mut content,
        };
        adapter
            .builder()
            .as_ref()
            .map_or(0, |b| b.builder_transient().leaf_node_size())
    }
    /// Whether the tree must be built (`ToBuildBVH`); `base.is_dirty()`.
    pub fn is_dirty(&self) -> bool {
        self.lock_content().object_transient().is_dirty
    }
}

/// The content of a set without its elements, to reach the builder of the primitive set.
struct ContentOnly<'a> {
    content: &'a mut BvhPrimitiveSet<BvhVec3d>,
}

impl BvhSet<BvhVec3d> for ContentOnly<'_> {
    fn size(&self) -> i32 {
        0
    }
    fn element_box(&self, _index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        Ok(BvhBox::default())
    }
    fn center(&self, _index: i32, _axis: i32) -> Result<f64, Failure> {
        Ok(0.0)
    }
    fn swap(&mut self, _first: i32, _second: i32) -> Result<(), Failure> {
        Ok(())
    }
}

impl BvhPrimitiveSetAccess<BvhVec3d> for ContentOnly<'_> {
    fn primitive_set(&self) -> &BvhPrimitiveSet<BvhVec3d> {
        self.content
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<BvhVec3d> {
        self.content
    }
}

/// A sensitive entity made of elements in a BVH tree.
pub trait Select3DSensitiveSet: Select3DSensitiveEntity {
    /// The embedded set state; `set.set_base()`.
    fn set_base(&self) -> &Select3DSensitiveSetBase;
    /// `Size` (pure virtual): the number of elements; `set.size()`.
    fn size(&self) -> i32;
    /// `Box` (pure virtual): the box of an element; `set.element_box(0)`.
    fn element_box(&self, idx: i32) -> Select3DBndBox3d;
    /// `Center` (pure virtual): the center of an element along an axis; an invalid
    /// coordinate index raises `Standard_OutOfRange`. Example: `set.center(0, 1)?`.
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure>;
    /// `Swap` (pure virtual): exchanges two elements; `set.swap(0, 1)`.
    fn swap(&self, idx1: i32, idx2: i32);
    /// `overlapsElement` (pure virtual): the overlap test of one element with depth.
    /// Example: `set.overlaps_element(&mut pick, &manager, 0, false)`.
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure>;
    /// `elementIsInside` (pure virtual): the inclusion test of one element.
    /// Example: `set.element_is_inside(&manager, 0, false)`.
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure>;
    /// `distanceToCOG` (pure virtual); `set.distance_to_cog(&manager)`.
    fn distance_to_cog(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure>;
}

rustycad_tkernel::impl_transient_interface!(
    dyn Select3DSensitiveSet,
    "Select3D_SensitiveSet",
    dyn Select3DSensitiveEntity
);

/// The sensitive set seen as the BVH primitive set of its content.
struct SetPrimitives<'a, S: Select3DSensitiveSet + ?Sized> {
    set: &'a S,
    content: &'a mut BvhPrimitiveSet<BvhVec3d>,
}

impl<S: Select3DSensitiveSet + ?Sized> BvhSet<BvhVec3d> for SetPrimitives<'_, S> {
    fn size(&self) -> i32 {
        self.set.size()
    }
    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        Ok(self.set.element_box(index))
    }
    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        self.set.center(index, axis)
    }
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.set.swap(first, second);
        Ok(())
    }
    fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.cached_box()
    }
}

impl<S: Select3DSensitiveSet + ?Sized> BvhPrimitiveSetAccess<BvhVec3d> for SetPrimitives<'_, S> {
    fn primitive_set(&self) -> &BvhPrimitiveSet<BvhVec3d> {
        self.content
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<BvhVec3d> {
        self.content
    }
}

/// `Select3D_SensitiveSet::BVH`: builds the tree when dirty; `select3d_set_build_bvh(&*set)?`.
pub fn select3d_set_build_bvh<S: Select3DSensitiveSet + ?Sized>(set: &S) -> Result<(), Failure> {
    let mut content = set.set_base().lock_content();
    let mut primitives = SetPrimitives {
        set,
        content: &mut content,
    };
    primitives.update()
}

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
                *set.set_base().detected_idx.write() = idx;
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
    *set.set_base().detected_idx.write() = -1;
    if set.size() < 1 {
        return Ok(false);
    }
    with_built_tree(set, |tree, global_box| {
        traverse_set(
            set,
            manager,
            tree,
            global_box,
            pick_result,
            to_check_all_inside,
        )
    })
}

/// The body of `matches` once the tree is built.
fn traverse_set<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    manager: &dyn SelectBasicsSelectingVolumeManager,
    tree: &BvhTree<BvhVec3d>,
    global_box: &Select3DBndBox3d,
    pick_result: &mut SelectBasicsPickResult,
    to_check_all_inside: bool,
) -> Result<bool, Failure> {
    let mut is_full_inside = true;
    if !manager.overlaps_box_inside(
        global_box.corner_min(),
        global_box.corner_max(),
        Some(&mut is_full_inside),
    )? {
        return Ok(false);
    }
    let mut matches_nb = -1;
    let to_check_full_inside = manager.get_active_selection_type() != SelectMgrSelectionType::Point;
    if to_check_full_inside && is_full_inside {
        let range = ElementRange {
            first: 0,
            last: set.size() - 1,
            is_full_inside: true,
        };
        if !process_elements(
            set,
            manager,
            &range,
            to_check_all_inside,
            pick_result,
            &mut matches_nb,
        )? {
            return Ok(false);
        }
    } else if !traverse(
        set,
        manager,
        tree,
        to_check_full_inside,
        to_check_all_inside,
        pick_result,
        &mut matches_nb,
    )? {
        return Ok(false);
    }
    if matches_nb != -1 {
        pick_result.set_dist_to_geom_center(set.distance_to_cog(manager)?);
    }
    Ok(matches_nb != -1 || (!to_check_all_inside && !manager.is_overlap_allowed()))
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

/// `Select3D_SensitiveSet::BoundingBox`: an invalid box (RealLast to RealFirst).
/// Example: `select3d_set_bounding_box()`.
pub fn select3d_set_bounding_box() -> Select3DBndBox3d {
    BvhBox::from_corners(
        Vec3::new(real_last(), real_last(), real_last()),
        Vec3::new(real_first(), real_first(), real_first()),
    )
}

/// `Select3D_SensitiveSet::CenterOfGeometry`: the point (RealLast, RealLast, RealLast).
/// Example: `select3d_set_center_of_geometry()`.
pub fn select3d_set_center_of_geometry() -> Point3 {
    Point3::new(real_last(), real_last(), real_last())
}

/// The native `Select3D_SensitiveSet::DumpJson` body (the content dumps nothing natively).
/// Example: `select3d_set_dump_json(&*set, &mut stream, -1)?`.
pub fn select3d_set_dump_json<S: Select3DSensitiveSet + ?Sized>(
    set: &S,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "Select3D_SensitiveSet")?;
    base_class(stream, depth, &|s, d| {
        select3d_sensitive_entity_dump_json(set, s, d)
    })?;
    field_dumped(stream, depth, "&myContent", &|_, _| Ok(()))?;
    field_integer(stream, "myDetectedIdx", set.set_base().detected_idx())?;
    let bounds = set.bounding_box();
    field_dumped(stream, depth, "&aBoundingBox", &|s, _| bounds.dump_json(s))
}
