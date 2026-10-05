//! Borrow the native homogeneous component array without copying scalar values.
use super::{Vec2, Vec3, Vec4};

macro_rules! contiguous_component_storage {
    ($vector:ident, $count:literal) => {
        impl<T> $vector<T> {
            /// Borrows native component storage; `Vec3::new(1, 2, 3).as_components()[1] == 2`.
            #[allow(unsafe_code)] // Audited repr(C) homogeneous storage seam; checked by Miri.
            pub const fn as_components(&self) -> &[T; $count] {
                // repr(C) and homogeneous fields guarantee the array's size, alignment and order.
                // Casting the whole-struct borrow retains provenance for every component.
                unsafe { &*(self as *const Self).cast::<[T; $count]>() }
            }
            /// Borrows storage writably; `vector.as_components_mut()[1] = 7` updates `vector.y`.
            #[allow(unsafe_code)] // Exclusive whole-object provenance covers the complete array.
            pub const fn as_components_mut(&mut self) -> &mut [T; $count] {
                // The exclusive whole-struct borrow covers the complete homogeneous array.
                unsafe { &mut *(self as *mut Self).cast::<[T; $count]>() }
            }
        }
    };
}

contiguous_component_storage!(Vec2, 2);
contiguous_component_storage!(Vec3, 3);
contiguous_component_storage!(Vec4, 4);
