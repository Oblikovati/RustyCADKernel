//! Native matrix Map returns a reference to the full matrix, preserving storage identity.
use rustycad_tkernel::ncollection::{Mat3, Mat4, Vec3};
use std::{cell::Cell, rc::Rc};

#[test]
fn mapped_mat3_exposes_full_arithmetic_on_the_original_storage() {
    let mut elements = [2_i32, 0, 0, 1, 11, 0, 0, 0, 1];
    let origin = elements.as_mut_ptr();
    assert_eq!(Mat3::map_ref(&elements).determinant(), 22);
    let matrix = Mat3::map_ref_mut(&mut elements);
    assert_eq!(matrix.data_ptr(), origin.cast_const());
    assert_eq!(matrix.data_mut_ptr(), origin);
    matrix.transpose();
    assert_eq!(matrix.get_value(1, 0), 1);
    matrix.set_diagonal(Vec3::new(3, 4, 5));
    assert_eq!(elements, [3, 1, 0, 0, 4, 0, 0, 0, 5]);
}

#[test]
fn mapped_mat4_exposes_transform_and_inverse_without_a_copy() {
    let mut elements = Mat4::<f64>::identity().data();
    let origin = elements.as_mut_ptr();
    let matrix = Mat4::map_ref_mut(&mut elements);
    assert_eq!(matrix.data_mut_ptr(), origin);
    matrix.translate(Vec3::new(2.0, 3.0, 4.0));
    let inverse = matrix.inverted().unwrap();
    assert_eq!(inverse.get_column(3).xyz(), Vec3::new(-2.0, -3.0, -4.0));
    assert!(matrix.multiplied(&inverse).is_identity());
    assert_eq!(Mat4::map_ref(&elements).data_ptr(), origin.cast_const());
    assert_eq!(&elements[12..15], &[2.0, 3.0, 4.0]);
    assert_eq!(Mat4::map_ref(&elements).get_mat3().determinant(), 1.0);
}

#[repr(align(128))]
struct AlignedElement;

#[test]
fn transparent_matrices_keep_scalar_alignment_and_zero_sized_layout() {
    assert_eq!(size_of::<Mat3<f64>>(), size_of::<[f64; 9]>());
    assert_eq!(size_of::<Mat4<f64>>(), size_of::<[f64; 16]>());
    assert_eq!(align_of::<Mat3<AlignedElement>>(), 128);
    assert_eq!(align_of::<Mat4<AlignedElement>>(), 128);
    let mut zero_sized = [(); 9];
    let matrix = Mat3::map_ref_mut(&mut zero_sized);
    assert_eq!(matrix.as_slice().len(), 9);
    assert!(!matrix.data_mut_ptr().is_null());
}

struct DropCount(Rc<Cell<usize>>);
impl Drop for DropCount {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn mapping_noncopy_elements_neither_copies_nor_drops_borrowed_storage() {
    let count = Rc::new(Cell::new(0));
    let mut elements: [DropCount; 16] = std::array::from_fn(|_| DropCount(count.clone()));
    let matrix = Mat4::map_ref_mut(&mut elements);
    *matrix.change_value(1, 2) = DropCount(count.clone());
    assert_eq!(count.get(), 1);
    assert_eq!(Mat4::map_ref(&elements).as_slice().len(), 16);
    drop(elements);
    assert_eq!(count.get(), 17);
}
