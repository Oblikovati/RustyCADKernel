//! Graphic3d_ShaderObject declaration generator, the twin of
//! tools/oracle/tkservice_probe_shader.cxx.
use super::common::{Reader, end, flag, text};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::graphic3d::{
    Graphic3dShaderObject, Graphic3dShaderObjectVariable, Graphic3dTypeOfShaderObject,
};

/// Reads `Count Stages...` as variables named `<prefix><index>`.
fn variables(r: &mut Reader<'_>, prefix: &str) -> Vec<Graphic3dShaderObjectVariable> {
    let count = r.integer();
    (0..count)
        .map(|index| {
            Graphic3dShaderObjectVariable::new(format!("{prefix}{index}").as_str(), r.integer())
        })
        .collect()
}

/// The stage type of a corpus value (always one stage bit).
fn stage(value: i32) -> Graphic3dTypeOfShaderObject {
    use Graphic3dTypeOfShaderObject as Tos;
    [
        Tos::Vertex,
        Tos::TessControl,
        Tos::TessEvaluation,
        Tos::Geometry,
        Tos::Fragment,
        Tos::Compute,
    ]
    .into_iter()
    .find(|kind| *kind as i32 == value)
    .expect("corpus stages are single stage bits")
}

/// `sv_shader_decl Kind HasIn HasOut NbGeomVerts NbUniforms Stages... NbInOuts Stages...`.
pub fn decl_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = stage(r.integer());
    let (has_in, has_out, verts) = (r.flag(), r.flag(), r.integer());
    let uniforms = variables(&mut r, "vec4 u");
    let in_outs = variables(&mut r, "vec4 v");
    let in_name = AsciiString::from(if has_in { "geomIn" } else { "" });
    let out_name = AsciiString::from(if has_out { "geomOut" } else { "" });
    let mut source = AsciiString::from("void main(){}");
    let shader = Graphic3dShaderObject::create_from_stage_source(
        &mut source,
        kind,
        &uniforms,
        &in_outs,
        &in_name,
        &out_name,
        verts,
    );
    text(source.as_bytes());
    flag(!shader.is_null());
    if !shader.is_null() {
        text(shader.source().as_bytes());
    }
    end();
}
