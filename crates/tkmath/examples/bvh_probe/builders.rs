//! Linear, spatial-median, sweep-plane and two-bin builds of meshes and box
//! sets (`bvh_builders`), comparing whole tree layouts.
use super::sort_build::{fill, nodes_out, order_out};
use super::{box_out, dispatch_vector, scalar_out};
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBox, BvhBoxSet, BvhBuilder, BvhBuilderHandle, BvhIndexedBoxSet,
    BvhLinearBuilder, BvhObject, BvhPrimitiveSetAccess, BvhScalar, BvhSet, BvhSpatialMedianBuilder,
    BvhSweepPlaneBuilder, BvhTree, BvhTriangulation, BvhVector, array,
};
use std::sync::Arc;

/// `bvh_query_builder`: 0 linear (flag = parallel), 1 spatial median (flag =
/// main axis), 2 sweep plane, 3 two-bin binned (flag = main axis), else 48 bins.
pub(super) fn query_builder<V: BvhVector>(
    kind: i32,
    leaf: i32,
    depth: i32,
    flag: i32,
) -> Arc<dyn BvhBuilder<V>> {
    match kind {
        0 => {
            let linear = BvhLinearBuilder::<V>::new(leaf, depth);
            linear.builder_transient().set_parallel(flag != 0);
            Arc::new(linear)
        }
        1 => Arc::new(BvhSpatialMedianBuilder::<V>::new(leaf, depth, flag != 0)),
        2 => Arc::new(BvhSweepPlaneBuilder::<V>::new(leaf, depth, 1)),
        3 => Arc::new(BvhBinnedBuilder::<V, 2>::new(leaf, depth, flag != 0, 1)),
        _ => Arc::new(BvhBinnedBuilder::<V, 48>::new(leaf, depth, flag != 0, 1)),
    }
}

/// The primitive sets the builder cases run on, with their element order output.
trait ProbeSet<V: BvhVector>: BvhPrimitiveSetAccess<V> + BvhObject<V> + Sized {
    fn empty(builder: BvhBuilderHandle<V>) -> Self;
    fn order_print(&self);
}
impl<V: BvhVector> ProbeSet<V> for BvhTriangulation<V> {
    fn empty(builder: BvhBuilderHandle<V>) -> Self {
        Self::new(builder)
    }
    fn order_print(&self) {
        order_out(self);
    }
}
impl<V: BvhVector> ProbeSet<V> for BvhBoxSet<V> {
    fn empty(builder: BvhBuilderHandle<V>) -> Self {
        Self::new(builder)
    }
    fn order_print(&self) {
        for index in 0..self.size() {
            print!("{} ", self.element(index).unwrap());
        }
    }
}
impl<V: BvhVector> ProbeSet<V> for BvhIndexedBoxSet<V> {
    fn empty(builder: BvhBuilderHandle<V>) -> Self {
        Self::new(builder)
    }
    fn order_print(&self) {
        for index in 0..self.size() {
            print!("{} ", self.element(index).unwrap());
        }
    }
}

/// `bvh_query_box`: set box, invalid box, first-vertex point box, widened box.
fn query_box<V: BvhVector, S: ProbeSet<V>>(set: &mut S, mode: i32, first: Option<V>) -> BvhBox<V> {
    match mode {
        1 => return BvhBox::default(),
        2 => return first.map(BvhBox::from_point).unwrap_or_default(),
        _ => {}
    }
    set.mark_dirty();
    let mut bounding = set.bounding_box().unwrap();
    if mode == 3 && bounding.is_valid() {
        for axis in 0..V::DIMENSION {
            let lowered = bounding.corner_min()[axis] - V::Scalar::from_int(1);
            bounding.corner_min_mut()[axis] = lowered;
            bounding.corner_max_mut()[axis] += V::Scalar::from_int(2);
        }
    }
    bounding
}

/// Explicit build, empty rebuild into the same tree, then the set's own BVH().
fn builders_on<V: BvhVector, S: ProbeSet<V>>(
    set: &mut S,
    builder: &Arc<dyn BvhBuilder<V>>,
    mode: i32,
    first: Option<V>,
) {
    let bounding = query_box(set, mode, first);
    box_out(&bounding);
    let mut tree = BvhTree::default();
    builder.build(set, &mut tree, &bounding).unwrap();
    nodes_out(&tree);
    set.order_print();
    if tree.length() > 0 {
        scalar_out(tree.estimate_sah().unwrap());
    }
    let mut empty = S::empty(Some(builder.clone()));
    builder.build(&mut empty, &mut tree, &bounding).unwrap();
    print!("{} {} ", tree.length(), tree.depth());
    set.mark_dirty();
    nodes_out(set.bvh().unwrap());
    set.order_print();
    println!();
}

/// v: kind leaf depth flag setkind boxmode count, then the vertex coordinates.
fn builders_run<V: BvhVector>(v: &[f64]) {
    let [kind, leaf, depth, flag, set_kind, mode, count] = std::array::from_fn(|i| v[i] as i32);
    let builder = query_builder::<V>(kind, leaf, depth, flag);
    let mut mesh = BvhTriangulation::<V>::new(Some(builder.clone()));
    fill(&mut mesh, &v[7..], count as usize);
    let first = array::value(&mesh.vertices, 0).ok().copied();
    match set_kind {
        0 => builders_on(&mut mesh, &builder, mode, first),
        1 => {
            let mut boxes = BvhBoxSet::<V>::new(Some(builder.clone()));
            for index in 0..count {
                boxes.add(index, mesh.element_box(index).unwrap());
            }
            builders_on(&mut boxes, &builder, mode, first);
        }
        _ => {
            let mut boxes = BvhIndexedBoxSet::<V>::new(Some(builder.clone()));
            for index in 0..count {
                boxes.add(index, mesh.element_box(index).unwrap());
            }
            builders_on(&mut boxes, &builder, mode, first);
        }
    }
}
pub fn bvh_builders_case(v: &[f64]) {
    dispatch_vector!(v[0], v[1], builders_run, &v[2..]);
}
