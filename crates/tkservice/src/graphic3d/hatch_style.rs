//! `Graphic3d_HatchStyle`: a 32x32 hatch bitmap, predefined or custom.
use crate::aspect::AspectHatchStyle;
use crate::image::{ImageFormat, ImagePixMap};
use crate::json_dump::{class_begin, field_dumped, field_integer};
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use std::io;
use std::sync::atomic::{AtomicI32, Ordering};

/// The predefined 32-row bitmaps, indexed by `Aspect_HatchStyle` value.
const PREDEFINED_PATTERNS: [[u32; 32]; 13] = [
    [
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
    ],
    [
        0xFFFFFFFF, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE,
        0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB,
        0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE,
        0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB,
        0xEEEEEEEE, 0xBBBBBBBB, 0xEEEEEEEE, 0xBBBBBBBB,
    ],
    [
        0x81818181, 0x24242424, 0x18181818, 0x42424242, 0x81818181, 0x24242424, 0x18181818,
        0x42424242, 0x81818181, 0x24242424, 0x18181818, 0x42424242, 0x81818181, 0x24242424,
        0x18181818, 0x42424242, 0x81818181, 0x24242424, 0x18181818, 0x42424242, 0x81818181,
        0x24242424, 0x18181818, 0x42424242, 0x81818181, 0x24242424, 0x18181818, 0x42424242,
        0x81818181, 0x24242424, 0x18181818, 0x42424242,
    ],
    [
        0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF,
        0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888,
        0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF,
        0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888,
        0xFFFFFFFF, 0x88888888, 0xFFFFFFFF, 0x88888888,
    ],
    [
        0xFFFFFFFF, 0x80808080, 0x80808080, 0x80808080, 0xFFFFFFFF, 0x80808080, 0x80808080,
        0x80808080, 0xFFFFFFFF, 0x80808080, 0x80808080, 0x80808080, 0xFFFFFFFF, 0x80808080,
        0x80808080, 0x80808080, 0xFFFFFFFF, 0x80808080, 0x80808080, 0x80808080, 0xFFFFFFFF,
        0x80808080, 0x80808080, 0x80808080, 0xFFFFFFFF, 0x80808080, 0x80808080, 0x80808080,
        0xFFFFFFFF, 0x80808080, 0x80808080, 0x80808080,
    ],
    [
        0x88888888, 0x22222222, 0x88888888, 0x22222222, 0x88888888, 0x22222222, 0x88888888,
        0x22222222, 0x88888888, 0x22222222, 0x88888888, 0x22222222, 0x88888888, 0x22222222,
        0x88888888, 0x22222222, 0x88888888, 0x22222222, 0x88888888, 0x22222222, 0x88888888,
        0x22222222, 0x88888888, 0x22222222, 0x88888888, 0x22222222, 0x88888888, 0x22222222,
        0x88888888, 0x22222222, 0x88888888, 0x22222222,
    ],
    [
        0x11111111, 0x44444444, 0x11111111, 0x44444444, 0x11111111, 0x44444444, 0x11111111,
        0x44444444, 0x11111111, 0x44444444, 0x11111111, 0x44444444, 0x11111111, 0x44444444,
        0x11111111, 0x44444444, 0x11111111, 0x44444444, 0x11111111, 0x44444444, 0x11111111,
        0x44444444, 0x11111111, 0x44444444, 0x11111111, 0x44444444, 0x11111111, 0x44444444,
        0x11111111, 0x44444444, 0x11111111, 0x44444444,
    ],
    [
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0xFFFFFFFF, 0x00000000, 0xFFFFFFFF, 0x00000000,
    ],
    [
        0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111,
        0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111,
        0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111,
        0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111, 0x11111111,
        0x11111111, 0x11111111, 0x11111111, 0x11111111,
    ],
    [
        0x80808080, 0x20202020, 0x08080808, 0x02020202, 0x80808080, 0x20202020, 0x08080808,
        0x02020202, 0x80808080, 0x20202020, 0x08080808, 0x02020202, 0x80808080, 0x20202020,
        0x08080808, 0x02020202, 0x80808080, 0x20202020, 0x08080808, 0x02020202, 0x80808080,
        0x20202020, 0x08080808, 0x02020202, 0x80808080, 0x20202020, 0x08080808, 0x02020202,
        0x80808080, 0x20202020, 0x08080808, 0x02020202,
    ],
    [
        0x01010101, 0x04040404, 0x10101010, 0x40404040, 0x01010101, 0x04040404, 0x10101010,
        0x40404040, 0x01010101, 0x04040404, 0x10101010, 0x40404040, 0x01010101, 0x04040404,
        0x10101010, 0x40404040, 0x01010101, 0x04040404, 0x10101010, 0x40404040, 0x01010101,
        0x04040404, 0x10101010, 0x40404040, 0x01010101, 0x04040404, 0x10101010, 0x40404040,
        0x01010101, 0x04040404, 0x10101010, 0x40404040,
    ],
    [
        0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000, 0xFFFFFFFF, 0x00000000, 0x00000000,
        0x00000000, 0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000, 0xFFFFFFFF, 0x00000000,
        0x00000000, 0x00000000, 0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000, 0xFFFFFFFF,
        0x00000000, 0x00000000, 0x00000000, 0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000,
        0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000,
    ],
    [
        0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010,
        0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010,
        0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010,
        0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010, 0x10101010,
        0x10101010, 0x10101010, 0x10101010, 0x10101010,
    ],
];

