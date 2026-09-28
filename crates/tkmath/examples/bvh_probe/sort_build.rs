//! Quick/radix sorting and binned builds of triangulations (`bvh_sort`, `bvh_build`).
use super::{box_out, dispatch_vector, scalar_out, vector_in, vector_out};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBox, BvhBuilder, BvhObject, BvhPrimitiveSetAccess, BvhQuickSorter,
    BvhRadixSorter, BvhSet, BvhSorter, BvhTree, BvhTriangulation, BvhVec4i, BvhVector, array,
};
use std::sync::Arc;

/// `bvh_fill`: 3*count vertices of N coordinates, then triangles (3i, 3i+1, 3i+2).
fn fill<V: BvhVector>(set: &mut BvhTriangulation<V>, coordinates: &[f64], count: usize) {
    for vertex in coordinates.chunks(V::DIMENSION).take(3 * count) {
        array::append(&mut set.vertices, vector_in(vertex));
    }
    for triangle in 0..count as i32 {
        let record = BvhVec4i::new(3 * triangle, 3 * triangle + 1, 3 * triangle + 2, 0);
        array::append(&mut set.elements, record);
    }
}
fn order_out<V: BvhVector>(set: &BvhTriangulation<V>) {
    for record in set.elements.iter() {
        print!("{} ", record.x / 3);
    }
}
fn sort_range<V: BvhVector>(
    sorter: &mut dyn BvhSorter<V>,
    set: &mut BvhTriangulation<V>,
    start: i32,
    last: i32,
) {
    if last < 0 {
        sorter.perform(set).unwrap();
    } else {
        sorter.perform_range(set, start, last).unwrap();
    }
}

/// v: axis start last boxmode parallel count, then the vertex coordinates.
fn sort_run<V: BvhVector>(v: &[f64]) {
    let [axis, start, last, mode, parallel, count] = std::array::from_fn(|i| v[i] as i32);
    let mut quick = BvhTriangulation::<V>::default();
    fill(&mut quick, &v[6..], count as usize);
    let mut radix = BvhTriangulation::<V>::default();
    radix.vertices = quick.vertices.clone();
    radix.elements = quick.elements.clone();
    sort_range(&mut BvhQuickSorter::new(axis), &mut quick, start, last);
    order_out(&quick);
    let mut bounding = BvhBox::default();
    if mode == 0 {
        radix.mark_dirty();
        bounding = radix.bounding_box().unwrap();
    }
    box_out(&bounding);
    let mut sorter = BvhRadixSorter::new(bounding);
    sorter.set_parallel(parallel != 0);
    sort_range(&mut sorter, &mut radix, start, last);
    for (code, primitive) in sorter.encoded_links().unwrap().iter() {
        print!("{code} {primitive} ");
    }
    order_out(&radix);
    println!();
}
pub fn bvh_sort_case(v: &[f64]) {
    dispatch_vector!(v[0], v[1], sort_run, &v[2..]);
}

fn nodes_out<V: BvhVector, A>(tree: &BvhTree<V, A>) {
    print!("{} {} ", tree.length(), tree.depth());
    for node in 0..tree.length() {
        let record = array::value(tree.node_info_buffer(), node).unwrap();
        print!("{} {} {} {} ", record.x, record.y, record.z, record.w);
        vector_out(tree.min_point(node).unwrap());
        vector_out(tree.max_point(node).unwrap());
    }
}
fn binned<V: BvhVector, const BINS: usize>(
    leaf: i32,
    depth: i32,
    main: bool,
) -> Arc<dyn BvhBuilder<V>> {
    Arc::new(BvhBinnedBuilder::<V, BINS>::new(leaf, depth, main, 1))
}
fn builder<V: BvhVector>(bins: i32, leaf: i32, depth: i32, main: bool) -> Arc<dyn BvhBuilder<V>> {
    match bins {
        2 => binned::<V, 2>(leaf, depth, main),
        4 => binned::<V, 4>(leaf, depth, main),
        16 => binned::<V, 16>(leaf, depth, main),
        32 => binned::<V, 32>(leaf, depth, main),
        _ => binned::<V, 48>(leaf, depth, main),
    }
}

/// v: bins (0 = default primitive set) leaf depth mainaxis count, then vertices.
fn build_run<V: BvhVector>(v: &[f64]) {
    let [bins, leaf, depth, main, count] = std::array::from_fn(|i| v[i] as i32);
    let mut set = BvhTriangulation::<V>::default();
    if bins != 0 {
        set.set_builder(Some(builder(bins, leaf, depth, main != 0)));
    }
    fill(&mut set, &v[5..], count as usize);
    box_out(&set.bounding_box().unwrap());
    set.mark_dirty();
    let tree = set.bvh().unwrap().clone();
    nodes_out(&tree);
    order_out(&set);
    if tree.length() > 0 {
        scalar_out(tree.estimate_sah().unwrap());
    }
    let quad = tree.collapse_to_quad_tree().unwrap();
    nodes_out(&quad);
    if quad.length() > 0 {
        print!("{} ", quad.child::<3>(0).unwrap());
    }
    box_out(&set.bounding_box().unwrap());
    let mut dump = DumpStream::default();
    tree.dump_json(&mut dump, -1).unwrap();
    println!("{}", dump.text());
}
pub fn bvh_build_case(v: &[f64]) {
    dispatch_vector!(v[0], v[1], build_run, &v[2..]);
}
