//! Supplemental evidence for the current serial `BvhQueueBuilder` adapter.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{
    BvhBox, BvhBoxSet, BvhBuilderTransient, BvhChildNodes, BvhPrimitiveRange, BvhQueueBuilder,
    BvhSet, BvhTree, BvhVec3d, queue_build,
};
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
        if last - first + 1 == 2 {
            self.rendezvous.wait_for_partner();
        }
        if first == last {
            return Ok(BvhChildNodes::default());
        }
        let middle = first + (last - first + 1) / 2;
        let bounds = BvhBox::from_corners(*tree.min_point(node)?, *tree.max_point(node)?);
        tree.set_inner(node)?;
        Ok(BvhChildNodes::new(
            bounds,
            bounds,
            BvhPrimitiveRange::new(first, middle - 1),
            BvhPrimitiveRange::new(middle, last),
        ))
    }
}

#[test]
fn generic_queue_builder_currently_serializes_node_callbacks() {
    let mut set = BvhBoxSet::<BvhVec3d>::new(None);
    for index in 0..4 {
        let low = BvhVec3d::new(f64::from(index), 0.0, 0.0);
        let high = BvhVec3d::new(f64::from(index + 1), 1.0, 1.0);
        set.add(index, BvhBox::from_corners(low, high));
    }
    let bounds = BvhBox::from_corners(BvhVec3d::new(0.0, 0.0, 0.0), BvhVec3d::new(4.0, 1.0, 1.0));
    let builder = RecordingBuilder::new();
    let mut tree = BvhTree::default();

    queue_build(&builder, &mut set, &mut tree, &bounds).unwrap();

    assert_eq!(*builder.rendezvous.arrivals.lock().unwrap(), 2);
    assert_eq!(builder.maximum.load(Ordering::SeqCst), 1);
    assert_eq!(tree.length(), 7);
}
