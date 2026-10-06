//! The templated converting constructors `NCollection_VecN(const NCollection_VecN<Other>&)`.
use super::{Vec2, Vec3, Vec4};
/// `static_cast<T>(value)` between the supported element types. Float-to-integer casts
/// truncate toward zero. Out-of-range and NaN floating-to-integer inputs are undefined
/// in C++; only the existing i32 adapter preserves the pinned x86-64 `INT_MIN` result.
/// Other integer destinations retain Rust's saturation outside the defined native range.
/// For example, `ElementCast::<f32>::cast(1.5_f64)` converts within the native range.
pub trait ElementCast<T> {
    /// Converts one element; `ElementCast::<i32>::cast(2.7_f64) == 2`.
    fn cast(self) -> T;
}
macro_rules! element_casts {
    ($($from:ty),+ => $to:ty) => {
        $(
            impl ElementCast<$to> for $from {
                fn cast(self) -> $to {
                    self as $to
                }
            }
        )+
    };
}
macro_rules! integral_source_casts {
    ($($target:ty),+) => {$(
        element_casts!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize => $target);
    )+};
}
integral_source_casts!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
element_casts!(f32, f64 => f32);
element_casts!(f32, f64 => f64);
macro_rules! floating_destination_casts {
    ($($target:ty),+) => {$(element_casts!(f32, f64 => $target);)+};
}
// Only in-range truncated floating values are defined by the native converting constructor.
floating_destination_casts!(i8, u8, i16, u16, u32, i64, u64, isize, usize);
macro_rules! boolean_casts {
    ($($target:ty),+) => {$(
        impl ElementCast<$target> for bool {
            fn cast(self) -> $target { u8::from(self) as $target }
        }
        impl ElementCast<bool> for $target {
            fn cast(self) -> bool { self != 0 as $target }
        }
    )+};
}
boolean_casts!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
impl ElementCast<bool> for bool {
    fn cast(self) -> bool {
        self
    }
}
/// x86-64 truncating float-to-int conversion: the "integer indefinite" `INT_MIN` for NaN and
/// for values outside `[-2^31, 2^31)`.
macro_rules! truncating_casts {
    ($($from:ty),+) => {
        $(
            impl ElementCast<i32> for $from {
                fn cast(self) -> i32 {
                    let bound: $from = 2_147_483_648.0;
                    if self.is_nan() || self >= bound || self < -bound {
                        return i32::MIN;
                    }
                    self as i32
                }
            }
        )+
    };
}
truncating_casts!(f32, f64);
macro_rules! vector_conversion {
    ($vector:ident, [$($field:ident),+]) => {
        impl<T> $vector<T> {
            /// Converts every component with `static_cast`, as the templated converting
            /// constructor; `Vec3::<f32>::convert_from(Vec3::new(1.0_f64, 2.0, 3.0))`.
            pub fn convert_from<U: ElementCast<T>>(other: $vector<U>) -> Self {
                Self { $($field: other.$field.cast()),+ }
            }
        }
    };
}
vector_conversion!(Vec2, [x, y]);
vector_conversion!(Vec3, [x, y, z]);
vector_conversion!(Vec4, [x, y, z, w]);

// Preserve the existing Rust i128/u128 vector extensions when constructing native zero constants.
element_casts!(i32 => i128);
element_casts!(i32 => u128);
