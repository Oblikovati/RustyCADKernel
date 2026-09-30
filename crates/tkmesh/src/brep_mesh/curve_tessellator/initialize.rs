//! Native tolerance selection, minimum sampling and internal-vertex insertion.
use super::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gcpnts::{AbscissaPoint, TangentialCriteria};
use rustycad_tkmath::geom_abs::CurveType;
impl BRepMeshCurveTessellator {
    pub(super) fn initialize(
        &mut self,
        edge: &EdgeHandle,
        parameters: &MeshParameters,
        minimum_points: i32,
    ) -> Result<(), Failure> {
        validate_minimum_size(parameters.min_size)?;
        let criteria = self.sampling_criteria(edge, parameters, minimum_points)?;
        self.initialize_distribution(criteria)?;
        self.initialize_surface_range()?;
        self.add_internal_vertices()?;
        self.split_by_deflection_2d(edge)
    }
    fn initialize_distribution(&mut self, criteria: TangentialCriteria) -> Result<(), Failure> {
        self.square_edge_deflection = criteria.curvature_deflection * criteria.curvature_deflection;
        self.square_min_size = std_max(
            self.square_edge_deflection,
            criteria.min_len * criteria.min_len,
        );
        let tolerance = BRepTool::edge_tolerance(&self.edge)?;
        self.square_edge_tolerance = tolerance * tolerance;
        self.distribution.initialize_range(
            &self.adaptor as &dyn Adaptor3dCurve,
            self.adaptor.first_parameter()?,
            self.adaptor.last_parameter()?,
            criteria,
        )
    }
    fn sampling_criteria(
        &self,
        edge: &EdgeHandle,
        parameters: &MeshParameters,
        minimum_points: i32,
    ) -> Result<TangentialCriteria, Failure> {
        let mut linear = 0.5 * edge.get_deflection();
        if self.edge.orientation() == Orientation::Internal {
            linear *= 0.5;
        }
        linear = std_max(linear, precision::confusion());
        let minimum_size = self.adjusted_minimum_size(parameters, linear)?;
        let threshold = match self.adaptor.get_type()? {
            CurveType::Circle | CurveType::Ellipse | CurveType::Parabola | CurveType::Hyperbola => {
                4
            }
            _ => 2,
        };
        Ok(TangentialCriteria {
            angular_deflection: std_max(0.5 * edge.get_angular_deflection(), precision::angular()),
            curvature_deflection: linear,
            minimum_of_points: minimum_points.max(threshold),
            u_tol: precision::p_confusion(),
            min_len: minimum_size,
        })
    }
    fn adjusted_minimum_size(
        &self,
        parameters: &MeshParameters,
        linear: f64,
    ) -> Result<f64, Failure> {
        let mut minimum_size = parameters.min_size;
        if parameters.adjust_min_size {
            minimum_size = std_min(
                minimum_size,
                MeshParameters::rel_min_size()
                    * AbscissaPoint::length_between_with_tolerance(
                        &self.adaptor as &dyn Adaptor3dCurve,
                        self.adaptor.first_parameter()?,
                        self.adaptor.last_parameter()?,
                        linear,
                    )?,
            );
        }
        Ok(minimum_size)
    }
    fn initialize_surface_range(&mut self) -> Result<(), Failure> {
        if !self.adaptor.transformed_curve().is_curve_on_surface() {
            return Ok(());
        }
        let curve = self.adaptor.transformed_curve().curve_on_surface()?;
        let surface = curve.get_surface();
        let du = surface.u_resolution(precision::confusion())?;
        let dv = surface.v_resolution(precision::confusion())?;
        self.face_range_u = [
            surface.first_u_parameter()? - du,
            surface.last_u_parameter()? + du,
        ];
        self.face_range_v = [
            surface.first_v_parameter()? - dv,
            surface.last_v_parameter()? + dv,
        ];
        Ok(())
    }
    fn add_internal_vertices(&mut self) -> Result<(), Failure> {
        for shape in Explorer::new(&self.edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if shape.orientation() != Orientation::Internal {
                continue;
            }
            let vertex = topods::vertex(&shape)?;
            self.distribution.add_point(
                BRepTool::pnt(&vertex)?,
                BRepTool::parameter(&vertex, &self.edge)?,
                true,
            )?;
        }
        Ok(())
    }
}

fn validate_minimum_size(minimum: f64) -> Result<(), Failure> {
    if minimum <= 0.0 {
        return Err(Failure::with_message(
            "Standard_Failure",
            &format!(
                "MinSize {}; expected initialized positive mesh minimum size",
                minimum
            ),
        ));
    }
    Ok(())
}
