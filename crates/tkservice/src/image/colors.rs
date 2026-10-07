//! The pixel structures of `Image_Color.hxx`: packed components in memory order, with
//! accessors that name the channels whatever the order.

/// Declares one pixel structure: its component type, count, channel indices and `Length`.
macro_rules! image_color {
    ($(#[$doc:meta])* $name:ident, $component:ty, $count:literal, $length:literal,
     [$(($getter:ident, $setter:ident, $index:literal, $channel:literal)),*]) => {
        $(#[$doc])*
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name {
            /// `v`: the components in memory order.
            pub v: [$component; $count],
        }
        impl $name {
            /// The native `Length()`: the number of color channels (an alpha padding byte
            /// is not counted); `Self::length()`.
            pub const fn length() -> i32 {
                $length
            }
            $(
                #[doc = concat!("The ", $channel, " channel; `pixel.", stringify!($getter), "()`.")]
                pub const fn $getter(&self) -> $component {
                    self.v[$index]
                }
                #[doc = concat!("The ", $channel, " channel, mutable; `*pixel.", stringify!($setter), "() = value`.")]
                pub fn $setter(&mut self) -> &mut $component {
                    &mut self.v[$index]
                }
            )*
        }
    };
}

image_color!(
    /// 3 bytes RGB (`Image_ColorRGB`).
    ImageColorRgb, u8, 3, 3,
    [(r, r_mut, 0, "red"), (g, g_mut, 1, "green"), (b, b_mut, 2, "blue")]
);
image_color!(
    /// 4 bytes RGB with an unused byte (`Image_ColorRGB32`); `pad` is the native `a_`.
    ImageColorRgb32, u8, 4, 3,
    [(r, r_mut, 0, "red"), (g, g_mut, 1, "green"), (b, b_mut, 2, "blue"), (pad, pad_mut, 3, "padding (`a_`)")]
);
image_color!(
    /// 4 bytes RGBA (`Image_ColorRGBA`).
    ImageColorRgba, u8, 4, 4,
    [(r, r_mut, 0, "red"), (g, g_mut, 1, "green"), (b, b_mut, 2, "blue"), (a, a_mut, 3, "alpha")]
);
image_color!(
    /// 3 bytes BGR (`Image_ColorBGR`).
    ImageColorBgr, u8, 3, 3,
    [(r, r_mut, 2, "red"), (g, g_mut, 1, "green"), (b, b_mut, 0, "blue")]
);
image_color!(
    /// 4 bytes BGR with an unused byte (`Image_ColorBGR32`); `pad` is the native `a_`.
    ImageColorBgr32, u8, 4, 3,
    [(r, r_mut, 2, "red"), (g, g_mut, 1, "green"), (b, b_mut, 0, "blue"), (pad, pad_mut, 3, "padding (`a_`)")]
);
image_color!(
    /// 4 bytes BGRA (`Image_ColorBGRA`).
    ImageColorBgra, u8, 4, 4,
    [(r, r_mut, 2, "red"), (g, g_mut, 1, "green"), (b, b_mut, 0, "blue"), (a, a_mut, 3, "alpha")]
);
image_color!(
    /// 2 floats RG (`Image_ColorRGF`).
    ImageColorRgf, f32, 2, 2,
    [(r, r_mut, 0, "red"), (g, g_mut, 1, "green")]
);
image_color!(
    /// 3 floats RGB (`Image_ColorRGBF`).
    ImageColorRgbf, f32, 3, 3,
    [(r, r_mut, 0, "red"), (g, g_mut, 1, "green"), (b, b_mut, 2, "blue")]
);
image_color!(
    /// 3 floats BGR (`Image_ColorBGRF`).
    ImageColorBgrf, f32, 3, 3,
    [(r, r_mut, 2, "red"), (g, g_mut, 1, "green"), (b, b_mut, 0, "blue")]
);
image_color!(
    /// 4 floats RGBA (`Image_ColorRGBAF`).
    ImageColorRgbaf, f32, 4, 4,
    [(r, r_mut, 0, "red"), (g, g_mut, 1, "green"), (b, b_mut, 2, "blue"), (a, a_mut, 3, "alpha")]
);
image_color!(
    /// 4 floats BGRA (`Image_ColorBGRAF`).
    ImageColorBgraf, f32, 4, 4,
    [(r, r_mut, 2, "red"), (g, g_mut, 1, "green"), (b, b_mut, 0, "blue"), (a, a_mut, 3, "alpha")]
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_follow_the_memory_order() {
        let mut bgra = ImageColorBgra { v: [1, 2, 3, 4] };
        assert_eq!((bgra.r(), bgra.g(), bgra.b(), bgra.a()), (3, 2, 1, 4));
        *bgra.r_mut() = 9;
        assert_eq!(bgra.v, [1, 2, 9, 4]);
        assert_eq!(
            (ImageColorRgb32::length(), ImageColorRgbaf::length()),
            (3, 4)
        );
        let rgf = ImageColorRgf { v: [0.5, 0.25] };
        assert_eq!((rgf.r(), rgf.g()), (0.5, 0.25));
    }
}
