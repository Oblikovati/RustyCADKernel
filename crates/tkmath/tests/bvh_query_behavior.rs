// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Supplemental BVH phase 2 behavior beyond the legacy suites: native quirks,
//! errors that replace upstream crashes, and the set/tree lending of traversals.
//! Numeric results of the same code paths are compared with OCCT by the
//! `bvh_builders`, `bvh_traverse`, `bvh_nearest` and `bvh_field` probes.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{
    BvhBaseTraverse, BvhBinnedBuilder, BvhBox, BvhBoxSet, BvhBuilder, BvhDistance,
    BvhDistanceField, BvhDistanceTraverse, BvhGeometry, BvhIndexedBoxSet, BvhLinearBuilder,
    BvhObject, BvhObjectSet, BvhPairTraverse, BvhPrimitiveSetAccess, BvhSet, BvhSetTraverse,
    BvhSpatialMedianBuilder, BvhSweepPlaneBuilder, BvhTraverse, BvhTree, BvhTriangulation,
    BvhUpdateBoundTask, BvhVec3d, BvhVec4i, array, direction_to_nearest_point,
    square_distance_to_object, update_bounds,
};
use std::sync::Arc;

fn unit_box(x: f64) -> BvhBox<BvhVec3d> {
    BvhBox::from_corners(BvhVec3d::new(x, 0.0, 0.0), BvhVec3d::new(x + 1.0, 1.0, 1.0))
}

fn box_set(builder: Arc<dyn BvhBuilder<BvhVec3d>>, xs: &[f64]) -> BvhBoxSet<BvhVec3d> {
    let mut set = BvhBoxSet::new(Some(builder));
    for (index, x) in xs.iter().enumerate() {
        set.add(index as i32, unit_box(*x));
    }
    set
}

fn mesh(xs: &[f64]) -> BvhTriangulation<BvhVec3d> {
    let mut set = BvhTriangulation::default();
    for (index, x) in xs.iter().enumerate() {
        let first = 3 * index as i32;
        array::append(&mut set.vertices, BvhVec3d::new(*x, 0.0, 0.0));
        array::append(&mut set.vertices, BvhVec3d::new(x + 1.0, 0.0, 0.0));
        array::append(&mut set.vertices, BvhVec3d::new(*x, 1.0, 0.0));
        array::append(
            &mut set.elements,
            BvhVec4i::new(first, first + 1, first + 2, 0),
        );
    }
    set
}

fn height(tree: &BvhTree<BvhVec3d>, node: i32) -> i32 {
    if tree.is_outer(node).unwrap() {
        return 0;
    }
    let left = height(tree, tree.child::<0>(node).unwrap());
    let right = height(tree, tree.child::<1>(node).unwrap());
    left.max(right) + 1
}

#[test]
fn linear_builder_keeps_levels_zero_and_reports_the_height_as_depth() {
    let builder = BvhLinearBuilder::<BvhVec3d>::new(1, 2);
    let mut set = box_set(
        Arc::new(BvhLinearBuilder::default()),
        &[0.0, 2.0, 4.0, 6.0, 8.0],
    );
    let mut tree = BvhTree::default();
    let bounding = set.combined_boxes();
    builder.build(&mut set, &mut tree, &bounding).unwrap();
    // The depth limit (2) is not applied and no node gets a level.
    assert_eq!(tree.depth(), height(&tree, 0));
    assert!(tree.depth() > 2);
    assert!((0..tree.length()).all(|node| tree.level(node).unwrap() == 0));
}

trait CombinedBoxes {
    fn combined_boxes(&self) -> BvhBox<BvhVec3d>;
}
impl<S: BvhSet<BvhVec3d>> CombinedBoxes for S {
    fn combined_boxes(&self) -> BvhBox<BvhVec3d> {
        rustycad_tkmath::bvh::combined_element_boxes(self).unwrap()
    }
}

#[test]
fn linear_builder_leaves_the_tree_untouched_for_an_empty_set() {
    let builder = BvhLinearBuilder::<BvhVec3d>::new(1, 32);
    let mut set = box_set(Arc::new(BvhLinearBuilder::default()), &[0.0, 3.0, 6.0]);
    let mut tree = BvhTree::default();
    let bounding = set.combined_boxes();
    builder.build(&mut set, &mut tree, &bounding).unwrap();
    let length = tree.length();
    let mut empty = BvhBoxSet::<BvhVec3d>::default();
    builder
        .build(&mut empty, &mut tree, &BvhBox::default())
        .unwrap();
    assert_eq!(tree.length(), length);
}

