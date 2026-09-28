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

//! Supplemental BVH behavior beyond the legacy suites: native quirks, errors,
//! JSON, primitive-set state, builders, sorters and threads. Expected dump texts
//! and IntFloor results were emitted by the pinned native build.
use rustycad_tkernel::ncollection::{Mat4, Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBox, BvhBuildQueue, BvhBuildThread, BvhBuildTool, BvhBuilder, BvhObject,
    BvhPrimitiveSetAccess, BvhProperties, BvhRadixSorter, BvhSet, BvhSorter, BvhTransform, BvhTree,
    BvhTriangulation, BvhVec3d, BvhVec4d, BvhVec4i, array, int_floor, main_axis,
};
use std::sync::Arc;

fn triangulation(centers: &[f64]) -> BvhTriangulation<BvhVec3d> {
    let mut set = BvhTriangulation::default();
    for (index, x) in centers.iter().enumerate() {
        let first = 3 * index as i32;
        for offset in [-0.5, 0.0, 0.5] {
            array::append(&mut set.vertices, BvhVec3d::new(x + offset, offset, 0.0));
        }
        array::append(
            &mut set.elements,
            BvhVec4i::new(first, first + 1, first + 2, 0),
        );
    }
    set
}

#[test]
fn int_floor_reproduces_x86_truncation_outside_the_int_range() {
    assert_eq!(int_floor(-3.5_f64), -4);
    assert_eq!(int_floor(-0.0_f64), 0);
    assert_eq!(int_floor(f64::NAN), i32::MIN);
    assert_eq!(int_floor(f64::INFINITY), i32::MIN);
    // INT_MIN > -inf, so upstream subtracts one and wraps to INT_MAX.
    assert_eq!(int_floor(f64::NEG_INFINITY), i32::MAX);
    assert_eq!(int_floor(2_147_483_648.0_f64), i32::MIN);
    assert_eq!(int_floor(2_147_483_647.5_f64), i32::MAX);
    // The float correction compares in single precision: 16777217 -> 16777216.
    assert_eq!(int_floor(16_777_217.0_f32), 16_777_216);
}

#[test]
fn box_json_round_trips_and_keeps_partial_writes() {
    let bounding = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(1.0, 2.0, 3.0));
    let mut dump = DumpStream::default();
    bounding.dump_json(&mut dump).unwrap();
    assert_eq!(
        dump.text(),
        "\"IsValid\": 1, \"MinPoint\": [0, 0, 0], \"MaxPoint\": [1, 2, 3]"
    );
    let mut parsed = BvhBox::from_point(BvhVec3d::new(7.0, 7.0, 7.0));
    let mut position = 1;
    assert!(parsed.init_from_json(dump.text(), &mut position).unwrap());
    assert_eq!(parsed, bounding);
    // Only MinPoint survives a dump cut inside MaxPoint.
    let mut partial = BvhBox::from_point(BvhVec3d::new(7.0, 7.0, 7.0));
    let mut cut_position = 1;
    let cut = &dump.text()[..45];
    assert!(!partial.init_from_json(cut, &mut cut_position).unwrap());
    assert_eq!(*partial.corner_min(), BvhVec3d::new(0.0, 0.0, 0.0));
    assert_eq!(*partial.corner_max(), BvhVec3d::new(7.0, 7.0, 7.0));
    assert_eq!(cut_position, 1);
}

#[test]
fn box_json_widens_the_fourth_dimension_and_clears_invalid_boxes() {
    let four = BvhBox::from_corners(BvhVec4d::new(1.0, 2.0, 3.0, 4.0), BvhVec4d::from_value(5.0));
    let mut dump = DumpStream::default();
    four.dump_json(&mut dump).unwrap();
    let mut parsed = BvhBox::<BvhVec4d>::default();
    assert!(parsed.init_from_json(dump.text(), &mut 1).unwrap());
    assert_eq!(
        (parsed.corner_min().w, parsed.corner_max().w),
        (-f64::MAX, f64::MAX)
    );
    let mut invalid = BvhBox::from_point(BvhVec4d::from_value(1.0));
    assert!(invalid.init_from_json("\"IsValid\": 0", &mut 1).unwrap());
    assert!(!invalid.is_valid());
}

#[test]
fn tree_accessors_report_the_offending_index() {
    let mut tree = BvhTree::<BvhVec3d>::default();
    let leaf = tree.add_leaf_node_uninit(0, 3);
    let failure: Failure = tree.min_point(leaf).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("index 0, expected 0..0"));
    assert!(tree.level(-1).is_err());
    assert!(tree.reserve(-1).is_err());
    assert_eq!(tree.nb_primitives(leaf).unwrap(), 4);
}

