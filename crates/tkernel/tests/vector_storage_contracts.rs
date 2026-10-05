//! Component borrows preserve native contiguous storage and address identity.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn vec2_component_borrow_exposes_fields_and_updates_original_storage() {
    let mut vector = Vec2::new(1_i32, 2);
    let components = vector.as_components();
    assert!(std::ptr::eq(&components[0], &vector.x));
    assert!(std::ptr::eq(&components[1], &vector.y));
    vector.as_components_mut()[1] = 7;
    assert_eq!(vector.y, 7);
    assert_eq!(vector.as_components(), &[1, 7]);
}

#[test]
fn vec3_component_borrow_exposes_fields_and_updates_original_storage() {
    let mut vector = Vec3::new(1_i32, 2, 3);
    let components = vector.as_components();
    assert!(std::ptr::eq(&components[0], &vector.x));
    assert!(std::ptr::eq(&components[1], &vector.y));
    assert!(std::ptr::eq(&components[2], &vector.z));
    vector.as_components_mut()[2] = 7;
    assert_eq!(vector.z, 7);
    assert_eq!(vector.as_components(), &[1, 2, 7]);
}

#[test]
fn vec4_component_borrow_exposes_fields_and_updates_original_storage() {
    let mut vector = Vec4::new(1_i32, 2, 3, 4);
    let components = vector.as_components();
    assert!(std::ptr::eq(&components[0], &vector.x));
    assert!(std::ptr::eq(&components[1], &vector.y));
    assert!(std::ptr::eq(&components[2], &vector.z));
    assert!(std::ptr::eq(&components[3], &vector.w));
    vector.as_components_mut()[3] = 7;
    assert_eq!(vector.w, 7);
    assert_eq!(vector.as_components(), &[1, 2, 3, 7]);
}

#[repr(align(32))]
struct AlignedComponent(i32);

#[test]
fn generic_component_alignment_and_stride_are_retained() {
    let [x, y, z] = [1, 2, 3].map(AlignedComponent);
    let mut vector = Vec3::new(x, y, z);
    assert_eq!(
        size_of::<Vec3<AlignedComponent>>(),
        size_of::<[AlignedComponent; 3]>()
    );
    assert_eq!(
        align_of::<Vec3<AlignedComponent>>(),
        align_of::<[AlignedComponent; 3]>()
    );
    let components = vector.as_components();
    assert_eq!(components.as_ptr().align_offset(32), 0);
    assert!(std::ptr::eq(&components[2], &vector.z));
    vector.as_components_mut()[2].0 = 7;
    assert_eq!(vector.z.0, 7);
}

struct DropTrackedComponent(Rc<Cell<usize>>);

impl Drop for DropTrackedComponent {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn component_views_do_not_copy_or_duplicate_noncopy_scalar_drops() {
    let dropped = Rc::new(Cell::new(0));
    let mut vector = Vec2::new(
        DropTrackedComponent(dropped.clone()),
        DropTrackedComponent(dropped.clone()),
    );
    assert!(Rc::ptr_eq(&vector.as_components()[1].0, &dropped));
    vector.as_components_mut().swap(0, 1);
    assert_eq!(dropped.get(), 0);
    drop(vector);
    assert_eq!(dropped.get(), 2);
}

#[test]
fn zero_sized_rust_component_views_keep_array_length() {
    let mut vector = Vec4::new((), (), (), ());
    assert_eq!(vector.as_components().len(), 4);
    assert_eq!(vector.as_components_mut().len(), 4);
}

#[test]
fn component_views_work_during_constant_evaluation() {
    const COMPONENTS: [i32; 3] = *Vec3::new(1, 2, 3).as_components();
    const MUTATED: Vec2<i32> = {
        let mut vector = Vec2::new(1, 2);
        vector.as_components_mut()[1] = 7;
        vector
    };
    assert_eq!(COMPONENTS, [1, 2, 3]);
    assert_eq!(MUTATED.y, 7);
}

#[test]
fn integer_vector_sizes_and_alignment_match_native_storage() {
    let sizes = [
        size_of::<Vec2<i32>>(),
        size_of::<Vec3<i32>>(),
        size_of::<Vec4<i32>>(),
    ];
    let alignments = [
        align_of::<Vec2<i32>>(),
        align_of::<Vec3<i32>>(),
        align_of::<Vec4<i32>>(),
    ];
    assert_eq!(sizes, [8, 12, 16]);
    assert_eq!(alignments, [4, 4, 4]);
}
