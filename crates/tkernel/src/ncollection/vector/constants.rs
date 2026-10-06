//! Native constructors explicitly invoke Element_t(0), independently of its default constructor.
use super::{ElementCast, Vec2, Vec3, Vec4};
macro_rules! vector_zero_default {
    ($vector:ident, [$($field:ident),+]) => {
        impl<T> Default for $vector<T> where i32: ElementCast<T> {
            fn default() -> Self { Self { $($field: 0_i32.cast()),+ } }
        }
    };
}
vector_zero_default!(Vec2, [x, y]);
vector_zero_default!(Vec3, [x, y, z]);
vector_zero_default!(Vec4, [x, y, z, w]);
