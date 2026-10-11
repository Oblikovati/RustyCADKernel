//! Retained state, borrowing, runtime identity and safe undefined-native boundaries.
use super::*;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    tcollection::{AsciiString, ExtendedString},
};
use rustycad_tkservice::aspect::{
    AspectTypeOfColorScaleData as ScaleData, AspectTypeOfColorScalePosition as ScalePosition,
};
#[test]
fn scale_defaults_and_runtime_descriptor_match_native_constructor() {
    let scale = AisColorScale::new().unwrap();
    assert_eq!(scale.dynamic_type().unwrap().name(), "AIS_ColorScale");
    assert_eq!(scale.get_range(), (0.0, 1.0));
    assert_eq!(scale.hue_range(), (230.0, 0.0));
    assert_eq!(scale.get_number_of_intervals(), 10);
    assert_eq!(scale.get_size(), (0, 0));
    assert_eq!(scale.get_position(), (0.0, 0.0));
    assert_eq!(scale.get_text_height(), 20);
    assert!(scale.is_label_at_border());
    assert!(!scale.is_reversed() && !scale.is_logarithmic() && !scale.is_smooth_transition());
    assert_eq!(scale.get_label_position(), ScalePosition::Right);
    assert_eq!(scale.get_title_position(), ScalePosition::Left);
    assert_eq!(scale.get_color_type(), ScaleData::Auto);
    assert_eq!(scale.get_label_type(), ScaleData::Auto);
    assert_eq!(scale.get_format().as_bytes(), b"%.4g");
    assert!(scale.get_title().is_empty());
}
#[test]
fn scale_property_updates_preserve_native_ordering_and_interval_guard() {
    let scale = AisColorScale::new().unwrap();
    scale.set_min(4.0);
    assert_eq!(scale.get_range(), (1.0, 4.0));
    scale.set_max(-2.0);
    assert_eq!(scale.get_min(), -2.0);
    assert_eq!(scale.get_max(), 1.0);
}
#[test]
fn scale_interval_guard_and_policy_updates() {
    let scale = AisColorScale::new().unwrap();
    scale.set_number_of_intervals(0);
    assert_eq!(scale.get_number_of_intervals(), 10);
    scale.set_number_of_intervals(3);
    scale.set_label_type(ScaleData::User);
    scale.set_color_type(ScaleData::User);
    scale.set_reversed(true);
    scale.set_logarithmic(true);
    scale.set_label_at_border(false);
    scale.set_smooth_transition(true);
    assert!(scale.is_reversed() && scale.is_logarithmic() && scale.is_smooth_transition());
    assert!(!scale.is_label_at_border());
    assert_eq!(scale.get_number_of_intervals(), 3);
    assert_eq!(scale.get_label_type(), ScaleData::User);
    assert_eq!(scale.get_color_type(), ScaleData::User);
}
#[test]
fn scale_placement_and_text_metrics_keep_retained_pixel_values() {
    let scale = AisColorScale::new().unwrap();
    scale.set_size(80, 210);
    scale.set_position(-3, 7);
    scale.set_breadth(91);
    scale.set_height(242);
    scale.set_x_position(4);
    scale.set_y_position(9);
    assert_eq!(scale.get_size(), (91, 242));
    assert_eq!(scale.get_position(), (4.0, 9.0));
    assert_eq!(
        (
            scale.get_breadth(),
            scale.get_height(),
            scale.get_x_position(),
            scale.get_y_position()
        ),
        (91, 242, 4, 9)
    );
}
#[test]
fn scale_label_title_positions_and_text_height_roundtrip() {
    let scale = AisColorScale::new().unwrap();
    scale.set_text_height(17);
    scale.set_label_position(ScalePosition::Center);
    scale.set_title_position(ScalePosition::Right);
    assert_eq!(scale.get_text_height(), 17);
    assert_eq!(scale.get_label_position(), ScalePosition::Center);
    assert_eq!(scale.get_title_position(), ScalePosition::Right);
}
#[test]
fn scale_no_context_text_metrics() {
    let scale = AisColorScale::new().unwrap();
    let text = ExtendedString::from_utf8("Temperature °C");
    assert_eq!(scale.text_size(&text, 17).unwrap(), [10, 1, 1]);
    assert_eq!(scale.text_width(&text).unwrap(), 10);
    assert_eq!(scale.text_height(&text).unwrap(), 2);
}
#[test]
fn scale_borrowed_and_copied_sequences_preserve_sparse_native_members() {
    let scale = AisColorScale::new().unwrap();
    let title = ExtendedString::from_utf8("Pressure");
    scale.set_title(&title);
    assert_eq!(&*scale.get_title(), &title);
    scale.set_format(&AsciiString::from("%.2f"));
    assert_eq!(scale.format().as_bytes(), b"%.2f");
    scale.set_label(&title, 3);
    scale.set_label(&title, 0);
    let mut labels = vec![];
    scale.get_labels(&mut labels);
    assert_eq!(labels.len(), 4);
    assert!(labels[0].is_empty());
    scale.set_labels(&labels[..2]);
    assert_eq!(scale.labels().len(), 2);
}
#[test]
fn scale_custom_color_sequences_and_invalid_format() {
    let scale = AisColorScale::new().unwrap();
    scale.set_interval_color(Color::named(NamedColor::RED), 3);
    scale.set_interval_color(Color::named(NamedColor::GREEN), -1);
    let mut colors = vec![];
    scale.copy_colors(&mut colors);
    assert_eq!(colors.len(), 4);
    scale.set_colors(&colors[..2]);
    assert_eq!(scale.get_colors().len(), 2);
    scale.set_format(&AsciiString::from("%s"));
    assert!(scale.get_label(1).is_err());
}
#[test]
fn scale_hls_endpoints_and_static_custom_lookup_use_configured_colors() {
    let scale = AisColorScale::new().unwrap();
    let green = Color::named(NamedColor::GREEN);
    let red = Color::named(NamedColor::RED);
    scale.set_color_range(green, red);
    assert_eq!(scale.color_range().unwrap(), (green, red));
}
#[test]
fn scale_hue_normalization_and_custom_static_lookup() {
    let scale = AisColorScale::new().unwrap();
    scale.set_hue_range(-30.0, 720.0);
    assert_eq!(scale.hue_range(), (-30.0, 720.0));
    assert_eq!(AisColorScale::hue_to_valid_range(-30.0), 330.0);
    assert_eq!(AisColorScale::hue_to_valid_range(720.0), 360.0);
}
#[test]
fn scale_static_lookup_hls_endpoints() {
    let green = Color::named(NamedColor::GREEN);
    let mut found = Color::default();
    assert!(
        AisColorScale::find_color_hls(
            0.0,
            0.0,
            1.0,
            5,
            [120.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            &mut found
        )
        .unwrap()
    );
    assert_eq!(found, green);
}
#[test]
fn zero_distance_uniform_palette_returns_defined_error_instead_of_native_loop() {
    assert!(AisColorScale::make_uniform_colors(5, 0.0, 0.0, 360.0).is_err());
}
#[test]
fn scale_context_text_metrics_use_live_driver_and_reject_absent_view() {
    use crate::v3d::V3dViewer;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkservice::{aspect::AspectNeutralWindow, null_driver::NullGraphicDriver};
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let scale = AisColorScale::new().unwrap();
    scale.set_context(&context);
    let text = ExtendedString::from_utf8("Temperature °C");
    assert!(scale.text_size(&text, 17).is_err());
    let view = viewer.create_view().unwrap();
    let window = AspectNeutralWindow::default();
    window.set_size(400, 300);
    view.set_window(&Handle::new(window).cast(), 0).unwrap();
    assert_eq!(scale.text_size(&text, 17).unwrap(), [0, 0, 0]);
}
#[test]
fn scale_native_constructor_appearance_and_runtime_interfaces() {
    use crate::{prs_mgr::PrsMgrPresentableObject, select_mgr::SelectMgrSelectableObject};
    use rustycad_tkernel::standard::RuntimeTyped;
    let scale = AisColorScale::new().unwrap();
    assert!(std::ptr::eq(
        scale.dynamic_type().unwrap(),
        AisColorScale::static_type()
    ));
    assert_eq!(
        scale.dynamic_type().unwrap().parent().unwrap().name(),
        "AIS_InteractiveObject"
    );
    assert!(!scale.cast::<dyn AisInteractiveObject>().is_null());
    assert!(!scale.cast::<dyn PrsMgrPresentableObject>().is_null());
    assert!(!scale.cast::<dyn SelectMgrSelectableObject>().is_null());
}
#[test]
fn scale_native_constructor_shading() {
    use crate::prs_mgr::PrsMgrPresentableObjectMembers;
    use rustycad_tkservice::graphic3d::{
        Graphic3dAlphaMode, Graphic3dAspectsMembers, Graphic3dTypeOfShadingModel,
    };
    let scale = AisColorScale::new().unwrap();
    let shading = scale.attributes().shading_aspect().aspect();
    assert_eq!(shading.shading_model(), Graphic3dTypeOfShadingModel::Unlit);
    assert_eq!(shading.alpha_mode(), Graphic3dAlphaMode::Opaque);
    assert_eq!(shading.interior_color(), Color::named(NamedColor::WHITE));
    assert!(!scale.has_color());
}
