//! Typed pixel access: the types that `Image_PixMap::Value<ColorType_t>` and
//! `Image_PixMapTypedData<PixelType_t>` reinterpret pixel bytes as.
//!
//! The native code reinterprets a pixel pointer; the port copies the native-endian bytes
//! into a value and back, which reads and writes the same bits.
use super::colors::*;
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};

/// A plain-old-data pixel value of `SIZE` bytes.
pub trait ImagePixel: Copy {
    /// `sizeof(PixelType_t)`.
    const SIZE: usize;
    /// Reads a value from its native-endian bytes; `u16::read_pixel(&[1, 0]) == 1`.
    fn read_pixel(bytes: &[u8]) -> Self;
    /// Writes the value as native-endian bytes; `1u16.write_pixel(&mut bytes)`.
    fn write_pixel(&self, bytes: &mut [u8]);
}

/// Implements [`ImagePixel`] for a scalar through its native-endian byte conversions.
macro_rules! scalar_pixel {
    ($($scalar:ty),*) => {$(
        impl ImagePixel for $scalar {
            const SIZE: usize = size_of::<$scalar>();
            fn read_pixel(bytes: &[u8]) -> Self {
                <$scalar>::from_ne_bytes(bytes[..Self::SIZE].try_into().expect("pixel bytes"))
            }
            fn write_pixel(&self, bytes: &mut [u8]) {
                bytes[..Self::SIZE].copy_from_slice(&self.to_ne_bytes());
            }
        }
    )*};
}
scalar_pixel!(u8, u16, f32);

/// Implements [`ImagePixel`] for an array-backed pixel of `$count` scalars.
macro_rules! array_pixel {
    ($name:ty, $scalar:ty, $count:literal, |$value:ident| $components:expr, |$array:ident| $build:expr) => {
        impl ImagePixel for $name {
            const SIZE: usize = $count * size_of::<$scalar>();
            fn read_pixel(bytes: &[u8]) -> Self {
                let step = size_of::<$scalar>();
                let $array: [$scalar; $count] =
                    std::array::from_fn(|i| <$scalar>::read_pixel(&bytes[i * step..]));
                $build
            }
            fn write_pixel(&self, bytes: &mut [u8]) {
                let step = size_of::<$scalar>();
                let $value = self;
                let components: [$scalar; $count] = $components;
                for (i, component) in components.iter().enumerate() {
                    component.write_pixel(&mut bytes[i * step..]);
                }
            }
        }
    };
}
array_pixel!(ImageColorRgb, u8, 3, |p| p.v, |v| Self { v });
array_pixel!(ImageColorRgb32, u8, 4, |p| p.v, |v| Self { v });
array_pixel!(ImageColorRgba, u8, 4, |p| p.v, |v| Self { v });
array_pixel!(ImageColorBgr, u8, 3, |p| p.v, |v| Self { v });
array_pixel!(ImageColorBgr32, u8, 4, |p| p.v, |v| Self { v });
array_pixel!(ImageColorBgra, u8, 4, |p| p.v, |v| Self { v });
array_pixel!(ImageColorRgf, f32, 2, |p| p.v, |v| Self { v });
array_pixel!(ImageColorRgbf, f32, 3, |p| p.v, |v| Self { v });
array_pixel!(ImageColorBgrf, f32, 3, |p| p.v, |v| Self { v });
array_pixel!(ImageColorRgbaf, f32, 4, |p| p.v, |v| Self { v });
array_pixel!(ImageColorBgraf, f32, 4, |p| p.v, |v| Self { v });
array_pixel!(Vec2<u16>, u16, 2, |p| [p.x, p.y], |v| Vec2::new(v[0], v[1]));
array_pixel!(Vec3<u8>, u8, 3, |p| [p.x, p.y, p.z], |v| Vec3::new(
    v[0], v[1], v[2]
));
array_pixel!(Vec4<u16>, u16, 4, |p| [p.x, p.y, p.z, p.w], |v| Vec4::new(
    v[0], v[1], v[2], v[3]
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_round_trip_through_their_bytes() {
        let mut bytes = [0u8; 16];
        ImageColorRgbaf {
            v: [0.5, 1.0, 0.0, 0.25],
        }
        .write_pixel(&mut bytes);
        assert_eq!(ImageColorRgbaf::read_pixel(&bytes).v, [0.5, 1.0, 0.0, 0.25]);
        Vec4::new(1u16, 2, 3, 4).write_pixel(&mut bytes);
        assert_eq!(<Vec2<u16>>::read_pixel(&bytes[4..]), Vec2::new(3, 4));
        assert_eq!(ImageColorBgra::SIZE, 4);
    }
}
