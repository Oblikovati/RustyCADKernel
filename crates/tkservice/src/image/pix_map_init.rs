//! The initialisation and layout members of `Image_PixMap`.
use super::{ImageFormat, ImagePixMap};
use rustycad_tkernel::ncollection::Vec3;

fn has_zero_size(size: Vec3<usize>) -> bool {
    size.x == 0 || size.y == 0 || size.z == 0
}

impl ImagePixMap {
    /// `InitWrapper`: one slice over the caller's bytes; see [`Self::init_wrapper_3d`].
    ///
    /// Example: `pixmap.init_wrapper(ImageFormat::Gray, Some(vec![0; 4]), 2, 2, 0)`.
    pub fn init_wrapper(
        &self,
        format: ImageFormat,
        bytes: Option<Vec<u8>>,
        size_x: usize,
        size_y: usize,
        size_row_bytes: usize,
    ) -> bool {
        self.init_wrapper_3d(format, bytes, Vec3::new(size_x, size_y, 1), size_row_bytes)
    }
    /// `InitWrapper3D`: clears, sets the format, then lays the image over `bytes`; false
    /// for a zero size or no bytes. The port adopts the bytes (`take_bytes` gives them back)
    /// where the native image aliases the caller's memory.
    ///
    /// Example: `pixmap.init_wrapper_3d(ImageFormat::Gray, Some(bytes), Vec3::new(2, 2, 1), 0)`.
    pub fn init_wrapper_3d(
        &self,
        format: ImageFormat,
        bytes: Option<Vec<u8>>,
        size: Vec3<usize>,
        size_row_bytes: usize,
    ) -> bool {
        self.clear();
        let mut state = self.change_data();
        state.format = format;
        let Some(bytes) = bytes.filter(|_| !has_zero_size(size)) else {
            return false;
        };
        state.data.init_3d(
            Self::size_pixel_bytes_of(format),
            size,
            size_row_bytes,
            Some(bytes),
        );
        true
    }
    /// `InitTrash`: one slice of new storage; see [`Self::init_trash_3d`].
    ///
    /// Example: `pixmap.init_trash(ImageFormat::Rgba, 4, 4, 0)`.
    pub fn init_trash(
        &self,
        format: ImageFormat,
        size_x: usize,
        size_y: usize,
        size_row_bytes: usize,
    ) -> bool {
        self.init_trash_3d(format, Vec3::new(size_x, size_y, 1), size_row_bytes)
    }
    /// `InitTrash3D`: clears, sets the format and allocates the image; rows are at least
    /// `SizeX * bpp` bytes. The native storage is uninitialised; the port's is zeroed.
    ///
    /// Example: `pixmap.init_trash_3d(ImageFormat::Gray, Vec3::new(2, 2, 2), 0)`.
    pub fn init_trash_3d(
        &self,
        format: ImageFormat,
        size: Vec3<usize>,
        size_row_bytes: usize,
    ) -> bool {
        self.clear();
        self.init_trash_after_clear(format, size, size_row_bytes)
    }
    pub(super) fn init_trash_after_clear(
        &self,
        format: ImageFormat,
        size: Vec3<usize>,
        size_row_bytes: usize,
    ) -> bool {
        // Reuse the current allocation body after a derived clear callback has returned.
        let mut state = self.change_data();
        state.format = format;
        if has_zero_size(size) {
            return false;
        }
        let bpp = Self::size_pixel_bytes_of(format);
        let row_bytes = size_row_bytes.max(size.x.wrapping_mul(bpp));
        state.data.init_3d(bpp, size, row_bytes, None);
        !state.data.is_empty()
    }
    /// `InitZero`: one slice filled with `value`; `pixmap.init_zero(ImageFormat::Gray, 2, 2, 0, 0)`.
    pub fn init_zero(
        &self,
        format: ImageFormat,
        size_x: usize,
        size_y: usize,
        size_row_bytes: usize,
        value: u8,
    ) -> bool {
        self.init_zero_3d(format, Vec3::new(size_x, size_y, 1), size_row_bytes, value)
    }
    /// `InitZero3D`: allocates (as `InitTrash`) and fills every byte with `value`.
    ///
    /// Example: `pixmap.init_zero_3d(ImageFormat::Gray, Vec3::new(2, 2, 2), 0, 255)`.
    pub fn init_zero_3d(
        &self,
        format: ImageFormat,
        size: Vec3<usize>,
        size_row_bytes: usize,
        value: u8,
    ) -> bool {
        if !self.init_trash_3d(format, size, size_row_bytes) {
            return false;
        }
        self.change_data().data.change_data().fill(value);
        true
    }
    /// `InitCopy`: a copy of another image's layout and bytes; false for `self` or an
    /// empty source. Example: `pixmap.init_copy(&other)`.
    pub fn init_copy(&self, copy: &ImagePixMap) -> bool {
        if std::ptr::eq(self, copy) {
            return false;
        }
        let source = copy.data().clone();
        let size = Vec3::new(source.data.size_x, source.data.size_y, source.data.size_z);
        if !self.init_trash_3d(source.format, size, source.data.size_row_bytes) {
            return false;
        }
        let mut state = self.change_data();
        let count = source.data.size();
        state.data.change_data()[..count].copy_from_slice(source.data.data());
        true
    }
    /// `Clear`: releases the buffer, keeping the format and the row order.
    ///
    /// Example: `pixmap.clear()`.
    pub fn clear(&self) {
        let mut state = self.change_data();
        let bpp = Self::size_pixel_bytes_of(state.format);
        state.data.take_bytes();
        state.data.init_3d(bpp, Vec3::new(0, 0, 0), 0, None);
    }
    /// True for top-down rows; `pixmap.is_top_down()`.
    pub fn is_top_down(&self) -> bool {
        self.data().data.top_to_down == 1
    }
    /// Sets the row order; `pixmap.set_top_down(true)`.
    pub fn set_top_down(&self, is_top_down: bool) {
        self.change_data().data.set_top_down(is_top_down);
    }
    /// `TopDownInc`: 1, or `size_t(-1)` for bottom-up rows; `pixmap.top_down_inc()`.
    pub fn top_down_inc(&self) -> usize {
        self.data().data.top_to_down
    }
    /// Bytes per pixel; `pixmap.size_pixel_bytes()`.
    pub fn size_pixel_bytes(&self) -> usize {
        self.data().data.size_bpp
    }
    /// Bytes per row; `pixmap.size_row_bytes()`.
    pub fn size_row_bytes(&self) -> usize {
        self.data().data.size_row_bytes
    }
    /// Padding bytes at the end of each row; `pixmap.row_extra_bytes()`.
    pub fn row_extra_bytes(&self) -> usize {
        let data = &self.data().data;
        data.size_row_bytes
            .wrapping_sub(data.size_x.wrapping_mul(data.size_bpp))
    }
    /// The row alignment; `pixmap.max_row_aligment_bytes()`.
    pub fn max_row_aligment_bytes(&self) -> usize {
        self.data().data.max_row_aligment_bytes()
    }
    /// Bytes per slice; `pixmap.size_slice_bytes()`.
    pub fn size_slice_bytes(&self) -> usize {
        self.data().data.size_slice_bytes
    }
    /// The buffer size; `pixmap.size_bytes()`.
    pub fn size_bytes(&self) -> usize {
        self.data().data.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_members_follow_the_native_layout() {
        let pixmap = ImagePixMap::default();
        assert!(!pixmap.init_trash(ImageFormat::Rgb, 0, 4, 0));
        assert_eq!(pixmap.format(), ImageFormat::Rgb);
        assert!(pixmap.init_trash(ImageFormat::Rgb, 3, 2, 12));
        assert_eq!(
            (
                pixmap.size_row_bytes(),
                pixmap.row_extra_bytes(),
                pixmap.size_bytes()
            ),
            (12, 3, 24)
        );
        assert!(pixmap.init_zero_3d(ImageFormat::Gray, Vec3::new(2, 2, 2), 0, 9));
        assert_eq!(pixmap.data().data.data(), &[9; 8]);
        let copy = ImagePixMap::default();
        assert!(copy.init_copy(&pixmap) && !pixmap.init_copy(&pixmap));
        assert_eq!(copy.size_xyz(), Vec3::new(2, 2, 2));
        assert!(copy.init_wrapper(ImageFormat::Gray, Some(vec![1, 2, 3, 4]), 2, 2, 0));
        copy.set_top_down(true);
        assert_eq!(copy.data().data.value_xy(1, 1), Some(&[4u8][..]));
        copy.clear();
        assert!(copy.is_empty() && copy.is_top_down());
    }
}
