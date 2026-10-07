//! `Graphic3d_ShaderProgram`: a set of shader stages with custom uniforms, vertex attributes
//! and the limits (lights, clip planes, fragment outputs) the driver compiles them with.
//!
//! The stages carry GLSL text for the OpenGL driver; the Vulkan driver reads the same program
//! description but compiles its own shaders (visualization-design.md section 3).
use super::{
    Graphic3dRenderTransparentMethod, Graphic3dShaderAttribute, Graphic3dShaderObject,
    Graphic3dShaderVariable, Graphic3dTextureSetBits, Graphic3dUniformValueTypeId,
};
use rustycad_tkernel::ncollection::{Mat3, Mat4, Vec2, Vec3, Vec4};
use rustycad_tkernel::osd::{CadPath, Directory, Environment, File};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};

/// `THE_PROGRAM_OBJECT_COUNTER`: the source of the `Graphic3d_ShaderProgram_<n>` ids.
static PROGRAM_OBJECT_COUNTER: AtomicI32 = AtomicI32::new(0);

/// The mutable members of a program.
struct ProgramState {
    id: AsciiString,
    shader_objects: Vec<Handle<Graphic3dShaderObject>>,
    variables: Vec<Handle<Graphic3dShaderVariable>>,
    attributes: Vec<Handle<Graphic3dShaderAttribute>>,
    header: AsciiString,
    nb_lights_max: i32,
    nb_shadow_maps: i32,
    nb_clip_planes_max: i32,
    nb_frag_outputs: i32,
    texture_set_bits: i32,
    oit_output: Graphic3dRenderTransparentMethod,
    has_def_sampler: bool,
    has_alpha_test: bool,
    is_pbr: bool,
}

/// A shader program; `let program = Graphic3dShaderProgram::default(); program.is_done() == false`.
pub struct Graphic3dShaderProgram {
    transient: StandardTransient,
    state: SharedState<ProgramState>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dShaderProgram,
    "Graphic3d_ShaderProgram",
    StandardTransient,
    transient
);

