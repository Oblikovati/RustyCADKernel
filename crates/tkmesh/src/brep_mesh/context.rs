// Rust translation of BRepMesh_Context.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMesh_Context`: the IMeshTools context with the BRepMesh strategies.
use super::{
    BRepMeshDelabellaMeshAlgoFactory, BRepMeshEdgeDiscret, BRepMeshFaceDiscret,
    BRepMeshMeshAlgoFactory, BRepMeshModelBuilder, BRepMeshModelHealer, BRepMeshModelPostProcessor,
    BRepMeshModelPreProcessor,
};
use crate::imesh_data::MeshShape;
use crate::imesh_tools::{Context, MeshAlgoFactory, MeshAlgoType, MeshContext};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::Message;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// `BRepMesh_Context(theMeshType)`: model builder, edge discretizer, healer,
/// pre-processor, face discretizer over the algorithm factory of the mesh type, and
/// post-processor.
///
/// ```
/// use rustycad_tkmesh::brep_mesh::BRepMeshContext;
/// use rustycad_tkmesh::imesh_tools::MeshAlgoType;
/// let context = BRepMeshContext::new(MeshAlgoType::Watson)?;
/// assert!(!context.context_base_ref().get_face_discret().is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepMeshContext {
    context: Context,
}
impl_transient!(BRepMeshContext, "BRepMesh_Context", Context, context, interfaces: [dyn MeshShape, dyn MeshContext]);

impl BRepMeshContext {
    /// `BRepMesh_Context(theMeshType)`, reading `CSF_MeshAlgo` from the process
    /// environment for the default type; `BRepMeshContext::new(MeshAlgoType::Default)?`.
    pub fn new(mesh_type: MeshAlgoType) -> Result<Self, Failure> {
        Self::with_environment(mesh_type, &|name: &str| std::env::var(name).ok())
    }

    /// The constructor with an injected environment lookup (OSD_Environment natively);
    /// `BRepMeshContext::with_environment(MeshAlgoType::Default, &|_| None)?`.
    pub fn with_environment(
        mesh_type: MeshAlgoType,
        environment: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, Failure> {
        let factory = algo_factory(resolve_mesh_type(mesh_type, environment));
        let context = Context::new();
        context.set_model_builder(&Handle::new(BRepMeshModelBuilder::new()).cast());
        context.set_edge_discret(&Handle::new(BRepMeshEdgeDiscret::new()).cast());
        context.set_model_healer(&Handle::new(BRepMeshModelHealer::new()).cast());
        context.set_pre_processor(&Handle::new(BRepMeshModelPreProcessor::new()).cast());
        context.set_face_discret(&Handle::new(BRepMeshFaceDiscret::new(&factory)).cast());
        context.set_post_processor(&Handle::new(BRepMeshModelPostProcessor::new()).cast());
        Ok(Self { context })
    }

    /// The IMeshTools context part; `context.context_base_ref().get_model()`.
    pub fn context_base_ref(&self) -> &Context {
        &self.context
    }
}

/// The DEFAULT type resolved by `CSF_MeshAlgo` ("watson"/"0", "delabella"/"1",
/// anything else Watson with a warning when not empty).
fn resolve_mesh_type(
    mesh_type: MeshAlgoType,
    environment: &dyn Fn(&str) -> Option<String>,
) -> MeshAlgoType {
    if mesh_type != MeshAlgoType::Default {
        return mesh_type;
    }
    let value = environment("CSF_MeshAlgo")
        .unwrap_or_default()
        .to_ascii_lowercase();
    match value.as_str() {
        "watson" | "0" => MeshAlgoType::Watson,
        "delabella" | "1" => MeshAlgoType::Delabella,
        _ => {
            if !value.is_empty() {
                let text = format!(
                    "BRepMesh_Context, ignore unknown algorithm '{value}' specified in CSF_MeshAlgo variable"
                );
                Message::send_warning_text(&AsciiString::new(&text));
            }
            MeshAlgoType::Watson
        }
    }
}

/// The algorithm factory of a resolved mesh type.
fn algo_factory(mesh_type: MeshAlgoType) -> Handle<dyn MeshAlgoFactory> {
    match mesh_type {
        MeshAlgoType::Delabella => Handle::new(BRepMeshDelabellaMeshAlgoFactory::new()).cast(),
        _ => Handle::new(BRepMeshMeshAlgoFactory::new()).cast(),
    }
}

impl MeshShape for BRepMeshContext {
    fn get_shape(&self) -> Shape {
        self.context.get_shape()
    }
    fn set_shape(&self, shape: &Shape) {
        self.context.set_shape(shape);
    }
}

impl MeshContext for BRepMeshContext {
    fn context_base(&self) -> &Context {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_type_follows_csf_mesh_algo() {
        assert_eq!(
            resolve_mesh_type(MeshAlgoType::Default, &|_| Some("WATSON".into())),
            MeshAlgoType::Watson
        );
        assert_eq!(
            resolve_mesh_type(MeshAlgoType::Default, &|_| Some("1".into())),
            MeshAlgoType::Delabella
        );
        assert_eq!(
            resolve_mesh_type(MeshAlgoType::Default, &|_| Some("other".into())),
            MeshAlgoType::Watson
        );
        assert_eq!(
            resolve_mesh_type(MeshAlgoType::Default, &|_| None),
            MeshAlgoType::Watson
        );
        assert_eq!(
            resolve_mesh_type(MeshAlgoType::Delabella, &|_| None),
            MeshAlgoType::Delabella
        );
    }

    #[test]
    fn mesh_type_selects_the_factory() {
        assert!(
            algo_factory(MeshAlgoType::Delabella)
                .is_instance_name("BRepMesh_DelabellaMeshAlgoFactory")
        );
        assert!(algo_factory(MeshAlgoType::Watson).is_instance_name("BRepMesh_MeshAlgoFactory"));
        assert!(BRepMeshContext::with_environment(MeshAlgoType::Delabella, &|_| None).is_ok());
    }
}
