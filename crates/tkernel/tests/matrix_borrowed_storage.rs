//! Native raw matrix storage access borrows column-major elements without copying.
use rustycad_tkernel::ncollection::{Mat3, Mat4};

macro_rules! storage_case {
    ($name:ident, $matrix:ident, $side:literal, $count:literal) => {
        #[test]
        fn $name() {
            let mut matrix =
                $matrix::from_data(std::array::from_fn::<_, $count, _>(|index| index as i64));
            let address = matrix.change_value(0, 0) as *mut i64 as *const i64;
            assert_eq!(matrix.as_slice().as_ptr(), address);
            assert_eq!(matrix.as_slice().len(), $count);
            assert_eq!(matrix.as_slice()[2 * $side + 1], 2 * $side + 1);
            let storage = matrix.as_mut_slice();
            assert_eq!(storage.as_ptr(), address);
            storage[2 * $side + 1] = 77;
            assert_eq!(matrix.get_value(1, 2), 77);
            matrix.set_value(1, 2, 99);
            assert_eq!(matrix.as_slice()[2 * $side + 1], 99);
        }
    };
}
storage_case!(mat3_borrows_column_major_storage, Mat3, 3, 9);
storage_case!(mat4_borrows_column_major_storage, Mat4, 4, 16);
