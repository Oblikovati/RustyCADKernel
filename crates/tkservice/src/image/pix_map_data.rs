//! `Image_PixMapData` and `Image_PixMapTypedData`: the pixel buffer and its 2D/3D layout.
//!
//! The native class is an `NCollection_Buffer` (owned memory or a wrapped user pointer) plus
//! a pointer to the topmost row. The port keeps the bytes in an owned `Vec<u8>` (a wrapper
//! adopts the caller's bytes, see `ImagePixMap::init_wrapper`) and the top row as an
//! offset; rows of a bottom-up image are addressed backwards from it, as the native
//! `TopToDown == size_t(-1)` multiplication does.
use super::pixel::ImagePixel;
use rustycad_tkernel::ncollection::Vec3;

/// `size_t(-1)`: the `TopToDown` value of a bottom-up image.
pub const IMAGE_BOTTOM_UP: usize = usize::MAX;

/// A pixel buffer with its layout; `ImagePixMapData::default().is_empty()`.
#[derive(Clone, Debug, PartialEq)]
pub struct ImagePixMapData {
    /// The buffer (`myData`, `mySize`); `None` for a null buffer.
    bytes: Option<Vec<u8>>,
    /// `myTopRowPtr` as an offset into the buffer.
    top_row: usize,
    /// `SizeBPP`: bytes per pixel.
    pub size_bpp: usize,
    /// `SizeX`: width in pixels.
    pub size_x: usize,
    /// `SizeY`: height in pixels.
    pub size_y: usize,
    /// `SizeZ`: depth in pixels.
    pub size_z: usize,
    /// `SizeRowBytes`: bytes per line.
    pub size_row_bytes: usize,
    /// `SizeSliceBytes`: bytes per 2D slice.
    pub size_slice_bytes: usize,
    /// `TopToDown`: 1 for top-down rows, [`IMAGE_BOTTOM_UP`] for bottom-up rows.
    pub top_to_down: usize,
}

impl Default for ImagePixMapData {
    /// A null buffer, zero sizes and bottom-up rows, as `Image_PixMapData()`.
    fn default() -> Self {
        Self {
            bytes: None,
            top_row: 0,
            size_bpp: 0,
            size_x: 0,
            size_y: 0,
            size_z: 0,
            size_row_bytes: 0,
            size_slice_bytes: 0,
            top_to_down: IMAGE_BOTTOM_UP,
        }
    }
}

