//! Prs3d_Drawer: own values, link inheritance, own default aspects and local attribute clearing.
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkservice::aspect::{AspectTypeOfDeflection, AspectTypeOfLine};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsMembers as _, Graphic3dAspectsRenderMembers as _, Graphic3dGroupAspect,
    Graphic3dShaderProgram, Graphic3dTypeOfShadingModel,
};
use rustycad_tkv3d::prs3d::{Prs3dDrawer, Prs3dTypeOfHlr, Prs3dVertexDrawMode};
use std::f64::consts::PI;

fn linked_pair() -> (Handle<Prs3dDrawer>, Handle<Prs3dDrawer>) {
    let defaults = Handle::new(Prs3dDrawer::new());
    defaults.setup_own_defaults().unwrap();
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.set_link(&defaults);
    (defaults, drawer)
}

#[test]
fn values_fall_back_to_link_then_constants() {
    let drawer = Prs3dDrawer::new();
    assert_eq!(drawer.discretisation(), 30);
    assert_eq!(drawer.maximal_chordial_deviation(), 0.0001);
    assert_eq!(drawer.deviation_angle(), 20.0 * PI / 180.0);
    assert_eq!(drawer.type_of_hlr(), Prs3dTypeOfHlr::PolyAlgo);
    assert_eq!(drawer.vertex_draw_mode(), Prs3dVertexDrawMode::Isolated);
    assert_eq!(drawer.face_boundary_upper_continuity(), Shape::CN);
    let (defaults, linked) = linked_pair();
    defaults.set_discretisation(12);
    defaults.set_type_of_deflection(AspectTypeOfDeflection::Absolute);
    defaults.set_wire_draw(false);
    assert_eq!(linked.discretisation(), 12);
    assert_eq!(
        linked.type_of_deflection(),
        AspectTypeOfDeflection::Absolute
    );
    assert!(!linked.wire_draw());
    linked.set_wire_draw(true);
    assert!(linked.wire_draw() && linked.has_own_wire_draw());
    linked.unset_own_wire_draw();
    assert!(!linked.wire_draw());
}

#[test]
fn deviation_keeps_previous_value() {
    let drawer = Prs3dDrawer::new();
    assert_eq!(drawer.previous_deviation_coefficient(), 0.0);
    drawer.set_deviation_coefficient(0.01);
    assert_eq!(drawer.previous_deviation_coefficient(), 0.001);
    drawer.set_deviation_coefficient(0.02);
    assert_eq!(drawer.previous_deviation_coefficient(), 0.01);
    drawer.update_previous_deviation_coefficient();
    assert_eq!(drawer.previous_deviation_coefficient(), 0.02);
    drawer.unset_deviation_coefficient();
    assert!(!drawer.has_own_deviation_coefficient());
    assert_eq!(drawer.previous_deviation_coefficient(), 0.0);
    drawer.set_hlr_angle(0.5);
    assert_eq!(drawer.hlr_angle(), 0.5);
    assert!(drawer.has_own_hlr_deviation_angle());
}

#[test]
fn aspects_are_inherited_until_own() {
    let (defaults, drawer) = linked_pair();
    assert!(!drawer.has_own_line_aspect());
    assert_eq!(drawer.line_aspect(), defaults.line_aspect());
    assert!(drawer.set_own_line_aspects(&Handle::default()).unwrap());
    assert!(drawer.has_own_line_aspect());
    assert_ne!(drawer.line_aspect(), defaults.line_aspect());
    assert!(!drawer.set_own_line_aspects(&Handle::default()).unwrap());
    let hidden = drawer.hidden_line_aspect().aspect();
    assert_eq!(hidden.line_kind(), AspectTypeOfLine::Dash);
    assert_eq!(
        drawer.u_iso_aspect().number(),
        defaults.u_iso_aspect().number()
    );
}

#[test]
fn own_aspects_copy_the_link_graphic_aspects() {
    let (defaults, drawer) = linked_pair();
    defaults
        .wire_aspect()
        .set_color(Color::named(NamedColor::BLUE));
    drawer.set_own_line_aspects(&Handle::default()).unwrap();
    let color = drawer.wire_aspect().aspect().interior_color();
    assert_eq!(color, Color::named(NamedColor::BLUE));
    assert!(drawer.setup_own_point_aspect(&Handle::default()).unwrap());
    assert!(drawer.setup_own_shading_aspect(&Handle::default()).unwrap());
    assert!(drawer.set_own_datum_aspects(&Handle::default()).unwrap());
}

#[test]
fn clear_local_attributes_needs_a_link() {
    let lonely = Prs3dDrawer::new();
    lonely.set_discretisation(5);
    lonely.clear_local_attributes();
    assert!(lonely.has_own_discretisation());
    let (_, drawer) = linked_pair();
    drawer.set_discretisation(5);
    drawer.set_own_line_aspects(&Handle::default()).unwrap();
    drawer.set_dim_length_model_units(&AsciiString::from("mm"));
    drawer.set_type_of_hlr(Prs3dTypeOfHlr::Algo);
    drawer.clear_local_attributes();
    assert!(!drawer.has_own_discretisation());
    assert!(!drawer.has_own_line_aspect());
    assert!(!drawer.has_own_dim_length_model_units());
    assert!(!drawer.has_own_type_of_hlr());
}

#[test]
fn shader_program_and_shading_model_reach_own_aspects() {
    let (_, drawer) = linked_pair();
    let program = Handle::new(Graphic3dShaderProgram::default());
    assert!(
        drawer
            .set_shader_program(&program, Graphic3dGroupAspect::Line, true)
            .unwrap()
    );
    assert_eq!(drawer.line_aspect().aspect().shader_program(), program);
    assert!(
        drawer
            .set_shader_program(&program, Graphic3dGroupAspect::Text, true)
            .unwrap()
    );
    assert!(
        !drawer
            .set_shader_program(&program, Graphic3dGroupAspect::Text, true)
            .unwrap()
    );
    assert!(
        drawer
            .set_shading_model(Graphic3dTypeOfShadingModel::Unlit, true)
            .unwrap()
    );
    let model = drawer.shading_aspect().aspect().shading_model();
    assert_eq!(model, Graphic3dTypeOfShadingModel::Unlit);
}

#[test]
fn units_are_inherited_and_reset() {
    let (defaults, drawer) = linked_pair();
    defaults.set_dim_angle_display_units(&AsciiString::from("rad"));
    assert_eq!(drawer.dim_angle_display_units().as_bytes(), b"rad");
    drawer.set_dim_angle_display_units(&AsciiString::from("grad"));
    assert_eq!(drawer.dim_angle_display_units().as_bytes(), b"grad");
    drawer.unset_own_dim_angle_display_units();
    assert_eq!(drawer.dim_angle_display_units().as_bytes(), b"rad");
    assert_eq!(
        Prs3dDrawer::new().dim_angle_display_units().as_bytes(),
        b"deg"
    );
}
