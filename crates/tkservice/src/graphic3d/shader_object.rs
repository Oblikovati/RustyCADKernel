//! `Graphic3d_ShaderObject`: one stage of a shader program (its source text, the file it came
//! from and its stage type), plus the generator of the uniform and stage in/out declarations.
//!
//! The source is GLSL text for the OpenGL driver; the Vulkan driver compiles its own shaders
//! (visualization-design.md section 3), so the port keeps the text assembly exactly and does
//! not interpret it.
use super::Graphic3dTypeOfShaderObject;
use rustycad_tkernel::osd::{CadPath, File, OpenMode, Protection};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::atomic::{AtomicI32, Ordering};

/// `THE_SHADER_OBJECT_COUNTER`: the source of the `Graphic3d_ShaderObject_<n>` ids.
static SHADER_OBJECT_COUNTER: AtomicI32 = AtomicI32::new(0);

/// `Graphic3d_ShaderObject::ShaderVariable`: a variable name and its active stages as
/// `Graphic3d_TypeOfShaderObject` bits; `Graphic3dShaderObjectVariable::new("vec4 Color", 0x11)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graphic3dShaderObjectVariable {
    /// The declaration text (`Name`), such as `vec4 Color`.
    pub name: AsciiString,
    /// The active stages (`Stages`).
    pub stages: i32,
}
impl Graphic3dShaderObjectVariable {
    /// A variable of `stages`; `Graphic3dShaderObjectVariable::new("vec4 Color", 0x11)`.
    pub fn new(name: impl Into<AsciiString>, stages: i32) -> Self {
        Self {
            name: name.into(),
            stages,
        }
    }
}

/// `Graphic3d_ShaderObject::ShaderVariableList`.
pub type Graphic3dShaderVariableList = Vec<Graphic3dShaderObjectVariable>;

/// One shader stage.
///
/// Example: `Graphic3dShaderObject::create_from_source(Graphic3dTypeOfShaderObject::Vertex, "void main(){}".into())`.
pub struct Graphic3dShaderObject {
    transient: StandardTransient,
    id: AsciiString,
    source: AsciiString,
    path: CadPath,
    kind: Graphic3dTypeOfShaderObject,
}
rustycad_tkernel::impl_transient!(
    Graphic3dShaderObject,
    "Graphic3d_ShaderObject",
    StandardTransient,
    transient
);

impl Graphic3dShaderObject {
    /// The private constructor: a new `Graphic3d_ShaderObject_<n>` id.
    fn with_type(kind: Graphic3dTypeOfShaderObject) -> Self {
        let number = SHADER_OBJECT_COUNTER
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        let mut id = AsciiString::from("Graphic3d_ShaderObject_");
        id.assign_cat(number);
        Self {
            transient: StandardTransient::default(),
            id,
            source: AsciiString::default(),
            path: CadPath::default(),
            kind,
        }
    }
    /// `CreateFromFile`: a stage with the text of `path`, or a null handle when the file does
    /// not exist. A failing size query of an existing file is returned as the error.
    ///
    /// Example: `Graphic3dShaderObject::create_from_file(Graphic3dTypeOfShaderObject::Fragment, "a.fs".into())?`.
    pub fn create_from_file(
        kind: Graphic3dTypeOfShaderObject,
        path: AsciiString,
    ) -> Result<Handle<Self>, Failure> {
        let mut shader = Self::with_type(kind);
        shader.path = CadPath::new(path.as_bytes());
        let mut file = File::new(path.as_bytes());
        if !file.exists() {
            return Ok(Handle::default());
        }
        file.open(OpenMode::ReadOnly, &Protection::default())?;
        let size = file.size()?;
        file.read(&mut shader.source, size as i32)?;
        file.close()?;
        Ok(Handle::new(shader))
    }
    /// `CreateFromSource(type, source)`; `Graphic3dShaderObject::create_from_source(kind, text)`.
    pub fn create_from_source(
        kind: Graphic3dTypeOfShaderObject,
        source: AsciiString,
    ) -> Handle<Self> {
        let mut shader = Self::with_type(kind);
        shader.source = source;
        Handle::new(shader)
    }
    /// Whether the source is not empty; `shader.is_done()`.
    pub fn is_done(&self) -> bool {
        !self.source.is_empty()
    }
    /// The source file path (empty for a source stage); `shader.path()`.
    pub fn path(&self) -> &CadPath {
        &self.path
    }
    /// The source text; `shader.source()`.
    pub fn source(&self) -> &AsciiString {
        &self.source
    }
    /// The stage type; `shader.shader_type()`.
    pub fn shader_type(&self) -> Graphic3dTypeOfShaderObject {
        self.kind
    }
    /// The unique id; `shader.get_id()`.
    pub fn get_id(&self) -> &AsciiString {
        &self.id
    }
}

