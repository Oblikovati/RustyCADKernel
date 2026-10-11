//! Compare protected native label measurements and transformations; `cargo run --example ais_text_label_probe`.
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{ais::*, v3d::V3dViewer};
fn emit_case(label: &AisTextLabel, mode: i32) -> Result<(), Box<dyn std::error::Error>> {
    let horizontal = [
        Graphic3dHorizontalTextAlignment::Left,
        Graphic3dHorizontalTextAlignment::Center,
        Graphic3dHorizontalTextAlignment::Right,
    ];
    let vertical = [
        Graphic3dVerticalTextAlignment::Bottom,
        Graphic3dVerticalTextAlignment::Center,
        Graphic3dVerticalTextAlignment::Top,
        Graphic3dVerticalTextAlignment::Topfirstline,
    ];
    label.set_h_justification(horizontal[(mode % 3) as usize]);
    label.set_v_justification(vertical[(mode / 3) as usize]);
    let (mut center, width, height) = label
        .calculate_label_params(label.position())?
        .expect("font found");
    let plane = label.calculate_label_trsf(label.position(), &mut center)?;
    print!(
        "{mode} 1 {width:.17} {height:.17} {:.17} {:.17} {:.17}",
        center.x(),
        center.y(),
        center.z()
    );
    for row in 1..=3 {
        for col in 1..=4 {
            print!(" {:.17}", plane.value(row, col)?);
        }
    }
    println!();
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle())?;
    let context = AisInteractiveContext::new(&viewer)?;
    let label = AisTextLabel::new();
    label.set_context(&context);
    label.set_text(&ExtendedString::new("AV\tlabel\nsecond"));
    label.set_font("DejaVu Sans");
    label.set_height(16.);
    label.set_angle(0.37);
    label.set_orientation_3d(Axis2::with_x_direction(
        Point3::new(2., 3., 4.),
        Direction3::Y,
        Direction3::X,
    )?);
    for mode in 0..12 {
        emit_case(&label, mode)?;
    }
    Ok(())
}
