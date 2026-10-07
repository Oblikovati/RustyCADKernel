//! Unit tests of `Graphic3d_ShaderObject`; the declaration text is compared with the native
//! generator by the sv_shader_decl probe.
use super::*;
use Graphic3dTypeOfShaderObject as Tos;

fn var(name: &str, stages: i32) -> Graphic3dShaderObjectVariable {
    Graphic3dShaderObjectVariable::new(name, stages)
}
fn generated(
    kind: Tos,
    uniforms: &[Graphic3dShaderObjectVariable],
    in_outs: &[Graphic3dShaderObjectVariable],
    names: (&str, &str),
    verts: i32,
) -> String {
    let mut text = AsciiString::from("void main(){}");
    let (in_name, out_name) = (AsciiString::from(names.0), AsciiString::from(names.1));
    let shader = Graphic3dShaderObject::create_from_stage_source(
        &mut text, kind, uniforms, in_outs, &in_name, &out_name, verts,
    );
    assert_eq!(shader.source().as_bytes(), text.as_bytes());
    String::from_utf8_lossy(text.as_bytes()).into_owned()
}

#[test]
fn ids_are_unique_and_sources_mark_done() {
    let a = Graphic3dShaderObject::create_from_source(Tos::Vertex, "x".into());
    let b = Graphic3dShaderObject::create_from_source(Tos::Fragment, AsciiString::default());
    assert_ne!(a.get_id(), b.get_id());
    assert!(a.get_id().starts_with("Graphic3d_ShaderObject_"));
    assert!(a.is_done() && !b.is_done());
    assert_eq!(b.shader_type(), Tos::Fragment);
}

#[test]
fn missing_files_give_a_null_handle() {
    let shader =
        Graphic3dShaderObject::create_from_file(Tos::Vertex, "/nonexistent/shader.vs".into())
            .unwrap();
    assert!(shader.is_null());
}

#[test]
fn uniforms_and_plain_in_outs_are_declared_per_stage() {
    let uniforms = [var("vec4 occColor", 0x11), var("float occScale", 0x10)];
    let in_outs = [var("vec4 Color", 0x11)];
    let vertex = generated(Tos::Vertex, &uniforms, &in_outs, ("", ""), 0);
    assert_eq!(
        vertex,
        "\nuniform vec4 occColor;\nTHE_SHADER_OUT vec4 Color;void main(){}"
    );
    let fragment = generated(Tos::Fragment, &uniforms, &in_outs, ("", ""), 0);
    assert_eq!(
        fragment,
        "\nuniform vec4 occColor;\nuniform float occScale;\nTHE_SHADER_IN vec4 Color;void main(){}"
    );
}

#[test]
fn geometry_stages_use_interface_blocks() {
    let in_outs = [var("vec4 Color", 0x11)];
    let geometry = generated(Tos::Geometry, &[], &in_outs, ("geomIn", "geomOut"), 3);
    let expected = "\nlayout (triangles) in;\nlayout (triangle_strip, max_vertices = 3) out;\nin  VertexData\n{\n  vec4 Color;\n} geomIn[3];\nout VertexData\n{\n  vec4 Color;\n} geomOut;void main(){}";
    assert_eq!(geometry, expected);
    let vertex = generated(Tos::Vertex, &[], &in_outs, ("", "geomIn"), 3);
    assert_eq!(
        vertex,
        "\nout VertexData\n{\n  vec4 Color;\n} geomIn;void main(){}"
    );
}

#[test]
fn empty_sources_give_a_null_handle() {
    let mut text = AsciiString::default();
    let empty = AsciiString::default();
    assert!(
        Graphic3dShaderObject::create_from_stage_source(
            &mut text,
            Tos::Vertex,
            &[],
            &[],
            &empty,
            &empty,
            0
        )
        .is_null()
    );
}
