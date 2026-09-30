//! Deflection estimates following OCCT 3d097a03, including strict consistency bounds.
use super::{ShapeTool, shape_tool::native_max};
use crate::{
    imesh_data::{EdgeHandle, FaceHandle, WireHandle},
    imesh_tools::MeshParameters,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{Shape, Vertex},
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, StandardTransient},
};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
/// Native mesh tolerance calculation; `Deflection::is_consistent(0.1, 0.2, false, 0.1)`.
#[derive(Default)]
pub struct Deflection {
    transient: StandardTransient,
}
impl_transient!(
    Deflection,
    "BRepMesh_Deflection",
    StandardTransient,
    transient
);
impl Deflection {
    /// Scale relative tolerance by topology bounds; `Deflection::compute_absolute(&shape, 0.01, -1.0)?`.
    pub fn compute_absolute(shape: &Shape, relative: f64, maximum: f64) -> Result<f64, Failure> {
        if shape.is_null() {
            return Ok(relative);
        }
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(shape, &mut bounds, false)?;
        let mut size = relative;
        ShapeTool::box_max_dimension(&bounds, &mut size)?;
        let [x1, y1, z1, x2, y2, z2] = bounds.get()?;
        let maximum = if maximum > 0.0 {
            maximum
        } else {
            native_max(x2 - x1, native_max(y2 - y1, z2 - z1))
        };
        let coefficient = maximum / (2.0 * size);
        let coefficient = coefficient.clamp(0.5, 2.0);
        Ok(coefficient * size * relative)
    }
    /// Update linear and angular edge tolerance; `Deflection::compute_edge(&edge, 10.0, &parameters)?`.
    pub fn compute_edge(
        edge: &EdgeHandle,
        maximum: f64,
        parameters: &MeshParameters,
    ) -> Result<(), Failure> {
        let topology = edge.get_edge()?;
        let mut linear = parameters.deflection;
        if parameters.relative {
            linear = Self::compute_absolute(&topology, linear, maximum)?;
        }
        let (first_vertex, last_vertex) = topexp::vertices(&topology, false);
        let (curve, first, last) = ShapeTool::range_3d(&topology, false)?;
        if let Some(curve) = curve.get() {
            let first_distance = endpoint_distance(&first_vertex, curve, first)?;
            let last_distance = endpoint_distance(&last_vertex, curve, last)?;
            linear = native_max(native_max(first_distance, last_distance), linear);
        }
        edge.set_deflection(linear);
        edge.set_angular_deflection(parameters.angle);
        Ok(())
    }
    /// Average edge tolerances, using the configured value for an empty wire; `Deflection::compute_wire(&wire, &parameters)?`.
    pub fn compute_wire(wire: &WireHandle, parameters: &MeshParameters) -> Result<(), Failure> {
        let mut linear = 0.0;
        for index in 0..wire.edges_nb() {
            linear += wire.get_edge(index)?.upgrade().get_deflection();
        }
        linear = if wire.edges_nb() > 0 {
            linear / f64::from(wire.edges_nb())
        } else {
            parameters.deflection
        };
        wire.set_deflection(linear);
        Ok(())
    }
    /// Combine interior tolerance, wire averages and topology tolerances; `Deflection::compute_face(&face, &parameters)?`.
    pub fn compute_face(face: &FaceHandle, parameters: &MeshParameters) -> Result<(), Failure> {
        let mut requested = parameters.deflection_interior;
        if parameters.relative {
            requested = Self::compute_absolute(&face.get_shape(), requested, -1.0)?;
        }
        let mut linear = 0.0;
        if !parameters.force_face_deflection {
            linear = face_wire_average(face)?;
            linear = native_max(
                2.0 * ShapeTool::max_face_tolerance(&face.get_face()?)?,
                linear,
            );
        }
        face.set_deflection(native_max(requested, linear));
        Ok(())
    }
    /// Strict native consistency interval; `Deflection::is_consistent(0.1, 0.2, false, 0.1)`.
    pub fn is_consistent(current: f64, required: f64, allow_decrease: bool, ratio: f64) -> bool {
        current < (1.0 + ratio) * required
            && (!allow_decrease || current > (1.0 - ratio) * required)
    }
}
fn endpoint_distance(
    vertex: &Vertex,
    curve: &dyn GeomCurve,
    parameter: f64,
) -> Result<f64, Failure> {
    if vertex.is_null() {
        return Ok(-1.0);
    }
    Ok(BRepTool::pnt(vertex)?.distance(&curve.value(parameter)?))
}
fn face_wire_average(face: &FaceHandle) -> Result<f64, Failure> {
    let mut linear = 0.0;
    for index in 0..face.wires_nb() {
        linear += face.get_wire(index)?.get_deflection();
    }
    if face.wires_nb() > 0 {
        linear /= f64::from(face.wires_nb());
    }
    Ok(linear)
}