#[test]
fn linear_builder_reports_the_endless_middle_split_of_leaf_size_zero() {
    // Upstream recurses without end (stack overflow) on this input.
    let builder = BvhLinearBuilder::<BvhVec3d>::new(0, 32);
    let mut set = box_set(Arc::new(BvhLinearBuilder::default()), &[0.0, 0.0]);
    let mut tree = BvhTree::default();
    let bounding = set.combined_boxes();
    let failure = builder.build(&mut set, &mut tree, &bounding).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_StackOverflow");
    assert!(
        failure.message().contains("[0, 1)"),
        "{}",
        failure.message()
    );
}

#[test]
fn update_bound_task_matches_the_recursive_update() {
    let builder = BvhLinearBuilder::<BvhVec3d>::new(1, 32);
    let xs: Vec<f64> = (0..23).map(|index| f64::from(index * 7 % 23)).collect();
    let mut set = box_set(Arc::new(BvhLinearBuilder::default()), &xs);
    let mut tree = BvhTree::default();
    let bounding = set.combined_boxes();
    builder.build(&mut set, &mut tree, &bounding).unwrap();
    let mut recursive = tree.clone();
    let task_height = BvhUpdateBoundTask::new(true)
        .run(&set, &mut tree, 0, 0)
        .unwrap();
    assert_eq!(update_bounds(&set, &mut recursive, 0).unwrap(), task_height);
    assert_eq!(
        recursive.min_point_buffer().as_slice(),
        tree.min_point_buffer().as_slice()
    );
    assert_eq!(
        recursive.max_point_buffer().as_slice(),
        tree.max_point_buffer().as_slice()
    );
}

#[test]
fn sweep_plane_builder_does_not_split_nodes_thinner_than_the_minimum_size() {
    let mut set = mesh(&[0.0, 0.0, 0.0, 0.0]);
    for vertex in set.vertices.iter_mut() {
        *vertex = BvhVec3d::new(1.0, 2.0, 3.0);
    }
    let mut tree = BvhTree::default();
    let bounding = set.combined_boxes();
    BvhSweepPlaneBuilder::new(1, 32, 1)
        .build(&mut set, &mut tree, &bounding)
        .unwrap();
    assert_eq!(tree.length(), 1);
    assert_eq!(tree.nb_primitives(0).unwrap(), 4);
}

#[test]
fn spatial_median_builder_is_the_two_bin_binned_builder() {
    let xs: Vec<f64> = (0..40)
        .map(|index| f64::from(index * 13 % 40) * 0.5)
        .collect();
    let mut median_set = mesh(&xs);
    let mut binned_set = mesh(&xs);
    let bounding = median_set.combined_boxes();
    let (mut median, mut binned) = (BvhTree::default(), BvhTree::default());
    BvhSpatialMedianBuilder::new(2, 32, true)
        .build(&mut median_set, &mut median, &bounding)
        .unwrap();
    BvhBinnedBuilder::<BvhVec3d, 2>::new(2, 32, true, 1)
        .build(&mut binned_set, &mut binned, &bounding)
        .unwrap();
    assert_eq!(
        median.node_info_buffer().as_slice(),
        binned.node_info_buffer().as_slice()
    );
    assert_eq!(
        median.min_point_buffer().as_slice(),
        binned.min_point_buffer().as_slice()
    );
}

#[test]
fn box_sets_center_on_the_box_and_index_through_the_permutation() {
    let mut boxes = box_set(
        Arc::new(BvhBinnedBuilder::<BvhVec3d>::default()),
        &[0.0, 5.0],
    );
    assert_eq!(boxes.center(1, 0).unwrap(), 5.5);
    let failure = boxes.element_box(7).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains('7'), "{}", failure.message());
    boxes.swap(0, 1).unwrap();
    assert_eq!(boxes.element(0).unwrap(), 1);
    let mut indexed = BvhIndexedBoxSet::<BvhVec3d, &str>::default();
    indexed.add("left", unit_box(0.0));
    indexed.add("right", unit_box(5.0));
    indexed.swap(0, 1).unwrap();
    assert_eq!(indexed.element(0).unwrap(), "right");
    assert_eq!(indexed.element_box(0).unwrap(), unit_box(5.0));
    indexed.clear();
    assert_eq!(indexed.size(), 0);
    assert!(indexed.is_dirty());
}

