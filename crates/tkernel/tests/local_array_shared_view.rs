//! Supplemental LocalArray const-view copy-on-write behavior.
use rustycad_tkernel::ncollection::LocalArray;

#[test]
fn local_array_const_array1_view_detaches_on_write() {
    let mut source = LocalArray::<i32>::new(2);
    source[0] = 12;
    source[1] = 20;
    let mut view = source.to_array1_view();

    view[1] = 77;

    assert_eq!(source[1], 20);
    assert_eq!(view[1], 77);
}
