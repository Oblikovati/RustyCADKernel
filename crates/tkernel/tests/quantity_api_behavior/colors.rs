// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

use rustycad_tkernel::{
    ncollection::Vec3,
    quantity::{Color, ColorSpace, NamedColor},
};

fn rgb(r: f64, g: f64, b: f64) -> Color {
    Color::new(r, g, b, ColorSpace::Rgb).expect("valid color")
}

#[test]
fn every_named_color_finds_its_first_table_twin() {
    for name in NamedColor::all() {
        // Duplicated table entries (GRAY0 == BLACK) resolve to the earliest index.
        let nearest = Color::named(name).name();
        assert_eq!(nearest.srgb(), name.srgb(), "{}", name.name());
        assert!(nearest.index() <= name.index(), "{}", name.name());
    }
    assert_eq!(Color::named(NamedColor::GRAY0).name(), NamedColor::BLACK);
    assert_eq!(NamedColor::all().count(), 509);
}

#[test]
fn nearest_name_searches_srgb_and_validates_like_the_constructor() {
    assert_eq!(Color::nearest_name(1.0, 0.0, 0.0), Ok(NamedColor::RED));
    // Linear 0.214041 is sRGB 0.5, nearest to GRAY50 (index 182) rather than a dark gray.
    assert_eq!(rgb(0.214041, 0.214041, 0.214041).name().index(), 182);
    let failure = Color::nearest_name(1.5, 0.0, 0.0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert_eq!(
        Color::nearest_name(f64::NAN, 0.0, 0.0),
        Ok(NamedColor::BLACK)
    );
}

#[test]
fn named_color_indices_and_string_names() {
    assert_eq!(NamedColor::from_index(0), Some(NamedColor::BLACK));
    assert_eq!(NamedColor::from_index(508), Some(NamedColor::WHITE));
    assert_eq!(NamedColor::from_index(509), None);
    assert_eq!(NamedColor::from_index(-1), None);
    assert_eq!(Color::string_name(NamedColor::MATRABLUE), "MATRABLUE");
    assert_eq!(NamedColor::MATRABLUE.srgb(), [0.0, 0.0, 0.75]);
    assert_eq!(NamedColor::BLUE1, NamedColor::BLUE);
    assert_eq!(NamedColor::YELLOW1.name(), "YELLOW");
}

#[test]
fn color_from_name_is_case_insensitive_with_prefix_and_aliases() {
    assert_eq!(NamedColor::from_name("red"), Some(NamedColor::RED));
    assert_eq!(
        NamedColor::from_name("Quantity_NOC_Gray50"),
        Some(NamedColor::GRAY50)
    );
    assert_eq!(NamedColor::from_name("tomato1"), Some(NamedColor::TOMATO));
    assert_eq!(NamedColor::from_name("xQUANTITY_NOC_RED"), None);
    assert_eq!(NamedColor::from_name("QUANTITY_NOC_"), None);
    assert_eq!(NamedColor::from_name("RED\0garbage"), Some(NamedColor::RED));
    assert_eq!(NamedColor::from_name(""), None);
    let color = Color::from_name("blue").expect("known name");
    assert_eq!(color.rgb(), [0.0, 0.0, 1.0]);
    assert!(Color::from_name("BLUE2x").is_none());
}

#[test]
fn vector_constructor_validates_linear_rgb() {
    let color = Color::from_vec3(Vec3::new(0.5, 0.25, 0.125)).expect("in range");
    assert_eq!(color.rgb(), [0.5, 0.25, 0.125]);
    assert_eq!(color.rgb_vec3().data(), [0.5, 0.25, 0.125]);
    let failure = Color::from_vec3(Vec3::new(0.0, -1e-30, 0.0)).unwrap_err();
    assert_eq!(failure.message(), "Color out");
}

#[test]
fn change_intensity_and_contrast_scale_hls_by_percentage() {
    let mut color = rgb(0.2, 0.4, 0.6);
    let before = Color::linear_rgb_to_hls(color.rgb());
    color.change_intensity(-10.0).expect("valid HLS");
    let after = Color::linear_rgb_to_hls(color.rgb());
    assert!((after[1] - before[1] * 0.9).abs() < 1e-6);
    let mut contrast = rgb(0.2, 0.4, 0.6);
    contrast.change_contrast(10.0).expect("valid HLS");
    assert!(Color::linear_rgb_to_hls(contrast.rgb())[2] > before[2]);
    // A result outside [0; 1] leaves the color unchanged.
    let mut white = rgb(1.0, 1.0, 1.0);
    white.change_intensity(50.0).expect("valid HLS");
    assert_eq!(white.rgb(), [1.0, 1.0, 1.0]);
}

#[test]
fn delta_reports_saturation_then_light_differences() {
    let (dc, di) = rgb(1.0, 0.0, 0.0).delta(rgb(0.214041, 0.214041, 0.214041));
    assert_eq!(dc, 1.0);
    assert!((di - 0.5).abs() < 1e-6);
}
