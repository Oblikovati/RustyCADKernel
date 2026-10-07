//! `Graphic3d_CubeMapPacked`: a cube map whose six sides are tiles of one image (a 1x6, 6x1,
//! 2x3 or 3x2 grid) or the faces of one DDS file.
use super::cube_map::Graphic3dCubeMapBase;
use super::texture_root::graphic3d_convert_to_compatible;
use super::{
    Graphic3dCubeMap, Graphic3dCubeMapOrder, Graphic3dCubeMapSide, Graphic3dTextureMap,
    Graphic3dTextureRoot, Graphic3dTextureRootBase, Graphic3dValidatedCubeMapOrder,
};
use crate::image::{
    ImageAlienPixMap, ImageCompressedPixMap, ImageDdsParser, ImagePixMap, ImageSupportedFormats,
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// A cube map packed in one image or DDS file.
///
/// Example: `Graphic3dCubeMapPacked::from_pix_map(pixmap, &Graphic3dCubeMapOrder::default_order())`.
pub struct Graphic3dCubeMapPacked {
    transient: StandardTransient,
    base: Graphic3dCubeMapBase,
    order: Graphic3dCubeMapOrder,
    tile_number_x: SharedState<u32>,
}
impl Graphic3dTextureRoot for Graphic3dCubeMapPacked {
    fn texture_root_base(&self) -> &Graphic3dTextureRootBase {
        &self.base.root
    }
}
impl Graphic3dTextureMap for Graphic3dCubeMapPacked {}
rustycad_tkernel::impl_transient!(Graphic3dCubeMapPacked, "Graphic3d_CubeMapPacked", dyn Graphic3dCubeMap, transient,
    interfaces: [dyn Graphic3dTextureRoot, dyn Graphic3dTextureMap, dyn Graphic3dCubeMap]);

/// `checkImage`: the tile grid width of a 1x6, 6x1, 2x3 or 3x2 image, else `None`. A zero
/// width or height divides by zero natively (SIGFPE); here it does not match the 1x6 and 6x1
/// shapes.
fn tile_number_x_of(size_x: usize, size_y: usize) -> Option<u32> {
    let ratio = |a: usize, b: usize| b != 0 && a.is_multiple_of(b) && a / b == 6;
    if ratio(size_y, size_x) {
        Some(1)
    } else if ratio(size_x, size_y) {
        Some(6)
    } else if size_x.is_multiple_of(2) && size_y.is_multiple_of(3) && size_x / 2 == size_y / 3 {
        Some(2)
    } else if size_x.is_multiple_of(3) && size_y.is_multiple_of(2) && size_x / 3 == size_y / 2 {
        Some(3)
    } else {
        None
    }
}

/// `checkOrder`: an order of six distinct locations in 0..=5, else `Standard_Failure`.
///
/// Example: `graphic3d_cube_map_packed_check_order(&[0, 1, 2, 3, 4, 5])?`.
pub fn graphic3d_cube_map_packed_check_order(order: &[u32]) -> Result<bool, Failure> {
    let valid = order.len() == 6
        && order
            .iter()
            .enumerate()
            .all(|(i, value)| *value <= 5 && !order[i + 1..].contains(value));
    if !valid {
        let message =
            format!("Invalid order format in tiles of Graphic3d_CubeMapPacked; got {order:?}");
        return Err(Failure::with_type("Standard_Failure", &message, ""));
    }
    Ok(true)
}

impl Graphic3dCubeMapPacked {
    /// A DDS or image file; `Graphic3dCubeMapPacked::from_file(&"sky.dds".into(), &Graphic3dCubeMapOrder::default_order())`.
    pub fn from_file(file_path: &AsciiString, order: &Graphic3dValidatedCubeMapOrder) -> Self {
        let base = Graphic3dCubeMapBase::from_file(file_path, false);
        Self {
            transient: StandardTransient::default(),
            base,
            order: *order.order(),
            tile_number_x: SharedState::new(1),
        }
    }
    /// An image whose shape gives the tile grid; an image of another shape is dropped.
    /// Example: `Graphic3dCubeMapPacked::from_pix_map(pixmap, &Graphic3dCubeMapOrder::default_order())`.
    pub fn from_pix_map(
        image: Handle<ImagePixMap>,
        order: &Graphic3dValidatedCubeMapOrder,
    ) -> Self {
        let base = Graphic3dCubeMapBase::from_pix_map(Handle::default(), false);
        let texture = Self {
            transient: StandardTransient::default(),
            base,
            order: *order.order(),
            tile_number_x: SharedState::new(1),
        };
        if let Some(tiles) = image
            .get()
            .and_then(|pixmap| tile_number_x_of(pixmap.width(), pixmap.height()))
        {
            *texture.tile_number_x.write() = tiles;
            texture.base.root.fields.write().pix_map = image;
        }
        texture
    }
    /// The side-to-tile order (`myOrder`); `cube.order()`.
    pub fn order(&self) -> Graphic3dCubeMapOrder {
        self.order
    }
    /// The tile grid width (`myTileNumberX`); `cube.tile_number_x()`.
    pub fn tile_number_x(&self) -> u32 {
        *self.tile_number_x.read()
    }
    /// `tryLoadImage`: the decoded file (never without an image library) when its shape fits.
    fn try_load_image(&self, supported: &Handle<ImageSupportedFormats>, file_path: &AsciiString) {
        let image = Handle::new(ImageAlienPixMap::default());
        if !image.load(file_path) {
            return;
        }
        let pixmap: Handle<ImagePixMap> = image.cast();
        let Some(tiles) = pixmap
            .get()
            .and_then(|p| tile_number_x_of(p.width(), p.height()))
        else {
            return;
        };
        *self.tile_number_x.write() = tiles;
        graphic3d_convert_to_compatible(supported, &pixmap);
        self.base.root.fields.write().pix_map = pixmap;
    }
    /// The tile of the current side as a new image. The native wrapper aliases the tile
    /// inside the packed image; here it holds a copy of the tile rows (row stride kept).
    fn tile_wrapper(&self, pixmap: &ImagePixMap, tiles: u32) -> Handle<ImagePixMap> {
        let tile_size = pixmap.width() / tiles as usize;
        let is_top_down = pixmap.is_top_down();
        self.base.root.fields.write().is_top_down = is_top_down;
        let mut order = self.order;
        if !is_top_down {
            pixmap.set_top_down(true);
            order.swap(Graphic3dCubeMapSide::PosY, Graphic3dCubeMapSide::NegY);
        }
        let location = u32::from(order.location_of_side_index(self.base.current_side_index()));
        let tile_x = location % tiles;
        let mut tile_y = location / tiles;
        tile_y = if is_top_down {
            tile_y
        } else {
            (6 / tiles).wrapping_sub(1).wrapping_sub(tile_y)
        };
        let bytes = tile_bytes(
            pixmap,
            tile_y as usize * tile_size,
            tile_x as usize * tile_size,
            tile_size,
        );
        let wrapper = ImagePixMap::default();
        let wrapped = wrapper.init_wrapper(
            pixmap.format(),
            Some(bytes),
            tile_size,
            tile_size,
            pixmap.size_row_bytes(),
        );
        pixmap.set_top_down(is_top_down);
        if wrapped {
            Handle::new(wrapper)
        } else {
            Handle::default()
        }
    }
}

/// The bytes from pixel `(row, col)` of a top-down image over `rows` rows of the image's row
/// stride, zero-padded where the last row runs past the image buffer.
fn tile_bytes(pixmap: &ImagePixMap, row: usize, col: usize, rows: usize) -> Vec<u8> {
    let state = pixmap.data();
    let row_bytes = state.data.size_row_bytes;
    let start = row
        .wrapping_mul(row_bytes)
        .wrapping_add(col.wrapping_mul(state.data.size_bpp));
    let source = state.data.data();
    let mut bytes = vec![0u8; rows.wrapping_mul(row_bytes)];
    let available = source.len().saturating_sub(start).min(bytes.len());
    if available > 0 {
        bytes[..available].copy_from_slice(&source[start..start + available]);
    }
    bytes
}

impl Graphic3dCubeMap for Graphic3dCubeMapPacked {
    fn cube_map_base(&self) -> &Graphic3dCubeMapBase {
        &self.base
    }
    /// The face of the current side's tile index in a 6-face square DDS file, or null.
    fn compressed_value(
        &self,
        supported: &Handle<ImageSupportedFormats>,
    ) -> Handle<ImageCompressedPixMap> {
        let root = &self.base.root;
        if *self.tile_number_x.read() == 0 || root.fields.read().pix_map.is_some() {
            return Handle::default();
        }
        let path = root.fields.read().path.system_name();
        if path.is_empty() {
            return Handle::default();
        }
        let tile = self
            .order
            .location_of_side_index(self.base.current_side_index());
        let image = ImageDdsParser::load(supported, &path, i32::from(tile), 0);
        let Some(compressed) = image
            .get()
            .filter(|c| c.nb_faces() == 6 && c.size_x() == c.size_y())
        else {
            return Handle::default();
        };
        root.fields.write().is_top_down = compressed.is_top_down();
        image
    }
    /// The tile of the current side (loading the file first when needed), or null.
    fn value(&self, supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        let root = &self.base.root;
        if *self.tile_number_x.read() == 0 {
            return Handle::default();
        }
        if root.fields.read().pix_map.is_null() {
            let path = root.fields.read().path.system_name();
            if !path.is_empty() {
                self.try_load_image(supported, &path);
            }
        }
        let image = root.fields.read().pix_map.clone();
        let tiles = *self.tile_number_x.read();
        image
            .get()
            .map_or_else(Handle::default, |pixmap| self.tile_wrapper(pixmap, tiles))
    }
}
