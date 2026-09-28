//! Behaviour of the array-family members completed for the NCollection API ledger:
//! bound arithmetic, the Array2 layout helpers, the handle-shared HArray1/HArray2, the
//! const array views and SparseArray's block-table capacity.
use rustycad_tkernel::ncollection::{
    AliasedArray, Array1, Array2, HArray1, HArray2, LinearVector, LocalArray, SparseArray,
};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardTransient};

#[test]
fn array1_upper_bound_update_length_and_range_size() {
    let mut array = Array1::<i32>::with_bounds(1, 5).unwrap();
    array.update_upper_bound(10);
    assert_eq!((array.lower(), array.upper(), array.length()), (6, 10, 5));
    assert_eq!(Array1::<i32>::range_size(1, 5), 5);
    assert_eq!(Array1::<i32>::range_size(3, 2), 0);
    // A negative int extent sign-extends, as `static_cast<size_t>(int)` does.
    assert_eq!(Array1::<i32>::range_size(5, 0), usize::MAX - 3);
}

#[test]
fn array2_layout_helpers_match_the_native_formulas() {
    assert_eq!(Array2::<i32>::array_size(2, 3), 6);
    assert_eq!(Array2::<i32>::range_size(-2, 2), 5);
    assert_eq!(Array2::<i32>::upper_for_size(1, 3), 3);
    assert_eq!(Array2::<i32>::begin_position(1, 2, 1, 3), 4);
    assert_eq!(Array2::<i32>::last_position(1, 2, 1, 3), 9);
    let grid = Array2::<i32>::with_bounds(1, 2, 1, 3).unwrap();
    assert_eq!(
        grid.array1().lower(),
        Array2::<i32>::begin_position(1, 2, 1, 3)
    );
    assert_eq!(
        grid.array1().upper(),
        Array2::<i32>::last_position(1, 2, 1, 3)
    );
}

#[test]
fn array2_counts_ends_and_flat_offsets() {
    let mut grid = Array2::<i32>::with_size(2, 3);
    for (offset, value) in grid.iter_mut().enumerate() {
        *value = offset as i32;
    }
    assert_eq!((grid.length(), grid.row_size(), grid.col_size()), (6, 2, 3));
    assert_eq!((grid.row_length(), grid.col_length()), (3, 2));
    assert_eq!((*grid.first().unwrap(), *grid.last().unwrap()), (0, 5));
    *grid.change_first().unwrap() = 10;
    *grid.change_last().unwrap() = 15;
    *grid.change_at_offset(4).unwrap() = 14;
    assert_eq!(*grid.at_offset(4).unwrap(), 14);
    assert_eq!((grid[(0, 0)], grid[(1, 2)]), (10, 15));
    assert!(grid.at_offset(6).is_err());
}

#[test]
fn array2_constructor_accepts_an_empty_dimension() {
    let empty = Array2::<i32>::with_bounds(1, 0, 1, 3).unwrap();
    assert_eq!(
        (empty.nb_rows(), empty.nb_columns(), empty.is_empty()),
        (0, 3, true)
    );
    assert_eq!((empty.upper_row(), empty.upper_col()), (0, 3));
    assert!(Array2::<i32>::with_bounds(1, -1, 1, 3).is_err());
    assert!(Array2::from_shared(&[0.0; 0], 1, 0, 1, 0).is_ok());
}

#[test]
fn harray1_edits_are_seen_through_every_handle() {
    let shared = Handle::new(HArray1::with_value(1, 3, 0.5).unwrap());
    let other = shared.clone();
    shared
        .get()
        .unwrap()
        .change_array1()
        .set_value(2, 1.5)
        .unwrap();
    let values = other
        .get()
        .unwrap()
        .array1()
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(values, [0.5, 1.5, 0.5]);
    assert_eq!(HArray1::<i32>::default().array1().len(), 0);
    assert!(HArray1::<i32>::with_bounds(3, 1).is_err());
}

