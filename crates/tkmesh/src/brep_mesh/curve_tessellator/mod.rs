//! Native edge sampling and surface-aware deflection refinement.
use crate::{
    imesh_data::{EdgeHandle, FaceHandle},
    imesh_tools::{CurveTessellator, MeshParameters},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topods::{self, Edge},
};
use rustycad_tkernel::{
    impl_transient, precision,
    standard::{Failure, StandardTransient},
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, top_abs::Orientation};
use rustycad_tkgeombase::gcpnts::TangentialDeflection;
use rustycad_tkmath::{geom_abs::SurfaceType, gp::Point3};
mod initialize;
mod refine;
/// Native BRepMesh_CurveTessellator; `BRepMeshCurveTessellator::new(&edge, &parameters, 2)?`.
pub struct BRepMeshCurveTessellator {
    transient: StandardTransient,
    edge: Edge,
    adaptor: BRepAdaptorCurve,
    distribution: TangentialDeflection,
    square_edge_deflection: f64,
    square_min_size: f64,
    square_edge_tolerance: f64,
    face_range_u: [f64; 2],
    face_range_v: [f64; 2],
}
impl_transient!(BRepMeshCurveTessellator,"BRepMesh_CurveTessellator",dyn CurveTessellator,transient,interfaces:[dyn CurveTessellator]);
impl BRepMeshCurveTessellator {
    /// Tessellate the edge's 3D representation; `BRepMeshCurveTessellator::new(&edge, &parameters, 2)?`.
    pub fn new(
        edge: &EdgeHandle,
        parameters: &MeshParameters,
        minimum_points: i32,
    ) -> Result<Self, Failure> {
        let topology = edge.get_edge()?;
        let adaptor = BRepAdaptorCurve::with_edge(&topology)?;
        Self::from_adaptor(edge, topology, adaptor, parameters, minimum_points)
    }
    /// Tessellate its oriented curve on a face; `BRepMeshCurveTessellator::on_face(&edge, orientation, &face, &parameters, 2)?`.
    pub fn on_face(
        edge: &EdgeHandle,
        orientation: Orientation,
        face: &FaceHandle,
        parameters: &MeshParameters,
        minimum_points: i32,
    ) -> Result<Self, Failure> {
        let topology = topods::edge(&edge.get_shape().oriented(orientation))?;
        let adaptor = BRepAdaptorCurve::with_edge_on_face(&topology, &face.get_face()?)?;
        Self::from_adaptor(edge, topology, adaptor, parameters, minimum_points)
    }
    fn from_adaptor(
        edge: &EdgeHandle,
        topology: Edge,
        adaptor: BRepAdaptorCurve,
        parameters: &MeshParameters,
        minimum_points: i32,
    ) -> Result<Self, Failure> {
        let mut result = Self {
            transient: StandardTransient::default(),
            edge: topology,
            adaptor,
            distribution: TangentialDeflection::new(),
            square_edge_deflection: 0.0,
            square_min_size: 0.0,
            square_edge_tolerance: 0.0,
            face_range_u: [0.0; 2],
            face_range_v: [0.0; 2],
        };
        result.initialize(edge, parameters, minimum_points)?;
        Ok(result)
    }
    fn within_surface_bounds(&self, point: Point3, parameter: f64) -> Result<bool, Failure> {
        if !self.adaptor.transformed_curve().is_curve_on_surface() {
            return Ok(true);
        }
        let curve = self.adaptor.transformed_curve().curve_on_surface()?;
        let surface = curve.get_surface();
        if accepts_unbounded_surface(&*surface)? {
            return Ok(true);
        }
        let uv = curve.get_curve().d0(parameter)?;
        if uv.x() > self.face_range_u[0]
            && uv.x() < self.face_range_u[1]
            && uv.y() > self.face_range_v[0]
            && uv.y() < self.face_range_v[1]
        {
            return Ok(true);
        }
        Ok(point.square_distance(&surface.d0(uv.x(), uv.y())?) < self.square_edge_tolerance)
    }
}
impl CurveTessellator for BRepMeshCurveTessellator {
    fn points_nb(&self) -> i32 {
        self.distribution.nb_points()
    }
    fn value(&self, index: i32, point: &mut Point3, parameter: &mut f64) -> Result<bool, Failure> {
        *point = self.distribution.value(index)?;
        *parameter = self.distribution.parameter(index)?;
        // OCCT 3d097a03 leaves the vertex-tolerance rejection commented out.
        self.within_surface_bounds(*point, *parameter)
    }
}

fn accepts_unbounded_surface(
    surface: &dyn rustycad_tkg3d::adaptor3d::Adaptor3dSurface,
) -> Result<bool, Failure> {
    if !matches!(
        surface.get_type()?,
        SurfaceType::BSplineSurface | SurfaceType::BezierSurface | SurfaceType::OtherSurface
    ) || surface.is_u_periodic()?
        || surface.is_v_periodic()?
    {
        return Ok(true);
    }
    Ok(false)
}
