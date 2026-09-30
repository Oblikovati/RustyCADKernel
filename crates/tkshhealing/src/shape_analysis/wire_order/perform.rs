// Rust translation of ShapeAnalysis_WireOrder::Perform (ModelingAlgorithms/TKShHealing/ShapeAnalysis/
// ShapeAnalysis_WireOrder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//
// pdn 11.03.99 S4135: the reordering does not depend on the tolerance; pdn 09.05.99 S4174:
// the order of the edges of a complete torus is kept.

//! `ShapeAnalysis_WireOrder::Perform`: chains the edges into loops by the nearest ends,
//! then merges the loops.
use super::{ModeType, ShapeAnalysisWireOrder};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_last, real_small};
use rustycad_tkmath::gp::{Point2, Point3};

/// The ends of every edge, 1-based as natively (index 0 unused).
struct EdgeEnds {
    begins_3d: Vec<Point3>,
    ends_3d: Vec<Point3>,
    begins_2d: Vec<Point2>,
    ends_2d: Vec<Point2>,
    both: bool,
}

/// The ends of the sequence under construction.
#[derive(Clone, Copy, Default)]
struct SeqEnds {
    first_3d: Point3,
    last_3d: Point3,
    first_2d: Point2,
    last_2d: Point2,
}

/// The best edge found for the next joint.
struct Best {
    joint: i32,
    min_3d: f64,
    edge: usize,
    found: bool,
    connected: bool,
}

/// The four squared distances between the sequence ends and one edge.
#[derive(Clone, Copy)]
struct JointDistances {
    tail_head: f64,
    tail_tail: f64,
    head_tail: f64,
    head_head: f64,
}

impl JointDistances {
    /// The distance of joint type `joint` (0 tail-head, 1 head-tail, 2 tail-tail, 3
    /// head-head).
    fn of_joint(&self, joint: i32) -> f64 {
        match joint {
            0 => self.tail_head,
            1 => self.head_tail,
            2 => self.tail_tail,
            _ => self.head_head,
        }
    }

    /// The bit mask of the joints closer than `tol2`.
    fn mask(&self, tol2: f64) -> i32 {
        let bit = |d: f64, b: i32| if d < tol2 { 1 << b } else { 0 };
        bit(self.tail_head, 0)
            | bit(self.tail_tail, 2)
            | bit(self.head_tail, 1)
            | bit(self.head_head, 3)
    }
}

const TOL2: f64 = precision::square_confusion();
const TOL_P2: f64 = precision::square_p_confusion();

impl EdgeEnds {
    fn of(order: &ShapeAnalysisWireOrder, nb: usize) -> Self {
        let both = order.mode == ModeType::ModeBoth;
        let p3 = |i: usize| Point3::from(order.xyz[i]);
        let p2 = |i: usize| Point2::from(order.xy[i]);
        let mut ends = EdgeEnds {
            begins_3d: vec![Point3::default(); nb + 1],
            ends_3d: vec![Point3::default(); nb + 1],
            begins_2d: vec![Point2::default(); nb + 1],
            ends_2d: vec![Point2::default(); nb + 1],
            both,
        };
        for i in 1..=nb {
            ends.begins_3d[i] = p3(2 * i - 2);
            ends.ends_3d[i] = p3(2 * i - 1);
            if both {
                ends.begins_2d[i] = p2(2 * i - 2);
                ends.ends_2d[i] = p2(2 * i - 1);
            }
        }
        ends
    }

    fn distances_3d(&self, seq: &SeqEnds, i: usize) -> JointDistances {
        JointDistances {
            tail_head: seq.last_3d.square_distance(&self.begins_3d[i]),
            tail_tail: seq.last_3d.square_distance(&self.ends_3d[i]),
            head_tail: seq.first_3d.square_distance(&self.ends_3d[i]),
            head_head: seq.first_3d.square_distance(&self.begins_3d[i]),
        }
    }

