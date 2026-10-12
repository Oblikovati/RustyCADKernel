//! Native corpus mutations stay identical to the C++ oracle, including no-op and custom-state cases.
use super::*;
use crate::prs_mgr::PrsMgrPresentableObject;
use rustycad_tkernel::{
    quantity::{Color, ColorSpace},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::*;
pub(super) fn apply_appearance_mutation(cube: &AisViewCube, id: i32, delta: f64) {
    if id < 12 {
        mutate_appearance_dimensions(cube, id, delta);
    } else if id < 20 {
        mutate_appearance_labels(cube, id);
    } else {
        mutate_appearance_style(cube, id);
    }
}
fn mutate_appearance_dimensions(cube: &AisViewCube, id: i32, delta: f64) {
    match id {
        0 => cube.set_size(70.0 + delta, false),
        1 => cube.set_size(90.0 + delta, true),
        2 => cube.set_box_facet_extension(10.5 + delta),
        3 => cube.set_axes_padding(7.0 + delta),
        4 => cube.set_box_edge_gap(delta),
        5 => cube.set_box_edge_min_size(2.0 + delta),
        6 => cube.set_box_corner_min_size(2.0 + delta),
        7 => cube.set_round_radius(delta).unwrap(),
        8 => cube.set_axes_radius(1.0 + delta),
        9 => cube.set_axes_cone_radius(3.0 + delta),
        10 => cube.set_axes_sphere_radius(4.0 + delta),
        11 => cube.set_font_height(11.2 + delta),
        _ => panic!("dimension mutation={id}; expected 0..=11"),
    }
}
fn mutate_appearance_labels(cube: &AisViewCube, id: i32) {
    match id {
        12 => cube.set_draw_axes(false),
        13 => cube.set_draw_edges(false),
        14 => cube.set_draw_vertices(false),
        15 => cube.set_y_up(true, true),
        16 => cube.set_y_up(true, false),
        17 => mutate_cube_roof_label(cube),
        18 => cube.set_axes_labels(
            &AsciiString::from("u"),
            &AsciiString::from("v"),
            &AsciiString::from("w"),
        ),
        19 => cube.set_font(&AsciiString::from("mono-test")),
        _ => panic!("label mutation={id}; expected 12..=19"),
    }
}
fn mutate_appearance_style(cube: &AisViewCube, id: i32) {
    let color = Color::new(0.2, 0.4, 0.6, ColorSpace::Rgb).unwrap();
    match id {
        20 => cube.set_box_color(color).unwrap(),
        21 => cube.set_box_transparency(0.3).unwrap(),
        22 => cube.set_inner_color(color).unwrap(),
        23 => cube.set_text_color(color),
        24 => cube.set_color(color).unwrap(),
        25 => cube.unset_color().unwrap(),
        26 => cube.set_transparency(0.3).unwrap(),
        27 => cube.unset_transparency().unwrap(),
        28 => cube
            .set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))
            .unwrap(),
        29 => cube.unset_material().unwrap(),
        30..=34 => mutate_appearance_resets(cube, id),
        _ => panic!("style mutation={id}; expected 20..=34"),
    }
}
pub(super) fn prepare_custom_appearance(cube: &AisViewCube) {
    cube.set_y_up(true, false);
    cube.set_box_side_label(
        crate::v3d::V3dTypeOfOrientation::Zpos,
        &AsciiString::from("custom-top"),
    )
    .unwrap();
    cube.set_axes_labels(
        &AsciiString::from("custom-x"),
        &AsciiString::from("custom-y"),
        &AsciiString::from("custom-z"),
    );
    prepare_custom_cube_dimensions(cube);
    cube.set_box_color(Color::new(0.7, 0.2, 0.1, ColorSpace::Rgb).unwrap())
        .unwrap();
    cube.set_box_transparency(0.45).unwrap();
    cube.set_inner_color(Color::new(0.1, 0.8, 0.2, ColorSpace::Rgb).unwrap())
        .unwrap();
    cube.set_text_color(Color::new(0.8, 0.1, 0.4, ColorSpace::Rgb).unwrap());
}

fn prepare_custom_cube_dimensions(cube: &AisViewCube) {
    cube.set_axes_radius(2.4);
    cube.set_axes_cone_radius(4.2);
    cube.set_axes_sphere_radius(5.3);
    cube.set_size(100.0, false);
    cube.set_box_facet_extension(-2.0);
    cube.set_axes_padding(0.0);
    cube.set_box_edge_min_size(5.0);
    cube.set_box_corner_min_size(6.0);
    cube.set_box_edge_gap(2.0);
    cube.set_round_radius(0.3).unwrap();
    cube.set_draw_axes(false);
    cube.set_draw_edges(false);
    cube.set_draw_vertices(false);
    cube.set_font(&AsciiString::from("custom-font"));
    cube.set_font_height(23.0);
}

fn mutate_appearance_resets(cube: &AisViewCube, id: i32) {
    match id {
        30 => cube.unset_attributes(),
        31 => cube.unset_hilight_attributes(),
        32 => cube.reset_styles().unwrap(),
        33 => ais_view_cube_default_attributes(cube).unwrap(),
        34 => ais_view_cube_default_highlight_attributes(cube).unwrap(),
        _ => panic!("reset mutation={id}; expected 30..=34"),
    }
}

fn mutate_cube_roof_label(cube: &AisViewCube) {
    cube.set_box_side_label(
        crate::v3d::V3dTypeOfOrientation::Zpos,
        &AsciiString::from("roof"),
    )
    .unwrap();
}
