//! Supplemental evidence for the current serial `BvhQueueBuilder` adapter.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBox, BvhBoxSet, BvhBuilderTransient, BvhChildNodes, BvhMutexSet,
    BvhNodeSnapshot, BvhParallelQueueBuilder, BvhPrimitiveRange, BvhQueueBuilder, BvhSet,
    BvhSharedSet, BvhTree, BvhVec3d, parallel_queue_build, queue_build,
};
use rustycad_tkmath::bvh::{BvhBuildQueue, BvhBuildThread, BvhBuildTool};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

struct TimedRendezvous {
    arrivals: Mutex<usize>,
    changed: Condvar,
}

impl TimedRendezvous {
    fn wait_for_partner(&self) {
        let mut arrivals = self.arrivals.lock().unwrap();
        *arrivals += 1;
        if *arrivals == 2 {
            self.changed.notify_all();
            return;
        }
        let _ = self
            .changed
            .wait_timeout_while(arrivals, Duration::from_millis(40), |count| *count < 2)
            .unwrap();
    }
}

struct RecordingBuilder {
    transient: BvhBuilderTransient,
    active: AtomicUsize,
    maximum: AtomicUsize,
    rendezvous: TimedRendezvous,
}

impl RecordingBuilder {
    fn new() -> Self {
        Self {
            transient: BvhBuilderTransient::new(1, 8),
            active: AtomicUsize::new(0),
            maximum: AtomicUsize::new(0),
            rendezvous: TimedRendezvous {
                arrivals: Mutex::new(0),
                changed: Condvar::new(),
            },
        }
    }

    fn record_enter(&self) {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum.fetch_max(active, Ordering::SeqCst);
    }
}

impl BvhQueueBuilder<BvhVec3d> for RecordingBuilder {
    fn queue_builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }

    fn num_of_threads(&self) -> i32 {
        2
    }

    fn build_node(
        &self,
        _set: &mut dyn BvhSet<BvhVec3d>,
        tree: &mut BvhTree<BvhVec3d>,
        node: i32,
    ) -> Result<BvhChildNodes<BvhVec3d>, Failure> {
        self.record_enter();
        let children = self.split_node(tree, node)?;
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(children)
    }
}

impl RecordingBuilder {
    fn split_node(
        &self,
        tree: &mut BvhTree<BvhVec3d>,
        node: i32,
    ) -> Result<BvhChildNodes<BvhVec3d>, Failure> {
        let first = tree.beg_primitive(node)?;
        let last = tree.end_primitive(node)?;
        let bounds = BvhBox::from_corners(*tree.min_point(node)?, *tree.max_point(node)?);
        self.wait_for_pair(first, last);
        let children = child_nodes(first, last, bounds);
        if children.is_valid() {
            tree.set_inner(node)?;
        }
        Ok(children)
    }

    fn wait_for_pair(&self, first: i32, last: i32) {
        if last - first + 1 == 2 {
            self.rendezvous.wait_for_partner();
        }
    }
}

impl BvhParallelQueueBuilder<BvhVec3d> for RecordingBuilder {
    fn queue_builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }

    fn num_of_threads(&self) -> i32 {
        2
    }

    fn build_node(
        &self,
        _set: &dyn BvhSharedSet<BvhVec3d>,
        node: BvhNodeSnapshot<BvhVec3d>,
        _mark_parent_inner: &mut dyn FnMut() -> Result<(), Failure>,
    ) -> Result<BvhChildNodes<BvhVec3d>, Failure> {
        self.record_enter();
        self.wait_for_pair(node.begin, node.end);
        let bounds = BvhBox::from_corners(node.minimum, node.maximum);
        let children = child_nodes(node.begin, node.end, bounds);
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(children)
    }
}

fn child_nodes(first: i32, last: i32, bounds: BvhBox<BvhVec3d>) -> BvhChildNodes<BvhVec3d> {
    if first == last {
        return BvhChildNodes::default();
    }
    let middle = first + (last - first + 1) / 2;
    BvhChildNodes::new(
        bounds,
        bounds,
        BvhPrimitiveRange::new(first, middle - 1),
        BvhPrimitiveRange::new(middle, last),
    )
}

fn set_with_four_boxes() -> BvhBoxSet<BvhVec3d> {
    let mut set = BvhBoxSet::<BvhVec3d>::new(None);
    for index in 0..4 {
        let low = BvhVec3d::new(f64::from(index), 0.0, 0.0);
        let high = BvhVec3d::new(f64::from(index + 1), 1.0, 1.0);
        set.add(index, BvhBox::from_corners(low, high));
    }
    set
}

fn full_bounds() -> BvhBox<BvhVec3d> {
    BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(4.0, 1.0, 1.0))
}

