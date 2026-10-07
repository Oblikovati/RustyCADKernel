//! Graphic3d textures, the twin of tools/oracle/tkservice_probe_texture.cxx.
use super::common::{Reader, end, flag, int};
use super::pbr::vec4;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{
    Graphic3dCubeMap, Graphic3dCubeMapMembers, Graphic3dCubeMapOrder, Graphic3dCubeMapPacked,
    Graphic3dTexture1Dsegment, Graphic3dTextureRootMembers,
};
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};

/// The image format of a corpus code: 0 Gray, 1 RGB, 2 RGBA, 3 BGR.
fn format(code: i32) -> ImageFormat {
    [
        ImageFormat::Gray,
        ImageFormat::Rgb,
        ImageFormat::Rgba,
        ImageFormat::Bgr,
    ][code as usize]
}

/// The patterned image of a request.
fn patterned(size_x: usize, size_y: usize, code: i32, top_down: bool) -> Handle<ImagePixMap> {
    let image = ImagePixMap::default();
    image.set_top_down(top_down);
    image.init_trash(format(code), size_x, size_y, 0);
    for (i, byte) in image
        .change_data()
        .data
        .change_data()
        .iter_mut()
        .enumerate()
    {
        *byte = ((i * 31 + 7) & 255) as u8;
    }
    Handle::new(image)
}

/// Prints one tile: size, row order and pixel bytes.
fn print_tile(tile: &ImagePixMap) {
    int(tile.width() as i32);
    int(tile.height() as i32);
    flag(tile.is_top_down());
    let state = tile.data();
    for row in 0..tile.height() {
        for col in 0..tile.width() {
            let pixel = state.data.value(row, col).unwrap_or_default();
            pixel.iter().for_each(|byte| int(i32::from(*byte)));
        }
    }
}

/// `sv_cubemap SizeX SizeY Format TopDown O0..O5`.
pub fn cubemap_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (sx, sy, code, top_down) = (
        r.integer() as usize,
        r.integer() as usize,
        r.integer(),
        r.flag(),
    );
    let o: Vec<u8> = (0..6).map(|_| r.integer() as u8).collect();
    let image = patterned(sx, sy, code, top_down);
    match Graphic3dCubeMapOrder::new(o[0], o[1], o[2], o[3], o[4], o[5]).validated() {
        Err(error) => print!("{} ", error.exception_type()),
        Ok(order) => {
            let cube = Graphic3dCubeMapPacked::from_pix_map(image.clone(), &order);
            cube.reset();
            while cube.more() {
                let tile = cube.value(&Handle::default());
                flag(tile.is_some());
                if let Some(pixels) = tile.get() {
                    print_tile(pixels);
                }
                cube.next();
            }
            flag(cube.is_top_down());
            flag(image.is_top_down());
        }
    }
    end();
}

/// `sv_segment X1 Y1 Z1 X2 Y2 Z2`.
pub fn segment_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let v: Vec<f32> = (0..6).map(|_| r.real() as f32).collect();
    let texture = Graphic3dTexture1Dsegment::from_file(&AsciiString::from("ramp.png"));
    texture.set_segment(v[0], v[1], v[2], v[3], v[4], v[5]);
    let params = texture.get_params();
    vec4(params.gen_plane_s());
    vec4(params.gen_plane_t());
    int(params.gen_mode() as i32);
    end();
}