#[test]
fn object_set_swaps_handles_and_reports_null_ones() {
    let mut objects = BvhObjectSet::<BvhVec3d>::default();
    let mut first = mesh(&[3.0]);
    first.mark_dirty();
    objects.objects_mut().append(Some(Box::new(first)));
    objects.objects_mut().append(None);
    let failure = objects.element_box(1).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    objects.swap(0, 0).unwrap();
    assert_eq!(objects.element_box(0).unwrap().corner_min().x, 3.0);
    objects.swap(0, 1).unwrap();
    assert!(objects.objects().value(0).unwrap().is_none());
    assert_eq!(objects.element_box(1).unwrap().corner_min().x, 3.0);
    assert!(objects.swap(0, 2).is_err());
    objects.clear();
    assert_eq!(objects.size(), 0);
}

#[test]
fn geometry_starts_clean_and_fails_on_a_null_builder() {
    let mut geometry = BvhGeometry::<BvhVec3d>::default();
    let mut object = mesh(&[1.0]);
    object.mark_dirty();
    geometry.objects_mut().append(Some(Box::new(object)));
    assert!(!geometry.is_dirty());
    assert!(!geometry.cached_box().unwrap().is_valid());
    assert_eq!(geometry.bvh().unwrap().length(), 0);
    geometry.mark_dirty();
    geometry.set_builder(None);
    let failure = geometry.update().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    assert!(geometry.is_dirty());
}

/// Accepts everything with the default left-to-right rules.
struct Everything;
impl BvhBaseTraverse<f64> for Everything {}
impl BvhTraverse<BvhVec3d> for Everything {
    fn reject_node(&self, _: &BvhVec3d, _: &BvhVec3d, _: &mut f64) -> bool {
        false
    }
    fn accept(&mut self, _: i32, _: &f64) -> Result<bool, Failure> {
        Ok(true)
    }
}
impl BvhPairTraverse<BvhVec3d> for Everything {
    fn reject_node(
        &self,
        _: &BvhVec3d,
        _: &BvhVec3d,
        _: &BvhVec3d,
        _: &BvhVec3d,
        _: &mut f64,
    ) -> bool {
        false
    }
    fn accept(&mut self, _: i32, _: i32) -> Result<bool, Failure> {
        Ok(true)
    }
}

/// A chain of `inner` nodes whose left child is the next inner node and whose
/// right child is a leaf: every level stacks one right leaf.
fn left_chain(inner: i32) -> BvhTree<BvhVec3d> {
    let mut tree = BvhTree::default();
    let point = BvhVec3d::new(0.0, 0.0, 0.0);
    for level in 0..inner {
        tree.add_inner_node(point, point, 2 * level + 2, 2 * level + 1);
        tree.add_leaf_node(point, point, level, level);
    }
    tree.add_leaf_node(point, point, inner, inner);
    tree
}

#[test]
fn traversal_stacks_raise_program_errors_past_their_capacity() {
    // Pair traversal stacks two pairs per level of these chains.
    let deep = left_chain(60);
    let failure = BvhTraverse::select_tree(&mut Everything, Some(&deep)).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(failure.message().contains("BVH stack overflow"));
    assert_eq!(
        BvhTraverse::select_tree(&mut Everything, Some(&left_chain(31))).unwrap(),
        32
    );
    let failure = Everything
        .select_trees(Some(&deep), Some(&deep))
        .unwrap_err();
    assert!(failure.message().contains("BVH pair stack overflow"));
}

/// Point-to-triangle distance selector of the `BVH_Distance` documentation.
struct NearestTriangle<'a> {
    state: BvhDistance<f64, BvhVec3d>,
    set: Option<&'a mut BvhTriangulation<BvhVec3d>>,
}
impl BvhBaseTraverse<f64> for NearestTriangle<'_> {
    fn is_metric_better(&self, left: &f64, right: &f64) -> bool {
        self.state.is_metric_better(left, right)
    }
    fn reject_metric(&self, metric: &f64) -> bool {
        self.state.reject_metric(metric)
    }
    fn stop(&self) -> bool {
        self.state.stop()
    }
}
impl BvhTraverse<BvhVec3d> for NearestTriangle<'_> {
    fn reject_node(&self, min: &BvhVec3d, max: &BvhVec3d, metric: &mut f64) -> bool {
        *metric = rustycad_tkmath::bvh::tools::point_box_square_distance_by_corners(
            self.state.object(),
            min,
            max,
        );
        self.state.reject_metric(metric)
    }
    fn accept(&mut self, index: i32, _: &f64) -> Result<bool, Failure> {
        let set = self.set.as_deref().unwrap();
        let bounding = set.element_box(index)?;
        let distance = (bounding.corner_min().x - self.state.object().x).powi(2);
        if distance < self.state.distance() {
            self.state.set_distance(distance);
            return Ok(true);
        }
        Ok(false)
    }
}
impl BvhSetTraverse<BvhVec3d> for NearestTriangle<'_> {
    type Set = BvhTriangulation<BvhVec3d>;
    fn bvh_set_mut(&mut self) -> Option<&mut BvhTriangulation<BvhVec3d>> {
        self.set.as_deref_mut()
    }
}
impl BvhDistanceTraverse<BvhVec3d> for NearestTriangle<'_> {
    type Object = BvhVec3d;
    fn distance_state(&self) -> &BvhDistance<f64, BvhVec3d> {
        &self.state
    }
    fn distance_state_mut(&mut self) -> &mut BvhDistance<f64, BvhVec3d> {
        &mut self.state
    }
}