/// The id counter of custom patterns, starting after the predefined styles.
static HATCH_STYLE_COUNTER: AtomicI32 = AtomicI32::new(AspectHatchStyle::Nb as i32 - 1);

/// A hatch pattern; `Graphic3dHatchStyle::predefined(AspectHatchStyle::Grid)`.
pub struct Graphic3dHatchStyle {
    transient: StandardTransient,
    /// `myPattern`: the custom bitmap, null for a predefined style.
    pattern: Handle<Buffer>,
    /// `myHatchType`: the style index (custom ones get new ids).
    hatch_type: i32,
}
rustycad_tkernel::impl_transient!(
    Graphic3dHatchStyle,
    "Graphic3d_HatchStyle",
    StandardTransient,
    transient
);

impl Graphic3dHatchStyle {
    /// A custom style from a 32x32 gray image; `Standard_ProgramError` for another image.
    ///
    /// Example: `Graphic3dHatchStyle::from_pattern(&image)?`.
    pub fn from_pattern(pattern: &Handle<ImagePixMap>) -> Result<Self, Failure> {
        let Some(image) = pattern.get() else {
            return Err(program_error("Null pointer to a hatch pattern image"));
        };
        if image.width() != 32 || image.height() != 32 || image.format() != ImageFormat::Gray {
            return Err(program_error(&format!(
                "Hatch pattern must be a 32*32 bitmap (Image_Format_Gray format); got {}x{} {:?}",
                image.width(),
                image.height(),
                image.format()
            )));
        }
        let bytes = image.data().data.data().to_vec();
        let mut buffer = Buffer::new(Some(BaseAllocator::aligned(16)), bytes.len())?;
        buffer.as_mut_slice()[..bytes.len()].copy_from_slice(&bytes);
        let hatch_type = HATCH_STYLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(Self {
            transient: StandardTransient::default(),
            pattern: Handle::new(buffer),
            hatch_type,
        })
    }
    /// A predefined style; `Graphic3dHatchStyle::predefined(AspectHatchStyle::Grid)`.
    pub fn predefined(style: AspectHatchStyle) -> Self {
        Self {
            transient: StandardTransient::default(),
            pattern: Handle::default(),
            hatch_type: style as i32,
        }
    }
    /// The 128 bytes of the 32x32 bitmap (rows of native-endian 32-bit words), `None` for
    /// an unknown style. Example: `style.pattern()`.
    pub fn pattern(&self) -> Option<Vec<u8>> {
        if let Some(buffer) = self.pattern.get() {
            return Some(buffer.as_slice().to_vec());
        }
        let rows = PREDEFINED_PATTERNS.get(usize::try_from(self.hatch_type).ok()?)?;
        Some(rows.iter().flat_map(|row| row.to_ne_bytes()).collect())
    }
    /// The style index; `style.hatch_type()`.
    pub fn hatch_type(&self) -> i32 {
        self.hatch_type
    }
    /// `DumpJson`: class name, custom bitmap and style index; `style.dump_json(&mut s, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Graphic3d_HatchStyle")?;
        if let Some(buffer) = self.pattern.get() {
            field_dumped(stream, depth, "myPattern.get()", &|s, d| {
                buffer.dump_json(s, d)
            })?;
        }
        field_integer(stream, "myHatchType", self.hatch_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predefined_and_custom_patterns() {
        let grid = Graphic3dHatchStyle::predefined(AspectHatchStyle::Diagonal45);
        let bytes = grid.pattern().unwrap();
        assert_eq!(bytes.len(), 128);
        assert_eq!(&bytes[4..8], &0x22222222u32.to_ne_bytes());
        assert!(Graphic3dHatchStyle::from_pattern(&Handle::default()).is_err());
        let image = ImagePixMap::default();
        image.init_zero(ImageFormat::Gray, 32, 32, 0, 7);
        let custom = Graphic3dHatchStyle::from_pattern(&Handle::new(image)).unwrap();
        assert!(custom.hatch_type() >= AspectHatchStyle::Nb as i32);
        assert_eq!(custom.pattern().unwrap(), vec![7; 1024]);
    }
}
