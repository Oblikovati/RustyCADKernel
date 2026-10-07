//! Graphic3d_RenderingParams, Graphic3d_ZLayerSettings and Graphic3d_PresentationAttributes,
//! the twin of tools/oracle/tkservice_probe_params.cxx.
use super::common::{Reader, end, flag, real, text};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::AspectTypeOfHighlightMethod;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectFillArea3d, Graphic3dPresentationAttributes,
    Graphic3dPresentationAttributesMembers, Graphic3dPresentationAttributesSource,
    Graphic3dRenderingParams, Graphic3dZLayerSettings,
};
use std::io;

fn dumped(write: impl FnOnce(&mut DumpStream) -> io::Result<()>) {
    dumped_masked(write, &[]);
}

/// The dump with the object of each `"Key": {...}` replaced by `?`, as `sv_masked`.
fn dumped_masked(write: impl FnOnce(&mut DumpStream) -> io::Result<()>, keys: &[&str]) {
    let mut stream = DumpStream::default();
    if let Err(error) = write(&mut stream) {
        return print!("dump-error:{error} ");
    }
    let mut dump = stream.bytes().to_vec();
    for key in keys {
        dump = masked(&dump, key);
    }
    text(&dump);
}

/// The end (exclusive) of the object that opens at `open`.
fn object_end(text: &[u8], open: usize) -> usize {
    let (mut end, mut depth) = (open, 0);
    loop {
        depth += match text[end] {
            b'{' => 1,
            b'}' => -1,
            _ => 0,
        };
        end += 1;
        if depth == 0 || end >= text.len() {
            return end;
        }
    }
}

pub fn masked(text: &[u8], key: &str) -> Vec<u8> {
    let head = format!("\"{key}\": {{").into_bytes();
    let mut out = text.to_vec();
    let mut from = 0;
    while let Some(found) = out[from..]
        .windows(head.len())
        .position(|w| w == head.as_slice())
    {
        let open = from + found + head.len() - 1;
        let end = object_end(&out, open);
        out.splice(open..end, *b"?");
        from += found + 1;
    }
    out
}

fn rendering(r: &mut Reader<'_>) -> Result<(), Failure> {
    let mut params = Graphic3dRenderingParams::new()?;
    params.resolution = r.integer() as u32;
    real(f64::from(params.resolution_ratio()));
    dumped_masked(
        |s| params.dump_json(s, -1),
        &["ChartSize", "Params.Params3d"],
    );
    Ok(())
}

fn layer(r: &mut Reader<'_>) {
    let mut s = Graphic3dZLayerSettings::default();
    s.set_name(&AsciiString::from("layer"));
    s.set_origin(r.xyz());
    s.set_culling_distance(r.real());
    s.set_culling_size(r.real());
    match r.integer() {
        1 => s.set_depth_offset_positive(),
        2 => s.set_depth_offset_negative(),
        _ => {}
    }
    s.set_immediate(r.flag());
    s.set_raytracable(r.flag());
    s.set_environment_texture(r.flag());
    s.set_enable_depth_test(r.flag());
    s.set_enable_depth_write(r.flag());
    s.set_clear_depth(r.flag());
    s.set_render_in_depth_prepass(r.flag());
    flag(s.has_culling_distance());
    flag(s.has_culling_size());
    flag(s.origin_transformation().is_null());
    dumped(|stream| s.dump_json(stream, -1));
}

fn attributes(r: &mut Reader<'_>) -> Result<(), Failure> {
    let a = Graphic3dPresentationAttributes::default();
    a.set_method(if r.integer() == 0 {
        AspectTypeOfHighlightMethod::Color
    } else {
        AspectTypeOfHighlightMethod::Boundbox
    });
    let c = r.xyz();
    a.set_color(Color::new(c.x(), c.y(), c.z(), ColorSpace::Rgb)?);
    a.set_transparency(r.real() as f32);
    a.set_z_layer(r.integer());
    a.set_display_mode(r.integer());
    if r.flag() {
        a.set_basic_fill_area_aspect(Handle::new(Graphic3dAspectFillArea3d::default()));
    }
    real(f64::from(a.transparency()));
    real(f64::from(a.color_rgba().alpha()));
    dumped(|s| a.dump_json(s, -1));
    Ok(())
}

/// `sv_params`: the rendering parameters, a Z layer or presentation attributes, with dumps.
pub fn params_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let result = match r.integer() {
        0 => rendering(&mut r),
        1 => {
            layer(&mut r);
            Ok(())
        }
        _ => attributes(&mut r),
    };
    super::tried(result);
    end();
}