/// The text blocks `CreateFromSource` prepends to a stage source.
#[derive(Default)]
struct StageDeclarations {
    uniforms: AsciiString,
    in_outs: AsciiString,
    in_structs: AsciiString,
    out_structs: AsciiString,
}

/// The arguments of the generating `CreateFromSource` that the declaration helpers share.
struct StageRequest<'a> {
    kind: i32,
    in_name: &'a AsciiString,
    out_name: &'a AsciiString,
    nb_geom_input_verts: i32,
}

/// The lowest and highest stage bit of a variable (`IntegerLast`/`IntegerFirst` when none).
fn stage_range(stages: i32) -> (i32, i32) {
    let mut range = (i32::MAX, i32::MIN);
    let mut stage = Graphic3dTypeOfShaderObject::Vertex as i32;
    while stage <= Graphic3dTypeOfShaderObject::Compute as i32 {
        if stages & stage != 0 {
            range = (range.0.min(stage), range.1.max(stage));
        }
        stage <<= 1;
    }
    range
}

fn cat(text: &mut AsciiString, parts: &[&[u8]]) {
    parts.iter().for_each(|part| text.assign_cat(*part));
}

impl StageDeclarations {
    /// Opens the `VertexData` interface blocks for an all-stages variable.
    fn open_structs(&mut self, request: &StageRequest<'_>, (lower, upper): (i32, i32)) {
        if request.kind == lower {
            self.out_structs = AsciiString::from("\nout VertexData\n{");
        } else if request.kind == upper {
            self.in_structs = AsciiString::from("\nin VertexData\n{");
        } else {
            self.in_structs = AsciiString::from("\nin  VertexData\n{");
            self.out_structs = AsciiString::from("\nout VertexData\n{");
        }
    }
    /// Declares one stage in/out variable (the loop body of the native code).
    fn add_in_out(&mut self, request: &StageRequest<'_>, variable: &Graphic3dShaderObjectVariable) {
        let (lower, upper) = stage_range(variable.stages);
        if request.kind < lower || request.kind > upper {
            return;
        }
        let geometry = Graphic3dTypeOfShaderObject::Geometry as i32;
        let has_geom_stage =
            request.nb_geom_input_verts > 0 && lower < geometry && upper >= geometry;
        let all_stages = lower == Graphic3dTypeOfShaderObject::Vertex as i32
            && upper == Graphic3dTypeOfShaderObject::Fragment as i32;
        let named = has_geom_stage || !request.in_name.is_empty() || !request.out_name.is_empty();
        if named && self.in_structs.is_empty() && self.out_structs.is_empty() && all_stages {
            self.open_structs(request, (lower, upper));
        }
        self.declare(request, variable, (lower, upper), all_stages);
    }
    /// Writes the variable into the interface blocks or as a `THE_SHADER_IN/OUT` line.
    fn declare(
        &mut self,
        request: &StageRequest<'_>,
        variable: &Graphic3dShaderObjectVariable,
        (lower, upper): (i32, i32),
        all_stages: bool,
    ) {
        let name = variable.name.as_bytes();
        if all_stages && (!self.in_structs.is_empty() || !self.out_structs.is_empty()) {
            for block in [&mut self.in_structs, &mut self.out_structs] {
                if !block.is_empty() {
                    cat(block, &[b"\n  ", name, b";"]);
                }
            }
        } else if request.kind == lower {
            cat(&mut self.in_outs, &[b"\nTHE_SHADER_OUT ", name, b";"]);
        } else if request.kind == upper {
            cat(&mut self.in_outs, &[b"\nTHE_SHADER_IN ", name, b";"]);
        }
    }
    /// Closes the interface blocks and adds the geometry-stage layout.
    fn close(&mut self, request: &StageRequest<'_>) {
        let geometry = request.kind == Graphic3dTypeOfShaderObject::Geometry as i32;
        let verts = AsciiString::from(request.nb_geom_input_verts);
        if geometry {
            let mut layout = AsciiString::from(
                "\nlayout (triangles) in;\nlayout (triangle_strip, max_vertices = ",
            );
            cat(&mut layout, &[verts.as_bytes(), b") out;"]);
            self.uniforms.prepend(&layout);
        }
        if !self.in_structs.is_empty() && geometry {
            cat(
                &mut self.in_structs,
                &[
                    b"\n} ",
                    request.in_name.as_bytes(),
                    b"[",
                    verts.as_bytes(),
                    b"];",
                ],
            );
        } else if !self.in_structs.is_empty() {
            close_block(&mut self.in_structs, request.in_name);
        }
        if !self.out_structs.is_empty() {
            close_block(&mut self.out_structs, request.out_name);
        }
    }
    /// The prepended text: uniforms, in blocks, out blocks, in/out lines.
    fn joined(&self) -> AsciiString {
        let mut text = self.uniforms.clone();
        [&self.in_structs, &self.out_structs, &self.in_outs]
            .into_iter()
            .for_each(|part| text.append(part));
        text
    }
}