#[test]
fn harray1_buffer_constructor_views_or_ignores_the_buffer() {
    static VALUES: [i32; 4] = [7, 8, 9, 10];
    let viewed = HArray1::from_buffer(&VALUES, 0, 2, true).unwrap();
    assert_eq!(viewed.array1().as_slice(), [7, 8, 9]);
    assert!(!viewed.array1().is_deletable());
    let fresh = HArray1::from_buffer(&VALUES, 1, 2, false).unwrap();
    assert_eq!(
        (fresh.array1().lower(), fresh.array1().as_slice()),
        (1, &[0, 0][..])
    );
    assert!(HArray1::from_buffer(&VALUES, 1, 5, true).is_err());
    viewed.change_array1().set_value(0, 1).unwrap();
    assert_eq!((VALUES[0], *viewed.array1().value(0).unwrap()), (7, 1));
}

#[test]
fn harray_copies_detach_from_their_source() {
    let mut buffer = [1, 2, 3];
    let mut source = Array1::from_slice(&mut buffer);
    source.update_lower_bound(4);
    let copy = HArray1::from_array(&source);
    source[4] = 100;
    assert_eq!((copy.array1().lower(), copy.array1()[4]), (4, 1));
    assert!(copy.array1().is_deletable());
    let grid = Array2::<i32>::with_bounds(0, 1, 5, 6).unwrap();
    let shared = HArray2::from_array(&grid);
    assert_eq!((shared.array2().lower_col(), shared.array2().len()), (5, 4));
}

#[test]
fn harray2_initializes_and_shares_the_rectangle() {
    let grid = Handle::new(HArray2::with_value(1, 2, 1, 3, 7_i32).unwrap());
    grid.get()
        .unwrap()
        .change_array2()
        .set_value(2, 3, 9)
        .unwrap();
    let alias = grid.clone();
    assert_eq!(alias.get().unwrap().array2()[(2, 3)], 9);
    assert_eq!(alias.get().unwrap().array2()[(1, 1)], 7);
    assert_eq!(
        HArray2::<i32>::with_bounds(1, 2, 1, 2)
            .unwrap()
            .array2()
            .len(),
        4
    );
}

#[test]
fn harray_runtime_types_name_the_template_and_derive_from_transient() {
    let one = <HArray1<f64> as RuntimeTyped>::static_type();
    let other = <HArray1<i32> as RuntimeTyped>::static_type();
    assert_eq!(
        (one.name(), other.name()),
        ("NCollection_HArray1", "NCollection_HArray1")
    );
    assert!(!std::ptr::eq(one, other));
    assert!(std::ptr::eq(
        one,
        <HArray1<f64> as RuntimeTyped>::static_type()
    ));
    assert!(one.is_subtype(<StandardTransient as RuntimeTyped>::static_type()));
    let handle = Handle::new(HArray2::<i32>::from_owned(Array2::default()));
    assert_eq!(handle.dynamic_type().unwrap().name(), "NCollection_HArray2");
    assert!(handle.is_kind_name("Standard_Transient"));
}

#[test]
fn const_array_views_detach_on_write() {
    let local = LocalArray::<i32, 4>::new(3);
    let mut view = local.to_array1_view();
    view[1] = 5;
    assert_eq!((view.upper(), local[1]), (2, 0));
    let mut vector = LinearVector::with_value(2, 4);
    let mut vector_view = vector.to_array1_view();
    vector_view[0] = 1;
    assert_eq!((vector_view[0], vector[0]), (1, 4));
    *vector.set_value(3, 8) += 1;
    assert_eq!(vector.as_slice(), [4, 4, 0, 9]);
}

#[test]
fn linear_vector_storage_queries_follow_the_native_pointer() {
    let mut vector = LinearVector::<i32>::default();
    assert!(!vector.has_data());
    vector.append(1);
    assert_eq!((vector.has_data(), vector.capacity()), (true, 2));
    vector.clear(false);
    assert!(vector.has_data());
    vector.clear(true);
    assert!(!vector.has_data());
    assert_eq!(LinearVector::<i32>::max_size(), usize::MAX);
}