fn four_leaf_tree() -> BvhTree<BvhVec3d> {
    let mut tree = BvhTree::default();
    let unit = |x: f64| (BvhVec3d::new(x, 0.0, 0.0), BvhVec3d::new(x + 1.0, 1.0, 1.0));
    // Node 0 must be the root for SAH and collapse, as builders arrange it.
    let root = tree.add_inner_node(unit(0.0).0, unit(3.0).1, -1, -1);
    let leaves: Vec<i32> = (0..4)
        .map(|index| {
            let (min, max) = unit(f64::from(index));
            tree.add_leaf_node(min, max, index, index)
        })
        .collect();
    let left = tree.add_inner_node(unit(0.0).0, unit(1.0).1, leaves[0], leaves[1]);
    let right = tree.add_inner_node(unit(2.0).0, unit(3.0).1, leaves[2], leaves[3]);
    *tree.child_mut::<0>(root).unwrap() = left;
    *tree.child_mut::<1>(root).unwrap() = right;
    tree
}

#[test]
fn collapse_to_quad_tree_adopts_grandchildren_breadth_first() {
    let tree = four_leaf_tree();
    let quad = tree.collapse_to_quad_tree().unwrap();
    assert_eq!((quad.length(), quad.depth()), (5, 1));
    assert_eq!(
        *array::value(quad.node_info_buffer(), 0).unwrap(),
        BvhVec4i::new(0, 1, 3, 0)
    );
    assert_eq!(quad.child::<3>(0).unwrap(), 4);
    assert!(quad.is_outer(4).unwrap());
    assert!(tree.estimate_sah().unwrap() > 0.0);
}

#[test]
fn tree_dump_uses_the_native_keys() {
    let mut tree = BvhTree::<BvhVec3d>::default();
    tree.add_leaf_node(
        BvhVec3d::new(0.0, 0.0, 0.0),
        BvhVec3d::new(6.0, 6.0, 5.0),
        0,
        1,
    );
    let mut dump = DumpStream::default();
    tree.dump_json(&mut dump, -1).unwrap();
    assert_eq!(
        dump.text(),
        "\"className\": \"BVH_TreeBaseTransient\", \"Depth\": 0, \"Length\": 1, \
         \"className\": \"BVH_TreeNode\", \"theNodeIndex\": 0, \"Pointer\": {\"CornerMin\": [0, 0, 0], \
         \"CornerMax\": [6, 6, 5], \"Gap\": 0, \"Flags\": 0}, \"BegPrimitive(theNodeIndex)\": 0, \
         \"EndPrimitive(theNodeIndex)\": 1, \"Level(theNodeIndex)\": 0, \"IsOuter(theNodeIndex)\": 1"
    );
    let mut shallow = DumpStream::default();
    tree.dump_node(0, &mut shallow, 0).unwrap();
    assert!(!shallow.text().contains("Pointer"));
}

#[test]
fn primitive_set_starts_clean_and_builds_only_when_dirty() {
    let mut set = triangulation(&[4.0, 0.0, 2.0, 8.0, 6.0, 1.0, 3.0]);
    assert!(!set.is_dirty());
    assert!(!set.bounding_box().unwrap().is_valid());
    assert_eq!(set.bvh().unwrap().length(), 0);
    set.mark_dirty();
    let tree = set.bvh().unwrap().clone();
    assert!(!set.is_dirty());
    assert!(tree.length() >= 3);
    assert_eq!(set.object_box().unwrap().corner_max().x, 8.5);
    set.set_builder(None);
    set.mark_dirty();
    let failure = set.update().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
}

#[test]
fn binned_builder_splits_coincident_centroids_at_the_index_middle() {
    let mut set = triangulation(&[1.0; 6]);
    let builder = BvhBinnedBuilder::<BvhVec3d, 4>::new(1, 32, false, 1);
    let mut tree = BvhTree::default();
    let bounding = set.element_box(0).unwrap();
    builder.build(&mut set, &mut tree, &bounding).unwrap();
    // float((0 + 5) / 2.f) = 2.5 truncates to 2: the left child covers [0, 1],
    // and its children (nodes 3 and 4) are single-primitive leaves, as natively.
    assert_eq!((tree.length(), tree.depth()), (11, 4));
    assert_eq!(tree.child::<0>(tree.child::<0>(0).unwrap()).unwrap(), 3);
    assert_eq!(
        (
            tree.beg_primitive(3).unwrap(),
            tree.end_primitive(4).unwrap()
        ),
        (0, 1)
    );
    assert!(tree.is_outer(3).unwrap() && tree.is_outer(4).unwrap());
    assert_eq!(main_axis(&BvhVec3d::new(1.0, 1.0, 1.0)), 0);
    assert_eq!(main_axis(&BvhVec3d::new(1.0, 2.0, 2.0)), 2);
}