/// `"\n}"`, then ` <name>` when named, then `;`.
fn close_block(block: &mut AsciiString, name: &AsciiString) {
    block.assign_cat("\n}");
    if !name.is_empty() {
        cat(block, &[b" ", name.as_bytes()]);
    }
    block.assign_cat(";");
}

impl Graphic3dShaderObject {
    /// `CreateFromSource(source, type, uniforms, stageInOuts, inName, outName, nbGeomInputVerts)`:
    /// prepends to `source` the uniforms of this stage and the in/out declarations (interface
    /// blocks when a geometry stage or block names are involved), then creates the stage.
    /// Returns a null handle for an empty source.
    ///
    /// Example: `Graphic3dShaderObject::create_from_stage_source(&mut text, kind, &uniforms, &in_outs, &"".into(), &"".into(), 0)`.
    pub fn create_from_stage_source(
        source: &mut AsciiString,
        kind: Graphic3dTypeOfShaderObject,
        uniforms: &[Graphic3dShaderObjectVariable],
        stage_in_outs: &[Graphic3dShaderObjectVariable],
        in_name: &AsciiString,
        out_name: &AsciiString,
        nb_geom_input_verts: i32,
    ) -> Handle<Self> {
        if source.is_empty() {
            return Handle::default();
        }
        let request = StageRequest {
            kind: kind as i32,
            in_name,
            out_name,
            nb_geom_input_verts,
        };
        let mut blocks = StageDeclarations::default();
        for variable in uniforms
            .iter()
            .filter(|variable| variable.stages & request.kind != 0)
        {
            cat(
                &mut blocks.uniforms,
                &[b"\nuniform ", variable.name.as_bytes(), b";"],
            );
        }
        stage_in_outs
            .iter()
            .for_each(|variable| blocks.add_in_out(&request, variable));
        blocks.close(&request);
        source.prepend(&blocks.joined());
        Self::create_from_source(kind, source.clone())
    }
}

#[cfg(test)]
#[path = "shader_object_tests.rs"]
mod tests;
