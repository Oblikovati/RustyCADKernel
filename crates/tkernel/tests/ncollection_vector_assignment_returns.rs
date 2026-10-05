//! Borrowed OCCT component assignments return the existing destination reference.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};

macro_rules! assignment_return_contract {
    ($test:ident, $vector:ident, $count:literal) => {
        #[test]
        fn $test() {
            let mut destination = $vector::from_value(12_i32);
            let original: *const $vector<i32> = &destination;
            let right = $vector::from_value(3);
            let returned = destination.add_assign_returning(&right);
            assert!(std::ptr::eq(returned, original));
            assert_eq!(returned.data(), [15; $count]);
            let returned = returned.sub_assign_returning(&right);
            assert!(std::ptr::eq(returned, original));
            assert_eq!(returned.data(), [12; $count]);
            let returned = returned.mul_assign_returning(&right);
            assert!(std::ptr::eq(returned, original));
            assert_eq!(returned.data(), [36; $count]);
            let returned = returned.div_assign_returning(&right);
            assert!(std::ptr::eq(returned, original));
            assert_eq!(returned.data(), [12; $count]);
            assert_eq!(right.data(), [3; $count]);
        }
    };
}

assignment_return_contract!(vec2_assignments_return_destination, Vec2, 2);
assignment_return_contract!(vec3_assignments_return_destination, Vec3, 3);
assignment_return_contract!(vec4_assignments_return_destination, Vec4, 4);
