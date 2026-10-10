//! Both native overloads retain the supplied aspects and emit actual label objects.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
use rustycad_tkv3d::{dsg_prs::DsgPrsXYZAxisPresentation, prs3d::*, v3d::V3dViewer};
#[test]
fn axis_overloads_keep_labels_and_apply_text_aspect_even_for_empty_text() {
    for styled in [false, true] {
        for label in ["X", "", "Y\0ignored"] {
            check_axis_presentation(styled, label);
        }
    }
}
fn check_axis_presentation(styled: bool, label: &str) {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let line = Handle::new(
        Prs3dLineAspect::new(Color::named(NamedColor::RED), AspectTypeOfLine::Solid, 1.).unwrap(),
    );
    let end = Point3::new(10., 0., 0.);
    let direction = Direction3::new(1., 0., 0.).unwrap();
    if styled {
        DsgPrsXYZAxisPresentation::add_with_aspects(
            &prs.cast(),
            &line,
            &Handle::new(Prs3dArrowAspect::new().unwrap()),
            &Handle::new(Prs3dTextAspect::new()),
            &direction,
            10.,
            label,
            &Point3::default(),
            &end,
        )
        .unwrap();
    } else {
        DsgPrsXYZAxisPresentation::add(
            &prs.cast(),
            &line,
            &direction,
            10.,
            label,
            &Point3::default(),
            &end,
        )
        .unwrap();
    }
    let trace = trace.read();
    assert_eq!(trace.texts.len(), usize::from(!label.is_empty()));
    if let Some(text) = trace.texts.first() {
        assert_eq!(text.height(), 1. / 81.);
        assert_eq!(text.position(), end);
        assert_eq!(text.text().to_c_string()[0], label.as_bytes()[0]);
    }
    assert!(trace.events.windows(3).any(|v| v
        == [
            4.,
            Graphic3dTypeOfPrimitiveArray::Segments as i32 as f64,
            2.
        ]));
}