fn leaf_ranges(tree: &BvhTree<BvhVec3d>) -> Vec<(i32, i32)> {
    let mut ranges: Vec<(i32, i32)> = (0..tree.length())
        .filter(|node| tree.is_outer(*node).unwrap())
        .map(|node| {
            (
                tree.beg_primitive(node).unwrap(),
                tree.end_primitive(node).unwrap(),
            )
        })
        .collect();
    ranges.sort_unstable();
    ranges
}

#[test]
fn multi_threaded_queue_build_covers_every_primitive_once() {
    let centers: Vec<f64> = (0..200)
        .map(|index| f64::from((index * 37) % 200))
        .collect();
    let mut set = triangulation(&centers);
    set.mark_dirty();
    let bounding = set.bounding_box().unwrap();
    let builder = BvhBinnedBuilder::<BvhVec3d, 16>::new(4, 32, false, 4);
    builder.builder_transient().set_parallel(true);
    let mut tree = BvhTree::default();
    builder.build(&mut set, &mut tree, &bounding).unwrap();
    let ranges = leaf_ranges(&tree);
    assert_eq!(ranges.first().unwrap().0, 0);
    assert_eq!(ranges.last().unwrap().1, 199);
    assert!(ranges.windows(2).all(|pair| pair[0].1 + 1 == pair[1].0));
    assert!(ranges.iter().all(|(begin, end)| end - begin < 4));
}

#[test]
fn radix_sorter_orders_by_morton_code_in_serial_and_parallel_mode() {
    let centers = [9.0, 3.0, 7.0, 3.0, 0.0, 5.0, 1.0];
    let mut serial = triangulation(&centers);
    let mut parallel = triangulation(&centers);
    serial.mark_dirty();
    let bounding = serial.bounding_box().unwrap();
    let mut sorter = BvhRadixSorter::new(bounding);
    assert!(sorter.encoded_links().is_none());
    sorter.perform(&mut serial).unwrap();
    let codes: Vec<u32> = sorter
        .encoded_links()
        .unwrap()
        .iter()
        .map(|link| link.0)
        .collect();
    assert!(codes.windows(2).all(|pair| pair[0] <= pair[1]));
    let mut concurrent = BvhRadixSorter::new(bounding);
    concurrent.set_parallel(true);
    concurrent.perform(&mut parallel).unwrap();
    assert_eq!(serial.elements.as_slice(), parallel.elements.as_slice());
    assert_eq!(serial.center(0, 0).unwrap(), 0.0);
}

#[test]
fn transform_constructor_keeps_identity_inverse_until_set() {
    let mut matrix = Mat4::<f64>::identity();
    matrix.set_diagonal(Vec4::new(2.0, 4.0, 1.0, 1.0));
    matrix.set_column3(3, Vec3::new(1.0, 0.0, 0.0));
    let constructed = BvhTransform::new(matrix);
    assert!(constructed.inversed().is_identity());
    let mut assigned = BvhTransform::default();
    assigned.set_transform(matrix);
    assert_eq!(
        assigned.inversed().get_diagonal(),
        Vec4::new(0.5, 0.25, 1.0, 1.0)
    );
    // Apply keeps the minimum W on every corner; W = 0 would divide by zero.
    let unit = BvhBox::from_corners(BvhVec4d::new(0.0, 0.0, 0.0, 1.0), BvhVec4d::from_value(1.0));
    let moved = assigned.apply(&unit);
    assert_eq!(*moved.corner_min(), BvhVec4d::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(*moved.corner_max(), BvhVec4d::new(3.0, 4.0, 1.0, 1.0));
    let properties: Arc<dyn BvhProperties> = Arc::new(assigned);
    let mut set = triangulation(&[0.0]);
    set.set_properties(Some(properties));
    let stored = set
        .properties()
        .unwrap()
        .as_any()
        .downcast_ref::<BvhTransform<f64>>();
    assert_eq!(stored.unwrap().transform(), &matrix);
}

/// Fails on node 3 so a worker abandons the queue mid-build.
struct FailingTool<'a> {
    queue: &'a BvhBuildQueue,
}
impl BvhBuildTool for FailingTool<'_> {
    fn perform(&self, node: i32) -> Result<(), Failure> {
        if node == 3 {
            return Err(Failure::new("node 3 cannot be split"));
        }
        self.queue.enqueue(node + 1);
        Ok(())
    }
}

#[test]
fn failing_build_tool_releases_its_busy_slot() {
    let queue = BvhBuildQueue::new();
    queue.enqueue(0);
    let tool = FailingTool { queue: &queue };
    let worker = BvhBuildThread::new(&tool, &queue);
    let failure = std::thread::scope(|scope| BvhBuildThread::wait(worker.run(scope))).unwrap_err();
    assert_eq!(failure.message(), "node 3 cannot be split");
    assert!(!queue.has_busy_threads());
}