    fn distances_2d(&self, seq: &SeqEnds, i: usize) -> JointDistances {
        JointDistances {
            tail_head: seq.last_2d.square_distance(self.begins_2d[i]),
            tail_tail: seq.last_2d.square_distance(self.ends_2d[i]),
            head_tail: seq.first_2d.square_distance(self.ends_2d[i]),
            head_head: seq.first_2d.square_distance(self.begins_2d[i]),
        }
    }

    /// The sequence ends when edge `i` starts a new sequence.
    fn start(&self, i: usize) -> SeqEnds {
        let mut seq = SeqEnds {
            first_3d: self.begins_3d[i],
            last_3d: self.ends_3d[i],
            ..SeqEnds::default()
        };
        if self.both {
            seq.first_2d = self.begins_2d[i];
            seq.last_2d = self.ends_2d[i];
        }
        seq
    }

    /// The start of edge `idx` in its orientation (reversed when negative).
    fn oriented_begin(&self, idx: i32) -> Point3 {
        if idx > 0 {
            self.begins_3d[idx as usize]
        } else {
            self.ends_3d[(-idx) as usize]
        }
    }

    /// The end of edge `idx` in its orientation.
    fn oriented_end(&self, idx: i32) -> Point3 {
        if idx > 0 {
            self.ends_3d[idx as usize]
        } else {
            self.begins_3d[(-idx) as usize]
        }
    }
}

/// The joint type and squared distance of the 3D algorithm for one edge: the better of
/// the tail and head joints (with the rule 0 < 1 < 2 < 3 when both are within `TOL2`).
fn joint_3d(d: &JointDistances) -> (i32, f64) {
    let (tail_joint, to_tail) = if d.tail_head <= d.tail_tail {
        (0, d.tail_head)
    } else {
        (2, d.tail_tail)
    };
    let (head_joint, to_head) = if d.head_tail <= d.head_head {
        (1, d.head_tail)
    } else {
        (3, d.head_head)
    };
    if (to_tail - to_head).abs() < TOL2 {
        if tail_joint < head_joint {
            return (tail_joint, to_tail);
        }
        return (head_joint, to_head);
    }
    if to_tail <= to_head {
        (tail_joint, to_tail)
    } else {
        (head_joint, to_head)
    }
}

impl Best {
    fn new() -> Self {
        Best {
            joint: 3,
            min_3d: real_last(),
            edge: 0,
            found: false,
            connected: false,
        }
    }

    /// The miscible mode: a joint closed in both 3D and 2D wins over the 3D algorithm;
    /// returns true when the edge was handled by it (the loop continues).
    fn consider_both(&mut self, d3: &JointDistances, d2: &JointDistances, i: usize) -> bool {
        let full = d3.mask(TOL2) & d2.mask(TOL_P2);
        if full != 0 {
            let joint = (0..4).find(|j| full & (1 << j) != 0).unwrap_or(3);
            if !self.connected || joint < self.joint {
                self.found = true;
                self.connected = true;
                self.min_3d = d3.of_joint(joint);
                self.joint = joint;
                self.edge = i;
            }
        }
        self.connected
    }

    /// The 3D rule: a better distance, or a better joint type at an equal or confused
    /// distance.
    fn consider_3d(&mut self, joint: i32, cur_min: f64, i: usize) {
        if !(self.min_3d > TOL2 || joint < self.joint) {
            return;
        }
        if cur_min < self.min_3d
            || ((cur_min == self.min_3d || cur_min < TOL2) && joint < self.joint)
        {
            self.found = true;
            self.min_3d = cur_min;
            self.joint = joint;
            self.edge = i;
        }
    }
}

/// Builds the loops: each step joins the best remaining edge to the head or tail of the
/// current sequence, or closes it and starts a new one.
fn build_loops(ends: &EdgeEnds, nb: usize) -> Vec<Vec<i32>> {
    let mut used = vec![false; nb + 1];
    used[1] = true;
    let mut seq_ends = ends.start(1);
    let mut sequence = vec![1];
    let mut loops = Vec::new();
    loop {
        let best = find_best(ends, &seq_ends, &used, nb);
        if !best.found {
            break;
        }
        let close_dist = seq_ends.first_3d.square_distance(&seq_ends.last_3d);
        if best.min_3d <= real_small() || best.min_3d < close_dist {
            join(ends, &mut seq_ends, &mut sequence, &best);
        } else {
            loops.push(std::mem::take(&mut sequence));
            seq_ends = ends.start(best.edge);
            sequence.push(best.edge as i32);
        }
        used[best.edge] = true;
    }
    loops.push(sequence);
    loops
}