#[test]
fn set_selection_builds_the_dirty_tree_and_gives_it_back() {
    let mut set = mesh(&[0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
    set.mark_dirty();
    let mut state = BvhDistance::default();
    state.set_object(BvhVec3d::new(21.0, 0.0, 0.0));
    let mut tool = NearestTriangle {
        state,
        set: Some(&mut set),
    };
    assert_eq!(tool.compute_distance().unwrap(), 1.0);
    assert!(tool.state.is_done());
    assert!(!set.is_dirty());
    assert!(set.bvh().unwrap().length() > 1);
    let mut unset = NearestTriangle {
        state: BvhDistance::default(),
        set: None,
    };
    assert_eq!(unset.compute_distance().unwrap(), f64::MAX);
    assert!(!unset.state.is_done());
}

#[test]
fn distance_field_rejects_bad_sizes_and_voxel_offsets() {
    let failure = BvhDistanceField::<BvhVec3d>::new(-1, true).unwrap_err();
    assert_eq!(failure.exception_type(), "std::bad_array_new_length");
    assert!(BvhDistanceField::<BvhVec3d>::new(1291, true).is_err());
    let field = BvhDistanceField::<BvhVec3d>::new(4, true).unwrap();
    assert_eq!(field.packed_data().len(), 64);
    assert!(field.voxel(0, 0, 0).is_ok());
    let failure = field.voxel(64, 0, 0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn distance_to_other_objects_is_the_scalar_maximum() {
    let mut boxes = box_set(Arc::new(BvhBinnedBuilder::<BvhVec3d>::default()), &[0.0]);
    let mut is_outside = false;
    let point = BvhVec3d::new(0.5, 0.5, 0.5);
    let distance = square_distance_to_object(Some(&mut boxes), &point, &mut is_outside).unwrap();
    assert_eq!(distance, f64::MAX);
    assert!(!is_outside);
    assert_eq!(
        square_distance_to_object(None, &point, &mut is_outside).unwrap(),
        f64::MAX
    );
}

#[test]
fn distance_field_of_a_clean_geometry_holds_the_root_of_the_scalar_maximum() {
    let mut geometry = BvhGeometry::<BvhVec3d>::default();
    geometry.objects_mut().append(Some(Box::new(mesh(&[0.0]))));
    let mut field = BvhDistanceField::new(4, true).unwrap();
    assert!(field.build(&mut geometry).unwrap());
    assert_eq!(field.dimension_x(), 4);
    // An invalid geometry box gives infinite corners and NaN voxel centers.
    assert_eq!(field.corner_min().x, f64::INFINITY);
    let root = f64::MAX.sqrt();
    assert!(field.packed_data().iter().all(|voxel| *voxel == root));
}

#[test]
fn direction_to_nearest_point_accepts_edge_boundaries() {
    let (a, b, c) = (
        BvhVec3d::new(0.0, 0.0, 0.0),
        BvhVec3d::new(2.0, 0.0, 0.0),
        BvhVec3d::new(0.0, 2.0, 0.0),
    );
    let above = direction_to_nearest_point(&BvhVec3d::new(0.5, 0.5, 3.0), &a, &b, &c);
    assert_eq!(above, BvhVec3d::new(0.0, 0.0, 3.0));
    let below_edge = direction_to_nearest_point(&BvhVec3d::new(1.0, -1.0, 0.0), &a, &b, &c);
    assert_eq!(below_edge, BvhVec3d::new(0.0, -1.0, 0.0));
    assert_eq!(
        direction_to_nearest_point(&a, &a, &b, &c),
        BvhVec3d::new(0.0, 0.0, 0.0)
    );
}