impl Default for Graphic3dShaderProgram {
    /// No stages, 8 lights, 8 clip planes, 1 output, default sampler on, a new id.
    fn default() -> Self {
        let number = PROGRAM_OBJECT_COUNTER
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        let mut id = AsciiString::from("Graphic3d_ShaderProgram_");
        id.assign_cat(number);
        let state = ProgramState {
            id,
            shader_objects: Vec::new(),
            variables: Vec::new(),
            attributes: Vec::new(),
            header: AsciiString::default(),
            nb_lights_max: Self::THE_MAX_LIGHTS_DEFAULT,
            nb_shadow_maps: 0,
            nb_clip_planes_max: Self::THE_MAX_CLIP_PLANES_DEFAULT,
            nb_frag_outputs: Self::THE_NB_FRAG_OUTPUTS,
            texture_set_bits: Graphic3dTextureSetBits::NONE.0,
            oit_output: Graphic3dRenderTransparentMethod::BlendUnordered,
            has_def_sampler: true,
            has_alpha_test: false,
            is_pbr: false,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
}

/// Identity, header, limits and stages.
impl Graphic3dShaderProgram {
    /// The default number of lights (`THE_MAX_LIGHTS_DEFAULT`).
    pub const THE_MAX_LIGHTS_DEFAULT: i32 = 8;
    /// The default number of clipping planes (`THE_MAX_CLIP_PLANES_DEFAULT`).
    pub const THE_MAX_CLIP_PLANES_DEFAULT: i32 = 8;
    /// The default number of fragment outputs (`THE_NB_FRAG_OUTPUTS`).
    pub const THE_NB_FRAG_OUTPUTS: i32 = 1;

    /// Whether there is a stage and every stage has a source; `program.is_done()`.
    pub fn is_done(&self) -> bool {
        let state = self.state.read();
        !state.shader_objects.is_empty()
            && state.shader_objects.iter().all(|shader| shader.is_done())
    }
    /// The unique id; `program.get_id()`.
    pub fn get_id(&self) -> AsciiString {
        self.state.read().id.clone()
    }
    /// Replaces the id; `program.set_id("custom".into())`.
    pub fn set_id(&self, id: AsciiString) {
        self.state.write().id = id;
    }
    /// The GLSL header (version and extensions); `program.header()`.
    pub fn header(&self) -> AsciiString {
        self.state.read().header.clone()
    }
    /// Replaces the header; `program.set_header("#version 150".into())`.
    pub fn set_header(&self, header: AsciiString) {
        self.state.write().header = header;
    }
    /// Appends a header line after a newline when the header is not empty;
    /// `program.append_to_header(&"#extension X : enable".into())`.
    pub fn append_to_header(&self, line: &AsciiString) {
        let mut state = self.state.write();
        if !state.header.is_empty() {
            state.header.assign_cat("\n");
        }
        state.header.append(line);
    }
    /// The light array length; `program.nb_lights_max()`.
    pub fn nb_lights_max(&self) -> i32 {
        self.state.read().nb_lights_max
    }
    /// Sets the light array length; `program.set_nb_lights_max(4)`.
    pub fn set_nb_lights_max(&self, count: i32) {
        self.state.write().nb_lights_max = count;
    }
    /// The shadow map array length; `program.nb_shadow_maps()`.
    pub fn nb_shadow_maps(&self) -> i32 {
        self.state.read().nb_shadow_maps
    }
    /// Sets the shadow map array length; `program.set_nb_shadow_maps(1)`.
    pub fn set_nb_shadow_maps(&self, count: i32) {
        self.state.write().nb_shadow_maps = count;
    }
    /// The clipping plane array length; `program.nb_clip_planes_max()`.
    pub fn nb_clip_planes_max(&self) -> i32 {
        self.state.read().nb_clip_planes_max
    }
    /// Sets the clipping plane array length; `program.set_nb_clip_planes_max(2)`.
    pub fn set_nb_clip_planes_max(&self, count: i32) {
        self.state.write().nb_clip_planes_max = count;
    }
    /// Attaches a stage unless it is null or already attached; `program.attach_shader(&shader)`.
    pub fn attach_shader(&self, shader: &Handle<Graphic3dShaderObject>) -> bool {
        let mut state = self.state.write();
        if shader.is_null()
            || state
                .shader_objects
                .iter()
                .any(|attached| attached == shader)
        {
            return false;
        }
        state.shader_objects.push(shader.clone());
        true
    }
    /// Detaches an attached stage; `program.detach_shader(&shader)`.
    pub fn detach_shader(&self, shader: &Handle<Graphic3dShaderObject>) -> bool {
        let mut state = self.state.write();
        if shader.is_null() {
            return false;
        }
        let Some(index) = state
            .shader_objects
            .iter()
            .position(|attached| attached == shader)
        else {
            return false;
        };
        state.shader_objects.remove(index);
        true
    }
    /// The attached stages; `program.shader_objects().len()`.
    pub fn shader_objects(&self) -> Vec<Handle<Graphic3dShaderObject>> {
        self.state.read().shader_objects.clone()
    }
}

/// Variables, attributes and flags.
impl Graphic3dShaderProgram {
    /// The custom uniform variables; `program.variables()`.
    pub fn variables(&self) -> Vec<Handle<Graphic3dShaderVariable>> {
        self.state.read().variables.clone()
    }
    /// The custom vertex attributes; `program.vertex_attributes()`.
    pub fn vertex_attributes(&self) -> Vec<Handle<Graphic3dShaderAttribute>> {
        self.state.read().attributes.clone()
    }
    /// Replaces the vertex attributes; `program.set_vertex_attributes(vec![attribute])`.
    pub fn set_vertex_attributes(&self, attributes: Vec<Handle<Graphic3dShaderAttribute>>) {
        self.state.write().attributes = attributes;
    }
    /// The fragment output count; `program.nb_fragment_outputs()`.
    pub fn nb_fragment_outputs(&self) -> i32 {
        self.state.read().nb_frag_outputs
    }
    /// Sets the fragment output count; `program.set_nb_fragment_outputs(2)`.
    pub fn set_nb_fragment_outputs(&self, count: i32) {
        self.state.write().nb_frag_outputs = count;
    }
    /// Whether the fragment stage performs the alpha test; `program.has_alpha_test()`.
    pub fn has_alpha_test(&self) -> bool {
        self.state.read().has_alpha_test
    }
    /// Sets the alpha-test flag; `program.set_alpha_test(true)`.
    pub fn set_alpha_test(&self, alpha_test: bool) {
        self.state.write().has_alpha_test = alpha_test;
    }
    /// Whether the program declares the default sampler; `program.has_default_sampler()`.
    pub fn has_default_sampler(&self) -> bool {
        self.state.read().has_def_sampler
    }
    /// Sets the default-sampler flag; `program.set_default_sampler(false)`.
    pub fn set_default_sampler(&self, has_sampler: bool) {
        self.state.write().has_def_sampler = has_sampler;
    }
    /// The order-independent transparency output; `program.oit_output()`.
    pub fn oit_output(&self) -> Graphic3dRenderTransparentMethod {
        self.state.read().oit_output
    }
    /// Sets the transparency output; `program.set_oit_output(Graphic3dRenderTransparentMethod::BlendOit)`.
    pub fn set_oit_output(&self, output: Graphic3dRenderTransparentMethod) {
        self.state.write().oit_output = output;
    }
    /// Whether the program uses the PBR pipeline; `program.is_pbr()`.
    pub fn is_pbr(&self) -> bool {
        self.state.read().is_pbr
    }
    /// Sets the PBR flag; `program.set_pbr(true)`.
    pub fn set_pbr(&self, is_pbr: bool) {
        self.state.write().is_pbr = is_pbr;
    }
    /// The declared texture units (`Graphic3d_TextureSetBits`); `program.texture_set_bits()`.
    pub fn texture_set_bits(&self) -> i32 {
        self.state.read().texture_set_bits
    }
    /// Sets the declared texture units; `program.set_texture_set_bits(1)`.
    pub fn set_texture_set_bits(&self, bits: i32) {
        self.state.write().texture_set_bits = bits;
    }
    /// `PushVariable<T>`: appends a uniform when its name is not empty;
    /// `program.push_variable("occScale", 2.0f32)`.
    pub fn push_variable<T: Graphic3dUniformValueTypeId + Send + Sync>(
        &self,
        name: impl Into<AsciiString>,
        value: T,
    ) -> bool {
        let variable = Graphic3dShaderVariable::create(name, value);
        if variable.is_null() || !variable.is_done() {
            return false;
        }
        self.state.write().variables.push(variable);
        true
    }
    /// Removes every custom uniform; `program.clear_variables()`.
    pub fn clear_variables(&self) {
        self.state.write().variables.clear();
    }
}

/// The typed `PushVariable*` shortcuts.
impl Graphic3dShaderProgram {
    /// `PushVariableFloat`; `program.push_variable_float("a", 1.0)`.
    pub fn push_variable_float(&self, name: impl Into<AsciiString>, value: f32) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableVec2`; `program.push_variable_vec2("a", Vec2::new(1.0, 2.0))`.
    pub fn push_variable_vec2(&self, name: impl Into<AsciiString>, value: Vec2<f32>) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableVec3`; `program.push_variable_vec3("a", Vec3::new(1.0, 2.0, 3.0))`.
    pub fn push_variable_vec3(&self, name: impl Into<AsciiString>, value: Vec3<f32>) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableVec4`; `program.push_variable_vec4("a", Vec4::new(1.0, 2.0, 3.0, 4.0))`.
    pub fn push_variable_vec4(&self, name: impl Into<AsciiString>, value: Vec4<f32>) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableInt`; `program.push_variable_int("a", 1)`.
    pub fn push_variable_int(&self, name: impl Into<AsciiString>, value: i32) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableVec2i`; `program.push_variable_vec2i("a", Vec2::new(1, 2))`.
    pub fn push_variable_vec2i(&self, name: impl Into<AsciiString>, value: Vec2<i32>) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableVec3i`; `program.push_variable_vec3i("a", Vec3::new(1, 2, 3))`.
    pub fn push_variable_vec3i(&self, name: impl Into<AsciiString>, value: Vec3<i32>) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableVec4i`; `program.push_variable_vec4i("a", Vec4::new(1, 2, 3, 4))`.
    pub fn push_variable_vec4i(&self, name: impl Into<AsciiString>, value: Vec4<i32>) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableMat3`; `program.push_variable_mat3("a", Mat3::default())`.
    pub fn push_variable_mat3(&self, name: impl Into<AsciiString>, value: Mat3<f32>) -> bool {
        self.push_variable(name, value)
    }
    /// `PushVariableMat4`; `program.push_variable_mat4("a", Mat4::default())`.
    pub fn push_variable_mat4(&self, name: impl Into<AsciiString>, value: Mat4<f32>) -> bool {
        self.push_variable(name, value)
    }
}

/// The value of an environment variable, empty when unset.
fn environment_value(name: &str) -> AsciiString {
    Environment::named(name)
        .map(|mut variable| variable.value())
        .unwrap_or_default()
}

/// The folder of `ShadersFolder`: `CSF_ShadersDirectory`, else `CASROOT/src/Shaders`.
fn shaders_folder_from_environment() -> AsciiString {
    let folder = environment_value("CSF_ShadersDirectory");
    if !folder.is_empty() {
        return folder;
    }
    let mut folder = environment_value("CASROOT");
    if !folder.is_empty() {
        folder.assign_cat("/src/Shaders");
    }
    folder
}

impl Graphic3dShaderProgram {
    /// `ShadersFolder`: the folder of the standard GLSL programs, read once from the environment.
    /// The first call raises `Standard_Failure` when the folder or its `Declarations.glsl` is
    /// missing. Native quirk: the folder is marked defined before that check, so later calls
    /// return it without raising.
    ///
    /// Example: `Graphic3dShaderProgram::shaders_folder()?`.
    pub fn shaders_folder() -> Result<AsciiString, Failure> {
        static FOLDER: OnceLock<AsciiString> = OnceLock::new();
        let mut first_call = false;
        let folder = FOLDER.get_or_init(|| {
            first_call = true;
            shaders_folder_from_environment()
        });
        if !first_call || folder.is_empty() {
            return Ok(folder.clone());
        }
        let mut declarations = folder.clone();
        declarations.assign_cat("/Declarations.glsl");
        let directory = Directory::with_path(CadPath::new(folder.as_bytes()));
        if !directory.exists() || !File::new(declarations.as_bytes()).exists() {
            eprintln!(
                "Standard GLSL programs are not found in: {}",
                String::from_utf8_lossy(folder.as_bytes())
            );
            let message = format!(
                "CSF_ShadersDirectory or CASROOT is set incorrectly; got folder '{}'",
                String::from_utf8_lossy(folder.as_bytes())
            );
            return Err(Failure::with_type("Standard_Failure", &message, ""));
        }
        Ok(folder.clone())
    }
}

#[cfg(test)]
#[path = "shader_program_tests.rs"]
mod tests;