#[test]
fn aliased_array_bounds_ends_and_raw_elements() {
    let mut array = AliasedArray::new(size_of::<i32>(), 3).unwrap();
    array.init(5_i32).unwrap();
    *array.change_first::<i32>().unwrap() = 1;
    *array.change_last::<i32>().unwrap() = 9;
    assert_eq!(
        (array.lower(), array.upper(), array.is_allocated()),
        (0, 2, true)
    );
    assert_eq!(
        (
            *array.first::<i32>().unwrap(),
            *array.last::<i32>().unwrap()
        ),
        (1, 9)
    );
    assert_eq!(array.byte_value(2).unwrap(), 9_i32.to_ne_bytes());
    array
        .change_byte_value(1)
        .unwrap()
        .copy_from_slice(&7_i32.to_ne_bytes());
    assert_eq!(*array.value::<i32>(1).unwrap(), 7);
    assert!(array.byte_value(3).is_err());
    assert_eq!(AliasedArray::empty(4).unwrap().upper(), -1);
    assert!(!AliasedArray::empty(4).unwrap().is_allocated());
}

#[test]
fn aliased_array_move_takes_storage_and_empties_the_source() {
    let mut source = AliasedArray::new(size_of::<u16>(), 2).unwrap();
    source.init(3_u16).unwrap();
    let mut destination = AliasedArray::new(size_of::<u8>(), 5).unwrap();
    destination.move_from(&mut source);
    assert_eq!((destination.stride(), destination.len()), (2, 2));
    assert_eq!(*destination.value::<u16>(1).unwrap(), 3);
    assert_eq!(
        (source.len(), source.stride(), source.is_deletable()),
        (0, 2, false)
    );
}

#[test]
fn sparse_block_table_grows_from_the_increment_by_doubling() {
    let mut array = SparseArray::new(4);
    assert_eq!(array.capacity(), 0);
    array.set_value(0, 1);
    assert_eq!(array.capacity(), 16); // 4 block pointers x 4 slots
    array.set_value(16, 2);
    assert_eq!(array.capacity(), 32);
    array.set_value(100, 3); // block 25 needs 32 pointers
    assert_eq!(array.capacity(), 128);
    assert!(array.unset_value(100));
    assert_eq!(array.capacity(), 128);
    array.clear();
    assert_eq!(array.capacity(), 0);
}

#[test]
fn sparse_assignment_and_exchange_carry_the_block_table() {
    let mut source = SparseArray::new(2);
    source.set_value(9, 'a'); // block 4 -> table of 8 pointers
    let mut destination = SparseArray::new(3);
    destination.set_value(0, 'z');
    destination.assign(&source);
    assert_eq!((destination.capacity(), destination.len()), (16, 1));
    let mut other = SparseArray::new(5);
    other.exchange(&mut destination);
    assert_eq!((other.capacity(), destination.capacity()), (16, 0));
    assert_eq!(other.entries().collect::<Vec<_>>(), [(9, &'a')]);
}

#[test]
fn sparse_mutable_entries_pair_keys_with_values() {
    let mut array = SparseArray::new(3);
    array.set_value(7, 70);
    array.set_value(1, 10);
    array
        .entries_mut()
        .for_each(|(key, value)| *value += key as i32);
    assert_eq!(array.entries().collect::<Vec<_>>(), [(1, &11), (7, &77)]);
}

#[test]
fn dynamic_array_keeps_the_typed_allocator_resource() {
    use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, OccAllocator};
    let resource = BaseAllocator::common();
    let typed = OccAllocator::<i32>::new(resource.clone());
    let mut array = DynamicArray::with_occ_allocator(4, &typed);
    array.append(5);
    assert!(array.allocator() == Some(&resource));
    assert_eq!(array[0], 5);
}