/// The best remaining edge for the current sequence ends.
fn find_best(ends: &EdgeEnds, seq: &SeqEnds, used: &[bool], nb: usize) -> Best {
    let mut best = Best::new();
    for i in (1..=nb).filter(|&i| !used[i]) {
        let d3 = ends.distances_3d(seq, i);
        let (joint, cur_min) = joint_3d(&d3);
        if ends.both && best.consider_both(&d3, &ends.distances_2d(seq, i), i) {
            continue;
        }
        best.consider_3d(joint, cur_min, i);
    }
    best
}

/// Inserts the best edge by its joint type and moves the sequence ends.
fn join(ends: &EdgeEnds, seq: &mut SeqEnds, sequence: &mut Vec<i32>, best: &Best) {
    let e = best.edge;
    let n = e as i32;
    match best.joint {
        0 => {
            sequence.push(n);
            seq.last_3d = ends.ends_3d[e];
            seq.last_2d = if ends.both {
                ends.ends_2d[e]
            } else {
                seq.last_2d
            };
        }
        1 => {
            sequence.insert(0, n);
            seq.first_3d = ends.begins_3d[e];
            seq.first_2d = if ends.both {
                ends.begins_2d[e]
            } else {
                seq.first_2d
            };
        }
        2 => {
            sequence.push(-n);
            seq.last_3d = ends.begins_3d[e];
            seq.last_2d = if ends.both {
                ends.begins_2d[e]
            } else {
                seq.last_2d
            };
        }
        _ => {
            sequence.insert(0, -n);
            seq.first_3d = ends.ends_3d[e];
            seq.first_2d = if ends.both {
                ends.ends_2d[e]
            } else {
                seq.first_2d
            };
        }
    }
}

/// The place where one loop joins the main loop: the loop, its first edge, the main loop
/// edge it follows and whether it keeps its direction.
struct LoopJoint {
    dist: f64,
    loop_num: usize,
    cur_it: usize,
    main_it: usize,
    direct: bool,
}

impl LoopJoint {
    fn none() -> Self {
        LoopJoint {
            dist: real_last(),
            loop_num: 0,
            cur_it: 0,
            main_it: 0,
            direct: false,
        }
    }

    /// Takes `other` when it is better by more than `TOL2`.
    fn improve(&mut self, other: LoopJoint) {
        if other.dist < self.dist && (self.dist - other.dist).abs() > TOL2 {
            *self = other;
        }
    }
}

/// The best gap of the main loop for the gap of `cur_loop` before its edge `cur_it`.
fn best_main_gap(ends: &EdgeEnds, main: &[i32], cur_loop: &[i32], cur_it: usize) -> LoopJoint {
    let len = cur_loop.len();
    let prev_it = if cur_it == 1 { len } else { cur_it - 1 };
    let cur_first = ends.oriented_begin(cur_loop[cur_it - 1]);
    let cur_last = ends.oriented_end(cur_loop[prev_it - 1]);
    let mut best = LoopJoint::none();
    let main_len = main.len();
    let mut it2 = 1;
    while it2 <= main_len && best.dist != 0.0 {
        let next_it2 = if it2 == main_len { 1 } else { it2 + 1 };
        let main_first = ends.oriented_end(main[it2 - 1]);
        let main_last = ends.oriented_begin(main[next_it2 - 1]);
        let direct_dist =
            cur_first.square_distance(&main_first) + cur_last.square_distance(&main_last);
        let mut reverse_dist =
            cur_first.square_distance(&main_last) + cur_last.square_distance(&main_first);
        let join_dist = if direct_dist < TOL2 || direct_dist < 2.0 * reverse_dist {
            reverse_dist = direct_dist;
            direct_dist
        } else {
            reverse_dist
        };
        best.improve(LoopJoint {
            dist: join_dist,
            loop_num: 0,
            cur_it,
            main_it: it2,
            direct: direct_dist <= reverse_dist,
        });
        it2 += 1;
    }
    best
}

