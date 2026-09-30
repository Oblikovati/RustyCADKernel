// Rust translation of FoundationClasses/TKMath/Poly/Poly_Triangulation.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_Triangulation`: nodes, triangles and optional UV nodes and normals of a mesh.
mod access;
mod arrays;
mod attributes;
mod bounds;
mod deferred;
use super::{
    ArrayOfNodes, ArrayOfUvNodes, MeshPurpose, Triangle, TriangulationParameters,
    errors::in_member, mesh_purpose,
};
use crate::{
    bnd::BoundingBox3,
    gp::{Point2, Point3},
};
pub use deferred::DeferredTriangulation;
use rustycad_tkernel::{
    ncollection::{Array1, Vec3},
    standard::{
        Failure, Handle, SharedState, StandardTransient,
        dump::{self, DumpOutput},
    },
};
use std::io;
/// The fields OCCT keeps in `Poly_Triangulation`, guarded together.
pub(crate) struct TriangulationFields {
    pub(crate) deflection: f64,
    pub(crate) nodes: ArrayOfNodes<'static>,
    pub(crate) triangles: Array1<'static, Triangle>,
    pub(crate) uv_nodes: ArrayOfUvNodes<'static>,
    pub(crate) normals: Array1<'static, Vec3<f32>>,
    pub(crate) purpose: MeshPurpose,
    pub(crate) parameters: Handle<TriangulationParameters>,
}
impl Default for TriangulationFields {
    fn default() -> Self {
        Self {
            deflection: 0.0,
            nodes: ArrayOfNodes::default(),
            triangles: Array1::default(),
            uv_nodes: ArrayOfUvNodes::default(),
            normals: Array1::default(),
            purpose: mesh_purpose::NONE,
            parameters: Handle::default(),
        }
    }
}
/// `Poly_Triangulation`, shared through `Handle` and changed through any of them;
/// `Triangulation::with_sizes(3, 1, false, false)?`.
pub struct Triangulation {
    base: StandardTransient,
    fields: SharedState<TriangulationFields>,
    // myCachedMinMax and its shared mutex.
    cached_min_max: SharedState<Option<BoundingBox3>>,
    deferred: Option<Box<dyn DeferredTriangulation>>,
}
rustycad_tkernel::impl_transient!(Triangulation, "Poly_Triangulation", StandardTransient, base);
impl Default for Triangulation {
    fn default() -> Self {
        Self::from_fields(TriangulationFields::default())
    }
}
impl Triangulation {
    /// Constructs an empty triangulation; `Triangulation::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    pub(crate) fn from_fields(fields: TriangulationFields) -> Self {
        Self {
            base: StandardTransient::default(),
            fields: SharedState::new(fields),
            cached_min_max: SharedState::new(None),
            deferred: None,
        }
    }
    /// Allocates nodes, triangles and optionally UV nodes and normals;
    /// `Triangulation::with_sizes(4, 2, true, false)?`.
    pub fn with_sizes(
        nb_nodes: i32,
        nb_triangles: i32,
        has_uv_nodes: bool,
        has_normals: bool,
    ) -> Result<Self, Failure> {
        const MEMBER: &str = "Poly_Triangulation::Poly_Triangulation";
        let mut fields = TriangulationFields {
            nodes: ArrayOfNodes::new(node_count(nb_nodes)).map_err(|e| in_member(MEMBER, e))?,
            triangles: Array1::with_bounds(1, nb_triangles).map_err(|e| e.into_failure(MEMBER))?,
            ..TriangulationFields::default()
        };
        if has_uv_nodes {
            fields.uv_nodes.resize(node_count(nb_nodes), false)?;
        }
        if has_normals {
            fields
                .normals
                .resize_bounds(0, nb_nodes - 1, false)
                .map_err(|e| e.into_failure(MEMBER))?;
        }
        Ok(Self::from_fields(fields))
    }
    /// Copies 3D nodes and triangles; the triangle array keeps the input's lower bound,
    /// because the native constructor assigns it with `NCollection_Array1::Assign`;
    /// `Triangulation::from_nodes(&nodes, &triangles)?`.
    pub fn from_nodes(
        nodes: &Array1<'_, Point3>,
        triangles: &Array1<'_, Triangle>,
    ) -> Result<Self, Failure> {
        let mut fields = TriangulationFields {
            nodes: ArrayOfNodes::new(nodes.len())?,
            ..TriangulationFields::default()
        };
        for (offset, node) in nodes.iter().enumerate() {
            fields.nodes.set_value(offset, *node)?;
        }
        fields.triangles.assign(triangles);
        Ok(Self::from_fields(fields))
    }
    /// Copies 3D nodes, UV nodes of the same count and triangles;
    /// `Triangulation::from_nodes_and_uv(&nodes, &uv, &triangles)?`.
    pub fn from_nodes_and_uv(
        nodes: &Array1<'_, Point3>,
        uv_nodes: &Array1<'_, Point2>,
        triangles: &Array1<'_, Triangle>,
    ) -> Result<Self, Failure> {
        let triangulation = Self::from_nodes(nodes, triangles)?;
        {
            let mut fields = triangulation.fields.write();
            fields.uv_nodes = ArrayOfUvNodes::new(nodes.len())?;
            // Poly_ArrayOfUVNodes::operator= requires equal sizes, as the native Assign.
            if uv_nodes.len() != nodes.len() {
                return Err(Failure::with_type(
                    "Standard_DimensionMismatch",
                    &format!(
                        "Poly_ArrayOfUVNodes::Assign(), {} UV nodes; expected {}",
                        uv_nodes.len(),
                        nodes.len()
                    ),
                    "",
                ));
            }
            for (offset, node) in uv_nodes.iter().enumerate() {
                fields.uv_nodes.set_value(offset, *node)?;
            }
        }
        Ok(triangulation)
    }
    /// Attaches a deferred storage, the Rust form of a subclass overriding the late-load
    /// interface; `Triangulation::with_deferred(Box::new(storage))`.
    pub fn with_deferred(storage: Box<dyn DeferredTriangulation>) -> Self {
        Self {
            deferred: Some(storage),
            ..Self::default()
        }
    }
    /// Deep copy with copied parameters and cached range, as `Copy()`; `triangulation.copy()`.
    pub fn copy(&self) -> Handle<Self> {
        Handle::new(Self::copied_from(self))
    }
    /// The native copy constructor; the deferred storage is not copied; `Triangulation::copied_from(&other)`.
    pub fn copied_from(other: &Self) -> Self {
        let source = other.fields.read();
        let parameters = source
            .parameters
            .get()
            .map_or_else(Handle::default, TriangulationParameters::copy);
        let copy = Self::from_fields(TriangulationFields {
            deflection: source.deflection,
            nodes: source.nodes.clone(),
            triangles: source.triangles.clone(),
            uv_nodes: source.uv_nodes.clone(),
            normals: source.normals.clone(),
            purpose: source.purpose,
            parameters,
        });
        if let Some(range) = other.cached_min_max.read().as_ref() {
            copy.set_cached_min_max(range);
        }
        copy
    }
    /// Returns the deflection; `triangulation.deflection()`.
    pub fn deflection(&self) -> f64 {
        self.fields.read().deflection
    }
    /// Sets the deflection; `triangulation.set_deflection(0.1)`.
    pub fn set_deflection(&self, deflection: f64) {
        self.fields.write().deflection = deflection;
    }
    /// Returns the (possibly null) parameters handle; `triangulation.parameters()`.
    pub fn parameters(&self) -> Handle<TriangulationParameters> {
        self.fields.read().parameters.clone()
    }
    /// Replaces the parameters handle; `triangulation.set_parameters(params)`.
    pub fn set_parameters(&self, parameters: Handle<TriangulationParameters>) {
        self.fields.write().parameters = parameters;
    }
    /// Returns the mesh purpose bits; `triangulation.mesh_purpose()`.
    pub fn mesh_purpose(&self) -> MeshPurpose {
        self.fields.read().purpose
    }
    /// Sets the mesh purpose bits; `triangulation.set_mesh_purpose(mesh_purpose::CALCULATION)`.
    pub fn set_mesh_purpose(&self, purpose: MeshPurpose) {
        self.fields.write().purpose = purpose;
    }
    /// Writes the class name, deflection, array sizes and purpose; `triangulation.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        let fields = self.fields.read();
        dump::dump_class_begin(output, "Poly_Triangulation")?;
        dump::dump_real(output, "Deflection", fields.deflection)?;
        dump_size(output, "Nodes.Size", fields.nodes.len())?;
        if !fields.uv_nodes.is_empty() {
            dump_size(output, "UVNodes.Size", fields.uv_nodes.len())?;
        }
        if !fields.normals.is_empty() {
            dump_size(output, "Normals.Size", fields.normals.len())?;
        }
        dump_size(output, "Triangles.Size", fields.triangles.len())?;
        dump_size(output, "Purpose", fields.purpose as usize)
    }
}
/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` for an unsigned count; `dump_size(&mut stream, "Nodes.Size", 3)?`.
pub(crate) fn dump_size(output: &mut impl DumpOutput, name: &str, value: usize) -> io::Result<()> {
    dump::add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": {value}").as_bytes())
}
/// A native `int` node count passed where OCCT converts to `size_t`; `node_count(3) == 3`.
pub(crate) fn node_count(count: i32) -> usize {
    // A negative count wraps to a huge size_t natively and fails to allocate.
    usize::try_from(count).unwrap_or(usize::MAX)
}
