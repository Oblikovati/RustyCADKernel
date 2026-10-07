//! Exact TKService numeric oracle counterpart of tools/oracle/tkservice_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
mod sv_probe;
use sv_probe::{array, aspects, camera, grid, hid, marker, material, pbr, pixel, shader, texture};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkservice_probe.cxx (`probe_cases`); each
/// package adds its operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // Aspect grids (sv_probe/grid.rs)
    ("sv_rgrid", grid::rectangular_case),
    ("sv_cgrid", grid::circular_case),
    // WNT_HIDSpaceMouse (sv_probe/hid.rs)
    ("sv_hid", hid::hid_case),
    // Image_PixMap pixel conversions (sv_probe/pixel.rs)
    ("sv_pixel", pixel::pixel_case),
    ("sv_rawpixel", pixel::raw_pixel_case),
    ("sv_half", pixel::half_case),
    // Graphic3d_PBRMaterial and Graphic3d_BSDF (sv_probe/pbr.rs)
    ("sv_pbr_lut", pbr::lut_case),
    ("sv_pbr_factor", pbr::factor_case),
    ("sv_pbr_bsdf", pbr::bsdf_case),
    ("sv_fresnel", pbr::fresnel_case),
    // Graphic3d_MaterialAspect (sv_probe/material.rs)
    ("sv_material", material::material_case),
    ("sv_material_rank", material::rank_case),
    ("sv_material_edit", material::edit_case),
    // Graphic3d_ShaderObject declarations (sv_probe/shader.rs)
    ("sv_shader_decl", shader::decl_case),
    // Graphic3d textures (sv_probe/texture.rs)
    ("sv_cubemap", texture::cubemap_case),
    ("sv_segment", texture::segment_case),
    // Graphic3d_MarkerImage (sv_probe/marker.rs)
    ("sv_marker", marker::marker_case),
    // Graphic3d_Aspects (sv_probe/aspects.rs)
    ("sv_aspects_dump", aspects::dump_case),
    // Graphic3d_ArrayOfPrimitives (sv_probe/array.rs)
    ("sv_array", array::array_case),
    // Graphic3d_Camera (sv_probe/camera.rs)
    ("sv_camera", camera::camera_case),
    ("sv_camera_fit", camera::fit_case),
    ("sv_camera_interp", camera::interp_case),
];

fn main() {
    for line in io::stdin().lock().lines() {
        let line: String = line.expect("probe input line");
        let mut fields = line.split_whitespace();
        let Some(op) = fields.next() else { continue };
        let values: Vec<f64> = fields
            .map(|field| field.parse().expect("numeric probe field"))
            .collect();
        let Some((_, case)) = PROBE_CASES.iter().find(|(name, _)| *name == op) else {
            eprintln!("Unknown probe operation: {op}");
            std::process::exit(2);
        };
        case(&values);
    }
}
