use super::{ShapeFixEdge, seam::second_seam_pcurve};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisSurface};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkmath::toploc::Location;

type ProjectedPCurve = (Handle<dyn Geom2dCurve>, (f64, f64));

struct PCurveRepair<'a> {
    edge: &'a Edge,
    surface: &'a Handle<dyn GeomSurface>,
    location: &'a Location,
    analysis: &'a Handle<ShapeAnalysisSurface>,
    seam: bool,
    precision: f64,
}

impl ShapeFixEdge {
    /// Project missing p-curves on a face; `tool.fix_add_pcurve(&edge, &face, false, 0.0)?`.
    pub fn fix_add_pcurve(
        &mut self,
        edge: &Edge,
        face: &Face,
        seam: bool,
        precision: f64,
    ) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.fix_add_pcurve_on_surface(edge, &surface, &location, seam, precision)
    }

    /// Project on a located surface; `tool.fix_add_pcurve_on_surface(&edge, &surface, &location, false, 0.0)?`.
    pub fn fix_add_pcurve_on_surface(
        &mut self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        seam: bool,
        precision: f64,
    ) -> Result<bool, Failure> {
        let located = if location.is_identity() {
            surface.clone()
        } else {
            surface.transformed(location.transformation())?.cast()
        };
        let analysis = Handle::new(ShapeAnalysisSurface::new(&located)?);
        self.fix_add_pcurve_with_analysis(edge, surface, location, seam, &analysis, precision)
    }

    /// Reuse a caller's surface analysis; `tool.fix_add_pcurve_on_face_with_analysis(&edge, &face, false, &analysis, 0.0)?`.
    pub fn fix_add_pcurve_on_face_with_analysis(
        &mut self,
        edge: &Edge,
        face: &Face,
        seam: bool,
        analysis: &Handle<ShapeAnalysisSurface>,
        precision: f64,
    ) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.fix_add_pcurve_with_analysis(edge, &surface, &location, seam, analysis, precision)
    }

    /// Use explicit placement and surface analysis;
    /// `tool.fix_add_pcurve_with_analysis(&edge, &surface, &location, false, &analysis, 0.0)?`.
    pub fn fix_add_pcurve_with_analysis(
        &mut self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        seam: bool,
        analysis: &Handle<ShapeAnalysisSurface>,
        precision: f64,
    ) -> Result<bool, Failure> {
        self.status = 0;
        let repair = PCurveRepair {
            edge,
            surface,
            location,
            analysis,
            seam,
            precision,
        };
        if repair.already_present()? || !surface.cast::<GeomPlane>().is_null() {
            return Ok(false);
        }
        match self.project_missing_pcurve(&repair) {
            Ok(false) => return Ok(false),
            Err(_) => self.add_status(ShapeExtendStatus::Fail2),
            Ok(true) => {}
        }
        self.add_status(ShapeExtendStatus::Done1);
        Ok(true)
    }

    fn project_missing_pcurve(&mut self, repair: &PCurveRepair<'_>) -> Result<bool, Failure> {
        let precision = if repair.precision > 0.0 {
            repair.precision
        } else {
            BRepTool::edge_tolerance(repair.edge)?
        };
        let (curve, first, last) = BRepTool::curve(repair.edge)?;
        if curve.is_null() {
            self.add_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        let (pcurve, range) = self.project_or_reuse(repair, &curve, (first, last), precision)?;
        repair.store_pcurves(&pcurve, range)?;
        if self.projector.status(ShapeExtendStatus::Done3) {
            BRepBuilder::default().update_edge_curve(repair.edge, &curve, 0.0)?;
            BRepBuilder::default().range(
                repair.edge,
                curve.first_parameter(),
                curve.last_parameter(),
                true,
            )?;
        }
        Ok(true)
    }

    fn project_or_reuse(
        &mut self,
        repair: &PCurveRepair<'_>,
        curve: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        precision: f64,
    ) -> Result<ProjectedPCurve, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        if analyzer.has_pcurve_on_surface(repair.edge, repair.surface, repair.location)? {
            let (pcurve, first, last) =
                analyzer.pcurve_on_surface(repair.edge, repair.surface, repair.location, false)?;
            return Ok((pcurve, (first, last)));
        }
        let (first, last) = topexp::vertices(repair.edge, false);
        let endpoint_tolerances = (endpoint_tolerance(&first)?, endpoint_tolerance(&last)?);
        let mut pcurve = Handle::default();
        self.projector.init_analysis(repair.analysis, precision);
        self.projector
            .perform(curve, range, &mut pcurve, endpoint_tolerances)?;
        if self.projector.status(ShapeExtendStatus::Done4) {
            self.add_status(ShapeExtendStatus::Done2);
        }
        Ok((pcurve, range))
    }
}

impl PCurveRepair<'_> {
    fn already_present(&self) -> Result<bool, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        if self.seam {
            return analyzer.is_seam_on_surface(self.edge, self.surface, self.location);
        }
        analyzer.has_pcurve_on_surface(self.edge, self.surface, self.location)
    }

    fn store_pcurves(
        &self,
        pcurve: &Handle<dyn Geom2dCurve>,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        if !self.seam {
            return builder.update_edge_pcurve(
                self.edge,
                pcurve,
                self.surface,
                self.location,
                0.0,
                None,
            );
        }
        let second = second_seam_pcurve(pcurve, self.surface, self.analysis, self.precision)?;
        builder.update_edge_pcurves(
            self.edge,
            (pcurve, &second),
            self.surface,
            self.location,
            0.0,
            None,
        )?;
        builder.range_on_surface(self.edge, self.surface, self.location, range.0, range.1)
    }
}

fn endpoint_tolerance(vertex: &Vertex) -> Result<f64, Failure> {
    if vertex.is_null() {
        return Ok(-1.0);
    }
    BRepTool::vertex_tolerance(vertex)
}
