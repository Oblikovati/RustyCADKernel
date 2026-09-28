//! Additional safety and storage-contract tests, separate from legacy identities.
use rustycad_tkernel::ncollection::{Array1, Array2, ArrayError, LocalArray};
use rustycad_tkernel::standard::ArrayStreamBuffer;
use std::io::{BufRead, Read, Seek, SeekFrom};

#[test]
fn array_origins_and_checked_errors() {
    let mut buffer = [1, 2];
    let mut array = Array1::from_slice(&mut buffer);
    array.update_lower_bound(-2);
    assert_eq!(array[-2], 1);
    assert_eq!(array.upper(), -1);
    assert!(array.value(0).is_err());
    assert!(array.change_at(2).is_err());
    assert!(Array1::<i32>::with_bounds(2, 0).is_err());
    assert!(array.resize_bounds(2, 0, true).is_err());
    assert_eq!(
        Array1::<i32>::default().first().err().unwrap().to_string(),
        "index 0: expected [0, -1]"
    );
}
#[test]
fn rectangles_reject_wrong_shapes_and_indices() {
    // Native RangeSize(2, 1) == 0: an empty dimension; only a negative extent is refused.
    let empty = Array2::<i32>::with_bounds(2, 1, 0, 1).unwrap();
    assert_eq!(
        (empty.nb_rows(), empty.nb_columns(), empty.len()),
        (0, 2, 0)
    );
    assert!(Array2::<i32>::with_bounds(3, 1, 0, 1).is_err());
    assert!(Array2::<i32>::from_buffer(&mut [0; 3], 2, 2).is_err());
    let mut array = Array2::<i32>::with_size(2, 3);
    assert!(array.value(-1, 0).is_err());
    assert!(array.at(0, 3).is_err());
    assert!(array.change_at(2, 0).is_err());
    let mismatch = array.copy_values(&Array2::with_size(3, 2)).unwrap_err();
    assert!(matches!(mismatch, ArrayError::ShapeMismatch { .. }));
    assert_eq!(mismatch.to_string(), "shape (3, 2): expected (2, 3)");
    array.change_array1().init(9);
    assert_eq!(array[(0, 0)], 9);
    array.resize(0, 3, true).unwrap();
    assert_eq!(
        (array.nb_rows(), array.nb_columns(), array.len()),
        (0, 3, 0)
    );
}
#[test]
fn local_deallocation_releases_elements() {
    let mut array = LocalArray::<i32, 2>::new(10);
    array.deallocate();
    assert!(array.is_empty());
    array.allocate(1);
    assert_eq!(array.len(), 1);
}
#[test]
fn stream_putback_peek_and_end_offset_conventions() {
    let mut stream = ArrayStreamBuffer::new(b"ABC");
    assert_eq!(stream.peek(), Some(b'A'));
    assert_eq!(stream.put_back(None), None);
    let mut byte = [0];
    stream.read_exact(&mut byte).unwrap();
    assert_eq!(stream.put_back(Some(b'Z')), None);
    assert_eq!(stream.put_back(Some(b'A')), Some(b'A'));
    stream.consume(2);
    assert_eq!(stream.put_back(None), Some(b'B'));
    assert_eq!(stream.seek_occt(SeekFrom::End(1)).unwrap(), 2);
    assert_eq!(stream.seek(SeekFrom::End(-1)).unwrap(), 2);
    assert_eq!(stream.seek_occt(SeekFrom::End(10)).unwrap(), 0);
    assert!(stream.seek(SeekFrom::Current(-1)).is_err());
    assert_eq!(stream.seek_occt(SeekFrom::Start(1)).unwrap(), 1);
}
