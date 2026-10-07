//! Graphic3d_Text: the text value, orientation and alignment members.
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::font::FontTextFormatter;
use rustycad_tkservice::graphic3d::{
    Graphic3dHorizontalTextAlignment, Graphic3dText, Graphic3dVerticalTextAlignment,
};

fn utf8(text: &String8) -> Vec<u8> {
    let units = text.to_c_string();
    units[..units.iter().position(|&b| b == 0).unwrap_or(units.len())].to_vec()
}

#[test]
fn defaults_and_setters() {
    let text = Graphic3dText::new(16.0);
    assert_eq!(text.height(), 16.0);
    assert_eq!(
        text.horizontal_alignment(),
        Graphic3dHorizontalTextAlignment::Left
    );
    assert_eq!(
        text.vertical_alignment(),
        Graphic3dVerticalTextAlignment::Bottom
    );
    assert!(!text.has_plane() && text.has_own_anchor_point());
    text.set_text_str("label");
    assert_eq!(utf8(&text.text()), b"label");
    text.set_text_ascii(&AsciiString::from("other"));
    assert_eq!(utf8(&text.text()), b"other");
    text.set_height(20.0);
    text.set_horizontal_alignment(Graphic3dHorizontalTextAlignment::Center);
    text.set_vertical_alignment(Graphic3dVerticalTextAlignment::Top);
    text.set_own_anchor_point(false);
    assert_eq!(text.height(), 20.0);
    assert!(!text.has_own_anchor_point());
    text.set_text_formatter(Handle::new(FontTextFormatter::default()));
    assert!(text.text_formatter().is_some());
}

#[test]
fn orientation_and_position() {
    let text = Graphic3dText::new(10.0);
    text.set_position(&Point3::new(1.0, 2.0, 3.0));
    assert_eq!(text.position().z(), 3.0);
    let axis = Axis2::new(
        Point3::new(4.0, 5.0, 6.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    )
    .unwrap();
    text.set_orientation(&axis);
    assert!(text.has_plane());
    assert_eq!(text.position().x(), 4.0);
    text.reset_orientation();
    assert!(!text.has_plane());
    assert_eq!(text.position().x(), 0.0);
}
