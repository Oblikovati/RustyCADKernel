//! Unit tests of `Graphic3d_ShaderProgram`.
use super::*;
use crate::graphic3d::Graphic3dTypeOfShaderObject;

fn stage(text: &str) -> Handle<Graphic3dShaderObject> {
    Graphic3dShaderObject::create_from_source(Graphic3dTypeOfShaderObject::Vertex, text.into())
}

#[test]
fn defaults_follow_the_native_constructor() {
    let program = Graphic3dShaderProgram::default();
    assert!(program.get_id().starts_with("Graphic3d_ShaderProgram_"));
    assert_eq!(program.nb_lights_max(), 8);
    assert_eq!(program.nb_clip_planes_max(), 8);
    assert_eq!(program.nb_fragment_outputs(), 1);
    assert!(program.has_default_sampler() && !program.has_alpha_test() && !program.is_pbr());
    assert_eq!(
        program.oit_output(),
        Graphic3dRenderTransparentMethod::BlendUnordered
    );
    assert_ne!(program.get_id(), Graphic3dShaderProgram::default().get_id());
}

#[test]
fn stages_attach_once_and_detach() {
    let program = Graphic3dShaderProgram::default();
    assert!(!program.is_done());
    let shader = stage("void main(){}");
    assert!(program.attach_shader(&shader));
    assert!(!program.attach_shader(&shader));
    assert!(!program.attach_shader(&Handle::default()));
    assert!(program.is_done());
    assert!(program.attach_shader(&stage("")));
    assert!(!program.is_done());
    assert!(program.detach_shader(&shader));
    assert!(!program.detach_shader(&shader));
    assert_eq!(program.shader_objects().len(), 1);
}

#[test]
fn header_lines_are_joined_by_newlines() {
    let program = Graphic3dShaderProgram::default();
    program.append_to_header(&"#version 150".into());
    program.append_to_header(&"#define A".into());
    assert_eq!(program.header().as_bytes(), b"#version 150\n#define A");
}

#[test]
fn variables_need_a_name() {
    let program = Graphic3dShaderProgram::default();
    assert!(program.push_variable_float("occScale", 2.0));
    assert!(program.push_variable_vec3i("occIndex", Vec3::new(1, 2, 3)));
    assert!(!program.push_variable_int("", 1));
    assert_eq!(program.variables().len(), 2);
    program.clear_variables();
    assert!(program.variables().is_empty());
    program.set_vertex_attributes(vec![Handle::new(Graphic3dShaderAttribute::new(
        "occColor", 3,
    ))]);
    assert_eq!(program.vertex_attributes()[0].location(), 3);
}
