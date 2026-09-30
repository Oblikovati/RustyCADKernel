//! The templated converting constructors `NCollection_VecN(const NCollection_VecN<Other>&)`.
use super::{Vec2, Vec3, Vec4};
/// `static_cast<T>(value)` between the supported element types. Float-to-integer casts
/// truncate toward zero. Out-of-range and NaN inputs are undefined behavior in C++; the
/// pinned x86-64 profile converts them with `cvttss2si`/`cvttsd2si`, which returns `INT_MIN`,
/// and this port does the same; `ElementCast::<f32>::cast(1.5_f64)`.
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
element_casts!(f32, f64, i32 => f32);
element_casts!(f32, f64, i32 => f64);
element_casts!(i32 => i32);
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
