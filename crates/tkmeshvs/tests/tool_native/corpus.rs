//! Each missing key is isolated on a freshly populated original drawer.
use super::{MeshVSDrawer, row};
use martensite_tkmeshvs::MeshVSDrawerAttribute as K;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
    tcollection::AsciiString,
};
pub(super) fn defaults(rows: &mut String) {
    for kind in 0..5 {
        let drawer = Handle::new(MeshVSDrawer::default());
        row(rows, "defaults", kind, 0, &drawer, true);
        if kind != 0 {
            row(rows, "null", kind, 0, &Handle::default(), true);
        }
        row(rows, "empty", kind, 0, &drawer, false);
    }
}
pub(super) fn missing(rows: &mut String) {
    for (kind, keys) in MISSING_KEYS.iter().enumerate() {
        for key in *keys {
            let drawer = populated();
            remove(&drawer, *key);
            row(rows, "missing", kind as i32, *key, &drawer, false);
        }
    }
}
pub(super) fn raw_enums(rows: &mut String) {
    for (group, (kind, key, lower, upper)) in RAW_GROUPS.iter().enumerate() {
        for value in *lower..=*upper {
            let drawer = populated();
            drawer.set_integer(*key, value);
            row(
                rows,
                "raw",
                *kind,
                group as i32 * 100 + value,
                &drawer,
                false,
            );
        }
    }
}
pub(super) fn failures(rows: &mut String) {
    for kind in 0..5 {
        row(rows, "custom", kind, 0, &populated(), false);
    }
    let widths = [0., -1., 1e-300, f64::NAN, f64::INFINITY];
    for kind in 1..4 {
        for (index, width) in widths.iter().enumerate() {
            let drawer = populated();
            let key = match kind {
                1 => K::EdgeWidth,
                2 => K::BeamWidth,
                _ => K::MarkerScale,
            };
            drawer.set_double(key as i32, *width);
            row(rows, "width", kind, index as i32, &drawer, false);
        }
    }
}
fn populated() -> Handle<MeshVSDrawer> {
    let drawer = Handle::new(MeshVSDrawer::default());
    fill(&drawer);
    simple(&drawer);
    text(&drawer);
    drawer
}
fn fill(drawer: &MeshVSDrawer) {
    drawer.set_color(K::InteriorColor as i32, &linear_color(0.125, 0.25, 0.5));
    drawer.set_color(K::BackInteriorColor as i32, &linear_color(0.5, 0.25, 0.125));
    drawer.set_color(K::EdgeColor as i32, &Color::named(NamedColor::RED));
    drawer.set_double(K::EdgeWidth as i32, 2.25);
    for (key, value) in FILL_INTEGERS {
        drawer.set_integer(*key as i32, *value);
    }
}
fn linear_color(red: f64, green: f64, blue: f64) -> Color {
    Color::new(
        red,
        green,
        blue,
        rustycad_tkernel::quantity::ColorSpace::Rgb,
    )
    .unwrap()
}
const FILL_INTEGERS: &[(K, i32)] = &[
    (K::InteriorStyle, 1),
    (K::EdgeType, 2),
    (K::HatchStyle, 3),
    (K::FrontMaterial, 3),
    (K::BackMaterial, 8),
];

fn simple(drawer: &MeshVSDrawer) {
    drawer.set_color(K::BeamColor as i32, &Color::named(NamedColor::BLUE));
    drawer.set_double(K::BeamWidth as i32, 1.75);
    drawer.set_integer(K::BeamType as i32, 1);
    drawer.set_color(K::MarkerColor as i32, &Color::named(NamedColor::GREEN));
    drawer.set_double(K::MarkerScale as i32, 3.25);
    drawer.set_integer(K::MarkerType as i32, 7);
}
fn text(drawer: &MeshVSDrawer) {
    drawer.set_color(K::TextColor as i32, &Color::named(NamedColor::MAGENTA1));
    drawer.set_double(K::TextExpansionFactor as i32, -2.);
    drawer.set_double(K::TextSpace as i32, 3.5);
    drawer.set_ascii_string(K::TextFont as i32, &AsciiString::new("Custom"));
    drawer.set_integer(K::TextStyle as i32, 1);
    drawer.set_integer(K::TextDisplayType as i32, 5);
    drawer.set_integer(K::TextFontAspect as i32, 2);
}
fn remove(drawer: &MeshVSDrawer, key: i32) {
    drawer.remove_integer(key);
    drawer.remove_double(key);
    drawer.remove_color(key);
    drawer.remove_ascii_string(key);
}
const MISSING_KEYS: &[&[i32]] = &[
    &[7, 8],
    &[1, 2, 3, 5, 0, 4, 6],
    &[11, 10, 9],
    &[13, 14, 12],
    &[15, 18, 19, 17, 20, 21, 23],
];
const RAW_GROUPS: &[(i32, i32, i32, i32)] = &[
    (1, 0, -4, 3),
    (1, 4, -8, 7),
    (1, 6, 0, 15),
    (2, 9, -8, 7),
    (3, 12, -16, 15),
    (4, 20, 0, 1),
    (4, 21, 0, 7),
    (4, 23, -4, 3),
];

pub(super) fn font_bytes(rows: &mut String) {
    for (index, bytes) in [b"A\0Z".as_slice(), b"\xffX", b""].iter().enumerate() {
        let drawer = populated();
        drawer.set_ascii_string(K::TextFont as i32, &AsciiString::from_view(bytes));
        row(rows, "fontbytes", 4, index as i32, &drawer, false);
    }
}
pub(super) fn early_exit(rows: &mut String) {
    let keys = [K::InteriorStyle, K::BeamType, K::MarkerType];
    let widths = [K::EdgeWidth, K::BeamWidth, K::MarkerScale];
    for kind in 1..4 {
        let drawer = populated();
        drawer.set_double(widths[kind - 1] as i32, 0.);
        drawer.remove_integer(keys[kind - 1] as i32);
        row(rows, "early", kind as i32, 0, &drawer, false);
    }
}
