//! Shared-set worker path for `BVH_QueueBuilder::Build`.
use super::super::bounding_box::BvhBox;
use super::super::build_queue::BvhBuildQueue;
use super::super::build_thread::{BvhBuildThread, BvhBuildTool};
use super::super::queue_builder::{BvhChildNodes, BvhParallelQueueBuilder};
use super::super::set::BvhSharedSet;
use super::super::tree::{BvhNodeSnapshot, BvhTree};
use super::super::vector::BvhVector;
use super::build::run_workers;
use rustycad_tkernel::standard::Failure;
use std::sync::{Mutex, PoisonError};

struct SharedBuildTool<'a, V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized> {
    tree: Mutex<BvhTree<V>>,
    set: &'a dyn BvhSharedSet<V>,
    queue: &'a BvhBuildQueue,
    builder: &'a B,
}

impl<V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized> BvhBuildTool
    for SharedBuildTool<'_, V, B>
{
    fn perform(&self, node: i32) -> Result<(), Failure> {
        let snapshot = self.snapshot(node)?;
        let children = self.builder.build_node(self.set, snapshot)?;
        if !children.is_valid() {
            return Ok(());
        }
        self.publish_children(node, &children)
    }
}

impl<V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized> SharedBuildTool<'_, V, B> {
    fn snapshot(&self, node: i32) -> Result<BvhNodeSnapshot<V>, Failure> {
        self.lock_tree().node_snapshot(node)
    }

    fn publish_children(&self, node: i32, children: &BvhChildNodes<V>) -> Result<(), Failure> {
        let mut tree = self.lock_tree();
        tree.set_inner(node)?;
        self.builder
            .add_children(&mut tree, self.queue, node, children)
    }

    fn lock_tree(&self) -> std::sync::MutexGuard<'_, BvhTree<V>> {
        self.tree.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Builds a BVH with parallel split callbacks over a synchronized set.
/// Example: `parallel_queue_build(&builder, &set, &mut tree, &bounds)?`.
pub fn parallel_queue_build<V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized>(
    builder: &B,
    set: &dyn BvhSharedSet<V>,
    tree: &mut BvhTree<V>,
    bounding: &BvhBox<V>,
) -> Result<(), Failure> {
    tree.clear();
    let size = set.size();
    if size == 0 {
        return Ok(());
    }
    let root = tree.add_leaf_node_box(bounding, 0, size - 1);
    if size == 1 {
        return Ok(());
    }
    let queue = BvhBuildQueue::new();
    queue.enqueue(root);
    prepare_workers(builder, set, tree, &queue, size)
}

fn prepare_workers<V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized>(
    builder: &B,
    set: &dyn BvhSharedSet<V>,
    tree: &mut BvhTree<V>,
    queue: &BvhBuildQueue,
    size: i32,
) -> Result<(), Failure> {
    if builder.num_of_threads() <= 1 {
        return run_single_worker(builder, set, tree, queue);
    }
    tree.reserve(2 * size - 1)?;
    run_parallel_workers(builder, set, tree, queue)?;
    tree.reserve(tree.length())
}

fn run_parallel_workers<V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized>(
    builder: &B,
    set: &dyn BvhSharedSet<V>,
    tree: &mut BvhTree<V>,
    queue: &BvhBuildQueue,
) -> Result<(), Failure> {
    let tool = shared_build_tool(builder, set, tree, queue);
    let result = run_workers(&tool, queue, builder.num_of_threads());
    *tree = tool.into_tree();
    result
}

fn shared_build_tool<'a, V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized>(
    builder: &'a B,
    set: &'a dyn BvhSharedSet<V>,
    tree: &mut BvhTree<V>,
    queue: &'a BvhBuildQueue,
) -> SharedBuildTool<'a, V, B> {
    SharedBuildTool {
        tree: Mutex::new(std::mem::take(tree)),
        set,
        queue,
        builder,
    }
}

fn run_single_worker<V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized>(
    builder: &B,
    set: &dyn BvhSharedSet<V>,
    tree: &mut BvhTree<V>,
    queue: &BvhBuildQueue,
) -> Result<(), Failure> {
    let tool = SharedBuildTool {
        tree: Mutex::new(std::mem::take(tree)),
        set,
        queue,
        builder,
    };
    let result = BvhBuildThread::new(&tool, queue).execute();
    *tree = tool.into_tree();
    result
}

impl<V: BvhVector, B: BvhParallelQueueBuilder<V> + ?Sized> SharedBuildTool<'_, V, B> {
    fn into_tree(self) -> BvhTree<V> {
        self.tree
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