#[test]
fn generic_queue_builder_currently_serializes_node_callbacks() {
    let mut set = set_with_four_boxes();
    let builder = RecordingBuilder::new();
    let mut tree = BvhTree::default();

    queue_build(&builder, &mut set, &mut tree, &full_bounds()).unwrap();

    assert_eq!(*builder.rendezvous.arrivals.lock().unwrap(), 2);
    assert_eq!(builder.maximum.load(Ordering::SeqCst), 1);
    assert_eq!(tree.length(), 7);
}

#[test]
fn shared_set_builder_runs_native_parallel_child_splits() {
    let set = BvhMutexSet::new(set_with_four_boxes());
    let builder = RecordingBuilder::new();
    let mut tree = BvhTree::default();

    parallel_queue_build(&builder, &set, &mut tree, &full_bounds()).unwrap();

    assert_eq!(builder.maximum.load(Ordering::SeqCst), 2);
    assert_eq!(tree.length(), 7);
    assert!(!tree.is_outer(0).unwrap());
}

#[test]
fn parallel_binned_builder_preserves_leaf_ranges() {
    let mut serial_set = set_with_four_boxes();
    let mut serial_tree = BvhTree::default();
    let serial = BvhBinnedBuilder::<BvhVec3d, 4>::new(1, 8, false, 1);
    queue_build(&serial, &mut serial_set, &mut serial_tree, &full_bounds()).unwrap();

    let shared_set = BvhMutexSet::new(set_with_four_boxes());
    let mut parallel_tree = BvhTree::default();
    let parallel = BvhBinnedBuilder::<BvhVec3d, 4>::new(1, 8, false, 2);
    parallel_queue_build(&parallel, &shared_set, &mut parallel_tree, &full_bounds()).unwrap();

    assert_eq!(leaf_ranges(&serial_tree), leaf_ranges(&parallel_tree));
    assert_eq!(parallel_tree.length(), serial_tree.length());
    assert_eq!(shared_set.into_inner().unwrap().size(), 4);
}

fn leaf_ranges(tree: &BvhTree<BvhVec3d>) -> Vec<(i32, i32)> {
    let mut ranges = (0..tree.length())
        .filter_map(|node| {
            tree.is_outer(node)
                .ok()
                .filter(|is_leaf| *is_leaf)
                .map(|_| {
                    (
                        tree.beg_primitive(node).unwrap(),
                        tree.end_primitive(node).unwrap(),
                    )
                })
        })
        .collect::<Vec<_>>();
    ranges.sort_unstable();
    ranges
}

struct PanickingBuildTool;

impl BvhBuildTool for PanickingBuildTool {
    fn perform(&self, _node: i32) -> Result<(), Failure> {
        panic!("injected worker panic")
    }
}

#[test]
fn worker_panic_releases_busy_count_before_unwinding() {
    let queue = BvhBuildQueue::new();
    queue.enqueue(7);
    let worker = BvhBuildThread::new(&PanickingBuildTool, &queue);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker.execute()));

    assert!(outcome.is_err());
    assert!(!queue.has_busy_threads());
}

struct FailOnSwapSet {
    inner: BvhBoxSet<BvhVec3d>,
}

impl BvhSet<BvhVec3d> for FailOnSwapSet {
    fn size(&self) -> i32 {
        self.inner.size()
    }

    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.inner.element_box(index)
    }

    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        self.inner.center(index, axis)
    }

    fn swap(&mut self, _first: i32, _second: i32) -> Result<(), Failure> {
        Err(Failure::new("injected partition failure"))
    }
}

fn reverse_order_boxes() -> BvhBoxSet<BvhVec3d> {
    let mut set = BvhBoxSet::<BvhVec3d>::new(None);
    for index in 0..4 {
        let position = f64::from(3 - index);
        let low = BvhVec3d::new(position, 0.0, 0.0);
        let high = BvhVec3d::new(position + 1.0, 1.0, 1.0);
        set.add(index, BvhBox::from_corners(low, high));
    }
    set
}

#[test]
fn binned_split_failure_keeps_native_inner_parent_state() {
    let parallel_tree = failed_parallel_tree();
    assert!(!parallel_tree.is_outer(0).unwrap());
}

fn failed_parallel_tree() -> BvhTree<BvhVec3d> {
    let builder = BvhBinnedBuilder::<BvhVec3d, 4>::new(1, 8, false, 2);
    let mut set = FailOnSwapSet {
        inner: reverse_order_boxes(),
    };
    let mut tree = BvhTree::default();
    assert!(
        rustycad_tkmath::bvh::BvhBuilder::build(&builder, &mut set, &mut tree, &full_bounds(),)
            .is_err()
    );
    tree
}

#[test]
fn binned_builder_build_uses_parallel_shared_set_when_configured() {
    let builder = BvhBinnedBuilder::<BvhVec3d, 4>::new(1, 8, false, 2);
    let mut set = set_with_four_boxes();
    let mut tree = BvhTree::default();

    rustycad_tkmath::bvh::BvhBuilder::build(&builder, &mut set, &mut tree, &full_bounds()).unwrap();

    assert_eq!(tree.length(), 7);
    assert_eq!(leaf_ranges(&tree).len(), 4);
}
