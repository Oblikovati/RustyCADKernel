//! Single and pair traversal logs, BVH_Distance and BVH_PairDistance (`bvh_traverse`).
use super::builders::query_builder;
use super::distances::{PairDistance, PointDistance};
use super::selectors::{BoxSelector, LogSelector, PairSelector};
use super::sort_build::{fill, nodes_out};
use super::{dispatch_vector, flag, scalar_out, vector_in};
use rustycad_tkmath::bvh::{
    BvhBox, BvhDistance, BvhDistanceTraverse, BvhObject, BvhPairDistance, BvhPairDistanceTraverse,
    BvhPairTraverse, BvhPrimitiveSetAccess, BvhScalar, BvhTraverse, BvhTree, BvhTriangulation,
    BvhVector,
};
use std::cell::RefCell;

/// The query thresholds of one request, narrowed to the scalar type.
struct Thresholds<T> {
    max_sq: T,
    accept_sq: T,
    pair_sq: T,
    stop: i32,
}

fn single_logs<V: BvhVector>(tree: &BvhTree<V>, point: V, limits: &Thresholds<V::Scalar>) {
    let mut single = LogSelector {
        point,
        max_sq: limits.max_sq,
        accept_sq: limits.accept_sq,
        stop_count: limits.stop,
        count: 0,
        log: RefCell::new(String::new()),
    };
    print!(
        "{} {}",
        single.select_tree(Some(tree)).unwrap(),
        single.log.borrow()
    );
    let mut query = BvhBox::from_point(point);
    query.add(&(point + V::splat(limits.accept_sq)));
    let mut boxes = BoxSelector {
        query,
        log: String::new(),
    };
    print!("{} {}", boxes.select_tree(Some(tree)).unwrap(), boxes.log);
}

fn pair_logs<V: BvhVector>(
    first: &BvhTree<V>,
    second: &BvhTree<V>,
    limits: &Thresholds<V::Scalar>,
) {
    let selector = |stop_count| PairSelector::<V> {
        pair_sq: limits.pair_sq,
        stop_count,
        count: 0,
        log: RefCell::new(String::new()),
    };
    let mut pairs = selector(limits.stop);
    let selected = pairs.select_trees(Some(first), Some(second)).unwrap();
    print!("{selected} {}", pairs.log.borrow());
    let mut own = selector(0);
    let selected = own.select_trees(Some(first), Some(first)).unwrap();
    print!("{selected} {}", own.log.borrow());
}

fn distance_logs<V: BvhVector>(
    first: &mut BvhTriangulation<V>,
    second: &mut BvhTriangulation<V>,
    point: V,
) {
    let mut state = BvhDistance::default();
    state.set_object(point);
    let mut distance = PointDistance {
        state,
        set: Some(first),
        log: String::new(),
    };
    scalar_out(distance.compute_distance().unwrap());
    print!("{} {}", flag(distance.state.is_done()), distance.log);
    let mut unset = PointDistance::<V> {
        state: BvhDistance::default(),
        set: None,
        log: String::new(),
    };
    scalar_out(unset.compute_distance().unwrap());
    print!("{} ", flag(unset.state.is_done()));
    let mut pair = PairDistance {
        state: BvhPairDistance::default(),
        first,
        second,
        log: String::new(),
    };
    scalar_out(pair.compute_distance().unwrap());
    println!("{} {}", flag(pair.state.is_done()), pair.log);
}

/// v: kind leaf count1 count2 stop point(4) maxsq acceptsq pairsq, then both vertex lists.
fn traverse_run<V: BvhVector>(v: &[f64]) {
    let [kind, leaf, count1, count2, stop] = std::array::from_fn(|i| v[i] as i32);
    let point: V = vector_in(&v[5..]);
    let limits = Thresholds {
        max_sq: V::Scalar::from_double(v[9]),
        accept_sq: V::Scalar::from_double(v[10]),
        pair_sq: V::Scalar::from_double(v[11]),
        stop,
    };
    let mut first = BvhTriangulation::<V>::new(Some(query_builder(kind, leaf, 32, 0)));
    let mut second = BvhTriangulation::<V>::new(Some(query_builder(kind, leaf, 32, 0)));
    let split = 12 + 3 * count1 as usize * V::DIMENSION;
    fill(&mut first, &v[12..], count1 as usize);
    fill(&mut second, &v[split..], count2 as usize);
    first.mark_dirty();
    second.mark_dirty();
    let first_tree = first.bvh().unwrap().clone();
    let second_tree = second.bvh().unwrap().clone();
    nodes_out(&first_tree);
    nodes_out(&second_tree);
    single_logs(&first_tree, point, &limits);
    pair_logs(&first_tree, &second_tree, &limits);
    distance_logs(&mut first, &mut second, point);
}
pub fn bvh_traverse_case(v: &[f64]) {
    dispatch_vector!(v[0], v[1], traverse_run, &v[2..]);
}
