// Copyright (c) 2013-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The component swizzles of `NCOLLECTION_VEC_COMPONENTS_2D/3D` and the colour aliases.
use super::{Vec2, Vec3, Vec4};

/// `NCOLLECTION_VEC_COMPONENTS_2D(a, b)`: the `ab()` and `ba()` copies.
macro_rules! components_2d {
    ($a:ident, $b:ident, $ab:ident, $ba:ident) => {
        #[doc = concat!("Copies (", stringify!($a), ", ", stringify!($b), "), as `", stringify!($ab), "()`.")]
        pub const fn $ab(&self) -> Vec2<T> {
            Vec2::new(self.$a(), self.$b())
        }
        #[doc = concat!("Copies (", stringify!($b), ", ", stringify!($a), "), as `", stringify!($ba), "()`.")]
        pub const fn $ba(&self) -> Vec2<T> {
            Vec2::new(self.$b(), self.$a())
        }
    };
}
/// `NCOLLECTION_VEC_COMPONENTS_3D(a, b, c)`: the six permutation copies in macro order.
macro_rules! components_3d {
    ($a:ident, $b:ident, $c:ident, [$abc:ident, $acb:ident, $bac:ident, $bca:ident, $cba:ident, $cab:ident]) => {
        components_3d!(@one $abc, $a, $b, $c);
        components_3d!(@one $acb, $a, $c, $b);
        components_3d!(@one $bac, $b, $a, $c);
        components_3d!(@one $bca, $b, $c, $a);
        components_3d!(@one $cba, $c, $b, $a);
        components_3d!(@one $cab, $c, $a, $b);
    };
    (@one $name:ident, $first:ident, $second:ident, $third:ident) => {
        #[doc = concat!("Copies (", stringify!($first), ", ", stringify!($second), ", ", stringify!($third), "), as `", stringify!($name), "()`.")]
        pub const fn $name(&self) -> Vec3<T> {
            Vec3::new(self.$first(), self.$second(), self.$third())
        }
    };
}
/// Component getters by position, as the const `x()`, `y()`, ...; the fields are public, so
/// the non-const reference overloads are plain field access.
macro_rules! position_getters {
    ($vector:ident, [$($field:ident),+]) => {
        impl<T: Copy> $vector<T> {
            $(
                #[doc = concat!("Reads `", stringify!($field), "`, as the const `", stringify!($field), "()`.")]
                pub const fn $field(&self) -> T {
                    self.$field
                }
            )+
        }
    };
}
/// The colour aliases `r()`, `g()`, `b()`, `a()` and their non-const reference overloads.
macro_rules! color_aliases {
    ($vector:ident, [$(($field:ident, $color:ident, $color_mut:ident)),+]) => {
        impl<T: Copy> $vector<T> {
            $(
                #[doc = concat!("Reads `", stringify!($field), "` through its colour alias, as `", stringify!($color), "()`.")]
                pub const fn $color(&self) -> T {
                    self.$field
                }
                #[doc = concat!("Borrows `", stringify!($field), "` mutably, as the non-const `", stringify!($color), "()`.")]
                pub const fn $color_mut(&mut self) -> &mut T {
                    &mut self.$field
                }
            )+
        }
    };
}
position_getters!(Vec2, [x, y]);
position_getters!(Vec3, [x, y, z]);
position_getters!(Vec4, [x, y, z, w]);
color_aliases!(Vec3, [(x, r, r_mut), (y, g, g_mut), (z, b, b_mut)]);
color_aliases!(
    Vec4,
    [(x, r, r_mut), (y, g, g_mut), (z, b, b_mut), (w, a, a_mut)]
);

impl<T: Copy> Vec2<T> {
    components_2d!(x, y, xy, yx);
}
impl<T: Copy> Vec3<T> {
    components_2d!(x, y, xy, yx);
    components_2d!(x, z, xz, zx);
    components_2d!(y, z, yz, zy);
    components_3d!(x, y, z, [xyz, xzy, yxz, yzx, zyx, zxy]);
}
impl<T: Copy> Vec4<T> {
    components_2d!(x, y, xy, yx);
    components_2d!(x, z, xz, zx);
    components_2d!(x, w, xw, wx);
    components_2d!(y, z, yz, zy);
    components_2d!(y, w, yw, wy);
    components_2d!(z, w, zw, wz);
    components_3d!(x, y, z, [xyz, xzy, yxz, yzx, zyx, zxy]);
    components_3d!(x, y, w, [xyw, xwy, yxw, ywx, wyx, wxy]);
    components_3d!(x, z, w, [xzw, xwz, zxw, zwx, wzx, wxz]);
    components_3d!(y, z, w, [yzw, ywz, zyw, zwy, wzy, wyz]);
    components_3d!(r, g, b, [rgb, rbg, grb, gbr, bgr, brg]);
}