impl ImagePixMapData {
    /// `Init(alloc, bpp, x, y, rowBytes, data)` for one slice; see [`Self::init_3d`].
    ///
    /// Example: `data.init(4, 2, 2, 0, None)`.
    pub fn init(
        &mut self,
        size_bpp: usize,
        size_x: usize,
        size_y: usize,
        size_row_bytes: usize,
        bytes: Option<Vec<u8>>,
    ) -> bool {
        self.init_3d(
            size_bpp,
            Vec3::new(size_x, size_y, 1),
            size_row_bytes,
            bytes,
        )
    }
    /// `Init(alloc, bpp, xyz, rowBytes, data)`: adopts `bytes` or allocates zeroed storage
    /// (the native allocation is uninitialised); a zero row size means `SizeX * bpp`.
    /// Returns false for an empty buffer. Example: `data.init_3d(1, Vec3::new(2, 2, 2), 0, None)`.
    pub fn init_3d(
        &mut self,
        size_bpp: usize,
        size_xyz: Vec3<usize>,
        size_row_bytes: usize,
        bytes: Option<Vec<u8>>,
    ) -> bool {
        self.size_bpp = size_bpp;
        (self.size_x, self.size_y, self.size_z) = (size_xyz.x, size_xyz.y, size_xyz.z);
        self.size_row_bytes = if size_row_bytes != 0 {
            size_row_bytes
        } else {
            self.size_x.wrapping_mul(size_bpp)
        };
        self.size_slice_bytes = self.size_row_bytes.wrapping_mul(self.size_y);
        let size = self.size_slice_bytes.wrapping_mul(self.size_z);
        self.bytes = bytes.or_else(|| (size != 0).then(|| vec![0; size]));
        let is_top_down = self.top_to_down == 1;
        self.set_top_down(is_top_down);
        !self.is_empty()
    }
    /// True for a null buffer; `data.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_none()
    }
    /// `Size()`: the buffer size in bytes; `data.size()`.
    pub fn size(&self) -> usize {
        self.bytes.as_ref().map_or(0, Vec::len)
    }
    /// The whole buffer (`Data()`); `data.data()`.
    pub fn data(&self) -> &[u8] {
        self.bytes.as_deref().unwrap_or_default()
    }
    /// The whole buffer, mutable (`ChangeData()`); `data.change_data()`.
    pub fn change_data(&mut self) -> &mut [u8] {
        self.bytes.as_deref_mut().unwrap_or_default()
    }
    /// Gives the buffer back (the wrapped bytes of `InitWrapper`); `data.take_bytes()`.
    pub fn take_bytes(&mut self) -> Option<Vec<u8>> {
        self.bytes.take()
    }
    /// Zeroes the buffer; `data.zero_data()`.
    pub fn zero_data(&mut self) {
        self.change_data().fill(0);
    }
    /// The byte offset of `row` from the top row, natively `SizeRowBytes * row * TopToDown`
    /// in wrapping `size_t` arithmetic reinterpreted as `ptrdiff_t`.
    fn row_offset(&self, row: usize) -> usize {
        self.top_row.wrapping_add(
            self.size_row_bytes
                .wrapping_mul(row)
                .wrapping_mul(self.top_to_down),
        )
    }
    fn slice_offset(&self, slice: usize) -> usize {
        self.size_slice_bytes.wrapping_mul(slice)
    }
    /// The bytes from `offset` for `length`, `None` outside the buffer (native: undefined).
    fn span(&self, offset: usize, length: usize) -> Option<&[u8]> {
        self.bytes
            .as_ref()?
            .get(offset..offset.checked_add(length)?)
    }
    fn span_mut(&mut self, offset: usize, length: usize) -> Option<&mut [u8]> {
        self.bytes
            .as_mut()?
            .get_mut(offset..offset.checked_add(length)?)
    }
    /// The bytes of a row (`Row`), `None` outside the image; `data.row(0)`.
    pub fn row(&self, row: usize) -> Option<&[u8]> {
        self.span(self.row_offset(row), self.size_row_bytes)
    }
    /// The bytes of a row, mutable (`ChangeRow`); `data.change_row(0)`.
    pub fn change_row(&mut self, row: usize) -> Option<&mut [u8]> {
        let offset = self.row_offset(row);
        self.span_mut(offset, self.size_row_bytes)
    }
    fn pixel_offset(&self, row: usize, col: usize, slice: usize) -> usize {
        self.row_offset(row)
            .wrapping_add(self.size_bpp.wrapping_mul(col))
            .wrapping_add(self.slice_offset(slice))
    }
    /// The bytes of a pixel by row and column (`Value`); `data.value(0, 1)`.
    pub fn value(&self, row: usize, col: usize) -> Option<&[u8]> {
        self.span(self.pixel_offset(row, col, 0), self.size_bpp)
    }
    /// The bytes of a pixel, mutable (`ChangeValue`); `data.change_value(0, 1)`.
    pub fn change_value(&mut self, row: usize, col: usize) -> Option<&mut [u8]> {
        let offset = self.pixel_offset(row, col, 0);
        self.span_mut(offset, self.size_bpp)
    }
    /// The bytes of a pixel by X and Y (`ValueXY`); `data.value_xy(1, 0)`.
    pub fn value_xy(&self, x: usize, y: usize) -> Option<&[u8]> {
        self.value(y, x)
    }
    /// The bytes of a pixel by X and Y, mutable (`ChangeValueXY`); `data.change_value_xy(1, 0)`.
    pub fn change_value_xy(&mut self, x: usize, y: usize) -> Option<&mut [u8]> {
        self.change_value(y, x)
    }
    /// The bytes of a slice from its start (`Slice`); `data.slice(1)`.
    pub fn slice(&self, slice: usize) -> Option<&[u8]> {
        self.span(self.slice_offset(slice), self.size_slice_bytes)
    }
    /// The bytes of a slice, mutable (`ChangeSlice`); `data.change_slice(1)`.
    pub fn change_slice(&mut self, slice: usize) -> Option<&mut [u8]> {
        let offset = self.slice_offset(slice);
        self.span_mut(offset, self.size_slice_bytes)
    }
    /// The bytes of a row of a slice (`SliceRow`); `data.slice_row(1, 0)`.
    pub fn slice_row(&self, slice: usize, row: usize) -> Option<&[u8]> {
        self.span(
            self.row_offset(row).wrapping_add(self.slice_offset(slice)),
            self.size_row_bytes,
        )
    }
    /// The bytes of a row of a slice, mutable (`ChangeSliceRow`); `data.change_slice_row(1, 0)`.
    pub fn change_slice_row(&mut self, slice: usize, row: usize) -> Option<&mut [u8]> {
        let offset = self.row_offset(row).wrapping_add(self.slice_offset(slice));
        self.span_mut(offset, self.size_row_bytes)
    }
    /// The bytes of a voxel (`ValueXYZ`); `data.value_xyz(0, 0, 1)`.
    pub fn value_xyz(&self, x: usize, y: usize, z: usize) -> Option<&[u8]> {
        self.span(self.pixel_offset(y, x, z), self.size_bpp)
    }
    /// The bytes of a voxel, mutable (`ChangeValueXYZ`); `data.change_value_xyz(0, 0, 1)`.
    pub fn change_value_xyz(&mut self, x: usize, y: usize, z: usize) -> Option<&mut [u8]> {
        let offset = self.pixel_offset(y, x, z);
        self.span_mut(offset, self.size_bpp)
    }
    /// The largest power of two (up to 16) dividing both the row size and the buffer
    /// address; `data.max_row_aligment_bytes()`.
    pub fn max_row_aligment_bytes(&self) -> usize {
        let address = self.data().as_ptr() as usize;
        let mut alignment = 2;
        while alignment <= 16 {
            if !self.size_row_bytes.is_multiple_of(alignment) || !address.is_multiple_of(alignment)
            {
                return alignment >> 1;
            }
            alignment <<= 1;
        }
        alignment
    }
    /// Sets the row order; a bottom-up image's top row is its last row in memory.
    ///
    /// Example: `data.set_top_down(true)`.
    pub fn set_top_down(&mut self, is_top_down: bool) {
        self.top_to_down = if is_top_down { 1 } else { IMAGE_BOTTOM_UP };
        self.top_row = if self.top_to_down == 1 || self.bytes.is_none() {
            0
        } else {
            self.size_row_bytes
                .wrapping_mul(self.size_y.wrapping_sub(1))
        };
    }
    /// `Image_PixMapTypedData::Init(value)`: fills every pixel of the first slice.
    ///
    /// Example: `data.fill_typed(ImageColorRgba { v: [0, 0, 0, 255] })`.
    pub fn fill_typed<P: ImagePixel>(&mut self, value: P) {
        for row in 0..self.size_y {
            for col in 0..self.size_x {
                self.set_typed(row, col, value);
            }
        }
    }
    /// `Image_PixMapTypedData::Value`: the pixel as `P`, `None` outside the image.
    ///
    /// Example: `data.typed::<u8>(0, 0)`.
    pub fn typed<P: ImagePixel>(&self, row: usize, col: usize) -> Option<P> {
        self.span(self.pixel_offset(row, col, 0), P::SIZE)
            .map(P::read_pixel)
    }
    /// `Image_PixMapTypedData::ChangeValue`: writes a pixel; false outside the image.
    ///
    /// Example: `data.set_typed(0, 0, 255u8)`.
    pub fn set_typed<P: ImagePixel>(&mut self, row: usize, col: usize, value: P) -> bool {
        let offset = self.pixel_offset(row, col, 0);
        self.span_mut(offset, P::SIZE)
            .map(|bytes| value.write_pixel(bytes))
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_up_rows_start_at_the_last_line() {
        let mut data = ImagePixMapData::default();
        assert!(data.init(1, 3, 2, 4, None));
        assert_eq!(
            (data.size_row_bytes, data.size_slice_bytes, data.size()),
            (4, 8, 8)
        );
        data.change_row(0).unwrap()[0] = 7;
        assert_eq!(data.data()[4], 7);
        data.set_top_down(true);
        assert_eq!(data.row(0).unwrap()[0], 0);
        assert_eq!(data.value_xy(0, 1).unwrap()[0], 7);
        assert!(data.row(2).is_none());
    }

    #[test]
    fn typed_access_and_fill() {
        let mut data = ImagePixMapData::default();
        data.set_top_down(true);
        data.init_3d(2, Vec3::new(2, 2, 2), 0, None);
        data.fill_typed(0x0102u16);
        assert_eq!(data.typed::<u16>(1, 1), Some(0x0102));
        assert_eq!(data.value_xyz(0, 0, 1).unwrap(), &[0, 0]);
        assert_eq!(data.slice(1).unwrap().len(), 8);
        assert!(data.max_row_aligment_bytes() >= 2);
        data.zero_data();
        assert_eq!(data.typed::<u16>(0, 0), Some(0));
    }
}
