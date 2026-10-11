//! Native annotation state, presentation ordering and measured sensitivity.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
    tcollection::ExtendedString,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfDisplayText, font::*, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn attached_label() -> (
    Handle<AisTextLabel>,
    Handle<AisInteractiveContext>,
    display_driver::DisplayTrace,
) {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let label = AisTextLabel::new();
    label.set_context(&context);
    label.set_font("DejaVu Sans");
    label.set_text(&ExtendedString::new("AV"));
    (label, context, trace)
}
fn label_selection(label: &AisTextLabel, mode: i32) -> Handle<SelectMgrSelection> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    label.compute_selection(&selection, mode).unwrap();
    selection
}
#[test]
fn annotation_constructor_and_retained_setters_preserve_native_identity() {
    let label = AisTextLabel::new();
    assert_eq!(label.text(), ExtendedString::new("?"));
    assert!(!label.has_orientation_3d());
    assert!(!label.has_flipping());
    assert!(label.has_own_anchor_point());
    assert!(label.text_formatter().is_null());
    assert_eq!(label.kind_of_interactive(), AisKindOfInteractive::None);
    assert_eq!(label.signature(), -1);
    assert!(label.accept_display_mode(0));
    assert!(!label.accept_display_mode(1));
    let axes = Axis2::new(Point3::new(2., 3., 4.), Direction3::Y).unwrap();
    label.set_orientation_3d(axes);
    label.set_flipping(true);
    label.set_own_anchor_point(false);
    label.set_text(&ExtendedString::new("text"));
    let formatter = Handle::new(FontTextFormatter::default());
    label.set_text_formatter(&formatter);
    assert_eq!(label.text_formatter(), formatter);
    assert_eq!(label.position(), axes.location());
    assert_eq!(label.orientation_3d().direction(), Direction3::Y);
    label.unset_orientation_3d();
    label.set_position(Point3::new(5., 6., 7.));
    assert!(!label.has_orientation_3d());
    assert!(label.has_flipping());
    assert_eq!(label.position(), Point3::new(5., 6., 7.));
}
#[test]
fn label_appearance_converts_radians_preserves_alpha_and_ignores_material() {
    let label = AisTextLabel::new();
    let aspect = label.attributes().text_aspect().aspect();
    label.set_height(23.);
    label.set_font("DejaVu Sans");
    label.set_font_aspect(FontFontAspect::Bold);
    label.set_angle(std::f64::consts::FRAC_PI_2);
    label.set_zoomable(true);
    label.set_h_justification(Graphic3dHorizontalTextAlignment::Right);
    label.set_v_justification(Graphic3dVerticalTextAlignment::Top);
    label.set_display_type(AspectTypeOfDisplayText::Subtitle);
    label.set_color(Color::named(NamedColor::RED)).unwrap();
    label.set_color_subtitle(Color::named(NamedColor::GREEN));
    label.set_transparency(0.25).unwrap();
    assert_eq!(label.font_name().as_bytes(), b"DejaVu Sans");
    assert_eq!(label.font_aspect(), FontFontAspect::Bold);
    assert_eq!(aspect.text_angle(), 90.);
    assert!(aspect.is_text_zoomable());
    assert_eq!(
        aspect.text_display_type(),
        AspectTypeOfDisplayText::Subtitle
    );
    assert_eq!(aspect.interior_color_rgba().alpha(), 0.75);
    assert_eq!(aspect.edge_color_rgba().alpha(), 0.75);
    assert!(label.has_color());
    assert_eq!(label.attributes().text_aspect().height(), 23.);
    label
        .set_material(&Graphic3dMaterialAspect::default())
        .unwrap();
    assert!(!label.has_material());
    label.unset_transparency().unwrap();
    assert_eq!(aspect.interior_color_rgba().alpha(), 1.);
    assert_eq!(aspect.edge_color_rgba().alpha(), 1.);
}
#[test]
fn measured_label_rectangle_tracks_alignment_rotation_and_selection_snapshot() {
    let (label, _context, _trace) = attached_label();
    label.set_position(Point3::new(2., 3., 4.));
    let (center, width, height) = label
        .calculate_label_params(label.position())
        .unwrap()
        .unwrap();
    assert!(width > 0. && height > 0.);
    assert_eq!(center, Point3::new(2. + width / 2., 3. + height / 2., 4.));
    let selection = label_selection(&label, 0);
    let old = selection.entities()[0].base_sensitive();
    assert_eq!(old.owner_id().priority(), 10);
    assert_eq!(
        old.owner_id().selectable().object_address(),
        label.object_address()
    );
    let rectangle = old.cast::<Select3DSensitiveFace>().get_points();
    assert_eq!(rectangle.len(), 5);
    assert_eq!(rectangle[0], rectangle[4]);
    label.set_angle(std::f64::consts::FRAC_PI_2);
    let rotated = label_selection(&label, 0).entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveFace>()
        .get_points();
    assert_ne!(rectangle, rotated);
    assert_eq!(old.cast::<Select3DSensitiveFace>().get_points(), rectangle);
    assert!(label_selection(&label, 1).entities().is_empty());
    label.set_h_justification(Graphic3dHorizontalTextAlignment::Center);
    label.set_v_justification(Graphic3dVerticalTextAlignment::Center);
    assert_eq!(
        label
            .calculate_label_params(label.position())
            .unwrap()
            .unwrap()
            .0,
        label.position()
    );
}
#[test]
fn label_presentation_sets_persistence_brackets_and_formatter_then_selection_origin() {
    let (label, context, trace) = attached_label();
    label.set_position(Point3::new(2., 3., 4.));
    label.set_orientation_3d(Axis2::new(label.position(), Direction3::Z).unwrap());
    label.set_flipping(true);
    let formatter = Handle::new(FontTextFormatter::default());
    label.set_text_formatter(&formatter);
    let manager = context.main_prs_mgr().clone();
    let prs = manager
        .presentation(&label.cast(), 0, true, &Handle::default())
        .unwrap();
    assert_eq!(
        label.transform_persistence().mode(),
        Graphic3dTransModeFlags::ZOOM_PERS
    );
    assert_eq!(
        label.transform_persistence().anchor_point().unwrap(),
        label.position()
    );
    assert_eq!(trace.read().texts.len(), 1);
    let text = trace.read().texts[0].clone();
    assert!(text.has_plane());
    assert!(!text.has_own_anchor_point());
    assert_eq!(text.position(), Point3::default());
    assert_eq!(text.text_formatter(), formatter);
    let events = trace.read().events.clone();
    let begin = events.iter().position(|v| *v == 5.).unwrap();
    assert_eq!(events[begin + 1], 1.);
    assert_eq!(events[events.len() - 4], 0.);
    assert!(prs.groups()[0].bounding_box().is_valid());
    let face = label_selection(&label, 0).entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveFace>();
    assert_eq!(face.get_points()[0], Point3::default());
}
#[test]
fn invalid_font_size_falls_back_to_a_point_and_keeps_unclosed_flip_bracket() {
    let (label, context, trace) = attached_label();
    label.set_height(0.);
    label.set_orientation_3d(Axis2::default());
    label.set_flipping(true);
    assert!(
        label
            .calculate_label_params(Point3::default())
            .unwrap()
            .is_none()
    );
    let selection = label_selection(&label, 0);
    assert!(
        !selection.entities()[0]
            .base_sensitive()
            .cast::<Select3DSensitivePoint>()
            .is_null()
    );
    context
        .main_prs_mgr()
        .presentation(&label.cast(), 0, true, &Handle::default())
        .unwrap();
    assert_eq!(trace.read().events.iter().filter(|v| **v == 5.).count(), 1);
}
#[test]
fn measurement_requires_a_live_context_instead_of_dereferencing_null() {
    let label = AisTextLabel::new();
    assert!(
        label
            .calculate_label_params(Point3::default())
            .unwrap_err()
            .to_string()
            .contains("context is null")
    );
}
#[test]
fn screen_label_keeps_explicit_2d_persistence_until_zoomable() {
    let (label, context, trace) = attached_label();
    let pers = Handle::new(
        Graphic3dTransformPers::with_corner(
            Graphic3dTransModeFlags::TMF_2D,
            rustycad_tkservice::aspect::AspectTypeOfTriedronPosition::LEFT_LOWER,
            rustycad_tkernel::ncollection::Vec2::new(4, 5),
        )
        .unwrap(),
    );
    label.set_position(Point3::new(2., 3., 4.));
    label.set_transform_persistence(&pers.cast());
    context.display(&label.cast(), false).unwrap();
    assert_eq!(
        label.transform_persistence().mode(),
        Graphic3dTransModeFlags::TMF_2D
    );
    assert_eq!(trace.read().texts[0].position(), label.position());
    label.set_zoomable(true);
    context.redisplay(&label.cast(), false, false).unwrap();
    assert_eq!(
        label.transform_persistence().mode(),
        Graphic3dTransModeFlags::ROTATE_PERS
    );
    assert_eq!(
        trace.read().texts.last().unwrap().position(),
        Point3::default()
    );
}