/// The best joint of one loop into the main loop.
fn best_loop_joint(ends: &EdgeEnds, main: &[i32], cur_loop: &[i32], loop_num: usize) -> LoopJoint {
    let mut best = LoopJoint::none();
    for cur_it in 1..=cur_loop.len() {
        let mut candidate = best_main_gap(ends, main, cur_loop, cur_it);
        candidate.cur_it = cur_it;
        best.improve(candidate);
    }
    best.loop_num = loop_num;
    best
}

/// Merges every loop into the first one, nearest loop first.
fn merge_loops(ends: &EdgeEnds, mut loops: Vec<Vec<i32>>) -> Result<Vec<i32>, Failure> {
    let mut main = loops.remove(0);
    while !loops.is_empty() {
        let mut best = LoopJoint::none();
        for (index, cur_loop) in loops.iter().enumerate() {
            best.improve(best_loop_joint(ends, &main, cur_loop, index + 1));
        }
        if best.loop_num == 0 {
            // Natively `aLoops.Value(0)` raises when no gap is finite.
            return Err(Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "ShapeAnalysis_WireOrder::Perform: loop 0 of {}; no finite joint distance",
                    loops.len()
                ),
            ));
        }
        let joined = loops.remove(best.loop_num - 1);
        let factor = if best.direct { 1 } else { -1 };
        let len = joined.len();
        for i in 0..len {
            let idx = if best.cur_it + i > len {
                best.cur_it + i - len
            } else {
                best.cur_it + i
            };
            main.insert(best.main_it + i, joined[idx - 1] * factor);
        }
    }
    Ok(main)
}

/// The status of the new order: 0 unchanged, 1 reordered, -1 reversed, 3 only shifted.
fn order_status(main: &[i32]) -> i32 {
    let mut status = 0;
    for (i, &value) in main.iter().enumerate() {
        if i as i32 + 1 != value && status >= 0 {
            status = if value > 0 { 1 } else { -1 };
        }
    }
    if status == 0 {
        return 0;
    }
    if is_shift(main) { 3 } else { status }
}

/// The edges are only shifted forward or backward (cyclically), not reordered.
fn is_shift(main: &[i32]) -> bool {
    let len = main.len() as i32;
    let (mut forward, mut reverse) = (true, true);
    let pairs = main
        .windows(2)
        .map(|w| (w[0], w[1]))
        .chain(std::iter::once((main[main.len() - 1], main[0])));
    for (first, second) in pairs {
        if second - first != 1 && (first != len || second != 1) {
            forward = false;
        }
        if first - second != 1 && (second != len || first != 1) {
            reverse = false;
        }
    }
    forward || reverse
}

impl ShapeAnalysisWireOrder {
    /// `Perform(closed)`: computes the order (`closed` is not used natively). With no
    /// edge, nothing is done and the status is 0. Each step joins the remaining edge
    /// whose end is nearest to an end of the current sequence (in the miscible mode, an
    /// edge joined in both 3D and 2D first), or starts a new loop when closing is better;
    /// the loops are then merged at their best gaps, or kept one after another in the
    /// keep-loops mode; `order.perform(true)?`.
    pub fn perform(&mut self, _closed: bool) -> Result<(), Failure> {
        self.stat = 0;
        let nb = self.nb_edges() as usize;
        if nb == 0 {
            return Ok(());
        }
        self.ord = Some(vec![0; nb]);
        let ends = EdgeEnds::of(self, nb);
        let loops = build_loops(&ends, nb);
        let main = if self.keep_loops {
            loops.concat()
        } else {
            merge_loops(&ends, loops)?
        };
        self.ord = Some(main.clone());
        self.stat = order_status(&main);
        Ok(())
    }
}
