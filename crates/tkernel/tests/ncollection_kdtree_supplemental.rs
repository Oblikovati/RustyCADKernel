//! Exact native ordering and behavior outside the legacy happy-path cloud distributions.
use rustycad_tkernel::ncollection::{Array1, KDTree};
#[test]
fn gcc_equal_coordinate_permutation_and_heap_ties_are_preserved() {
    let mut tree = KDTree::<[f64; 2], 2, true>::default();
    tree.build_radii(&[[0.0; 2]; 4], &[1.0; 4]);
    let mut distance = 99.0;
    assert_eq!(tree.nearest_point(&[0.0; 2], Some(&mut distance)), 3);
    assert_eq!(distance, 0.0);
    assert_eq!(tree.nearest_weighted(&[0.0; 2], None), 3);
    assert_eq!(
        tree.nearest_points(&[0.0; 2], 0.0, &mut distance),
        [3, 4, 1, 2]
    );
    assert_eq!(tree.range_search(&[0.0; 2], 1.0), [3, 4, 1, 2]);
    assert_eq!(tree.box_search(&[-1.0; 2], &[1.0; 2]), [3, 4, 1, 2]);
    assert_eq!(tree.containing_search(&[0.0; 2]), [3, 4, 1, 2]);
    let mut visited = Vec::new();
    tree.for_each_in_range(&[0.0; 2], 1.0, |index| visited.push(index));
    assert_eq!(visited, [3, 4, 1, 2]);
    let mut indices = Array1::default();
    let mut distances = Array1::default();
    assert_eq!(
        tree.k_nearest_points(&[0.0; 2], 3, &mut indices, &mut distances),
        3
    );
    assert_eq!(indices.as_slice(), [4, 1, 3]);
    assert_eq!(distances.as_slice(), [0.0; 3]);
}
#[test]
fn empty_queries_reset_outputs_and_reject_invalid_indices() {
    let tree = KDTree::<[f64; 2], 2, true>::default();
    let mut distance = 99.0;
    assert_eq!(tree.nearest_point(&[0.0; 2], Some(&mut distance)), 0);
    assert_eq!(distance, 0.0);
    distance = 99.0;
    assert_eq!(tree.nearest_weighted(&[0.0; 2], Some(&mut distance)), 0);
    assert_eq!(distance, 0.0);
    assert!(tree.point(0).is_err());
    assert!(tree.radius(1).is_err());
    let mut indices = Array1::with_bounds(5, 8).unwrap();
    let mut distances = Array1::with_bounds(2, 9).unwrap();
    assert_eq!(
        tree.k_nearest_points(&[0.0; 2], 2, &mut indices, &mut distances),
        0
    );
    assert!(indices.is_empty());
    assert!(distances.is_empty());
    tree.for_each_in_range(&[0.0; 2], 10.0, |_| panic!("empty tree callback"));
}
#[test]
fn overflowing_squared_distances_still_fill_k_nearest_heap() {
    let points: Vec<_> = (0..65).map(|i| [f64::from(i) * 1e200, 0.0]).collect();
    let mut tree = KDTree::<[f64; 2], 2>::default();
    tree.build(&points);
    let mut indices = Array1::default();
    let mut distances = Array1::default();
    assert_eq!(
        tree.k_nearest_points(&[-1e200; 2], 65, &mut indices, &mut distances),
        65
    );
    assert!(distances.as_slice().iter().all(|value| value.is_infinite()));
    let mut squared = 0.0;
    assert_eq!(tree.nearest_point(&[-1e200; 2], Some(&mut squared)), 0);
    assert_eq!(squared, f64::MAX);
}
