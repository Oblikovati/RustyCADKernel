//! The YUV to RGB shader programs of `Graphic3d_MediaTextureSet`, text for text as the native
//! constructor assembles them (the Vulkan driver reads the same description).
use super::{Graphic3dShaderObject, Graphic3dShaderProgram, Graphic3dTypeOfShaderObject};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;

/// The vertex stage: the texture coordinates and the transformed vertex.
const SRC_VERT: &str = "\nout vec2 TexCoord;\nvoid main()\n{\n  TexCoord = occTexCoord.st;\n gl_Position = occProjectionMatrix * occWorldViewMatrix * occModelWorldMatrix * occVertex;\n}";

/// The fragment stage: the three planes sampled and converted.
const SRC_FRAG: &str = "\nin vec2 TexCoord;\nuniform sampler2D occSampler1;\nuniform sampler2D occSampler2;\nvec3 convertToRGB (in vec3 theYUV);\nvoid main()\n{\n  vec3 aYUV = vec3 (occTexture2D (occSampler0, TexCoord.st).r,\n                    occTexture2D (occSampler1, TexCoord.st).r,\n                    occTexture2D (occSampler2, TexCoord.st).r);\n occSetFragColor (vec4 (convertToRGB (aYUV), 1.0));\n}";

/// `F_SHADER_YUV2RGB_MPEG`: the reduced (MPEG) range conversion.
const YUV2RGB_MPEG: &str = "\nconst float TheRangeBits = 1.0;\nvec3 convertToRGB (in vec3 theYUV)\n{\n  vec3 aYUV = theYUV.rgb;\n  aYUV   *= TheRangeBits;\n  aYUV.x  = 1.1643 * (aYUV.x - 0.0625);\n  aYUV.y -= 0.5;\n  aYUV.z -= 0.5;\n  vec3 aColor = vec3(0.0);\n  aColor.r = aYUV.x +  1.5958 * aYUV.z;\n  aColor.g = aYUV.x - 0.39173 * aYUV.y - 0.81290 * aYUV.z;\n  aColor.b = aYUV.x +   2.017 * aYUV.y;\n  return aColor;\n}";

/// `F_SHADER_YUV2RGB_FULL`: the full (JPEG) range conversion.
const YUV2RGB_FULL: &str = "\nconst float TheRangeBits = 1.0;\nvec3 convertToRGB (in vec3 theYUV)\n{\n  vec3 aYUV = theYUV.rgb;\n  aYUV   *= TheRangeBits;\n  aYUV.x  = aYUV.x;\n  aYUV.y -= 0.5;\n  aYUV.z -= 0.5;\n  vec3 aColor = vec3(0.0);\n  aColor.r = aYUV.x + 1.402 * aYUV.z;\n  aColor.g = aYUV.x - 0.344 * aYUV.y - 0.714 * aYUV.z;\n  aColor.b = aYUV.x + 1.772 * aYUV.y;\n  return aColor;\n}";

/// The YUV program (`#version 150`, no lights, no clip planes) of a range;
/// `yuv_shader_program(true)` is `myShaderYUVJ`.
pub(crate) fn yuv_shader_program(full_range: bool) -> Handle<Graphic3dShaderProgram> {
    let program = Graphic3dShaderProgram::default();
    program.set_header(AsciiString::from("#version 150"));
    program.set_nb_lights_max(0);
    program.set_nb_clip_planes_max(0);
    let conversion = if full_range {
        YUV2RGB_FULL
    } else {
        YUV2RGB_MPEG
    };
    let fragment = format!("{SRC_FRAG}{conversion}");
    program.attach_shader(&Graphic3dShaderObject::create_from_source(
        Graphic3dTypeOfShaderObject::Vertex,
        AsciiString::from(SRC_VERT),
    ));
    program.attach_shader(&Graphic3dShaderObject::create_from_source(
        Graphic3dTypeOfShaderObject::Fragment,
        AsciiString::from(fragment.as_str()),
    ));
    Handle::new(program)
}
