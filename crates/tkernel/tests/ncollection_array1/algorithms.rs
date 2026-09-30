// Same 100 std::mt19937(1)/uniform_int_distribution(0,RAND_MAX) values from
// OCCT NCollection_Array1_Test.cxx, emitted with GCC 13.3/libstdc++ on Linux.

fn array_algorithm_inputs() -> (Array1<'static, i32>, Vec<i32>) {
    let vector: Vec<i32> = include_str!("../fixtures/array1_mt19937_seed1.txt")
        .lines().map(|line| line.parse().unwrap()).collect();
    assert_eq!(100, vector.len());
    let mut array = Array1::with_bounds(1, 100).unwrap();
    for (index, value) in vector.iter().enumerate() { array[index as i32 + 1] = *value; }
    (array, vector)
}

#[test]
fn NCollection_Array1Test__STLAlgorithmCompatibility_MinMax() {
    let (array, vector) = array_algorithm_inputs();
    assert_eq!(array.iter().min().unwrap(), vector.iter().min().unwrap());
    assert_eq!(array.iter().max().unwrap(), vector.iter().max().unwrap());
}

#[test]
fn NCollection_Array1Test__STLAlgorithmCompatibility_Replace() {
    let (mut array, mut vector) = array_algorithm_inputs();
    let target = *vector.last().unwrap();
    for value in &mut array { if *value == target { *value = -1; } }
    for value in &mut vector { if *value == target { *value = -1; } }
    assert!(array.iter().eq(vector.iter()));
}

#[test]
fn NCollection_Array1Test__STLAlgorithmCompatibility_Sort() {
    let (mut array, mut vector) = array_algorithm_inputs();
    array.as_mut_slice().sort();
    vector.sort();
    assert!(array.iter().eq(vector.iter()));
}

#[derive(Default)]
struct Array1MultiArgType { a: i32, b: f64 }

#[test]
#[allow(clippy::approx_constant)] // Preserve the upstream literal exactly.
fn NCollection_Array1Test__EmplaceValue() {
    let mut array = Array1::<Array1MultiArgType>::with_bounds(1, 5).unwrap();
    let first = array.emplace_value(1, || Array1MultiArgType { a: 42, b: 3.14 }).unwrap();
    assert_eq!(42, first.a);
    assert!((3.14 - first.b).abs() <= 1e-10);
    let third = array.emplace_value(3, || Array1MultiArgType { a: 100, b: 2.71 }).unwrap();
    assert_eq!(100, third.a);
    assert!((2.71 - third.b).abs() <= 1e-10);
    assert_eq!(42, array[1].a);
    assert_eq!(100, array[3].a);
    for index in [2, 4, 5] { assert_eq!(0, array[index].a); }
}

#[test]
fn NCollection_Array1Test__EmplaceValue_ReplacesExisting() {
    let mut array = Array1::<Array1MultiArgType>::with_bounds(1, 3).unwrap();
    array.emplace_value(1, || Array1MultiArgType { a: 10, b: 1.0 }).unwrap();
    array.emplace_value(2, || Array1MultiArgType { a: 20, b: 2.0 }).unwrap();
    array.emplace_value(3, || Array1MultiArgType { a: 30, b: 3.0 }).unwrap();
    let replaced = array.emplace_value(2, || Array1MultiArgType { a: 200, b: 20.0 }).unwrap();
    assert_eq!(200, replaced.a);
    assert!((20.0 - replaced.b).abs() <= 1e-10);
    assert_eq!(10, array[1].a);
    assert_eq!(200, array[2].a);
    assert_eq!(30, array[3].a);
}
