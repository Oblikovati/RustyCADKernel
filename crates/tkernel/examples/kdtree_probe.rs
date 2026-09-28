//! Exact result ordering and floating-point bits for the native KDTree oracle.
use rustycad_tkernel::ncollection::{Array1, KDTree};
use std::{
    io::{self, BufRead},
    str::{FromStr, SplitWhitespace},
};
fn take<T: FromStr>(input: &mut SplitWhitespace<'_>) -> T
where
    T::Err: std::fmt::Debug,
{
    input.next().unwrap().parse().unwrap()
}
fn point<const D: usize>(input: &mut SplitWhitespace<'_>) -> [f64; D] {
    std::array::from_fn(|_| take(input))
}
fn print_range(values: &[usize]) {
    print!(" {}", values.len());
    for value in values {
        print!(" {value}");
    }
}
fn probe<const D: usize>(input: &mut SplitWhitespace<'_>) {
    let count: usize = take(input);
    let mut points = Vec::new();
    let mut radii = Vec::new();
    for _ in 0..count {
        points.push(point::<D>(input));
        radii.push(take(input));
    }
    let query = point(input);
    let min = point(input);
    let max = point(input);
    let radius: f64 = take(input);
    let tolerance: f64 = take(input);
    let k: usize = take(input);
    let mut tree = KDTree::<[f64; D], D, true>::default();
    tree.build_radii(&points, &radii);
    print_queries(&tree, query, min, max, radius, tolerance, k);
}
fn print_queries<const D: usize>(
    tree: &KDTree<[f64; D], D, true>,
    query: [f64; D],
    min: [f64; D],
    max: [f64; D],
    radius: f64,
    tolerance: f64,
    k: usize,
) {
    let mut distance = 0.0;
    let mut gap = 0.0;
    let nearest = tree.nearest_point(&query, Some(&mut distance));
    let weighted = tree.nearest_weighted(&query, Some(&mut gap));
    print!(
        "{nearest} {} {weighted} {}",
        distance.to_bits(),
        gap.to_bits()
    );
    print_range(&tree.nearest_points(&query, tolerance, &mut distance));
    print!(" {}", distance.to_bits());
    print_range(&tree.range_search(&query, radius));
    let mut visited = Vec::new();
    tree.for_each_in_range(&query, radius, |index| visited.push(index));
    print_range(&visited);
    print_range(&tree.box_search(&min, &max));
    print_range(&tree.containing_search(&query));
    let mut indices = Array1::default();
    let mut distances = Array1::default();
    let count = tree.k_nearest_points(&query, k, &mut indices, &mut distances);
    print!(" {count}");
    for (index, distance) in indices.as_slice().iter().zip(distances.as_slice()) {
        print!(" {index} {}", distance.to_bits());
    }
    println!();
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut input = line.split_whitespace();
        if take::<usize>(&mut input) == 2 {
            probe::<2>(&mut input);
        } else {
            probe::<3>(&mut input);
        }
    }
}
